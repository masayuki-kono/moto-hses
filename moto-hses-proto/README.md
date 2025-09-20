# moto-hses-proto

[![Crates.io](https://img.shields.io/crates/v/moto-hses-proto)](https://crates.io/crates/moto-hses-proto)
[![Documentation](https://docs.rs/moto-hses-proto/badge.svg)](https://docs.rs/moto-hses-proto)
[![License](https://img.shields.io/crates/l/moto-hses-proto)](https://crates.io/crates/moto-hses-proto)
[![Gate of Main](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml)
[![Security Audit](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml)

Protocol definitions and serialization for Yaskawa High-Speed Ethernet Server (HSES).

## Overview

This crate provides the core protocol definitions and serialization/deserialization functionality for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol over UDP.

## Features

- **Type-safe protocol definitions**: Rust structs and enums for all HSES message types
- **Efficient serialization**: Zero-copy deserialization where possible
- **Comprehensive error handling**: Detailed error types for protocol violations
- **Unicode support**: Proper handling of Japanese text in robot data

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
moto-hses-proto = "0.0.1"
```

## Example

```rust
use moto_hses_proto::*;

// Create a read variable command
let command = Command::ReadVariable {
    variable_type: VariableType::Integer,
    index: 0,
};

// Serialize to bytes
let mut buffer = Vec::new();
command.serialize(&mut buffer)?;

// Deserialize from bytes
let (parsed_command, _) = Command::deserialize(&buffer)?;
```

## Protocol Support

This crate implements the HSES protocol as specified in the official Yaskawa documentation:

- **Variable operations**: Read/write integer, real, string, and position variables
- **I/O operations**: Digital and analog I/O control
- **File operations**: File transfer and management
- **Status monitoring**: Robot status and alarm information
- **Position control**: Cartesian and joint position data

## Error Handling

The crate provides comprehensive error handling through the `HsesError` type:

```rust
use moto_hses_proto::HsesError;

match result {
    Ok(data) => println!("Success: {:?}", data),
    Err(HsesError::InvalidMessage) => println!("Invalid message format"),
    Err(HsesError::UnsupportedCommand) => println!("Command not supported"),
    Err(e) => println!("Other error: {}", e),
}
```

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](https://github.com/masayuki-kono/moto-hses/blob/main/LICENSE) file for details.

## Related Crates

- [`moto-hses-client`](https://crates.io/crates/moto-hses-client) - Async UDP client for HSES communication
- [`moto-hses-mock`](https://crates.io/crates/moto-hses-mock) - Mock HSES server for testing

## References

- [Yaskawa HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf)
- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) - C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) - Python reference implementation
