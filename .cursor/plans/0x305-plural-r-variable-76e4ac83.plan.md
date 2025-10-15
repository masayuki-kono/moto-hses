<!-- 76e4ac83-db05-405b-83b3-0ad42488931d 84bdd02a-4c22-4535-bcf3-5886f228f067 -->
# 0x305 Plural Real Type Variable (R) Reading/Writing Command Implementation

## Overview

Implement the 0x305 command (Plural Real Type Variable (R) Reading/Writing Command) to provide functionality for reading and writing multiple real type variables (f32) in a single request, supporting up to 118 R variable data items (extended setting support).

## Command Specification

- **Command ID**: 0x305
- **Instance**: Variable number (first variable number with which reading/writing is executed)
  - Standard setting: 0-99
  - Extended setting: 0-999 (varies by configuration)
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural R variable data
  - Byte0-3: Number of R variable data (Maximum value: 118)
  - Byte4-7: R variable data 1 (4 bytes, f32, little-endian)
  - Byte8-11: R variable data 2 (4 bytes, f32, little-endian)
  - ...
  - Byte(3 + (Number - 1) * 4 + 1)-Byte(3 + Number * 4): R variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - R variable data section is valid only when writing
    - Each R variable data is 4 bytes (f32, little-endian), and the payload contains the number of R variable data specified by the Number field
- **Response**: Same structure as request, with R variable data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/variable.rs`

Add structures for Plural R Variable command (following the same pattern as 0x304):

```rust
/// Read multiple real type variables (R) command (0x305)
#[derive(Debug, Clone)]
pub struct ReadMultipleRealVariables {
    pub start_variable_number: u16,  // Support extended variable settings (0-999)
    pub count: u32,  // Number of R variable data (max 118)
}

impl ReadMultipleRealVariables {
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
    type Response = Vec<f32>;  // Array of R variable values
    fn command_id() -> u16 { 0x305 }
    fn instance(&self) -> u16 { self.start_variable_number }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple real type variables (R) command (0x305)
#[derive(Debug, Clone)]
pub struct WriteMultipleRealVariables {
    pub start_variable_number: u16,  // Support extended variable settings (0-999)
    pub values: Vec<f32>,  // R variable values to write
}

impl WriteMultipleRealVariables {
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
    fn command_id() -> u16 { 0x305 }
    fn instance(&self) -> u16 { self.start_variable_number }
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

Add exports for `ReadMultipleRealVariables` and `WriteMultipleRealVariables`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods:

```rust
/// Read multiple real type variables (R) (0x305 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number (0-999 for extended settings)
/// * `count` - Number of variables to read (max 118)
///
/// # Returns
///
/// Vector of variable values (f32)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_real_variables(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<f32>, ClientError> {
    let command = ReadMultipleRealVariables::new(start_variable_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    
    // Response format: Byte0-3 = count, Byte4-N = variable data (4 bytes each)
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
    
    // Parse variable values (4 bytes each)
    let expected_len = 4 + (count as usize * 4);
    if response.len() != expected_len {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization(format!("Invalid response length: got {} bytes, expected {expected_len}", response.len()))
        ));
    }
    
    let mut values = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = 4 + i * 4;
        let value = f32::from_le_bytes([
            response[offset], 
            response[offset + 1], 
            response[offset + 2], 
            response[offset + 3]
        ]);
        values.push(value);
    }
    Ok(values)
}

/// Write multiple real type variables (R) (0x305 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number (0-999 for extended settings)
/// * `values` - Variable values to write (max 118 items)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_real_variables(
    &self,
    start_variable_number: u16,
    values: Vec<f32>,
) -> Result<(), ClientError> {
    let command = WriteMultipleRealVariables::new(start_variable_number, values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add batch operations for real type variables:

```rust
/// Get multiple real type variable values
pub fn get_multiple_real_variables(&self, start_variable: u16, count: usize) -> Vec<f32> {
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let var_num = start_variable + u16::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u16::MAX"))
            .expect("Variable index should fit in u16");
        let var_data = self.get_variable(var_num);
        // R variable is 4 bytes (f32)
        let value = var_data.map_or(0.0_f32, |data| {
            if data.len() >= 4 {
                f32::from_le_bytes([data[0], data[1], data[2], data[3]])
            } else {
                0.0
            }
        });
        values.push(value);
    }
    values
}

/// Set multiple real type variable values
pub fn set_multiple_real_variables(&mut self, start_variable: u16, values: &[f32]) {
    for (i, &value) in values.iter().enumerate() {
        let var_num = start_variable + u16::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u16::MAX"))
            .expect("Variable index should fit in u16");
        self.set_variable(var_num, value.to_le_bytes().to_vec());
    }
}
```

### 4. Plural R Variable Handler Implementation

**File**: `moto-hses-mock/src/handlers/variable.rs`

Add handler for 0x305 command:

```rust
/// Handler for plural real type variable operations (0x305)
pub struct PluralRealVarHandler;

impl CommandHandler for PluralRealVarHandler {
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
        
