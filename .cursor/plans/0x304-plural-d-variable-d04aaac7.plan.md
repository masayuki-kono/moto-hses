<!-- d04aaac7-6ed7-4a5b-a9c0-1ed42c544f2f abeb8c90-934b-4338-958c-52e7da068ded -->
# 0x304 Plural Double Precision Integer Type Variable (D) Reading/Writing Command Implementation

## Overview

Implement the 0x304 command (Plural Double Precision Integer Type Variable (D) Reading/Writing Command) to provide functionality for reading and writing multiple double precision integer type variables (i32) in a single request, supporting up to 118 D variable data items (extended setting support).

## Command Specification

- **Command ID**: 0x304
- **Instance**: Variable number (first variable number with which reading/writing is executed)
  - Standard setting: 0-99
  - Extended setting: 0-999 (varies by configuration)
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural D variable data
  - Byte0-3: Number of D variable data (Maximum value: 118)
  - Byte4-7: D variable data 1 (4 bytes, i32, little-endian)
  - Byte8-11: D variable data 2 (4 bytes, i32, little-endian)
  - ...
  - Byte(3 + (Number - 1) * 4 + 1)-Byte(3 + Number * 4): D variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - D variable data section is valid only when writing
    - Each D variable data is 4 bytes (i32, little-endian), and the payload contains the number of D variable data specified by the Number field
- **Response**: Same structure as request, with D variable data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/variable.rs`

Add structures for Plural D Variable command (following the same pattern as 0x303):

```rust
/// Read multiple double precision integer variables (D) command (0x304)
#[derive(Debug, Clone)]
pub struct ReadMultipleDoubleVariables {
    pub start_variable_number: u16,  // Support extended variable settings (0-999)
    pub count: u32,  // Number of D variable data (max 118)
}

impl ReadMultipleDoubleVariables {
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
    type Response = Vec<i32>;  // Array of D variable values
    fn command_id() -> u16 { 0x304 }
    fn instance(&self) -> u16 { self.start_variable_number }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple double precision integer variables (D) command (0x304)
#[derive(Debug, Clone)]
pub struct WriteMultipleDoubleVariables {
    pub start_variable_number: u16,  // Support extended variable settings (0-999)
    pub values: Vec<i32>,  // D variable values to write
}

impl WriteMultipleDoubleVariables {
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
    fn command_id() -> u16 { 0x304 }
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

Add exports for `ReadMultipleDoubleVariables` and `WriteMultipleDoubleVariables`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods:

```rust
/// Read multiple double precision integer variables (D) (0x304 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number (0-999 for extended settings)
/// * `count` - Number of variables to read (max 118)
///
/// # Returns
///
/// Vector of variable values (i32)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_double_variables(
    &self,
    start_variable_number: u16,
    count: u32,
) -> Result<Vec<i32>, ClientError> {
    let command = ReadMultipleDoubleVariables::new(start_variable_number, count)?;
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
        let value = i32::from_le_bytes([
            response[offset], 
            response[offset + 1], 
            response[offset + 2], 
            response[offset + 3]
        ]);
        values.push(value);
    }
    Ok(values)
}

/// Write multiple double precision integer variables (D) (0x304 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number (0-999 for extended settings)
/// * `values` - Variable values to write (max 118 items)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_double_variables(
    &self,
    start_variable_number: u16,
    values: Vec<i32>,
) -> Result<(), ClientError> {
    let command = WriteMultipleDoubleVariables::new(start_variable_number, values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add batch operations for double precision integer variables:

```rust
/// Get multiple double precision integer variable values
pub fn get_multiple_double_variables(&self, start_variable: u16, count: usize) -> Vec<i32> {
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let var_num = start_variable + u16::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u16::MAX"))
            .expect("Variable index should fit in u16");
        let var_data = self.get_variable(var_num);
        // D variable is 4 bytes (i32)
        let value = var_data.map_or(0_i32, |data| {
            if data.len() >= 4 {
                i32::from_le_bytes([data[0], data[1], data[2], data[3]])
            } else {
                0
            }
        });
        values.push(value);
    }
    values
}

/// Set multiple double precision integer variable values
pub fn set_multiple_double_variables(&mut self, start_variable: u16, values: &[i32]) {
    for (i, &value) in values.iter().enumerate() {
        let var_num = start_variable + u16::try_from(i)
            .map_err(|_| format!("Variable index {i} exceeds u16::MAX"))
            .expect("Variable index should fit in u16");
        self.set_variable(var_num, value.to_le_bytes().to_vec());
    }
}
```

### 4. Plural D Variable Handler Implementation

**File**: `moto-hses-mock/src/handlers/variable.rs`

Add handler for 0x304 command:

```rust
/// Handler for plural double precision integer variable operations (0x304)
pub struct PluralDoubleVarHandler;

impl CommandHandler for PluralDoubleVarHandler {
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
                let values = state.get_multiple_double_variables(start_variable, count as usize);
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
                    let value = i32::from_le_bytes([
                        message.payload[offset],
                        message.payload[offset + 1],
                        message.payload[offset + 2],
                        message.payload[offset + 3],
                    ]);
                    values.push(value);
                }
                
