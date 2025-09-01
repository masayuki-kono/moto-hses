# Client API

## Overview

This document describes the API design for the Rust HSES client library, inspired by the C++ reference implementation and incorporating design insights from the Python implementation. The API is designed to be type-safe, async-first, and easy to use while maintaining high performance.

## Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time safety
2. **Async-First**: Modern async/await patterns with Tokio
3. **Template-Based**: Type-safe command definitions inspired by C++ templates
4. **Zero-Copy**: Efficient memory usage where possible
5. **Error Handling**: Comprehensive error handling with thiserror
6. **Efficient Batch Operations**: Support for optimized multiple variable operations
7. **Design Insights**: API design incorporating successful patterns from existing implementations

## Core API Design

### Client Creation

```rust
use moto_hses_client::{HsesClient, ClientConfig};
use std::time::Duration;

// Basic client creation
let client = HsesClient::new("192.168.1.100:10040").await?;

// With custom configuration
let config = ClientConfig {
    timeout: Duration::from_millis(500),
    retry_count: 3,
    retry_delay: Duration::from_millis(100),
    buffer_size: 8192,
};
let client = HsesClient::new_with_config("192.168.1.100:10040", config).await?;
```

### Enhanced Variable Operations

```rust
use moto_hses_client::{HsesClient, Variable, VariableType};

// Single variable operations - Type-safe with Variable<T> objects
let mut var_b0 = Variable::with_default(VariableType::Byte, 0);
client.read_variable(&mut var_b0).await?;
println!("B0 = {}", var_b0.value);

let mut var_i1 = Variable::new(VariableType::Integer, 1, 0i16);
client.read_variable(&mut var_i1).await?;
println!("I1 = {}", var_i1.value);

let mut var_r2 = Variable::new(VariableType::Real, 2, 0.0f32);
client.read_variable(&mut var_r2).await?;
println!("R2 = {}", var_r2.value);

// String variables
let mut var_s3 = Variable::new(VariableType::String, 3, String::new());
client.read_variable(&mut var_s3).await?;
println!("S3 = '{}'", var_s3.value);

// Position variables
let mut var_p4 = Variable::new(VariableType::RobotPosition, 4, Position::default());
client.read_variable(&mut var_p4).await?;
println!("P4 = {:?}", var_p4.value);

// Writing variables
let var_write = Variable::new(VariableType::Integer, 10, 1000i16);
client.write_variable(&var_write).await?;

// Multiple variable operations with automatic optimization
let mut vars = vec![
    Variable::with_default(VariableType::Integer, 0),
    Variable::with_default(VariableType::Integer, 1),
    Variable::with_default(VariableType::Integer, 2),
    Variable::with_default(VariableType::Integer, 3),
];

// Efficient batch read - automatically groups consecutive variables
client.read_variables(&mut vars).await?;
for var in &vars {
    println!("I{} = {}", var.index, var.value);
}

// Batch write
let write_vars = vec![
    Variable::new(VariableType::Integer, 10, 100i16),
    Variable::new(VariableType::Integer, 11, 200i16),
    Variable::new(VariableType::Integer, 12, 300i16),
];
client.write_variables(&write_vars).await?;
```

### Status and Position Operations

```rust
// Read robot status with enhanced error handling
let status = client.read_status().await?;
println!("Robot running: {}", status.is_running());
println!("Servo on: {}", status.is_servo_on());
println!("Alarm: {}", status.has_alarm());
println!("Teach mode: {}", status.is_teach_mode());
println!("Play mode: {}", status.is_play_mode());
println!("Remote mode: {}", status.is_remote_mode());

// Read current position
let position = client.read_position(1, CoordinateSystemType::RobotPulse).await?;
println!("Current position: {:?}", position);

// Access position properties based on type
match position {
    Position::Pulse(pulse_pos) => {
        println!("Joint values: {:?}", pulse_pos.joints());
        println!("Tool number: {}", pulse_pos.tool());
    }
    Position::Cartesian(cart_pos) => {
        println!("X: {}, Y: {}, Z: {}", cart_pos.x(), cart_pos.y(), cart_pos.z());
        println!("RX: {}, RY: {}, RZ: {}", cart_pos.rx(), cart_pos.ry(), cart_pos.rz());
        println!("Tool number: {}", cart_pos.tool());
    }
}

// Read position error data
let error_data = client.read_position_error(1).await?;
println!("Axis 1 error: {}", error_data.axis_1);
println!("Axis 2 error: {}", error_data.axis_2);
// ... other axes

// Read torque data
let torque_data = client.read_torque(1).await?;
println!("Axis 1 torque: {}", torque_data.axis_1);
println!("Axis 2 torque: {}", torque_data.axis_2);
// ... other axes
```

### File Operations

