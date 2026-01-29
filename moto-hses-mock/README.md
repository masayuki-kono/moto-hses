# moto-hses-mock

[![Crates.io](https://img.shields.io/crates/v/moto-hses-mock)](https://crates.io/crates/moto-hses-mock)
[![Documentation](https://docs.rs/moto-hses-mock/badge.svg)](https://docs.rs/moto-hses-mock)
[![License](https://img.shields.io/crates/l/moto-hses-mock)](https://crates.io/crates/moto-hses-mock)
[![Gate of Main](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml)
[![Security Audit](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml)
![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/masayuki-kono/moto-hses?utm_source=oss&utm_medium=github&utm_campaign=masayuki-kono%2Fmoto-hses&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

Mock HSES UDP server for testing and development.

## Overview

This crate provides a mock implementation of the Yaskawa High-Speed Ethernet Server (HSES) protocol for testing and development purposes. It simulates a real Yaskawa robot's HSES server, allowing you to test your client applications without requiring actual hardware.

## Verified Robot Models

The following robot models have been tested and verified for compatibility:

| Robot Model | Status |
|-------------|--------|
| DX100 | ❌ Not verified |
| FS100 | ❌ Not verified |
| DX200 | ❌ Not verified |
| YRC1000 | ❌ Not verified |
| YRC1000micro | ✅ Verified |

## Supported Commands

### Robot Control Commands

| Command No | Command Name |
|------------|--------------|
| 0x70 | Alarm Data Reading Command |
| 0x71 | Alarm History Reading Command |
| 0x72 | Read Status Information |
| 0x73 | Executing Job Information Reading Command |
| 0x75 | Robot Position Data Reading Command |
| 0x78 | I/O Data Reading / Writing Command |
| 0x79 | Register Data Reading / Writing Command |
| 0x7A | Byte Variable (B) Reading / Writing Command |
| 0x7B | Integer Type Variable (I) Reading / Writing Command |
| 0x7C | Double Precision Integer Type Variable (D) Reading / Writing Command |
| 0x7D | Real Type Variable (R) Reading / Writing Command |
| 0x7E | Character Type Variable (S) Reading / Writing Command |
| 0x82 | Alarm Reset / Error Cancel Command |
| 0x83 | Hold / Servo On/off Command |
| 0x84 | Step / Cycle / Continuous Switching Command |
| 0x86 | Start-up (Job Start) Command |
| 0x87 | Job Select Command |
| 0x300 | Plural I/O Data Reading / Writing Command |
| 0x301 | Plural Register Data Reading / Writing Command |
| 0x302 | Plural Byte Type Variable (B) Reading / Writing Command |
| 0x303 | Plural Integer Type Variable (I) Reading / Writing Command |
| 0x304 | Plural Double Precision Integer Type Variable (D) Reading / Writing Command |
| 0x305 | Plural Real Type Variable (R) Reading / Writing Command |
| 0x306 | Plural Character Type Variable (S) Reading / Writing Command |

### File Control Commands

| Service | Command Name |
|---------|--------------|
| 0x09 | File Delete |
| 0x16 | File saving command (Controller to the PC) |
| 0x32 | File list acquiring |

## Features

- **Full HSES protocol support**: Implements all major HSES protocol commands
- **Configurable responses**: Customize robot behavior and responses
- **Async implementation**: Built on Tokio for high-performance testing

## Installation

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
moto-hses-mock = "0.3.3"
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
        .host("192.168.0.3")
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

## Examples

The crate includes examples demonstrating various usage patterns:

- `mock_basic_usage.rs` - Basic mock server usage
- Integration with `moto-hses-client` for end-to-end testing

### Running Examples

```bash
# Run the basic usage example
cargo run --example mock_basic_usage

# Run with custom address and port
cargo run --example mock_basic_usage -- 192.168.0.3 10040 10041
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
