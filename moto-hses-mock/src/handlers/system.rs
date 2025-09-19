//! System information and status command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for status reading (0x72)
pub struct StatusHandler;

impl CommandHandler for StatusHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        use moto_hses_proto::VariableType;

        let attribute = message.sub_header.attribute;
        let mut data = match attribute {
            1 => state.status.data1.serialize(state.text_encoding)?,
            2 => state.status.data2.serialize(state.text_encoding)?,
            _ => state.status.serialize(state.text_encoding)?, // Default to complete status
        };

        // Extend to 8 bytes if needed
        if data.len() < 8 {
            data.extend(vec![0u8; 8 - data.len()]);
        }

        Ok(data)
    }
}

/// Handler for axis name reading (0x74)
pub struct AxisNameHandler;

impl CommandHandler for AxisNameHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 56]; // 7 axes * 8 bytes each

        // Set default axis names
        let axis_names =
            ["1st_axis", "2nd_axis", "3rd_axis", "4th_axis", "5th_axis", "6th_axis", "7th_axis"];

        for (i, name) in axis_names.iter().enumerate() {
            let name_bytes =
                moto_hses_proto::encoding_utils::encode_string(name, state.text_encoding);
            let start = i * 8;
            let len = name_bytes.len().min(7);
            data[start..start + len].copy_from_slice(&name_bytes[0..len]);
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
            let value = i32::try_from(i).map_err(|_| {
                proto::ProtocolError::InvalidMessage("Invalid axis value".to_string())
            })?;
            data[i * 4..(i + 1) * 4].copy_from_slice(&(value * 100).to_le_bytes());
        }

        Ok(data)
    }
}

/// Handler for management time acquisition (0x88)
pub struct ManagementTimeHandler;

impl CommandHandler for ManagementTimeHandler {
    fn handle(
        &self,
        _message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 32];

        // Set some default time values as strings
        let start_time = "1234567890";
        let elapse_time = "987654321";

        // Copy start time (16 bytes)
        let start_bytes =
            moto_hses_proto::encoding_utils::encode_string(start_time, state.text_encoding);
        let start_len = start_bytes.len().min(15);
        data[0..start_len].copy_from_slice(&start_bytes[0..start_len]);

        // Copy elapse time (16 bytes)
        let elapse_bytes =
            moto_hses_proto::encoding_utils::encode_string(elapse_time, state.text_encoding);
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
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let mut data = vec![0u8; 48];

        // Software version (16 bytes)
        let version = "V1.0.0";
        let version_bytes =
            moto_hses_proto::encoding_utils::encode_string(version, state.text_encoding);
        let len = version_bytes.len().min(15);
        data[0..len].copy_from_slice(&version_bytes[0..len]);

        // Model (16 bytes)
        let model = "FS100";
        let model_bytes =
            moto_hses_proto::encoding_utils::encode_string(model, state.text_encoding);
        let len = model_bytes.len().min(15);
        data[16..16 + len].copy_from_slice(&model_bytes[0..len]);

        // Parameter version (16 bytes)
        let param_version = "P1.0.0";
        let param_version_bytes =
            moto_hses_proto::encoding_utils::encode_string(param_version, state.text_encoding);
        let len = param_version_bytes.len().min(15);
        data[32..32 + len].copy_from_slice(&param_version_bytes[0..len]);

        Ok(data)
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
                3 => {
                    // HLOCK (Programming Pendant and I/O operation system interlock)
                    state.set_hlock(value == 1);
                }
                _ => {}
            }
        }

        Ok(vec![])
    }
}
