<!-- 3ca1b338-7e9b-44d1-898d-16750247666a 3e3f3b3a-33b4-4f8e-b6d0-2ad810d5eed1 -->
# 0x306 Plural Character Type Variable (S) Reading/Writing Command Implementation

## Overview

Implement the 0x306 command (Plural Character Type Variable (S) Reading/Writing Command) to provide functionality for reading and writing multiple character type variables (16-byte strings) in a single request, supporting up to 29 S variable data items.

## Command Specification

- **Command ID**: 0x306
- **Instance**: Variable number (first variable number with which reading/writing is executed)
  - Standard setting: 0-99
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural S variable data
  - Byte0-3: Number of S variable data (Maximum value: 29)
  - Byte4-19: S variable data 1 (16 bytes)
  - Byte20-35: S variable data 2 (16 bytes)
  - ...
  - Byte(3 + (Number - 1) * 16 + 1)-Byte(3 + Number * 16): S variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - S variable data section is valid only when writing
    - Each S variable data is 16 bytes, and the payload contains the number of S variable data specified by the Number field
- **Response**: Same structure as request, with S variable data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/variable.rs`

Add structures for Plural S Variable command (following the same pattern as 0x305):

```rust
/// Read multiple character type variables (S) command (0x306)
#[derive(Debug, Clone)]
pub struct ReadMultipleCharacterVariables {
    pub start_variable_number: u16,  // Support extended variable settings (0-99 for standard)
    pub count: u32,  // Number of S variable data (max 29)
}

impl ReadMultipleCharacterVariables {
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
    type Response = Vec<[u8; 16]>;  // Array of 16-byte S variable values
    fn command_id() -> u16 { 0x306 }
    fn instance(&self) -> u16 { self.start_variable_number }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple character type variables (S) command (0x306)
#[derive(Debug, Clone)]
pub struct WriteMultipleCharacterVariables {
    pub start_variable_number: u16,  // Support extended variable settings (0-99 for standard)
    pub values: Vec<[u8; 16]>,  // S variable values to write (16 bytes each)
}

impl WriteMultipleCharacterVariables {
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
    fn command_id() -> u16 { 0x306 }
    fn instance(&self) -> u16 { self.start_variable_number }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x34 }  // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len())
            .map_err(|_| ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX", self.values.len()
            )))?;
        let mut payload = count.to_le_bytes().to_vec();
        for value in &self.values {
            payload.extend_from_slice(value);
        }
        Ok(payload)
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add exports for `ReadMultipleCharacterVariables` and `WriteMultipleCharacterVariables`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods with encoding support (following the same pattern as single character variables):

```rust
/// Read multiple character type variables (S) (0x306 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number (0-99 for standard settings)
/// * `count` - Number of variables to read (max 29)
///
/// # Returns
///
/// Vector of variable values as strings decoded with the client's text encoding
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_character_variables(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<String>, ClientError> {
    let command = ReadMultipleCharacterVariables::new(start_variable_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    
    // Response format: Byte0-3 = count, Byte4-N = variable data (16 bytes each)
    if response.len() < 4 {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization(format!("Response too short: {} bytes (need at least 4)", response.len()))
        ));
    }
    
    let response_count = u32::from_le_bytes([
        response[0], response[1], response[2], response[3]
    ]);
    
    if response_count != count {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization(format!("Count mismatch: expected {count}, got {response_count}"))
        ));
    }
    
    // Parse variable values (16 bytes each)
    let expected_len = 4 + (count as usize * 16);
    if response.len() != expected_len {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization(format!("Invalid response length: got {} bytes, expected {expected_len}", response.len()))
        ));
    }
    
    let mut values = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = 4 + i * 16;
        let value_bytes = &response[offset..offset + 16];
        
        // Decode bytes to string using client's text encoding
        // Remove null bytes from the end
        let trimmed_bytes = value_bytes
            .iter()
            .position(|&b| b == 0)
            .map_or(value_bytes, |pos| &value_bytes[..pos]);
        
        let value_string = moto_hses_proto::encoding_utils::decode_string_with_fallback(
            trimmed_bytes,
            self.config.text_encoding,
        );
        values.push(value_string);
    }
    Ok(values)
}

/// Write multiple character type variables (S) (0x306 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number (0-99 for standard settings)
/// * `values` - Variable values to write as strings (max 29 items, each up to 16 bytes when encoded)
///
/// # Errors
///
/// Returns an error if communication fails, parameters are invalid, or strings exceed 16 bytes when encoded
pub async fn write_multiple_character_variables(
    &self,
    start_variable_number: u16,
    values: Vec<String>,
) -> Result<(), ClientError> {
    // Convert strings to 16-byte arrays with proper encoding
    let mut encoded_values = Vec::with_capacity(values.len());
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
        
        let mut value_array = [0u8; 16];
        value_array[..encoded_bytes.len()].copy_from_slice(&encoded_bytes);
        encoded_values.push(value_array);
    }
    
    let command = WriteMultipleCharacterVariables::new(start_variable_number, encoded_values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add batch operations for character type variables:

```rust
/// Get multiple character type variable values
pub fn get_multiple_character_variables(&self, start_variable: u16, count: usize) -> Vec<[u8; 16]> {
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let var_num = start_variable + u16::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u16::MAX"))
            .expect("Variable index should fit in u16");
        let var_data = self.get_variable(var_num);
        // S variable is 16 bytes
        let mut value = [0u8; 16];
        if let Some(data) = var_data {
            let copy_len = data.len().min(16);
            value[..copy_len].copy_from_slice(&data[..copy_len]);
        }
        values.push(value);
    }
    values
}

