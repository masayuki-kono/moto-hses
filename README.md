# moto-hses

Rust implementation of Yaskawa High-Speed Ethernet Server (HSES) client (skeleton).

## Crates

- `moto-hses-proto` — Protocol types & codecs (placeholder)
- `moto-hses-client` — Async UDP client using Tokio (placeholder)
- `moto-hses-mock` — Local mock HSES UDP server for E2E testing

## Docs

- `docs/specs/hses-protocol.md`
- `docs/specs/io-mapping.md`
- `docs/design/architecture.md`
- `docs/design/client-api.md`
- `docs/design/error-handling.md`
- `docs/adr/0001-adopt-hses.md`

## Quick start (mock server + client example)

```bash
# Terminal 1: start mock server (listens on 127.0.0.1:12222 by default)
cargo run -p moto-hses-mock

# Terminal 2: run client example against the mock
cargo run -p moto-hses-client --example read_status -- 127.0.0.1:12222
```

## Notes

- Wire compatibility is NOT implemented yet. Replace placeholder header/commands with real HSES spec.
