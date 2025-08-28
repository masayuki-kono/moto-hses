# HSES Protocol Specification

## Manual

[FS100 OPTIONS INSTRUCTIONS FOR HIGH-SPEED ETHERNET SERVER FUNCTION](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf)

## Protocol Overview

HSES (High Speed Ethernet Server) is a UDP-based communication protocol for Yaskawa robots.

### Communication Specifications

- **Protocol**: UDP
- **Default Port**: 10040
- **Endianness**: Big-endian
- **Timeout**: 300ms (recommended)

### Message Structure

#### Header (8 bytes)

```
0-1:   Message ID (2 bytes, big-endian)
2-3:   Command ID (2 bytes, big-endian)
4-5:   Data Length (2 bytes, big-endian)
6-7:   Reserved (2 bytes, always 0)
```

#### Data Section

- Variable length
- Structure varies depending on the command

## Main Commands

### 1. Read Variable

- **Command ID**: 0x0001
- **Data Structure**:
  ```
  0-1:   Variable Type (2 bytes)
  2-3:   Variable Number (2 bytes)
  ```

#### Variable Types

- `0x0001`: Integer 32-bit
- `0x0002`: Float 32-bit
- `0x0003`: String
- `0x0004`: Position (6 floats)

### 2. Write Variable

- **Command ID**: 0x0002
- **Data Structure**:
  ```
  0-1:   Variable Type (2 bytes)
  2-3:   Variable Number (2 bytes)
  4-:    Variable Data (variable length)
  ```

### 3. Execute Job

- **Command ID**: 0x0003
- **Data Structure**:
  ```
  0-1:   Job Number (2 bytes)
  2-3:   Reserved (2 bytes)
  ```

### 4. Get Status

- **Command ID**: 0x0004
- **Data Structure**: None

## Response Structure

### Success Response

- **Message ID**: Same as request
- **Command ID**: Same as request
- **Status**: 0x0000 (success)
- **Data**: Command-specific data

### Error Response

- **Message ID**: Same as request
- **Command ID**: Same as request
- **Status**: Error code
- **Data**: Error message (optional)

#### Error Codes

- `0x0001`: Invalid Command
- `0x0002`: Invalid Variable
- `0x0003`: Invalid Data
- `0x0004`: Communication Error
- `0x0005`: Timeout
- `0x0006`: System Error

## Implementation Guidelines

### Rust Implementation Considerations

1. **Async Processing**: Asynchronous UDP communication using Tokio
2. **Error Handling**: Type-safe error handling using thiserror
3. **Memory Efficiency**: Efficient byte operations using the bytes crate
4. **Type Safety**: Safe API design leveraging Rust's strong type system

### Recommended Architecture

- **moto-hses-proto**: Protocol definitions and serialization
- **moto-hses-client**: Asynchronous client implementation
- **moto-hses-mock**: Mock server for testing
