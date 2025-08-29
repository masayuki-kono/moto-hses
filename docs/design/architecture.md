# Architecture Design

## Overview

This document describes the architecture of the Rust HSES client library, inspired by the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) and incorporating design insights from the Python implementation [fs100](https://github.com/fih-mobile/fs100).

## Core Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time safety
2. **Async-First**: Modern async/await patterns with Tokio
3. **Zero-Copy**: Efficient memory usage with the bytes crate
4. **Error Handling**: Comprehensive error handling with thiserror
5. **Template-Based Commands**: Type-safe command definitions inspired by C++ templates
6. **Efficient Batch Operations**: Optimized multiple variable operations incorporating successful design patterns
7. **Design Insights**: Architecture incorporating successful patterns from existing implementations

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   moto-hses-    │    │   moto-hses-    │    │   moto-hses-    │
│      proto      │    │     client      │    │      mock       │
│                 │    │                 │    │                 │
│ • Protocol      │    │ • Async Client  │    │ • Mock Server   │
│ • Message Types │    │ • High-level    │    │ • Test Utils    │
│ • Serialization │    │   API           │    │ • Scenarios     │
│ • Commands      │    │ • Connection    │    │ • Assertions    │
│ • Types         │    │   Management    │    │                 │
│ • Variable      │    │ • Batch Ops     │    │                 │
│   Operations    │    │ • Error Handling│    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Application   │
                    │                 │
                    │ • Robot Control │
                    │ • File Operations│
                    │ • Status Monitor│
                    │ • Variable Mgmt │
                    └─────────────────┘
```

## Core Components

### moto-hses-proto

Protocol definitions and serialization layer with enhanced variable support.

#### Key Types

```rust
// Variable type definitions incorporating design insights from Python implementation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableType {
    Io = 0x78,           // 1 byte
    Register = 0x79,     // 2 bytes
    Byte = 0x7a,         // 1 byte
    Integer = 0x7b,      // 2 bytes
    Double = 0x7c,       // 4 bytes
    Real = 0x7d,         // 4 bytes
    String = 0x7e,       // 16 bytes
    RobotPosition = 0x7f, // Position data
    BasePosition = 0x80,  // Base position data
    ExternalAxis = 0x81,  // External axis data
}

// Variable object for type-safe operations
#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub var_type: VariableType,
    pub index: u8,
    pub value: T,
}

// Type-safe command definitions with improved variable support
pub trait Command {
    type Response;
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
}

// Single variable operations
pub struct ReadVar<T> {
    pub index: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVar<T> {
    pub index: u8,
    pub value: T,
}

// Multiple variable operations incorporating efficient design patterns
pub struct ReadVars<T> {
    pub start_index: u8,
    pub count: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVars<T> {
    pub start_index: u8,
    pub values: Vec<T>,
}

// Unified Position type using enum
#[derive(Debug, Clone)]
pub enum Position {
    Pulse(PulsePosition),
    Cartesian(CartesianPosition),
}

// Enhanced coordinate system definitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    Base,
    Robot,
    Tool,
    User(u8), // User1-User16
}

// Pose configuration with bit flags
#[derive(Debug, Clone, Copy)]
pub struct PoseConfiguration {
    pub no_flip: bool,
    pub lower_arm: bool,
    pub back: bool,
    pub high_r: bool,
    pub high_t: bool,
    pub high_s: bool,
}
```

#### Enhanced Error Handling

```rust
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("buffer underflow")]
    Underflow,
    #[error("invalid header")]
    InvalidHeader,
    #[error("unknown command 0x{0:04X}")]
    UnknownCommand(u16),
    #[error("unsupported operation")]
    Unsupported,
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("deserialization error: {0}")]
    Deserialization(String),
    #[error("invalid variable type")]
    InvalidVariableType,
    #[error("invalid coordinate system")]
    InvalidCoordinateSystem,
    #[error("position data error: {0}")]
    PositionError(String),
    #[error("file operation error: {0}")]
    FileError(String),
    #[error("system info error: {0}")]
    SystemInfoError(String),
}
```

### moto-hses-client

High-level client API with enhanced functionality.

#### Client Features

```rust
pub struct HsesClient {
    socket: UdpSocket,
    controller: SocketAddr,
    seq: u16,
    config: ClientConfig,
}

