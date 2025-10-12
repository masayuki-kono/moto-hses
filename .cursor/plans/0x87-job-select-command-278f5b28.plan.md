<!-- 278f5b28-196c-491e-bc05-68036230c648 ec7c660f-ef12-4973-bbfb-131a8164d5a2 -->
# 0x87 Job Select Command Implementation

## Overview

Implement the 0x87 command (Job Select Command) to provide functionality for selecting robot jobs for execution.

### Command Specification

- **Command ID**: 0x87
- **Instance**: Type of job to select
  - `1`: Set the job in execution
  - `10`: Set master job (Task 0)
  - `11`: Set master job (Task 1)
  - `12`: Set master job (Task 2)
  - `13`: Set master job (Task 3)
  - `14`: Set master job (Task 4)
  - `15`: Set master job (Task 5)
- **Attribute**: 1 (fixed)
- **Service**: 0x02 (Set_Attribute_All)
- **Payload**: 9 × 32-bit integers (36 bytes)
  - Integers 1-8: Job name (32 characters maximum)
  - Integer 9: Line number (0 to 9999)
- **Response**: Status only (no payload)

## Implementation Steps

### 1. Protocol Layer Implementation

**File**: `moto-hses-proto/src/commands/job.rs`

Add structures and types for Job Select command:

```rust
/// Job select type (instance value for 0x87)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobSelectType {
    InExecution = 1,
    MasterTask0 = 10,
    MasterTask1 = 11,
    MasterTask2 = 12,
    MasterTask3 = 13,
    MasterTask4 = 14,
    MasterTask5 = 15,
}

/// Command for selecting job (0x87)
#[derive(Debug, Clone)]
pub struct JobSelectCommand {
    pub select_type: JobSelectType,
    pub job_name: String,
    pub line_number: u32,
}

impl JobSelectCommand {
    pub fn new(select_type: JobSelectType, job_name: String, line_number: u32) -> Self {
        Self { select_type, job_name, line_number }
    }
}

impl Command for JobSelectCommand {
    type Response = ();
    fn command_id() -> u16 { 0x87 }
    fn instance(&self) -> u16 { self.select_type as u16 }
    fn attribute(&self) -> u8 { 1 }  // Fixed to 1
    fn service(&self) -> u8 { 0x02 }  // Set_Attribute_All
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Job name: 32 bytes (8 x 4-byte integers)
        // Line number: 4 bytes (1 x 4-byte integer)
        // Total: 36 bytes
    }
}
```

**File**: `moto-hses-proto/src/commands/mod.rs`

Add `JobSelectCommand` and `JobSelectType` to exports.

### 2. Client API Implementation

**File**: `moto-hses-client/src/protocol.rs`

Add client API method:

```rust
/// Select job for execution (0x87 command)
///
/// # Arguments
///
/// * `select_type` - Type of job to select
/// * `job_name` - Name of the job to select (max 32 characters)
/// * `line_number` - Starting line number (0 to 9999)
///
/// # Errors
///
/// Returns an error if communication fails or parameters are invalid
pub async fn select_job(
    &self,
    select_type: JobSelectType,
    job_name: impl Into<String>,
    line_number: u32,
) -> Result<(), ClientError> {
    let job_name = job_name.into();
    
    // Validate job name length (max 32 characters)
    if job_name.len() > 32 {
        return Err(ClientError::InvalidParameter("Job name exceeds 32 characters".to_string()));
    }
    
    // Validate line number (0 to 9999)
    if line_number > 9999 {
        return Err(ClientError::InvalidParameter("Line number must be 0-9999".to_string()));
    }
    
    let command = JobSelectCommand::new(select_type, job_name, line_number);
    self.send_command_with_retry(command, Division::Robot).await?;
    Ok(())
}
```

### 3. MockState Extension

**File**: `moto-hses-mock/src/state.rs`

Add selected job state management:

```rust
// Add to MockState struct:
pub selected_job: Option<SelectedJobInfo>,

// Add new struct:
#[derive(Debug, Clone)]
pub struct SelectedJobInfo {
    pub job_name: String,
    pub line_number: u32,
    pub select_type: u16,  // Instance value
}

// Add methods to MockState impl:
pub fn set_selected_job(&mut self, job_name: String, line_number: u32, select_type: u16) {
    self.selected_job = Some(SelectedJobInfo {
        job_name,
        line_number,
        select_type,
    });
}

pub fn get_selected_job(&self) -> Option<&SelectedJobInfo> {
    self.selected_job.as_ref()
}
```

### 4. Handler Implementation

**File**: `moto-hses-mock/src/handlers/job.rs`

Improve existing `JobSelectHandler`:

