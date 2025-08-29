# Architecture Design

## Overview

This document describes the architecture of the Rust HSES client library, inspired by the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet).

## Core Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time safety
2. **Async-First**: Modern async/await patterns with Tokio
3. **Zero-Copy**: Efficient memory usage with the bytes crate
4. **Error Handling**: Comprehensive error handling with thiserror
5. **Template-Based Commands**: Type-safe command definitions inspired by C++ templates

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
                    └─────────────────┘
```

## Core Components

### moto-hses-proto

Protocol definitions and serialization layer.

#### Key Types

```rust
// Type-safe command definitions inspired by C++ templates
pub trait Command {
    type Response;
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
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

#### Command System

```rust
// Type-safe command definitions
pub struct ReadVar<T> {
    pub index: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVar<T> {
    pub index: u8,
    pub value: T,
}

pub struct ReadStatus;
pub struct ReadCurrentPosition {
    pub control_group: u8,
    pub coordinate_system: CoordinateSystemType,
}

// Command implementations
impl Command for ReadVar<i32> {
    type Response = i32;
    fn command_id() -> u16 { 0x7C }
    // ...
}
```

### moto-hses-client

Asynchronous client implementation with high-level API.

#### Client Architecture

```rust
pub struct HsesClient {
    inner: Arc<InnerClient>,
    config: ClientConfig,
}

struct InnerClient {
    socket: UdpSocket,
    remote_addr: SocketAddr,
    request_id: AtomicU8,
    pending_requests: Arc<Mutex<HashMap<u8, PendingRequest>>>,
}

// High-level async API
impl HsesClient {
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        let command = ReadVar::<T>::new(index);
        self.send_command(command).await
    }

    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: VariableType,
    {
        let command = WriteVar::<T>::new(index, value);
        self.send_command(command).await
    }

    pub async fn read_status(&self) -> Result<Status, ClientError> {
        self.send_command(ReadStatus).await
    }

    pub async fn read_position(&self, control_group: u8, coord_system: CoordinateSystemType) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition { control_group, coordinate_system: coord_system };
        self.send_command(command).await
    }
}
```

#### Connection Management

```rust
impl HsesClient {
    pub async fn connect(addr: &str, config: ClientConfig) -> Result<Self, ClientError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let remote_addr: SocketAddr = addr.parse()?;
        socket.connect(remote_addr).await?;

        Ok(Self {
            inner: Arc::new(InnerClient {
                socket,
                remote_addr,
                request_id: AtomicU8::new(1),
                pending_requests: Arc::new(Mutex::new(HashMap::new())),
            }),
            config,
        })
    }
}
```

### moto-hses-mock

Mock server for testing and development.

#### Mock Server Features

```rust
pub struct MockHsesServer {
    variables: Arc<RwLock<HashMap<u8, VariableValue>>>,
    positions: Arc<RwLock<HashMap<u8, Position>>>,
    status: Arc<RwLock<Status>>,
    handlers: Arc<RwLock<HashMap<u16, Box<dyn RequestHandler>>>>,
}

impl MockHsesServer {
    pub fn with_variable<T: VariableType>(mut self, index: u8, value: T) -> Self {
        self.variables.write().unwrap().insert(index, value.into());
        self
    }

    pub fn with_position(mut self, index: u8, position: Position) -> Self {
        self.positions.write().unwrap().insert(index, position);
        self
    }

    pub fn with_custom_handler<F>(mut self, command_id: u16, handler: F) -> Self
    where
        F: RequestHandler + 'static,
    {
        self.handlers.write().unwrap().insert(command_id, Box::new(handler));
        self
    }
}
```

## Type System Design

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
    // ...
}

impl VariableType for i16 {
    fn command_id() -> u16 { 0x7B }
    // ...
}

impl VariableType for i32 {
    fn command_id() -> u16 { 0x7C }
    // ...
}

impl VariableType for f32 {
    fn command_id() -> u16 { 0x7D }
    // ...
}

impl VariableType for Position {
    fn command_id() -> u16 { 0x7F }
    // ...
}
```

### Position Types

```rust
#[derive(Debug, Clone)]
pub struct PulsePosition {
    pub joints: [i32; 8],
    pub tool: i32,
    pub size: u8, // Number of active joints
}

#[derive(Debug, Clone)]
pub struct CartesianPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rx: f64,
    pub ry: f64,
    pub rz: f64,
    pub frame: CoordinateSystem,
    pub configuration: PoseConfiguration,
    pub tool: i32,
}

// Unified Position type
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

## Error Handling Strategy

### Error Types Hierarchy

```
Error
├── ClientError
│   ├── ConnectionError
│   ├── TimeoutError
│   ├── ProtocolError
│   └── InvalidVariableError
├── ProtocolError
│   ├── SerializationError
│   ├── DeserializationError
│   ├── InvalidMessageError
│   └── InvalidCommandError
└── MockError
    ├── HandlerNotFound
    ├── InvalidResponse
    └── ServerError
```

### Error Handling Patterns

```rust
// Result type aliases
pub type Result<T> = std::result::Result<T, Error>;
pub type ClientResult<T> = std::result::Result<T, ClientError>;
pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

// Error conversion
impl From<ProtocolError> for ClientError {
    fn from(err: ProtocolError) -> Self {
        ClientError::ProtocolError(err)
    }
}
```

## Performance Considerations

### Memory Management

1. **Zero-Copy Operations**: Use bytes crate for efficient buffer management
2. **Arc for Sharing**: Share client state across async tasks
3. **Pooled Buffers**: Reuse message buffers to reduce allocations
4. **Async I/O**: Non-blocking UDP operations with Tokio

### Concurrency

1. **Request ID Management**: Atomic operations for request ID allocation
2. **Pending Requests**: Thread-safe HashMap for tracking requests
3. **Connection Pooling**: Reuse connections when possible
4. **Batch Operations**: Support for multiple commands in single request

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

## Migration from Current Design

### Key Changes

1. **Template-Based Commands**: Replace enum-based commands with trait-based system
2. **Unified Position Type**: Use enum instead of separate types
3. **Enhanced Type Safety**: Leverage Rust's type system more effectively
4. **Async-First API**: Improve async patterns and error handling
5. **Better Mock Support**: Enhanced testing capabilities

### Backward Compatibility

- Maintain existing API where possible
- Provide migration guides for breaking changes
- Support both old and new patterns during transition