```rust
// Get file list
let files = client.get_file_list("JBI").await?;
for file in files {
    println!("File: {}", file);
}

// Send file to controller
client.send_file("local_file.jbi").await?;

// Receive file from controller
client.recv_file("remote_file.jbi", "./downloads/").await?;

// Delete file on controller
client.delete_file("unwanted_file.jbi").await?;
```

### System Information and Management

```rust
// Get system information
let system_info = client.get_system_info(11, SystemInfoType::SystemSoftwareVersion).await?;
println!("System software version: {}", system_info.software_version);
println!("Model name: {}", system_info.model);
println!("Parameter version: {}", system_info.parameter_version);

// Get management time
let time_info = client.get_management_time(ManagementTimeType::ControlPowerOn).await?;
println!("Start time: {}", time_info.start);
println!("Elapsed time: {}", time_info.elapse);
```

### Power and Control Operations

```rust
// Power control
client.switch_power(PowerType::Servo, PowerSwitch::On).await?;

// Cycle selection
client.select_cycle(CycleType::Continuous).await?;

// Movement operations
let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));
client.move_to_position(
    MoveType::JointAbsolutePos,
    CoordinateSystem::Base,
    SpeedClass::Percent,
    50.0,
    position,
    0, 0, 1, 0, 0, 0
).await?;
```

### Enhanced Error Handling

```rust
use moto_hses_client::{HsesClient, ClientError, ProtocolError};

match client.read_variable(&mut var).await {
    Ok(_) => println!("Variable read successfully"),
    Err(ClientError::Proto(ProtocolError::InvalidVariableType)) => {
        println!("Invalid variable type specified");
    }
    Err(ClientError::Proto(ProtocolError::PositionError(msg))) => {
        println!("Position error: {}", msg);
    }
    Err(ClientError::Io(e)) => {
        println!("IO error: {}", e);
    }
    Err(ClientError::Timeout) => {
        println!("Operation timed out");
    }
}
```

### Configuration and Advanced Features

```rust
// Client configuration with retry logic
let config = ClientConfig {
    timeout: Duration::from_millis(1000),
    retry_count: 3,
    retry_delay: Duration::from_millis(100),
    buffer_size: 8192,
    enable_debug: true,
    connection_pool_size: 5,
};

let client = HsesClient::connect_with_config("192.168.1.100:10040", config).await?;

// Connection pooling for high-performance applications
let pool = HsesClientPool::new(config, "192.168.1.100:10040").await?;
let client = pool.get().await?;

// Event-driven status monitoring
let mut status_stream = client.status_stream().await?;
while let Some(status) = status_stream.next().await {
    match status {
        Ok(status) => {
            if status.has_alarm() {
                println!("Alarm detected!");
            }
        }
        Err(e) => println!("Status error: {}", e),
    }
}
```

## Performance Considerations

### Efficient Variable Operations

The library automatically optimizes multiple variable operations by grouping consecutive variables of the same type, incorporating efficient design patterns:

```rust
// This will be optimized to use the plural command (0x33)
let mut vars = vec![
    Variable::with_default(VariableType::Integer, 0),
    Variable::with_default(VariableType::Integer, 1),
    Variable::with_default(VariableType::Integer, 2),
    Variable::with_default(VariableType::Integer, 3),
];
client.read_variables(&mut vars).await?;
```

The optimization includes:

1. **Consecutive Variable Grouping**: Automatically groups consecutive variables of the same type
2. **Plural Commands**: Uses HSE plural commands (0x33) for reading multiple variables in a single network call
3. **Automatic Padding**: Handles padding requirements for 1-byte variable types

### Connection Management

```rust
// Automatic connection management with pooling
let pool = HsesClientPool::new(config, "192.168.1.100:10040").await?;

// Concurrent operations
let futures: Vec<_> = (0..10).map(|i| {
    let pool = pool.clone();
    async move {
        let client = pool.get().await?;
        let mut var = Variable::with_default(VariableType::Integer, i);
        client.read_variable(&mut var).await?;
        Ok::<_, ClientError>(var.value)
    }
}).collect();

let results = futures::future::join_all(futures).await;
```

## Design Insights

The API design incorporates successful patterns from existing implementations:

### Variable Object Pattern

The `Variable<T>` object pattern provides type safety while maintaining flexibility:

```rust
// Type-safe variable creation and operations
let mut var = Variable::with_default(VariableType::Integer, 0);
client.read_variable(&mut var).await?;
println!("Value: {}", var.value);
```

### Efficient Batch Operations

The library implements efficient batch operations that automatically optimize network usage:

```rust
// Automatic optimization of multiple variable operations
let mut vars = vec![
    Variable::with_default(VariableType::Integer, 0),
    Variable::with_default(VariableType::Integer, 1),
    Variable::with_default(VariableType::Integer, 2),
    Variable::with_default(VariableType::Integer, 3),
];

// Automatically uses plural command for efficiency
client.read_variables(&mut vars).await?;
```

