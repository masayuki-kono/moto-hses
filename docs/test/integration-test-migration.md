# Integration Test Migration Plan

## Overview

The current integration tests use a pattern-matching approach that executes client sample programs and matches output messages, which has low maintainability issues. This document describes the migration plan to new integration tests using Rust's standard test framework.

## Current Issues

1. **Output Message Dependency**: Pattern matching of output messages from each sample program in TOML files
2. **Maintenance Burden**: Need to modify `integration_test.toml` whenever message content changes
3. **Fragility**: Tests break easily when output format changes
4. **Debugging Difficulty**: Testing based on output strings rather than actual API behavior

## Features of New Test Framework

### 1. Rust Standard Test Framework

- Asynchronous tests using `#[tokio::test]`
- Standard assertions like `assert_eq!`, `assert!`
- Focus on return value verification

### 2. Structured Tests

```
moto-hses-client/
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── variable_operations.rs
│   │   ├── position_operations.rs
│   │   ├── alarm_operations.rs
│   │   ├── io_operations.rs
│   │   ├── register_operations.rs
│   │   ├── file_operations.rs
│   │   ├── connection_management.rs
│   │   ├── read_status.rs
│   │   ├── read_executing_job_info.rs
│   │   └── hold_servo_control.rs
│   └── common/
│       ├── mod.rs
│       ├── test_utils.rs
│       └── mock_server_setup.rs
```

### 3. Test Characteristics

- **Actual API Communication**: Testing actual communication with mock servers
- **Return Value Verification**: Direct verification of function return values
- **Error Handling**: Comprehensive testing including error cases
- **Parallel Execution**: Each test can run independently
- **Customized Initial State**: Different mock server initial states for each test

## Migration Steps

### Phase 1: Implementation of New Test Framework ✅

1. **Create Test Directory Structure**

   - Create `tests/integration/` directory
   - Create `tests/common/` directory
   - Implement each test module

2. **Implement Common Utilities**

   - `MockServerManager`: Mock server startup/shutdown management
   - `test_utils`: Helper functions for testing
   - Common client creation
   - Customized mock server configuration for each test

3. **Implement Tests for Each Feature**
   - Variable operation tests
   - Position operation tests
   - Alarm operation tests
   - I/O operation tests
   - Register operation tests
   - File operation tests
   - Connection management tests
   - Status reading tests
   - Executing job info tests
   - Hold/servo control tests

### Phase 2: CI/CD Updates ✅

1. **GitHub Actions Updates**

   - Add execution of new test framework
   - Parallel execution with legacy tests
   - Test execution in release mode

2. **Cargo.toml Updates**
   - Add required dependencies
   - Test configuration

### Phase 3: Gradual Migration

1. **Parallel Execution Period**

   - Run new test framework and legacy tests in parallel
   - Ensure both test suites pass

2. **Gradual Legacy Test Deprecation**

   - Disable legacy tests once new tests are stable
   - Remove `integration_test.toml`
   - Remove `integration_test.sh`

3. **Examples Simplification**
   - Change current examples to concise usage examples
   - Remove complex test logic from examples

## How to Run New Tests

### Local Execution

```bash
# Run all integration tests
cargo test --package moto-hses-client --test integration

# Run specific test
cargo test --package moto-hses-client --test integration variable_operations

# Run in debug mode
RUST_LOG=debug cargo test --package moto-hses-client --test integration
```

### CI/CD Execution

- Automatic execution in GitHub Actions
- Execution on pull requests
- Execution on pushes to main branch

## Benefits

### 1. Improved Maintainability

- Not affected by message content changes
- Testing actual API behavior
- Type-safe tests

### 2. Easier Debugging

- Detailed information on failure
- Standard Rust test output
- IDE support for test execution

### 3. Extensibility

- Easy addition of tests for new features
- Parallel test execution
- Addition of custom assertions

### 4. Performance

- Parallel test execution
- Efficient mock server management
- Proper resource cleanup

### 5. Test Flexibility

- Customized initial state for each test
- Direct control of mock server configuration
- More realistic test scenarios

## Considerations

### 1. Mock Server Dependencies

- Each test uses independent ports
- Proper cleanup implementation
- Port conflict avoidance

### 2. Test Stability

- Appropriate network timeout settings
- Retry mechanism implementation
- Ensuring independence between tests

### 3. Error Handling

- Proper handling of network errors
- Response to mock server startup failures
- Detailed information on test failures

## Future Extensions

### 1. Performance Testing

- Add throughput tests
- Add latency tests
- Implement load testing

### 2. Integration Test Extensions

- Test more complex scenarios
- Comprehensive error case testing
- Add boundary value tests

### 3. Test Reporting

- Measure test coverage
- Collect performance metrics
- Visualize test results

## Conclusion

The migration to the new test framework is expected to bring the following improvements:

1. **Significant Improvement in Maintainability**: Not affected by message content changes
2. **Easier Debugging**: Leveraging standard Rust test tools
3. **Extensibility**: Easy addition of tests for new features
4. **Reliability**: Testing actual API behavior

This migration is expected to significantly improve the test quality and development efficiency of the moto-hses project.
