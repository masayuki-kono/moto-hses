//! Variable related commands (`ReadVar`, `WriteVar`)

use super::command_trait::Command;
use crate::{HsesPayload, error::ProtocolError};
use std::marker::PhantomData;

/// Command ID mapping for variable types
pub trait VariableCommandId {
    fn command_id() -> u16;
}

// Implement VariableCommandId for each variable type
impl VariableCommandId for u8 {
    fn command_id() -> u16 {
        0x7a
    }
}

impl VariableCommandId for i16 {
    fn command_id() -> u16 {
        0x7b
    }
}

impl VariableCommandId for i32 {
    fn command_id() -> u16 {
        0x7c
    }
}

impl VariableCommandId for f32 {
    fn command_id() -> u16 {
        0x7d
    }
}

impl VariableCommandId for Vec<u8> {
    fn command_id() -> u16 {
        0x7e
    }
}

pub struct ReadVar<T: HsesPayload + VariableCommandId> {
    pub index: u8,
    pub _phantom: PhantomData<T>,
}

impl<T: HsesPayload + VariableCommandId> Command for ReadVar<T> {
    type Response = T;
    fn command_id() -> u16 {
        T::command_id()
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        u16::from(self.index) // Variable number (0-99 for byte, 0-999 for int/real)
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

pub struct WriteVar<T: HsesPayload + VariableCommandId> {
    pub index: u8,
    pub value: T,
}

impl<T: HsesPayload + VariableCommandId> Command for WriteVar<T> {
    type Response = ();
    fn command_id() -> u16 {
        T::command_id()
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // WriteVar requires encoding, but Command trait doesn't support it
        // This is a design limitation - we'll use UTF-8 as default
        self.value.serialize(crate::encoding::TextEncoding::Utf8)
    }
    fn instance(&self) -> u16 {
        u16::from(self.index) // Variable number (0-99 for byte, 0-999 for int/real)
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

/// Read multiple byte variables (B) command (0x302)
#[derive(Debug, Clone)]
pub struct ReadMultipleByteVariables {
    pub start_variable_number: u8,
    pub count: u32, // Number of B variable data (max 474, must be multiple of 2)
}

impl ReadMultipleByteVariables {
    /// Create a new `ReadMultipleByteVariables` command
    ///
    /// # Errors
    ///
    /// Returns an error if the variable number is invalid or count is out of range
    pub fn new(start_variable_number: u8, count: u32) -> Result<Self, ProtocolError> {
        // Validate variable number (0-99)
        if start_variable_number > 99 {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid variable number: {start_variable_number} (valid range: 0-99)"
            )));
        }
        // Validate count (max 474, must be > 0, must be multiple of 2)
        if count == 0 || count > 474 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-474)"
            )));
        }
        if !count.is_multiple_of(2) {
            return Err(ProtocolError::InvalidMessage(format!(
                "Count must be multiple of 2: {count}"
            )));
        }
        // Validate range doesn't exceed maximum variable number
        let end_variable = u32::from(start_variable_number) + count - 1;
        if end_variable > 99 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Variable range exceeds maximum: {start_variable_number}-{end_variable} (max 99)"
            )));
        }
        Ok(Self { start_variable_number, count })
    }
}

impl Command for ReadMultipleByteVariables {
    type Response = Vec<u8>; // Array of B variable values
    fn command_id() -> u16 {
        0x302
    }
    fn instance(&self) -> u16 {
        u16::from(self.start_variable_number)
    }
    fn attribute(&self) -> u8 {
        0 // Fixed to 0 for plural commands
    }
    fn service(&self) -> u8 {
        0x33 // Read plural data
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple byte variables (B) command (0x302)
#[derive(Debug, Clone)]
pub struct WriteMultipleByteVariables {
    pub start_variable_number: u8,
    pub values: Vec<u8>, // B variable values to write
}

impl WriteMultipleByteVariables {
    /// Create a new `WriteMultipleByteVariables` command
    ///
    /// # Errors
    ///
    /// Returns an error if the variable number is invalid or count is out of range
    pub fn new(start_variable_number: u8, values: Vec<u8>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 474, must be > 0, must be multiple of 2)
        if count == 0 || count > 474 || !count.is_multiple_of(2) {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-474 and multiple of 2)"
            )));
        }
        // Validate variable number range (0-99)
        if start_variable_number > 99 {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid variable number: {start_variable_number} (valid range: 0-99)"
            )));
        }
        let end_variable = u32::from(start_variable_number)
            + u32::try_from(count)
                .map_err(|_| ProtocolError::InvalidMessage("Count too large".to_string()))?
            - 1;
        if end_variable > 99 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Variable range exceeds maximum: {start_variable_number}-{end_variable} (max 99)"
            )));
        }
        Ok(Self { start_variable_number, values })
    }
}

