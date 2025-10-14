<!-- 663489d7-ba96-4d40-9ef0-d869b9e82610 eda758f1-9caf-4e55-a5ca-5de78f61238b -->
# 0x301 Plural Register Data Reading/Writing Command Implementation

## Overview

Implement the 0x301 command (Plural Register Data Reading/Writing Command) to provide functionality for reading and writing multiple register data in a single request, supporting up to 237 register data items.

### Command Specification

- **Command ID**: 0x301
- **Instance**: Variable number (first register number with which reading/writing is executed)
  - `0 to 999`: Register number (writable registers: 0 to 559)
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data (Reads out the fixed size specified by the data part)
  - `0x34`: Write plural data (Writes the fixed size specified by the data part)
- **Payload**: Plural register data
  - Byte0-3: Number of register data (Maximum value: 237)
  - Byte4-5: Register data 1 (2 bytes, i16)
  - Byte6-7: Register data 2 (2 bytes, i16)
  - ...
  - Byte(3 + Number * 2 - 1)-Byte(3 + Number * 2): Register data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - Register data section is valid only when writing
    - Each register data is 2 bytes (i16), and the payload contains the number of register data specified by the Number field
- **Response**: Same structure as request, with register data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/register.rs`

Add structures for Plural Register command (similar to existing 0x79 implementation):

```rust
/// Read multiple registers command (0x301)
#[derive(Debug, Clone)]
pub struct ReadMultipleRegisters {
    pub start_register_number: u16,
    pub count: u32,  // Number of register data (max 237)
}

impl ReadMultipleRegisters {
    pub fn new(start_register_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate register number (0-999)
        if start_register_number > 999 {
            return Err(ProtocolError::InvalidCommand);
        }
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidData);
        }
        // Validate range doesn't exceed maximum register number
        let end_register = start_register_number as u32 + count - 1;
        if end_register > 999 {
            return Err(ProtocolError::InvalidData);
        }
        Ok(Self { start_register_number, count })
    }
}

impl Command for ReadMultipleRegisters {
    type Response = Vec<i16>;  // Array of register values
    fn command_id() -> u16 { 0x301 }
    fn instance(&self) -> u16 { self.start_register_number }
    fn attribute(&self) -> u8 { 0 }  // Different from 0x79 (which uses 1)
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple registers command (0x301)
#[derive(Debug, Clone)]
pub struct WriteMultipleRegisters {
    pub start_register_number: u16,
    pub values: Vec<i16>,  // Register values to write
}

impl WriteMultipleRegisters {
    pub fn new(start_register_number: u16, values: Vec<i16>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidData);
        }
        // Validate writable range (0-559 for writes)
        if start_register_number > 559 {
            return Err(ProtocolError::InvalidCommand);
        }
        let end_register = start_register_number as u32 + count as u32 - 1;
        if end_register > 559 {
            return Err(ProtocolError::InvalidData);
        }
        Ok(Self { start_register_number, values })
    }
}

