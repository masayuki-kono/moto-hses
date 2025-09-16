//! I/O and register command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Validate I/O number range according to HSES protocol specification
const fn is_valid_io_number(io_number: u16) -> bool {
    matches!(
        io_number,
        1..=128 |           // Robot user input
        1001..=1128 |       // Robot user output
        2001..=2127 |       // External input
        2501..=2628 |       // Network input
        3001..=3128 |       // External output
        3501..=3628 |       // Network output
        4001..=4160 |       // Robot system input
        5001..=5200 |       // Robot system output
        6001..=6064 |       // Interface panel input
        7001..=7999 |       // Auxiliary relay
        8001..=8064 |       // Robot control status signal
        8201..=8220         // Pseudo input
    )
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

        // Validate I/O number range
        if !is_valid_io_number(io_number) {
            return Err(proto::ProtocolError::InvalidCommand);
        }

        match service {
            0x0e => {
                // Read
                let value = u8::from(state.get_io_state(io_number));
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
