<!-- f46a9182-712f-423d-a989-b2b67e876dfc a774db77-6522-495f-bf9a-b347ca3c3d4a -->
# 0x86 Start-up (Job Start) Command Implementation

## Overview

Implement the 0x86 command (Start-up / Job Start Command) to provide functionality for starting robot job execution.

### Command Specification

- **Command ID**: 0x86
- **Instance**: 1 (fixed)
- **Attribute**: 1 (fixed)
- **Service**: 0x10 (Set_Attribute_Single)
- **Payload**: 4-byte 32-bit integer (fixed value 1)
- **Response**: Status only (no payload)

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/job.rs`

Add `JobStartCommand` struct to existing file:

```rust
/// Command for starting job execution (0x86)
#[derive(Debug, Clone)]
pub struct JobStartCommand;

impl Command for JobStartCommand {
    type Response = ();
    fn command_id() -> u16 { 0x86 }
    fn instance(&self) -> u16 { 1 }
    fn attribute(&self) -> u8 { 1 }
    fn service(&self) -> u8 { 0x10 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Fixed value 1 as 32-bit integer (little-endian)
        Ok(vec![1, 0, 0, 0])
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add `JobStartCommand` to exports.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API method:

```rust
/// Start job execution (0x86 command)
///
/// # Errors
///
/// Returns an error if communication fails
pub async fn start_job(&self) -> Result<(), ClientError> {
    let command = JobStartCommand;
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension (verify existing implementation)

**File**: `moto-hses-mock/src/state.rs`

Verify that `set_running()` and `get_running()` are already implemented and adjust if necessary.

### 4. Handler Implementation

**File**: `moto-hses-mock/src/handlers/job.rs`

Improve existing `JobStartHandler`:

- Validate Instance=1, Attribute=1, Service=0x10
- Validate 4-byte payload (fixed value 1)
- Update MockState and return empty response
- Add error handling
```rust
impl CommandHandler for JobStartHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        // Validate instance, attribute, service
        if message.sub_header.instance != 1 {
            return Err(proto::ProtocolError::InvalidCommand);
        }
        if message.sub_header.attribute != 1 {
            return Err(proto::ProtocolError::InvalidService);
        }
        if message.sub_header.service != 0x10 {
            return Err(proto::ProtocolError::InvalidService);
        }
        
        // Validate payload (should be 4 bytes with value 1)
        if message.payload.len() != 4 {
            return Err(proto::ProtocolError::InvalidMessage(format!("Invalid payload length: got {} bytes, expected 4", message.payload.len())));
        }
        
        state.set_running(true);
        Ok(vec![])
    }
}
```

### 5. Unit Tests

**File**: `moto-hses-proto/src/commands/job.rs`

- `JobStartCommand` struct tests
- Serialization tests (4 bytes, fixed value 1)
- Command trait implementation tests (ID, Instance, Attribute, Service)

### 6. Integration Tests

**File**: `moto-hses-client/tests/integration/job_control.rs` (new file)

- Job start command tests
- MockServer state change verification (using `Arc<MockServer>`)
- Error handling scenario tests
- Multiple job start tests

**Files**: `moto-hses-client/tests/integration/mod.rs` and `integration_tests.rs`

- Register test module

### 7. Example Code

**File**: `moto-hses-client/examples/job_start.rs` (new file)

- Extract common patterns from integration tests
- Provide concise and practical usage examples
- Include error handling

### 8. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add new feature to supported operations list
- Add new example to examples list
- **⚠️ IMPORTANT**: Do NOT add example execution commands to "Running Examples" section
  - The "Running Examples" section should only contain `alarm_operations` example
  - Adding other example execution commands (like `job_start`) is not needed and should be avoided
  - This prevents README.md from becoming cluttered with unnecessary example calls

### 9. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

## Key Implementation Points

- Instance, Attribute, and Service are fixed values, so validation is critical
- Payload is only 4-byte 32-bit integer with fixed value 1
- Response has status only (no payload)
- MockState needs running state management
- Integration tests should verify state directly using `Arc<MockServer>`

## Testing Considerations

### MockServer State Verification Pattern

For commands that modify MockServer state, integration tests should verify that the state changes correctly:

1. **Direct State Access**: Use `Arc<MockServer>` to access server state directly
2. **State Verification**: After each command, verify the expected state change
3. **Pattern for Other Commands**: This pattern should be applied to other state-modifying commands
```rust
// Example pattern for state verification tests
let server = Arc::new(MockServerBuilder::new()
    .build().await?);

let server_clone = Arc::clone(&server);
let server_handle = tokio::spawn(async move {
    server_clone.run().await?;
});

// Execute command
client.start_job().await?;
wait_for_operation().await;

// Verify state change
let is_running = server.get_running().await;
assert_eq!(is_running, true);
```

## Implementation Feedback & Lessons Learned

### Issues Encountered During Implementation

1. **README.md Example Commands Issue**

   - **Problem**: Accidentally added `job_start` example execution command to "Running Examples" section
   - **Solution**: Only `alarm_operations` should be in "Running Examples" section
   - **Prevention**: Added explicit warning in Documentation Updates section

2. **Handler Validation Complexity**

   - **Problem**: Initial handler implementation was too simple and didn't validate all required fields
   - **Solution**: Need comprehensive validation for Instance, Attribute, Service, and Payload
   - **Lesson**: Always validate all protocol fields, not just the command ID

3. **MockState Integration**

   - **Problem**: MockState methods for running state were already implemented but needed verification
   - **Solution**: Verify existing MockState methods before implementing new ones
   - **Lesson**: Check existing MockState implementation before adding new methods

4. **Integration Test State Verification**

   - **Problem**: State verification pattern needed refinement for reliable testing
   - **Solution**: Use `Arc<MockServer>` with proper async handling and wait patterns
   - **Lesson**: State verification tests need careful timing and proper async patterns

5. **Unnecessary Unit Tests**

   - **Problem**: Created tests for trivial functionality that is guaranteed at compile time
   - **Solution**: Removed `test_job_start_command_new` and `test_job_start_command_response_type`
   - **Lesson**: Only test actual logic and behavior, not compile-time guarantees
   - **Keep**: Tests for command trait values and serialization are valuable

6. **README.md Update Oversight**

   - **Problem**: Forgot to update README.md files in moto-hses-proto, moto-hses-mock, and root repository
   - **Solution**: Added 0x86 command to "Supported Commands" section in all README.md files
   - **Lesson**: Always update ALL README.md files when adding new commands
   - **Files to update**: moto-hses-client/README.md, moto-hses-proto/README.md, moto-hses-mock/README.md, and root README.md

### To-dos

- [x] Protocol layer implementation - JobStartCommand in job.rs
- [x] Export JobStartCommand in commands/mod.rs
- [x] Client API implementation - start_job() method in protocol.rs
- [x] Improve JobStartHandler with proper validation and error handling
- [x] Create unit tests for JobStartCommand
- [x] Create integration tests in job_control.rs with state verification
- [x] Create example code in examples/job_start.rs
- [x] Run quality checks (fmt, clippy, test, doc)