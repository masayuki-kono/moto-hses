# Client API

## Overview

This document describes the API design for the Rust HSES client library, inspired by the C++ reference implementation. The API is designed to be type-safe, async-first, and easy to use while maintaining high performance.

## Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time safety
2. **Async-First**: Modern async/await patterns with Tokio
3. **Template-Based**: Type-safe command definitions inspired by C++ templates
4. **Zero-Copy**: Efficient memory usage where possible
5. **Error Handling**: Comprehensive error handling with thiserror

## Core API Design

### Client Creation

```rust
use moto_hses_client::{HsesClient, ClientConfig};
use std::time::Duration;

// Basic client creation
let client = HsesClient::connect("192.168.1.100:10040").await?;

// With custom configuration
let config = ClientConfig {
    timeout: Duration::from_millis(500),
    retry_count: 3,
    retry_delay: Duration::from_millis(100),
    buffer_size: 8192,
};
let client = HsesClient::connect_with_config("192.168.1.100:10040", config).await?;
```

### Variable Operations

```rust
use moto_hses_client::{HsesClient, VariableType};

// Type-safe variable reading - Commands 0x7A-0x81
let value: u8 = client.read_variable(0).await?;        // Command 0x7A (B variable)
let value: i16 = client.read_variable(1).await?;       // Command 0x7B (I variable)
let value: i32 = client.read_variable(2).await?;       // Command 0x7C (D variable)
let value: f32 = client.read_variable(3).await?;       // Command 0x7D (R variable)
let value: String = client.read_variable(4).await?;    // Command 0x7E (S variable)
let value: Position = client.read_variable(5).await?;  // Command 0x7F (P variable)

// Type-safe variable writing - Commands 0x7A-0x81
client.write_variable(0, 42u8).await?;                 // Command 0x7A (B variable)
client.write_variable(1, 1000i16).await?;              // Command 0x7B (I variable)
client.write_variable(2, 1000000i32).await?;           // Command 0x7C (D variable)
client.write_variable(3, 3.14f32).await?;              // Command 0x7D (R variable)
client.write_variable(4, "Hello Robot".to_string()).await?; // Command 0x7E (S variable)

// Position writing - Command 0x7F
let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));
client.write_variable(5, position).await?;             // Command 0x7F (P variable)
```

### Status and Position Operations

```rust
// Read robot status - Command 0x72
let status = client.read_status().await?;
println!("Robot running: {}", status.is_running());
println!("Servo on: {}", status.is_servo_on());
println!("Alarm: {}", status.has_alarm());
println!("Teach mode: {}", status.is_teach_mode());
println!("Play mode: {}", status.is_play_mode());
println!("Remote mode: {}", status.is_remote_mode());

// Read alarm data - Command 0x70
let alarm_data = client.read_alarm_data(1, AlarmAttribute::AlarmCode).await?;
println!("Latest alarm code: {}", alarm_data.alarm_code);

let alarm_name = client.read_alarm_data(1, AlarmAttribute::AlarmName).await?;
println!("Latest alarm name: {}", alarm_name);

// Read system information - Command 0x89
let system_info = client.get_system_info(11, SystemInfoType::SystemSoftwareVersion).await?;
println!("System software version: {}", system_info);

let model_info = client.get_system_info(11, SystemInfoType::ModelName).await?;
println!("Model name: {}", model_info);

// Read current position
let position = client.read_position(1, CoordinateSystemType::RobotPulse).await?;
match position {
    Position::Pulse(pulse_pos) => {
        println!("Joint 1: {}", pulse_pos.joints()[0]);
        println!("Tool: {}", pulse_pos.tool());
    }
    Position::Cartesian(cart_pos) => {
        println!("X: {}, Y: {}, Z: {}", cart_pos.x(), cart_pos.y(), cart_pos.z());
        println!("RX: {}, RY: {}, RZ: {}", cart_pos.rx(), cart_pos.ry(), cart_pos.rz());
    }
}

// Read position with specific coordinate system
let base_position = client.read_position(1, CoordinateSystemType::BasePulse).await?;
let cartesian_position = client.read_position(1, CoordinateSystemType::RobotCartesian).await?;
```