pub struct ClientConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub buffer_size: usize,
    pub enable_debug: bool,
    pub connection_pool_size: usize,
}

// Enhanced variable operations
impl HsesClient {
    // Single variable operations
    pub async fn read_variable<T>(&mut self, var: &mut Variable<T>) -> Result<(), ClientError>
    where T: VariableType;

    pub async fn write_variable<T>(&mut self, var: &Variable<T>) -> Result<(), ClientError>
    where T: VariableType;

    // Multiple variable operations with automatic optimization
    pub async fn read_variables<T>(&mut self, vars: &mut [Variable<T>]) -> Result<(), ClientError>
    where T: VariableType;

    pub async fn write_variables<T>(&mut self, vars: &[Variable<T>]) -> Result<(), ClientError>
    where T: VariableType;

    // Status and position operations
    pub async fn read_status(&mut self) -> Result<Status, ClientError>;
    pub async fn read_position(&mut self, robot_no: u8) -> Result<PositionInfo, ClientError>;
    pub async fn read_position_error(&mut self, robot_no: u8) -> Result<PositionError, ClientError>;
    pub async fn read_torque(&mut self, robot_no: u8) -> Result<TorqueData, ClientError>;

    // File operations
    pub async fn get_file_list(&mut self, extension: &str) -> Result<Vec<String>, ClientError>;
    pub async fn send_file(&mut self, filename: &str) -> Result<(), ClientError>;
    pub async fn recv_file(&mut self, filename: &str, local_dir: &str) -> Result<(), ClientError>;
    pub async fn delete_file(&mut self, filename: &str) -> Result<(), ClientError>;

    // System information
    pub async fn get_system_info(&mut self, system_type: u8, info_type: u8) -> Result<SystemInfo, ClientError>;
    pub async fn get_management_time(&mut self, time_type: u8) -> Result<ManagementTime, ClientError>;

    // Power and control operations
    pub async fn switch_power(&mut self, power_type: u8, switch_action: u8) -> Result<(), ClientError>;
    pub async fn select_cycle(&mut self, cycle_type: u8) -> Result<(), ClientError>;
    pub async fn move_to_position(&mut self, move_type: u8, coordinate: u8, speed_class: u8,
                                 speed: f32, position: Position, form: u8, extended_form: u8,
                                 robot_no: u8, station_no: u8, tool_no: u8, user_coor_no: u8) -> Result<(), ClientError>;
}
```

#### Efficient Variable Operations

The client automatically optimizes multiple variable operations by grouping consecutive variables of the same type, incorporating efficient design patterns:

```rust
// Automatic grouping of consecutive variables
impl HsesClient {
    fn group_consecutive_variables<T>(vars: &[Variable<T>]) -> Vec<Vec<&Variable<T>>> {
        // Implementation that groups consecutive variables for efficient batch operations
        // Uses the same logic as Python implementation's _group_nums method
    }

    async fn read_consecutive_variables<T>(&mut self, vars: &mut [Variable<T>]) -> Result<(), ClientError>
    where T: VariableType {
        // Uses plural command (0x33) for efficient batch reading
        // Automatically handles padding for 1-byte variable types
        // Similar to Python implementation's _read_consecutive_variables method
    }
}
```

### moto-hses-mock

Mock server for testing and development.

```rust
pub struct MockServer {
    socket: UdpSocket,
    handlers: HashMap<u16, Box<dyn CommandHandler>>,
}

