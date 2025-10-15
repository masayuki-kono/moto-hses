<!-- 1912404a-dcc9-43b7-993b-9b5a3096b6bb 5d1907cd-de2d-49fd-8443-c81ee51939d1 -->
# 0x300 Plural I/O Data Reading/Writing Command Implementation

## Overview

Implement the 0x300 command (Plural I/O Data Reading/Writing Command) to provide functionality for reading and writing multiple I/O data in a single request, supporting up to 474 I/O data items.

### Command Specification

- **Command ID**: 0x300
- **Instance**: Logical number of the I/O data (starting I/O number)
  - `1 to 512`: Robot user input
  - `1001 to 1512`: Robot user output
  - `2001 to 2128`: External input
  - `2701 to 2956`: Network input
  - `3001 to 3128`: External output
  - `3701 to 3956`: Network output
  - `4001 to 4256`: Robot system input
  - `5001 to 5512`: Robot system output
  - `6001 to 6064`: Interface panel input
  - `7001 to 7999`: Auxiliary relay
  - `8001 to 8512`: Robot control status signal
  - `8701 to 8720`: Pseudo input
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data (only network input signals are writable)
- **Payload**:
  - Byte0-3: Number of I/O data (Maximum: 474, must be multiple of 2)
  - Byte4-N: I/O data bytes (when writing)
  - Each I/O data is 1 byte containing 8 I/O states (bit 0-7)
- **Response**: Same structure as request, with I/O data when reading

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/io.rs`

Add structures for Plural I/O command (extending existing 0x78 implementation):

```rust
/// Read multiple I/O data command (0x300)
#[derive(Debug, Clone)]
pub struct ReadMultipleIo {
    pub start_io_number: u16,
    pub count: u32,  // Number of I/O data (max 474, must be multiple of 2)
}

impl ReadMultipleIo {
    pub fn new(start_io_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate I/O number using existing IoCategory infrastructure
        if !IoCategory::is_valid_io_number(start_io_number) {
            return Err(ProtocolError::InvalidCommand);
        }
        // Validate count (max 474, must be multiple of 2)
        if count == 0 || count > 474 || count % 2 != 0 {
            return Err(ProtocolError::InvalidMessage(format!("Invalid data")));
        }
        Ok(Self { start_io_number, count })
    }
}