impl Command for WriteMultipleRegisters {
    type Response = ();
    fn command_id() -> u16 { 0x301 }
    fn instance(&self) -> u16 { self.start_register_number }
    fn attribute(&self) -> u8 { 0 }  // Different from 0x79 (which uses 1)
    fn service(&self) -> u8 { 0x34 }  // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = self.values.len() as u32;
        let mut payload = count.to_le_bytes().to_vec();
        for value in &self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        Ok(payload)
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add exports for `ReadMultipleRegisters` and `WriteMultipleRegisters`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods:

```rust
/// Read multiple registers (0x301 command)
///
/// # Arguments
///
/// * `start_register_number` - Starting register number (0-999)
/// * `count` - Number of registers to read (max 237)
///
/// # Returns
///
/// Vector of register values (i16)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_registers(
    &self,
    start_register_number: u16,
    count: u32,
) -> Result<Vec<i16>, ClientError> {
    let command = ReadMultipleRegisters::new(start_register_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    
    // Response format: Byte0-3 = count, Byte4-N = register data (2 bytes each)
    if response.len() < 4 {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization("Response too short".to_string())
        ));
    }
    
    let response_count = u32::from_le_bytes([
        response[0], response[1], response[2], response[3]
    ]);
    
    if response_count != count {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization("Count mismatch".to_string())
        ));
    }
    
    // Parse register values (2 bytes each)
    let expected_len = 4 + (count as usize * 2);
    if response.len() != expected_len {
        return Err(ClientError::ProtocolError(
            ProtocolError::Deserialization("Invalid response length".to_string())
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

/// Write multiple registers (0x301 command)
///
/// Note: Only registers 0-559 are writable
///
/// # Arguments
///
/// * `start_register_number` - Starting register number (0-559)
/// * `values` - Register values to write (max 237)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_registers(
    &self,
    start_register_number: u16,
    values: Vec<i16>,
) -> Result<(), ClientError> {
    let command = WriteMultipleRegisters::new(start_register_number, values)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add batch operations for registers (if not already present):

```rust
/// Get multiple register values
pub fn get_multiple_registers(&self, start_register: u16, count: usize) -> Vec<i16> {
    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let reg_num = start_register + i as u16;
        values.push(self.get_register(reg_num));
    }
    values
}

/// Set multiple register values
pub fn set_multiple_registers(&mut self, start_register: u16, values: &[i16]) {
    for (i, &value) in values.iter().enumerate() {
        let reg_num = start_register + i as u16;
        self.set_register(reg_num, value);
    }
}
```

### 4. Handler File Restructuring

**File**: `moto-hses-mock/src/handlers/register.rs` (new file)

Create new register handler file and move existing `RegisterHandler` from `io.rs`:

```rust
//! Register command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for single register operations (0x79)
pub struct RegisterHandler;

impl CommandHandler for RegisterHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let reg_number = message.sub_header.instance;
        let service = message.sub_header.service;

        // Validate register number range (0-999 for read, 0-559 for write)
        if reg_number > 999 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Invalid register number: {reg_number} (must be 0-999)"
            )));
        }

        match service {
            0x0e => {
                // Read - return 2 bytes (i16)
                let value = state.get_register(reg_number);
                Ok(value.to_le_bytes().to_vec())
            }
            0x10 => {
                // Write - validate writable range (0-559)
                if reg_number > 559 {
                    return Err(proto::ProtocolError::InvalidCommand);
                }
                
                if message.payload.len() >= 2 {
                    let value = i16::from_le_bytes([message.payload[0], message.payload[1]]);
                    state.set_register(reg_number, value);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
```

### 5. Plural Register Handler Implementation

**File**: `moto-hses-mock/src/handlers/register.rs` (same file)

Add handler for 0x301 command:

```rust
/// Handler for plural register operations (0x301)
pub struct PluralRegisterHandler;

impl CommandHandler for PluralRegisterHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_register = message.sub_header.instance;
        let service = message.sub_header.service;
        
        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }
        
        // Validate register number range (0-999)
        if start_register > 999 {
            return Err(proto::ProtocolError::InvalidCommand);
        }
        
        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage("Payload too short".to_string()));
        }
        
        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);
        
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(proto::ProtocolError::InvalidData);
        }
        
        // Validate range doesn't exceed maximum register number
        let end_register = start_register as u32 + count - 1;
        if end_register > 999 {
            return Err(proto::ProtocolError::InvalidData);
        }
        
        match service {
            0x33 => {
                // Read - return count + register data
                let values = state.get_multiple_registers(start_register, count as usize);
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
                    return Err(proto::ProtocolError::InvalidMessage("Invalid payload length".to_string()));
                }
                
                // Only registers 0-559 are writable
                if start_register > 559 || end_register > 559 {
                    return Err(proto::ProtocolError::InvalidCommand);
                }
                
                // Parse register values
                let mut values = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let offset = 4 + i * 2;
                    let value = i16::from_le_bytes([
                        message.payload[offset],
                        message.payload[offset + 1],
                    ]);
                    values.push(value);
                }
                
                state.set_multiple_registers(start_register, &values);
                
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

