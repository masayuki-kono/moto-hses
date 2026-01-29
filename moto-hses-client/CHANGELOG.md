# Changelog

All notable changes to this project will be documented in this file.

## [0.3.3] - 2026-01-29

### Changed
- Version bump for patch release

## [0.3.2] - 2026-01-28

### Documentation
- Add new agent skills (hses-packet-analysis, rust-code-quality-guide) to README documentation

## [0.3.1] - 2026-01-26

### Changed
- Version bump for patch release

## [0.3.0] - 2026-01-26

### Features
- **Thread-Safe Client**:
  - Add `SharedHsesClient` wrapper for thread-safe concurrent access
  - Add `HsesClientOps` trait for abstraction over `HsesClient` and `SharedHsesClient`
  - Support safe concurrent access from multiple Tokio tasks

## [0.2.1] - 2026-01-25

### Changed
- Remove unused third-party/fs100 submodule
- Migrate documentation references to external skills

## [0.2.0] - 2025-10-17

### Features
- **Robot Control Commands**:
  - Start-up (Job Start) (0x86)
  - Job Select (0x87)
  - Plural I/O Data Reading/Writing (0x300)
  - Plural Register Data Reading/Writing (0x301)
  - Plural Variable Operations (0x302-0x306): Byte, Integer, Double Precision Integer, Real, Character types

## [0.1.2] - 2025-10-07

### Changed
- Update moto-hses-proto dependency to version 0.1

## [0.1.1] - 2025-10-07

### Documentation
- Add verified robot models and supported commands sections to README

## [0.1.0] - 2025-10-07

### Features
- **Robot Control Commands**:
  - Alarm Data Reading (0x70)
  - Alarm History Reading (0x71)
  - Status Information Reading (0x72)
  - Executing Job Information Reading (0x73)
  - Robot Position Data Reading (0x75)
  - I/O Data Reading/Writing (0x78)
  - Register Data Reading/Writing (0x79)
  - Variable Operations (0x7A-0x7E): Byte, Integer, Double Precision Integer, Real, Character types
  - Alarm Reset/Error Cancel (0x82)
  - Hold/Servo On/off (0x83)
  - Step/Cycle/Continuous Switching (0x84)

- **File Control Commands**:
  - File Delete (0x09)
  - File Saving from Controller to PC (0x16)
  - File List Acquiring (0x32)

### Testing Status
- ✅ YRC1000micro robot model verified
- ⚠️ Other robot models (DX100, FS100, DX200, YRC1000) not yet verified

## [0.0.2] - 2025-09-22

### Changed
- Updated README.md with alarm-based usage examples
- Improved documentation accuracy and consistency

## [0.0.1] - 2025-09-22

### Added
- Initial release of moto-hses-client
- Basic HSES client implementation with UDP communication
- Alarm operations functionality (verified working)

### Features
- **Alarm Operations**: Read and clear robot alarms

### Testing Status
- ✅ Alarm operations functionality verified
- ⚠️ Other features require further testing

---
