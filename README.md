# moto-hses

Rust implementation of Yaskawa High-Speed Ethernet Server (HSES) client library.

## Overview

This library provides a type-safe, asynchronous Rust client for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol. Based on the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet).

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
    // Create client
    let client = HsesClient::new("192.168.1.100:10040").await?;

    // Read robot status
    let status = client.read_status().await?;
    println!("Robot running: {}", status.is_running());

    // Read variable
    let value: i32 = client.read_variable(0, VariableType::Integer).await?;
    println!("D000: {}", value);

    // Write variable
    client.write_variable(0, 42i32).await?;

    // Execute job
    client.execute_job(1).await?;

    // Read I/O data
    let input = client.read_io(IoType::RobotUserInput, 1).await?;
    println!("Input 1: {}", input);

    Ok(())
}
```

### Mock Server Testing

```bash
# Terminal 1: Start mock server
cargo run -p moto-hses-mock

# Terminal 2: Run client example against mock
cargo run -p moto-hses-client --example read_status -- 127.0.0.1 10040

# Terminal 3: Run file operations example
cargo run -p moto-hses-client --example file_operations -- 127.0.0.1 10041
```

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

### Phase 3: Mock Server (moto-hses-mock) 🔄

- [x] Mock server design
- [ ] Mock server implementation
- [ ] Test utilities

### Phase 4: Testing & Documentation 🔄

- [x] Testing strategy
- [ ] Unit tests
- [ ] Integration tests
- [ ] Performance tests
- [ ] Documentation

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

# Integration tests
cargo test --test integration_test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests
5. Submit a pull request

## License

Apache-2.0

## References

- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) — C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) — Python reference implementation
- [FS100 HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf) — Official HSES documentation
