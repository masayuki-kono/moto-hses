//! Register command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for single register operations (0x79)
pub struct RegisterHandler;

impl CommandHandler for RegisterHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let reg_number = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate register number range (0-999 for read, 0-559 for write)
        if reg_number > 999 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid register number: {reg_number} (must be 0-999)"
            )));
        }

        match service {
            0x0e => {
                // Read - return 2 bytes (i16)
                let value = state.get_register(reg_number);
                Ok(value.to_le_bytes().to_vec())
            }
            0x10 => {
                // Write - validate writable range (0-559)
                if reg_number > 559 {
                    return Err(proto::ProtocolError::InvalidCommand);
                }

                if message.payload.len() >= 2 {
                    let value = i16::from_le_bytes([message.payload[0], message.payload[1]]);
                    state.set_register(reg_number, value);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for plural register operations (0x301)
pub struct PluralRegisterHandler;

impl CommandHandler for PluralRegisterHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_register = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Validate register number range (0-999)
        if start_register > 999 {
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

        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-237)"
            )));
        }

        // Validate range doesn't exceed maximum register number
        let end_register = u32::from(start_register) + count - 1;
        if end_register > 999 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Register range exceeds maximum: {start_register}-{end_register} (max 999)"
            )));
        }

        match service {
            0x33 => {
                // Read - return count + register data
                let values = state.get_multiple_registers(start_register, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                for value in values {
                    response.extend_from_slice(&value.to_le_bytes());
                }
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + (count as usize * 2);
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(
                        "Invalid payload length".to_string(),
                    ));
                }

                // Only registers 0-559 are writable
                if start_register > 559 || end_register > 559 {
                    return Err(proto::ProtocolError::InvalidCommand);
                }

                // Parse register values
                let mut values = Vec::new();
                for i in 0..count as usize {
                    let offset = 4 + i * 2;
                    let value =
                        i16::from_le_bytes([message.payload[offset], message.payload[offset + 1]]);
                    values.push(value);
                }

                state.set_multiple_registers(start_register, &values);

                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