        // Validate count (max 118, must be > 0)
        if count == 0 || count > 118 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-118)"
            )));
        }
        
        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_real_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                for value in values {
                    response.extend_from_slice(&value.to_le_bytes());
                }
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + (count as usize * 4);
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {expected_len}", message.payload.len()
                    )));
                }
                
                // Parse variable values (4 bytes each)
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 4;
                    let value = f32::from_le_bytes([
                        message.payload[offset],
                        message.payload[offset + 1],
                        message.payload[offset + 2],
                        message.payload[offset + 3],
                    ]);
                    values.push(value);
                }
                
                state.set_multiple_real_variables(start_variable, &values);
                
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

Register handler for 0x305 command in `CommandHandlerRegistry::new()`:

```rust
registry.register(0x305, Box::new(PluralRealVarHandler));
```

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/variable.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 118, must be > 0)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + 4 bytes per value)
- Command trait implementation tests (ID, Instance, Attribute, Service)
- Test with various f32 values (positive, negative, zero, special values)

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/variable_operations.rs` (extend existing)

Add integration tests:

- Read multiple real type variables from various ranges
- Write multiple real type variables
- Verify state changes using read-back verification
- Test boundary conditions (count = 1, count = 118)
- Test count validation (0, 119)
- Test with various f32 values (positive, negative, zero, large values, small values)
- Test floating point precision handling

### 8. Example Code

**File**: `moto-hses-client/examples/real_variable_operations.rs`

Update existing real variable operations example to include plural operations:

- Reading multiple real type variables (0x305 command)
- Writing multiple real type variables (0x305 command)
- Note: Examples focus on happy path, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x305 command to "Supported Commands" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x305 command to "Supported Commands" section

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

- Attribute is 0 (same as 0x304 and other plural variable commands)
- Service 0x33 for read, 0x34 for write
- **Count validation is critical**: max 118, must be > 0 (no multiple-of-2 requirement)
- **Data size**: Each R variable data is 4 bytes (f32, little-endian)
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + variable data (4 bytes each)
  - Read response: count (4 bytes) + variable data (4 bytes each)
  - Write response: only count (4 bytes)
- MockState integration with existing variable state management
- Follow the same pattern as 0x304 (Plural D Variable Command) for consistency
- Handle f32 floating-point serialization/deserialization correctly

## Testing Considerations

### Integration Test Patterns

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Read-Back Verification**: After write operations, read back values to verify changes
2. **Boundary Testing**: Test minimum (count=1) and maximum (count=118) values
3. **Value Range Testing**: Test positive, negative, zero, large, and small f32 values
4. **Error Cases**: Test invalid ranges and count violations
```rust
// Example pattern for R variable tests
let _server = create_variable_test_server().await?;
let client = create_test_client().await?;

// Write multiple real type variables
let values = vec![1.5, -2.75, 0.0, 123.456];
client.write_multiple_real_variables(0, values.clone()).await?;

// Read back and verify
let read_values = client.read_multiple_real_variables(0, 4).await?;
assert_eq!(read_values, values);

// Test maximum count (118)
let max_values: Vec<f32> = (0..118).map(|i| i as f32 * 1.5).collect();
client.write_multiple_real_variables(0, max_values.clone()).await?;
let read_max = client.read_multiple_real_variables(0, 118).await?;
assert_eq!(read_max, max_values);
```


## Implementation Feedback for Rules Update

The implementation was completed successfully with no significant issues requiring rule updates.

### To-dos

- [x] Protocol layer implementation - Add ReadMultipleRealVariables and WriteMultipleRealVariables structs to variable.rs with proper validation
- [x] Export ReadMultipleRealVariables and WriteMultipleRealVariables in commands/mod.rs
- [x] Client API implementation - Add read_multiple_real_variables() and write_multiple_real_variables() methods in protocol.rs
- [x] MockState extension - Add get_multiple_real_variables() and set_multiple_real_variables() methods
- [x] Handler implementation - Add PluralRealVarHandler in handlers/variable.rs with validation and state management
- [x] Handler registration - Register PluralRealVarHandler for 0x305 command
- [x] Create unit tests for ReadMultipleRealVariables and WriteMultipleRealVariables including validation and serialization
- [x] Create integration tests with read-back verification for read/write operations
- [x] Update example code in real_variable_operations.rs demonstrating plural real variable operations
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x305 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update plan To-dos section status after implementation completion
- [x] Update plan content to reflect actual implementation
- [x] Update Implementation Feedback section with lessons learned