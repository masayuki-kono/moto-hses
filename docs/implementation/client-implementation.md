# HSES Client Implementation

## Overview

The `moto-hses-client` crate provides a type-safe, asynchronous Rust client for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol.

## Features

### Core Functionality

- **Type-safe API**: Leverage Rust's type system for compile-time safety
- **Async-first**: Built on Tokio for efficient asynchronous I/O
- **Comprehensive error handling**: Type-safe error handling with thiserror
- **Memory efficient**: Zero-copy operations using the bytes crate
- **Extensible**: Modular design for easy extension and testing

### Connection Management

- **Automatic connection**: Client automatically connects on creation
- **Connection validation**: Tests connection by reading robot status
- **Retry logic**: Configurable retry attempts with exponential backoff
- **Reconnection support**: Manual reconnection capability
- **Connection status tracking**: Real-time connection state monitoring

### Error Handling

- **Timeout handling**: Configurable timeouts for all operations
- **Retry mechanism**: Automatic retry with configurable attempts and delays
- **Error categorization**: Specific error types for different failure modes
- **Graceful degradation**: Proper error propagation and handling

## API Reference

### Client Creation

```rust
// Create client with default configuration
let client = HsesClient::new("192.168.1.100:10040").await?;

// Create client with custom configuration
let config = ClientConfig {
    timeout: Duration::from_millis(500),
    retry_count: 5,
    retry_delay: Duration::from_millis(200),
    buffer_size: 8192,
    connection_timeout: Duration::from_secs(10),
};
let client = HsesClient::new_with_config("192.168.1.100:10040", config).await?;
```

### Configuration Options

```rust
pub struct ClientConfig {
    pub timeout: Duration,              // Response timeout
    pub retry_count: u32,               // Number of retry attempts
    pub retry_delay: Duration,          // Delay between retries
    pub buffer_size: usize,             // UDP buffer size
    pub connection_timeout: Duration,   // Connection establishment timeout
}
```

### Core Operations

#### Status Reading

```rust
// Read robot status
let status = client.read_status().await?;
println!("Running: {}", status.is_running());
println!("Servo on: {}", status.is_servo_on());
println!("Alarm: {}", status.has_alarm());
```

#### Variable Operations

```rust
// Read variables
let int_value: i32 = client.read_variable(0).await?;
let float_value: f32 = client.read_variable(0).await?;
let byte_value: u8 = client.read_variable(0).await?;

// Write variables
client.write_variable(0, 42i32).await?;
client.write_variable(0, 3.14159f32).await?;
client.write_variable(0, 255u8).await?;
```

#### Convenience Methods

```rust
// Type-specific convenience methods
let int_val = client.read_int(0).await?;
let float_val = client.read_float(0).await?;
let byte_val = client.read_byte(0).await?;

client.write_int(0, 42).await?;
client.write_float(0, 3.14159).await?;
client.write_byte(0, 255).await?;
```

#### Position Reading

```rust
// Read current position
let position = client.read_position(1, CoordinateSystemType::RobotPulse).await?;
println!("Position: {:?}", position);
```

#### Connection Management

```rust
// Check connection status
if client.is_connected() {
    println!("Connected to robot");
}

// Reconnect if needed
client.reconnect().await?;
```

### Error Types

```rust
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("Invalid variable: {0}")]
    InvalidVariable(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Connection failed after {0} retries")]
    ConnectionFailed(u32),
}
```

## Examples

### Basic Usage

```bash
cargo run -p moto-hses-client --example basic_usage -- 192.168.1.100:10040
```

### Connection Management

```bash
cargo run -p moto-hses-client --example connection_management -- 192.168.1.100:10040
```

### Status Reading

```bash
cargo run -p moto-hses-client --example read_status -- 192.168.1.100:10040
```

## Implementation Details

### Architecture

The client uses a layered architecture:

1. **Transport Layer**: UDP socket management with Tokio
2. **Protocol Layer**: HSES message serialization/deserialization
3. **Command Layer**: Type-safe command definitions
4. **API Layer**: High-level client interface

### Thread Safety

- Uses `Arc<Mutex<>>` for shared state
- Atomic operations for request ID generation
- Thread-safe connection status tracking

### Performance Considerations

- Zero-copy operations where possible
- Efficient buffer management
- Configurable timeouts and retries
- Async/await for non-blocking I/O

### Error Recovery

- Automatic retry with exponential backoff
- Connection validation on startup
- Graceful error propagation
- Comprehensive error categorization

## Testing

### Unit Tests

```bash
cargo test -p moto-hses-client
```

### Integration Tests

```bash
# Start mock server
cargo run -p moto-hses-mock

# Run client examples against mock
cargo run -p moto-hses-client --example basic_usage -- 127.0.0.1:12222
```

## Future Enhancements

### Planned Features

- [ ] I/O operations (read_io, write_io)
- [ ] Job execution (execute_job, stop_job)
- [ ] String variable support
- [ ] Batch operations
- [ ] Connection pooling
- [ ] Metrics and monitoring

### Extensibility

The client is designed to be easily extensible:

- Generic command system for new operations
- Pluggable transport layer
- Configurable error handling
- Modular architecture

## Contributing

When adding new features:

1. Follow the existing code style
2. Add comprehensive tests
3. Update documentation
4. Consider backward compatibility
5. Add examples for new functionality
