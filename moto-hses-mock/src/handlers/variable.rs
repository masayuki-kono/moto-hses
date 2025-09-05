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
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Protocol specification compliant: 4 bytes (Byte 0: B variable, Byte 1-3: Reserved)
                    if !value.is_empty() {
                        let mut response = vec![0u8; 4];
                        response[0] = value[0];
                        Ok(response)
                    } else {
                        Ok(vec![0, 0, 0, 0])
                    }
                } else {
                    // Protocol specification compliant: 4 bytes (Byte 0: B variable, Byte 1-3: Reserved)
                    Ok(vec![0, 0, 0, 0])
                }
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
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Protocol specification compliant: 4 bytes (Byte 0-1: I variable, Byte 2-3: Reserved)
                    if value.len() >= 2 {
                        let mut response = vec![0u8; 4];
                        response[0..2].copy_from_slice(&value[0..2]);
                        Ok(response)
                    } else {
                        Ok(vec![0, 0, 0, 0])
                    }
                } else {
                    // Protocol specification compliant: 4 bytes (Byte 0-1: I variable, Byte 2-3: Reserved)
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if message.payload.len() >= 4 {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for double variable operations (0x7c)
pub struct DoubleVarHandler;

impl CommandHandler for DoubleVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Python client expects 4 bytes
                    if value.len() >= 4 {
                        Ok(value[0..4].to_vec())
                    } else {
                        Ok(vec![0, 0, 0, 0])
                    }
                } else {
                    // Return 4 bytes for double variable as expected by Python client
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if message.payload.len() >= 8 {
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
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    // Python client expects 4 bytes
                    if value.len() >= 4 {
                        Ok(value[0..4].to_vec())
                    } else {
                        // Extend existing value to 4 bytes
                        let mut extended_value = value.clone();
                        extended_value.extend(vec![0u8; 4 - value.len()]);
                        Ok(extended_value)
                    }
                } else {
                    // Return 4 bytes for real variable as expected by Python client
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => {
                // Write
                if message.payload.len() >= 4 {
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
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;

        match service {
            0x0e => {
                // Read
                if let Some(value) = state.get_variable(var_index) {
                    Ok(value.clone())
                } else {
                    Ok(vec![0u8; 16]) // Empty string
                }
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
