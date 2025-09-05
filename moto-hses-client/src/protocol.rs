//! Protocol communication for HSES client

use moto_hses_proto::alarm::ReadAlarmHistory;
use moto_hses_proto::{
    Alarm, Command, CoordinateSystemType, Position, ReadAlarmData, ReadCurrentPosition, ReadStatus,
    ReadStatusData1, ReadStatusData2, ReadVar, Status, StatusData1, StatusData2, StatusWrapper,
    VariableType, WriteVar,
};
use std::sync::atomic::Ordering;
use tokio::time::{sleep, timeout};

use crate::types::{ClientError, HsesClient};

impl HsesClient {
    // High-level API methods
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        let command = ReadVar::<T> {
            index,
            _phantom: std::marker::PhantomData,
        };
        let response = self.send_command_with_retry(command).await?;
        self.deserialize_response(&response)
    }

    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: VariableType,
    {
        let command = WriteVar::<T> { index, value };
        let response = self.send_command_with_retry(command).await?;
        self.deserialize_response(&response)
    }

    /// Read complete status information (both Data 1 and Data 2) efficiently
    /// Uses service=0x01 (Get_Attribute_All) with attribute=0 to get both data in one request
    pub async fn read_status(&self) -> Result<Status, ClientError> {
        let response = self.send_command_with_retry(ReadStatus).await?;
        let result: StatusWrapper = self.deserialize_response(&response)?;
        Ok(result.into())
    }

    /// Read status data 1 (basic status information)
    pub async fn read_status_data1(&self) -> Result<StatusData1, ClientError> {
        let response = self.send_command_with_retry(ReadStatusData1).await?;
        self.deserialize_response(&response)
    }

    /// Read status data 2 (additional status information)
    pub async fn read_status_data2(&self) -> Result<StatusData2, ClientError> {
        let response = self.send_command_with_retry(ReadStatusData2).await?;
        self.deserialize_response(&response)
    }

    pub async fn read_position(
        &self,
        control_group: u8,
        coord_system: CoordinateSystemType,
    ) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition {
            control_group,
            coordinate_system: coord_system,
        };
        let response = self.send_command_with_retry(command).await?;
        self.deserialize_response(&response)
    }

    pub async fn read_alarm_data(
        &self,
        instance: u16,
        attribute: u8,
    ) -> Result<Alarm, ClientError> {
        let command = ReadAlarmData::new(instance, attribute);
        self.read_alarm_attribute(command, attribute).await
    }

    pub async fn read_alarm_history(
        &self,
        instance: u16,
        attribute: u8,
    ) -> Result<Alarm, ClientError> {
        let command = ReadAlarmHistory::new(instance, attribute);
        self.read_alarm_attribute(command, attribute).await
    }

    // Common helper method for alarm attribute reading
    async fn read_alarm_attribute<C: Command>(
        &self,
        command: C,
        attribute: u8,
    ) -> Result<Alarm, ClientError>
    where
        C::Response: VariableType + Into<Alarm>,
    {
        if attribute == 0 {
            // Service = 0x01 (Get_Attribute_All) - Return all data
            let response = self.send_command_with_retry(command).await?;
            let deserialized: C::Response = self.deserialize_response(&response)?;
            Ok(deserialized.into())
        } else {
            // Service = 0x0E (Get_Attribute_Single) - Return only specified attribute
            let response = self.send_command_with_retry(command).await?;

            // Attribute-specific deserialization
            match attribute {
                1 => {
                    // Alarm code (4 bytes)
                    if response.len() >= 4 {
                        let code = u32::from_le_bytes([
                            response[0],
                            response[1],
                            response[2],
                            response[3],
                        ]);
                        Ok(Alarm::new(code, 0, 0, String::new(), String::new()))
                    } else {
                        Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                    }
                }
                2 => {
                    // Alarm data (4 bytes)
                    if response.len() >= 4 {
                        let data = u32::from_le_bytes([
                            response[0],
                            response[1],
                            response[2],
                            response[3],
                        ]);
                        Ok(Alarm::new(0, data, 0, String::new(), String::new()))
                    } else {
                        Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                    }
                }
                3 => {
                    // Alarm type (4 bytes)
                    if response.len() >= 4 {
                        let alarm_type = u32::from_le_bytes([
                            response[0],
                            response[1],
                            response[2],
                            response[3],
                        ]);
                        Ok(Alarm::new(0, 0, alarm_type, String::new(), String::new()))
                    } else {
                        Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                    }
                }
                4 => {
                    // Alarm time (16 bytes)
                    if response.len() >= 16 {
                        let time_end = response.iter().position(|&b| b == 0).unwrap_or(16);
                        let time = String::from_utf8_lossy(&response[..time_end]).to_string();
                        Ok(Alarm::new(0, 0, 0, time, String::new()))
                    } else {
                        Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                    }
                }
                5 => {
                    // Alarm name (32 bytes)
                    if response.len() >= 32 {
                        let name_end = response.iter().position(|&b| b == 0).unwrap_or(32);
                        let name = String::from_utf8_lossy(&response[..name_end]).to_string();
                        Ok(Alarm::new(0, 0, 0, String::new(), name))
                    } else {
                        Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                    }
                }
                _ => {
                    // Invalid attribute
                    Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                }
            }
        }
    }

    pub async fn read_io(&self, _io_type: u8, _index: u8) -> Result<bool, ClientError> {
        // TODO: Implement I/O reading command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError(
            "I/O reading not yet implemented".to_string(),
        ))
    }

    pub async fn write_io(
        &self,
        _io_type: u8,
        _index: u8,
        _value: bool,
    ) -> Result<(), ClientError> {
        // TODO: Implement I/O writing command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError(
            "I/O writing not yet implemented".to_string(),
        ))
    }

    pub async fn execute_job(&self, _job_number: u8) -> Result<(), ClientError> {
        // TODO: Implement I/O reading command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError(
            "Job execution not yet implemented".to_string(),
        ))
    }

    pub async fn stop_job(&self) -> Result<(), ClientError> {
        // TODO: Implement job stop command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError(
            "Job stop not yet implemented".to_string(),
        ))
    }

    // Command sending with retry logic (returns raw bytes)
    async fn send_command_with_retry<C: Command>(
        &self,
        command: C,
    ) -> Result<Vec<u8>, ClientError> {
        let mut last_error = None;
        let mut attempts = 0;

        while attempts < self.config.retry_count {
            match self.send_command_once(&command).await {
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

    // Single command sending (no retry, returns raw bytes)
    async fn send_command_once<C: Command>(&self, command: &C) -> Result<Vec<u8>, ClientError> {
        let request_id = self.inner.request_id.fetch_add(1, Ordering::Relaxed);
        let payload = command.serialize()?;

        // Create and send message
        let message = self.create_message(
            C::command_id(),
            request_id,
            payload,
            command.instance(),
            command.attribute(),
        )?;
        eprintln!(
            "Sending message to {}: {} bytes",
            self.inner.remote_addr,
            message.len()
        );
        self.inner
            .socket
            .send_to(&message, self.inner.remote_addr)
            .await?;

        // Wait for response
        let response = self.wait_for_response(request_id).await?;

        // Return raw response payload
        Ok(response)
    }

    fn create_message(
        &self,
        command: u16,
        request_id: u8,
        payload: Vec<u8>,
        instance: u16,
        attribute: u8,
    ) -> Result<Vec<u8>, ClientError> {
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

        // Instance
        message.extend_from_slice(&instance.to_le_bytes());

        // Attribute
        message.push(attribute);

        // Service depends on command type and attribute
        let service = self.get_service_for_command(command, attribute);
        message.push(service);

        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        message.extend(payload);

        Ok(message)
    }

    async fn wait_for_response(&self, request_id: u8) -> Result<Vec<u8>, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];

        loop {
            let (len, _addr) = timeout(
                self.config.timeout,
                self.inner.socket.recv_from(&mut buffer),
            )
            .await
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
            let payload = response_data[32..32 + payload_size].to_vec();

            return Ok(payload);
        }
    }

    fn deserialize_response<T>(&self, data: &[u8]) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        T::deserialize(data).map_err(ClientError::from)
    }

    fn get_service_for_command(&self, command: u16, attribute: u8) -> u8 {
        match command {
            0x70 | 0x71 | 0x72 | 0x75 => {
                // Commands that support both Get_Attribute_All and Get_Attribute_Single
                if attribute == 0 {
                    0x01 // Get_Attribute_All
                } else {
                    0x0e // Get_Attribute_Single
                }
            }
            0x7A..=0x81 => {
                // Variable read/write commands - attribute is always 1
                // For now, assume read operations (0x0E)
                // TODO: Add support for write operations (0x10, 0x02)
                0x0e // Get_Attribute_Single
            }
            _ => {
                // Default to Get_Attribute_Single for unknown commands
                0x0e
            }
        }
    }
}
