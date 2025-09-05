//! Command handlers for mock server

use crate::state::MockState;
use moto_hses_proto as proto;
use std::sync::Arc;

/// Command handler trait
pub trait CommandHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError>;
}

/// Handler for alarm data reading (0x70)
pub struct AlarmDataHandler;

impl CommandHandler for AlarmDataHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let instance = message.sub_header.instance as usize;
        let attribute = message.sub_header.attribute;

        // Python client expects 60 bytes: 4+4+4+16+32
        let mut data = vec![0u8; 60];

        if instance == 0 || instance > state.alarms.len() {
            // Default alarm data when no alarms exist
            data[0..4].copy_from_slice(&0u32.to_le_bytes()); // Default alarm code
            data[4..8].copy_from_slice(&0u32.to_le_bytes()); // Default alarm data
            data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                              // data[12..28] and data[28..60] remain 0 (default time and name)
        } else {
            let alarm = &state.alarms[instance - 1];

            if attribute == 0 {
                // For attribute 0, return complete alarm data
                let alarm_data = alarm.serialize_complete()?;

                // Copy complete alarm data to response
                if alarm_data.len() >= 60 {
                    data[..60].copy_from_slice(&alarm_data[..60]);
                } else {
                    data[..alarm_data.len()].copy_from_slice(&alarm_data);
                }
            } else {
                // For specific attributes, use the existing logic
                let alarm_data = alarm.serialize(attribute)?;

                // Copy alarm data to appropriate positions
                if alarm_data.len() >= 8 {
                    data[0..8].copy_from_slice(&alarm_data[0..8]);
                }
                // Fill remaining fields with default values
                data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                                  // data[12..28] and data[28..60] remain 0 (default time and name)
            }
        }

        Ok(data)
    }
}

/// Handler for status reading (0x72)
pub struct StatusHandler;

impl CommandHandler for StatusHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        use moto_hses_proto::VariableType;
        let mut data = state.status.serialize()?;

        // Extend to 8 bytes if needed
        if data.len() < 8 {
            data.extend(vec![0u8; 8 - data.len()]);
        }

        Ok(data)
    }
}

/// Handler for current position reading (0x75)
pub struct PositionHandler;

impl CommandHandler for PositionHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        state.position.serialize()
    }
}

/// Handler for I/O operations (0x78)
pub struct IoHandler;

