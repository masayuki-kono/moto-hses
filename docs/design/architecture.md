# Architecture Design

## Overview

This document describes the architecture for the Rust implementation of the Yaskawa HSES (High Speed Ethernet Server) client library.

## Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time safety
2. **Async-First**: Built on Tokio for efficient asynchronous I/O
3. **Error Handling**: Comprehensive error handling with thiserror
4. **Memory Efficiency**: Zero-copy operations where possible using bytes crate
5. **Extensibility**: Modular design for easy extension and testing

## Module Structure

```
moto-hses/
├── moto-hses-proto/     # Protocol definitions and serialization
├── moto-hses-client/    # Async client implementation
└── moto-hses-mock/      # Mock server for testing
```

## Detailed Module Design

### moto-hses-proto

Core protocol definitions and serialization logic.

#### Key Components

1. **Message Types**

   - `Message`: Base message structure
   - `Command`: Enum of all supported commands
   - `Response`: Response message structure
   - `VariableType`: Enum of variable types

2. **Serialization**

   - `Serializer`: Little-endian serialization
   - `Deserializer`: Little-endian deserialization
   - `MessageBuilder`: Fluent API for message construction

3. **Error Types**
   - `ProtocolError`: Protocol-specific errors
   - `SerializationError`: Serialization/deserialization errors

#### Example API

```rust
use moto_hses_proto::{Message, Command, VariableType};

let message = Message::new()
    .with_command(Command::ReadVariable {
        var_type: VariableType::Int32,
        var_number: 1,
    })
    .build();
```

### moto-hses-client

Asynchronous client implementation for HSES communication.

#### Key Components

1. **Client**

   - `HsesClient`: Main client struct
   - `ClientConfig`: Configuration options
   - `Connection`: Connection management

2. **Commands**

   - `ReadStatus`: Read robot status information
   - `ReadPosition`: Read robot position data
   - `ReadVariable`: Read robot variables (D, R, S, P types)
   - `WriteVariable`: Write robot variables
   - `ExecuteJob`: Execute robot jobs
   - `StartJob`: Start job execution
   - `ReadIO`: Read I/O data
   - `WriteIO`: Write I/O data
   - `FileOperations`: File loading, saving, deletion, listing

3. **Error Handling**
   - `ClientError`: Client-specific errors
   - `TimeoutError`: Communication timeouts
   - `ProtocolError`: Protocol violations

#### Example API

```rust
use moto_hses_client::{HsesClient, ReadVariable};
use std::time::Duration;

let client = HsesClient::new("192.168.1.100:10040").await?;

let value: i32 = client
    .read_variable(1, VariableType::Int32)
    .timeout(Duration::from_millis(300))
    .await?;
```

### moto-hses-mock

Mock server for testing and development.

#### Key Components

1. **Mock Server**

   - `MockHsesServer`: Mock HSES server implementation
   - `ServerConfig`: Mock server configuration
   - `RequestHandler`: Custom request handlers

2. **Testing Utilities**
   - `TestClient`: Test client with mock server
   - `ScenarioBuilder`: Build test scenarios
   - `Assertions`: Test assertions

#### Example Usage

```rust
use moto_hses_mock::{MockHsesServer, TestClient};

let server = MockHsesServer::new()
    .with_variable(1, 42i32)
    .start().await?;

let client = TestClient::connect_to_mock(server).await?;
let value: i32 = client.read_variable(1).await?;
assert_eq!(value, 42);
```

## Error Handling Strategy

### Error Types Hierarchy

```
Error
├── ClientError
│   ├── ConnectionError
│   ├── TimeoutError
│   └── ProtocolError
├── ProtocolError
│   ├── SerializationError
│   ├── DeserializationError
│   └── InvalidMessageError
└── SystemError
    ├── IoError
    └── ConfigurationError
```

### Error Propagation

- Use `thiserror` for type-safe error handling
- Implement `From` traits for automatic error conversion
- Provide context information for debugging
- Support error chaining with `source` method

## Configuration Management

### Client Configuration

```rust
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub remote_addr: SocketAddr,
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub buffer_size: usize,
}
```

### Environment Variables

- `HSES_HOST`: Default robot host
- `HSES_PORT`: Default robot port
- `HSES_TIMEOUT`: Default timeout in milliseconds
- `HSES_RETRY_COUNT`: Default retry count

## Testing Strategy

### Unit Tests

- Protocol serialization/deserialization
- Error handling
- Configuration parsing
- Utility functions

### Integration Tests

- End-to-end communication with mock server
- Error scenarios
- Timeout handling
- Retry logic

### Performance Tests

- Throughput measurement
- Latency measurement
- Memory usage
- Concurrent connection handling

## Security Considerations

1. **Input Validation**: Validate all input parameters
2. **Buffer Management**: Prevent buffer overflows
3. **Resource Limits**: Limit concurrent connections
4. **Error Information**: Avoid leaking sensitive information in errors

## Performance Considerations

1. **Zero-Copy**: Use `Bytes` for efficient memory management
2. **Connection Pooling**: Reuse UDP sockets when possible
3. **Async I/O**: Non-blocking operations with Tokio
4. **Memory Allocation**: Minimize allocations in hot paths

## Future Extensions

1. **TLS Support**: Secure communication over UDP
2. **Compression**: Message compression for large data
3. **Caching**: Client-side caching for frequently accessed variables
4. **Metrics**: Built-in metrics and monitoring
5. **Plugin System**: Extensible command system