impl Command for ReadMultipleIo {
    type Response = Vec<u8>;  // Array of I/O data bytes
    fn command_id() -> u16 { 0x300 }
    fn instance(&self) -> u16 { self.start_io_number }
    fn attribute(&self) -> u8 { 0 }  // Different from 0x78 (which uses 1)
    fn service(&self) -> u8 { 0x33 }  // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple I/O data command (0x300)
#[derive(Debug, Clone)]
pub struct WriteMultipleIo {
    pub start_io_number: u16,
    pub io_data: Vec<u8>,  // Each byte contains 8 I/O states
}

impl WriteMultipleIo {
    pub fn new(start_io_number: u16, io_data: Vec<u8>) -> Result<Self, ProtocolError> {
        // Validate I/O number using existing IoCategory infrastructure
        if !IoCategory::is_valid_io_number(start_io_number) {
            return Err(ProtocolError::InvalidCommand);
        }
        let count = io_data.len();
        // Validate count (max 474, must be multiple of 2)
        if count == 0 || count > 474 || count % 2 != 0 {
            return Err(ProtocolError::InvalidMessage(format!("Invalid data")));
        }
        Ok(Self { start_io_number, io_data })
    }
}

impl Command for WriteMultipleIo {
    type Response = ();
    fn command_id() -> u16 { 0x300 }
    fn instance(&self) -> u16 { self.start_io_number }
    fn attribute(&self) -> u8 { 0 }  // Different from 0x78 (which uses 1)
    fn service(&self) -> u8 { 0x34 }  // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.io_data.len())
            .map_err(|_| ProtocolError::InvalidMessage(format!("I/O data count {} too large for u32 conversion", self.io_data.len())))?;
        let mut payload = count.to_le_bytes().to_vec();
        payload.extend_from_slice(&self.io_data);
        Ok(payload)
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add exports for `ReadMultipleIo` and `WriteMultipleIo`.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API methods:

```rust
/// Read multiple I/O data (0x300 command)
///
/// # Arguments
///
/// * `start_io_number` - Starting I/O number
/// * `count` - Number of I/O data to read (max 474, must be multiple of 2)
///
/// # Returns
///
/// Vector of I/O data bytes, where each byte contains 8 I/O states
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn read_multiple_io(
    &self,
    start_io_number: u16,
    count: u32,
) -> Result<Vec<u8>, ClientError> {
    let command = ReadMultipleIo::new(start_io_number, count)?;
    let response = self.send_command_with_retry(command, Division::Robot).await?;
    
    // Response format: Byte0-3 = count, Byte4-N = I/O data
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
    
    Ok(response[4..].to_vec())
}

/// Write multiple I/O data (0x300 command)
///
/// Note: Only network input signals are writable
///
/// # Arguments
///
/// * `start_io_number` - Starting I/O number (must be network input: 2701-2956)
/// * `io_data` - I/O data bytes to write (max 474, must be multiple of 2)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn write_multiple_io(
    &self,
    start_io_number: u16,
    io_data: Vec<u8>,
) -> Result<(), ClientError> {
    let command = WriteMultipleIo::new(start_io_number, io_data)?;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Verify that existing I/O state management supports multiple I/O operations. May need to add batch operations:

```rust
// Add methods if not already present:
pub fn get_multiple_io_states(&self, start_io_number: u16, count: usize) -> Vec<u8> {
    // Return vector of I/O data bytes
}

pub fn set_multiple_io_states(&mut self, start_io_number: u16, io_data: &[u8]) {
    // Set multiple I/O states from byte array
}
```

### 4. Handler Implementation

**File**: `moto-hses-mock/src/handlers/io.rs`

Add handler for 0x300 command:

```rust
/// Handler for plural I/O operations (0x300)
pub struct PluralIoHandler;

impl CommandHandler for PluralIoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let start_io_number = message.sub_header.instance;
        let service = message.sub_header.service;
        
        // Validate attribute (should be 0)
        if message.sub_header.attribute != 0 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }
        
        // Validate I/O number range
        if !IoCategory::is_valid_io_number(start_io_number) {
            return Err(proto::ProtocolError::InvalidCommand);
        }
        
