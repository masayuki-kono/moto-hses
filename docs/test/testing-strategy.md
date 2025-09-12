# Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for the Rust HSES client library.

## Testing Pyramid

```
    End-to-End Tests (Few)
       /    \
      /      \
  Unit Tests (Many)
```

- **Unit Tests**: Individual components, protocol implementation, and mock server tests
- **End-to-End Tests**: Complete client-server integration tests

## Unit Tests

### Protocol Layer Tests

### Mock Server Protocol Tests

```rust
#[cfg(test)]
mod serialization_tests {
    #[test]
    fn test_serialize_read_variable_command() {
        let command = Command::ReadVariable {
            var_type: VariableType::Integer,
            var_number: 0,
        };

        let data = Serializer::serialize_command(&command).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(data[0..2], [0x00, 0x01]); // Variable type
        assert_eq!(data[2..4], [0x00, 0x01]); // Variable number
    }
}
```

### Client Layer Tests

```rust
#[cfg(test)]
mod client_tests {
    #[tokio::test]
    async fn test_client_creation() {
        let client = HsesClient::new("127.0.0.1:10040").await;
        assert!(client.is_ok());
    }
}
```

These tests verify the mock server's protocol implementation by sending UDP messages directly:

```rust
#[tokio::test]
async fn test_status_command() {
    let (addr, _handle) = test_utils::start_test_server().await.unwrap();

    // Create a UDP socket to send commands
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    // Create status read command (0x72)
    let message = proto::HsesRequestMessage::new(
        1,      // Division: Robot
        0,      // ACK: Request
        1,      // Request ID
        0x72,   // Command: Read Status
        1,      // Instance
        1,      // Attribute: Data 1
        0x0e,   // Service: Get_Attribute_Single
        vec![], // No payload
    );

    let data = message.encode();
    socket.send_to(&data, addr).await.unwrap();

    // Wait for response and validate
    let mut buf = vec![0u8; 1024];
    let (n, _) = socket.recv_from(&mut buf).await.unwrap();
    let response = proto::HsesResponseMessage::decode(&buf[..n]).unwrap();

    assert_eq!(response.header.ack, 1); // Should be ACK
    assert_eq!(response.sub_header.service, 0x8e); // 0x0e + 0x80
}
```

## End-to-End Integration Tests

### Client-Mock Server Communication

These tests verify the complete client-server integration using the actual client library:

```bash
# Run comprehensive integration tests
cargo test --test integration_tests
```

The script tests:

- Client connection establishment
- Status reading operations
- Position reading operations
- Alarm data operations
- Variable read/write operations
- Convenience methods
- Communication integrity validation

## Performance Tests

```rust
#[tokio::test]
async fn test_read_throughput() {
    let server = MockHsesServer::new("127.0.0.1:10048")
        .await
        .unwrap()
        .with_variable(0, 42i32)
        .await;

    server.start().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = HsesClient::new("127.0.0.1:10048").await.unwrap();

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let _: i32 = client.read_variable(0, VariableType::Integer).await.unwrap();
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    assert!(throughput > 100.0); // Minimum 100 ops/sec
}
```

## Test Coverage Goals

- **Unit Tests**: >90% line coverage (including protocol and mock server tests)
- **End-to-End Tests**: >80% integration coverage
- **Error Paths**: 100% error handling coverage
- **Performance**: All performance benchmarks pass

## Test Execution

### Running Tests Locally

```bash
# Run all unit tests (including protocol and mock server tests)
cargo test

# Run mock server protocol tests specifically
cargo test --test protocol_communication_tests

# Run comprehensive integration tests
cargo test --test integration_tests
```

### CI/CD Integration

The following tests run automatically on pull requests:

1. **Unit Tests**: Code formatting, Clippy, unit tests (including protocol and mock server tests)
2. **Security Audit**: Security vulnerability checks
3. **End-to-End Integration Tests**: Client-server integration tests

## Best Practices

1. **Arrange-Act-Assert**: Structure tests with clear sections
2. **Descriptive Names**: Use descriptive test function names
3. **Single Responsibility**: Each test should test one thing
4. **Independent Tests**: Tests should not depend on each other
5. **Fast Execution**: Tests should run quickly
