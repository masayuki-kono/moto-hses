//! Variable type implementations

use crate::error::ProtocolError;
use crate::types::VariableType;
use bytes::Buf;

// Implementations for basic variable types
impl VariableType for u8 {
    fn command_id() -> u16 {
        0x7a
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = vec![0u8; 4];
        data[0] = *self;
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        Ok(data[0])
    }
}

impl VariableType for i32 {
    fn command_id() -> u16 {
        0x7b
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_i32_le())
    }
}

impl VariableType for f32 {
    fn command_id() -> u16 {
        0x7d
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_f32_le())
    }
}

// Unit type implementation for write operations
impl VariableType for () {
    fn command_id() -> u16 {
        0x00
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn deserialize(_data: &[u8]) -> Result<Self, ProtocolError> {
        Ok(())
    }
}
