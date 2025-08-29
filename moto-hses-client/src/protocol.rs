//! Protocol communication for HSES client

use tokio::time::{timeout, sleep};
use std::sync::atomic::Ordering;
use moto_hses_proto::{
    Command, VariableType, Position, Status, StatusWrapper, ReadStatus, ReadCurrentPosition,
    ReadVar, WriteVar, CoordinateSystemType
};

use crate::types::{HsesClient, ClientError};

impl HsesClient {
    // High-level API methods
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        let command = ReadVar::<T> { index, _phantom: std::marker::PhantomData };
        self.send_command_with_retry(command).await
    }

    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: VariableType,
    {
        let command = WriteVar::<T> { index, value };
        self.send_command_with_retry(command).await
    }

    pub async fn read_status(&self) -> Result<Status, ClientError> {
        let result: StatusWrapper = self.send_command_with_retry(ReadStatus).await?;
        Ok(result.into())
    }

    pub async fn read_position(&self, control_group: u8, coord_system: CoordinateSystemType) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition { control_group, coordinate_system: coord_system };
        self.send_command_with_retry(command).await
    }

    pub async fn read_io(&self, _io_type: u8, _index: u8) -> Result<bool, ClientError> {
        // TODO: Implement I/O reading command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError("I/O reading not yet implemented".to_string()))
    }

    pub async fn write_io(&self, _io_type: u8, _index: u8, _value: bool) -> Result<(), ClientError> {
        // TODO: Implement I/O writing command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError("I/O writing not yet implemented".to_string()))
    }

    pub async fn execute_job(&self, _job_number: u8) -> Result<(), ClientError> {
        // TODO: Implement I/O reading command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError("Job execution not yet implemented".to_string()))
    }

    pub async fn stop_job(&self) -> Result<(), ClientError> {
        // TODO: Implement job stop command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError("Job stop not yet implemented".to_string()))
    }

    // Generic command sending with retry logic
    async fn send_command_with_retry<C: Command>(&self, command: C) -> Result<C::Response, ClientError> 
    where
        C::Response: VariableType,
    {
        let mut last_error = None;
        let mut attempts = 0;

        while attempts < self.config.retry_count {
            match self.send_command(&command).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts < self.config.retry_count {
                        sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ClientError::SystemError("Unknown error".to_string())))
    }

    // Generic command sending
    async fn send_command<C: Command>(&self, command: &C) -> Result<C::Response, ClientError> 
    where
        C::Response: VariableType,
    {
        let request_id = self.inner.request_id.fetch_add(1, Ordering::Relaxed);
        let payload = command.serialize()?;

        // Create and send message
        let message = self.create_message(C::command_id(), request_id, payload)?;
        eprintln!("Sending message to {}: {} bytes", self.inner.remote_addr, message.len());
        self.inner.socket.send_to(&message, self.inner.remote_addr).await?;

        // Wait for response
        let response = self.wait_for_response(request_id).await?;

        // Deserialize response
        self.deserialize_response::<C::Response>(&response)
    }

    fn create_message(&self, command: u16, request_id: u8, payload: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        let mut message = Vec::new();

        // Magic bytes "YERC"
        message.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        message.extend_from_slice(&0x20u16.to_le_bytes());

        // Payload size
        message.extend_from_slice(&(payload.len() as u16).to_le_bytes());

        // Reserved magic constant
        message.push(0x03);

        // Division (Robot)
        message.push(0x01);

        // ACK (Request)
        message.push(0x00);

        // Request ID
        message.push(request_id);

        // Block number (0 for requests)
        message.extend_from_slice(&0u32.to_le_bytes());

        // Reserved (8 bytes of '9')
        message.extend_from_slice(b"99999999");

        // Command
        message.extend_from_slice(&command.to_le_bytes());

        // Instance (0 for most commands)
        message.extend_from_slice(&0u16.to_le_bytes());

        // Attribute (1 for most commands)
        message.push(1);

        // Service (Get_Attribute_Single for reads, Set_Attribute_Single for writes)
        message.push(0x0e);

        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        message.extend(payload);

        Ok(message)
    }

    async fn wait_for_response(&self, request_id: u8) -> Result<Vec<u8>, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];

        loop {
            let (len, _addr) = timeout(self.config.timeout, self.inner.socket.recv_from(&mut buffer)).await
                .map_err(|_| ClientError::TimeoutError("Response timeout".to_string()))??;

            let response_data = &buffer[..len];
            
            // Debug: Log received data
            eprintln!("Received response: {} bytes", len);
            if len >= 4 {
                eprintln!("Magic bytes: {:?}", &response_data[0..4]);
            }
            if len >= 11 {
                eprintln!("Request ID: 0x{:02x}", response_data[11]);
            }
            if len >= 10 {
                eprintln!("ACK: 0x{:02x}", response_data[10]);
            }

            // Parse response header
            if response_data.len() < 32 {
                continue;
            }

            // Verify magic bytes "YERC"
            if &response_data[0..4] != b"YERC" {
                continue;
            }

            // Check request ID (byte 11)
            let response_request_id = response_data[11];
            if response_request_id != request_id {
                continue;
            }

            // Check ACK (byte 10, should be 0x01 for response)
            let ack = response_data[10];
            if ack != 0x01 {
                continue;
            }

            // Extract payload size (bytes 6-7)
            let payload_size = u16::from_le_bytes([response_data[6], response_data[7]]) as usize;
            
            // Ensure we have enough data
            if response_data.len() < 32 + payload_size {
                continue;
            }

            // Extract payload (starting from byte 32)
            let payload = response_data[32..32+payload_size].to_vec();

            return Ok(payload);
        }
    }

    fn deserialize_response<T>(&self, data: &[u8]) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        T::deserialize(data).map_err(ClientError::from)
    }
}
