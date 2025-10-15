//! Variable type implementations

use crate::error::ProtocolError;
use crate::payload::HsesPayload;
use bytes::Buf;

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
        // D variable: 4 bytes (actual data type size)
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
        // R variable: 4 bytes (actual data type size)
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

impl HsesPayload for String {
    fn serialize(&self, encoding: crate::encoding::TextEncoding) -> Result<Vec<u8>, ProtocolError> {
        // S variables are 16 bytes (4 × 32-bit integers)
        let encoded_bytes = crate::encoding_utils::encode_string(self, encoding);

        if encoded_bytes.len() > 16 {
            return Err(ProtocolError::InvalidMessage(format!(
                "String exceeds 16 bytes when encoded: {} bytes",
                encoded_bytes.len()
            )));
        }

        // Pad to 16 bytes with null terminator
        let mut result = vec![0u8; 16];
        result[..encoded_bytes.len()].copy_from_slice(&encoded_bytes);
        Ok(result)
    }

    fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // S variables should be 16 bytes, but handle shorter responses gracefully
        let byte_array = if data.len() >= 16 { &data[..16] } else { data };

        // Find null terminator
        let trimmed_bytes =
            byte_array.iter().position(|&b| b == 0).map_or(byte_array, |pos| &byte_array[..pos]);

        // Decode using specified encoding with fallback
        let string = crate::encoding_utils::decode_string_with_fallback(trimmed_bytes, encoding);

        Ok(string)
    }
}

// Multiple variable payload implementations

impl HsesPayload for Vec<u8> {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // Multiple B variables: serialize as byte array
        Ok(self.clone())
    }

    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // Multiple B variables: deserialize as byte array
        Ok(data.to_vec())
    }
}

impl HsesPayload for Vec<i16> {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // Multiple I variables: serialize as little-endian byte array
        let mut result = Vec::with_capacity(self.len() * 2);
        for value in self {
            result.extend_from_slice(&value.to_le_bytes());
        }
        Ok(result)
    }

    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // Multiple I variables: deserialize from little-endian byte array
        if !data.len().is_multiple_of(2) {
            return Err(ProtocolError::Deserialization(format!(
                "Invalid data length for i16 array: {} bytes (must be multiple of 2)",
                data.len()
            )));
        }

        let mut result = Self::with_capacity(data.len() / 2);
        let mut buf = data;
        while buf.remaining() >= 2 {
            result.push(buf.get_i16_le());
        }
        Ok(result)
    }
}

impl HsesPayload for Vec<i32> {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // Multiple D variables: serialize as little-endian byte array
        let mut result = Vec::with_capacity(self.len() * 4);
        for value in self {
            result.extend_from_slice(&value.to_le_bytes());
        }
        Ok(result)
    }

    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // Multiple D variables: deserialize from little-endian byte array
        if !data.len().is_multiple_of(4) {
            return Err(ProtocolError::Deserialization(format!(
                "Invalid data length for i32 array: {} bytes (must be multiple of 4)",
                data.len()
            )));
        }

        let mut result = Self::with_capacity(data.len() / 4);
        let mut buf = data;
        while buf.remaining() >= 4 {
            result.push(buf.get_i32_le());
        }
        Ok(result)
    }
}

impl HsesPayload for Vec<f32> {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        // Multiple R variables: serialize as little-endian byte array
        let mut result = Vec::with_capacity(self.len() * 4);
        for value in self {
            result.extend_from_slice(&value.to_le_bytes());
        }
        Ok(result)
    }

    fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // Multiple R variables: deserialize from little-endian byte array
        if !data.len().is_multiple_of(4) {
            return Err(ProtocolError::Deserialization(format!(
                "Invalid data length for f32 array: {} bytes (must be multiple of 4)",
                data.len()
            )));
        }

        let mut result = Self::with_capacity(data.len() / 4);
        let mut buf = data;
        while buf.remaining() >= 4 {
            result.push(buf.get_f32_le());
        }
        Ok(result)
    }
}

impl HsesPayload for Vec<String> {
    fn serialize(&self, encoding: crate::encoding::TextEncoding) -> Result<Vec<u8>, ProtocolError> {
        // Multiple S variables: serialize as byte array (each element is 16 bytes)
        let mut result = Vec::with_capacity(self.len() * 16);
        for string in self {
            let encoded_bytes = crate::encoding_utils::encode_string(string, encoding);

            if encoded_bytes.len() > 16 {
                return Err(ProtocolError::InvalidMessage(format!(
                    "String exceeds 16 bytes when encoded: {} bytes",
                    encoded_bytes.len()
                )));
            }

            // Pad to 16 bytes with null terminator
            let mut byte_array = vec![0u8; 16];
            byte_array[..encoded_bytes.len()].copy_from_slice(&encoded_bytes);
            result.extend_from_slice(&byte_array);
        }
        Ok(result)
    }

    fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        // Multiple S variables: deserialize from byte array (each element is 16 bytes)
        if !data.len().is_multiple_of(16) {
            return Err(ProtocolError::Deserialization(format!(
                "Invalid data length for String array: {} bytes (must be multiple of 16)",
                data.len()
            )));
        }

        let mut result = Self::with_capacity(data.len() / 16);
        for chunk in data.chunks(16) {
            // Find null terminator
            let trimmed_bytes =
                chunk.iter().position(|&b| b == 0).map_or(chunk, |pos| &chunk[..pos]);

            // Decode using specified encoding with fallback
            let string =
                crate::encoding_utils::decode_string_with_fallback(trimmed_bytes, encoding);
            result.push(string);
        }
        Ok(result)
    }
}
