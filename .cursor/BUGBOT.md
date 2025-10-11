# BUGBOT Rules for moto-hses

## Project Overview
moto-hses is a Rust client library for Yaskawa's HSES (High-Speed Ethernet Server) protocol. It consists of three crates: moto-hses-proto, moto-hses-client, and moto-hses-mock.

## Code Review Focus Areas

### 1. Architecture Consistency
- **Layer Separation**: Verify proper separation of responsibilities between protocol, client, and test layers

### 2. HSES Protocol Compliance
- **Command Implementation**: Verify command implementations comply with HSES protocol specifications
- **Data Structures**: Check accuracy of byte order, padding, and data types

### 3. Documentation Quality
- Verify that code examples in the following documents are consistent with the code changes in the PR:
    - README.md
    - moto-hses-proto/README.md
    - moto-hses-mock/README.md
    - moto-hses-client/README.md

## Auto-Checked Items (No Review Required)
The following items are automatically checked by GitHub Actions, so no manual review is needed:
- Code formatting (`cargo fmt`)
- Clippy warnings (`cargo clippy`)
- Unit and integration test execution
- Documentation build
- Security audit

## Reference Materials
- [Implementation Rules](.cursor/rules/implementation-rules.mdc) - Development rules and testing strategy
- [HSES Protocol Specification](docs/specs/hses-protocol.md) - Protocol specification
- [Architecture Design](docs/design/architecture.md) - Architecture design
- [Protocol Commands Reference](docs/design/protocol-commands.md) - Command reference
