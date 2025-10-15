<!-- a0448b26-93d1-4bc6-bb06-55daed5a74bf 12878785-19ca-4799-911f-0d6f4de597ca -->
# 0x303 Plural Integer Type Variable (I) Reading/Writing Command Implementation

## Overview

Implement the 0x303 command (Plural Integer Type Variable (I) Reading/Writing Command) to provide functionality for reading and writing multiple integer type variables (i16) in a single request, supporting up to 237 I variable data items (extended setting support).

## Command Specification

- **Command ID**: 0x303
- **Instance**: Variable number (first variable number with which reading/writing is executed)
  - Standard setting: 0-99
  - Extended setting: 0-999 (varies by configuration)
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data (Reads out the fixed size specified by the data part)
  - `0x34`: Write plural data (Writes the fixed size specified by the data part)
- **Payload**: Plural I variable data
  - Byte0-3: Number of I variable data (Maximum value: 237)
  - Byte4-5: I variable data 1 (2 bytes, i16, little-endian)
  - Byte6-7: I variable data 2 (2 bytes, i16, little-endian)
  - ...
  - Byte(3 + (Number - 1) * 2 + 1)-Byte(3 + Number * 2): I variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - I variable data section is valid only when writing
    - Each I variable data is 2 bytes (i16, little-endian), and the payload contains the number of I variable data specified by the Number field
- **Response**: Same structure as request, with I variable data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/variable.rs`

Add structures for Plural I Variable command (following the same pattern as 0x302):

```rust
/// Read multiple integer variables (I) command (0x303)
#[derive(Debug, Clone)]
pub struct ReadMultipleIntegerVariables {
    pub start_variable_number: u8,
    pub count: u32,  // Number of I variable data (max 237)
}

impl ReadMultipleIntegerVariables {
    pub fn new(start_variable_number: u8, count: u32) -> Result<Self, ProtocolError> {
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
    type Response = Vec<i16>;  // Array of I variable values
    fn command_id() -> u16 { 0x303 }
    fn instance(&self) -> u16 { u16::from(self.start_variable_number) }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple integer variables (I) command (0x303)
#[derive(Debug, Clone)]
pub struct WriteMultipleIntegerVariables {
    pub start_variable_number: u8,
    pub values: Vec<i16>,  // I variable values to write
}

impl WriteMultipleIntegerVariables {
    pub fn new(start_variable_number: u8, values: Vec<i16>) -> Result<Self, ProtocolError> {
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
    fn command_id() -> u16 { 0x303 }
    fn instance(&self) -> u16 { u16::from(self.start_variable_number) }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x34 }  // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len())
            .map_err(|_| ProtocolError::InvalidMessage(format!(
                "Values count {} exceeds u32::MAX", self.values.len()
            )))?;
        let mut payload = count.to_le_bytes().to_vec();
        for &value in &self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        Ok(payload)
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add exports for `ReadMultipleIntegerVariables` and `WriteMultipleIntegerVariables`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods:

```rust
/// Read multiple integer variables (I) (0x303 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number
/// * `count` - Number of variables to read (max 237)
///
/// # Returns
///
/// Vector of variable values (i16)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_integer_variables(
    &self,
    start_variable_number: u8,
    count: u32,
) -> Result<Vec<i16>, ClientError> {
    let command = ReadMultipleIntegerVariables::new(start_variable_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    
    // Response format: Byte0-3 = count, Byte4-N = variable data (2 bytes each)
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
    
    // Parse variable values (2 bytes each)
    let expected_len = 4 + (count as usize * 2);
    if response.len() != expected_len {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization(format!("Invalid response length: got {} bytes, expected {expected_len}", response.len()))
        ));
    }
    
    let mut values = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = 4 + i * 2;
        let value = i16::from_le_bytes([response[offset], response[offset + 1]]);
        values.push(value);
    }
    Ok(values)
}

/// Write multiple integer variables (I) (0x303 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number
/// * `values` - Variable values to write (max 237 items)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_integer_variables(
    &self,
    start_variable_number: u8,
    values: Vec<i16>,
) -> Result<(), ClientError> {
    let command = WriteMultipleIntegerVariables::new(start_variable_number, values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add batch operations for integer variables:

```rust
/// Get multiple integer variable values
pub fn get_multiple_integer_variables(&self, start_variable: u8, count: usize) -> Vec<i16> {
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let var_num = start_variable + u8::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u8::MAX"))
            .expect("Variable index should fit in u8");
        let var_data = self.get_variable(var_num);
        // I variable is 2 bytes (i16)
        let value = var_data.map_or(0_i16, |data| {
            if data.len() >= 2 {
                i16::from_le_bytes([data[0], data[1]])
            } else {
                0
            }
        });
        values.push(value);
    }
    values
}

/// Set multiple integer variable values
pub fn set_multiple_integer_variables(&mut self, start_variable: u8, values: &[i16]) {
    for (i, &value) in values.iter().enumerate() {
        let var_num = start_variable + u8::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u8::MAX"))
            .expect("Variable index should fit in u8");
        self.set_variable(var_num, value.to_le_bytes().to_vec());
    }
}
```

### 4. Plural I Variable Handler Implementation

**File**: `moto-hses-mock/src/handlers/variable.rs`

Add handler for 0x303 command:

```rust
/// Handler for plural integer variable operations (0x303)
pub struct PluralIntegerVarHandler;

impl CommandHandler for PluralIntegerVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidInstance(format!(
                "Variable index {} exceeds u8::MAX", message.sub_header.instance
            ))
        })?;
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
        
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-237)"
            )));
        }
        
        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_integer_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                for value in values {
                    response.extend_from_slice(&value.to_le_bytes());
                }
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + (count as usize * 2);
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {expected_len}", message.payload.len()
                    )));
                }
                
                // Parse variable values (2 bytes each)
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 2;
                    let value = i16::from_le_bytes([
                        message.payload[offset],
                        message.payload[offset + 1],
                    ]);
                    values.push(value);
                }
                
                state.set_multiple_integer_variables(start_variable, &values);
                
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

