//! Variable command definitions for HSES protocol

use crate::{HsesPayload, commands::Command, error::ProtocolError};
use std::marker::PhantomData;

/// Trait for variable command IDs
pub trait VariableCommandId {
    fn command_id() -> u16;
}

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

impl VariableCommandId for String {
    fn command_id() -> u16 {
        0x7e
    }
}

/// Command ID and validation for multiple variable operations
pub trait MultipleVariableCommandId {
    /// Returns the command ID for multiple variable operations (0x302-0x306)
    fn multiple_command_id() -> u16;

    /// Returns the size in bytes of a single element
    fn element_size() -> usize;

    /// Returns the maximum count for this variable type
    fn max_count() -> u32;

    /// Validates the count for this variable type
    ///
    /// # Errors
    /// Returns `ProtocolError` if count is invalid
    fn validate_count(count: u32) -> Result<(), ProtocolError>;
}

impl MultipleVariableCommandId for u8 {
    fn multiple_command_id() -> u16 {
        0x302
    }
    fn element_size() -> usize {
        1
    }
    fn max_count() -> u32 {
        474
    }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > Self::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                Self::max_count()
            )));
        }
        if !count.is_multiple_of(2) {
            return Err(ProtocolError::InvalidMessage(format!(
                "Count must be multiple of 2: {count}"
            )));
        }
        Ok(())
    }
}

impl MultipleVariableCommandId for i16 {
    fn multiple_command_id() -> u16 {
        0x303
    }
    fn element_size() -> usize {
        2
    }
    fn max_count() -> u32 {
        237
    }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > Self::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                Self::max_count()
            )));
        }
        Ok(())
    }
}

impl MultipleVariableCommandId for i32 {
    fn multiple_command_id() -> u16 {
        0x304
    }
    fn element_size() -> usize {
        4
    }
    fn max_count() -> u32 {
        118
    }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > Self::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                Self::max_count()
            )));
        }
        Ok(())
    }
}

impl MultipleVariableCommandId for f32 {
    fn multiple_command_id() -> u16 {
        0x305
    }
    fn element_size() -> usize {
        4
    }
    fn max_count() -> u32 {
        118
    }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > Self::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                Self::max_count()
            )));
        }
        Ok(())
    }
}

// Implementation for String (S variables - 16 bytes each)
impl MultipleVariableCommandId for String {
    fn multiple_command_id() -> u16 {
        0x306
    }
    fn element_size() -> usize {
        16
    }
    fn max_count() -> u32 {
        29
    }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > Self::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                Self::max_count()
            )));
        }
        Ok(())
    }
}

