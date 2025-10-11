# moto-hses

[![Gate of Main](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml)
[![Security Audit](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml)
![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/masayuki-kono/moto-hses?utm_source=oss&utm_medium=github&utm_campaign=masayuki-kono%2Fmoto-hses&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

Rust implementation of Yaskawa High-Speed Ethernet Server (HSES) client library.

## ⚠️ Important Notice

This repository is an experimental project using LLM-assisted development. Documentation and implementation reviews are incomplete. Please use with caution.

## Overview

This library provides a type-safe, asynchronous Rust client for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol.

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

### File Control Commands

| Service | Command Name |
|---------|--------------|
| 0x09 | File Delete |
| 0x16 | File saving command (Controller to the PC) |
| 0x32 | File list acquiring |

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

### Examples

Comprehensive examples are available in the [`examples/`](moto-hses-client/examples/) directory:

- [`alarm_operations.rs`](moto-hses-client/examples/alarm_operations.rs) — Alarm data handling
- [`io_operations.rs`](moto-hses-client/examples/io_operations.rs) — I/O operations
- [`position_operations.rs`](moto-hses-client/examples/position_operations.rs) — Position data operations
- [`variable_operations.rs`](moto-hses-client/examples/variable_operations.rs) — Variable read/write operations
- [`file_operations.rs`](moto-hses-client/examples/file_operations.rs) — File transfer operations
- [`hold_servo_control.rs`](moto-hses-client/examples/hold_servo_control.rs) — Servo control operations
- [`register_operations.rs`](moto-hses-client/examples/register_operations.rs) — Register operations
- [`read_executing_job_info.rs`](moto-hses-client/examples/read_executing_job_info.rs) — Job information
- [`read_status.rs`](moto-hses-client/examples/read_status.rs) — Status monitoring

### Running Examples

```bash
# Run a specific example
RUST_LOG=info cargo run -p moto-hses-client --example alarm_operations -- 192.168.0.3 10040
```

### Mock Server Testing

#### Manual Testing

```bash
# Terminal 1: Start mock server
RUST_LOG=info cargo run -p moto-hses-mock --example mock_basic_usage

# Terminal 2: Run client examples against mock
RUST_LOG=info cargo run -p moto-hses-client --example alarm_operations -- 127.0.0.1 10040
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

**Integration tests** verify:

- Client-server communication
- All client operations with validation

## References

- [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) — C++ reference implementation
- [hsinkoyu/fs100](https://github.com/hsinkoyu/fs100) — Python reference implementation
- [FS100 HSES Manual](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf) — Official HSES documentation
