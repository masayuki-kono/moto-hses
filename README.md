# moto-hses

Rust implementation of Yaskawa High-Speed Ethernet Server (HSES) client library.

## ⚠️ Important Notice

This repository is an experimental project using LLM-assisted development. Documentation and implementation reviews are incomplete. Please use with caution.

## Overview

This library provides a type-safe, asynchronous Rust client for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol.

## Features

- **Type-safe API**: Leverage Rust's type system for compile-time safety
- **Async-first**: Built on Tokio for efficient asynchronous I/O
- **Comprehensive error handling**: Type-safe error handling with thiserror
- **Memory efficient**: Zero-copy operations using the bytes crate
- **Extensible**: Modular design for easy extension and testing

## Crates

- `moto-hses-proto` — Protocol definitions and serialization
- `moto-hses-client` — Async UDP client using Tokio
- `moto-hses-mock` — Local mock HSES UDP server for testing

## Documentation

### Specifications

- [`docs/specs/hses-protocol.md`](docs/specs/hses-protocol.md) — HSES protocol specification and implementation guidelines

### Design Documents

- [`docs/design/architecture.md`](docs/design/architecture.md) — System architecture and design principles
- [`docs/design/client-api.md`](docs/design/api-design.md) — API design and usage examples
- [`docs/design/implementation-guide.md`](docs/design/implementation-guide.md) — Step-by-step implementation guide

### Testing

- [`docs/test/testing-strategy.md`](docs/test/testing-strategy.md) — Testing strategy and best practices

### Architecture Decision Records

- `docs/adr/0001-adopt-hses.md` — Decision to adopt HSES protocol

## Quick Start

### Basic Usage

```rust
use moto_hses_client::{HsesClient, VariableType};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = HsesClient::new("192.168.1.100:10040").await?;

    // Read robot status
    let status = client.read_status().await?;
    println!("Robot running: {}", status.is_running());

    // Read variable (using generic type parameter)
    let value: i32 = client.read_variable::<i32>(0).await?;
    println!("D000: {}", value);

    // Write variable
    client.write_variable(0, 42i32).await?;

    // Read current position
    let position = client.read_position(1, CoordinateSystemType::RobotPulse).await?;
    println!("Current position: {:?}", position);

    // Read alarm data
    let alarm = client.read_alarm_data(1, 0).await?;
    println!("Alarm: Code={}, Name={}", alarm.code, alarm.name);

    // Convenience methods for status checking
    let is_running = client.is_running().await?;
    let is_servo_on = client.is_servo_on().await?;
    let has_alarm = client.has_alarm().await?;

    println!("Running: {}, Servo: {}, Alarm: {}", is_running, is_servo_on, has_alarm);

    Ok(())
}
```

### Alarm Operations

```rust
use moto_hses_client::{HsesClient, Alarm};

// Read alarm data
let alarm = client.read_alarm_data(1, 0).await?;
println!("Alarm: Code={}, Name={}", alarm.code, alarm.name);

// Read specific alarm attributes
let alarm_code = client.read_alarm_data(1, 1).await?; // Code only
let alarm_name = client.read_alarm_data(1, 5).await?; // Name only
```

> **Note**: For detailed alarm operations examples, see [`examples/alarm_operations.rs`](moto-hses-client/examples/alarm_operations.rs)

### Advanced Usage with Custom Configuration

```rust
use moto_hses_client::{HsesClient, ClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = ClientConfig {
        timeout: Duration::from_millis(500),
        retry_count: 5,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
    };

    // Create client with custom configuration
    let client = HsesClient::new_with_config("192.168.1.100:10040", config).await?;

    // Read different variable types
    let int_value: i32 = client.read_variable::<i32>(0).await?;
    let float_value: f32 = client.read_variable::<f32>(1).await?;
    let byte_value: u8 = client.read_variable::<u8>(2).await?;

    println!("D000: {}, R001: {}, B002: {}", int_value, float_value, byte_value);

    Ok(())
}
```

### Available Examples

```bash
# Basic usage (status, position, variables, alarm data)
cargo run -p moto-hses-client --example basic_usage -- 127.0.0.1 10040

# Detailed alarm operations
cargo run -p moto-hses-client --example alarm_operations -- 127.0.0.1 10040

# Other examples
cargo run -p moto-hses-client --example connection_management -- 127.0.0.1 10040
cargo run -p moto-hses-client --example file_operations -- 127.0.0.1 10041
cargo run -p moto-hses-client --example read_status -- 127.0.0.1 10040
```

### Mock Server Testing

#### Manual Testing

```bash
# Terminal 1: Start mock server
cargo run -p moto-hses-mock --example mock_basic_usage

# Terminal 2: Run client examples against mock
cargo run -p moto-hses-client --example basic_usage -- 127.0.0.1 10040
cargo run -p moto-hses-client --example alarm_operations -- 127.0.0.1 10040
```

#### Automated Integration Testing

```bash
# Run protocol communication tests
cargo test --test protocol_communication_tests

# Run end-to-end integration tests
./scripts/integration_test.sh
```

**Protocol communication tests** verify:

- Mock server protocol implementation
- Message encoding/decoding
- Command handling
- UDP communication with mock server

**End-to-end tests** verify:

- Client-server communication
- All client operations with validation
- Communication integrity
- Automatic resource cleanup

## Implementation Status

### Phase 1: Protocol Layer (moto-hses-proto) ✅

- [x] Protocol specification
- [x] Message types and structures
- [x] Serialization/deserialization
- [x] Error handling

### Phase 2: Client Layer (moto-hses-client) ✅

- [x] Client architecture design
- [x] API design
- [x] Basic client implementation
- [x] Connection management
- [x] Error handling and retry logic
- [x] Variable read/write operations
- [x] Status and position reading
- [x] Convenience methods

### Phase 3: Mock Server (moto-hses-mock) 🔄

- [x] Mock server design
- [x] Mock server implementation
- [x] Test utilities

### Phase 4: Testing & Documentation 🔄

- [x] Testing strategy
- [x] Unit tests
- [ ] Integration tests
- [ ] Performance tests
- [x] Basic documentation

## ⚠️ Implementation Notes

### Currently Implemented Features

- ✅ Client connection and configuration
- ✅ Variable reading/writing (Integer, Float, Byte)
- ✅ Robot status reading
- ✅ Position reading
- ✅ Alarm data reading (0x70 command)
- ✅ Convenience methods for status checking
- ✅ Error handling and retry logic

### Partially Implemented Features

- 🔄 I/O operations (`read_io`, `write_io`) - Basic structure exists but not fully implemented
- 🔄 Job control (`execute_job`, `stop_job`) - Basic structure exists but not fully implemented

### Planned Features

- 📋 Multiple variable batch operations
- 📋 File operations
- 📋 Advanced robot control commands

## Development

### Prerequisites

- Rust 1.70+
- Tokio runtime
- Network access for UDP communication

### Building

```bash
# Build all crates
cargo build

# Run examples
cargo run -p moto-hses-client --example basic_usage -- 127.0.0.1 10040
```

### Testing

```bash
# Unit tests
cargo test

# Protocol communication tests (Mock server protocol)
cargo test --test protocol_communication_tests

# End-to-end integration tests (Client + Mock server)
./scripts/integration_test.sh
```

## License

Apache-2.0

## References

- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) — C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) — Python reference implementation
- [FS100 HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf) — Official HSES documentation
