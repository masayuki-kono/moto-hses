# Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for the Rust HSES client library.

## Testing Pyramid

```
    E2E Tests (Few)
       /    \
      /      \
Integration Tests (Some)
     /    \
    /      \
  Unit Tests (Many)
```

## Unit Tests

### Protocol Layer Tests

```rust
#[cfg(test)]
mod serialization_tests {
    #[test]
    fn test_serialize_read_variable_command() {
        let command = Command::ReadVariable {
            var_type: VariableType::Int32,
            var_number: 1,
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

## Integration Tests

### Client-Mock Server Communication

```rust
#[tokio::test]
async fn test_read_variable() {
    let server = MockHsesServer::new("127.0.0.1:10041")
        .await
        .unwrap()
        .with_variable(1, 42i32)
        .await;

    server.start().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = HsesClient::new("127.0.0.1:10041").await.unwrap();
    let value: i32 = client.read_variable(1, VariableType::Int32).await.unwrap();

    assert_eq!(value, 42);
}
```

## Performance Tests

```rust
#[tokio::test]
async fn test_read_throughput() {
    let server = MockHsesServer::new("127.0.0.1:10048")
        .await
        .unwrap()
        .with_variable(1, 42i32)
        .await;

    server.start().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = HsesClient::new("127.0.0.1:10048").await.unwrap();

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let _: i32 = client.read_variable(1, VariableType::Int32).await.unwrap();
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    assert!(throughput > 100.0); // Minimum 100 ops/sec
}
```

## Test Coverage Goals

- **Unit Tests**: >90% line coverage
- **Integration Tests**: >80% integration coverage
- **Error Paths**: 100% error handling coverage
- **Performance**: All performance benchmarks pass

## Best Practices

1. **Arrange-Act-Assert**: Structure tests with clear sections
2. **Descriptive Names**: Use descriptive test function names
3. **Single Responsibility**: Each test should test one thing
4. **Independent Tests**: Tests should not depend on each other
5. **Fast Execution**: Tests should run quickly