### Batch Operations

```rust
// Batch reading - Commands 0x302-0x309
let values = client.read_variables(&[
    (0, VariableType::Byte),      // Command 0x302 (Plural B variables)
    (1, VariableType::Integer),   // Command 0x303 (Plural I variables)
    (2, VariableType::Double),    // Command 0x304 (Plural D variables)
    (3, VariableType::Real),      // Command 0x305 (Plural R variables)
    (4, VariableType::String),    // Command 0x306 (Plural S variables)
]).await?;

// Batch writing - Commands 0x302-0x309
let variables = vec![
    (0, 42u8),                    // Command 0x302 (Plural B variables)
    (1, 1000i16),                 // Command 0x303 (Plural I variables)
    (2, 1000000i32),              // Command 0x304 (Plural D variables)
    (3, 3.14f32),                 // Command 0x305 (Plural R variables)
    (4, "Hello Robot".to_string()), // Command 0x306 (Plural S variables)
];
client.write_variables(&variables).await?;
```

### I/O Operations

```rust
use moto_hses_client::IoType;

// Read I/O data - Command 0x78
let input_value = client.read_io(IoType::RobotUserInput, 1).await?;      // Command 0x78
let output_value = client.read_io(IoType::RobotUserOutput, 1001).await?; // Command 0x78
let network_input = client.read_io(IoType::NetworkInput, 2501).await?;   // Command 0x78

// Write I/O data (network input only) - Command 0x78
client.write_io(IoType::NetworkInput, 2501, true).await?;   // Command 0x78
client.write_io(IoType::NetworkInput, 2502, false).await?;  // Command 0x78

// Batch I/O operations - Command 0x300
let io_values = client.read_multiple_io(&[
    (IoType::RobotUserInput, 1),    // Command 0x300 (Plural I/O data)
    (IoType::RobotUserInput, 2),    // Command 0x300 (Plural I/O data)
    (IoType::RobotUserOutput, 1001), // Command 0x300 (Plural I/O data)
]).await?;
```

### File Operations

```rust
// File list operations - returns Vec<String> of filenames - Service 0x32
let job_files: Vec<String> = client.read_file_list("*.JOB")
    .on_progress(|bytes_received| println!("Received: {} bytes", bytes_received))
    .await?;                                                                 // Service 0x32 (File list acquiring)

println!("Found JOB files:");
for filename in &job_files {
    println!("  - {}", filename);
}

// Read file content as string (for JOB files) - Service 0x16
let job_content: String = client.read_file("TEST.JOB")
    .on_progress(|bytes_received| println!("Received: {} bytes", bytes_received))
    .await?;                                                                 // Service 0x16 (File saving command)

println!("JOB file content:");
println!("{}", job_content);

// Read file content as bytes (for binary files) - Service 0x16
let binary_content: Vec<u8> = client.read_file_as_bytes("DATA.BIN")
    .on_progress(|bytes_received| println!("Received: {} bytes", bytes_received))
    .await?;                                                                 // Service 0x16 (File saving command)

// Write file content (string for JOB files) - Service 0x15
let new_job_content = r#"
PROGRAM TEST
    MOV P1
    MOV P2
    END
"#.to_string();

client.write_file("NEW_TEST.JOB", new_job_content)
    .on_progress(|bytes_sent, bytes_total| {
        println!("Sent: {}/{} bytes", bytes_sent, bytes_total);
    })
    .await?;                                                                 // Service 0x15 (File loading command)

// Write file content (bytes for binary files) - Service 0x15
let binary_data = vec![0x01, 0x02, 0x03, 0x04];
client.write_file_as_bytes("DATA.BIN", binary_data)
    .on_progress(|bytes_sent, bytes_total| {
        println!("Sent: {}/{} bytes", bytes_sent, bytes_total);
    })
    .await?;                                                                 // Service 0x15 (File loading command)

client.delete_file("TEST.JOB").await?;                                      // Service 0x09 (File delete)
```

