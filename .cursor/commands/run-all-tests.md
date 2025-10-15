# Comprehensive Test Suite Execution

## Overview

This document outlines the comprehensive testing procedures for the Moto-HSES project.

## Procedures

```bash
cargo test --all-features --workspace
```

- **Required**: All tests must pass
- **Fix if needed**: Follow the troubleshooting guide below

## Test Failure Troubleshooting

### Unit Test Specific Commands

```bash
# Run tests for specific package if workspace test fails
cd moto-hses-proto && cargo test
cd moto-hses-client && cargo test
cd moto-hses-mock && cargo test
```

### Integration Test Specific Commands

```bash
# Run protocol communication tests
cargo test --test protocol_communication_tests

# Run comprehensive integration tests
cargo test --test integration_tests

# Check integration test logs if needed
tail -f moto-hses-client/logs/integration_tests.log
```

## Failure Resolution Workflow

1. **Re-run the check** to verify the fix
2. **If still failing**, investigate the specific error messages
3. **Apply targeted fixes** based on error analysis
4. **Repeat until all checks pass**
