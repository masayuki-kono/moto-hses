# HSES Protocol Specification

## Manual

[FS100 OPTIONS INSTRUCTIONS FOR HIGH-SPEED ETHERNET SERVER FUNCTION](https://www.motoman.com/getmedia/16B5CD92-BD0B-4DE0-9DC9-B71D0B6FE264/160766-1CD.pdf.aspx?ext=.pdf)

## Protocol Overview

HSES (High Speed Ethernet Server) is a UDP-based communication protocol for Yaskawa robots.

### Communication Specifications

- **Protocol**: UDP
- **Ports**:
  - Robot Control: 10040
  - File Control: 10041
- **Endianness**: Little-endian

### Message Structure

#### Header (32 bytes)

```
0-3:   Magic bytes "YERC" (4 bytes)
4-5:   Header size (2 bytes, little-endian, always 0x20)
6-7:   Payload size (2 bytes, little-endian)
8:     Reserved magic constant (1 byte, always 0x03)
9:     Division (1 byte)
        - 0x01: Robot commands
        - 0x02: File commands
10:    ACK (1 byte)
        - 0x00: Request
        - 0x01: Response
11:    Request ID (1 byte)
12-15: Block number (4 bytes, little-endian)
16-23: Reserved (8 bytes, always "99999999")
24-25: Command (2 bytes, little-endian)
26-27: Instance (2 bytes, little-endian)
28:    Attribute (1 byte)
29:    Service (1 byte)
30-31: Padding (2 bytes, always 0x00)
```

#### Payload Section

- Variable length (max 0x1DF bytes)
- Structure varies depending on the command and service

## Main Commands

### Robot Commands (Division = 0x01)

#### 1. Read Status Information

- **Command**: 0x0072
- **Instance**: 1
- **Attribute**: 0
- **Service**: 0x01 (get_all)
- **Payload**: None

#### 2. Read Robot Position

- **Command**: 0x0075
- **Instance**: Control group + offset
  - Robot pulse: +1
  - Base pulse: +11
  - Station pulse: +21
  - Robot cartesian: +101
- **Attribute**: 0
- **Service**: 0x01 (get_all)
- **Payload**: None

#### 3. Read/Write Variables

##### Integer 32-bit (D variables)

- **Command**: 0x007c
- **Instance**: Variable number
- **Attribute**: 0
- **Service**: 0x0e (get_single) / 0x10 (set_single)
- **Payload**: 4 bytes (little-endian)

##### Float 32-bit (R variables)

- **Command**: 0x007d
- **Instance**: Variable number
- **Attribute**: 0
- **Service**: 0x0e (get_single) / 0x10 (set_single)
- **Payload**: 4 bytes (little-endian)

##### String (S variables)

- **Command**: 0x007e
- **Instance**: Variable number
- **Attribute**: 0
- **Service**: 0x0e (get_single) / 0x10 (set_single)
- **Payload**: Variable length string

##### Robot Position (P variables)

- **Command**: 0x007f
- **Instance**: Variable number
- **Attribute**: 0
- **Service**: 0x0e (get_single) / 0x10 (set_single)
- **Payload**: Position data (see Position Format)

#### 4. Execute Job

- **Command**: 0x0073
- **Instance**: Job number
- **Attribute**: 0
- **Service**: 0x02 (set_all)
- **Payload**: None

#### 5. Start Job

- **Command**: 0x0086
- **Instance**: Job number
- **Attribute**: 0
- **Service**: 0x02 (set_all)
- **Payload**: None

#### 6. Set Servo Enabled

- **Command**: 0x0083
- **Instance**: Power type (1=HOLD, 2=SERVO, 3=HLOCK)
- **Attribute**: 0x01
- **Service**: 0x10 (set_single)
- **Payload**: 4 bytes (switch value: 1=ON, 2=OFF)

#### 7. Set Execution Mode

- **Command**: 0x0084
- **Instance**: 2
- **Attribute**: 0x01
- **Service**: 0x10 (set_single)
- **Payload**: 4 bytes (cycle type: 1=STEP, 2=ONE_CYCLE, 3=CONTINUOUS)

#### 8. Show Message on Pendant

- **Command**: 0x0085
- **Instance**: 1
- **Attribute**: 1
- **Service**: 0x10 (set_single)
- **Payload**: 32 bytes (text message, padded with zeros)

#### 9. Read System Information

- **Command**: 0x0089
- **Instance**: System type (11=R1, 12=R2, 21=S1, 22=S2, 23=S3, 101=APPLICATION)
- **Attribute**: 0
- **Service**: 0x01 (get_all)
- **Payload**: None

#### 10. Read Management Time

- **Command**: 0x0088
- **Instance**: Time type (various management time types)
- **Attribute**: 0
- **Service**: 0x01 (get_all)
- **Payload**: None

### File Commands (Division = 0x02)

File commands use a different port (10041) and have a simpler structure.

#### 1. Read File

- **Command**: 0x00
- **Service**: 0x16
- **Payload**: File name

#### 2. Write File

- **Command**: 0x00
- **Service**: 0x15
- **Payload**: File name + data

#### 3. Delete File

- **Command**: 0x00
- **Service**: 0x09
- **Payload**: File name

#### 4. Read File List

- **Command**: 0x00
- **Service**: 0x32
- **Payload**: File extension filter

## Position Format

### Pulse Position

```
0-3:   Position type (4 bytes, little-endian, 0x00)
4-7:   Joint configuration (4 bytes, little-endian, 0x00)
8-11:  Tool number (4 bytes, little-endian)
12-15: User coordinate (4 bytes, little-endian, 0x00)
16-19: Extended joint configuration (4 bytes, little-endian, 0x00)
20-23: Joint 1 (4 bytes, little-endian, pulses)
24-27: Joint 2 (4 bytes, little-endian, pulses)
28-31: Joint 3 (4 bytes, little-endian, pulses)
32-35: Joint 4 (4 bytes, little-endian, pulses)
36-39: Joint 5 (4 bytes, little-endian, pulses)
40-43: Joint 6 (4 bytes, little-endian, pulses)
44-47: Joint 7 (4 bytes, little-endian, pulses)
48-51: Joint 8 (4 bytes, little-endian, pulses)
```

### Cartesian Position

```
0-3:   Position type (4 bytes, little-endian)
       - 0x10: Base frame
       - 0x11: Robot frame
       - 0x12: Tool frame
       - 0x13-0x22: User frames 1-16
4-7:   Joint configuration (4 bytes, little-endian)
8-11:  Tool number (4 bytes, little-endian)
12-15: User coordinate number (4 bytes, little-endian)
16-19: Extended joint configuration (4 bytes, little-endian, 0x00)
20-23: X coordinate (4 bytes, little-endian, micrometers)
24-27: Y coordinate (4 bytes, little-endian, micrometers)
28-31: Z coordinate (4 bytes, little-endian, micrometers)
32-35: RX rotation (4 bytes, little-endian, millidegrees)
36-39: RY rotation (4 bytes, little-endian, millidegrees)
40-43: RZ rotation (4 bytes, little-endian, millidegrees)
44-47: Padding (4 bytes, little-endian, 0x00)
48-51: Padding (4 bytes, little-endian, 0x00)
```

## Response Structure

### Success Response

- **Magic**: "YERC"
- **Header size**: 0x20
- **ACK**: 0x01
- **Status**: 0x00 (success)
- **Extra status**: 0x0000
- **Payload**: Command-specific data

### Error Response

- **Magic**: "YERC"
- **Header size**: 0x20
- **ACK**: 0x01
- **Status**: Error code
- **Extra status**: Additional error information
- **Payload**: Error message (optional)

#### Error Codes

- `0x00`: Success
- `0x01`: Invalid command
- `0x02`: Invalid instance
- `0x03`: Invalid attribute
- `0x04`: Invalid service
- `0x05`: Invalid data
- `0x06`: Communication error
- `0x07`: Timeout
- `0x08`: System error

## Variable Types

### Supported Variable Types

- `B variables`: 8-bit unsigned integers (std::uint8_t)
- `I variables`: 16-bit signed integers (std::int16_t)
- `D variables`: 32-bit signed integers (std::int32_t)
- `R variables`: 32-bit floating point (float)
- `P variables`: Robot positions (Position)
- `S variables`: Strings (std::string)

### Variable Numbering

- Variables are numbered starting from 1
- Instance field contains the variable number
- Different variable types use different commands

## Implementation Guidelines

### Rust Implementation Considerations

1. **Async Processing**: Asynchronous UDP communication using Tokio
2. **Error Handling**: Type-safe error handling using thiserror
3. **Memory Efficiency**: Zero-copy operations using the bytes crate
4. **Type Safety**: Strong type system for safe API design
5. **Little-endian**: All multi-byte values are little-endian

### Recommended Architecture

- **moto-hses-proto**: Protocol definitions and serialization
- **moto-hses-client**: Asynchronous client implementation
- **moto-hses-mock**: Mock server for testing

### Key Implementation Notes

1. **Header Size**: Always 32 bytes (0x20)
2. **Max Payload**: 479 bytes (0x1DF)
3. **Magic Bytes**: "YERC" at the beginning
4. **Request ID**: Unique identifier for request/response matching
5. **Division**: Distinguishes between robot and file commands
6. **Service**: Defines the operation type (get, set, etc.)