### Job Operations

```rust
// Job execution - Command 0x86 (Start-up command)
client.start_job().await?;

// Job selection - Command 0x87 (Job select command)
client.select_job("MAIN.JOB", 1).await?; // job_name, line_number

// Get executing job information - Command 0x73
let job_info = client.get_executing_job_info(1, JobInfoAttribute::JobName).await?;
println!("Current job: {}", job_info);

let line_number = client.get_executing_job_info(1, JobInfoAttribute::LineNumber).await?;
println!("Line number: {}", line_number);

let step_number = client.get_executing_job_info(1, JobInfoAttribute::StepNumber).await?;
println!("Step number: {}", step_number);

let speed_override = client.get_executing_job_info(1, JobInfoAttribute::SpeedOverride).await?;
println!("Speed override: {}", speed_override);

// Get all job information at once
let all_job_info = client.get_all_executing_job_info(1).await?;
println!("Job: {}, Line: {}, Step: {}, Speed: {}",
    all_job_info.job_name,
    all_job_info.line_number,
    all_job_info.step_number,
    all_job_info.speed_override
);
```

### Move Operations

```rust
use moto_hses_client::{Speed, SpeedType, MoveFrame};

// Cartesian move - Command 0x8A
let target = CartesianPosition::new(
    100.0, 200.0, 300.0,  // X, Y, Z
    0.0, 0.0, 0.0,        // RX, RY, RZ
    CoordinateSystem::Base,
    PoseConfiguration::default(),
    1
);

let speed = Speed::new(SpeedType::Translation, 100); // 10.0 mm/s
client.move_cartesian(1, target, speed).await?;      // Command 0x8A (Move instruction - Cartesian)

// Pulse move - Command 0x8B
let target = PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1);
let speed = Speed::new(SpeedType::Joint, 50); // 0.5% of max speed
client.move_pulse(1, target, speed).await?;           // Command 0x8B (Move instruction - Pulse)
```

## Type Definitions

### Variable Types

```rust
// Type-safe variable type system
pub trait VariableType: Send + Sync + 'static {
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError>;
}

impl VariableType for u8 {
    fn command_id() -> u16 { 0x7A }
    // Implementation for B variables
}

impl VariableType for i16 {
    fn command_id() -> u16 { 0x7B }
    // Implementation for I variables
}

impl VariableType for i32 {
    fn command_id() -> u16 { 0x7C }
    // Implementation for D variables
}

impl VariableType for f32 {
    fn command_id() -> u16 { 0x7D }
    // Implementation for R variables
}

impl VariableType for Position {
    fn command_id() -> u16 { 0x7F }
    // Implementation for P variables
}
```

### Position Types

```rust
#[derive(Debug, Clone)]
pub struct PulsePosition {
    joints: [i32; 8],
    tool: i32,
    size: u8, // Number of active joints
}

impl PulsePosition {
    pub fn new(joints: [i32; 8], tool: i32) -> Self {
        Self { joints, tool, size: 8 }
    }

    pub fn joints(&self) -> &[i32] {
        &self.joints[..self.size as usize]
    }

    pub fn tool(&self) -> i32 {
        self.tool
    }
}

#[derive(Debug, Clone)]
pub struct CartesianPosition {
    x: f64,
    y: f64,
    z: f64,
    rx: f64,
    ry: f64,
    rz: f64,
    frame: CoordinateSystem,
    configuration: PoseConfiguration,
    tool: i32,
}

impl CartesianPosition {
    pub fn new(
        x: f64, y: f64, z: f64,
        rx: f64, ry: f64, rz: f64,
        frame: CoordinateSystem,
        configuration: PoseConfiguration,
        tool: i32,
    ) -> Self {
        Self { x, y, z, rx, ry, rz, frame, configuration, tool }
    }

    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }
    pub fn rx(&self) -> f64 { self.rx }
    pub fn ry(&self) -> f64 { self.ry }
    pub fn rz(&self) -> f64 { self.rz }
    pub fn frame(&self) -> CoordinateSystem { self.frame }
    pub fn configuration(&self) -> PoseConfiguration { self.configuration }
    pub fn tool(&self) -> i32 { self.tool }
}

#[derive(Debug, Clone)]
pub enum Position {
    Pulse(PulsePosition),
    Cartesian(CartesianPosition),
}

impl Position {
    pub fn is_pulse(&self) -> bool {
        matches!(self, Position::Pulse(_))
    }

    pub fn is_cartesian(&self) -> bool {
        matches!(self, Position::Cartesian(_))
    }

    pub fn as_pulse(&self) -> Option<&PulsePosition> {
        match self {
            Position::Pulse(pos) => Some(pos),
            _ => None,
        }
    }

    pub fn as_cartesian(&self) -> Option<&CartesianPosition> {
        match self {
            Position::Cartesian(pos) => Some(pos),
            _ => None,
        }
    }
}
```