/// Set multiple character type variable values
pub fn set_multiple_character_variables(&mut self, start_variable: u16, values: &[[u8; 16]]) {
    for (i, value) in values.iter().enumerate() {
        let var_num = start_variable + u16::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u16::MAX"))
            .expect("Variable index should fit in u16");
        self.set_variable(var_num, value.to_vec());
    }
}
```

### 4. Plural S Variable Handler Implementation

**File**: `moto-hses-mock/src/handlers/variable.rs`

Add handler for 0x306 command:

```rust
/// Handler for plural character type variable operations (0x306)
pub struct PluralCharacterVarHandler;

impl CommandHandler for PluralCharacterVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = message.sub_header.instance;
        let service = message.sub_header.service;
        
        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }
        
        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Payload too short: {} bytes for start_variable {start_variable} (need at least 4 bytes)", 
                message.payload.len()
            )));
        }
        
        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);
        
        // Validate count (max 29, must be > 0)
        if count == 0 || count > 29 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-29)"
            )));
        }
        
        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_character_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                for value in values {
                    response.extend_from_slice(&value);
                }
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + (count as usize * 16);
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {expected_len}", message.payload.len()
                    )));
                }
                
                // Parse variable values (16 bytes each)
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 16;
                    let mut value = [0u8; 16];
                    value.copy_from_slice(&message.payload[offset..offset + 16]);
                    values.push(value);
                }
                
                state.set_multiple_character_variables(start_variable, &values);
                
                // Return only count
                Ok(count.to_le_bytes().to_vec())
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
```

### 5. Handler Registration

**File**: `moto-hses-mock/src/handlers/registry.rs`

Register handler for 0x306 command in `CommandHandlerRegistry::new()`:

```rust
registry.register(0x306, Box::new(PluralCharacterVarHandler));
```

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/variable.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 29, must be > 0)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + 16 bytes per value)
- Command trait implementation tests (ID, Instance, Attribute, Service)
- Test with various byte patterns (ASCII text, UTF-8, binary data, empty/null-padded strings)

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/variable_operations.rs` (extend existing)

Add integration tests with string-based API:

- Read multiple character type variables from various ranges
- Write multiple character type variables using string values
- Verify state changes using read-back verification with string comparison
- Test boundary conditions (count = 1, count = 29)
- Test count validation (0, 30)
- Test with various string patterns (ASCII strings, UTF-8 strings, mixed content)
- Test string encoding handling and validation
- Test string length validation (max 16 bytes when encoded)
- Test null byte trimming in read operations

### 8. Example Code

**File**: `moto-hses-client/examples/string_variable_operations.rs`

Update string variable operations example to include plural operations:

- Reading multiple character type variables (0x306 command) with string return values
- Writing multiple character type variables (0x306 command) with string input values
- Demonstrate string encoding/decoding with client's text encoding
- Show verification of read/write consistency with string comparison
- Note: Examples focus on happy path, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x306 command to "Supported Commands" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x306 command to "Supported Commands" section

### 10. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all`
2. `cargo clippy --all-features --workspace`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

