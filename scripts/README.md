# Integration Test Script

This directory contains scripts for running integration tests for the moto-hses project.

## File Structure

- `integration_test.sh` - Main integration test script
- `integration_test.toml` - Test configuration file (defines expected outputs)
- `README.md` - This file

## Usage

### Basic Execution

```bash
./integration_test.sh
```

### Debug Mode Execution

```bash
DEBUG_MODE=true ./integration_test.sh
```

In debug mode, the following additional information is displayed:

- Detailed validation process for each test
- Comparison of expected vs actual values on failure
- Complete output content

## Test Configuration

Expected test outputs are defined in the `integration_test.toml` file. You can modify test expectations by editing this file.

### Configuration File Structure

```toml
[tests.test_name]
command = "example_name"
port = "10040"
description = "Test description"
optional = false

[[tests.test_name.expected_outputs]]
type = "connection"
pattern = "Successfully connected"
description = "Connection test"
```

## Improved Features

### 1. Configuration-Based Testing

- Moved hardcoded expectations to configuration file
- Easy to add or modify tests
- Improved maintainability

### 2. Structured Output Validation

- Generic validation functions
- Detailed error information
- Detailed display in debug mode

### 3. Automatic Cleanup

- Automatic cleanup of temporary files
- Proper process termination handling

### 4. Comprehensive Reporting

- Detailed test result statistics
- Color-coded output
- Log file recording

## Troubleshooting

### When Tests Fail

1. Run in debug mode to check details:

   ```bash
   DEBUG_MODE=true ./integration_test.sh
   ```

2. Check log files:

   ```bash
   cat logs/integration_test_summary.log
   ```

3. Check mock server logs:

   ```bash
   cat logs/mock_server.log
   ```

4. Check detailed test outputs:
   ```bash
   ls logs/integration_test_detailed_outputs/
   cat logs/integration_test_detailed_outputs/basic_operations.log
   ```

### Modifying Configuration Files

You can adjust expectations by editing `integration_test.toml`. Re-run tests after making changes.

### Adding New Tests

1. Add a new test section to `integration_test.toml`
2. Add a new test function to `integration_test.sh` if needed
3. Call the new test in the `main()` function

## Comparison with Previous Approach

### Previous Issues

- Hardcoded string matching
- Tests breaking when output format changes
- Difficulty in debugging
- Low maintainability

### Post-Improvement Benefits

- Flexible management through configuration files
- Structured validation process
- Detailed debug information
- High maintainability and extensibility
