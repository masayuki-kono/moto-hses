//! Job and movement command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for reading executing job information (0x73)
pub struct ExecutingJobInfoHandler;

impl CommandHandler for ExecutingJobInfoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let task_type = message.sub_header.instance;
        let attribute = message.sub_header.attribute;
        let service = message.sub_header.service;

        // Validate task type (1-6)
        if !matches!(task_type, 1..=6) {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid task type: {task_type} (valid range: 1-6)"
            )));
        }

        // Validate attribute (0-4)
        if attribute > 4 {
            return Err(proto::ProtocolError::InvalidService);
        }

        // Create ExecutingJobInfo based on attribute
        let job_info = match attribute {
            0..=4 => state.executing_job.clone().unwrap_or_default(),
            _ => {
                return Err(proto::ProtocolError::InvalidService);
            }
        };

        match service {
            0x0e => job_info.serialize(attribute, state.text_encoding),
            0x01 => job_info.serialize_complete(state.text_encoding),
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for job start (0x86)
pub struct JobStartHandler;

impl CommandHandler for JobStartHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        // Validate instance, attribute, service
        if message.sub_header.instance != 1 {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid instance: {} (expected: 1)",
                message.sub_header.instance
            )));
        }
        if message.sub_header.attribute != 1 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }
        if message.sub_header.service != 0x10 {
            return Err(proto::ProtocolError::InvalidService);
        }

        // Validate payload (should be 4 bytes with value 1)
        if message.payload.len() != 4 {
            return Err(proto::ProtocolError::InvalidMessage("Invalid payload length".to_string()));
        }

        // Validate payload content (should be [1, 0, 0, 0])
        if message.payload != [1, 0, 0, 0] {
            return Err(proto::ProtocolError::InvalidMessage(
                "Invalid payload content".to_string(),
            ));
        }

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
        // Validate instance (select type)
        let select_type = message.sub_header.instance;
        if select_type != 1 && !(10..=15).contains(&select_type) {
            return Err(proto::ProtocolError::InvalidMessage("Invalid instance".to_string()));
        }

        // Validate attribute (should be 1)
        if message.sub_header.attribute != 1 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Validate service (should be 0x02 for Set_Attribute_All)
        if message.sub_header.service != 0x02 {
            return Err(proto::ProtocolError::InvalidService);
        }

        // Validate payload (should be 36 bytes: 32 bytes for job name + 4 bytes for line number)
        if message.payload.len() != 36 {
            return Err(proto::ProtocolError::InvalidMessage("Invalid payload length".to_string()));
        }

        // Parse job name (first 32 bytes, fixed length)
        let job_name_bytes = &message.payload[0..32];
        // Decode using the MockState's text encoding (same as client's encoding)
        let job_name =
            proto::encoding_utils::decode_string_with_fallback(job_name_bytes, state.text_encoding);
        // Remove null characters from the end
        let job_name = job_name.trim_end_matches('\0').to_string();

        // Parse line number (last 4 bytes, little-endian)
        let line_number = u32::from_le_bytes([
            message.payload[32],
            message.payload[33],
            message.payload[34],
            message.payload[35],
        ]);

        // Validate line number (0 to 9999)
        if line_number > 9999 {
            return Err(proto::ProtocolError::InvalidMessage(
                "Line number out of range".to_string(),
            ));
        }

        // Update state
        state.set_selected_job(job_name, line_number, select_type);

        Ok(vec![])
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
                    if let Ok(position) =
                        proto::Position::deserialize(&message.payload[0..52], state.text_encoding)
                    {
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
                    if let Ok(position) =
                        proto::Position::deserialize(&message.payload[0..52], state.text_encoding)
                    {
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
