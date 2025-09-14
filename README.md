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
- [`docs/design/protocol-commands.md`](docs/design/protocol-commands.md) — HSES Protocol Command Reference

### Testing

- [`docs/test/testing-strategy.md`](docs/test/testing-strategy.md) — Testing strategy and best practices

## Quick Start

### Basic Usage

```rust
use moto_hses_client::{HsesClient, VariableType, ClientError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
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

    // Check robot status
    let status = client.read_status().await?;
    println!("Running: {}, Servo: {}, Alarm: {}",
             status.is_running(), status.is_servo_on(), status.has_alarm());

    Ok(())
}
```

### Alarm Operations

```rust
use moto_hses_client::{HsesClient, Alarm, ClientError};

// Read alarm data
let alarm = client.read_alarm_data(1, 0).await?;
println!("Alarm: Code={}, Name={}", alarm.code, alarm.name);

// Read specific alarm attributes
let alarm_code = client.read_alarm_data(1, 1).await?; // Code only
let alarm_name = client.read_alarm_data(1, 5).await?; // Name only
```

> **Note**: For detailed alarm operations examples, see [`examples/alarm_operations.rs`](moto-hses-client/examples/alarm_operations.rs)

### I/O Operations

```rust
use moto_hses_client::{HsesClient, ClientError};

// Read I/O state
let io_state = client.read_io(1).await?; // Read robot user input #1
println!("I/O #1 state: {}", if io_state { "ON" } else { "OFF" });

// Write I/O state
client.write_io(1001, true).await?; // Set robot user output #1001 to ON
```

> **Note**: For detailed I/O operations examples, see [`examples/io_operations.rs`](moto-hses-client/examples/io_operations.rs)

### Advanced Usage with Custom Configuration

```rust
use moto_hses_client::{HsesClient, ClientConfig, ClientError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Create custom configuration
    let config = ClientConfig {
        host: "192.168.1.100".to_string(),
        port: 10040,
        timeout: Duration::from_millis(500),
        retry_count: 5,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
    };

    // Create client with custom configuration
    let client = HsesClient::new_with_config(config).await?;

    // Read different variable types
    let int_value: i32 = client.read_variable::<i32>(0).await?;
    let float_value: f32 = client.read_variable::<f32>(1).await?;
    let byte_value: u8 = client.read_variable::<u8>(2).await?;

    println!("D000: {}, R001: {}, B002: {}", int_value, float_value, byte_value);

    Ok(())
}
```

### Mock Server Testing

#### Manual Testing

```bash
# Terminal 1: Start mock server
cargo run -p moto-hses-mock --example mock_basic_usage

# Terminal 2: Run client examples against mock
cargo run -p moto-hses-client --example basic_usage -- 127.0.0.1 10040
cargo run -p moto-hses-client --example alarm_operations -- 127.0.0.1 10040
cargo run -p moto-hses-client --example io_operations -- 127.0.0.1 10040
```

#### Automated Integration Testing

```bash
# Run protocol communication tests
cargo test --test protocol_communication_tests

# Run comprehensive integration tests
cargo test --test integration_tests
```

**Protocol communication tests** verify:

- Mock server protocol implementation
- Message encoding/decoding
- Command handling
- UDP communication with mock server

**Integration tests** verify:

- Client-server communication
- All client operations with validation
- Communication integrity
- Automatic resource cleanup
- MockServer configuration and expected value validation

## License

Apache-2.0

## References

- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) — C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) — Python reference implementation
- [FS100 HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf) — Official HSES documentation