```rust
impl CommandHandler for JobSelectHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        // Validate instance (select type)
        let select_type = message.sub_header.instance;
        if select_type != 1 && !(10..=15).contains(&select_type) {
            return Err(proto::ProtocolError::InvalidInstance);
        }
        
        // Validate attribute (should be 1)
        if message.sub_header.attribute != 1 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }
        
        // Validate service (should be 0x02 for Set_Attribute_All)
        if message.sub_header.service != 0x02 {
            return Err(proto::ProtocolError::InvalidService);
        }
        
        // Validate payload (should be 36 bytes: 32 bytes for job name + 4 bytes for line number)
        if message.payload.len() != 36 {
            return Err(proto::ProtocolError::InvalidMessage("Invalid payload length".to_string()));
        }
        
        // Parse job name (first 32 bytes, fixed length)
        let job_name_bytes = &message.payload[0..32];
        // Decode using the MockState's text encoding (same as client's encoding)
        let job_name = state.text_encoding.decode(job_name_bytes);
        
        // Parse line number (last 4 bytes, little-endian)
        let line_number = u32::from_le_bytes([
            message.payload[32],
            message.payload[33],
            message.payload[34],
            message.payload[35],
        ]);
        
        // Validate line number (0 to 9999)
        if line_number > 9999 {
            return Err(proto::ProtocolError::InvalidMessage("Line number out of range".to_string()));
        }
        
        // Update state
        state.set_selected_job(job_name, line_number, select_type);
        
        Ok(vec![])
    }
}
```

### 5. Unit Tests

**File**: `moto-hses-proto/src/commands/job.rs`

Add unit tests for Job Select command:

- `JobSelectCommand` struct tests
- Serialization tests (36 bytes: 32 bytes job name + 4 bytes line number)
- Command trait implementation tests (ID, Instance, Attribute, Service)
- Job name encoding tests (handle both ASCII and multi-byte characters)
- Line number validation tests

### 6. Integration Tests

**File**: `moto-hses-client/tests/integration/job_control.rs`

Add to existing job control integration tests:

- Job select command tests (various job names and line numbers)
- MockServer state change verification (using `Arc<MockServer>`)
- All job select types (InExecution, MasterTask0-5)
- Error handling scenario tests (invalid job names, line numbers out of range)
- Job name length validation tests

### 7. Example Code

**File**: `moto-hses-client/examples/job_select.rs` (new file)

Create example demonstrating:

- Selecting a job for execution
- Selecting master jobs for different tasks
- Setting job name and line number
- Error handling for invalid parameters

### 8. Documentation Updates

**File**: `moto-hses-client/README.md`

- Add new feature to supported operations list
- Add new example to examples list
- **⚠️ IMPORTANT**: Do NOT add example execution commands to "Running Examples" section
  - The "Running Examples" section should only contain `alarm_operations` example
  - Adding other example execution commands is not needed

**Files**: `moto-hses-proto/README.md`, `moto-hses-mock/README.md`, and root `README.md`

- Add 0x87 command to "Supported Commands" section

### 9. Quality Checks

Run the following checks in order after implementation:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`
4. `cargo doc --all-features --no-deps`

## Key Implementation Points

- Instance value varies (1, 10-15) based on job select type
- Attribute is 1 (fixed)
- Service is 0x02 (Set_Attribute_All), not 0x10
- Payload is 36 bytes: 32 bytes for job name + 4 bytes for line number
- Job name handling: 32 bytes fixed length, encode/decode using client's text_encoding (SJIS for Japanese)
- Line number range validation (0-9999) is critical
- Response has status only (no payload)
- MockState needs selected job state management (separate from executing job)
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
client.select_job(JobSelectType::InExecution, "TEST.JOB", 0).await?;
wait_for_operation().await;

// Verify state change
let selected_job = server.get_selected_job().await;
assert!(selected_job.is_some());
assert_eq!(selected_job.unwrap().job_name, "TEST.JOB");
assert_eq!(selected_job.unwrap().line_number, 0);
```

## Implementation Feedback & Lessons Learned

None

### To-dos

- [x] Protocol layer implementation - JobSelectCommand, JobSelectType, and JobSelectAttribute in job.rs
- [x] Export JobSelectCommand and related types in commands/mod.rs
- [x] Client API implementation - select_job() method in protocol.rs with validation
- [x] MockState extension - add SelectedJobInfo struct and selected job management methods
- [x] Improve JobSelectHandler with proper validation, payload parsing, and error handling
- [x] Create unit tests for JobSelectCommand including serialization and validation
- [x] Add integration tests in job_control.rs with state verification using Arc<MockServer>
- [x] Create example code in examples/job_select.rs demonstrating various job selection scenarios
- [x] Update README.md files in all crates (client, proto, mock, root) with 0x87 command
- [x] Run quality checks (fmt, clippy, test, doc)
- [x] Update Implementation Feedback section with lessons learned during and after implementation