### Status Types

```rust
#[derive(Debug, Clone)]
pub struct Status {
    pub step: bool,
    pub one_cycle: bool,
    pub continuous: bool,
    pub running: bool,
    pub speed_limited: bool,
    pub teach: bool,
    pub play: bool,
    pub remote: bool,
    pub teach_pendant_hold: bool,
    pub external_hold: bool,
    pub command_hold: bool,
    pub alarm: bool,
    pub error: bool,
    pub servo_on: bool,
}

impl Status {
    pub fn is_running(&self) -> bool { self.running }
    pub fn is_servo_on(&self) -> bool { self.servo_on }
    pub fn has_alarm(&self) -> bool { self.alarm }
    pub fn has_error(&self) -> bool { self.error }
    pub fn is_teach_mode(&self) -> bool { self.teach }
    pub fn is_play_mode(&self) -> bool { self.play }
    pub fn is_remote_mode(&self) -> bool { self.remote }
}
```

### Speed and Configuration Types

```rust
#[derive(Debug, Clone, Copy)]
pub enum SpeedType {
    Joint,       // 0.01% of max speed
    Translation, // 0.1 mm/s
    Rotation     // 0.1 degrees/s
}

#[derive(Debug, Clone)]
pub struct Speed {
    pub speed_type: SpeedType,
    pub value: u32,
}

impl Speed {
    pub fn new(speed_type: SpeedType, value: u32) -> Self {
        Self { speed_type, value }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PoseConfiguration {
    pub no_flip: bool,
    pub lower_arm: bool,
    pub back: bool,
    pub high_r: bool,
    pub high_t: bool,
    pub high_s: bool,
}

impl Default for PoseConfiguration {
    fn default() -> Self {
        Self {
            no_flip: true,
            lower_arm: false,
            back: false,
            high_r: false,
            high_t: false,
            high_s: false,
        }
    }
}
```

### Job Operation Types

