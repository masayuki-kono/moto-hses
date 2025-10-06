# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2025-01-27

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
- Updated README.md with MockServerBuilder usage examples
- Improved documentation accuracy and consistency

## [0.0.1] - 2025-09-22

### Added
- Initial release of moto-hses-mock
- Mock HSES UDP server for testing and development
- Alarm operations mock implementation (verified working)
- Configurable mock responses for different test scenarios

### Features
- **Alarm Operations**: Mock server responses for alarm reading and clearing

### Testing Status
- ✅ Alarm operations functionality verified
- ⚠️ Other features require further testing

---
