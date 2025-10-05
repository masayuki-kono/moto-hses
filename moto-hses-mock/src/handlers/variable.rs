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
        let var_index = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidMessage("Variable index too large".to_string())
        })?;
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for B variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidCommand);
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
        let var_index = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidMessage("Variable index too large".to_string())
        })?;
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for I variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidCommand);
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
        let var_index = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidMessage("Variable index too large".to_string())
        })?;
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for D variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidCommand);
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
        let var_index = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidMessage("Variable index too large".to_string())
        })?;
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for R variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidCommand);
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
        let var_index = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidMessage("Variable index too large".to_string())
        })?;
        let service = message.sub_header.service;

        // Validate variable index range (0-99 for S variables)
        if var_index > 99 {
            return Err(proto::ProtocolError::InvalidCommand);
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
