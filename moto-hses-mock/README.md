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

## Installation

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
moto-hses-mock = "0.0.2"
tokio = { version = "1.0", features = ["full"] }
```

## Usage

```rust
use moto_hses_mock::{MockServer, MockServerBuilder};
use moto_hses_proto::Alarm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start mock server with custom configuration using builder pattern
    let server = MockServerBuilder::new()
        .host("192.168.1.100")
        .robot_port(20000)
        .file_port(20001)
        .with_alarm(Alarm::new(1001, 0, 0, "2024-01-01 12:00:00".to_string(), "Test alarm".to_string()))
        .with_io_state(1, true)
        .build()
        .await?;
    
    // Server will run until the program exits
    server.run().await?;
    
    Ok(())
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

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](https://github.com/masayuki-kono/moto-hses/blob/main/LICENSE) file for details.

## Related Crates

- [`moto-hses-proto`](https://crates.io/crates/moto-hses-proto) - Protocol definitions and serialization
- [`moto-hses-client`](https://crates.io/crates/moto-hses-client) - Async UDP client for HSES communication

## References

- [Yaskawa HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf)
- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) - C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) - Python reference implementation