/// Trait for deserializing multiple variable responses
pub trait MultipleVariableResponse: Sized + MultipleVariableCommandId {
    /// Parse a single element from byte slice at given offset
    ///
    /// # Errors
    /// Returns `ProtocolError` if parsing fails
    fn parse_element(
        data: &[u8],
        offset: usize,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError>;

    /// Deserialize multiple variables from response data
    ///
    /// # Errors
    /// Returns `ProtocolError` if deserialization fails
    fn deserialize_multiple(
        data: &[u8],
        expected_count: u32,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<Self>, ProtocolError> {
        // Common validation logic
        if data.len() < 4 {
            return Err(ProtocolError::Deserialization(format!(
                "Response too short: {} bytes (need at least 4)",
                data.len()
            )));
        }

        let response_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if response_count != expected_count {
            return Err(ProtocolError::Deserialization(format!(
                "Count mismatch: expected {expected_count}, got {response_count}"
            )));
        }

        let element_size = Self::element_size();
        let expected_len = 4 + (expected_count as usize * element_size);
        if data.len() != expected_len {
            return Err(ProtocolError::Deserialization(format!(
                "Invalid response length: got {} bytes, expected {expected_len}",
                data.len()
            )));
        }

        // Parse elements
        let mut values = Vec::with_capacity(expected_count as usize);
        for i in 0..expected_count as usize {
            let offset = 4 + i * element_size;
            values.push(Self::parse_element(data, offset, encoding)?);
        }
        Ok(values)
    }
}

impl MultipleVariableResponse for u8 {
    fn parse_element(
        data: &[u8],
        offset: usize,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        let byte_slice = &data[offset..=offset];
        Self::deserialize(byte_slice, encoding)
    }
}

impl MultipleVariableResponse for i16 {
    fn parse_element(
        data: &[u8],
        offset: usize,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        let byte_slice = &data[offset..offset + 2];
        Self::deserialize(byte_slice, encoding)
    }
}

impl MultipleVariableResponse for i32 {
    fn parse_element(
        data: &[u8],
        offset: usize,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        let byte_slice = &data[offset..offset + 4];
        Self::deserialize(byte_slice, encoding)
    }
}

impl MultipleVariableResponse for f32 {
    fn parse_element(
        data: &[u8],
        offset: usize,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        let byte_slice = &data[offset..offset + 4];
        Self::deserialize(byte_slice, encoding)
    }
}

impl MultipleVariableResponse for String {
    fn parse_element(
        data: &[u8],
        offset: usize,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        let byte_array = &data[offset..offset + 16];
        Self::deserialize(byte_array, encoding)
    }
}

/// Read multiple variables command (generic)
/// `T` may be `f32`, which does not implement `Eq`, so we only derive `PartialEq`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct ReadMultipleVariables<T: MultipleVariableCommandId + PartialEq> {
    pub start_variable_number: u16,
    pub count: u32,
    pub _phantom: PhantomData<T>,
}

impl<T: MultipleVariableCommandId + PartialEq> ReadMultipleVariables<T> {
    /// Create a new `ReadMultipleVariables` command
    ///
    /// # Errors
    /// Returns an error if count validation fails
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        T::validate_count(count)?;
        Ok(Self { start_variable_number, count, _phantom: PhantomData })
    }
}

impl<T: MultipleVariableCommandId + PartialEq> Command for ReadMultipleVariables<T> {
    type Response = Vec<T>;
    fn command_id() -> u16 {
        T::multiple_command_id()
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0 // Fixed to 0 for plural commands
    }
    fn service(&self) -> u8 {
        0x33 // Read plural data
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple variables command (generic)
/// `T` may be `f32`, which does not implement `Eq`, so we only derive `PartialEq`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct WriteMultipleVariables<T: MultipleVariableCommandId + PartialEq> {
    pub start_variable_number: u16,
    pub values: Vec<T>,
}

/// Write multiple string variables command with encoding support
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteMultipleStringVariables {
    pub start_variable_number: u16,
    pub values: Vec<String>,
    pub text_encoding: crate::encoding::TextEncoding,
}

impl<T: MultipleVariableCommandId + PartialEq + Clone + HsesPayload> WriteMultipleVariables<T> {
    /// Create a new `WriteMultipleVariables` command
    ///
    /// # Errors
    /// Returns an error if count validation fails
    pub fn new(start_variable_number: u16, values: Vec<T>) -> Result<Self, ProtocolError> {
        let count = u32::try_from(values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!("Values count {} exceeds u32::MAX", values.len()))
        })?;
        T::validate_count(count)?;
        Ok(Self { start_variable_number, values })
    }
}

// Type-specific Command implementations for WriteMultipleVariables
impl Command for WriteMultipleVariables<u8> {
    type Response = ();
    fn command_id() -> u16 {
        0x302
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x34
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;

        // Validate count using the type's max_count
        if count == 0 || count > u8::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                u8::max_count()
            )));
        }

        let mut payload = count.to_le_bytes().to_vec();
        let serialized_values = self.values.serialize(crate::encoding::TextEncoding::Utf8)?;
        payload.extend_from_slice(&serialized_values);
        Ok(payload)
    }
}

impl Command for WriteMultipleVariables<i16> {
    type Response = ();
    fn command_id() -> u16 {
        0x303
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x34
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;

        // Validate count using the type's max_count
        if count == 0 || count > i16::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                i16::max_count()
            )));
        }

        let mut payload = count.to_le_bytes().to_vec();
        let serialized_values = self.values.serialize(crate::encoding::TextEncoding::Utf8)?;
        payload.extend_from_slice(&serialized_values);
        Ok(payload)
    }
}

