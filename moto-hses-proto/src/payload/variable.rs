//! Variable type implementations

use crate::error::ProtocolError;
use crate::payload::HsesPayload;
use bytes::Buf;

// Implementations for basic variable types
impl HsesPayload for u8 {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // B variable: 1 byte (actual data type size)
        Ok(vec![*self])
    }
    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::Underflow);
        }
        Ok(data[0])
    }
}

impl HsesPayload for i16 {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // I variable: 2 bytes (actual data type size)
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        if data.len() < 2 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_i16_le())
    }
}

impl HsesPayload for i32 {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_i32_le())
    }
}

impl HsesPayload for f32 {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_f32_le())
    }
}

impl HsesPayload for Vec<u8> {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // S variables are 16 bytes (4 × 32-bit integers)
        // Pad with null bytes to 16 bytes
        let mut result = vec![0u8; 16];
        let copy_len = std::cmp::min(self.len(), 16);
        result[..copy_len].copy_from_slice(&self[..copy_len]);
        Ok(result)
    }

    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // S variables should be 16 bytes, but handle shorter responses gracefully
        // Always pad to 16 bytes first, then remove trailing nulls for consistent behavior
        let mut padded_data = [0u8; 16];
        let copy_len = std::cmp::min(data.len(), 16);
        padded_data[..copy_len].copy_from_slice(&data[..copy_len]);

        // Remove trailing null bytes for cleaner API
        let trimmed_len = padded_data.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);

        // If all bytes are null, return empty vector
        if trimmed_len == 0 {
            Ok(vec![])
        } else {
            Ok(padded_data[..trimmed_len].to_vec())
        }
    }
}