`````rust
#[derive(Debug, Clone, Copy)]
pub enum JobInfoAttribute {
    JobName = 1,
    LineNumber = 2,
    StepNumber = 3,
    SpeedOverride = 4,
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub job_name: String,
    pub line_number: u32,
    pub step_number: u32,
    pub speed_override: u32,
}

impl JobInfo {
    pub fn new(job_name: String, line_number: u32, step_number: u32, speed_override: u32) -> Self {
        Self {
            job_name,
            line_number,
            step_number,
            speed_override,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TaskType {
    Master = 1,
    SubTask1 = 2,
    SubTask2 = 3,
    SubTask3 = 4,
    SubTask4 = 5,
    SubTask5 = 6,
}

#[derive(Debug, Clone, Copy)]
pub enum JobSelectType {
    SetExecutionJob = 1,
    SetMasterJobTask0 = 10,
    SetMasterJobTask1 = 11,
    SetMasterJobTask2 = 12,
    SetMasterJobTask3 = 13,
    SetMasterJobTask4 = 14,
    SetMasterJobTask5 = 15,
}

#[derive(Debug, Clone, Copy)]
pub enum AlarmAttribute {
    AlarmCode = 1,
    AlarmData = 2,
    AlarmType = 3,
    AlarmTime = 4,
    AlarmName = 5,
}

#[derive(Debug, Clone)]
pub struct AlarmData {
    pub alarm_code: u32,
    pub alarm_data: u32,
    pub alarm_type: u32,
    pub alarm_time: String,
    pub alarm_name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SystemInfoType {
    SystemSoftwareVersion = 1,
    ModelName = 2,
    ParameterVersion = 3,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub software_version: String,
    pub model_name: String,
    pub parameter_version: String,
}

### File Operation Types

````rust
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub modified_time: Option<SystemTime>,
}

impl FileInfo {
    pub fn new(filename: String, size: u64) -> Self {
        Self {
            filename,
            size,
            modified_time: None,
        }
    }

    pub fn with_modified_time(mut self, time: SystemTime) -> Self {
        self.modified_time = Some(time);
        self
    }
}

#[derive(Debug, Clone)]
pub struct FileOperationBuilder<T> {
    client: HsesClient,
    filename: String,
    content: T,
    progress_callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
}

impl<T> FileOperationBuilder<T> {
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    pub async fn execute(self) -> Result<T, ClientError> {
        // Implementation would handle the actual file operation
        // with progress reporting
        todo!("Implementation needed")
    }
}

// Specialized builders for different content types
pub struct StringFileBuilder {
    inner: FileOperationBuilder<String>,
}

impl StringFileBuilder {
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.inner = self.inner.on_progress(callback);
        self
    }

    pub async fn execute(self) -> Result<String, ClientError> {
        self.inner.execute().await
    }
}

pub struct BytesFileBuilder {
    inner: FileOperationBuilder<Vec<u8>>,
}

impl BytesFileBuilder {
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.inner = self.inner.on_progress(callback);
        self
    }

    pub async fn execute(self) -> Result<Vec<u8>, ClientError> {
        self.inner.execute().await
    }
}

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] moto_hses_proto::ProtocolError),

    #[error("Invalid variable: {0}")]
    InvalidVariable(String),

    #[error("Invalid job: {0}")]
    InvalidJob(String),

    #[error("System error: {0}")]
    SystemError(String),

    #[error("Robot error: {0}")]
    RobotError(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File access denied: {0}")]
    FileAccessDenied(String),

    #[error("Invalid file encoding: {0}")]
    InvalidFileEncoding(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Invalid magic bytes")]
    InvalidMagicBytes,

    #[error("Invalid header size")]
    InvalidHeaderSize,

    #[error("Invalid command: {0}")]
    InvalidCommand(u16),

    #[error("Invalid variable type: {0}")]
    InvalidVariableType(String),
}
`````

## Protocol Command Mapping

This API is designed based on the following HSES protocol commands:

### Robot Commands (Division = 0x01)