Register handler for 0x303 command in `CommandHandlerRegistry::new()`:

```rust
registry.register(0x303, Box::new(PluralIntegerVarHandler));
```

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/variable.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 237, must be > 0)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + 2 bytes per value)
- Command trait implementation tests (ID, Instance, Attribute, Service)

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/variable_operations.rs` (extend existing)

Add integration tests:

- Read multiple integer variables from various ranges
- Write multiple integer variables
- Verify state changes using read-back verification
- Test boundary conditions (count = 1, count = 237)
- Test count validation (0, 238)
- Test maximum safe count calculations

### 8. Example Code

**File**: `moto-hses-client/examples/variable_operations.rs` (extend existing)

Add to variable operations example (happy path only):

- Reading multiple integer variables (0x303 command)
- Writing multiple integer variables (0x303 command)
- Note: Error handling not needed in examples, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x303 command to "Supported Commands" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x303 command to "Supported Commands" section

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

- Attribute is 0 (same as 0x302 plural byte variable command)
- Service 0x33 for read, 0x34 for write
- **Count validation is critical**: max 237, must be > 0 (no multiple-of-2 requirement unlike 0x302)
- **Data size**: Each I variable data is 2 bytes (i16, little-endian)
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + variable data (2 bytes each)
  - Read response: count (4 bytes) + variable data (2 bytes each)
  - Write response: only count (4 bytes)
- MockState integration with existing variable state management
- Follow the same pattern as 0x302 (Plural Byte Variable Command) for consistency

## Testing Considerations

### Integration Test Patterns

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Read-Back Verification**: After write operations, read back values to verify changes
2. **Boundary Testing**: Test minimum (count=1) and maximum (count=237) values
3. **Error Cases**: Test invalid ranges and count violations
```rust
// Example pattern for I variable tests
let _server = create_variable_test_server().await?;
let client = create_test_client().await?;

// Write multiple integer variables
let values = vec![100, -200, 300, -400];
client.write_multiple_integer_variables(0, values.clone()).await?;

// Read back and verify
let read_values = client.read_multiple_integer_variables(0, 4).await?;
assert_eq!(read_values, values);

// Test maximum count (237)
let max_values: Vec<i16> = (0..237).map(|i| i16::try_from(i).unwrap_or(0)).collect();
client.write_multiple_integer_variables(0, max_values.clone()).await?;
let read_max = client.read_multiple_integer_variables(0, 237).await?;
assert_eq!(read_max, max_values);
```


## Implementation Feedback for Rules Update

The implementation was completed successfully with no significant issues requiring rule updates. The following observations were made:

### Positive Aspects

1. **Consistent Pattern Reuse**: Following the 0x302 (Plural Byte Variable) implementation pattern significantly reduced development time and complexity.
2. **Comprehensive Testing**: Unit tests and integration tests provided solid verification coverage.
3. **Clear Specifications**: The HSES protocol specification was detailed and accurate, enabling straightforward implementation.
4. **Clippy Integration**: Clippy warnings (e.g., `must_use` attribute) helped maintain code quality and were easy to resolve.

### Implementation Notes

- All unit tests passed (16 tests for variable operations)
- All integration tests passed (86 tests total, including 4 new 0x303 tests)
- No Clippy warnings after fixes
- Documentation updated across all crates
- Example code successfully demonstrates the new functionality

### Recommendations

No new rules are needed. The existing coding standards and development rules were sufficient for this implementation.

### To-dos

- [x] Protocol layer implementation - Add ReadMultipleIntegerVariables and WriteMultipleIntegerVariables structs to variable.rs with proper validation
- [x] Export ReadMultipleIntegerVariables and WriteMultipleIntegerVariables in commands/mod.rs
- [x] Client API implementation - Add read_multiple_integer_variables() and write_multiple_integer_variables() methods in protocol.rs
- [x] MockState extension - Add get_multiple_integer_variables() and set_multiple_integer_variables() methods
- [x] Handler implementation - Add PluralIntegerVarHandler in handlers/variable.rs with validation and state management
- [x] Handler registration - Register PluralIntegerVarHandler for 0x303 command
- [x] Create unit tests for ReadMultipleIntegerVariables and WriteMultipleIntegerVariables including validation and serialization
- [x] Create integration tests with read-back verification for read/write operations
- [x] Extend example code demonstrating plural integer variable operations
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x303 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update plan To-dos section status after implementation completion
- [x] Update plan content to reflect actual implementation
- [x] Update Implementation Feedback section with lessons learned