# moto-hses-client

[![Crates.io](https://img.shields.io/crates/v/moto-hses-client)](https://crates.io/crates/moto-hses-client)
[![Documentation](https://docs.rs/moto-hses-client/badge.svg)](https://docs.rs/moto-hses-client)
[![License](https://img.shields.io/crates/l/moto-hses-client)](https://crates.io/crates/moto-hses-client)
[![Gate of Main](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml)
[![Security Audit](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml)
![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/masayuki-kono/moto-hses?utm_source=oss&utm_medium=github&utm_campaign=masayuki-kono%2Fmoto-hses&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

Async UDP client for Yaskawa High-Speed Ethernet Server (HSES) communication.

## Overview

This crate provides a high-level, type-safe, asynchronous Rust client for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol over UDP. It's built on top of Tokio and provides a modern async/await API.

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

### File Control Commands

| Service | Command Name |
|---------|--------------|
| 0x09 | File Delete |
| 0x16 | File saving command (Controller to the PC) |
| 0x32 | File list acquiring |

## Features

- **Async/await support**: Built on Tokio for high-performance async I/O
- **Type-safe API**: Leverages Rust's type system for compile-time safety
- **Comprehensive operations**: Support for all HSES protocol operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
moto-hses-client = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Usage

```rust
use moto_hses_client::HsesClient;
use moto_hses_proto::AlarmAttribute;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = HsesClient::new("192.168.0.3:10040").await?;

    // Read alarm data
    let alarm = client.read_alarm_data(1, AlarmAttribute::All).await?;
    println!("Alarm Code: {}", alarm.code);
    println!("Alarm Name: {}", alarm.name);

    // Reset alarm
    client.reset_alarm().await?;
    println!("Alarm reset completed");

    Ok(())
}
```

## Examples

The crate includes comprehensive examples in the `examples/` directory:

- `alarm_operations.rs` - Alarm data handling
- `cycle_mode_control.rs` - Cycle mode switching operations
- `io_operations.rs` - I/O operations
- `job_start.rs` - Job start operations
- `position_operations.rs` - Position data operations
- `variable_operations.rs` - Variable read/write operations
- `file_operations.rs` - File transfer operations
- `hold_servo_control.rs` - Servo control operations
- `register_operations.rs` - Register operations
- `read_executing_job_info.rs` - Job information
- `read_status.rs` - Status monitoring

### Running Examples

```bash
# Run a specific example
RUST_LOG=info cargo run --example alarm_operations -- 192.168.0.3 10040
```

## Testing

The crate can be tested using the separate `moto-hses-mock` crate:

```toml
[dev-dependencies]
moto-hses-mock = "0.1"
```

```rust
use moto_hses_client::HsesClient;
use moto_hses_mock::{MockServer, MockServerBuilder};
use moto_hses_proto::Alarm;

#[tokio::test]
async fn test_alarm_operations() {
    // Start mock server with test alarm
    let server = MockServerBuilder::new()
        .host("127.0.0.1")
        .robot_port(10040)
        .file_port(10041)
        .with_alarm(Alarm::new(1001, 0, 0, "2024-01-01 12:00:00".to_string(), "Test alarm".to_string()))
        .build()
        .await
        .unwrap();
    
    // Start the server in the background
    let server_handle = tokio::spawn(async move {
        server.run().await.unwrap();
    });
    
    let client = HsesClient::new("127.0.0.1:10040").await.unwrap();
    
    // Test alarm operations
    let alarm = client.read_alarm_data(1, AlarmAttribute::All).await.unwrap();
    assert_eq!(alarm.code, 1001);
    assert_eq!(alarm.name, "Test alarm");
    
    // Test alarm reset
    client.reset_alarm().await.unwrap();
    
    // Clean up
    server_handle.abort();
}
```

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](https://github.com/masayuki-kono/moto-hses/blob/main/LICENSE) file for details.

## Related Crates

- [`moto-hses-proto`](https://crates.io/crates/moto-hses-proto) - Protocol definitions and serialization
- [`moto-hses-mock`](https://crates.io/crates/moto-hses-mock) - Mock HSES server for testing

## References

- [Yaskawa HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf)
- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) - C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) - Python reference implementation
