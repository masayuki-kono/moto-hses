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

/// Handler for plural I/O operations (0x300)
pub struct PluralIoHandler;

impl CommandHandler for PluralIoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_io_number = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Validate I/O number range
        if !IoCategory::is_valid_io_number(start_io_number) {
            return Err(proto::ProtocolError::InvalidCommand);
        }

        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage("Payload too short".to_string()));
        }

        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);

        // Validate count (max 474, must be multiple of 2)
        if count == 0 || count > 474 || count % 2 != 0 {
            return Err(proto::ProtocolError::InvalidMessage("Invalid count".to_string()));
        }

        match service {
            0x33 => {
                // Read - validate full range before reading
                let count_u16 = u16::try_from(count).map_err(|_| {
                    proto::ProtocolError::InvalidMessage("Count too large".to_string())
                })?;
                let end_io_number = start_io_number
                    .checked_add(count_u16.checked_sub(1).ok_or_else(|| {
                        proto::ProtocolError::InvalidMessage("Count is zero".to_string())
                    })?)
                    .ok_or_else(|| {
                        proto::ProtocolError::InvalidMessage("I/O range overflow".to_string())
                    })?;

                // Validate that the entire range falls within the same category
                if !IoCategory::is_valid_io_number(end_io_number) {
                    return Err(proto::ProtocolError::InvalidMessage(
                        "I/O range exceeds category bounds".to_string(),
                    ));
                }

                // Read - return count + I/O data
                let io_data = state
                    .get_multiple_io_states(start_io_number, count as usize)
                    .map_err(proto::ProtocolError::InvalidMessage)?;
                let mut response = count.to_le_bytes().to_vec();
                response.extend_from_slice(&io_data);
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + count as usize;
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(
                        "Invalid payload length".to_string(),
                    ));
                }

                // Only network input signals are writable
                if !(2701..=2956).contains(&start_io_number) {
                    return Err(proto::ProtocolError::InvalidCommand);
                }

                // Validate the full range of I/O numbers being written
                let io_data = &message.payload[4..];
                let io_data_count = io_data.len();
                let io_data_count_u16 = u16::try_from(io_data_count).map_err(|_| {
                    proto::ProtocolError::InvalidMessage("I/O data count too large".to_string())
                })?;
                let end_io_number = start_io_number + (io_data_count_u16 * 8) - 1;

                // Check that the entire range falls within network input range (2701..=2956)
                if end_io_number > 2956 {
                    return Err(proto::ProtocolError::InvalidCommand);
                }
                state
                    .set_multiple_io_states(start_io_number, io_data)
                    .map_err(proto::ProtocolError::InvalidMessage)?;

                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
