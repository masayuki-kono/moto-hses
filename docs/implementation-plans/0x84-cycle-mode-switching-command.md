# 0x84 Cycle Mode Command Implementation Plan

## Overview

This document outlines the implementation plan for the 0x84 command (Step / Cycle / Continuous Switching Command) in the Moto-HSES Rust client library.

## Command Specification

### HSES Protocol Details

- **Command ID**: 0x84
- **Instance**: Fixed to 2
- **Attribute**: Fixed to 1
- **Service**: 0x10 (Set_Attribute_Single)
- **Payload**: 4-byte 32-bit integer
  - `1`: STEP
  - `2`: ONE CYCLE
  - `3`: CONTINUOUS
- **Response**: Status only (no payload)

### Request Structure

```
Command: 0x84
Instance: 2 (fixed)
Attribute: 1 (fixed)
Service: 0x10 (Set_Attribute_Single)
Payload: 4 bytes (32-bit integer)
  - 1: STEP
  - 2: ONE CYCLE
  - 3: CONTINUOUS
```

### Response Structure

```
Status: Command execution result
  - 0x00: Success
  - Other: Error
Added Status: Error code (if status is non-zero)
Payload: None
```

## Implementation Plan

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/cycle_mode.rs`

```rust
// Cycle mode switching command (0x84)
#[derive(Debug, Clone)]
pub struct CycleModeSwitchingCommand {
    pub mode: CycleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleMode {
    Step = 1,
    OneCycle = 2,
    Continuous = 3,
}

impl Command for CycleModeSwitchingCommand {
    type Response = ();
    fn command_id() -> u16 { 0x84 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(vec![self.mode as u8, 0, 0, 0])
    }
    fn instance(&self) -> u16 { 2 }
    fn attribute(&self) -> u8 { 1 }
    fn service(&self) -> u8 { 0x10 }
}
```

### 2. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add to `MockState` struct:
```rust
pub struct MockState {
    // ... existing fields ...
    pub cycle_mode: CycleMode,
}

impl MockState {
    pub fn set_cycle_mode(&mut self, mode: CycleMode) {
        self.cycle_mode = mode;
    }
    
    pub fn get_cycle_mode(&self) -> CycleMode {
        self.cycle_mode
    }
}
```

### 3. Mock Server Handler

**File**: `moto-hses-mock/src/handlers/cycle_mode_switching.rs`

```rust
pub struct CycleModeSwitchingHandler;

impl CommandHandler for CycleModeSwitchingHandler {
    fn handle(&self, message: &HsesRequestMessage, state: &mut MockState) -> Result<Vec<u8>, ProtocolError> {
        // Validate instance=2, attribute=1, service=0x10
        // Parse 4-byte payload to get cycle mode
        // Update state with new cycle mode
        // Return empty payload (success response)
    }
}
```

### 4. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

```rust
impl HsesClient {
    /// Set cycle mode (0x84 command)
    pub async fn set_cycle_mode(&self, mode: CycleMode) -> Result<(), ClientError> {
        let command = CycleModeSwitchingCommand { mode };
        let _response = self.send_command_with_retry(command, Division::Robot).await?;
        Ok(())
    }
}
```

### 5. Handler Registry Registration

**File**: `moto-hses-mock/src/handlers/registry.rs`

```rust
// Register 0x84 cycle mode switching handler
registry.insert(0x84, Box::new(CycleModeSwitchingHandler));
```

## Implementation Order

1. **Protocol Layer** (`moto-hses-proto/src/cycle_mode.rs`)
2. **Client API** (`moto-hses-client/src/protocol.rs`)
3. **MockState Extension** (add cycle_mode field)
4. **Handler Implementation** (`moto-hses-mock/src/handlers/cycle_mode_switching.rs`)
5. **Registry Registration**
6. **Remove Old Handler** (SelectCycleHandler from job.rs)
7. **Unit Tests**
8. **Integration Tests** (`moto-hses-client/tests/integration/cycle_mode_control.rs`)
9. **Example Code** (`moto-hses-client/examples/cycle_mode_control.rs`) - Extract from integration tests
10. **Documentation Updates** (`moto-hses-client/README.md`) - Add new example and features
11. **Quality Checks** - Run formatting, clippy, tests, and documentation checks

## Testing Strategy

### Unit Tests

- Command struct tests
- Serialization/deserialization
- Validation logic
- Error handling

### Integration Tests

- **File**: `moto-hses-client/tests/integration/cycle_mode_control.rs`
- **Test Cases**:
  - All cycle modes (Step, OneCycle, Continuous)
  - Error handling scenarios
  - Mock server communication
  - Various usage patterns and combinations
- **Registration**: Add module to `mod.rs` and `integration_tests.rs`

### Example Code

- **File**: `moto-hses-client/examples/cycle_mode_control.rs`
- **Content**: Extract common usage patterns from integration tests
- **Pattern**: Simplified version of integration test scenarios
- **Benefits**: Reuse tested code patterns, ensure examples work correctly

### Documentation Updates

- **File**: `moto-hses-client/README.md`
- **Updates**:
  - Add new example to examples list
  - Add new feature to supported operations
  - Add example execution command
- **Pattern**: Follow existing documentation structure
- **Benefits**: Keep documentation current and comprehensive

## Notes

- Instance and Attribute are fixed values, so validation is critical
- Payload is only 4-byte 32-bit integer
- Response has no payload (status only)
- MockState needs cycle_mode state management

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

## Quality Assurance

### Pre-Push Quality Checks

Before pushing code, run the following checks in order:

1. **Code Formatting** (Fast check)
   ```bash
   cargo fmt --all -- --check
   ```
   - **When**: After each major implementation step
   - **Fix**: `cargo fmt --all` if needed

2. **Static Analysis** (Fast check)
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
   - **When**: After each implementation step
   - **Fix**: Address all warnings before proceeding

3. **Tests** (Slower check)
   ```bash
   cargo test --all-features --workspace
   ```
   - **When**: After unit tests and integration tests
   - **Fix**: Ensure all tests pass

4. **Documentation Build** (Slower check)
   ```bash
   cargo doc --all-features --no-deps
   ```
   - **When**: After documentation updates
   - **Fix**: Address rustdoc warnings

### Recommended Check Points

- **After Step 1-6**: Run formatting and clippy checks
- **After Step 7-8**: Run all tests
- **After Step 10**: Run documentation build
- **Before Push**: Run all quality checks

## Status

- [x] Specification analysis
- [x] Protocol layer implementation
- [x] MockState extension
- [x] Handler implementation
- [x] Registry registration
- [x] Unit tests
- [x] Integration tests
- [x] Example code
- [x] Documentation updates

## Related Files

- `moto-hses-proto/src/cycle_mode.rs` (new) - Contains `CycleModeSwitchingCommand`
- `moto-hses-client/src/protocol.rs` (modify) - Add client API methods
- `moto-hses-client/examples/cycle_mode_control.rs` (new) - Example code
- `moto-hses-client/tests/integration/cycle_mode_control.rs` (new) - Integration tests
- `moto-hses-client/tests/integration/mod.rs` (modify) - Register test module
- `moto-hses-client/README.md` (modify) - Update documentation
- `moto-hses-mock/src/handlers/cycle_mode_switching.rs` (new) - Contains `CycleModeSwitchingHandler`
- `moto-hses-mock/src/state.rs` (modify) - Add cycle_mode field
- `moto-hses-mock/src/handlers/registry.rs` (modify) - Update handler registration
- `moto-hses-mock/src/handlers/job.rs` (modify) - Remove SelectCycleHandler