        // Parse count from payload (first 4 bytes)
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!(
                "Payload too short: {} bytes for start_io {} (need at least 4 bytes)", 
                message.payload.len(), start_io_number
            )));
        }
        
        let count = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);
        
        // Validate count (max 474, must be multiple of 2)
        if count > 474 || count % 2 != 0 {
            return Err(proto::ProtocolError::InvalidMessage(format!("Invalid data")));
        }
        
        match service {
            0x33 => {
                // Read - return count + I/O data
                let io_data = state.get_multiple_io_states(start_io_number, count as usize);
                let mut response = count.to_le_bytes().to_vec();
                response.extend_from_slice(&io_data);
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
                
                // Only network input signals are writable
                if !(2701..=2956).contains(&start_io_number) {
                    return Err(proto::ProtocolError::InvalidCommand);
                }
                
                let io_data = &message.payload[4..];
                state.set_multiple_io_states(start_io_number, io_data);
                
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

Register handler for 0x300 command.

### 6. Unit Tests

**File**: `moto-hses-proto/src/commands/io.rs`

Add unit tests:

- Command struct construction tests
- Count validation tests (max 474, must be multiple of 2)
- Serialization tests for read command (4 bytes for count only)
- Serialization tests for write command (4 bytes count + data bytes)
- Command trait implementation tests (ID, Instance, Attribute, Service)
- I/O number range validation tests

### 7. Integration Tests

**File**: `moto-hses-client/tests/integration/io_operations.rs` (existing or new)

Add integration tests:

- Read multiple I/O data from various ranges
- Write multiple I/O data to network input signals
- Verify state changes using `Arc<MockServer>`
- Test boundary conditions (count = 2, count = 474)
- Test count validation (odd numbers should fail)
- Test write to non-writable ranges (should fail)
- Test I/O number range validation

### 8. Example Code

**File**: `moto-hses-client/examples/io_operations.rs` (existing file - extend)

Add to existing I/O operations example (happy path only):

- Reading multiple I/O data (0x300 command)
- Writing multiple I/O data to network input signals (0x300 command)
- Processing I/O data bytes (extracting individual bit states)
- Demonstrate efficiency gains over individual I/O operations
- Note: Error handling not needed in examples, integration tests cover error cases

### 9. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add 0x300 command to "Supported Commands" section
- Add plural_io_operations to examples list
- **⚠️ IMPORTANT**: Do NOT add example execution commands to "Running Examples" section

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x300 command to "Supported Commands" section

### 10. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

## Key Implementation Points

- Attribute is 0 (different from single I/O command which uses 1)
- Service 0x33 for read, 0x34 for write
- Count validation is critical: max 474, must be multiple of 2
- Payload structure differs for read vs write:
  - Read request: only count (4 bytes)
  - Write request: count (4 bytes) + I/O data
  - Read response: count (4 bytes) + I/O data
  - Write response: only count (4 bytes)
- Each I/O data byte contains 8 I/O states (bit 0-7)
- Only network input signals (2701-2956) are writable
- I/O number validation should use existing `IoCategory` infrastructure
- MockState integration with existing I/O state management

## Testing Considerations

### MockServer State Verification Pattern

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Direct State Access**: Use `Arc<MockServer>` to access server state directly
2. **State Verification**: After each command, verify the expected state change
3. **Batch Operations**: Test multiple I/O operations with various counts
```rust
// Example pattern for state verification tests
let server = Arc::new(MockServerBuilder::new()
    .build().await?);

let server_clone = Arc::clone(&server);
let server_handle = tokio::spawn(async move {
    server_clone.run().await?;
});

// Write multiple I/O data
let io_data = vec![0b10101010, 0b01010101];
client.write_multiple_io(2701, io_data.clone()).await?;
wait_for_operation().await;

// Read back and verify
let read_data = client.read_multiple_io(2701, 2).await?;
assert_eq!(read_data, io_data);
```


## Implementation Feedback & Lessons Learned

### Critical Safety Issue: MAX Fallback Pattern

**Issue**: During implementation, we discovered a dangerous pattern using `unwrap_or(u16::MAX)` for type conversions.

**Problem**: 
```rust
// ❌ DANGEROUS - Silent failure with wrong value
let offset = u16::try_from(i * 8 + bit).unwrap_or(u16::MAX);
```

**Impact**: This pattern can cause severe issues by silently mapping out-of-range values to `u16::MAX`, leading to incorrect I/O number calculations and potential system failures.

**Solution**: Use proper error handling with `map_err`:
```rust
// ✅ SAFE - Explicit error handling
let offset = u16::try_from(i * 8 + bit)
    .map_err(|_| format!("I/O offset {} exceeds u16::MAX", i * 8 + bit))?;
```

**Lesson**: Never use `unwrap_or(MAX_VALUE)` for type conversions. Always use proper error handling to prevent silent failures that can lead to severe system issues.

...

### To-dos

- [x] Protocol layer implementation - Add ReadMultipleIo and WriteMultipleIo structs to io.rs with proper validation
- [x] Export ReadMultipleIo and WriteMultipleIo in commands/mod.rs
- [x] Client API implementation - Add read_multiple_io() and write_multiple_io() methods in protocol.rs
- [x] MockState extension - Add or verify get_multiple_io_states() and set_multiple_io_states() methods
- [x] Handler implementation - Add PluralIoHandler in handlers/io.rs with validation and state management
- [x] Handler registration - Register PluralIoHandler for 0x300 command
- [x] Create unit tests for ReadMultipleIo and WriteMultipleIo including validation and serialization
- [x] Create integration tests with MockServer state verification for read/write operations
- [x] Create example code in examples/plural_io_operations.rs demonstrating usage
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x300 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update Implementation Feedback section with lessons learned during and after implementation