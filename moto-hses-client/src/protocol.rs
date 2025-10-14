//! Protocol communication for HSES client

use moto_hses_proto::{
    Alarm, AlarmAttribute, AlarmReset, Command, DeleteFile, Division, ExecutingJobInfo,
    HoldServoControl, HsesPayload, Position, ReadAlarmData, ReadAlarmHistory, ReadCurrentPosition,
    ReadExecutingJobInfo, ReadFileList, ReadIo, ReadStatus, ReadStatusData1, ReadStatusData2,
    ReadVar, ReceiveFile, SendFile, Status, StatusData1, StatusData2, VariableCommandId, WriteIo,
    WriteVar,
    commands::{
        JobSelectCommand, JobSelectType, JobStartCommand, ReadMultipleByteVariables,
        ReadMultipleIo, WriteMultipleByteVariables, WriteMultipleIo, parse_file_content,
        parse_file_list,
    },
};
use std::fmt::Write;
use std::sync::atomic::Ordering;
use tokio::time::{sleep, timeout};

use crate::types::{ClientError, HsesClient};

/// Sequence control parameters
#[derive(Debug, Clone)]
struct SequenceParams {
    request_id: u8,
    block_number: u32,
    ack: u8,
}

/// Request-specific parameters
#[derive(Debug, Clone)]
struct RequestParams {
    division: Division,
    command: u16,
    instance: u16,
    attribute: u8,
    service: u8,
}