impl Command for WriteMultipleVariables<i32> {
    type Response = ();
    fn command_id() -> u16 {
        0x304
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x34
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;

        // Validate count using the type's max_count
        if count == 0 || count > i32::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                i32::max_count()
            )));
        }

        let mut payload = count.to_le_bytes().to_vec();
        let serialized_values = self.values.serialize(crate::encoding::TextEncoding::Utf8)?;
        payload.extend_from_slice(&serialized_values);
        Ok(payload)
    }
}

impl Command for WriteMultipleVariables<f32> {
    type Response = ();
    fn command_id() -> u16 {
        0x305
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x34
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;

        // Validate count using the type's max_count
        if count == 0 || count > f32::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                f32::max_count()
            )));
        }

        let mut payload = count.to_le_bytes().to_vec();
        let serialized_values = self.values.serialize(crate::encoding::TextEncoding::Utf8)?;
        payload.extend_from_slice(&serialized_values);
        Ok(payload)
    }
}

impl Command for WriteMultipleStringVariables {
    type Response = ();
    fn command_id() -> u16 {
        0x306
    }
    fn instance(&self) -> u16 {
        self.start_variable_number
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x34
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                self.values.len()
            ))
        })?;

        // Validate count for String variables (1-29)
        if count == 0 || count > String::max_count() {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-{})",
                String::max_count()
            )));
        }

        let mut payload = count.to_le_bytes().to_vec();
        let serialized_values = self.values.serialize(self.text_encoding)?;
        payload.extend_from_slice(&serialized_values);
        Ok(payload)
    }
}

/// `T` may be `f32`, which does not implement `Eq`, so we only derive `PartialEq`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct ReadVariable<T: HsesPayload + VariableCommandId + PartialEq> {
    pub index: u16, // Support extended variable settings (0-999)
    pub _phantom: PhantomData<T>,
}

impl<T: HsesPayload + VariableCommandId + PartialEq> Command for ReadVariable<T> {
    type Response = T;
    fn command_id() -> u16 {
        T::command_id()
    }
    fn instance(&self) -> u16 {
        self.index
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x0E // Get_Attribute_Single
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(vec![])
    }
}

/// `T` may be `f32`, which does not implement `Eq`, so we only derive `PartialEq`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct WriteVariable<T: HsesPayload + VariableCommandId + PartialEq> {
    pub index: u16, // Support extended variable settings (0-999)
    pub value: T,
}

/// Write string variable command with encoding support
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteStringVar {
    pub index: u16, // Support extended variable settings (0-999)
    pub value: String,
    pub text_encoding: crate::encoding::TextEncoding,
}

impl<T: HsesPayload + VariableCommandId + PartialEq> Command for WriteVariable<T> {
    type Response = ();
    fn command_id() -> u16 {
        T::command_id()
    }
    fn instance(&self) -> u16 {
        self.index
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.value.serialize(crate::encoding::TextEncoding::Utf8)
    }
}

