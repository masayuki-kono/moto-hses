//! Position-related command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

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
