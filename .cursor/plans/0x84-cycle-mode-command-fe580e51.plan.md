<!-- fe580e51-883f-44ef-a714-d076f99e226f 6cbf8244-b94a-4bf5-870c-9fcdb311fe6f -->
# 0x84 Cycle Mode Command Implementation

## Overview

Implement the 0x84 command (Step / Cycle / Continuous Switching Command) to switch robot operation modes between STEP, ONE CYCLE, and CONTINUOUS.

### Command Specification

- **Command ID**: 0x84
- **Instance**: 2 (fixed)
- **Attribute**: 1 (fixed)
- **Service**: 0x10 (Set_Attribute_Single)
- **Payload**: 4-byte 32-bit integer (1=STEP, 2=ONE CYCLE, 3=CONTINUOUS)
- **Response**: Status only (no payload)

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/cycle_mode.rs` (new file)

```rust
// Implement CycleMode enum and CycleModeSwitchingCommand struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleMode {
    Step = 1,
    OneCycle = 2,
    Continuous = 3,
}

#[derive(Debug, Clone)]
pub struct CycleModeSwitchingCommand {
    pub mode: CycleMode,
}

impl Command for CycleModeSwitchingCommand {
    type Response = ();
    fn command_id() -> u16 { 0x84 }
    fn instance(&self) -> u16 { 2 }
    fn attribute(&self) -> u8 { 1 }
    fn service(&self) -> u8 { 0x10 }
}
```

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

```rust
impl HsesClient {
    pub async fn set_cycle_mode(&self, mode: CycleMode) -> Result<(), ClientError> {
        let command = CycleModeSwitchingCommand { mode };
        self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

- Add `cycle_mode: CycleMode` field
- Implement `set_cycle_mode()` and `get_cycle_mode()` methods

### 4. Handler Implementation

**File**: `moto-hses-mock/src/handlers/cycle_mode_switching.rs` (new file)

- Implement `CycleModeSwitchingHandler`
- Validate Instance=2, Attribute=1, Service=0x10
- Parse 4-byte payload to get CycleMode
- Update MockState and return empty response

### 5. Handler Registration

**File**: `moto-hses-mock/src/handlers/registry.rs`

- Register handler for 0x84 command

### 6. Remove Old Handler

**File**: `moto-hses-mock/src/handlers/job.rs`

- Remove old `SelectCycleHandler`

### 7. Unit Tests

- Command struct tests
- Serialization/deserialization tests
- Validation logic tests
- Error handling tests

### 8. Integration Tests

**File**: `moto-hses-client/tests/integration/cycle_mode_control.rs` (new file)

- Test all cycle modes (Step, OneCycle, Continuous)
- Test error handling scenarios
- Test communication with MockServer
- Verify MockState changes using `Arc<MockServer>`

**Files**: `moto-hses-client/tests/integration/mod.rs` and `integration_tests.rs`

- Register test module

### 9. Example Code

**File**: `moto-hses-client/examples/cycle_mode_control.rs` (new file)

- Extract common patterns from integration tests
- Provide concise and practical usage examples

### 10. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add new feature to supported operations list
- Add new example to examples list
- Add example execution commands

### 11. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

## Key Implementation Points

- Instance and Attribute are fixed values, so validation is critical
- Payload is only 4-byte 32-bit integer
- Response has status only (no payload)
- MockState needs cycle mode state management
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
    .with_initial_state(initial_value)
    .build().await?);

let server_clone = Arc::clone(&server);
let server_handle = tokio::spawn(async move {
    server_clone.run().await?;
});

// Execute command
client.execute_command().await?;
wait_for_operation().await;

// Verify state change
let current_state = server.get_state().await;
assert_eq!(current_state, expected_value);
```

### Applicable Commands

This testing pattern should be considered for:

- Commands that modify MockState fields
- Commands without read-back capability in the protocol
- Commands where state verification is critical for functionality

### To-dos

- [x] Protocol layer implementation (cycle_mode.rs)
- [x] Client API implementation (protocol.rs)
- [x] MockState extension (state.rs)
- [x] Handler implementation (cycle_mode_switching.rs)
- [x] Handler registration and old handler removal
- [x] Unit tests creation
- [x] Integration tests creation
- [x] Example code creation
- [x] Documentation updates (README.md)
- [x] Quality checks (fmt, clippy, test, doc)