                state.set_multiple_double_variables(start_variable, &values);
                
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

Register handler for 0x304 command in `CommandHandlerRegistry::new()`:

```rust
registry.register(0x304, Box::new(PluralDoubleVarHandler));
```

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/variable.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 118, must be > 0)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + 4 bytes per value)
- Command trait implementation tests (ID, Instance, Attribute, Service)

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/variable_operations.rs` (extend existing)

Add integration tests:

- Read multiple double precision integer variables from various ranges
- Write multiple double precision integer variables
- Verify state changes using read-back verification
- Test boundary conditions (count = 1, count = 118)
- Test count validation (0, 119)
- Test with large i32 values (positive and negative)

### 8. Example Code

**File**: `moto-hses-client/examples/double_variable_operations.rs`

Update existing double variable operations example to include plural operations:

- Reading multiple double precision integer variables (0x304 command)
- Writing multiple double precision integer variables (0x304 command)
- Note: Examples focus on happy path, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x304 command to "Supported Commands" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x304 command to "Supported Commands" section

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

- Attribute is 0 (same as 0x302 and 0x303 plural variable commands)
- Service 0x33 for read, 0x34 for write
- **Count validation is critical**: max 118, must be > 0 (no multiple-of-2 requirement)
- **Data size**: Each D variable data is 4 bytes (i32, little-endian)
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + variable data (4 bytes each)
  - Read response: count (4 bytes) + variable data (4 bytes each)
  - Write response: only count (4 bytes)
- MockState integration with existing variable state management
- Follow the same pattern as 0x303 (Plural Integer Variable Command) for consistency

## Testing Considerations

### Integration Test Patterns

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Read-Back Verification**: After write operations, read back values to verify changes
2. **Boundary Testing**: Test minimum (count=1) and maximum (count=118) values
3. **Value Range Testing**: Test positive, negative, and extreme i32 values
4. **Error Cases**: Test invalid ranges and count violations
```rust
// Example pattern for D variable tests
let _server = create_variable_test_server().await?;
let client = create_test_client().await?;

// Write multiple double precision integer variables
let values = vec![1000000, -2000000, 2147483647, -2147483648];
client.write_multiple_double_variables(0, values.clone()).await?;

// Read back and verify
let read_values = client.read_multiple_double_variables(0, 4).await?;
assert_eq!(read_values, values);

// Test maximum count (118)
let max_values: Vec<i32> = (0..118).map(|i| i * 1000).collect();
client.write_multiple_double_variables(0, max_values.clone()).await?;
let read_max = client.read_multiple_double_variables(0, 118).await?;
assert_eq!(read_max, max_values);
```


## Implementation Feedback for Rules Update

The implementation was completed successfully with no significant issues requiring rule updates. 

### To-dos

- [x] Protocol layer implementation - Add ReadMultipleDoubleVariables and WriteMultipleDoubleVariables structs to variable.rs with proper validation
- [x] Export ReadMultipleDoubleVariables and WriteMultipleDoubleVariables in commands/mod.rs
- [x] Client API implementation - Add read_multiple_double_variables() and write_multiple_double_variables() methods in protocol.rs
- [x] MockState extension - Add get_multiple_double_variables() and set_multiple_double_variables() methods
- [x] Handler implementation - Add PluralDoubleVarHandler in handlers/variable.rs with validation and state management
- [x] Handler registration - Register PluralDoubleVarHandler for 0x304 command
- [x] Create unit tests for ReadMultipleDoubleVariables and WriteMultipleDoubleVariables including validation and serialization
- [x] Create integration tests with read-back verification for read/write operations
- [x] Update example code in double_variable_operations.rs demonstrating plural double variable operations
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x304 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update plan To-dos section status after implementation completion
- [x] Update plan content to reflect actual implementation
- [x] Update Implementation Feedback section with lessons learned