impl CommandHandler for IoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let io_number = message.sub_header.instance;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                let value = if state.get_io_state(io_number) { 1 } else { 0 };
                Ok(vec![value, 0, 0, 0])
            }
            0x10 => {
                // Write
                if message.payload.len() >= 4 {
                    let value = i32::from_le_bytes([
                        message.payload[0],
                        message.payload[1],
                        message.payload[2],
                        message.payload[3],
                    ]);
                    state.set_io_state(io_number, value != 0);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for register operations (0x79)
pub struct RegisterHandler;

impl CommandHandler for RegisterHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let reg_number = message.sub_header.instance;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                let value = state.get_register(reg_number);
                Ok(value.to_le_bytes().to_vec())
            }
            0x10 => {
                // Write
                if message.payload.len() >= 4 {
                    let value = i32::from_le_bytes([
                        message.payload[0],
                        message.payload[1],
                        message.payload[2],
                        message.payload[3],
                    ]);
                    state.set_register(reg_number, value);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for byte variable operations (0x7a)
pub struct ByteVarHandler;

impl CommandHandler for ByteVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Protocol specification compliant: 4 bytes (Byte 0: B variable, Byte 1-3: Reserved)
                    if !value.is_empty() {
                        let mut response = vec![0u8; 4];
                        response[0] = value[0];
                        Ok(response)
                    } else {
                        Ok(vec![0, 0, 0, 0])
                    }
                } else {
                    // Protocol specification compliant: 4 bytes (Byte 0: B variable, Byte 1-3: Reserved)
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if !message.payload.is_empty() {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for integer variable operations (0x7b)
pub struct IntegerVarHandler;

impl CommandHandler for IntegerVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Protocol specification compliant: 4 bytes (Byte 0-1: I variable, Byte 2-3: Reserved)
                    if value.len() >= 2 {
                        let mut response = vec![0u8; 4];
                        response[0..2].copy_from_slice(&value[0..2]);
                        Ok(response)
                    } else {
                        Ok(vec![0, 0, 0, 0])
                    }
                } else {
                    // Protocol specification compliant: 4 bytes (Byte 0-1: I variable, Byte 2-3: Reserved)
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if message.payload.len() >= 4 {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for real variable operations (0x7d)
pub struct RealVarHandler;

impl CommandHandler for RealVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Python client expects 4 bytes
                    if value.len() >= 4 {
                        Ok(value[0..4].to_vec())
                    } else {
                        // Extend existing value to 4 bytes
                        let mut extended_value = value.clone();
                        extended_value.extend(vec![0u8; 4 - value.len()]);
                        Ok(extended_value)
                    }
                } else {
                    // Return 4 bytes for real variable as expected by Python client
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if message.payload.len() >= 4 {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for position variable operations (0x7f)
pub struct PositionVarHandler;

impl CommandHandler for PositionVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let _var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x01 => {
                // GetAll
                state.position.serialize()
            }
            0x02 => {
                // SetAll
                if message.payload.len() >= 52 {
                    if let Ok(position) = proto::Position::deserialize(&message.payload) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            0x0e => {
                // Read
                state.position.serialize()
            }
            0x10 => {
                // Write
                if message.payload.len() >= 52 {
                    if let Ok(position) = proto::Position::deserialize(&message.payload) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for alarm reset/error cancel (0x82)
pub struct AlarmResetHandler;

impl CommandHandler for AlarmResetHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let reset_type = message.sub_header.instance;

        match reset_type {
            1 => {
                // RESET
                state.clear_alarms();
            }
            2 => {
                // CANCEL
                state.status.error = false;
            }
            _ => {}
        }

        Ok(vec![])
    }
}

/// Handler for hold/servo control (0x83)
pub struct HoldServoHandler;

impl CommandHandler for HoldServoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let control_type = message.sub_header.instance;

        if message.payload.len() >= 4 {
            let value = i32::from_le_bytes([
                message.payload[0],
                message.payload[1],
                message.payload[2],
                message.payload[3],
            ]);

            match control_type {
                1 => {
                    // HOLD
                    state.set_hold(value == 1);
                }
                2 => {
                    // Servo ON
                    state.set_servo(value == 1);
                }
                _ => {}
            }
        }

        Ok(vec![])
    }
}

/// Handler for job start (0x86)
pub struct JobStartHandler;

impl CommandHandler for JobStartHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        state.set_running(true);
        Ok(vec![])
    }
}

/// Handler for job select (0x87)
pub struct JobSelectHandler;

impl CommandHandler for JobSelectHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let select_type = message.sub_header.instance;

        if select_type == 1 {
            // Set execution job
            if message.payload.len() >= 4 {
                // In a real implementation, this would parse the job name
                state.set_current_job(Some("SELECTED.JOB".to_string()));
            }
        }

        Ok(vec![])
    }
}

/// Handler for alarm info reading (0x71)
pub struct AlarmInfoHandler;

impl CommandHandler for AlarmInfoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let alarm_number = message.sub_header.instance as usize;
        let attribute = message.sub_header.attribute;

        // Python client expects 60 bytes: 4+4+4+16+32
        let mut data = vec![0u8; 60];

        if alarm_number == 0 || alarm_number > state.alarms.len() {
            // Default alarm data when no alarms exist
            data[0..4].copy_from_slice(&0u32.to_le_bytes()); // Default alarm code
            data[4..8].copy_from_slice(&0u32.to_le_bytes()); // Default alarm data
            data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                              // data[12..28] and data[28..60] remain 0 (default time and name)
        } else {
            let alarm = &state.alarms[alarm_number - 1];

            if attribute == 0 {
                // For attribute 0, return complete alarm data
                let alarm_data = alarm.serialize_complete()?;

                // Copy complete alarm data to response
                if alarm_data.len() >= 60 {
                    data[..60].copy_from_slice(&alarm_data[..60]);
                } else {
                    data[..alarm_data.len()].copy_from_slice(&alarm_data);
                }
            } else {
                // For specific attributes, use the existing logic
                let alarm_data = alarm.serialize(attribute)?;

                // Copy alarm data to appropriate positions
                if alarm_data.len() >= 8 {
                    data[0..8].copy_from_slice(&alarm_data[0..8]);
                }
                // Fill remaining fields with default values
                data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                                  // data[12..28] and data[28..60] remain 0 (default time and name)
            }
        }

        Ok(data)
    }
}

