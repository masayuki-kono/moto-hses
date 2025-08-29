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

- Header (32 bytes)
- Payload (Max: 479 bytes)

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
        -  Identifying ID for command session (increment this ID every time the client side outputs a new command. In reply to this, server side answers the received value.)
12-15: Block number (4 bytes, little-endian)
         - Range: 0 to 0x7fff_ffff for normal packets, 0x8000_0000 to 0xffff_ffff for final packets
         - Request: Always 0
         - Response (single response): 0x8000_0000
         - Response (multiple responses):
             - Data packets (except last) Increment by 1 from previous block number
             - Last data packet Add 0x8000_0000 to the previous block number
             - ACK packets Same as the corresponding data packet's block number
16-23: Reserved (8 bytes, always "99999999")
24-31: Sub-header (8 bytes, structure differs for Request/Response)
```

**Example (File Saving with multiple data blocks)**:

- Client → Server: Request (Block number: 0)
- Server → Client: Data1 (Block number: 1)
- Client → Server: ACK1 (Block number: 1)
- Server → Client: Data2 (Block number: 2)
- Client → Server: ACK2 (Block number: 2)
- ...
- Server → Client: DataN (last) (Block number: 0x8000_000N)
- Client → Server: ACKN (Block number: 0x8000_000N)

#### Sub-header Structure

**Request Sub-header (24-31):**

```
24-25: Command (2 bytes, little-endian)
26-27: Instance (2 bytes, little-endian)
28:    Attribute (1 byte)
29:    Service (1 byte)
30-31: Padding (2 bytes, always 0x00)
```

**Response Sub-header (24-31):**

```
24:    Service (1 byte)
        - Add 0x80 to service (request)
25:    Status (1 byte)
        - `0x00`: Normal reply
        - `0x1f`: Abnormal reply (added status size = 1 or 2)
        - Other than `0x1f`: Abnormal reply (added status size = 0)
            - `0x08`:  Requested command is not defined
            - `0x09`:  The element number of the invalid data is detected
            - `0x28`:  An instance of the requested data does not exist in the specified command
26:    Added status size (1 byte)
        - `0`: Not specified
        - `1`: 1 WORD data
        - `2`: 2 WORD data
27:    Padding (1 byte)
28-29: Added status (2 bytes, little-endian)
        - Error code (interpretation depends on added status size)
        - For details, see Added status codes section
