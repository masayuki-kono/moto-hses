<!-- de4a0b07-908b-45c8-9bfd-4ad536db66cf 9f5055f4-59cb-4587-bd78-02006c9fbe56 -->
# 0x302 Plural Byte Type Variable (B) Reading/Writing Command Implementation

## Overview

Implement the 0x302 command (Plural Byte Type Variable (B) Reading/Writing Command) to provide functionality for reading and writing multiple byte type variables in a single request, supporting up to 474 B variable data items (extended setting support).

### Command Specification

- **Command ID**: 0x302
- **Instance**: Variable number (first variable number with which reading/writing is executed)
  - Standard setting: 0-99
  - Extended setting: 0-999 (varies by configuration)
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data (Reads out the fixed size specified by the data part)
  - `0x34`: Write plural data (Writes the fixed size specified by the data part)
- **Payload**: Plural B variable data
  - Byte0-3: Number of B variable data (Maximum value: 474, must be specified as a multiple of 2)
  - Byte4: B variable data 1 (1 byte, u8)
  - Byte5: B variable data 2 (1 byte, u8)
  - ...
  - Byte(3 + Number): B variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - B variable data section is valid only when writing
    - Each B variable data is 1 byte (u8), and the payload contains the number of B variable data specified by the Number field
- **Response**: Same structure as request, with B variable data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/variable.rs`

Add structures for Plural B Variable command (following the same pattern as 0x301 in register.rs):

```rust
/// Read multiple byte variables (B) command (0x302)
#[derive(Debug, Clone)]
pub struct ReadMultipleByteVariables {
    pub start_variable_number: u8,
    pub count: u32,  // Number of B variable data (max 474, must be multiple of 2)
}