impl Command for WriteStringVar {
    type Response = ();
    fn command_id() -> u16 {
        String::command_id()
    }
    fn instance(&self) -> u16 {
        self.index
    }
    fn attribute(&self) -> u8 {
        0
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.value.serialize(self.text_encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Allow expect in tests as they should not fail with valid inputs
    #[allow(clippy::expect_used)]
    #[test]
    fn test_read_variable_construction() {
        let cmd = ReadVariable::<u8> { index: 10, _phantom: PhantomData };
        assert_eq!(cmd.index, 10);
    }

    #[test]
    fn test_write_variable_construction() {
        let cmd = WriteVariable::<u8> { index: 10, value: 42 };
        assert_eq!(cmd.index, 10);
        assert_eq!(cmd.value, 42);
    }

    #[test]
    fn test_variable_command_ids() {
        assert_eq!(u8::command_id(), 0x7a);
        assert_eq!(i16::command_id(), 0x7b);
        assert_eq!(i32::command_id(), 0x7c);
        assert_eq!(f32::command_id(), 0x7d);
        assert_eq!(String::command_id(), 0x7e);
    }

    #[test]
    fn test_read_variable_command_trait() {
        let cmd = ReadVariable::<u8> { index: 5, _phantom: PhantomData };
        assert_eq!(ReadVariable::<u8>::command_id(), 0x7a);
        assert_eq!(cmd.instance(), 5);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x0E);
    }

    #[test]
    fn test_write_variable_command_trait() {
        let cmd = WriteVariable::<u8> { index: 5, value: 100 };
        assert_eq!(WriteVariable::<u8>::command_id(), 0x7a);
        assert_eq!(cmd.instance(), 5);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x10);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_variable_serialization() {
        let cmd = WriteVariable::<u8> { index: 5, value: 100 };
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![100]);
    }

    #[test]
    fn test_multiple_variable_command_ids() {
        assert_eq!(u8::multiple_command_id(), 0x302);
        assert_eq!(i16::multiple_command_id(), 0x303);
        assert_eq!(i32::multiple_command_id(), 0x304);
        assert_eq!(f32::multiple_command_id(), 0x305);
        assert_eq!(String::multiple_command_id(), 0x306);
    }

    #[test]
    fn test_multiple_variable_element_sizes() {
        assert_eq!(u8::element_size(), 1);
        assert_eq!(i16::element_size(), 2);
        assert_eq!(i32::element_size(), 4);
        assert_eq!(f32::element_size(), 4);
        assert_eq!(String::element_size(), 16);
    }

    #[test]
    fn test_multiple_variable_max_counts() {
        assert_eq!(u8::max_count(), 474);
        assert_eq!(i16::max_count(), 237);
        assert_eq!(i32::max_count(), 118);
        assert_eq!(f32::max_count(), 118);
        assert_eq!(String::max_count(), 29);
    }

