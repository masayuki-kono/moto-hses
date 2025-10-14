# moto-hses-proto

[![Crates.io](https://img.shields.io/crates/v/moto-hses-proto)](https://crates.io/crates/moto-hses-proto)
[![Documentation](https://docs.rs/moto-hses-proto/badge.svg)](https://docs.rs/moto-hses-proto)
[![License](https://img.shields.io/crates/l/moto-hses-proto)](https://crates.io/crates/moto-hses-proto)
[![Gate of Main](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/gate-of-main.yml)
[![Security Audit](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml/badge.svg)](https://github.com/masayuki-kono/moto-hses/actions/workflows/security-audit.yml)
![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/masayuki-kono/moto-hses?utm_source=oss&utm_medium=github&utm_campaign=masayuki-kono%2Fmoto-hses&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

Protocol definitions and serialization for Yaskawa High-Speed Ethernet Server (HSES).

## Overview

This crate provides the core protocol definitions and serialization/deserialization functionality for communicating with Yaskawa robots using the HSES (High Speed Ethernet Server) protocol over UDP.

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
| 0x302 | Plural Byte Variable (B) Reading / Writing Command |

### File Control Commands

| Service | Command Name |
|---------|--------------|
| 0x09 | File Delete |
| 0x16 | File saving command (Controller to the PC) |
| 0x32 | File list acquiring |

## Features

- **Type-safe protocol definitions**: Rust structs and enums for all HSES message types
- **Efficient serialization**: Zero-copy deserialization where possible
- **Comprehensive error handling**: Detailed error types for protocol violations
- **Japanese language support**: Proper handling of Japanese text (Shift-JIS) in robot data

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
moto-hses-proto = "0.1"
```

## Usage

```rust
use moto_hses_proto::{HsesRequestMessage, HsesResponseMessage, Service, Division, ReadAlarmData, Alarm, AlarmAttribute};

// Create a read alarm command
let read_alarm = ReadAlarmData::new(1, AlarmAttribute::All);

// Create HSES request message
let request = HsesRequestMessage::new(
    Division::Robot as u8,  // division
    0,                      // ack (request)
    1,                      // request_id
    ReadAlarmData::command_id(), // command
    read_alarm.instance(),  // instance
    read_alarm.attribute(), // attribute
    read_alarm.service(),   // service
    vec![],                 // payload
)?;

// Serialize to bytes
let request_bytes = request.encode();

// Deserialize from bytes
let parsed_request = HsesRequestMessage::decode(&request_bytes)?;

// Example: Create an alarm for testing
let alarm = Alarm::new(
    1001,  // alarm code
    0,     // data
    0,     // alarm type
    "2024-01-01 12:00:00".to_string(), // time
    "Test alarm".to_string()           // name
);
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
