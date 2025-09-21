# moto-hses-mock

[![Crates.io](https://img.shields.io/crates/v/moto-hses-mock)](https://crates.io/crates/moto-hses-mock)
[![Documentation](https://docs.rs/moto-hses-mock/badge.svg)](https://docs.rs/moto-hses-mock)
[![License](https://img.shields.io/crates/l/moto-hses-mock)](https://crates.io/crates/moto-hses-mock)
[![Gate of Main](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml)
[![Security Audit](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml)

Mock HSES UDP server for testing and development.

## Overview

This crate provides a mock implementation of the Yaskawa High-Speed Ethernet Server (HSES) protocol for testing and development purposes. It simulates a real Yaskawa robot's HSES server, allowing you to test your client applications without requiring actual hardware.

## Features

- **Full HSES protocol support**: Implements all major HSES protocol commands
- **Configurable responses**: Customize robot behavior and responses
- **Async implementation**: Built on Tokio for high-performance testing
- **Logging support**: Detailed logging for debugging
- **Easy integration**: Simple API for test setup
- **Realistic simulation**: Mimics real robot behavior patterns

## Usage

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
moto-hses-mock = "0.0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use moto_hses_mock::MockServer;

#[tokio::test]
async fn test_client_operations() {
    // Start mock server
    let server = MockServer::new("127.0.0.1:10040").await.unwrap();
    
    // Your client code here
    let client = moto_hses_client::HsesClient::new("127.0.0.1:10040").await.unwrap();
    let status = client.read_status().await.unwrap();
    
    // Assertions
    assert!(status.is_running());
}
```

## Basic Usage

### Starting a Mock Server

```rust
use moto_hses_mock::MockServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start mock server on default address
    let server = MockServer::new("127.0.0.1:10040").await?;
    
    // Server will run until the program exits
    server.run().await?;
    
    Ok(())
}
```

### Custom Configuration

```rust
use moto_hses_mock::{MockServer, MockConfig};

#[tokio::test]
async fn test_with_custom_config() {
    let config = MockConfig::default()
        .with_robot_running(true)
        .with_alarm_code(0)
        .with_variable_value(0, 42);
    
    let server = MockServer::with_config("127.0.0.1:10040", config).await.unwrap();
    
    // Test your client with custom configuration
}
```

## Supported Operations

The mock server supports all major HSES operations:

### Variable Operations
- Read/write integer, real, string, and position variables
- Configurable variable values
- Batch operations

### I/O Operations
- Digital I/O simulation
- Analog I/O operations
- I/O status monitoring

### File Operations
- File upload/download simulation
- File management operations
- Directory operations

### Status Operations
- Robot status simulation
- Alarm data generation
- System information

### Position Operations
- Cartesian position simulation
- Joint position data
- Position monitoring

## Examples

The crate includes examples demonstrating various usage patterns:

- `mock_basic_usage.rs` - Basic mock server usage
- Integration with `moto-hses-client` for end-to-end testing

### Running Examples

```bash
# Run the basic usage example
cargo run --example mock_basic_usage

# Run with custom address and port
cargo run --example mock_basic_usage -- 192.168.1.100 10040 10041
```

## Testing Integration

### With moto-hses-client

```rust
use moto_hses_client::HsesClient;
use moto_hses_mock::MockServer;

#[tokio::test]
async fn test_variable_operations() {
    // Start mock server
    let _server = MockServer::new("127.0.0.1:10040").await.unwrap();
    
    // Create client
    let client = HsesClient::new("127.0.0.1:10040").await.unwrap();
    
    // Test variable operations
    let value = client.read_i32(0).await.unwrap();
    assert_eq!(value, 0); // Default mock value
    
    client.write_i32(0, 42).await.unwrap();
    let value = client.read_i32(0).await.unwrap();
    assert_eq!(value, 42);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_integration() {
    let server = MockServer::new("127.0.0.1:10040").await.unwrap();
    let client = HsesClient::new("127.0.0.1:10040").await.unwrap();
    
    // Run comprehensive integration tests
    test_all_client_operations(&client).await;
}
```

## Configuration

The mock server can be configured with various options:

```rust
use moto_hses_mock::{MockServer, MockConfig};

let config = MockConfig::default()
    .with_robot_running(true)
    .with_alarm_code(0)
    .with_variable_value(0, 42)
    .with_io_state(0, true);

let server = MockServer::with_config("127.0.0.1:10040", config).await?;
```

## Logging

Enable logging to see detailed server activity:

```rust
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let server = MockServer::new("127.0.0.1:10040").await.unwrap();
    server.run().await.unwrap();
}
```

## Dependencies

- **tokio**: Async runtime
- **log**: Logging framework
- **env_logger**: Logging implementation
- **moto-hses-proto**: Protocol definitions

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](https://github.com/masayuki-kono/moto-hses/blob/main/LICENSE) file for details.

## Related Crates

- [`moto-hses-proto`](https://crates.io/crates/moto-hses-proto) - Protocol definitions and serialization
- [`moto-hses-client`](https://crates.io/crates/moto-hses-client) - Async UDP client for HSES communication

## References

- [Yaskawa HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf)
- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) - C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) - Python reference implementation