30-31: Padding (2 bytes, always 0x00)
```

#### Payload Section

- Variable length (max 0x1DF bytes)
- Structure varies depending on the command and service

## Robot Commands (Division = 0x01)

### Command List

| No. | Command ID | Name                                                                        | Description                                  |
| --- | ---------- | --------------------------------------------------------------------------- | -------------------------------------------- |
| 1   | 0x70       | Alarm data reading command                                                  | Read current alarm data                      |
| 2   | 0x71       | Alarm history reading command                                               | Read alarm history                           |
| 3   | 0x72       | Status information reading command                                          | Read robot status information                |
| 4   | 0x73       | Executing job information reading command                                   | Read currently executing job info            |
| 5   | 0x74       | Axis configuration information reading command                              | Read axis configuration                      |
| 6   | 0x75       | Robot position data reading command                                         | Read robot position data                     |
| 7   | 0x76       | Position error reading command                                              | Read position error                          |
| 8   | 0x77       | Torque data reading command                                                 | Read torque data                             |
| 9   | 0x78       | I/O data reading / writing command                                          | Read/write I/O data                          |
| 10  | 0x79       | Register data reading / writing command                                     | Read/write register data                     |
| 11  | 0x7A       | Byte variable (B) reading / writing command                                 | Read/write byte variables                    |
| 12  | 0x7B       | Integer type variable (I) reading / writing command                         | Read/write integer variables                 |
| 13  | 0x7C       | Double precision integer type variable (D) reading / writing command        | Read/write double integer variables          |
| 14  | 0x7D       | Real type variable (R) reading / writing command                            | Read/write real variables                    |
| 15  | 0x7E       | Character type variable (S) reading / writing command                       | Read/write string variables                  |
| 16  | 0x7F       | Robot position type variable (P) reading / writing command                  | Read/write position variables                |
| 17  | 0x80       | Base position type variable (BP) reading / writing command                  | Read/write base position variables           |
| 18  | 0x81       | External axis type variable (EX) reading / writing command                  | Read/write external axis variables           |
| 19  | 0x82       | Alarm reset / error cancel command                                          | Reset alarms and cancel errors               |
| 20  | 0x83       | HOLD / servo ON/OFF command                                                 | Control HOLD and servo power                 |
| 21  | 0x84       | Step / cycle / continuous switching command                                 | Switch execution modes                       |
| 22  | 0x85       | Character string display command to the programming pendant                 | Display messages on pendant                  |
| 23  | 0x86       | Start-up (job START) command                                                | Start job execution                          |
| 24  | 0x87       | Job select command                                                          | Select job to execute                        |
| 25  | 0x88       | Management time acquiring command                                           | Get management time                          |
| 26  | 0x89       | System information acquiring command                                        | Get system information                       |
| 27  | 0x300      | Plural I/O data reading / writing command                                   | Read/write multiple I/O data                 |
| 28  | 0x301      | Plural register data reading / writing command                              | Read/write multiple register data            |
| 29  | 0x302      | Plural byte type variable (B) reading / writing command                     | Read/write multiple byte variables           |
| 30  | 0x303      | Plural integer type variable (I) reading / writing command                  | Read/write multiple integer variables        |
| 31  | 0x304      | Plural double precision integer type variable (D) reading / writing command | Read/write multiple double integer variables |
| 32  | 0x305      | Plural real type variable (R) reading / writing command                     | Read/write multiple real variables           |
| 33  | 0x306      | Plural character type variable (S) reading / writing command                | Read/write multiple string variables         |
| 34  | 0x307      | Plural robot position type variable (P) reading / writing command           | Read/write multiple position variables       |
| 35  | 0x308      | Plural base position type variable (BP) reading / writing command           | Read/write multiple base position variables  |
| 36  | 0x309      | Plural external axis type variable (EX) reading / writing command           | Read/write multiple external axis variables  |
| 37  | 0x30A      | Alarm data reading command (for applying the sub code character strings)    | Read alarm data with sub code strings        |
| 38  | 0x30B      | Alarm history reading command (for applying the sub character strings)      | Read alarm history with sub strings          |
| 39  | 0x8A       | Move instruction command (Type Cartesian coordinates)                       | Move robot using Cartesian coordinates       |
| 40  | 0x8B       | Move instruction command (Type Pulse)                                       | Move robot using pulse coordinates           |
| 41  | 0x8C       | 32-byte character type variable (S) reading / writing command               | Read/write 32-byte string variables          |
| 42  | 0x30C      | 32-byte character type variable (S) multiple reading / writing command      | Read/write multiple 32-byte string variables |
| 43  | 0x0411     | Encoder temperature reading command                                         | Read encoder temperature                     |
| 44  | 0x0413     | Converter temperature reading command                                       | Read converter temperature                   |

**Notes**:

- Commands 37-40: Available for system software version FS1.14 or higher
- Commands 41-42: 32-byte S variable compatible commands (16-byte S variable commands are also available)
- Command 43: Available from YBS3.10-00 onwards
- Command 44: Available from YBS4.10-00 onwards

### Representative Command Examples

#### Read Status Information (Command 0x72)

**Request Structure:**

- **Command**: 0x72
- **Instance**: Fixed to 1
- **Attribute**: Specifies which status data to read
  - `1`: Data 1
  - `2`: Data 2
- **Service**:
  - `0x0E` (Get_Attribute_Single): Reads data of a specified element number
  - `0x01` (Get_Attribute_All): Reads data of all element numbers (specify 0 for element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Normal response
  - Other than `0x00`: Abnormal response (error occurred)
- **Added status size**: Size of additional status data
  - `0`: Not specified (no added status data)
  - `1`: 1 WORD (2 bytes) of added status data
  - `2`: 2 WORD (4 bytes) of added status data
- **Added status**: Error code if added status size is 1 or 2
- **Payload**: Status information data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Data 1:**

- `bit0`: Step
- `bit1`: 1 cycle
- `bit2`: Automatic and continuous
- `bit3`: Running
- `bit4`: In-guard safe operation
- `bit5`: Teach
- `bit6`: Play
- `bit7`: Command remote

**Data 2:**

- `bit0`: (Not defined)
- `bit1`: In hold status (by programming pendant)
- `bit2`: In hold status (externally)
- `bit3`: In hold status (by command)
- `bit4`: Alarming
- `bit5`: Error occurring
- `bit6`: Servo ON
- `bit7`: (Not defined)

#### I/O Data Reading / Writing Command (Command 0x78)

**Request Structure:**

- **Command**: 0x78
- **Instance**: Logical number of the I/O data
  - `1 to 128`: Robot user input
  - `1001 to 1128`: Robot user output
  - `2001 to 2127`: External input
  - `2501 to 2628`: Network input
  - `3001 to 3128`: External output
  - `3501 to 3628`: Network output
  - `4001 to 4160`: Robot system input
  - `5001 to 5200`: Robot system output
  - `6001 to 6064`: Interface panel input
  - `7001 to 7999`: Auxiliary relay
  - `8001 to 8064`: Robot control status signal
  - `8201 to 8220`: Pseudo input
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out of all I/O data is enabled
  - `0x10` (Set_Attribute_Single): Only network input signal is writable
- **Payload**: Data exists during writing operation only
  - 32-bit integer (4 bytes): I/O data

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Data exists during reading operation only
  - 32-bit integer (4 bytes): I/O data
  - I/O data exists only when requested by the client

**Note**: For detailed specifications of all commands, refer to the official HSES manual. The above examples show the basic structure and common patterns used in robot control commands.

## File Commands (Division = 0x02)

File commands use a different port (10041) and have a simpler structure.

### Command List

| No. | Command ID | Instance | Attribute | Service | Name                                    | Description                        |
| --- | ---------- | -------- | --------- | ------- | --------------------------------------- | ---------------------------------- |
| 1   | 0x00       | 0x00     | 0x00      | 0x09    | File delete                             | Delete specified file              |
| 2   | -          | -        | -         | 0x15    | File loading command (PC to FS100)      | Upload file from PC to robot       |
| 3   | -          | -        | -         | 0x16    | File saving command (FS100 to PC)       | Download file from robot to PC     |
| 4   | -          | -        | -         | 0x32    | File list acquiring command             | Get list of files on robot         |
| 5   | -          | -        | -         | 0x16    | File saving command (Batch data backup) | Backup batch data from robot to PC |

**Note**: Command 5 is available for system software version FS1.14 or higher.

### Representative Command Examples

#### File Loading Command (Service 0x15)

**Request Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x15 (File loading process)
- **Payload**: Job name to be loaded
  - 32-bit integer format
  - Example: "TEST.JOB" (8 characters, 2 integers)
    - Integer 1: "TEST" (T, E, S, T)
    - Integer 2: "JOB." (J, O, B, .)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is 1
  - 2 WORD error code if added status size is 2
- **Payload**: No data part

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

## Added status codes

### Command Errors (1000s)

- `1010`: Command error
- `1011`: Error in number of command operands
- `1012`: Command operand value range over
- `1013`: Command operand length error
- `1020`: Disk full of files

### Operation Status (2000s)

- `2010`: Manipulator operating
- `2020`: Hold by programming pendant
- `2030`: Hold by playback panel
- `2040`: External hold
- `2050`: Command hold
- `2060`: Error/alarm occurring
- `2070`: Servo OFF
- `2080`: Incorrect mode
- `2090`: File accessing by other function
- `2100`: Command remote not set
- `2110`: This data cannot be accessed
- `2120`: This data cannot be loaded
- `2130`: Editing
- `2150`: Running the coordinate conversion function

### System Requirements (3000s)

- `3010`: Turn ON the servo power
- `3040`: Perform home positioning
- `3050`: Confirm positions
- `3070`: Current value not made
- `3220`: Panel lock; mode/cycle prohibit signal is ON
- `3230`: Panel lock; start prohibit signal is ON
- `3350`: User coordinate is not taught
- `3360`: User coordinate is destroyed
- `3370`: Incorrect control group
- `3380`: Incorrect base axis data
- `3390`: Relative job conversion prohibited (at CVTRJ)
- `3400`: Master job call prohibited (parameter)
- `3410`: Master job call prohibited (lamp ON during operation)
- `3420`: Master job call prohibited (teach lock)
- `3430`: Robot calibration data not defined
- `3450`: Servo power cannot be turned ON
- `3460`: Coordinate system cannot be set

### Memory and File Errors (4000s)

- `4010`: Insufficient memory capacity (job registered memory)
- `4012`: Insufficient memory capacity (position data registered memory)
- `4020`: Job editing prohibited
- `4030`: Same job name exists
- `4040`: No specified job
- `4060`: Set an execution job
- `4120`: Position data is destroyed
- `4130`: Position data not exist
- `4140`: Incorrect position variable type
- `4150`: END instruction for job which is not master job
- `4170`: Instruction data is destroyed
- `4190`: Invalid character in job name
- `4200`: Invalid character in the label name
- `4230`: Invalid instruction in this system
- `4420`: No step in job to be converted
- `4430`: Already converted
- `4480`: Teach user coordinate
- `4490`: Relative job/ independent control function not permitted

### Syntax and Format Errors (5000s)

- `5110`: Syntax error (syntax of instruction)
- `5120`: Position data error
- `5130`: No NOP or END
- `5170`: Format error (incorrect format)
- `5180`: Incorrect number of data
- `5200`: Data range over
- `5310`: Syntax error (except instruction)
- `5340`: Error in pseudo instruction specification
- `5370`: Error in condition file data record
- `5390`: Error in JOB data record
- `5430`: System data not same
- `5480`: Incorrect welding function type

### Device and System Errors (6000s)

- `6010`: The robot/station is under the operation
- `6020`: Not enough memory of the specified device
- `6030`: Cannot be accessed to the specified device
- `6040`: Unexpected auto backup request
- `6050`: CMOS size is over the RAM area
- `6060`: No memory allocation at the power supply on
- `6070`: Accessing error to backup file information
- `6080`: Failed in sorting backup file (Remove)
- `6090`: Failed in sorting backup file (Rename)
- `6100`: Drive name exceeds the specified values
- `6110`: Incorrect device
- `6120`: System error
- `6130`: Auto backup is not available
- `6140`: Cannot be backed up under the auto backup

### Protocol Errors (A000s)

- `A000`: Undefined command
- `A001`: Instance error
- `A002`: Attribute error
- `A100`: Replying data part size error (hardware limit)
- `A101`: Replying data part size error (software limit)

### Data Errors (B000s)

- `B001`: Undefined position variable
- `B002`: Data use prohibited
- `B003`: Requiring data size error
- `B004`: Out of range the data
- `B005`: Data undefined
- `B006`: Specified application unregistered
- `B007`: Specified type unregistered
- `B008`: Control group setting error
- `B009`: Speed setting error
- `B00A`: Operating speed is not setting
- `B00B`: Operation coordinate system setting error
- `B00C`: Type setting error
- `B00D`: Tool No. setting error
- `B00E`: User No. setting error

### System Errors (C000s, D000s, E000s, F000s)

- `C001`: System error (data area setting processing error)
- `C002`: System error (over the replying data area)
- `C003`: System error (size of the data element not same)
- `C800`: System error (customize API processing error)
- `CFFF`: Other error
- `D8FA`: Transmission exclusive error (BUSY or Semaphore error)
- `D8F1`: Processing the another command (BUSY condition)
- `E24F`: Wrong parameter setting for the system backup
- `E250`: System backup file creating error
- `E289`: System error
- `E28A`: System error
- `E28B`: Disconnect the communication due to receive timeout
- `E28C`: Cannot over write the target file
- `E29C`: The requested file does not exist or the file size is "0"
- `E2A0`: The wrong required pass
- `E2A7`: The relevant file is not in the requested file list
- `E2AF`: Receive the deletion request of the file that cannot to delete
- `E2B0`: System error
- `E2B1`: The directory cannot to be deleted
- `E2B2`: Receive the request of the sending/receiving file at the remote OFF state
- `E2B3`: File not found
- `E2B4`: The requested pass is too long
- `E2AA`: System error
- `E444`: Processing the another command (BUSY condition)
- `E49D`: Format error (data size 0)
- `E49E`: Format error (frame size over)
- `E49F`: Format error (frame size 0)
- `E4A1`: Format error (block number error)
- `E4A2`: Format error (ACK error)
- `E4A3`: Format error (processing category error)
- `E4A4`: Format error (access level error)
- `E4A5`: Format error (header size error)
- `E4A6`: Format error (identifier error)
- `E4A7`: Format error (the size of the requested command and received frame are different)
- `E4A8`: System error
- `E4A9`: System error
- `FFF0`: System error
- `FFF2`: System error
- `FFF3`: System error
- `FFF4`: System error
- `FFF5`: System error
- `FFF6`: Too many request and unable to process (BUSY condition)
- `FFF7`: System error
- `FFF8`: System error
- `FFFE`: The remote mode is detected, and disconnect the communication

**Note**: This list of Added Status Codes is based on the official HSES manual. However, error codes may vary depending on the controller model and firmware version. For the most accurate and up-to-date error codes, please refer to the official manual for your specific Yaskawa robot controller.

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
