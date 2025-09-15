//! Protocol communication for HSES client

use moto_hses_proto::alarm::{AlarmReset, ReadAlarmHistory};
use moto_hses_proto::{
    Alarm, Command, CoordinateSystemType, DeleteFile, Division, ExecutingJobInfo, HoldServoControl,
    Position, ReadAlarmData, ReadCurrentPosition, ReadExecutingJobInfo, ReadFileList, ReadIo,
    ReadStatus, ReadStatusData1, ReadStatusData2, ReadVar, ReceiveFile, SendFile, Status,
    StatusData1, StatusData2, VariableType, WriteIo, WriteVar,
};
use moto_hses_proto::{parse_file_content, parse_file_list};
use std::sync::atomic::Ordering;
use tokio::time::{sleep, timeout};

use crate::types::{ClientError, HsesClient};

/// Parameters for creating HSES messages
#[derive(Debug, Clone)]
struct MessageParams {
    command: u16,
    request_id: u8,
    payload: Vec<u8>,
    instance: u16,
    attribute: u8,
    service: u8,
    division: Division,
}

impl HsesClient {
    // High-level API methods
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        let command = ReadVar::<T> { index, _phantom: std::marker::PhantomData };
        let response = self.send_command_with_retry(command, Division::Robot).await?;
        self.deserialize_response(&response)
    }

    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: VariableType,
    {
        let command = WriteVar::<T> { index, value };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read complete status information (both Data 1 and Data 2) efficiently
    /// Uses service=0x01 (Get_Attribute_All) with attribute=0 to get both data in one request
    pub async fn read_status(&self) -> Result<Status, ClientError> {
        let response = self.send_command_with_retry(ReadStatus, Division::Robot).await?;
        self.deserialize_response(&response)
    }

    /// Read status data 1 (basic status information)
    pub async fn read_status_data1(&self) -> Result<StatusData1, ClientError> {
        let response = self.send_command_with_retry(ReadStatusData1, Division::Robot).await?;
        self.deserialize_response(&response)
    }

    /// Read status data 2 (additional status information)
    pub async fn read_status_data2(&self) -> Result<StatusData2, ClientError> {
        let response = self.send_command_with_retry(ReadStatusData2, Division::Robot).await?;
        self.deserialize_response(&response)
    }

    pub async fn read_position(
        &self,
        control_group: u8,
        coord_system: CoordinateSystemType,
    ) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition { control_group, coordinate_system: coord_system };
        let response = self.send_command_with_retry(command, Division::Robot).await?;
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

    /// Reset alarm (0x82 command with instance 1)
    ///
    /// This command resets the current alarm state.
    pub async fn reset_alarm(&self) -> Result<(), ClientError> {
        let command = AlarmReset::reset();
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Cancel error (0x82 command with instance 2)
    ///
    /// This command cancels the current error state.
    pub async fn cancel_error(&self) -> Result<(), ClientError> {
        let command = AlarmReset::cancel();
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Set HOLD state (0x83 command with instance 1)
    ///
    /// # Arguments
    /// * `enabled` - true for HOLD ON, false for HOLD OFF
    pub async fn set_hold(&self, enabled: bool) -> Result<(), ClientError> {
        let command =
            if enabled { HoldServoControl::hold_on() } else { HoldServoControl::hold_off() };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Set Servo power state (0x83 command with instance 2)
    ///
    /// # Arguments
    /// * `enabled` - true for Servo ON, false for Servo OFF
    pub async fn set_servo(&self, enabled: bool) -> Result<(), ClientError> {
        let command =
            if enabled { HoldServoControl::servo_on() } else { HoldServoControl::servo_off() };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Set HLOCK state (0x83 command with instance 3)
    ///
    /// HLOCK interlocks the Programming Pendant and I/O operation system signals.
    /// Only emergency stop and limited input signals are available while HLOCK is ON.
    ///
    /// # Arguments
    /// * `enabled` - true for HLOCK ON, false for HLOCK OFF
    pub async fn set_hlock(&self, enabled: bool) -> Result<(), ClientError> {
        let command =
            if enabled { HoldServoControl::hlock_on() } else { HoldServoControl::hlock_off() };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read executing job information
    ///
    /// # Arguments
    /// * `task_type` - Task type (1: Master task, 2-6: Sub tasks)
    /// * `attribute` - Information to read (0: All, 1: Job name, 2: Line number, 3: Step number, 4: Speed override value)
    pub async fn read_executing_job_info(
        &self,
        task_type: u16,
        attribute: u8,
    ) -> Result<ExecutingJobInfo, ClientError> {
        let command = ReadExecutingJobInfo::new(task_type, attribute);
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        // Use attribute-specific deserialization for single attributes
        if attribute > 0 {
            ExecutingJobInfo::deserialize_attribute(&response, attribute).map_err(ClientError::from)
        } else {
            // Use standard deserialization for complete data
            self.deserialize_response(&response)
        }
    }

    /// Read complete executing job information (all attributes)
    ///
    /// # Arguments
    /// * `task_type` - Task type (1: Master task, 2-6: Sub tasks)
    pub async fn read_executing_job_info_complete(
        &self,
        task_type: u16,
    ) -> Result<ExecutingJobInfo, ClientError> {
        self.read_executing_job_info(task_type, 0).await
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
            let response = self.send_command_with_retry(command, Division::Robot).await?;
            let deserialized: C::Response = self.deserialize_response(&response)?;
            Ok(deserialized.into())
        } else {
            // Service = 0x0E (Get_Attribute_Single) - Return only specified attribute
            let response = self.send_command_with_retry(command, Division::Robot).await?;

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

    pub async fn read_io(&self, io_number: u16) -> Result<bool, ClientError> {
        let command = ReadIo { io_number };
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        if response.len() >= 4 {
            let value = i32::from_le_bytes([response[0], response[1], response[2], response[3]]);
            Ok(value != 0)
        } else {
            Err(ClientError::ProtocolError(moto_hses_proto::ProtocolError::Deserialization(
                "Invalid response length for I/O read".to_string(),
            )))
        }
    }

    pub async fn write_io(&self, io_number: u16, value: bool) -> Result<(), ClientError> {
        let command = WriteIo { io_number, value };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    pub async fn read_register(&self, register_number: u16) -> Result<i16, ClientError> {
        use moto_hses_proto::types::ReadRegister;
        let command = ReadRegister { register_number };
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        if response.len() >= 2 {
            // Register data is 2 bytes (i16) + 2 bytes reserved = 4 bytes total
            // We only use the first 2 bytes for the actual register value
            let value = i16::from_le_bytes([response[0], response[1]]);
            Ok(value)
        } else {
            Err(ClientError::ProtocolError(moto_hses_proto::ProtocolError::Deserialization(
                "Invalid response length for register read".to_string(),
            )))
        }
    }

    pub async fn write_register(
        &self,
        register_number: u16,
        value: i16,
    ) -> Result<(), ClientError> {
        use moto_hses_proto::types::WriteRegister;
        let command = WriteRegister { register_number, value };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
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

    // File operations (Division = 0x02)

    /// Get file list from controller
    ///
    /// Returns a list of filenames available on the controller.
    pub async fn read_file_list(&self) -> Result<Vec<String>, ClientError> {
        let command = ReadFileList;
        let response = self.send_command_with_retry(command, Division::File).await?;
        parse_file_list(&response).map_err(ClientError::from)
    }

    /// Send file to controller
    ///
    /// # Arguments
    /// * `filename` - Name of the file to send
    /// * `content` - File content as bytes
    pub async fn send_file(&self, filename: &str, content: &[u8]) -> Result<(), ClientError> {
        let command = SendFile::new(filename.to_string(), content.to_vec());
        let _response = self.send_command_with_retry(command, Division::File).await?;
        Ok(())
    }

    /// Receive file from controller
    ///
    /// # Arguments
    /// * `filename` - Name of the file to receive
    ///
    /// Returns the file content as bytes.
    pub async fn receive_file(&self, filename: &str) -> Result<Vec<u8>, ClientError> {
        let command = ReceiveFile::new(filename.to_string());
        let response = self.send_command_with_retry(command, Division::File).await?;
        parse_file_content(&response).map_err(ClientError::from)
    }

    /// Delete file from controller
    ///
    /// # Arguments
    /// * `filename` - Name of the file to delete
    pub async fn delete_file(&self, filename: &str) -> Result<(), ClientError> {
        let command = DeleteFile::new(filename.to_string());
        let _response = self.send_command_with_retry(command, Division::File).await?;
        Ok(())
    }

    // Command sending with retry logic (returns raw bytes)
    async fn send_command_with_retry<C: Command>(
        &self,
        command: C,
        division: Division,
    ) -> Result<Vec<u8>, ClientError> {
        let mut last_error = None;
        let mut attempts = 0;

        while attempts < self.config.retry_count {
            match self.send_command_once(&command, division).await {
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
    async fn send_command_once<C: Command>(
        &self,
        command: &C,
        division: Division,
    ) -> Result<Vec<u8>, ClientError> {
        let request_id = self.inner.request_id.fetch_add(1, Ordering::Relaxed);
        let payload = command.serialize()?;

        // Create and send message
        let params = MessageParams {
            command: C::command_id(),
            request_id,
            payload,
            instance: command.instance(),
            attribute: command.attribute(),
            service: command.service(),
            division,
        };
        let message = self.create_message(params)?;
        eprintln!("Sending message to {}: {} bytes", self.inner.remote_addr, message.len());
        self.inner.socket.send_to(&message, self.inner.remote_addr).await?;

        // Wait for response
        let response = self.wait_for_response(request_id).await?;

        // Return raw response payload
        Ok(response)
    }

    fn create_message(&self, params: MessageParams) -> Result<Vec<u8>, ClientError> {
        let mut message = Vec::new();

        // Magic bytes "YERC"
        message.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        message.extend_from_slice(&0x20u16.to_le_bytes());

        // Payload size
        message.extend_from_slice(&(params.payload.len() as u16).to_le_bytes());

        // Reserved magic constant
        message.push(0x03);

        // Division (Robot or File)
        message.push(params.division as u8);

        // ACK (Request)
        message.push(0x00);

        // Request ID
        message.push(params.request_id);

        // Block number (0 for requests)
        message.extend_from_slice(&0u32.to_le_bytes());

        // Reserved (8 bytes of '9')
        message.extend_from_slice(b"99999999");

        // Command
        message.extend_from_slice(&params.command.to_le_bytes());

        // Instance
        message.extend_from_slice(&params.instance.to_le_bytes());

        // Attribute
        message.push(params.attribute);

        // Service
        message.push(params.service);

        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        message.extend(params.payload);

        Ok(message)
    }

    async fn wait_for_response(&self, request_id: u8) -> Result<Vec<u8>, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];

        loop {
            let (len, _addr) =
                timeout(self.config.timeout, self.inner.socket.recv_from(&mut buffer))
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

            // Check status (byte 25 in response sub-header)
            if response_data.len() >= 26 {
                let status = response_data[25];
                if status != 0x00 {
                    // Non-zero status indicates an error
                    return Err(ClientError::ProtocolError(
                        moto_hses_proto::ProtocolError::InvalidMessage(format!(
                            "Server returned error status: 0x{:02x}",
                            status
                        )),
                    ));
                }
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
}