    #[test]
    fn test_multiple_variable_count_validation() {
        // Valid counts
        assert!(u8::validate_count(2).is_ok());
        assert!(i16::validate_count(1).is_ok());
        assert!(i32::validate_count(1).is_ok());
        assert!(f32::validate_count(1).is_ok());
        assert!(String::validate_count(1).is_ok());

        // Invalid counts - too small
        assert!(u8::validate_count(0).is_err());
        assert!(i16::validate_count(0).is_err());
        assert!(i32::validate_count(0).is_err());
        assert!(f32::validate_count(0).is_err());
        assert!(String::validate_count(0).is_err());

        // Invalid counts - too large
        assert!(u8::validate_count(475).is_err());
        assert!(i16::validate_count(238).is_err());
        assert!(i32::validate_count(119).is_err());
        assert!(f32::validate_count(119).is_err());
        assert!(String::validate_count(30).is_err());

        // Invalid count for u8 - not multiple of 2
        assert!(u8::validate_count(3).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_variables_construction() {
        let cmd = ReadMultipleVariables::<u8>::new(0, 2).expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.count, 2);
    }

    #[test]
    fn test_read_multiple_variables_validation() {
        // Valid count
        assert!(ReadMultipleVariables::<u8>::new(0, 2).is_ok());

        // Invalid count - not multiple of 2
        assert!(ReadMultipleVariables::<u8>::new(0, 3).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_variables_command_trait() {
        let cmd = ReadMultipleVariables::<u8>::new(10, 2).expect("Valid command should not fail");
        assert_eq!(ReadMultipleVariables::<u8>::command_id(), 0x302);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_read_multiple_variables_serialization() {
        let cmd = ReadMultipleVariables::<u8>::new(5, 2).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![2, 0, 0, 0]); // 2 in little-endian
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_variables_construction() {
        let values = vec![1u8, 2u8];
        let cmd = WriteMultipleVariables::<u8>::new(0, values.clone())
            .expect("Valid command should not fail");
        assert_eq!(cmd.start_variable_number, 0);
        assert_eq!(cmd.values, values);
    }

    #[test]
    fn test_write_multiple_variables_validation() {
        // Valid values
        let values = vec![1u8, 2u8];
        assert!(WriteMultipleVariables::<u8>::new(0, values).is_ok());

        // Invalid values - not multiple of 2
        let values = vec![1u8, 2u8, 3u8];
        assert!(WriteMultipleVariables::<u8>::new(0, values).is_err());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_variables_command_trait() {
        let values = vec![1u8, 2u8];
        let cmd =
            WriteMultipleVariables::<u8>::new(10, values).expect("Valid command should not fail");
        assert_eq!(WriteMultipleVariables::<u8>::command_id(), 0x302);
        assert_eq!(cmd.instance(), 10);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_write_multiple_variables_serialization() {
        let values = vec![1u8, 2u8];
        let cmd =
            WriteMultipleVariables::<u8>::new(5, values).expect("Valid command should not fail");
        let serialized = cmd.serialize().expect("Serialization should not fail");
        assert_eq!(serialized, vec![2, 0, 0, 0, 1, 2]); // count + values
    }

    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]
    fn test_multiple_variable_response_parse_element() {
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8];

        // Test u8 parsing
        assert_eq!(u8::parse_element(&data, 0, crate::encoding::TextEncoding::Utf8).unwrap(), 1);
        assert_eq!(u8::parse_element(&data, 1, crate::encoding::TextEncoding::Utf8).unwrap(), 2);

        // Test i16 parsing
        assert_eq!(
            i16::parse_element(&data, 0, crate::encoding::TextEncoding::Utf8).unwrap(),
            0x0201
        ); // little-endian
        assert_eq!(
            i16::parse_element(&data, 2, crate::encoding::TextEncoding::Utf8).unwrap(),
            0x0403
        );

        // Test i32 parsing
        assert_eq!(
            i32::parse_element(&data, 0, crate::encoding::TextEncoding::Utf8).unwrap(),
            0x0403_0201
        ); // little-endian
        assert_eq!(
            i32::parse_element(&data, 4, crate::encoding::TextEncoding::Utf8).unwrap(),
            0x0807_0605
        );

        // Test f32 parsing
        let f32_data = [0x00, 0x00, 0x80, 0x3f]; // 1.0f32 in little-endian
        assert_eq!(
            f32::parse_element(&f32_data, 0, crate::encoding::TextEncoding::Utf8).unwrap(),
            1.0f32
        );

        // Test String parsing
        let mut array_data = [0u8; 32];
        // Test data: "Hello" + null terminator + padding
        array_data[0..5].copy_from_slice(b"Hello");
        // Test data: "World" + null terminator + padding
        array_data[16..21].copy_from_slice(b"World");

        let result =
            String::parse_element(&array_data, 0, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(result, "Hello");

        let result =
            String::parse_element(&array_data, 16, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(result, "World");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_multiple_variable_response_deserialize_multiple() {
        // Test u8 deserialization
        let mut data = vec![2, 0, 0, 0]; // count = 2
        data.extend_from_slice(&[1, 2]); // values

        let result =
            u8::deserialize_multiple(&data, 2, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(result, vec![1, 2]);

        // Test i16 deserialization
        let mut data = vec![2, 0, 0, 0]; // count = 2
        data.extend_from_slice(&[1, 0, 2, 0]); // values in little-endian

        let result =
            i16::deserialize_multiple(&data, 2, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(result, vec![1, 2]);

        // Test error cases
        let short_data = vec![1, 0, 0, 0]; // count = 1, but no data
        assert!(
            u8::deserialize_multiple(&short_data, 1, crate::encoding::TextEncoding::Utf8).is_err()
        );

        let wrong_count_data = vec![2, 0, 0, 0, 1]; // count = 2, but only 1 value
        assert!(
            u8::deserialize_multiple(&wrong_count_data, 2, crate::encoding::TextEncoding::Utf8)
                .is_err()
        );
    }
}
