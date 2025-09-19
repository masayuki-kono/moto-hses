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

These tests verify individual protocol components without network communication:

- **Message serialization/deserialization**: `HsesRequestMessage` and `HsesResponseMessage` encoding/decoding
- **Enum validation**: Division, Service, CoordinateSystem enum values

### Client Layer Tests

These tests verify client-side components in isolation:

### Mock Server Tests

These tests verify mock server components without actual UDP communication:

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
