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
    pub index: u16, // Support extended variable settings (0-999)
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
        self.index // Direct use since it's already u16
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

pub struct WriteVar<T: HsesPayload + VariableCommandId> {
    pub index: u16, // Support extended variable settings (0-999)
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
        self.index // Direct use since it's already u16
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
    pub start_variable_number: u16,
    pub count: u32, // Number of B variable data (max 474, must be multiple of 2)
}

impl ReadMultipleByteVariables {
    /// Create a new `ReadMultipleByteVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `count` - Number of variables to read (max 474, must be multiple of 2)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate count (max 474, must be > 0, must be multiple of 2)
        if count == 0 || count > 474 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-474)"
            )));
        }
        if !count.is_multiple_of(2) {
            return Err(ProtocolError::InvalidMessage(format!(
                "Count must be multiple of 2: {count} for start_variable {start_variable_number}"
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
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple byte variables (B) command (0x302)
#[derive(Debug, Clone)]
pub struct WriteMultipleByteVariables {
    pub start_variable_number: u16,
    pub values: Vec<u8>, // B variable values to write
}

impl WriteMultipleByteVariables {
    /// Create a new `WriteMultipleByteVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `values` - Variable values to write (max 474, must be multiple of 2 in length)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, values: Vec<u8>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 474, must be > 0, must be multiple of 2)
        if count == 0 || count > 474 || !count.is_multiple_of(2) {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-474 and multiple of 2)"
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
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} too large for u32 conversion",
                self.values.len()
            ))
        })?;
        let mut payload = count.to_le_bytes().to_vec();
        payload.extend_from_slice(&self.values);
        Ok(payload)
    }
}

/// Read multiple integer variables (I) command (0x303)
#[derive(Debug, Clone)]
pub struct ReadMultipleIntegerVariables {
    pub start_variable_number: u16,
    pub count: u32, // Number of I variable data (max 237)
}

impl ReadMultipleIntegerVariables {
    /// Create a new `ReadMultipleIntegerVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `count` - Number of variables to read (max 237)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-237)"
            )));
        }
        Ok(Self { start_variable_number, count })
    }
}

impl Command for ReadMultipleIntegerVariables {
    type Response = Vec<i16>; // Array of I variable values
    fn command_id() -> u16 {
        0x303
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple integer variables (I) command (0x303)
#[derive(Debug, Clone)]
pub struct WriteMultipleIntegerVariables {
    pub start_variable_number: u16,
    pub values: Vec<i16>, // I variable values to write
}

impl WriteMultipleIntegerVariables {
    /// Create a new `WriteMultipleIntegerVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `values` - Variable values to write (max 237 items)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, values: Vec<i16>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-237)"
            )));
        }
        Ok(Self { start_variable_number, values })
    }
}

impl Command for WriteMultipleIntegerVariables {
    type Response = ();
    fn command_id() -> u16 {
        0x303
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;
        let mut payload = count.to_le_bytes().to_vec();
        for &value in &self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        Ok(payload)
    }
}

/// Read multiple double precision integer variables (D) command (0x304)
#[derive(Debug, Clone)]
pub struct ReadMultipleDoubleVariables {
    pub start_variable_number: u16, // Support extended variable settings (0-999)
    pub count: u32,                 // Number of D variable data (max 118)
}

impl ReadMultipleDoubleVariables {
    /// Create a new `ReadMultipleDoubleVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `count` - Number of variables to read (max 118)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-118)"
            )));
        }
        Ok(Self { start_variable_number, count })
    }
}

impl Command for ReadMultipleDoubleVariables {
    type Response = Vec<i32>; // Array of D variable values
    fn command_id() -> u16 {
        0x304
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple double precision integer variables (D) command (0x304)
#[derive(Debug, Clone)]
pub struct WriteMultipleDoubleVariables {
    pub start_variable_number: u16, // Support extended variable settings (0-999)
    pub values: Vec<i32>,           // D variable values to write
}

impl WriteMultipleDoubleVariables {
    /// Create a new `WriteMultipleDoubleVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `values` - Variable values to write (max 118 items)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, values: Vec<i32>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-118)"
            )));
        }
        Ok(Self { start_variable_number, values })
    }
}

