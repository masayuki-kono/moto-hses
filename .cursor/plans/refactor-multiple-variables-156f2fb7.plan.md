<!-- 156f2fb7-ccce-420a-818b-92daace59491 1584b9bf-8ae1-4082-b075-f68d3e15d104 -->
# Multiple Variable Read/Write Interface Refactoring

## Overview

Refactor the type-specific multiple variable read/write methods (10 methods: `read_multiple_byte_variables`, `write_multiple_byte_variables`, etc.) in `protocol.rs` into generic `read_multiple_variables<T>` and `write_multiple_variables<T>` methods. Type-specific convenience methods will be moved to `convenience.rs`.

### Current State

**File**: `moto-hses-client/src/protocol.rs`

- 10 type-specific methods with significant code duplication (~500 lines)
- Each method handles: command creation, sending, response parsing, count validation, byte array conversion
- Types: u8 (B), i16 (I), i32 (D), f32 (R), String (S)

**File**: `moto-hses-proto/src/commands/variable.rs`

- 10 command structs with similar implementations
- Each struct has: validation logic, Command trait implementation, serialization

### Target State

**File**: `moto-hses-proto/src/commands/variable.rs`

- New `MultipleVariableCommandId` trait
- Generic `ReadMultipleVariables<T>` and `WriteMultipleVariables<T>` structs
- Response deserialization trait `MultipleVariableResponse<T>`

**File**: `moto-hses-client/src/protocol.rs`

- Generic `read_multiple_variables<T>` and `write_multiple_variables<T>` methods (~100 lines)
- Special methods for string variables with encoding support

**File**: `moto-hses-client/src/convenience.rs`

- Type-specific wrapper methods for ease of use

## Implementation Steps

### 1. Add MultipleVariableCommandId Trait to Proto Crate

**File**: `moto-hses-proto/src/commands/variable.rs`

Add new trait after the existing `VariableCommandId` trait:

```rust
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
    /// Returns ProtocolError if count is invalid
    fn validate_count(count: u32) -> Result<(), ProtocolError>;
}
```

Implement `MultipleVariableCommandId` for each type:

```rust
impl MultipleVariableCommandId for u8 {
    fn multiple_command_id() -> u16 { 0x302 }
    fn element_size() -> usize { 1 }
    fn max_count() -> u32 { 474 }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
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
        Ok(())
    }
}

impl MultipleVariableCommandId for i16 {
    fn multiple_command_id() -> u16 { 0x303 }
    fn element_size() -> usize { 2 }
    fn max_count() -> u32 { 237 }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-237)"
            )));
        }
        Ok(())
    }
}

impl MultipleVariableCommandId for i32 {
    fn multiple_command_id() -> u16 { 0x304 }
    fn element_size() -> usize { 4 }
    fn max_count() -> u32 { 118 }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > 118 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-118)"
            )));
        }
        Ok(())
    }
}

impl MultipleVariableCommandId for f32 {
    fn multiple_command_id() -> u16 { 0x305 }
    fn element_size() -> usize { 4 }
    fn max_count() -> u32 { 118 }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > 118 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-118)"
            )));
        }
        Ok(())
    }
}

// Implementation for [u8; 16] (S variables - 16 bytes each)
impl MultipleVariableCommandId for [u8; 16] {
    fn multiple_command_id() -> u16 { 0x306 }
    fn element_size() -> usize { 16 }
    fn max_count() -> u32 { 29 }
    fn validate_count(count: u32) -> Result<(), ProtocolError> {
        if count == 0 || count > 29 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-29)"
            )));
        }
        Ok(())
    }
}
```

### 2. Add Generic Command Structs to Proto Crate

**File**: `moto-hses-proto/src/commands/variable.rs`

Add generic command structs:

```rust
/// Read multiple variables command (generic)
#[derive(Debug, Clone)]
pub struct ReadMultipleVariables<T: MultipleVariableCommandId> {
    pub start_variable_number: u16,
    pub count: u32,
    pub _phantom: PhantomData<T>,
}

impl<T: MultipleVariableCommandId> ReadMultipleVariables<T> {
    /// Create a new ReadMultipleVariables command
    ///
    /// # Errors
    /// Returns an error if count validation fails
    pub fn new(start_variable_number: u16, count: u32) -> Result<Self, ProtocolError> {
        T::validate_count(count)?;
        Ok(Self {
            start_variable_number,
            count,
            _phantom: PhantomData,
        })
    }
}

impl<T: MultipleVariableCommandId> Command for ReadMultipleVariables<T> {
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
#[derive(Debug, Clone)]
pub struct WriteMultipleVariables<T: MultipleVariableCommandId> {
    pub start_variable_number: u16,
    pub values: Vec<T>,
}

impl<T: MultipleVariableCommandId + Clone> WriteMultipleVariables<T> {
    /// Create a new WriteMultipleVariables command
    ///
    /// # Errors
    /// Returns an error if count validation fails
    pub fn new(start_variable_number: u16, values: Vec<T>) -> Result<Self, ProtocolError> {
        let count = u32::try_from(values.len()).map_err(|_| {
            ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX",
                values.len()
            ))
        })?;
        T::validate_count(count)?;
        Ok(Self {
            start_variable_number,
            values,
        })
    }
}

impl<T: MultipleVariableCommandId> Command for WriteMultipleVariables<T> {
    type Response = ();
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
        0x34 // Write plural data
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Implementation depends on type - will be specialized
        unimplemented!("Use type-specific serialization")
    }
}
```

### 3. Add Response Deserialization Trait to Proto Crate

**File**: `moto-hses-proto/src/commands/variable.rs`

Add trait for deserializing multiple variable responses with common validation logic:

```rust
/// Trait for deserializing multiple variable responses
pub trait MultipleVariableResponse: Sized + MultipleVariableCommandId {
    /// Parse a single element from byte slice at given offset
    ///
    /// # Errors
    /// Returns ProtocolError if parsing fails
    fn parse_element(data: &[u8], offset: usize) -> Result<Self, ProtocolError>;
    
    /// Deserialize multiple variables from response data
    ///
    /// # Errors
    /// Returns ProtocolError if deserialization fails
    fn deserialize_multiple(data: &[u8], expected_count: u32) -> Result<Vec<Self>, ProtocolError> {
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
            values.push(Self::parse_element(data, offset)?);
        }
        Ok(values)
    }
}

impl MultipleVariableResponse for u8 {
    fn parse_element(data: &[u8], offset: usize) -> Result<Self, ProtocolError> {
        Ok(data[offset])
    }
}

impl MultipleVariableResponse for i16 {
    fn parse_element(data: &[u8], offset: usize) -> Result<Self, ProtocolError> {
        Ok(i16::from_le_bytes([data[offset], data[offset + 1]]))
    }
}

impl MultipleVariableResponse for i32 {
    fn parse_element(data: &[u8], offset: usize) -> Result<Self, ProtocolError> {
        Ok(i32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]))
    }
}

impl MultipleVariableResponse for f32 {
    fn parse_element(data: &[u8], offset: usize) -> Result<Self, ProtocolError> {
        Ok(f32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]))
    }
}

impl MultipleVariableResponse for [u8; 16] {
    fn parse_element(data: &[u8], offset: usize) -> Result<Self, ProtocolError> {
        let mut array = [0u8; 16];
        array.copy_from_slice(&data[offset..offset + 16]);
        Ok(array)
    }
}
```

### 4. Add Generic Methods to Client

**File**: `moto-hses-client/src/protocol.rs`

Add generic methods for reading and writing multiple variables (encoding-agnostic):