This approach reduces network round trips and improves performance for applications that need to read multiple variables.

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

| API Method                                                        | Command ID | Description                                                                 |
| ----------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------- |
| `read_alarm_data()`                                               | 0x70       | Alarm data reading command                                                  |
| `read_alarm_history()`                                            | 0x71       | Alarm history reading command                                               |
| `read_status()`                                                   | 0x72       | Status information reading command                                          |
| `get_executing_job_info()`                                        | 0x73       | Executing job information reading command                                   |
| `read_axis_config()`                                              | 0x74       | Axis configuration information reading command                              |
| `read_position()`                                                 | 0x75       | Robot position data reading command                                         |
| `read_position_error()`                                           | 0x76       | Position error reading command                                              |
| `read_torque()`                                                   | 0x77       | Torque data reading command                                                 |
| `read_io()`, `write_io()`                                         | 0x78       | I/O data reading / writing command                                          |
| `read_register()`, `write_register()`                             | 0x79       | Register data reading / writing command                                     |
| `read_variable<u8>()`, `write_variable()`                         | 0x7A       | Byte variable (B) reading / writing command                                 |
| `read_variable<i16>()`, `write_variable()`                        | 0x7B       | Integer type variable (I) reading / writing command                         |
| `read_variable<i32>()`, `write_variable()`                        | 0x7C       | Double precision integer type variable (D) reading / writing command        |
| `read_variable<f32>()`, `write_variable()`                        | 0x7D       | Real type variable (R) reading / writing command                            |
| `read_variable<String>()`, `write_variable()`                     | 0x7E       | Character type variable (S) reading / writing command                       |
| `read_variable<Position>()`, `write_variable()`                   | 0x7F       | Robot position type variable (P) reading / writing command                  |
| `read_variable<BasePosition>()`, `write_variable()`               | 0x80       | Base position type variable (BP) reading / writing command                  |
| `read_variable<ExternalAxis>()`, `write_variable()`               | 0x81       | External axis type variable (EX) reading / writing command                  |
| `reset_alarm()`                                                   | 0x82       | Alarm reset / error cancel command                                          |
| `set_hold()`, `set_servo()`                                       | 0x83       | HOLD / servo ON/OFF command                                                 |
| `set_execution_mode()`                                            | 0x84       | Step / cycle / continuous switching command                                 |
| `display_message()`                                               | 0x85       | Character string display command to the programming pendant                 |
| `start_job()`                                                     | 0x86       | Start-up (job START) command                                                |
| `select_job()`                                                    | 0x87       | Job select command                                                          |
| `get_management_time()`                                           | 0x88       | Management time acquiring command                                           |
| `get_system_info()`                                               | 0x89       | System information acquiring command                                        |
| `read_multiple_io()`                                              | 0x300      | Plural I/O data reading / writing command                                   |
| `read_multiple_registers()`                                       | 0x301      | Plural register data reading / writing command                              |
| `read_multiple_variables<u8>()`                                   | 0x302      | Plural byte type variable (B) reading / writing command                     |
| `read_multiple_variables<i16>()`                                  | 0x303      | Plural integer type variable (I) reading / writing command                  |
| `read_multiple_variables<i32>()`                                  | 0x304      | Plural double precision integer type variable (D) reading / writing command |
| `read_multiple_variables<f32>()`                                  | 0x305      | Plural real type variable (R) reading / writing command                     |
| `read_multiple_variables<String>()`                               | 0x306      | Plural character type variable (S) reading / writing command                |
| `read_multiple_variables<Position>()`                             | 0x307      | Plural robot position type variable (P) reading / writing command           |
| `read_multiple_variables<BasePosition>()`                         | 0x308      | Plural base position type variable (BP) reading / writing command           |
| `read_multiple_variables<ExternalAxis>()`                         | 0x309      | Plural external axis type variable (EX) reading / writing command           |
| `read_alarm_data_with_subcode()`                                  | 0x30A      | Alarm data reading command (for applying the sub code character strings)    |
| `read_alarm_history_with_subcode()`                               | 0x30B      | Alarm history reading command (for applying the sub character strings)      |
| `read_multiple_32byte_strings()`                                  | 0x30C      | 32-byte character type variable (S) multiple reading / writing command      |
| `move_cartesian()`                                                | 0x8A       | Move instruction command (Type Cartesian coordinates)                       |
| `move_pulse()`                                                    | 0x8B       | Move instruction command (Type Pulse)                                       |
| `read_32byte_string_variable()`, `write_32byte_string_variable()` | 0x8C       | 32-byte character type variable (S) reading / writing command               |
| `read_encoder_temperature()`                                      | 0x0411     | Encoder temperature reading command                                         |
| `read_converter_temperature()`                                    | 0x0413     | Converter temperature reading command                                       |

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