impl Command for WriteMultipleDoubleVariables {
    type Response = ();
    fn command_id() -> u16 {
        0x304
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;
        let mut payload = count.to_le_bytes().to_vec();
        for &value in &self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        Ok(payload)
    }
}

/// Read multiple real type variables (R) command (0x305)
#[derive(Debug, Clone)]
pub struct ReadMultipleRealVariables {
    pub start_variable_number: u16, // Support extended variable settings (0-999)
    pub count: u32,                 // Number of R variable data (max 118)
}

impl ReadMultipleRealVariables {
    /// Create a new `ReadMultipleRealVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `count` - Number of variables to read (max 118)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-118)"
            )));
        }
        Ok(Self { start_variable_number, count })
    }
}

impl Command for ReadMultipleRealVariables {
    type Response = Vec<f32>; // Array of R variable values
    fn command_id() -> u16 {
        0x305
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple real type variables (R) command (0x305)
#[derive(Debug, Clone)]
pub struct WriteMultipleRealVariables {
    pub start_variable_number: u16, // Support extended variable settings (0-999)
    pub values: Vec<f32>,           // R variable values to write
}

impl WriteMultipleRealVariables {
    /// Create a new `WriteMultipleRealVariables` command
    ///
    /// # Arguments
    ///
    /// * `start_variable_number` - Starting variable number
    /// * `values` - Variable values to write (max 118 items)
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are invalid
    pub fn new(start_variable_number: u16, values: Vec<f32>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-118)"
            )));
        }
        Ok(Self { start_variable_number, values })
    }
}

impl Command for WriteMultipleRealVariables {
    type Response = ();
    fn command_id() -> u16 {
        0x305
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;
        let mut payload = count.to_le_bytes().to_vec();
        for &value in &self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Allow expect in tests as they should not fail with valid inputs
    #[allow(clippy::expect_used)]
    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_byte_variables_creation() {
        // Valid cases
        let cmd = ReadMultipleByteVariables::new(0, 2).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 2);

        let cmd = ReadMultipleByteVariables::new(50, 4).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.count, 4);