```rust
/// Read multiple variables (generic)
///
/// This method is encoding-agnostic and works with raw byte arrays for S variables.
/// For string handling with encoding support, use convenience methods in convenience.rs.
///
/// # Errors
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_variables<T>(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<T>, ClientError>
where
    T: MultipleVariableCommandId + MultipleVariableResponse,
{
    let command = ReadMultipleVariables::<T>::new(start_variable_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    T::deserialize_multiple(&response, count).map_err(ClientError::from)
}

/// Write multiple variables (generic)
///
/// This method is encoding-agnostic and works with raw byte arrays for S variables.
/// For string handling with encoding support, use convenience methods in convenience.rs.
///
/// # Errors
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_variables<T>(
    &self,
    start_variable_number: u16,
    values: Vec<T>,
) -> Result<(), ClientError>
where
    T: MultipleVariableCommandId + Clone,
{
    let command = WriteMultipleVariables::<T>::new(start_variable_number, values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

**Note**: The generic methods do NOT handle text encoding. String encoding/decoding is handled in convenience.rs wrapper methods.

### 5. Add Convenience Wrapper Methods

**File**: `moto-hses-client/src/convenience.rs`

Add type-specific wrapper methods:

```rust
// Multiple variable operations

/// Read multiple byte variables (B)
///
/// # Errors
/// Returns an error if communication fails
pub async fn read_multiple_u8(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<u8>, ClientError> {
    self.read_multiple_variables::<u8>(start_variable_number, count).await
}

/// Write multiple byte variables (B)
///
/// # Errors
/// Returns an error if communication fails
pub async fn write_multiple_u8(
    &self,
    start_variable_number: u16,
    values: Vec<u8>,
) -> Result<(), ClientError> {
    self.write_multiple_variables(start_variable_number, values).await
}

/// Read multiple integer variables (I)
///
/// # Errors
/// Returns an error if communication fails
pub async fn read_multiple_i16(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<i16>, ClientError> {
    self.read_multiple_variables::<i16>(start_variable_number, count).await
}

/// Write multiple integer variables (I)
///
/// # Errors
/// Returns an error if communication fails
pub async fn write_multiple_i16(
    &self,
    start_variable_number: u16,
    values: Vec<i16>,
) -> Result<(), ClientError> {
    self.write_multiple_variables(start_variable_number, values).await
}

/// Read multiple double precision integer variables (D)
///
/// # Errors
/// Returns an error if communication fails
pub async fn read_multiple_i32(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<i32>, ClientError> {
    self.read_multiple_variables::<i32>(start_variable_number, count).await
}

/// Write multiple double precision integer variables (D)
///
/// # Errors
/// Returns an error if communication fails
pub async fn write_multiple_i32(
    &self,
    start_variable_number: u16,
    values: Vec<i32>,
) -> Result<(), ClientError> {
    self.write_multiple_variables(start_variable_number, values).await
}

/// Read multiple real type variables (R)
///
/// # Errors
/// Returns an error if communication fails
pub async fn read_multiple_f32(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<f32>, ClientError> {
    self.read_multiple_variables::<f32>(start_variable_number, count).await
}

/// Write multiple real type variables (R)
///
/// # Errors
/// Returns an error if communication fails
pub async fn write_multiple_f32(
    &self,
    start_variable_number: u16,
    values: Vec<f32>,
) -> Result<(), ClientError> {
    self.write_multiple_variables(start_variable_number, values).await
}

/// Read multiple string variables (S) with encoding support
///
/// Reads raw byte arrays using `read_multiple_variables<[u8; 16]>` and converts them to strings
/// using the client's text encoding configuration.
///
/// # Errors
/// Returns an error if communication fails
pub async fn read_multiple_strings(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<String>, ClientError> {
    // Read raw byte arrays from protocol layer
    let byte_arrays = self.read_multiple_variables::<[u8; 16]>(start_variable_number, count).await?;
    
    // Convert byte arrays to strings with encoding
    let mut strings = Vec::with_capacity(byte_arrays.len());
    for byte_array in byte_arrays {
        // Find null terminator
        let trimmed_bytes = byte_array
            .iter()
            .position(|&b| b == 0)
            .map_or(&byte_array[..], |pos| &byte_array[..pos]);
        
        // Decode using client's text encoding
        let string = moto_hses_proto::encoding_utils::decode_string_with_fallback(
            trimmed_bytes,
            self.config.text_encoding,
        );
        strings.push(string);
    }
    
    Ok(strings)
}

/// Write multiple string variables (S) with encoding support
///
/// Converts strings to raw byte arrays using the client's text encoding configuration,
/// then writes them using `write_multiple_variables<[u8; 16]>`.
///
/// # Errors
/// Returns an error if communication fails or if any string exceeds 16 bytes when encoded
pub async fn write_multiple_strings(
    &self,
    start_variable_number: u16,
    values: Vec<String>,
) -> Result<(), ClientError> {
    // Convert strings to byte arrays with encoding
    let mut byte_arrays = Vec::with_capacity(values.len());
    for (i, value) in values.iter().enumerate() {
        let encoded_bytes = moto_hses_proto::encoding_utils::encode_string(
            value,
            self.config.text_encoding,
        );
        
        if encoded_bytes.len() > 16 {
            return Err(ClientError::SystemError(format!(
                "String at index {i} exceeds 16 bytes when encoded: {} bytes",
                encoded_bytes.len()
            )));
        }
        
        let mut byte_array = [0u8; 16];
        byte_array[..encoded_bytes.len()].copy_from_slice(&encoded_bytes);
        byte_arrays.push(byte_array);
    }
    
    // Write raw byte arrays using protocol layer
    self.write_multiple_variables(start_variable_number, byte_arrays).await
}
```

### 6. Remove Old Methods from Protocol.rs

**File**: `moto-hses-client/src/protocol.rs`

Remove the following 10 methods:

- `read_multiple_byte_variables` (lines 82-119)
- `write_multiple_byte_variables` (lines 121-139)
- `read_multiple_integer_variables` (lines 141-196)
- `write_multiple_integer_variables` (lines 198-216)
- `read_multiple_double_variables` (lines 1230-1290)
- `write_multiple_double_variables` (lines 1292-1310)
- `read_multiple_real_variables` (lines 1312-1372)
- `write_multiple_real_variables` (lines 1374-1392)
- `read_multiple_character_variables` (lines 1394-1461)
- `write_multiple_character_variables` (lines 1463-1498)

### 7. Remove Old Command Structs from Proto Crate

**File**: `moto-hses-proto/src/commands/variable.rs`

Remove the following 10 structs and their implementations (including tests):

- `ReadMultipleByteVariables` / `WriteMultipleByteVariables` (lines 93-202)
- `ReadMultipleIntegerVariables` / `WriteMultipleIntegerVariables` (lines 204-310)
- `ReadMultipleDoubleVariables` / `WriteMultipleDoubleVariables` (lines 312-418)
- `ReadMultipleRealVariables` / `WriteMultipleRealVariables` (lines 420-526)
- `ReadMultipleCharacterVariables` / `WriteMultipleCharacterVariables` (lines 1030-1122)
- All related unit tests (lines 528-1244)

**File**: `moto-hses-proto/src/commands/mod.rs`

Update exports to remove old structs and add new generic ones:

```rust
pub use variable::{
    ReadMultipleVariables, WriteMultipleVariables,
    MultipleVariableCommandId, MultipleVariableResponse,
    ReadVar, WriteVar, VariableCommandId,
};
```

### 8. Update Integration Tests

**File**: `moto-hses-client/tests/integration/variable_operations.rs`

Update all test method calls:

- `read_multiple_byte_variables` → `read_multiple_u8`
- `write_multiple_byte_variables` → `write_multiple_u8`
- `read_multiple_integer_variables` → `read_multiple_i16`
- `write_multiple_integer_variables` → `write_multiple_i16`
- `read_multiple_double_variables` → `read_multiple_i32`
- `write_multiple_double_variables` → `write_multiple_i32`
- `read_multiple_real_variables` → `read_multiple_f32`
- `write_multiple_real_variables` → `write_multiple_f32`
- `read_multiple_character_variables` → `read_multiple_strings`
- `write_multiple_character_variables` → `write_multiple_strings`

### 9. Update Example Code

Update the following example files:

**File**: `moto-hses-client/examples/byte_variable_operations.rs`

- `read_multiple_byte_variables` → `read_multiple_u8`
- `write_multiple_byte_variables` → `write_multiple_u8`

**File**: `moto-hses-client/examples/integer_variable_operations.rs`

- `read_multiple_integer_variables` → `read_multiple_i16`
- `write_multiple_integer_variables` → `write_multiple_i16`

**File**: `moto-hses-client/examples/double_variable_operations.rs`

- `read_multiple_double_variables` → `read_multiple_i32`
- `write_multiple_double_variables` → `write_multiple_i32`

**File**: `moto-hses-client/examples/real_variable_operations.rs`

- `read_multiple_real_variables` → `read_multiple_f32`
- `write_multiple_real_variables` → `write_multiple_f32`

**File**: `moto-hses-client/examples/string_variable_operations.rs`

- `read_multiple_character_variables` → `read_multiple_strings`
- `write_multiple_character_variables` → `write_multiple_strings`

### 10. Run Quality Checks

Run the following checks in order:

1. `cargo fmt --all`
2. `cargo clippy --all-features --workspace`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

Fix any issues that arise from the refactoring.

### 11. Post-Implementation Tasks

- Update To-dos section status in this plan document
- Update plan content to reflect actual implementation details
- Update Implementation Feedback section with lessons learned

## Key Design Decisions

1. **Type Safety**: Use generics to ensure type consistency at compile time
2. **Backward Compatibility**: Convenience methods in convenience.rs provide same ease of use as before
3. **Code Reduction**: Reduce protocol.rs from ~500 lines to ~150 lines for multiple variable operations
4. **Encoding Support**: Special methods for S variables handle text encoding properly
5. **Unified Validation**: Centralize count validation in proto crate via trait methods
6. **Trait-Based Design**: Use traits (`MultipleVariableCommandId`, `MultipleVariableResponse`) for extensibility

## Benefits

1. **Reduced Code Duplication**: Eliminate ~400 lines of duplicated code
2. **Easier Maintenance**: Changes to multiple variable logic only need to be made once
3. **Type Safety**: Compiler enforces correct usage of variable types
4. **Consistent Behavior**: All variable types follow the same code path
5. **Better Testability**: Generic implementations can be tested once for all types

## Implementation Feedback for Rules Update

No significant issues requiring rule updates. The refactoring follows established patterns and coding standards. All Clippy warnings were addressed using existing rules (inline format arguments, proper error handling with try_from).

### To-dos

- [x] Add MultipleVariableCommandId trait and implementations to proto crate
- [x] Add ReadMultipleVariables<T> and WriteMultipleVariables<T> generic structs to proto crate
- [x] Add MultipleVariableResponse trait and implementations to proto crate
- [x] Implement serialization for WriteMultipleVariables<T> for each type
- [x] Add read_multiple_variables<T> and write_multiple_variables<T> to client protocol.rs
- [x] Add special string variable methods with encoding support to protocol.rs
- [x] Add convenience wrapper methods to convenience.rs
- [x] Remove old 10 methods from protocol.rs
- [x] Remove old 10 command structs from proto crate variable.rs
- [x] Update exports in proto commands/mod.rs
- [x] Update integration tests to use new method names
- [x] Update all example files to use new method names
- [x] Run cargo fmt, clippy, test, and doc
- [x] Update plan To-dos section status after implementation completion
- [x] Update plan content to reflect actual implementation