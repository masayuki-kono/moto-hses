//! I/O and register command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;
use moto_hses_proto::commands::io::IoCategory;

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

        // Validate I/O number range
        if !IoCategory::is_valid_io_number(io_number) {
            return Err(proto::ProtocolError::InvalidCommand);
        }

        match service {
            0x0e => {
                // Read - return 1 byte representing 8 I/O states
                let value = u8::from(state.get_io_state(io_number));
                Ok(vec![value])
            }
            0x10 => {
                // Write - accept 1 byte representing 8 I/O states
                if !message.payload.is_empty() {
                    let value = message.payload[0];
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

        // Validate register number range (0-999)
        if reg_number > 999 {
            return Err(proto::ProtocolError::InvalidCommand);
        }

        match service {
            0x0e => {
                // Read
                let value = state.get_register(reg_number);
                // Register data is 2 bytes (i16) + 2 bytes reserved = 4 bytes total
                let mut response = Vec::new();
                response.extend_from_slice(&value.to_le_bytes()); // 2 bytes
                response.extend_from_slice(&[0u8, 0u8]); // 2 bytes reserved
                Ok(response)
            }
            0x10 => {
                // Write
                if message.payload.len() >= 2 {
                    // Register data is 2 bytes (i16) + 2 bytes reserved = 4 bytes total
                    // We only use the first 2 bytes for the actual register value
                    let value = i16::from_le_bytes([message.payload[0], message.payload[1]]);
                    state.set_register(reg_number, value);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