### 11. Post-Implementation Tasks

- Update To-dos section status in this plan document (`.cursor/plans/`)
- Update plan content to reflect actual implementation details and any changes made during development
- Update Implementation Feedback section with lessons learned

## Key Implementation Points

- Attribute is 0 (same as other plural variable commands)
- Service 0x33 for read, 0x34 for write
- **Count validation is critical**: max 29, must be > 0
- **Data size**: Each S variable data is 16 bytes (fixed-size character array)
- **Response type**: `Vec<[u8; 16]>` to represent 16-byte character arrays
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + variable data (16 bytes each)
  - Read response: count (4 bytes) + variable data (16 bytes each)
  - Write response: only count (4 bytes)
- MockState integration with existing variable state management
- Follow the same pattern as 0x305 (Plural R Variable Command) for consistency
- Handle byte array serialization/deserialization correctly
- Character data may contain null-terminated strings or raw binary data

## Testing Considerations

### Integration Test Patterns

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Read-Back Verification**: After write operations, read back values to verify changes with string comparison
2. **Boundary Testing**: Test minimum (count=1) and maximum (count=29) values
3. **String Pattern Testing**: Test ASCII strings, UTF-8 strings, mixed content with proper encoding
4. **Error Cases**: Test invalid ranges, count violations, and string length validation
5. **Encoding Validation**: Test string length limits when encoded (max 16 bytes)
```rust
// Example pattern for S variable tests with string-based API
let _server = create_variable_test_server().await?;
let client = create_test_client().await?;

// Write multiple character type variables using strings
let values = vec!["Hello".to_string(), "World".to_string()];
client.write_multiple_character_variables(0, values.clone()).await?;

// Read back and verify with string comparison
let read_values = client.read_multiple_character_variables(0, 2).await?;
assert_eq!(read_values, values);

// Test maximum count (29) with string values
let max_values: Vec<String> = (0..29)
    .map(|i| format!("Test{i:02}"))
    .collect();
client.write_multiple_character_variables(20, max_values.clone()).await?;

// Test with various string patterns including UTF-8
let pattern_values = vec![
    "ASCII_STRING".to_string(),
    "こんにちは".to_string(),
    "Binary123".to_string(),
];
client.write_multiple_character_variables(50, pattern_values.clone()).await?;
```


## Implementation Feedback for Rules Update

The implementation was completed successfully with significant improvements over the original plan:

### Key Implementation Notes

1. **Encoding Support Enhancement**: The implementation went beyond the original plan by adding comprehensive encoding support to match the existing single character variable pattern. This ensures consistency across the codebase and provides better user experience.

2. **String-Based API**: Instead of raw byte arrays, the implementation provides string-based APIs that handle encoding/decoding automatically, making the API more user-friendly and consistent with other variable operations.

3. **Null Byte Handling**: Proper handling of null bytes in 16-byte character arrays ensures clean string output without padding artifacts.

4. **Validation Enhancement**: Added string length validation to ensure encoded strings don't exceed the 16-byte limit, providing clear error messages.

### No Significant Issues Requiring Rule Updates

The implementation followed existing patterns successfully and enhanced the original design with better usability and consistency. The encoding support addition was a valuable improvement that aligns with the project's overall architecture.

### To-dos

- [x] Protocol layer implementation - Add ReadMultipleCharacterVariables and WriteMultipleCharacterVariables structs to variable.rs with proper validation
- [x] Export ReadMultipleCharacterVariables and WriteMultipleCharacterVariables in commands/mod.rs
- [x] Client API implementation - Add read_multiple_character_variables() and write_multiple_character_variables() methods in protocol.rs
- [x] MockState extension - Add get_multiple_character_variables() and set_multiple_character_variables() methods
- [x] Handler implementation - Add PluralCharacterVarHandler in handlers/variable.rs with validation and state management
- [x] Handler registration - Register PluralCharacterVarHandler for 0x306 command
- [x] Create unit tests for ReadMultipleCharacterVariables and WriteMultipleCharacterVariables including validation and serialization
- [x] Create integration tests with read-back verification for read/write operations
- [x] Create or update example code in character_variable_operations.rs demonstrating plural character variable operations
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x306 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update plan To-dos section status after implementation completion
- [x] Update plan content to reflect actual implementation
- [x] Update Implementation Feedback section with lessons learned