// Mock implementations for all command types
impl MockServer {
    pub fn new() -> Self;
    pub async fn start(addr: SocketAddr) -> Result<Self, MockError>;
    pub fn register_handler<C: Command>(&mut self, handler: impl CommandHandler + 'static);
    pub async fn run(&mut self) -> Result<(), MockError>;
}
```

## Design Patterns

### 1. Type-Safe Variable Operations

Incorporating design insights from both C++ templates and Python's Variable class:

```rust
// Type-safe variable creation
let mut var_b0 = Variable::with_default(VariableType::Byte, 0);
let mut var_i1 = Variable::new(VariableType::Integer, 1, 0i16);
let mut var_r2 = Variable::new(VariableType::Real, 2, 0.0f32);

// Type-safe operations
client.read_variable(&mut var_b0).await?;
client.write_variable(&var_i1).await?;
```

### 2. Efficient Batch Operations

Optimized multiple variable operations with automatic optimization:

```rust
// Automatic grouping and optimization
let mut vars = vec![
    Variable::with_default(VariableType::Integer, 0),
    Variable::with_default(VariableType::Integer, 1),
    Variable::with_default(VariableType::Integer, 2),
    Variable::with_default(VariableType::Integer, 3),
];

// Automatically uses plural command for efficiency
client.read_variables(&mut vars).await?;
```

The optimization includes:

1. **Consecutive Variable Grouping**: Automatically groups consecutive variables of the same type
2. **Plural Commands**: Uses HSE plural commands (0x33) for reading multiple variables in a single network call
3. **Automatic Padding**: Handles padding requirements for 1-byte variable types

### 3. Error Handling

Comprehensive error handling with detailed error types:

```rust
match client.read_variable(&mut var).await {
    Ok(_) => println!("Success"),
    Err(ClientError::Proto(ProtocolError::InvalidVariableType)) => {
        println!("Invalid variable type");
    }
    Err(ClientError::Timeout) => {
        println!("Operation timed out");
    }
    Err(ClientError::Io(e)) => {
        println!("IO error: {}", e);
    }
}
```

## Performance Considerations

### 1. Zero-Copy Operations

- Use of `Bytes` and `BytesMut` for efficient memory management
- Minimal allocations during serialization/deserialization
- Efficient buffer reuse

### 2. Efficient Variable Operations

- Automatic grouping of consecutive variables
- Use of plural commands (0x33) for efficiency
- Reduced network round trips
- Automatic padding handling for 1-byte variable types

### 3. Connection Management

- Connection pooling for high-performance applications
- Automatic retry logic with exponential backoff
- Configurable timeouts and buffer sizes

### 4. Async/Await

- Non-blocking I/O operations
- Efficient resource utilization
- Support for concurrent operations

## Design Insights

The architecture incorporates successful patterns from existing implementations:

### Variable Object Pattern

The `Variable<T>` object pattern provides type safety while maintaining flexibility, incorporating design insights from successful implementations.

### Efficient Batch Operations

The library implements efficient batch operations that automatically optimize network usage, reducing round trips and improving performance for applications that need to read multiple variables.

## Future Enhancements

1. **WebSocket Support**: Real-time status streaming
2. **GraphQL Interface**: Modern API querying
3. **Plugin System**: Extensible command support
4. **Performance Monitoring**: Built-in metrics and profiling
5. **Configuration Management**: YAML/JSON configuration files
6. **Logging Integration**: Structured logging with tracing
7. **Security Features**: TLS encryption and authentication

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_variable() {
        let server = MockHsesServer::new()
            .with_variable(0, 42i32)
            .start()
            .await?;

        let client = HsesClient::connect("127.0.0.1:10040").await?;
        let value: i32 = client.read_variable(0).await?;
        assert_eq!(value, 42);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_workflow() {
    let server = MockHsesServer::new()
        .with_variable(0, 100i32)
        .with_position(1, Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1)))
        .start()
        .await?;

    let client = HsesClient::connect("127.0.0.1:10040").await?;

    // Read variable
    let value: i32 = client.read_variable(0).await?;
    assert_eq!(value, 100);

    // Read position
    let position = client.read_position(1, CoordinateSystemType::RobotPulse).await?;
    assert!(position.is_pulse());

    // Write variable
    client.write_variable(0, 200i32).await?;
}
```