| API Method                                          | Command ID | Description                                                                 |
| --------------------------------------------------- | ---------- | --------------------------------------------------------------------------- |
| `read_alarm_data()`                                 | 0x70       | Alarm data reading command                                                  |
| `read_alarm_history()`                              | 0x71       | Alarm history reading command                                               |
| `read_status()`                                     | 0x72       | Status information reading command                                          |
| `get_executing_job_info()`                          | 0x73       | Executing job information reading command                                   |
| `read_axis_config()`                                | 0x74       | Axis configuration information reading command                              |
| `read_position()`                                   | 0x75       | Robot position data reading command                                         |
| `read_position_error()`                             | 0x76       | Position error reading command                                              |
| `read_torque()`                                     | 0x77       | Torque data reading command                                                 |
| `read_io()`, `write_io()`                           | 0x78       | I/O data reading / writing command                                          |
| `read_register()`, `write_register()`               | 0x79       | Register data reading / writing command                                     |
| `read_variable<u8>()`, `write_variable()`           | 0x7A       | Byte variable (B) reading / writing command                                 |
| `read_variable<i16>()`, `write_variable()`          | 0x7B       | Integer type variable (I) reading / writing command                         |
| `read_variable<i32>()`, `write_variable()`          | 0x7C       | Double precision integer type variable (D) reading / writing command        |
| `read_variable<f32>()`, `write_variable()`          | 0x7D       | Real type variable (R) reading / writing command                            |
| `read_variable<String>()`, `write_variable()`       | 0x7E       | Character type variable (S) reading / writing command                       |
| `read_variable<Position>()`, `write_variable()`     | 0x7F       | Robot position type variable (P) reading / writing command                  |
| `read_variable<BasePosition>()`, `write_variable()` | 0x80       | Base position type variable (BP) reading / writing command                  |
| `read_variable<ExternalAxis>()`, `write_variable()` | 0x81       | External axis type variable (EX) reading / writing command                  |
| `reset_alarm()`                                     | 0x82       | Alarm reset / error cancel command                                          |
| `set_hold()`, `set_servo()`                         | 0x83       | HOLD / servo ON/OFF command                                                 |
| `set_execution_mode()`                              | 0x84       | Step / cycle / continuous switching command                                 |
| `display_message()`                                 | 0x85       | Character string display command to the programming pendant                 |
| `start_job()`                                       | 0x86       | Start-up (job START) command                                                |
| `select_job()`                                      | 0x87       | Job select command                                                          |
| `get_management_time()`                             | 0x88       | Management time acquiring command                                           |
| `get_system_info()`                                 | 0x89       | System information acquiring command                                        |
| `read_multiple_io()`                                | 0x300      | Plural I/O data reading / writing command                                   |
| `read_multiple_registers()`                         | 0x301      | Plural register data reading / writing command                              |
| `read_multiple_variables<u8>()`                     | 0x302      | Plural byte type variable (B) reading / writing command                     |
| `read_multiple_variables<i16>()`                    | 0x303      | Plural integer type variable (I) reading / writing command                  |
| `read_multiple_variables<i32>()`                    | 0x304      | Plural double precision integer type variable (D) reading / writing command |
| `read_multiple_variables<f32>()`                    | 0x305      | Plural real type variable (R) reading / writing command                     |
| `read_multiple_variables<String>()`                 | 0x306      | Plural character type variable (S) reading / writing command                |
| `read_multiple_variables<Position>()`               | 0x307      | Plural robot position type variable (P) reading / writing command           |
| `read_multiple_variables<BasePosition>()`           | 0x308      | Plural base position type variable (BP) reading / writing command           |
| `read_multiple_variables<ExternalAxis>()`           | 0x309      | Plural external axis type variable (EX) reading / writing command           |
| `read_alarm_data_with_subcode()`                    | 0x30A      | Alarm data reading command (for applying the sub code character strings)    |
| `read_alarm_history_with_subcode()`                 | 0x30B      | Alarm history reading command (for applying the sub character strings)      |
| `move_cartesian()`                                  | 0x8A       | Move instruction command (Type Cartesian coordinates)                       |
| `move_pulse()`                                      | 0x8B       | Move instruction command (Type Pulse)                                       |

### File Commands (Division = 0x02)

| API Method         | Service | Description                        |
| ------------------ | ------- | ---------------------------------- |
| `delete_file()`    | 0x09    | File delete                        |
| `write_file()`     | 0x15    | File loading command (PC to FS100) |
| `read_file()`      | 0x16    | File saving command (FS100 to PC)  |
| `read_file_list()` | 0x32    | File list acquiring command        |

## Best Practices

### Error Handling

1. Always handle errors explicitly
2. Use appropriate error types for different scenarios
3. Implement retry logic for transient errors
4. Log errors with sufficient context

### Performance

1. Use batch operations for multiple variables
2. Reuse client instances when possible
3. Set appropriate timeouts
4. Use connection pooling for high-frequency operations

### Type Safety

1. Use strongly-typed variable operations
2. Leverage Rust's type system for compile-time safety
3. Use enums for variable types and status values
4. Implement proper validation for all inputs

### Async Patterns

1. Use async/await consistently
2. Handle timeouts appropriately
3. Use proper error propagation
4. Avoid blocking operations in async contexts