        // Maximum valid count
        let cmd = ReadMultipleByteVariables::new(0, 474).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 474);

        // Valid case with any start variable number
        let cmd = ReadMultipleByteVariables::new(255, 2).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.count, 2);
    }

    #[test]
    fn test_read_multiple_byte_variables_validation() {
        // Invalid count: zero
        assert!(ReadMultipleByteVariables::new(0, 0).is_err());

        // Invalid count: too large
        assert!(ReadMultipleByteVariables::new(0, 475).is_err());
        assert!(ReadMultipleByteVariables::new(0, 1000).is_err());

        // Invalid count: not multiple of 2
        assert!(ReadMultipleByteVariables::new(0, 1).is_err());
        assert!(ReadMultipleByteVariables::new(0, 3).is_err());
        assert!(ReadMultipleByteVariables::new(0, 5).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_byte_variables_command_trait() {
        let cmd = ReadMultipleByteVariables::new(10, 4).expect("Valid command should not fail");
        assert_eq!(ReadMultipleByteVariables::command_id(), 0x302);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_byte_variables_serialization() {
        let cmd = ReadMultipleByteVariables::new(5, 6).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![6, 0, 0, 0]); // 6 in little-endian
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_byte_variables_creation() {
        // Valid cases
        let values = vec![10, 20];
        let cmd = WriteMultipleByteVariables::new(0, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values, values);

        let values = vec![1, 2, 3, 4];
        let cmd = WriteMultipleByteVariables::new(50, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.values, values);

        // Maximum valid count
        let values: Vec<u8> =
            (0..474).map(|i| u8::try_from(i % 256).expect("Should fit in u8")).collect();
        let cmd =
            WriteMultipleByteVariables::new(0, values).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values.len(), 474);

        // Valid case with any start variable number
        let values = vec![10, 20];
        let cmd = WriteMultipleByteVariables::new(255, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.values, values);
    }

    #[test]
    fn test_write_multiple_byte_variables_validation() {
        // Invalid count: empty
        assert!(WriteMultipleByteVariables::new(0, vec![]).is_err());

        // Invalid count: too large
        let large_values: Vec<u8> = vec![0; 475];
        assert!(WriteMultipleByteVariables::new(0, large_values).is_err());

        // Invalid count: not multiple of 2
        assert!(WriteMultipleByteVariables::new(0, vec![1]).is_err());
        assert!(WriteMultipleByteVariables::new(0, vec![1, 2, 3]).is_err());
        assert!(WriteMultipleByteVariables::new(0, vec![1, 2, 3, 4, 5]).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_byte_variables_command_trait() {
        let values = vec![10, 20, 30, 40];
        let cmd =
            WriteMultipleByteVariables::new(10, values).expect("Valid command should not fail");
        assert_eq!(WriteMultipleByteVariables::command_id(), 0x302);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_byte_variables_serialization() {
        let values = vec![10, 20, 30, 40];
        let cmd = WriteMultipleByteVariables::new(5, values.clone())
            .expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");

        // Expected: count (4 bytes) + values
        let mut expected = vec![4, 0, 0, 0]; // 4 in little-endian
        expected.extend_from_slice(&values);
        assert_eq!(serialized, expected);
    }

    // Tests for ReadMultipleIntegerVariables (0x303)
    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_integer_variables_creation() {
        // Valid cases
        let cmd = ReadMultipleIntegerVariables::new(0, 1).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 1);

        let cmd = ReadMultipleIntegerVariables::new(50, 4).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.count, 4);

        // Maximum valid count
        let cmd = ReadMultipleIntegerVariables::new(0, 237).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 237);

        // Valid case with any start variable number
        let cmd = ReadMultipleIntegerVariables::new(255, 1).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.count, 1);
    }

    #[test]
    fn test_read_multiple_integer_variables_validation() {
        // Invalid count: zero
        assert!(ReadMultipleIntegerVariables::new(0, 0).is_err());

        // Invalid count: too large
        assert!(ReadMultipleIntegerVariables::new(0, 238).is_err());
        assert!(ReadMultipleIntegerVariables::new(0, 1000).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_integer_variables_command_trait() {
        let cmd = ReadMultipleIntegerVariables::new(10, 4).expect("Valid command should not fail");
        assert_eq!(ReadMultipleIntegerVariables::command_id(), 0x303);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_integer_variables_serialization() {
        let cmd = ReadMultipleIntegerVariables::new(5, 3).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![3, 0, 0, 0]); // 3 in little-endian
    }

    // Tests for WriteMultipleIntegerVariables (0x303)
    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_integer_variables_creation() {
        // Valid cases
        let values = vec![100];
        let cmd = WriteMultipleIntegerVariables::new(0, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values, values);

        let values = vec![1, -2, 3, -4];
        let cmd = WriteMultipleIntegerVariables::new(50, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.values, values);

        // Maximum valid count
        let values: Vec<i16> = (0..237).map(|i| i16::try_from(i % 1000).unwrap_or(0)).collect();
        let cmd =
            WriteMultipleIntegerVariables::new(0, values).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values.len(), 237);

        // Valid case with any start variable number
        let values = vec![100, -200];
        let cmd = WriteMultipleIntegerVariables::new(255, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.values, values);
    }

    #[test]
    fn test_write_multiple_integer_variables_validation() {
        // Invalid count: empty
        assert!(WriteMultipleIntegerVariables::new(0, vec![]).is_err());

        // Invalid count: too large
        let large_values: Vec<i16> = vec![0; 238];
        assert!(WriteMultipleIntegerVariables::new(0, large_values).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_integer_variables_command_trait() {
        let values = vec![100, -200, 300, -400];
        let cmd =
            WriteMultipleIntegerVariables::new(10, values).expect("Valid command should not fail");
        assert_eq!(WriteMultipleIntegerVariables::command_id(), 0x303);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_integer_variables_serialization() {
        let values = vec![100, -200, 300];
        let cmd = WriteMultipleIntegerVariables::new(5, values.clone())
            .expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");

        // Expected: count (4 bytes) + values (2 bytes each)
        let mut expected = vec![3, 0, 0, 0]; // 3 in little-endian
        for value in values {
            expected.extend_from_slice(&value.to_le_bytes());
        }
        assert_eq!(serialized, expected);
    }

    // Tests for ReadMultipleDoubleVariables (0x304)
    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_double_variables_creation() {
        // Valid cases
        let cmd = ReadMultipleDoubleVariables::new(0, 1).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 1);

        let cmd = ReadMultipleDoubleVariables::new(50, 4).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.count, 4);

        // Maximum valid count
        let cmd = ReadMultipleDoubleVariables::new(0, 118).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 118);

        // Valid case with any start variable number
        let cmd = ReadMultipleDoubleVariables::new(255, 1).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.count, 1);
    }

    #[test]
    fn test_read_multiple_double_variables_validation() {
        // Invalid count: zero
        assert!(ReadMultipleDoubleVariables::new(0, 0).is_err());

        // Invalid count: too large
        assert!(ReadMultipleDoubleVariables::new(0, 119).is_err());
        assert!(ReadMultipleDoubleVariables::new(0, 1000).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_double_variables_command_trait() {
        let cmd = ReadMultipleDoubleVariables::new(10, 4).expect("Valid command should not fail");
        assert_eq!(ReadMultipleDoubleVariables::command_id(), 0x304);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_double_variables_serialization() {
        let cmd = ReadMultipleDoubleVariables::new(5, 3).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![3, 0, 0, 0]); // 3 in little-endian
    }

    // Tests for WriteMultipleDoubleVariables (0x304)
    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_double_variables_creation() {
        // Valid cases
        let values = vec![1_000_000];
        let cmd = WriteMultipleDoubleVariables::new(0, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values, values);

        let values = vec![1, -2, 3, -4];
        let cmd = WriteMultipleDoubleVariables::new(50, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.values, values);

        // Maximum valid count
        let values: Vec<i32> = (0..118).map(|i| i * 1000).collect();
        let cmd =
            WriteMultipleDoubleVariables::new(0, values).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values.len(), 118);

        // Valid case with any start variable number
        let values = vec![1_000_000, -2_000_000];
        let cmd = WriteMultipleDoubleVariables::new(255, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.values, values);
    }

    #[test]
    fn test_write_multiple_double_variables_validation() {
        // Invalid count: empty
        assert!(WriteMultipleDoubleVariables::new(0, vec![]).is_err());

        // Invalid count: too large
        let large_values: Vec<i32> = vec![0; 119];
        assert!(WriteMultipleDoubleVariables::new(0, large_values).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_double_variables_command_trait() {
        let values = vec![1_000_000, -2_000_000, 3_000_000, -4_000_000];
        let cmd =
            WriteMultipleDoubleVariables::new(10, values).expect("Valid command should not fail");
        assert_eq!(WriteMultipleDoubleVariables::command_id(), 0x304);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_double_variables_serialization() {
        let values = vec![1_000_000, -2_000_000, 3_000_000];
        let cmd = WriteMultipleDoubleVariables::new(5, values.clone())
            .expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");

        // Expected: count (4 bytes) + values (4 bytes each)
        let mut expected = vec![3, 0, 0, 0]; // 3 in little-endian
        for value in values {
            expected.extend_from_slice(&value.to_le_bytes());
        }
        assert_eq!(serialized, expected);
    }

    // Tests for ReadMultipleRealVariables (0x305)
    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_real_variables_creation() {
        // Valid cases
        let cmd = ReadMultipleRealVariables::new(0, 1).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 1);

        let cmd = ReadMultipleRealVariables::new(50, 4).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.count, 4);

        // Maximum valid count
        let cmd = ReadMultipleRealVariables::new(0, 118).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 118);

        // Valid case with any start variable number
        let cmd = ReadMultipleRealVariables::new(255, 1).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.count, 1);
    }

    #[test]
    fn test_read_multiple_real_variables_validation() {
        // Invalid count: zero
        assert!(ReadMultipleRealVariables::new(0, 0).is_err());

        // Invalid count: too large
        assert!(ReadMultipleRealVariables::new(0, 119).is_err());
        assert!(ReadMultipleRealVariables::new(0, 1000).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_real_variables_command_trait() {
        let cmd = ReadMultipleRealVariables::new(10, 4).expect("Valid command should not fail");
        assert_eq!(ReadMultipleRealVariables::command_id(), 0x305);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_real_variables_serialization() {
        let cmd = ReadMultipleRealVariables::new(5, 3).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![3, 0, 0, 0]); // 3 in little-endian
    }

    // Tests for WriteMultipleRealVariables (0x305)
    #[test]
    #[allow(clippy::expect_used, clippy::cast_precision_loss)]
    fn test_write_multiple_real_variables_creation() {
        // Valid cases
        let values = vec![1.5];
        let cmd = WriteMultipleRealVariables::new(0, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values, values);

        let values = vec![1.0, -2.5, std::f32::consts::PI, -4.0];
        let cmd = WriteMultipleRealVariables::new(50, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.values, values);

        // Maximum valid count
        let values: Vec<f32> = (0..118).map(|i| i as f32 * 1.5).collect();
        let cmd =
            WriteMultipleRealVariables::new(0, values).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values.len(), 118);

        // Valid case with any start variable number
        let values = vec![1.5, -2.75];
        let cmd = WriteMultipleRealVariables::new(255, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 255);
        assert_eq!(cmd.values, values);
    }

    #[test]
    fn test_write_multiple_real_variables_validation() {
        // Invalid count: empty
        assert!(WriteMultipleRealVariables::new(0, vec![]).is_err());

        // Invalid count: too large
        let large_values: Vec<f32> = vec![0.0; 119];
        assert!(WriteMultipleRealVariables::new(0, large_values).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_real_variables_command_trait() {
        let values = vec![1.5, -2.75, std::f32::consts::PI, -4.0];
        let cmd =
            WriteMultipleRealVariables::new(10, values).expect("Valid command should not fail");
        assert_eq!(WriteMultipleRealVariables::command_id(), 0x305);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_real_variables_serialization() {
        let values = vec![1.5, -2.75, std::f32::consts::PI];
        let cmd = WriteMultipleRealVariables::new(5, values.clone())
            .expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");

        // Expected: count (4 bytes) + values (4 bytes each)
        let mut expected = vec![3, 0, 0, 0]; // 3 in little-endian
        for value in values {
            expected.extend_from_slice(&value.to_le_bytes());
        }
        assert_eq!(serialized, expected);
    }
}

/// Read multiple character type variables (S) command (0x306)
#[derive(Debug, Clone)]
pub struct ReadMultipleCharacterVariables {
    pub start_variable_number: u16, // Support extended variable settings (0-99 for standard)
    pub count: u32,                 // Number of S variable data (max 29)
}

impl ReadMultipleCharacterVariables {
    /// # Errors
    ///
    /// Returns an error if count is 0 or exceeds 29
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate count (max 29, must be > 0)
        if count == 0 || count > 29 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-29)"
            )));
        }
        Ok(Self { start_variable_number, count })
    }
}