impl ReadMultipleByteVariables {
    pub fn new(start_variable_number: u8, count: u32) -> Result<Self, ProtocolError> {
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
    type Response = Vec<u8>;  // Array of B variable values
    fn command_id() -> u16 { 0x302 }
    fn instance(&self) -> u16 { u16::from(self.start_variable_number) }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple byte variables (B) command (0x302)
#[derive(Debug, Clone)]
pub struct WriteMultipleByteVariables {
    pub start_variable_number: u8,
    pub values: Vec<u8>,  // B variable values to write
}

impl WriteMultipleByteVariables {
    pub fn new(start_variable_number: u8, values: Vec<u8>) -> Result<Self, ProtocolError> {
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
    fn command_id() -> u16 { 0x302 }
    fn instance(&self) -> u16 { u16::from(self.start_variable_number) }
    fn attribute(&self) -> u8 { 0 }  // Fixed to 0 for plural commands
    fn service(&self) -> u8 { 0x34 }  // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len())
            .map_err(|_| ProtocolError::InvalidMessage(format!(
                "Values count {} too large for u32 conversion", self.values.len()
            )))?;
        let mut payload = count.to_le_bytes().to_vec();
        payload.extend_from_slice(&self.values);
        Ok(payload)
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add exports for `ReadMultipleByteVariables` and `WriteMultipleByteVariables`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods:

```rust
/// Read multiple byte variables (B) (0x302 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number
/// * `count` - Number of variables to read (max 474, must be multiple of 2)
///
/// # Returns
///
/// Vector of variable values (u8)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_byte_variables(
    &self,
    start_variable_number: u8,
    count: u32,
) -> Result<Vec<u8>, ClientError> {
    let command = ReadMultipleByteVariables::new(start_variable_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    
    // Response format: Byte0-3 = count, Byte4-N = variable data (1 byte each)
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
    
    // Parse variable values (1 byte each)
    let expected_len = 4 + count as usize;
    if response.len() != expected_len {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization(format!("Invalid response length: got {} bytes, expected {}", response.len(), expected_len))
        ));
    }
    
    let values = response[4..].to_vec();
    Ok(values)
}

/// Write multiple byte variables (B) (0x302 command)
///
/// # Arguments
///
/// * `start_variable_number` - Starting variable number
/// * `values` - Variable values to write (max 474, must be multiple of 2 in length)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_byte_variables(
    &self,
    start_variable_number: u8,
    values: Vec<u8>,
) -> Result<(), ClientError> {
    let command = WriteMultipleByteVariables::new(start_variable_number, values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add batch operations for byte variables (if not already present):

```rust
/// Get multiple byte variable values
pub fn get_multiple_byte_variables(&self, start_variable: u8, count: usize) -> Vec<u8> {
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let var_num = start_variable + u8::try_from(i).expect("Variable index should fit in u8");
        let var_data = self.get_variable(var_num);
        values.push(var_data.map_or(0, |data| data.first().copied().unwrap_or(0)));
    }
    values
}

/// Set multiple byte variable values
pub fn set_multiple_byte_variables(&mut self, start_variable: u8, values: &[u8]) {
    for (i, &value) in values.iter().enumerate() {
        let var_num = start_variable + u8::try_from(i).expect("Variable index should fit in u8");
        self.set_variable(var_num, vec![value]);
    }
}
```

### 4. Plural B Variable Handler Implementation

**File**: `moto-hses-mock/src/handlers/variable.rs`

Add handler for 0x302 command:

```rust
/// Handler for plural byte variable operations (0x302)
pub struct PluralByteVarHandler;

impl CommandHandler for PluralByteVarHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_variable = u8::try_from(message.sub_header.instance).map_err(|_| {
            proto::ProtocolError::InvalidInstance(format!(
                "Variable index {} too large for u8 conversion", message.sub_header.instance
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
                "Payload too short: {} bytes for start_variable {} (need at least 4 bytes)", 
                message.payload.len(), start_variable
            )));
        }
        
        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);
        
        // Validate count (max 474, must be > 0, must be multiple of 2)
        if count == 0 || count > 474 || !count.is_multiple_of(2) {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} for start_variable {start_variable} (must be 1-474 and multiple of 2)"
            )));
        }
        
        
        match service {
            0x33 => {
                // Read - return count + variable data
                let values = state.get_multiple_byte_variables(start_variable, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                response.extend_from_slice(&values);
                Ok(response)
            }
            0x34 => {
                // Write - validate payload length and update state
                let expected_len = 4 + count as usize;
                if message.payload.len() != expected_len {
                    return Err(proto::ProtocolError::InvalidMessage(format!(
                        "Invalid payload length: got {} bytes, expected {}", message.payload.len(), expected_len
                    )));
                }
                
                // Parse variable values (1 byte each)
                let values = message.payload[4..].to_vec();
                
                state.set_multiple_byte_variables(start_variable, &values);
                
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

Register handler for 0x302 command in `CommandHandlerRegistry::new()`:

```rust
registry.register(0x302, Box::new(PluralByteVarHandler));
```

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/variable.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 474, must be > 0, must be multiple of 2)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + 1 byte per value)
- Command trait implementation tests (ID, Instance, Attribute, Service)

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/variable_operations.rs` (extend or create)

Add integration tests:

- Read multiple byte variables from various ranges
- Write multiple byte variables
- Verify state changes using read-back verification
- Test boundary conditions (count = 2, count = 474)
- Test count validation (must be multiple of 2)
- Test odd count values (should fail)
- Test maximum safe count calculations

### 8. Example Code

**File**: `moto-hses-client/examples/variable_operations.rs` (extend existing or create new)

Add to variable operations example (happy path only):

- Reading multiple byte variables (0x302 command)
- Writing multiple byte variables (0x302 command)
- Demonstrate efficiency gains over individual variable operations
- Note: Error handling not needed in examples, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x302 command to "Supported Commands" section
- Add variable_operations to examples list (if creating new example)
- **⚠️ IMPORTANT**: Do NOT add example execution commands to "Running Examples" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x302 command to "Supported Commands" section

### 10. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

### 11. Post-Implementation Tasks

- Update To-dos section status in this plan document (`.cursor/plans/`)
- Update plan content to reflect actual implementation details and any changes made during development
- Update Implementation Feedback section with lessons learned

## Key Implementation Points

- Attribute is 0 (different from single byte variable command 0x7A which uses 1)
- Service 0x33 for read, 0x34 for write
- **Count validation is critical**: max 474, must be > 0, **must be multiple of 2** (extended setting support)
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + variable data (1 byte each)
  - Read response: count (4 bytes) + variable data (1 byte each)
  - Write response: only count (4 bytes)
- Each B variable data is 1 byte (u8), little-endian for count
- MockState integration with existing variable state management
- Follow the same pattern as 0x301 (Plural Register Command) for consistency

## Testing Considerations

### Integration Test Patterns

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Read-Back Verification**: After write operations, read back values to verify changes
2. **Boundary Testing**: Test minimum (count=2) and maximum (count=474) values
3. **Multiple of 2 Validation**: Test that odd counts are rejected
4. **Error Cases**: Test invalid ranges and count violations
```rust
// Example pattern for B variable tests
let _server = create_variable_test_server().await?;
let client = create_test_client().await?;

// Write multiple byte variables (count must be multiple of 2)
let values = vec![10, 20, 30, 40];
client.write_multiple_byte_variables(0, values.clone()).await?;

// Read back and verify
let read_values = client.read_multiple_byte_variables(0, 4).await?;
assert_eq!(read_values, values);

// Test count must be multiple of 2 (should fail)
match client.read_multiple_byte_variables(0, 3).await {
    Ok(_) => panic!("Should fail for odd count"),
    Err(_) => {} // Expected
}

// Test maximum count (474)
let max_values: Vec<u8> = (0..474).map(|i| u8::try_from(i % 256).expect("Should fit in u8")).collect();
client.write_multiple_byte_variables(0, max_values).await?;
```


### Proposed Rules Updates

## To-dos

- [x] Protocol layer implementation - Add ReadMultipleByteVariables and WriteMultipleByteVariables structs to variable.rs with proper validation
- [x] Export ReadMultipleByteVariables and WriteMultipleByteVariables in commands/mod.rs
- [x] Client API implementation - Add read_multiple_byte_variables() and write_multiple_byte_variables() methods in protocol.rs
- [x] MockState extension - Add get_multiple_byte_variables() and set_multiple_byte_variables() methods
- [x] Handler implementation - Add PluralByteVarHandler in handlers/variable.rs with validation and state management
- [x] Handler registration - Register PluralByteVarHandler for 0x302 command
- [x] Create unit tests for ReadMultipleByteVariables and WriteMultipleByteVariables including validation and serialization
- [x] Create integration tests with read-back verification for read/write operations
- [x] Create or extend example code demonstrating plural byte variable operations
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x302 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update plan To-dos section status after implementation completion
- [x] Update plan content to reflect actual implementation
- [x] Update Implementation Feedback section with lessons learned