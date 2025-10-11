# Comprehensive Test Suite Execution and Quality Assurance

## Overview

This document outlines the comprehensive testing and quality assurance procedures for the Moto-HSES project. It covers code formatting, static analysis, and test execution to ensure code quality, maintainability, and functionality across the entire workspace.

## Procedures

### 1. Code Formatting

```bash
cargo fmt --all -- --check
```

- **Required**: All code must be properly formatted
- **Fix if needed**: `cargo fmt --all`

### 2. Static Analysis

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

- **Required**: No Clippy warnings or errors
- **Fix if needed**: `cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged`

### 3. Tests

```bash
cargo test --all-features --workspace
```

- **Required**: All tests must pass
- **Fix if needed**: Follow the troubleshooting guide below

#### Test Failure Troubleshooting

##### Unit Test Specific Commands

```bash
# Run tests for specific package if workspace test fails
cd moto-hses-proto && cargo test
cd moto-hses-client && cargo test
cd moto-hses-mock && cargo test
```

##### Integration Test Specific Commands

```bash
# Run protocol communication tests
cargo test --test protocol_communication_tests

# Run comprehensive integration tests
cargo test --test integration_tests

# Check integration test logs if needed
tail -f moto-hses-client/logs/integration_tests.log
```

## Failure Resolution Workflow

1. **Run the check command** (e.g., `cargo fmt --all -- --check`)
2. **If it fails**, run the corresponding fix command (e.g., `cargo fmt --all`)
3. **Re-run the check** to verify the fix
4. **If still failing**, investigate the specific error messages
5. **Apply targeted fixes** based on error analysis
6. **Repeat until all checks pass**
