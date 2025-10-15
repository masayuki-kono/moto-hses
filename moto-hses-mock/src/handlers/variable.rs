//! Variable-related command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for byte variable operations (0x7a)
pub struct ByteVarHandler;

impl CommandHandler for ByteVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance; // Direct use since instance is u16
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for B variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid variable index: {var_index} (valid range: 0-99)"
            )));
        }

        match service {
            0x0e => {
                // Read
                state.get_variable(var_index).map_or_else(
                    || {
                        // B variable: 1 byte (actual data type size)
                        Ok(vec![0])
                    },
                    |value| {
                        if value.is_empty() {
                            Ok(vec![0])
                        } else {
                            // Return actual data type size (1 byte for B variable)
                            Ok(vec![value[0]])
                        }
                    },
                )
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
        let var_index = message.sub_header.instance; // Direct use since instance is u16
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for I variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid variable index: {var_index} (valid range: 0-99)"
            )));
        }

        match service {
            0x0e => {
                // Read
                state.get_variable(var_index).map_or_else(
                    || {
                        // I variable: 2 bytes (actual data type size)
                        Ok(vec![0, 0])
                    },
                    |value| {
                        if value.len() >= 2 {
                            // Return actual data type size (2 bytes for I variable)
                            Ok(value[0..2].to_vec())
                        } else {
                            Ok(vec![0, 0])
                        }
                    },
                )
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

/// Handler for double precision integer variable operations (0x7c)
pub struct DoubleVarHandler;

impl CommandHandler for DoubleVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance; // Direct use since instance is u16
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for D variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid variable index: {var_index} (valid range: 0-99)"
            )));
        }

        match service {
            0x0e => {
                // Read
                state.get_variable(var_index).map_or_else(
                    || {
                        // Return 4 bytes for 32-bit integer variable
                        Ok(vec![0, 0, 0, 0])
                    },
                    |value| {
                        // Protocol specification: 4 bytes for 32-bit integer (D variable)
                        if value.len() >= 4 {
                            Ok(value[0..4].to_vec())
                        } else {
                            Ok(vec![0, 0, 0, 0])
                        }
                    },
                )
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

/// Handler for real variable operations (0x7d)
pub struct RealVarHandler;

impl CommandHandler for RealVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance; // Direct use since instance is u16
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for R variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid variable index: {var_index} (valid range: 0-99)"
            )));
        }

        match service {
            0x0e => {
                // Read
                state.get_variable(var_index).map_or_else(
                    || {
                        // Return 4 bytes for real variable as expected by Python client
                        Ok(vec![0, 0, 0, 0])
                    },
                    |value| {
                        // Python client expects 4 bytes
                        if value.len() >= 4 {
                            Ok(value[0..4].to_vec())
                        } else {
                            // Extend existing value to 4 bytes
                            let mut extended_value = value.clone();
                            extended_value.extend(vec![0u8; 4 - value.len()]);
                            Ok(extended_value)
                        }
                    },
                )
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

/// Handler for string variable operations (0x7e)
pub struct StringVarHandler;

impl CommandHandler for StringVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance; // Direct use since instance is u16
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for S variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidInstance(format!(
                "Invalid variable index: {var_index} (valid range: 0-99)"
            )));
        }

        match service {
            0x0e => {
                // Read
                state.get_variable(var_index).map_or_else(
                    || {
                        // Return 16 bytes for uninitialized variables (all zeros)
                        Ok(vec![0u8; 16])
                    },
                    |value| {
                        // S variables are 16 bytes (4 × 32-bit integers)
                        // Pad with null bytes to 16 bytes
                        let mut response = vec![0u8; 16];
                        let copy_len = std::cmp::min(value.len(), 16);
                        response[..copy_len].copy_from_slice(&value[..copy_len]);
                        Ok(response)
                    },
                )
            }
            0x10 => {
                // Write
                if message.payload.len() >= 16 {
                    // Store the full 16-byte S variable data, but trim trailing nulls for storage
                    let data = &message.payload[..16];
                    let trimmed_len = data.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
                    state.set_variable(var_index, data[..trimmed_len].to_vec());
                } else if !message.payload.is_empty() {
                    // Handle shorter payloads by padding with zeros
                    let mut data = message.payload.clone();
                    data.resize(16, 0); // Pad to 16 bytes
                    let trimmed_len = data.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
                    state.set_variable(var_index, data[..trimmed_len].to_vec());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for plural byte variable operations (0x302)
pub struct PluralByteVarHandler;

impl CommandHandler for PluralByteVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Payload too short: {} bytes for start_variable {} (need at least 4 bytes)",
                message.payload.len(),
                start_variable
            )));
        }

        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);

        // Validate count (max 474, must be > 0, must be multiple of 2)
        if count == 0 || count > 474 || !count.is_multiple_of(2) {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-474 and multiple of 2)"
            )));
        }

        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_byte_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                response.extend_from_slice(&values);
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

                // Parse variable values (1 byte each)
                let values = message.payload[4..].to_vec();

                state.set_multiple_byte_variables(start_variable, &values);

                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for plural integer variable operations (0x303)
pub struct PluralIntegerVarHandler;

impl CommandHandler for PluralIntegerVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Payload too short: {} bytes for start_variable {start_variable} (need at least 4 bytes)",
                message.payload.len()
            )));
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
                "Invalid count: {count} for start_variable {start_variable} (must be 1-237)"
            )));
        }

        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_integer_variables(start_variable, count as usize);
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
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {expected_len}",
                        message.payload.len()
                    )));
                }

                // Parse variable values (2 bytes each)
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 2;
                    let value =
                        i16::from_le_bytes([message.payload[offset], message.payload[offset + 1]]);
                    values.push(value);
                }

                state.set_multiple_integer_variables(start_variable, &values);

                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for plural double precision integer variable operations (0x304)
pub struct PluralDoubleVarHandler;

impl CommandHandler for PluralDoubleVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Payload too short: {} bytes for start_variable {start_variable} (need at least 4 bytes)",
                message.payload.len()
            )));
        }

        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);

        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-118)"
            )));
        }

        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_double_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                for value in values {
                    response.extend_from_slice(&value.to_le_bytes());
                }
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + (count as usize * 4);
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {expected_len}",
                        message.payload.len()
                    )));
                }

                // Parse variable values (4 bytes each)
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 4;
                    let value = i32::from_le_bytes([
                        message.payload[offset],
                        message.payload[offset + 1],
                        message.payload[offset + 2],
                        message.payload[offset + 3],
                    ]);
                    values.push(value);
                }

                state.set_multiple_double_variables(start_variable, &values);

                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for plural real type variable operations (0x305)
pub struct PluralRealVarHandler;

impl CommandHandler for PluralRealVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Payload too short: {} bytes for start_variable {start_variable} (need at least 4 bytes)",
                message.payload.len()
            )));
        }

        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);

        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-118)"
            )));
        }

        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_real_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                for value in values {
                    response.extend_from_slice(&value.to_le_bytes());
                }
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + (count as usize * 4);
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {expected_len}",
                        message.payload.len()
                    )));
                }

                // Parse variable values (4 bytes each)
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 4;
                    let value = f32::from_le_bytes([
                        message.payload[offset],
                        message.payload[offset + 1],
                        message.payload[offset + 2],
                        message.payload[offset + 3],
                    ]);
                    values.push(value);
                }

                state.set_multiple_real_variables(start_variable, &values);

                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