Register handler for 0x301 command in `CommandHandlerRegistry::new()`.

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/register.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 237, must be > 0)
- Register number range validation tests (0-999 for read, 0-559 for write)
- Range overflow validation tests (start + count - 1 <= 999)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + 2 bytes per value)
- Command trait implementation tests (ID, Instance, Attribute, Service)

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/register_operations.rs` (extend existing file)

Add integration tests:

- Read multiple registers from various ranges
- Write multiple registers to writable range (0-559)
- Verify state changes using read-back verification
- Test boundary conditions (count = 1, count = 237)
- Test range validation (start + count - 1 <= 999)
- Test write to non-writable range (560-999, should fail)
- Test write to registers crossing writable boundary (should fail)
- Test maximum safe count calculations

### 8. Example Code

**File**: `moto-hses-client/examples/register_operations.rs` (extend existing or create new)

Add to register operations example (happy path only):

- Reading multiple registers (0x301 command)
- Writing multiple registers to writable range (0x301 command)
- Demonstrate efficiency gains over individual register operations
- Note: Error handling not needed in examples, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x301 command to "Supported Commands" section
- Add plural_register_operations to examples list (if creating new example)
- **⚠️ IMPORTANT**: Do NOT add example execution commands to "Running Examples" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x301 command to "Supported Commands" section

### 10. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

## Key Implementation Points

- Attribute is 0 (different from single register command 0x79 which uses 1)
- Service 0x33 for read, 0x34 for write
- Count validation is critical: max 237, must be > 0
- Register number validation: 0-999 for read, 0-559 for write
- Range validation: start_register + count - 1 must not exceed limits
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + register data (2 bytes each)
  - Read response: count (4 bytes) + register data (2 bytes each)
  - Write response: only count (4 bytes)
- Each register data is 2 bytes (i16), little-endian
- MockState integration with existing register state management
- Follow the same pattern as 0x300 (Plural I/O Command) for consistency

## Testing Considerations

### Integration Test Patterns

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Read-Back Verification**: After write operations, read back values to verify changes
2. **Boundary Testing**: Test minimum (count=1) and maximum (count=237) values
3. **Range Validation**: Test register number ranges and count limits
4. **Error Cases**: Test invalid ranges, non-writable registers, and count violations
```rust
// Example pattern for register tests
let _server = create_register_test_server().await?;
let client = create_test_client().await?;

// Write multiple registers
let values = vec![100, 200, 300];
client.write_multiple_registers(0, values.clone()).await?;

// Read back and verify
let read_values = client.read_multiple_registers(0, 3).await?;
assert_eq!(read_values, values);

// Test writable range boundary (559 is last writable register)
let boundary_values = vec![999];
client.write_multiple_registers(559, boundary_values).await?;

// Test non-writable range (should fail)
match client.write_multiple_registers(560, vec![123]).await {
    Ok(()) => panic!("Should fail for non-writable register"),
    Err(_) => {} // Expected
}
```


## Implementation Feedback & Lessons Learned

### Issues Encountered During Implementation

None

### To-dos

- [x] Protocol layer implementation - Add ReadMultipleRegisters and WriteMultipleRegisters structs to register.rs with proper validation
- [x] Export ReadMultipleRegisters and WriteMultipleRegisters in commands/mod.rs
- [x] Client API implementation - Add read_multiple_registers() and write_multiple_registers() methods in protocol.rs
- [x] MockState extension - Add get_multiple_registers() and set_multiple_registers() methods
- [x] Handler implementation - Add PluralRegisterHandler in handlers/register.rs with validation and state management
- [x] Handler registration - Register PluralRegisterHandler for 0x301 command
- [x] Create unit tests for ReadMultipleRegisters and WriteMultipleRegisters including validation and serialization
- [x] Create integration tests with read-back verification for read/write operations
- [x] Create or extend example code demonstrating plural register operations
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x301 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update Implementation Feedback section with lessons learned during and after implementation