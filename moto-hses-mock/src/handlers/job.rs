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
            return Err(proto::ProtocolError::InvalidCommand);
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
            0x0e => job_info.serialize(attribute),
            0x01 => job_info.serialize_complete(),
            _ => Err(proto::ProtocolError::InvalidService),
        }
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
                state.set_executing_job(Some(proto::ExecutingJobInfo::new(
                    "SELECTED.JOB".to_string(),
                    0,
                    0,
                    0,
                )));
            }
        }

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