/// Handler for executing job info reading (0x73)
pub struct ExecutingJobInfoHandler;

impl CommandHandler for ExecutingJobInfoHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 64];

        // Job name (32 bytes)
        if let Some(job_name) = &state.current_job {
            let name_bytes = job_name.as_bytes();
            let len = name_bytes.len().min(31);
            data[0..len].copy_from_slice(&name_bytes[0..len]);
        }

        // Line number (4 bytes)
        data[32..36].copy_from_slice(&1000u32.to_le_bytes());

        // Other fields remain 0

        Ok(data)
    }
}

/// Handler for axis name reading (0x74)
pub struct AxisNameHandler;

impl CommandHandler for AxisNameHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 56]; // 7 axes * 8 bytes each

        // Set default axis names
        let axis_names = [
            "1st_axis", "2nd_axis", "3rd_axis", "4th_axis", "5th_axis", "6th_axis", "7th_axis",
        ];

        for (i, name) in axis_names.iter().enumerate() {
            let name_bytes = name.as_bytes();
            let start = i * 8;
            let len = name_bytes.len().min(7);
            data[start..start + len].copy_from_slice(&name_bytes[0..len]);
        }

        Ok(data)
    }
}

/// Handler for position error reading (0x76)
pub struct PositionErrorHandler;

impl CommandHandler for PositionErrorHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 28]; // 7 axes * 4 bytes each

        // Set some default position errors
        for i in 0..7 {
            data[i * 4..(i + 1) * 4].copy_from_slice(&(i as u32 * 10).to_le_bytes());
        }

        Ok(data)
    }
}

/// Handler for torque reading (0x77)
pub struct TorqueHandler;

impl CommandHandler for TorqueHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 28]; // 7 axes * 4 bytes each

        // Set some default torque values
        for i in 0..7 {
            data[i * 4..(i + 1) * 4].copy_from_slice(&(i as i32 * 100).to_le_bytes());
        }

        Ok(data)
    }
}

/// Handler for double variable operations (0x7c)
pub struct DoubleVarHandler;

