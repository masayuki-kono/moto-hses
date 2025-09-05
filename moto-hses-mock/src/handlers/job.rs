//! Job and movement command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

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