impl Command for WriteMultipleByteVariables {
    type Response = ();
    fn command_id() -> u16 {
        0x302
    }
    fn instance(&self) -> u16 {
        u16::from(self.start_variable_number)
    }
    fn attribute(&self) -> u8 {
        0 // Fixed to 0 for plural commands
    }
    fn service(&self) -> u8 {
        0x34 // Write plural data
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len())
            .map_err(|_| ProtocolError::InvalidMessage("Values count too large".to_string()))?;
        let mut payload = count.to_le_bytes().to_vec();
        payload.extend_from_slice(&self.values);
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test_read_multiple_byte_variables_new() {
        // Valid cases
        assert!(ReadMultipleByteVariables::new(0, 2).is_ok());
        assert!(ReadMultipleByteVariables::new(50, 20).is_ok());
        assert!(ReadMultipleByteVariables::new(99, 2).is_err()); // 99 + 2 - 1 = 100 > 99
        assert!(ReadMultipleByteVariables::new(98, 2).is_ok()); // 98 + 2 - 1 = 99
        assert!(ReadMultipleByteVariables::new(0, 100).is_ok()); // Max 100 variables (0-99)
        assert!(ReadMultipleByteVariables::new(0, 50).is_ok());

        // Invalid variable number
        assert!(ReadMultipleByteVariables::new(100, 2).is_err());

        // Invalid count (must be > 0, <= 474, multiple of 2)
        assert!(ReadMultipleByteVariables::new(0, 0).is_err()); // count = 0
        assert!(ReadMultipleByteVariables::new(0, 475).is_err()); // count > 474
        assert!(ReadMultipleByteVariables::new(0, 102).is_err()); // count would exceed variable range
        assert!(ReadMultipleByteVariables::new(0, 1).is_err()); // odd count
        assert!(ReadMultipleByteVariables::new(0, 3).is_err()); // odd count
        assert!(ReadMultipleByteVariables::new(0, 473).is_err()); // odd count

        // Range overflow (start + count - 1 > 99)
        assert!(ReadMultipleByteVariables::new(99, 2).is_err()); // 99 + 2 - 1 = 100 > 99
        assert!(ReadMultipleByteVariables::new(50, 52).is_err()); // 50 + 52 - 1 = 101 > 99
    }

    #[test]
    fn test_read_multiple_byte_variables_command_trait() {
        let cmd = ReadMultipleByteVariables::new(10, 4).expect("Valid command should not fail");

        assert_eq!(ReadMultipleByteVariables::command_id(), 0x302);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    fn test_read_multiple_byte_variables_serialize() {
        let cmd = ReadMultipleByteVariables::new(10, 6).expect("Valid command should not fail");
        let payload = cmd.serialize().expect("Serialization should not fail");

        assert_eq!(payload.len(), 4);
        assert_eq!(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]), 6);
    }

    #[test]
    fn test_write_multiple_byte_variables_new() {
        // Valid cases
        assert!(WriteMultipleByteVariables::new(0, vec![10, 20]).is_ok());
        assert!(WriteMultipleByteVariables::new(50, vec![100, 200, 150, 250]).is_ok());
        assert!(WriteMultipleByteVariables::new(98, vec![99, 100]).is_ok());
        assert!(WriteMultipleByteVariables::new(0, vec![0; 100]).is_ok());

        // Invalid variable number
        assert!(WriteMultipleByteVariables::new(100, vec![10, 20]).is_err());

        // Invalid count (must be > 0, <= 474, multiple of 2)
        assert!(WriteMultipleByteVariables::new(0, vec![]).is_err()); // empty
        assert!(WriteMultipleByteVariables::new(0, vec![0; 475]).is_err()); // count > 474
        assert!(WriteMultipleByteVariables::new(0, vec![0; 102]).is_err()); // count would exceed variable range
        assert!(WriteMultipleByteVariables::new(0, vec![10]).is_err()); // odd count
        assert!(WriteMultipleByteVariables::new(0, vec![10, 20, 30]).is_err()); // odd count

        // Range overflow
        assert!(WriteMultipleByteVariables::new(99, vec![10, 20]).is_err()); // 99 + 2 - 1 = 100 > 99
        assert!(WriteMultipleByteVariables::new(50, vec![0; 52]).is_err()); // 50 + 52 - 1 = 101 > 99
    }

    #[test]
    fn test_write_multiple_byte_variables_command_trait() {
        let cmd = WriteMultipleByteVariables::new(10, vec![100, 200, 150, 250])
            .expect("Valid command should not fail");

        assert_eq!(WriteMultipleByteVariables::command_id(), 0x302);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    fn test_write_multiple_byte_variables_serialize_odd_count_fails() {
        // This should fail at the new() level due to odd count
        let result = WriteMultipleByteVariables::new(10, vec![100, 200, 150]);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_multiple_byte_variables_serialize_valid() {
        let cmd = WriteMultipleByteVariables::new(10, vec![100, 200, 150, 250])
            .expect("Valid command should not fail");
        let payload = cmd.serialize().expect("Serialization should not fail");

        assert_eq!(payload.len(), 4 + 4); // 4 bytes count + 4 bytes data

        // Check count
        assert_eq!(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]), 4);

        // Check data
        assert_eq!(payload[4], 100);
        assert_eq!(payload[5], 200);
        assert_eq!(payload[6], 150);
        assert_eq!(payload[7], 250);
    }

    #[test]
    fn test_write_multiple_byte_variables_serialize_maximum() {
        let large_values: Vec<u8> = (0..100).map(|i| u8::try_from(i).unwrap_or(0)).collect();
        let cmd = WriteMultipleByteVariables::new(0, large_values)
            .expect("Valid command should not fail");
        let payload = cmd.serialize().expect("Serialization should not fail");

        assert_eq!(payload.len(), 4 + 100); // 4 bytes count + 100 bytes data
        assert_eq!(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]), 100);
    }

    #[test]
    fn test_boundary_conditions() {
        // Test minimum valid count
        assert!(ReadMultipleByteVariables::new(0, 2).is_ok());
        assert!(WriteMultipleByteVariables::new(0, vec![10, 20]).is_ok());

        // Test maximum valid count (100 variables: 0-99)
        assert!(ReadMultipleByteVariables::new(0, 100).is_ok());
        assert!(WriteMultipleByteVariables::new(0, vec![0; 100]).is_ok());

        // Test variable number boundaries
        assert!(ReadMultipleByteVariables::new(99, 2).is_err()); // Would exceed range
        assert!(ReadMultipleByteVariables::new(98, 2).is_ok()); // Just within range
    }
}