impl Command for ReadMultipleCharacterVariables {
    type Response = Vec<[u8; 16]>; // Array of 16-byte S variable values
    fn command_id() -> u16 {
        0x306
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple character type variables (S) command (0x306)
#[derive(Debug, Clone)]
pub struct WriteMultipleCharacterVariables {
    pub start_variable_number: u16, // Support extended variable settings (0-99 for standard)
    pub values: Vec<[u8; 16]>,      // S variable values to write (16 bytes each)
}

impl WriteMultipleCharacterVariables {
    /// # Errors
    ///
    /// Returns an error if values count is 0 or exceeds 29
    pub fn new(start_variable_number: u16, values: Vec<[u8; 16]>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 29, must be > 0)
        if count == 0 || count > 29 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable_number} (must be 1-29)"
            )));
        }
        Ok(Self { start_variable_number, values })
    }
}

impl Command for WriteMultipleCharacterVariables {
    type Response = ();
    fn command_id() -> u16 {
        0x306
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Fixed to 0 for plural commands
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;
        let mut payload = count.to_le_bytes().to_vec();
        for value in &self.values {
            payload.extend_from_slice(value);
        }
        Ok(payload)
    }
}

#[cfg(test)]
mod character_variable_tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_multiple_character_variables_construction() {
        // Valid construction
        let cmd = ReadMultipleCharacterVariables::new(0, 1);
        assert!(cmd.is_ok());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 1);

        let cmd = ReadMultipleCharacterVariables::new(50, 29);
        assert!(cmd.is_ok());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.start_variable_number, 50);
        assert_eq!(cmd.count, 29);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_multiple_character_variables_validation() {
        // Invalid count: 0
        let result = ReadMultipleCharacterVariables::new(0, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid count: 0"));

        // Invalid count: > 29
        let result = ReadMultipleCharacterVariables::new(0, 30);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid count: 30"));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_character_variables_command_trait() {
        let cmd =
            ReadMultipleCharacterVariables::new(10, 5).expect("Valid command should not fail");
        assert_eq!(ReadMultipleCharacterVariables::command_id(), 0x306);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_character_variables_serialization() {
        let cmd = ReadMultipleCharacterVariables::new(5, 3).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![3, 0, 0, 0]); // 3 in little-endian
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::similar_names)]
    fn test_write_multiple_character_variables_construction() {
        // Valid construction
        let mut value1 = [0u8; 16];
        value1[..5].copy_from_slice(b"Hello");
        let mut value2 = [0u8; 16];
        value2[..5].copy_from_slice(b"World");
        let values = vec![value1, value2];

        let cmd = WriteMultipleCharacterVariables::new(0, values.clone());
        assert!(cmd.is_ok());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values.len(), 2);
        assert_eq!(cmd.values, values);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_write_multiple_character_variables_validation() {
        // Invalid count: 0
        let result = WriteMultipleCharacterVariables::new(0, vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid count: 0"));

        // Invalid count: > 29
        let values: Vec<[u8; 16]> = (0..30).map(|_| [0u8; 16]).collect();
        let result = WriteMultipleCharacterVariables::new(0, values);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid count: 30"));
    }

    #[test]
    #[allow(clippy::expect_used, clippy::similar_names)]
    fn test_write_multiple_character_variables_command_trait() {
        let mut value1 = [0u8; 16];
        value1[..4].copy_from_slice(b"Test");
        let values = vec![value1];
        let cmd = WriteMultipleCharacterVariables::new(10, values)
            .expect("Valid command should not fail");
        assert_eq!(WriteMultipleCharacterVariables::command_id(), 0x306);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::expect_used, clippy::similar_names)]
    fn test_write_multiple_character_variables_serialization() {
        let mut value1 = [0u8; 16];
        value1[..5].copy_from_slice(b"Hello");
        let mut value2 = [0u8; 16];
        value2[..5].copy_from_slice(b"World");
        let values = vec![value1, value2];
        let cmd = WriteMultipleCharacterVariables::new(5, values.clone())
            .expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");

        // Expected: count (4 bytes) + values (16 bytes each)
        let mut expected = vec![2, 0, 0, 0]; // 2 in little-endian
        for value in values {
            expected.extend_from_slice(&value);
        }
        assert_eq!(serialized, expected);
    }
}
