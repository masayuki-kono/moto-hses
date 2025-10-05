//! Payload trait for HSES protocol data types

use crate::error::ProtocolError;

/// Trait for HSES protocol payload data types
pub trait HsesPayload: Send + Sync + 'static {
    /// Serialize the variable to byte data with specified text encoding
    ///
    /// # Errors
    /// Returns `ProtocolError` if serialization fails
    fn serialize(&self, encoding: crate::encoding::TextEncoding) -> Result<Vec<u8>, ProtocolError>;
    /// Deserialize the variable from byte data with specified text encoding
    ///
    /// # Errors
    /// Returns `ProtocolError` if deserialization fails
    fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError>
    where
        Self: Sized;
}