impl CommandHandler for DoubleVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Python client expects 4 bytes
                    if value.len() >= 4 {
                        Ok(value[0..4].to_vec())
                    } else {
                        Ok(vec![0, 0, 0, 0])
                    }
                } else {
                    // Return 4 bytes for double variable as expected by Python client
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if message.payload.len() >= 8 {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for string variable operations (0x7e)
pub struct StringVarHandler;

impl CommandHandler for StringVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    Ok(value.clone())
                } else {
                    Ok(vec![0u8; 16]) // Empty string
                }
            }
            0x10 => {
                // Write
                if !message.payload.is_empty() {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for base position variable operations (0x80)
pub struct BasePositionVarHandler;

impl CommandHandler for BasePositionVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let _var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x01 => {
                // GetAll
                state.position.serialize()
            }
            0x02 => {
                // SetAll
                if message.payload.len() >= 36 {
                    // Parse base position data
                    let mut data = vec![0u8; 52];
                    data[0..36].copy_from_slice(&message.payload[0..36]);
                    if let Ok(position) = proto::Position::deserialize(&data) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            0x0e => {
                // Read
                state.position.serialize()
            }
            0x10 => {
                // Write
                if message.payload.len() >= 52 {
                    if let Ok(position) = proto::Position::deserialize(&message.payload) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for external axis variable operations (0x81)
pub struct ExternalAxisVarHandler;

impl CommandHandler for ExternalAxisVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let _var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x01 => {
                // GetAll
                state.position.serialize()
            }
            0x02 => {
                // SetAll
                if message.payload.len() >= 36 {
                    // Parse external axis data
                    let mut data = vec![0u8; 52];
                    data[0..36].copy_from_slice(&message.payload[0..36]);
                    if let Ok(position) = proto::Position::deserialize(&data) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            0x0e => {
                // Read
                state.position.serialize()
            }
            0x10 => {
                // Write
                if message.payload.len() >= 52 {
                    if let Ok(position) = proto::Position::deserialize(&message.payload) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for text display on pendant (0x85)
pub struct TextDisplayHandler;

impl CommandHandler for TextDisplayHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let service = message.sub_header.service;

        match service {
            0x10 => {
                // Write
                // Just acknowledge the text display command
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for management time acquisition (0x88)
pub struct ManagementTimeHandler;

impl CommandHandler for ManagementTimeHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 32];

        // Set some default time values as strings
        let start_time = "1234567890";
        let elapse_time = "987654321";

        // Copy start time (16 bytes)
        let start_bytes = start_time.as_bytes();
        let start_len = start_bytes.len().min(15);
        data[0..start_len].copy_from_slice(&start_bytes[0..start_len]);

        // Copy elapse time (16 bytes)
        let elapse_bytes = elapse_time.as_bytes();
        let elapse_len = elapse_bytes.len().min(15);
        data[16..16 + elapse_len].copy_from_slice(&elapse_bytes[0..elapse_len]);

        Ok(data)
    }
}

/// Handler for system info acquisition (0x89)
pub struct SystemInfoHandler;

impl CommandHandler for SystemInfoHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 48];

        // Software version (16 bytes)
        let version = "V1.0.0";
        let version_bytes = version.as_bytes();
        let len = version_bytes.len().min(15);
        data[0..len].copy_from_slice(&version_bytes[0..len]);

        // Model (16 bytes)
        let model = "FS100";
        let model_bytes = model.as_bytes();
        let len = model_bytes.len().min(15);
        data[16..16 + len].copy_from_slice(&model_bytes[0..len]);

        // Parameter version (16 bytes)
        let param_version = "P1.0.0";
        let param_version_bytes = param_version.as_bytes();
        let len = param_version_bytes.len().min(15);
        data[32..32 + len].copy_from_slice(&param_version_bytes[0..len]);

        Ok(data)
    }
}

/// Handler for MOV command (0x8a)
pub struct MovHandler;

impl CommandHandler for MovHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let service = message.sub_header.service;

        match service {
            0x02 => {
                // SetAll
                if message.payload.len() >= 104 {
                    // Parse position data and update state
                    if let Ok(position) = proto::Position::deserialize(&message.payload[0..52]) {
                        state.update_position(position);
                    }
                }
                // Set running to false after MOV command
                state.set_running(false);
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for PMOV command (0x8b)
pub struct PmovHandler;

impl CommandHandler for PmovHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let service = message.sub_header.service;

        match service {
            0x02 => {
                // SetAll
                if message.payload.len() >= 88 {
                    // Parse position data and update state
                    if let Ok(position) = proto::Position::deserialize(&message.payload[0..52]) {
                        state.update_position(position);
                    }
                }
                // Set running to false after PMOV command
                state.set_running(false);
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for SelectCycle command (0x84)
pub struct SelectCycleHandler;

impl CommandHandler for SelectCycleHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        _state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let service = message.sub_header.service;

        match service {
            0x10 => {
                // Select cycle type
                // For now, just acknowledge the cycle selection
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for file operations (0x00)
pub struct FileControlHandler;

impl CommandHandler for FileControlHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let service = message.sub_header.service;

        match service {
            0x01 => {
                // Get file list
                // Return a simple file list
                let file_list = "TEST.JOB\0";
                Ok(file_list.as_bytes().to_vec())
            }
            0x02 => {
                // Send file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    let content = message.payload[filename_pos + 1..].to_vec();
                    state.set_file(filename, content);
                }
                Ok(vec![])
            }
            0x03 => {
                // Receive file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    if let Some(content) = state.get_file(&filename) {
                        let mut response = filename.as_bytes().to_vec();
                        response.push(0);
                        response.extend(content);
                        return Ok(response);
                    }
                }
                Ok(vec![])
            }
            0x04 => {
                // Delete file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    state.delete_file(&filename);
                }
                Ok(vec![])
            }
            0x15 => {
                // Send file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    let content = message.payload[filename_pos + 1..].to_vec();
                    let filename_clone = filename.clone();
                    let content_len = content.len();
                    state.set_file(filename, content);
                    eprintln!("File saved: {} ({} bytes)", filename_clone, content_len);
                }
                Ok(vec![])
            }
            0x32 => {
                // Get file list (Python client uses this)
                // Return actual file list from state
                let files = state.get_file_list("*");
                let mut file_list = String::new();
                for file in files {
                    file_list.push_str(&file);
                    file_list.push('\0');
                }
                eprintln!("File list requested, returning: {:?}", file_list);
                Ok(file_list.as_bytes().to_vec())
            }
            0x16 => {
                // Receive file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    if let Some(content) = state.get_file(&filename) {
                        let mut response = filename.as_bytes().to_vec();
                        response.push(0);
                        response.extend(content);
                        eprintln!("File requested: {} ({} bytes)", filename, content.len());
                        return Ok(response);
                    } else {
                        eprintln!("File not found: {}", filename);
                    }
                }
                Ok(vec![])
            }
            0x09 => {
                // Delete file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    let deleted = state.delete_file(&filename);
                    eprintln!(
                        "File deletion requested: {} (deleted: {})",
                        filename, deleted
                    );
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Command handler registry
#[derive(Clone)]
pub struct CommandHandlerRegistry {
    handlers: std::collections::HashMap<u16, Arc<dyn CommandHandler + Send + Sync>>,
}

impl CommandHandlerRegistry {
    pub fn new() -> Self {
        let mut handlers = std::collections::HashMap::new();

        handlers.insert(
            0x00,
            Arc::new(FileControlHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x70,
            Arc::new(AlarmDataHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x71,
            Arc::new(AlarmInfoHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x72,
            Arc::new(StatusHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x73,
            Arc::new(ExecutingJobInfoHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x74,
            Arc::new(AxisNameHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x75,
            Arc::new(PositionHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x76,
            Arc::new(PositionErrorHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x77,
            Arc::new(TorqueHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x78,
            Arc::new(IoHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x79,
            Arc::new(RegisterHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x7a,
            Arc::new(ByteVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x7b,
            Arc::new(IntegerVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x7c,
            Arc::new(DoubleVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x7d,
            Arc::new(RealVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x7e,
            Arc::new(StringVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x7f,
            Arc::new(PositionVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x80,
            Arc::new(BasePositionVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x81,
            Arc::new(ExternalAxisVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x82,
            Arc::new(AlarmResetHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x83,
            Arc::new(HoldServoHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x85,
            Arc::new(TextDisplayHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x86,
            Arc::new(JobStartHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x87,
            Arc::new(JobSelectHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x88,
            Arc::new(ManagementTimeHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x89,
            Arc::new(SystemInfoHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x8a,
            Arc::new(MovHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x8b,
            Arc::new(PmovHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x84,
            Arc::new(SelectCycleHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );

        Self { handlers }
    }

    pub fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let command = message.sub_header.command;

        if let Some(handler) = self.handlers.get(&command) {
            handler.handle(message, state)
        } else {
            eprintln!("Unknown command: 0x{:04x}", command);
            Err(proto::ProtocolError::InvalidCommand)
        }
    }
}

impl Default for CommandHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