impl HsesClient {
    // High-level API methods
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: HsesPayload + VariableCommandId,
    {
        let command = ReadVar::<T> { index, _phantom: std::marker::PhantomData };
        let response = self.send_command_with_retry(command, Division::Robot).await?;
        T::deserialize(&response, self.config.text_encoding).map_err(ClientError::from)
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: HsesPayload + VariableCommandId,
    {
        let command = WriteVar::<T> { index, value };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read complete status information (both Data 1 and Data 2) efficiently
    /// Uses service=0x01 (`Get_Attribute_All`) with attribute=0 to get both data in one request
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_status(&self) -> Result<Status, ClientError> {
        let response = self.send_command_with_retry(ReadStatus, Division::Robot).await?;
        Status::deserialize(&response, self.config.text_encoding).map_err(ClientError::from)
    }

    /// Read status data 1 (basic status information)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_status_data1(&self) -> Result<StatusData1, ClientError> {
        let response = self.send_command_with_retry(ReadStatusData1, Division::Robot).await?;
        StatusData1::deserialize(&response, self.config.text_encoding).map_err(ClientError::from)
    }

    /// Read status data 2 (additional status information)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_status_data2(&self) -> Result<StatusData2, ClientError> {
        let response = self.send_command_with_retry(ReadStatusData2, Division::Robot).await?;
        StatusData2::deserialize(&response, self.config.text_encoding).map_err(ClientError::from)
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_position(&self, control_group: u8) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition { control_group };
        let response = self.send_command_with_retry(command, Division::Robot).await?;
        Position::deserialize(&response, self.config.text_encoding).map_err(ClientError::from)
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_alarm_data(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError> {
        let command = ReadAlarmData::new(instance, attribute);
        self.read_alarm_attribute(command, attribute).await
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_alarm_history(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError> {
        let command = ReadAlarmHistory::new(instance, attribute);
        self.read_alarm_attribute(command, attribute).await
    }

    /// Reset alarm (0x82 command with instance 1)
    ///
    /// This command resets the current alarm state.
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn reset_alarm(&self) -> Result<(), ClientError> {
        let command = AlarmReset::reset();
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Cancel error (0x82 command with instance 2)
    ///
    /// This command cancels the current error state.
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn cancel_error(&self) -> Result<(), ClientError> {
        let command = AlarmReset::cancel();
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Set HOLD state (0x83 command with instance 1)
    ///
    /// # Arguments
    /// * `enabled` - true for HOLD ON, false for HOLD OFF
    /// # Errors
    ///
    /// Returns an error if communication fails
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
    /// # Errors
    ///
    /// Returns an error if communication fails
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
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn set_hlock(&self, enabled: bool) -> Result<(), ClientError> {
        let command =
            if enabled { HoldServoControl::hlock_on() } else { HoldServoControl::hlock_off() };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Set cycle mode (0x84 command)
    ///
    /// # Arguments
    /// * `mode` - Cycle mode to set (Step, `OneCycle`, or Continuous)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn set_cycle_mode(
        &self,
        mode: moto_hses_proto::CycleMode,
    ) -> Result<(), ClientError> {
        let command = moto_hses_proto::CycleModeSwitchingCommand::new(mode);
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Start job execution (0x86 command)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn start_job(&self) -> Result<(), ClientError> {
        let command = JobStartCommand::new();
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Select job for execution (0x87 command)
    ///
    /// # Arguments
    ///
    /// * `select_type` - Type of job to select
    /// * `job_name` - Name of the job to select (max 32 bytes when encoded)
    /// * `line_number` - Starting line number (0 to 9999)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn select_job(
        &self,
        select_type: JobSelectType,
        job_name: impl Into<String>,
        line_number: u32,
    ) -> Result<(), ClientError> {
        let job_name = job_name.into();

        // Validate job name byte length (max 32 bytes when encoded)
        let encoded_bytes =
            moto_hses_proto::encoding_utils::encode_string(&job_name, self.config.text_encoding);
        if encoded_bytes.len() > 32 {
            return Err(ClientError::SystemError(
                "Job name exceeds 32 bytes when encoded".to_string(),
            ));
        }

        // Validate line number (0 to 9999)
        if line_number > 9999 {
            return Err(ClientError::SystemError("Line number must be 0-9999".to_string()));
        }

        let command =
            JobSelectCommand::new(select_type, job_name, line_number, self.config.text_encoding);
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read executing job information
    ///
    /// # Arguments
    /// * `task_type` - Task type (1: Master task, 2-6: Sub tasks)
    /// * `attribute` - Information to read (0: All, 1: Job name, 2: Line number, 3: Step number, 4: Speed override value)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_executing_job_info(
        &self,
        task_type: u16,
        attribute: u8,
    ) -> Result<ExecutingJobInfo, ClientError> {
        let command = ReadExecutingJobInfo::new(task_type, attribute);
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        // Use attribute-specific deserialization for single attributes
        if attribute > 0 {
            ExecutingJobInfo::deserialize_attribute(&response, attribute, self.config.text_encoding)
                .map_err(ClientError::from)
        } else {
            // Use standard deserialization for complete data
            ExecutingJobInfo::deserialize(&response, self.config.text_encoding)
                .map_err(ClientError::from)
        }
    }

    /// Read complete executing job information (all attributes)
    ///
    /// # Arguments
    /// * `task_type` - Task type (1: Master task, 2-6: Sub tasks)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_executing_job_info_complete(
        &self,
        task_type: u16,
    ) -> Result<ExecutingJobInfo, ClientError> {
        self.read_executing_job_info(task_type, 0).await
    }

    // Common helper method for alarm attribute reading
    async fn read_alarm_attribute<C: Command + Send + Sync>(
        &self,
        command: C,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError>
    where
        C::Response: HsesPayload + Into<Alarm>,
    {
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        if attribute == AlarmAttribute::All {
            // Service = 0x01 (Get_Attribute_All) - Return all data
            let deserialized: C::Response =
                C::Response::deserialize(&response, self.config.text_encoding)
                    .map_err(ClientError::from)?;
            Ok(deserialized.into())
        } else {
            // Service = 0x0E (Get_Attribute_Single) - Return only specified attribute

            // Attribute-specific deserialization
            match attribute {
                AlarmAttribute::Code => {
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
                AlarmAttribute::Data => {
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
                AlarmAttribute::Type => {
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
                AlarmAttribute::Time => {
                    // Alarm time (16 bytes)
                    if response.len() >= 16 {
                        let time_end = response.iter().position(|&b| b == 0).unwrap_or(16);
                        let time_bytes = &response[..time_end];
                        let time = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                            time_bytes,
                            self.config.text_encoding,
                        );
                        Ok(Alarm::new(0, 0, 0, time, String::new()))
                    } else {
                        Ok(Alarm::new(0, 0, 0, String::new(), String::new()))
                    }
                }
                AlarmAttribute::Name => {
                    // Alarm name (32 bytes)
                    if response.len() >= 32 {
                        let name_end = response.iter().position(|&b| b == 0).unwrap_or(32);
                        let name_bytes = &response[..name_end];
                        let name = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                            name_bytes,
                            self.config.text_encoding,
                        );
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

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_io(&self, io_number: u16) -> Result<u8, ClientError> {
        let command = ReadIo { io_number };
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        if response.len() == 1 {
            Ok(response[0])
        } else {
            Err(ClientError::ProtocolError(moto_hses_proto::ProtocolError::Deserialization(
                format!(
                    "Invalid response length for I/O read: expected 1 byte, got {}",
                    response.len()
                ),
            )))
        }
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_io(&self, io_number: u16, value: u8) -> Result<(), ClientError> {
        let command = WriteIo { io_number, value };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read multiple I/O data (0x300 command)
    ///
    /// # Arguments
    ///
    /// * `start_io_number` - Starting I/O number
    /// * `count` - Number of I/O data to read (max 474, must be multiple of 2)
    ///
    /// # Returns
    ///
    /// Vector of I/O data bytes, where each byte contains 8 I/O states
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_io(
        &self,
        start_io_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        let command = ReadMultipleIo::new(start_io_number, count)?;
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        // Response format: Byte0-3 = count, Byte4-N = I/O data
        if response.len() < 4 {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Response too short: expected at least 4 bytes, got {}",
                    response.len()
                )),
            ));
        }

        let response_count =
            u32::from_le_bytes([response[0], response[1], response[2], response[3]]);

        if response_count != count {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Count mismatch: expected {count}, got {response_count}"
                )),
            ));
        }

        // Convert count to usize and validate response length
        let response_count_usize = response_count as usize;
        let expected_length = 4 + response_count_usize;

        if response.len() != expected_length {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Invalid response length: expected {} bytes, got {}",
                    expected_length,
                    response.len()
                )),
            ));
        }

        Ok(response[4..4 + response_count_usize].to_vec())
    }

    /// Write multiple I/O data (0x300 command)
    ///
    /// Note: Only network input signals are writable
    ///
    /// # Arguments
    ///
    /// * `start_io_number` - Starting I/O number (must be network input: 2701-2956)
    /// * `io_data` - I/O data bytes to write (max 474, must be multiple of 2)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_io(
        &self,
        start_io_number: u16,
        io_data: Vec<u8>,
    ) -> Result<(), ClientError> {
        let command = WriteMultipleIo::new(start_io_number, io_data)?;
        self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_register(&self, register_number: u16) -> Result<i16, ClientError> {
        use moto_hses_proto::ReadRegister;
        let command = ReadRegister { register_number };
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        if response.len() >= 2 {
            // Register data is 2 bytes (i16) + 2 bytes reserved = 4 bytes total
            // We only use the first 2 bytes for the actual register value
            let value = i16::from_le_bytes([response[0], response[1]]);
            Ok(value)
        } else {
            Err(ClientError::ProtocolError(moto_hses_proto::ProtocolError::Deserialization(
                format!(
                    "Invalid response length for register read: expected 2 bytes, got {}",
                    response.len()
                ),
            )))
        }
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_register(
        &self,
        register_number: u16,
        value: i16,
    ) -> Result<(), ClientError> {
        use moto_hses_proto::WriteRegister;
        let command = WriteRegister { register_number, value };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read multiple registers (0x301 command)
    ///
    /// # Arguments
    ///
    /// * `start_register_number` - Starting register number (0-999)
    /// * `count` - Number of registers to read (max 237)
    ///
    /// # Returns
    ///
    /// Vector of register values (i16)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_registers(
        &self,
        start_register_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        use moto_hses_proto::commands::ReadMultipleRegisters;
        let command = ReadMultipleRegisters::new(start_register_number, count)?;
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        // Response format: Byte0-3 = count, Byte4-N = register data (2 bytes each)
        if response.len() < 4 {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Response too short: expected at least 4 bytes, got {}",
                    response.len()
                )),
            ));
        }

        let response_count =
            u32::from_le_bytes([response[0], response[1], response[2], response[3]]);

        if response_count != count {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Count mismatch: expected {count}, got {response_count}"
                )),
            ));
        }

        // Parse register values (2 bytes each)
        let expected_len = 4 + (count as usize * 2);
        if response.len() != expected_len {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Invalid response length: expected {expected_len} bytes, got {}",
                    response.len()
                )),
            ));
        }

        let mut values = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let offset = 4 + i * 2;
            let value = i16::from_le_bytes([response[offset], response[offset + 1]]);
            values.push(value);
        }

        Ok(values)
    }

    /// Write multiple registers (0x301 command)
    ///
    /// Note: Only registers 0-559 are writable
    ///
    /// # Arguments
    ///
    /// * `start_register_number` - Starting register number (0-559)
    /// * `values` - Register values to write (max 237)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_registers(
        &self,
        start_register_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        use moto_hses_proto::commands::WriteMultipleRegisters;
        let command = WriteMultipleRegisters::new(start_register_number, values)?;
        self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// Read multiple byte variables (B) (0x302 command)
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number (0-99)
    /// * `count` - Number of variables to read (max 474, must be multiple of 2)
    ///
    /// # Returns
    ///
    /// Vector of variable values (u8)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_byte_variables(
        &self,
        start_variable_number: u8,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        let command = ReadMultipleByteVariables::new(start_variable_number, count)?;
        let response = self.send_command_with_retry(command, Division::Robot).await?;

        // Response format: Byte0-3 = count, Byte4-N = variable data (1 byte each)
        if response.len() < 4 {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Response too short: expected at least 4 bytes, got {}",
                    response.len()
                )),
            ));
        }

        let response_count =
            u32::from_le_bytes([response[0], response[1], response[2], response[3]]);

        if response_count != count {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Count mismatch: expected {count}, got {response_count}"
                )),
            ));
        }

        // Parse variable values (1 byte each)
        let expected_len = 4 + count as usize;
        if response.len() != expected_len {
            return Err(ClientError::ProtocolError(
                moto_hses_proto::error::ProtocolError::Deserialization(format!(
                    "Invalid response length: expected {expected_len} bytes, got {}",
                    response.len()
                )),
            ));
        }

        let values = response[4..].to_vec();
        Ok(values)
    }

    /// Write multiple byte variables (B) (0x302 command)
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number (0-99)
    /// * `values` - Variable values to write (max 474, must be multiple of 2 in length)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_byte_variables(
        &self,
        start_variable_number: u8,
        values: Vec<u8>,
    ) -> Result<(), ClientError> {
        let command = WriteMultipleByteVariables::new(start_variable_number, values)?;
        self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub fn execute_job(&self, _job_number: u8) -> Result<(), ClientError> {
        // TODO: Implement I/O reading command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError("Job execution not yet implemented".to_string()))
    }

    /// # Errors
    ///
    /// Returns an error if communication fails
    pub fn stop_job(&self) -> Result<(), ClientError> {
        // TODO: Implement job stop command
        // For now, return a placeholder implementation
        Err(ClientError::SystemError("Job stop not yet implemented".to_string()))
    }

    // File operations (Division = 0x02)

    /// Get file list from controller
    ///
    /// # Arguments
    /// * `pattern` - File name pattern to filter results (e.g., "*.JBI", "*.DAT")
    ///
    /// Returns a list of filenames matching the pattern available on the controller.
    ///
    /// # Errors
    ///
    /// Returns an error if the file list request fails
    pub async fn read_file_list(&self, pattern: &str) -> Result<Vec<String>, ClientError> {
        let command = ReadFileList::new(pattern.to_string(), self.config.text_encoding);
        let response = self.send_command_with_retry(command, Division::File).await?;
        parse_file_list(&response, self.config.text_encoding).map_err(ClientError::from)
    }

    /// Send file to controller
    ///
    /// # Arguments
    /// * `filename` - Name of the file to send
    /// * `content` - File content as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the file send request fails
    pub async fn send_file(&self, filename: &str, content: &[u8]) -> Result<(), ClientError> {
        let command =
            SendFile::new(filename.to_string(), content.to_vec(), self.config.text_encoding);
        let _response = self.send_command_with_retry(command, Division::File).await?;
        Ok(())
    }

    /// Receive file from controller
    ///
    /// # Arguments
    /// * `filename` - Name of the file to receive
    ///
    /// Returns the file content as a string decoded with the client's text encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if the file receive request fails
    pub async fn receive_file(&self, filename: &str) -> Result<String, ClientError> {
        let command = ReceiveFile::new(filename.to_string(), self.config.text_encoding);
        let response = self.send_command_with_retry(command, Division::File).await?;
        let content_bytes = parse_file_content(&response).map_err(ClientError::from)?;

        // Decode bytes to string using client's text encoding
        let content_string = moto_hses_proto::encoding_utils::decode_string_with_fallback(
            &content_bytes,
            self.config.text_encoding,
        );

        Ok(content_string)
    }

    /// Delete file from controller
    ///
    /// # Arguments
    /// * `filename` - Name of the file to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the file delete request fails
    pub async fn delete_file(&self, filename: &str) -> Result<(), ClientError> {
        let command = DeleteFile::new(filename.to_string(), self.config.text_encoding);
        let _response = self.send_command_with_retry(command, Division::File).await?;
        Ok(())
    }

    // Command sending with retry logic (returns raw bytes)
    async fn send_command_with_retry<C: Command + Send + Sync>(
        &self,
        command: C,
        division: Division,
    ) -> Result<Vec<u8>, ClientError> {
        let mut last_error = None;
        let mut attempts = 0;
        let max_attempts = self.config.retry_count + 1; // Initial attempt + retries

        while attempts < max_attempts {
            match self.send_command_once(&command, division).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;

                    if attempts < max_attempts {
                        sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ClientError::SystemError("Unknown error".to_string())))
    }

    // Single command sending (no retry, returns raw bytes)
    async fn send_command_once<C: Command + Send + Sync>(
        &self,
        command: &C,
        division: Division,
    ) -> Result<Vec<u8>, ClientError> {
        let request_id = self.inner.request_id.fetch_add(1, Ordering::Relaxed);
        let payload = command.serialize()?;

        // Create and send message
        let request = RequestParams {
            division,
            command: C::command_id(),
            instance: command.instance(),
            attribute: command.attribute(),
            service: command.service(),
        };
        let sequence = SequenceParams {
            request_id,
            block_number: 0u32, // Block number (0 for requests)
            ack: 0x00,          // ACK (Request)
        };
        let message = Self::create_message(&request, &sequence, payload)?;
        debug!("Sending message to {}: {} bytes", self.inner.remote_addr, message.len());
        debug!("Message bytes: {message:02X?}");
        self.inner.socket.send_to(&message, self.inner.remote_addr).await?;

        // Wait for response
        let response = self.wait_for_response(request_id, division, command.service()).await?;

        // Return raw response payload
        Ok(response)
    }

    fn create_message(
        request: &RequestParams,
        sequence: &SequenceParams,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, ClientError> {
        Self::create_message_common(sequence, request, payload)
    }

    fn create_message_common(
        sequence: &SequenceParams,
        request: &RequestParams,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, ClientError> {
        let mut message = Vec::new();

        // Magic bytes "YERC"
        message.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        message.extend_from_slice(&0x20u16.to_le_bytes());

        // Payload size
        let payload_len = u16::try_from(payload.len()).map_err(|_| {
            ClientError::ProtocolError(moto_hses_proto::ProtocolError::InvalidMessage(
                "Payload too large for protocol".to_string(),
            ))
        })?;
        message.extend_from_slice(&payload_len.to_le_bytes());

        // Reserved magic constant
        message.push(0x03);

        // Division (Robot or File)
        message.push(request.division as u8);

        // ACK
        message.push(sequence.ack);

        // Request ID
        message.push(sequence.request_id);

        // Block number
        message.extend_from_slice(&sequence.block_number.to_le_bytes());

        // Reserved (8 bytes of '9')
        message.extend_from_slice(b"99999999");

        // Command
        message.extend_from_slice(&request.command.to_le_bytes());

        // Instance
        message.extend_from_slice(&request.instance.to_le_bytes());

        // Attribute
        message.push(request.attribute);

        // Service
        message.push(request.service);

        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        message.extend(payload);

        Ok(message)
    }

    async fn wait_for_response(
        &self,
        request_id: u8,
        division: Division,
        service: u8,
    ) -> Result<Vec<u8>, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];
        let mut all_payload = Vec::new();
        let mut expected_block_number = 1u32;

        loop {
            let (len, _addr) =
                timeout(self.config.timeout, self.inner.socket.recv_from(&mut buffer))
                    .await
                    .map_err(|_| ClientError::TimeoutError("Response timeout".to_string()))??;

            let response_data = &buffer[..len];

            // Debug: Log received data
            debug!("Received response: {len} bytes");
            debug!("Response data: {response_data:02X?}");
            if len >= 4 {
                debug!("Magic bytes: {:?}", &response_data[0..4]);
            }
            if len >= 11 {
                debug!("Request ID: 0x{:02x}", response_data[11]);
            }
            if len >= 10 {
                debug!("ACK: 0x{:02x}", response_data[10]);
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
                    let error_message = Self::build_error_message(status, response_data);
                    return Err(ClientError::ProtocolError(
                        moto_hses_proto::ProtocolError::ServerError(error_message),
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

            // Extract block number (bytes 12-15)
            let block_number = u32::from_le_bytes([
                response_data[12],
                response_data[13],
                response_data[14],
                response_data[15],
            ]);

            // Check if this is a single-block response (block_number == 0x8000_0000)
            if block_number == 0x8000_0000 {
                debug!("Received single-block response");
                return Ok(payload);
            }

            // Multi-block response handling for file control commands
            // Only read_file_list (0x32) and receive_file (0x16) use multi-block responses
            if service == 0x32 || service == 0x16 {
                // Check if this is the final block (0x8000_0000 flag)
                let is_final_block = (block_number & 0x8000_0000) != 0;
                let actual_block_number = block_number & 0x7FFF_FFFF;

                debug!("Received block {actual_block_number} (final: {is_final_block})");

                // Validate block number sequence
                if actual_block_number != expected_block_number {
                    debug!(
                        "Unexpected block number: expected {expected_block_number}, got {actual_block_number}"
                    );
                    continue;
                }

                // Accumulate payload
                all_payload.extend_from_slice(&payload);

                // Send ACK packet for this block
                if let Err(e) =
                    self.send_ack_packet(request_id, block_number, division, service).await
                {
                    debug!("Failed to send ACK packet: {e}");
                    // Continue anyway, as the main response was received
                }

                // If this is the final block, we're done
                if is_final_block {
                    debug!("Received final block, total payload size: {} bytes", all_payload.len());
                    return Ok(all_payload);
                }

                // Prepare for next block
                expected_block_number += 1;
            } else {
                // For other commands, treat as single-block response
                debug!("Received single-block response for service 0x{service:02x}");
                return Ok(payload);
            }
        }
    }

    /// Build error message with added status information
    fn build_error_message(status: u8, response_data: &[u8]) -> String {
        let mut error_message = format!("Server returned error status: 0x{status:02x}");

        if let Some(added_status) = Self::read_added_status(response_data) {
            let _ = write!(error_message, " (added status: 0x{added_status:X})");
        }

        error_message
    }

    /// Send ACK packet for file operations
    async fn send_ack_packet(
        &self,
        request_id: u8,
        block_number: u32,
        division: Division,
        service: u8,
    ) -> Result<(), ClientError> {
        let sequence = SequenceParams {
            request_id,
            block_number,
            ack: 0x01, // ACK (Response)
        };
        let request = RequestParams {
            division,
            command: 0x0000,  // Command (0x0000 for ACK)
            instance: 0x0000, // Instance (0x0000 for ACK)
            attribute: 0x00,  // Attribute (0x00 for ACK)
            service,          // Service (same as original request service)
        };
        let ack_message = Self::create_message_common(
            &sequence,
            &request,
            Vec::new(), // Empty payload for ACK
        )?;

        debug!("Sending ACK packet: {} bytes", ack_message.len());
        debug!("ACK message bytes: {ack_message:02X?}");

        self.inner.socket.send_to(&ack_message, self.inner.remote_addr).await?;
        Ok(())
    }

    /// Read added status from response data
    fn read_added_status(response_data: &[u8]) -> Option<u32> {
        // Check if we have enough data for added status size (byte 26)
        if response_data.len() < 27 {
            return None;
        }

        let added_status_size = response_data[26];
        if added_status_size == 0 {
            return None;
        }

        match added_status_size {
            1 => {
                // 1 WORD data (2 bytes) - added status is at bytes 28-29
                if response_data.len() >= 30 {
                    Some(u32::from(u16::from_le_bytes([response_data[28], response_data[29]])))
                } else {
                    None
                }
            }
            2 => {
                // 2 WORD data (4 bytes) - added status is at bytes 28-31
                if response_data.len() >= 32 {
                    Some(u32::from_le_bytes([
                        response_data[28],
                        response_data[29],
                        response_data[30],
                        response_data[31],
                    ]))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
