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

### Command details

#### Alarm Data Reading Command (Command 0x70)

**Request Structure:**

- **Command**: 0x70
- **Instance**: Specifies which alarm to retrieve
  - `1`: The latest alarm
  - `2`: The second alarm from the latest
  - `3`: The third alarm from the latest
  - `4`: The fourth alarm from the latest
  - _Note: Four alarms are displayed on the P.P display at a time. Specify one out of them._
- **Attribute**: Specifies what type of alarm information to retrieve
  - `1`: Alarm code
  - `2`: Alarm data
  - `3`: By alarm type
  - `4`: Alarm occurring time
  - `5`: Alarm character string name
  - _Note: Alarm code means the alarm No. Alarm data means the sub code which supports the alarm contents. There are some cases that the sub code for the occurring alarm would not appear._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Alarm data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Alarm Code (32-bit integer 1):**

- Range: 0x0001 to 0x270F (decimal value: 9999)
- Note: Setting values vary in accordance with the contents of the alarm type. Also, some alarms are not displayed with the sub code. In this case, the value is zero (0x0).

**Alarm Data (32-bit integer 2):**

- Additional data related to the alarm
- Interpretation depends on the Alarm type

**Alarm Type (32-bit integer 3):**

- Specifies the type and format of the Alarm data
- Values:
  - `0`: No alarm
  - `1`: Decimal UNSIGNED SHORT type (display: `[1]`)
  - `2`: UNSIGNED CHAR bit pattern (display: `[0000_0001]`)
  - `3`: User axis type (display: `[SLURBT]`)
  - `4`: Spacial coordinate type (display: `[XYZ]`)
  - `5`: Robot coordinate type (display: `[XYZRxRyRz]`)
  - `6`: Conveyor characteristic file (display: `[123]`)
  - `8`: Control group type (display: `[R1R2S1S2]`) for robot & station
  - `9`: Decimal SHORT type (display: `[-1]`)
  - `10`: UNSIGNED SHORT bit pattern (display: `[0000_0000_0000_0001]`)
  - `11`: Control group type (display: `[R1]`) for robot only
  - `12`: Control group type (display: `[R1S1B1]`) for robot, station and base
  - `20`: Control group LOW/HIGH logical axis (display: `[R1:LOW SLURBT, HIGH SLURBT]`)
  - `21`: Control group MIN/MAX logical axis (display: `[R1: MIN SLURBT, MAX SLURBT]`)
  - `22`: Control group MIN/MAX spacial coordinate (display: `[R1: MIN XYZ, MAX XYZ]`)
  - `23`: Logical axis of both control group 1 and control group 2 (display: `[R1: SLURBT, R2: SLURBT]`)
  - `24`: Logical axis 1 and 2 of the control group (display: `[R1: SLURBT, SLURBT]`)
  - `25`: Logical axis of the control group and UNSIGNED CHAR type (display: `[R1: SLURBT, 1]`)
  - `27`: Control group and UNSIGNED CHAR type (display: `[R1: 1]`)

**Alarm Occurring Time (32-bit integers 4-6):**

- Format: Character strings of 16 letters
- Example: "2011/10/10 15:49"

**Alarm Character String Name (32-bit integers 8-15):**

- Format: Character strings of 32 letters
- Note: It is transmitted in the form of the character strings whose language code was selected by the programming pendant and half- and full-width characters are mixed.

**Important Note**: For the alarm character strings name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant. Use the same language code as the FS100, or the characters corrupt in case the client side does not correspond to its language code.

#### Alarm History Reading Command (Command 0x71)

**Request Structure:**

- **Command**: 0x71
- **Instance**: Specifies the alarm number range
  - `1 to 100`: Major failure alarms
  - `1001 to 1100`: Monitor alarm alarms
  - `2001 to 2100`: User alarm (system) alarms
  - `3001 to 3100`: User alarm (user) alarms
  - `4001 to 4100`: OFF line alarm alarms
- **Attribute**: Specifies which alarm information to retrieve
  - `1`: Alarm code
  - `2`: Alarm data
  - `3`: Alarm type
  - `4`: Alarm occurring time
  - `5`: Alarm character strings name
  - _Note: Alarm code means the alarm No. Alarm data means the sub code which supports the alarm contents. There are some cases that the sub code for the occurring alarm would not appear._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of a specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Alarm history data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Alarm Code (32-bit integer 1):**

- Range: 0x0001 to 0x270F (decimal value: 9999)

**Alarm Data (32-bit integer 2):**

- Additional data related to the alarm
- Setting values vary depending on the specific alarm type
- Some alarms are not displayed with a sub-code; in such cases, the value is 0x0

**Alarm Type (32-bit integer 3):**

- Specifies the type of alarm, which dictates how the alarm data should be interpreted and displayed
- Values:
  - `0`: No alarm
  - `1`: Decimal UNSIGNED SHORT type (display: `[1]`)
  - `2`: UNSIGNED CHAR bit pattern (display: `[0000_0001]`)
  - `3`: User axis type (display: `[SLURBT]`)
  - `4`: Spacial coordinate type (display: `[XYZ]`)
  - `5`: Robot coordinate type (display: `[XYZRxRyRz]`)
  - `6`: Conveyor characteristic file (display: `[123]`)
  - `8`: Control group type (display: `[R1R2S1S2]`) for robot & station
  - `9`: Decimal SHORT type (display: `[-1]`)
  - `10`: UNSIGNED SHORT bit pattern (display: `[0000_0000_0000_0001]`)
  - `11`: Control group type (display: `[R1]`) for robot only
  - `12`: Control group type (display: `[R1S1B1]`) for robot, station and base
  - `20`: Control group LOW/HIGH logical axis (display: `[R1: LOW SLURBT, HIGH SLURBT]`)
  - `21`: Control group MIN/MAX logical axis (display: `[R1: MIN SLURBT, MAX SLURBT]`)
  - `22`: Control group MIN/MAX spacial coordinate (display: `[R1: MIN XYZ, MAX XYZ]`)
  - `23`: Logical axis of both control group 1 and control group 2 (display: `[R1: SLURBT, R2: SLURBT]`)
  - `24`: Logical axis 1 and 2 of the control group (display: `[R1: SLURBT, SLURBT]`)
  - `25`: Logical axis of the control group and UNSIGNED CHAR type (display: `[R1: SLURBT, 1]`)
  - `27`: Control group and UNSIGNED CHAR type (display: `[R1: 1]`)

**Alarm Occurring Time (32-bit integers 4-7):**

- Format: Character strings of 16 letters
- Example: "2011/10/10 15:49"

**Alarm Character String Name (32-bit integers 8-15):**

- Format: Character strings of 32 letters
- Note: It is transmitted in the form of the character strings whose language code was selected by the programming pendant and half- and full-width characters are mixed.

**Important Note**: For the alarm character strings name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant. Use the same language code as the FS100, or the characters corrupt in case the client side does not correspond to its language code.

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

#### Executing Job Information Reading Command (Command 0x73)

**Request Structure:**

- **Command**: 0x73
- **Instance**: Specifies the task type
  - `1`: Master task
  - `2`: Sub task 1
  - `3`: Sub task 2
  - `4`: Sub task 3
  - `5`: Sub task 4
  - `6`: Sub task 5
- **Attribute**: Specifies the job information attribute to read
  - `1`: Job name
  - `2`: Line number
  - `3`: Step number
  - `4`: Speed override value
  - _Note: Specify the status data number of the executing job information._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Job information data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Job Name (32-bit integers 1-8):**

- Format: Character strings of 32 letters
- Half-width character: 32 characters
- Full-width character: 16 characters

**Line Number (32-bit integer 9):**

- Range: 0 to 9999

**Step Number (32-bit integer 10):**

- Range: 1 to 9998

**Speed Override Value (32-bit integer 11):**

- Speed override value (unit: 0.01%)
- Examples:
  - 100% → 10000
  - 75% → 7500

**Important Note**: For the job name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant. Use the same language code as the FS100, or the characters corrupt in case the client side does not correspond to its language code.

#### Axis Configuration Information Reading Command (Command 0x74)

**Request Structure:**

- **Command**: 0x74
- **Instance**: Specifies the control group
  - `1 to 2`: R1 to R2 (Robot pulse value)
  - `11 to 12`: B1 to B2 (Base pulse value)
  - `21 to 23`: S1 to S3 (Station pulse value)
  - `101 to 102`: R1 to R2 (Robot cartesian coordinate)
  - `111 to 112`: B1 to B2 (Base cartesian coordinate)
- **Attribute**: Specifies the data number of axis information
  - `1`: Axis name of the first axis
  - `2`: Axis name of the second axis
  - `3`: Axis name of the third axis
  - `4`: Axis name of the fourth axis
  - `5`: Axis name of the fifth axis
  - `6`: Axis name of the sixth axis
  - `7`: Axis name of the seventh axis
  - `8`: Axis name of the eighth axis
  - _Note: Specify the data number of axis information. Each axis is justified for setting. '0' is set to nonexistent axis._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Axis configuration data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Coordinate Names (32-bit integers 1-8):**

**First coordinate name (32-bit integer 1):**

- "S" (R*: pulse) / "X" (R*/B*: cartesian value) / "1" (B*/S\*: pulse)

**Second coordinate name (32-bit integer 2):**

- "L" (R*: pulse) / "Y" (R*/B*: cartesian value) / "2" (B*/S\*: pulse)

**Third coordinate name (32-bit integer 3):**

- "U" (R*: pulse) / "Z" (R*/B*: cartesian value) / "3" (B*/S\*: pulse)

**Fourth coordinate name (32-bit integer 4):**

- "R" (R*: pulse) / "Rx" (R*: cartesian value) / "4" (B*/S*: pulse)

**Fifth coordinate name (32-bit integer 5):**

- "B" (R*: pulse) / "Ry" (R*: cartesian value) / "5" (B*/S*: pulse)

**Sixth coordinate name (32-bit integer 6):**

- "T" (R*: pulse) / "Rz" (R*: cartesian value) / "6" (B*/S*: pulse)

**Seventh coordinate name (32-bit integer 7):**

- "E" (R*: pulse) / "Rz" (R*: cartesian value) / "7" (B*/S*: pulse)

**Eighth coordinate name (32-bit integer 8):**

- (Not specified in manual)

**Control Group Abbreviations:**

- `*`: Each control group number
- `R`: Robot (R1 to R2)
- `S`: Station (S1 to S3)
- `B`: Base (B1 to B2)

#### Robot Position Data Reading Command (Command 0x75)

**Request Structure:**

- **Command**: 0x75
- **Instance**: Specifies the control group
  - `1 to 2`: R1 to R2 (Robot pulse value)
  - `11 to 12`: B1 to B2 (Base pulse value)
  - `21 to 23`: S1 to S3 (Station pulse value)
  - `101 to 102`: R1 to R2 (Robot cartesian coordinate)
  - _Note: Cartesian value can select the base coordinate only. (It cannot select the robot, user and tool coordinates.)_
- **Attribute**: Specifies the position information data number
  - `1`: Data type (0: pulse value / 16: base coordinate value)
  - `2`: Configuration (refer to "Details of data")
  - `3`: Tool number
  - `4`: User coordinate number
  - `5`: Extended configuration (refer to "Details of data")
  - `6`: First axis data
  - `7`: Second axis data
  - `8`: Third axis data
  - `9`: Fourth axis data
  - `10`: Fifth axis data
  - `11`: Sixth axis data
  - `12`: Seventh axis data
  - `13`: Eighth axis data
  - _Note: Each axis data is output by the same sequence as mentioned in chapter 3.3.5 "Axis Configuration Information Reading Command", and "0" is set to nonexistent axis._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Position data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Data Type (32-bit integer 1):**

- `0`: Pulse value
- `16`: Base coordinate value

**Configuration (32-bit integer 2):**

- 8 bits (bit0 to bit7):
  - `bit0`: 0: Front, 1: Back
  - `bit1`: 0: Upper arm, 1: Lower arm
  - `bit2`: 0: Flip, 1: No flip
  - `bit3`: 0: θR < 180, 1: θR ≥ 180
  - `bit4`: 0: θT < 180, 1: θT ≥ 180
  - `bit5`: 0: θS < 180, 1: θS ≥ 180
  - `bit6`: 0: Redundant front, 1: Redundant back
  - `bit7`: 0: Previous step regarded inverse kinematics specified, 1: Configuration regarded inverse kinematics specified

**Tool Number (32-bit integer 3):**

- Tool number

**User Coordinate Number (32-bit integer 4):**

- User coordinate number

**Extended Configuration (32-bit integer 5):**

- 8 bits (bit0 to bit7):
  - `bit0`: 0: θL < 180, 1: θL ≥ 180
  - `bit1`: 0: θU < 180, 1: θU ≥ 180
  - `bit2`: 0: θB < 180, 1: θB ≥ 180
  - `bit3`: 0: θE < 180, 1: θE ≥ 180
  - `bit4`: 0: θW < 180, 1: θW ≥ 180
  - `bit5`: Reserve
  - `bit6`: Reserve
  - `bit7`: Reserve

**Axis Data (32-bit integers 6-13):**

- First axis data (32-bit integer 6)
- Second axis data (32-bit integer 7)
- Third axis data (32-bit integer 8)
- Fourth axis data (32-bit integer 9)
- Fifth axis data (32-bit integer 10)
- Sixth axis data (32-bit integer 11)
- Seventh axis data (32-bit integer 12)
- Eighth axis data (32-bit integer 13)

**Note**: Please refer "3.9.4 Flip/ No flip" in "FS100 OPERATOR'S MANUAL" prepared for each application.

#### Position Error Reading Command (Command 0x76)

**Request Structure:**

- **Command**: 0x76
- **Instance**: Specifies the control group
  - `1 to 2`: R1 to R2 (Robot axis)
  - `11 to 12`: B1 to B2 (Base axis)
  - `21 to 23`: S1 to S3 (Station axis)
- **Attribute**: Specifies the axis number
  - `1`: First axis data
  - `2`: Second axis data
  - `3`: Third axis data
  - `4`: Fourth axis data
  - `5`: Fifth axis data
  - `6`: Sixth axis data
  - `7`: Seventh axis data
  - `8`: Eighth axis data
  - _Note: Specify the axis number. Each axis data is output by the same sequence as mentioned in chapter 3.3.5 "Axis Configuration Information Reading Command", and "0" is set to nonexistent axis._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Position error data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Axis Error Data (32-bit integers 1-8):**

- First axis data (32-bit integer 1)
- Second axis data (32-bit integer 2)
- Third axis data (32-bit integer 3)
- Fourth axis data (32-bit integer 4)
- Fifth axis data (32-bit integer 5)
- Sixth axis data (32-bit integer 6)
- Seventh axis data (32-bit integer 7)
- Eighth axis data (32-bit integer 8)

**Note**: Position variable data of each axis can be read out.

#### Torque Data Reading Command (Command 0x77)

**Request Structure:**

- **Command**: 0x77
- **Instance**: Specifies the control group
  - `1 to 2`: R1 to R2 (Robot axis)
  - `11 to 12`: B1 to B2 (Base axis)
  - `21 to 23`: S1 to S3 (Station axis)
  - _Note: Specify the control group._
- **Attribute**: Specifies the axis number
  - `1`: First axis data
  - `2`: Second axis data
  - `3`: Third axis data
  - `4`: Fourth axis data
  - `5`: Fifth axis data
  - `6`: Sixth axis data
  - `7`: Seventh axis data
  - `8`: Eighth axis data
  - _Note: Specify the axis number. Each axis data is output by the same sequence as mentioned in chapter 3.3.5 "Axis Configuration Information Reading Command", and "0" is set to nonexistent axis._
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all element numbers (specify 0 to the element number)
  - _Note: Specify the accessing method to the data._
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Torque data (32-bit integers, 4 bytes each)

**Response Data Structure:**

**Axis Torque Data (32-bit integers 1-8):**

- First axis data (32-bit integer 1)
- Second axis data (32-bit integer 2)
- Third axis data (32-bit integer 3)
- Fourth axis data (32-bit integer 4)
- Fifth axis data (32-bit integer 5)
- Sixth axis data (32-bit integer 6)
- Seventh axis data (32-bit integer 7)
- Eighth axis data (32-bit integer 8)

**Note**: Torque data of each axis can be read out.

#### I/O Data Reading / Writing Command (Command 0x78)

**Request Structure:**

- **Command**: 0x78
- **Instance**: Logical number of the I/O data
  - `1 to 512`: Robot user input
  - `1001 to 1512`: Robot user output
  - `2001 to 2128`: External input
  - `2701 to 2956`: Network input
  - `3001 to 3128`: External output
  - `3701 to 3956`: Network output
  - `4001 to 4256`: Robot system input
  - `5001 to 5512`: Robot system output
  - `6001 to 6064`: Interface panel input
  - `7001 to 7999`: Auxiliary relay
  - `8001 to 8512`: Robot control status signal
  - `8701 to 8720`: Pseudo input
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out of all I/O data is enabled
  - `0x10` (Set_Attribute_Single): Only network input signal is writable
- **Payload**: Data exists during reading operation only
  - 1 byte: I/O data (8 bits representing 8 I/O states)
    - Bit 0-7: I/O states (0 = OFF, 1 = ON)
    - Each bit represents one I/O state
  - I/O data exists only when requested by the client

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
  - 1 byte: I/O data (8 bits representing 8 I/O states)
    - Bit 0-7: I/O states (0 = OFF, 1 = ON)
    - Each bit represents one I/O state
  - I/O data exists only when requested by the client

**Note**: For detailed specifications of all commands, refer to the official HSES manual. The above examples show the basic structure and common patterns used in robot control commands.

#### Register Data Reading / Writing Command (Command 0x79)

**Request Structure:**

- **Command**: 0x79
- **Instance**: Register number
  - `0 to 999`: Readable register number
  - `0 to 559`: Writable register number
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out the specified register data
  - `0x10` (Set_Attribute_Single): Register 0 to 559 is writable
- **Payload**: Data exists during writing operation only
  - Byte 0-1: Register Data
  - Data exists during the writing operation only

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
  - Byte 0-1: Register Data
  - Register data exists only when requested by the client

#### Byte Variable (B) Reading / Writing Command (Command 0x7A)

**Request Structure:**

- **Command**: 0x7A
- **Instance**: Variable number
  - `0 to 99`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of the specified element number
  - `0x10` (Set_Attribute_Single): Write the data to the specified variable
  - `0x02` (Set_Attribute_All): Write the data to the specified variable
- **Payload**: Data exists during writing operation only
  - Byte 0: B variable
  - Data exists during the writing operation only

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
  - Byte 0: B variable
  - The data exists only when requested by the client

#### Integer Type Variable (I) Reading / Writing Command (Command 0x7B)

**Request Structure:**

- **Command**: 0x7B
- **Instance**: Variable number
  - `0 to 99`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of the specified element number
  - `0x10` (Set_Attribute_Single): Write the data to the specified variable
  - `0x02` (Set_Attribute_All): Write the data to the specified variable
- **Payload**: Data exists during writing operation only
  - Byte 0-1: I variable
  - Data exists during the writing operation only

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Data exists during writing operation only
  - Byte 0-1: I variable
  - The data exists only when requested by the client

#### Double Precision Integer Type Variable (D) Reading / Writing Command (Command 0x7C)

**Request Structure:**

- **Command**: 0x7C
- **Instance**: Variable number
  - `0 to 99`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of the specified element number
  - `0x10` (Set_Attribute_Single): Write the data to the specified variable
  - `0x02` (Set_Attribute_All): Write the data to the specified variable
- **Payload**: Data exists during writing operation only
  - Byte 0-3: D variable
  - Data exists during the writing operation only

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
  - Byte 0-3: D variable
  - The data exists only when requested by the client

#### Real Type Variable (R) Reading / Writing Command (Command 0x7D)

**Request Structure:**

- **Command**: 0x7D
- **Instance**: Variable number
  - `0 to 99`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of the specified element number
  - `0x10` (Set_Attribute_Single): Write the data to the specified variable
  - `0x02` (Set_Attribute_All): Write the data to the specified variable
- **Payload**: Data exists during writing operation only
  - Byte 0-3: R variable
  - Data exists during the writing operation only

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
  - Byte 0-3: R variable
  - The data exists only when requested by the client

#### Character Type Variable (S) Reading / Writing Command (Command 0x7E)

**Request Structure:**

- **Command**: 0x7E
- **Instance**: Variable number
  - `0 to 99`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Fixed to 1
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of the specified element number
  - `0x10` (Set_Attribute_Single): Write the data to the specified variable
  - `0x02` (Set_Attribute_All): Write the data to the specified variable
- **Payload**: Data exists during writing operation only
  - Byte 0-15: S variable (16-byte string)
  - Data exists during the writing operation only

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
  - Byte 0-15: S variable (16-byte string)
  - The data exists only when requested by the client

#### Robot Position Type Variable (P) Reading / Writing Command (Command 0x7F)

**Request Structure:**

- **Command**: 0x7F
- **Instance**: Variable number
  - `0 to 127`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Axis information data number
  - `1`: Data type
  - `2`: Configuration
  - `3`: Tool number
  - `4`: User coordinate number
  - `5`: Extended configuration
  - `6`: Coordinated data of the first axis
  - `7`: Coordinated data of the second axis
  - `8`: Coordinated data of the third axis
  - `9`: Coordinated data of the fourth axis
  - `10`: Coordinated data of the fifth axis
  - `11`: Coordinated data of the sixth axis
  - `12`: Coordinated data of the seventh axis
  - `13`: Coordinated data of the eighth axis
- **Service**:
  - `0x01` (Get_Attribute_All): Read out data of the specified element number
  - `0x02` (Set_Attribute_All): Write the data to the specified variable
- **Payload**: Data exists during writing operation only
  - 13 × 32-bit integers (52 bytes): Position variable data
    - Integer 1: Data type
      - `0`: Pulse value
      - `16`: Base coordinated value
      - `17`: Robot coordinated value
      - `18`: Tool coordinated value
      - `19`: User coordinated value
    - Integer 2: Configuration (see Details of data)
    - Integer 3: Tool number
    - Integer 4: User coordinate number
    - Integer 5: Extended configuration (see Details of data)
    - Integers 6-13: Coordinate data (1st to 8th axis)

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
  - 13 × 32-bit integers (52 bytes): Position variable data
    - Same structure as request payload
  - The data exists only when requested by the client

**Details of Data:**

**Configuration (8 bits):**

- `bit0`: 0: Front, 1: Back
- `bit1`: 0: Upper arm, 1: Lower arm
- `bit2`: 0: Flip, 1: No flip
- `bit3`: 0: θR < 180, 1: θR ≥ 180
- `bit4`: 0: θT < 180, 1: θT ≥ 180
- `bit5`: 0: θS < 180, 1: θS ≥ 180
- `bit6`: 0: Redundant front, 1: Redundant back
- `bit7`: 0: Previous step regarded inverse kinematics specified, 1: Configuration regarded inverse kinematics specified

**Extended configuration (8 bits):**

- `bit0`: 0: θL < 180, 1: θL ≥ 180
- `bit1`: 0: θU < 180, 1: θU ≥ 180
- `bit2`: 0: θB < 180, 1: θB ≥ 180
- `bit3`: 0: θE < 180, 1: θE ≥ 180
- `bit4`: 0: θW < 180, 1: θW ≥ 180
- `bit5-7`: Reserve

**Note**: For detailed configuration specifications, refer to "3.9.4 Flip/ No flip" in "FS100 OPERATOR'S MANUAL" prepared for each application.

#### Base Position Type Variable (Bp) Reading / Writing Command (Command 0x80)

**Request Structure:**

- **Command**: 0x80
- **Instance**: Variable number
  - `0 to 127`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Axis information data number
  - `1`: Data type
  - `2`: Coordinated data of the first axis
  - `3`: Coordinated data of the second axis
  - `4`: Coordinated data of the third axis
  - `5`: Coordinated data of the fourth axis
  - `6`: Coordinated data of the fifth axis
  - `7`: Coordinated data of the sixth axis
  - `8`: Coordinated data of the seventh axis
  - `9`: Coordinated data of the eighth axis
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out the specified data
  - `0x01` (Get_Attribute_All): Read out the data
  - `0x10` (Set_Attribute_Single): Write a specified data. If it is not an object element, keep the data previous to writing operation
  - `0x02` (Set_Attribute_All): Write the data
- **Payload**: Data exists during writing operation only
  - 9 × 32-bit integers (36 bytes): Base position variable data
    - Integer 1: Data type
      - `0`: Pulse value
      - `16`: Base coordinated value
    - Integers 2-9: Coordinate data (1st to 8th axis)

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
  - 9 × 32-bit integers (36 bytes): Base position variable data
    - Same structure as request payload
  - The data exists only when requested by the client

#### External Axis Type Variable (Ex) Reading / Writing Command (Command 0x81)

**Request Structure:**

- **Command**: 0x81
- **Instance**: Variable number
  - `0 to 127`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variables specified by the parameter when specifying the number
- **Attribute**: Axis information data number
  - `1`: Data type
  - `2`: Coordinated data of the first axis
  - `3`: Coordinated data of the second axis
  - `4`: Coordinated data of the third axis
  - `5`: Coordinated data of the fourth axis
  - `6`: Coordinated data of the fifth axis
  - `7`: Coordinated data of the sixth axis
  - `8`: Coordinated data of the seventh axis
  - `9`: Coordinated data of the eighth axis
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out the specified data
  - `0x01` (Get_Attribute_All): Read out the data
  - `0x10` (Set_Attribute_Single): Write a specified data. If it is not an object element, keep the data previous to writing operation
  - `0x02` (Set_Attribute_All): Write the data
- **Payload**: Data exists during writing operation only
  - 9 × 32-bit integers (36 bytes): External axis variable data
    - Integer 1: Data type
      - `0`: Pulse value
    - Integers 2-9: Coordinate data (1st to 8th axis)

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
  - 9 × 32-bit integers (36 bytes): External axis variable data
    - Same structure as request payload
  - The data exists only when requested by the client

#### Alarm Reset / Error Cancel Command (Command 0x82)

**Request Structure:**

- **Command**: 0x82
- **Instance**: Reset/Cancel type
  - `1`: RESET (Alarm reset)
  - `2`: CANCEL (Error cancel)
- **Attribute**: Fixed to 1
- **Service**:
  - `0x10` (Set_Attribute_Single): Execute the specified request
- **Payload**: Data exists during writing operation only
  - 32-bit integer (4 bytes): Fixed to 1
    - Byte 0: Data 1
    - Byte 1-3: Reserved

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: No data part

#### Hold / Servo On/off Command (Command 0x83)

**Request Structure:**

- **Command**: 0x83
- **Instance**: Type of OFF/ON command
  - `1`: HOLD
  - `2`: Servo ON
  - `3`: HLOCK (Refer to "Details of data" below)
- **Attribute**: Fixed to 1
- **Service**:
  - `0x10` (Set_Attribute_Single): Execute the specified request
- **Payload**: Data exists during writing operation only
  - 32-bit integer (4 bytes): ON/OFF specification
    - `1`: ON
    - `2`: OFF

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: No data part

**Details of Data (HLOCK):**

This data interlocks the P.P (Programming Pendant) and I/O operation system signals. Only the following operations are available while the interlock operation is ON:

- Emergency stop for the programming pendant
- Inputting signals excluding I/O mode switching, external start, external servo ON, cycle switch, inhibit I/O, inhibit PP/PANEL and master calling up

**Note**: HLOCK is invalid while the programming pendant is in edit mode or it is file accessing using other functions.

#### Step / Cycle / Continuous Switching Command (Command 0x84)

**Request Structure:**

- **Command**: 0x84
- **Instance**: Fixed to 2
- **Attribute**: Fixed to 1
- **Service**:
  - `0x10` (Set_Attribute_Single): Execute the specified request
- **Payload**: Data exists during writing operation only
  - 32-bit integer (4 bytes): CYCLE specification
    - `1`: STEP
    - `2`: ONE CYCLE
    - `3`: CONTINUOUS

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: No data part

#### Character String Display Command To The Programming Pendant (Command 0x85)

**Request Structure:**

- **Command**: 0x85
- **Instance**: Fixed to 1
- **Attribute**: Fixed to 1
- **Service**:
  - `0x10` (Set_Attribute_Single): Execute the specified request
- **Payload**: Data exists during writing operation only
  - 32-bit integer (4 bytes): Displaying message
    - Byte 0: Displaying message
    - Details: Set the character strings to be indicated on the programming pendant
      - Half-width character: 30 characters
      - Full-width character: 15 characters

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: No data part

**Note**:

- For the alarm character strings name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant.
- Use the same language code as the FS100, or the characters will be corrupt if the client side does not correspond to its language code.

#### Start-up (Job Start) Command (Command 0x86)

**Request Structure:**

- **Command**: 0x86
- **Instance**: Fixed to 1
- **Attribute**: Fixed to 1
- **Service**:
  - `0x10` (Set_Attribute_Single): Execute the specified request
- **Payload**: Data exists during writing operation only
  - 32-bit integer (4 bytes): Fixed to 1

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: No data part

#### Job Select Command (Command 0x87)

**Request Structure:**

- **Command**: 0x87
- **Instance**: Type of job to select
  - `1`: Set the executing job
  - `10`: Set master job (Task 0)
  - `11`: Set master job (Task 1)
  - `12`: Set master job (Task 2)
  - `13`: Set master job (Task 3)
  - `14`: Set master job (Task 4)
  - `15`: Set master job (Task 5)
- **Attribute**: Fixed to 1
- **Service**: 0x02 (Set_Attribute_All)
- **Payload**:
  - 9 × 32-bit integers (36 bytes): Job selection data
    - Integers 1-8: Job name
      - Half-width character: 32 characters
      - Full-width character: 16 characters
    - Integer 9: Line number (0 to 9999)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: No data part

**Note**:

- For the alarm character strings name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant.
- Use the same language code as the FS100, or the characters will be corrupt if the client side does not correspond to its language code.

#### Management Time Acquiring Command (Command 0x88)

**Request Structure:**

- **Command**: 0x88
- **Instance**: Type of management time to acquire
  - `1`: Control power ON time
  - `10`: Servo power ON time (TOTAL)
  - `11 to 12`: Servo power ON time (R1 to R2)
  - `21 to 23`: Servo power ON time (S1 to S3)
  - `110`: Play back time (TOTAL)
  - `111 to 112`: Play back time (R1 to R2)
  - `121 to 123`: Play back time (S1 to S3)
  - `210`: Motion time (TOTAL)
  - `211 to 212`: Motion time (R1 to R2)
  - `221 to 223`: Motion time (S1 to S3)
  - `301 to 308`: Operation time
- **Attribute**: Type of the management time
  - `1`: Operation start time
  - `2`: Elapse time
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all the element numbers (In this case, specify 0 to the element number)

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
  - 5 × 32-bit integers (20 bytes): Management time data
    - Integer 1: Operation start time (Character string of 16 characters)
      - Example: `2011/10/10 15:49`
    - Integer 5: Elapse time (Character string of 12 characters)
      - Example: `000000:00'00`

#### System Information Acquiring Command (Command 0x89)

**Request Structure:**

- **Command**: 0x89
- **Instance**: System type
  - `11 to 12`: Type information (R1 to R2)
  - `21 to 23`: Type information (S1 to S3)
  - `101`: Application information (User application only)
- **Attribute**: System information type
  - `1`: System software version
  - `2`: Model name / application
  - `3`: Parameter version
- **Service**:
  - `0x0E` (Get_Attribute_Single): Read out data of the specified element number
  - `0x01` (Get_Attribute_All): Read out data of all the element numbers (In this case, specify 0 to the element number)

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
  - 12 × 32-bit integers (48 bytes): System information data
    - Integers 1-6: System software version (Character strings: 24 characters)
      - Example: `FS1.03.00A (JP/US) -00`
      - Same string is returned regardless of instance (11-12, 21-23, or 101)
    - Integers 7-10: Model name / application (Character strings: 16 characters)
      - Model name is returned for R1-R2, NULL characters for S1-S3
      - Application name is returned when application is used
      - Example: `MPP0003-A0*` (Model), `GENERAL` (Application)
    - Integers 11-12: Parameter version (Character strings: 8 characters)
      - R1-R2: Parameter version
      - NULL characters returned for non-existent control groups
      - Example: `12.34`

#### Plural I/O Data Reading / Writing Command (Command 0x300)

**Request Structure:**

- **Command**: 0x300
- **Instance**: Logical number of the I/O data
  - Reads/writes multiple I/O data starting from the specified instance number for the count specified in the payload
  - `1 to 512`: Robot user input
  - `1001 to 1512`: Robot user output
  - `2001 to 2128`: External input
  - `2701 to 2956`: Network input
  - `3001 to 3128`: External output
  - `3701 to 3956`: Network output
  - `4001 to 4256`: Robot system input
  - `5001 to 5512`: Robot system output
  - `6001 to 6064`: Interface panel input
  - `7001 to 7999`: Auxiliary relay
  - `8001 to 8512`: Robot control status signal
  - `8701 to 8720`: Pseudo input
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data (Reads out the fixed size specified by the data part)
  - `0x34`: Write plural data (Writes the fixed size specified by the data part)
    - Note: Only the network input signal can be writable
- **Payload**: Plural I/O data
  - Byte0-3: Number of I/O data (Maximum value: 474, must be specified as a multiple of 2)
  - Byte4: I/O data 1
  - Byte5: I/O data 2
  - ...
  - Byte(3 + Number): I/O data “Number”
  - Note:
    - When reading, only the "Number" field is valid
    - I/O data section is valid only when writing
    - Each I/O data is 1 byte, and the payload contains the number of I/O data specified by the Number field
    - Each I/O data is represented in 8-bit format containing 8 I/O states, where bit value 0 is OFF and 1 is ON

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural I/O data
  - Byte0-3: Number of I/O data (Set to the same value as the Number specified in the request)
  - Byte4: I/O data 1
  - Byte5: I/O data 2
  - ...
  - Byte(3 + Number): I/O data “Number”
  - Note:
    - When writing, only the "Number" field is valid
    - I/O data section is valid only when reading
    - The structure of each I/O data is the same as the I/O data in the request

#### Plural Register Data Reading / Writing Command (Command 0x301)

**Request Structure:**

- **Command**: 0x301
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 999`: Register number (writable registers: 0 to 559)
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data (Reads out the fixed size specified by the data part)
  - `0x34`: Write plural data (Writes the fixed size specified by the data part)
- **Payload**: Plural register data
  - Byte0-3: Number of register data (Maximum value: 237)
  - Byte4-5: Register data 1
  - Byte6-7: Register data 2
  - ...
  - Byte(3 + (Number - 1) * 2 + 1)-Byte(3 + Number * 2): Register data “Number”
  - Note:
    - When reading, only the "Number" field is valid
    - Register data section is valid only when writing
    - Each register data is 2 byte, and the payload contains the number of register data specified by the Number field

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural register data
  - Byte0-3: Number of register data (Set to the same value as the Number specified in the request)
  - Byte4-5: Register data 1
  - Byte6-7: Register data 2
  - ...
  - Byte(3 + (Number - 1) * 2 + 1)-Byte(3 + Number * 2): Register data “Number”
  - Note:
    - When writing, only the "Number" field is valid
    - Register data section is valid only when reading

#### Plural Byte Type Variable (B) Reading / Writing Command (Command 0x302)

**Request Structure:**

- **Command**: 0x302
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 99`: For standard setting
  - Note: Since the extended variable is an optional function, follow the numbers of the variable specified by the parameter
- **Attribute**: Fixed to 0
- **Service**:
  - `0x33`: Read plural data (Reads out the fixed size specified by the data part)
  - `0x34`: Write plural data (Writes the fixed size specified by the data part)
- **Payload**: Plural B variable
  - Byte0-3: Number of B variable data (Maximum value: 474, must be specified as a multiple of 2)
    - **Note**: The actual maximum value is limited by the variable number range
    - **Standard setting**: Maximum 100 variables (limited by Instance range 0-99)
    - **Extended setting**: Maximum 474 variables (requires optional extended variable function)
  - Byte4:   B variable data 1
  - Byte5:   B variable data 2
  - ...
  - Byte(3 + Number): B variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - B variable data section is valid only when writing
    - Each B variable data is 1 byte, and the payload contains the number of B variable data specified by the Number field
    - **Variable range constraint**: start_variable + count - 1 must not exceed the maximum variable number (99 for standard, varies by configuration for extended)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural B variable
  - Byte0-3: Number of B variable data (Set to the same value as the Number specified in the request)
  - Byte4:   B variable data 1
  - Byte5:   B variable data 2
  - ...
  - Byte(3 + Number): B variable data "Number"
  - Note:
    - When writing, only the "Number" field is valid
    - B variable data section is valid only when reading

#### Plural Integer Type Variable (I) Reading / Writing Command (Command 0x303)

**Request Structure:**

- **Command**: 0x303
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 99`: For standard setting
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural I variable
  - Byte0-3: Number of I variable data (Maximum value: 237)
    - **Note**: The actual maximum value is limited by the variable number range
    - **Standard setting**: Maximum 100 variables (limited by Instance range 0-99)
    - **Extended setting**: Maximum 237 variables (requires optional extended variable function)
  - Byte4-5:   I variable data 1
  - Byte6-7:   I variable data 2
  - ...
  - Byte(3 + (Number - 1) * 2 + 1)-Byte(3 + Number * 2): I variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - I variable data section is valid only when writing
    - Each I variable data is 2 byte, and the payload contains the number of I variable data specified by the Number field
    - **Variable range constraint**: start_variable + count - 1 must not exceed the maximum variable number (99 for standard, varies by configuration for extended)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural I variable
  - Byte0-3: Number of I variable data (Set to the same value as the Number specified in the request)
  - Byte4-5:   I variable data 1
  - Byte6-7:   I variable data 2
  - ...
  - Byte(3 + (Number - 1) * 2 + 1)-Byte(3 + Number * 2): I variable data "Number"
  - Note:
    - When writing, only the "Number" field is valid
    - I variable data section is valid only when reading

#### Plural Double Precision Integer Type Variable (D) Reading / Writing Command (Command 0x304)

**Request Structure:**

- **Command**: 0x304
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 99`: For standard setting
  - Note: For extended variables, the numbers specified by the parameter should be followed as it's an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural D variable
  - Byte0-3: Number of D variable data (Maximum value: 118)
    - **Note**: The actual maximum value is limited by the variable number range
    - **Standard setting**: Maximum 100 variables (limited by Instance range 0-99)
    - **Extended setting**: Maximum 118 variables (requires optional extended variable function)
  - Byte4-7:   D variable data 1
  - Byte8-11:  D variable data 2
  - ...
  - Byte(3 + (Number - 1) * 4 + 1)-Byte(3 + Number * 4): D variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - D variable data section is valid only when writing
    - Each D variable data is 4 byte, and the payload contains the number of D variable data specified by the Number field
    - **Variable range constraint**: start_variable + count - 1 must not exceed the maximum variable number (99 for standard, varies by configuration for extended)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural D variable
  - Byte0-3: Number of D variable data (Set to the same value as the Number specified in the request)
  - Byte4-7:   D variable data 1
  - Byte8-11:  D variable data 2
  - ...
  - Byte(3 + (Number - 1) * 4 + 1)-Byte(3 + Number * 4): D variable data "Number"
  - Note:
    - When writing, only the "Number" field is valid
    - D variable data section is valid only when reading

#### Plural Real Type Variable (R) Reading / Writing Command (Command 0x305)

**Request Structure:**

- **Command**: 0x305
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 99`: For standard setting
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural R variable
  - Byte0-3: Number of R variable data (Maximum value: 118)
    - **Note**: The actual maximum value is limited by the variable number range
    - **Standard setting**: Maximum 100 variables (limited by Instance range 0-99)
    - **Extended setting**: Maximum 118 variables (requires optional extended variable function)
  - Byte4-7:   R variable data 1
  - Byte8-11:  R variable data 2
  - ...
  - Byte(3 + (Number - 1) * 4 + 1)-Byte(3 + Number * 4): R variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - R variable data section is valid only when writing
    - Each R variable data is 4 byte, and the payload contains the number of R variable data specified by the Number field
    - **Variable range constraint**: start_variable + count - 1 must not exceed the maximum variable number (99 for standard, varies by configuration for extended)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural R variable
  - Byte0-3: Number of R variable data (Set to the same value as the Number specified in the request)
  - Byte4-7:   R variable data 1
  - Byte8-11:  R variable data 2
  - ...
  - Byte(3 + (Number - 1) * 4 + 1)-Byte(3 + Number * 4): R variable data "Number"
  - Note:
    - When writing, only the "Number" field is valid
    - R variable data section is valid only when reading

#### Plural Character Type Variable (S) Reading / Writing Command (Command 0x306)

**Request Structure:**

- **Command**: 0x306
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 99`: For standard setting
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Plural S variable
  - Byte0-3: Number of S variable data (Maximum value: 29)
  - Byte4-19:   S variable data 1
  - Byte20-35:  S variable data 2
  - ...
  - Byte(3 + (Number - 1) * 16 + 1)-Byte(3 + Number * 16): S variable data "Number"
  - Note:
    - When reading, only the "Number" field is valid
    - S variable data section is valid only when writing
    - Each S variable data is 16 byte, and the payload contains the number of S variable data specified by the Number field
    - **Variable range constraint**: start_variable + count - 1 must not exceed the maximum variable number (99 for standard, varies by configuration for extended)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: No added status
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
- **Payload**: Plural S variable
  - Byte0-3: Number of S variable data (Set to the same value as the Number specified in the request)
  - Byte4-19:   S variable data 1
  - Byte20-35:  S variable data 2
  - ...
  - Byte(3 + (Number - 1) * 16 + 1)-Byte(3 + Number * 16): S variable data "Number"
  - Note:
    - When writing, only the "Number" field is valid
    - S variable data section is valid only when reading

#### Plural Robot Position Type Variable (P) Reading / Writing Command (Command 0x307)

**Request Structure:**

- **Command**: 0x307
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 127`: For standard setting
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Data exists during writing operation only
  - 118 × 32-bit integers (472 bytes): Plural P variable data
    - Integer 1: Number (Maximum: 9)
    - Integers 2-118: P variable data
      - Data type: 0 (Pulse value), 16 (Base coordinated value), 17 (Robot coordinated value), 18 (Tool coordinated value), 19 (User coordinated value)
      - Configuration, Tool number, User coordinate number, Extended configuration
      - First to Eighth coordinate data
      - Variable data part is valid only when writing
      - When reading, only the number of data is valid

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
- **Payload**: Data exists during reading operation only
  - Same structure as request payload
  - P variable data exists only when requested by the client

#### Plural Base Position Type Variable (Bp) Reading / Writing Command (Command 0x308)

**Request Structure:**

- **Command**: 0x308
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 127`: For standard setting
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Data exists during writing operation only
  - 118 × 32-bit integers (472 bytes): Plural Bp variable data
    - Integer 1: Number (Maximum: 13)
    - Integers 2-118: Bp variable data
      - Data type: 0x00 (Pulse value), 0x10 (Base coordinate value)
      - First to Eighth coordinate data
      - Each Bp variable occupies 9 consecutive 32-bit integers (1 for Data type + 8 for coordinates)
      - Variable data part is valid only when writing
      - When reading, only the number of data is valid

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
- **Payload**: Data exists during reading operation only
  - Same structure as request payload
  - Bp variable data exists only when requested by the client

#### Plural External Axis Type Variable (Ex) Reading / Writing Command (Command 0x309)

**Request Structure:**

- **Command**: 0x309
- **Instance**: Variable number (first number with which reading/writing is executed)
  - `0 to 127`: For standard setting
  - Note: Follow the numbers of the variable specified by the parameter since the extended variable is an optional function
- **Attribute**: Fixed to 0
  - Note: Only batch access of all elements is valid
- **Service**:
  - `0x33`: Read plural data
  - `0x34`: Write plural data
- **Payload**: Data exists during writing operation only
  - 118 × 32-bit integers (472 bytes): Plural Ex variable data
    - Integer 1: Number (Maximum: 13)
    - Integers 2-118: Ex variable data
      - Data type: 0 (Pulse value)
      - First to Eighth coordinate data
      - Each Ex variable occupies 9 consecutive 32-bit integers (1 for Data type + 8 for coordinates)
      - Variable data part is valid only when writing
      - When reading, only the number of data is valid

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
- **Payload**: Data exists during reading operation only
  - Same structure as request payload
  - Ex variable data exists only when requested by the client

#### Alarm Data Reading Command (for Applying the Sub Code Character Strings) (Command 0x30A)

**Request Structure:**

- **Command**: 0x30A
- **Instance**: Specifies which alarm to read
  - `1`: The latest alarm
  - `2`: The second alarm from the latest
  - `3`: The third alarm from the latest
  - `4`: The fourth alarm from the latest
  - Note: Up to four alarms are displayed on the P.P display at the same time. The user must specify one of them
- **Attribute**: Specifies the type of alarm information to retrieve
  - `1`: Alarm code (means the alarm No.)
  - `2`: Alarm data (means the sub code which supports the alarm contents; some alarms may not appear as the sub code)
  - `3`: By alarm type
  - `4`: Alarm occurring time
  - `5`: Alarm character string name
  - `6`: Sub code data additional information character strings (numbers of the Servo circuit boards [SV#*] where the alarms occurred; \* denotes number)
  - `7`: Sub code data character strings
  - `8`: Sub code data character strings reverse display information (sets [1] when characters are reverse)
- **Service**:
  - `0x0E`: Get_Attribute_Single (Read out data of the specified element number)
  - `0x01`: Get_Attribute_All (Read out data of all the element number; in this case, specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status code is "1"
  - 2 WORD error code if added status code is "2"
- **Payload**: 67 × 32-bit integers (268 bytes): Alarm data
  - Integer 1: Alarm code (0x0001 to 0x270F, decimal value: 9999)
  - Integer 2: Alarm data (depends on alarm type, 0x0 if no sub-code)
  - Integer 3: Alarm type (0: No alarm, 1: Decimal UNSIGNED SHORT, 2: UNSIGNED CHAR bit pattern, etc.)
  - Integers 4-7: Alarm occurring time (16 letters, e.g., "2011/10/10 15:49")
  - Integers 8-15: Alarm character strings name (32 letters, mixed half- and full-width characters)
  - Integers 16-19: Sub code data additional information character strings (16 letters, e.g., "[SV#1]")
  - Integers 20-43: Sub code data character strings (96 letters)
  - Integers 44-67: Sub code data character strings reverse display information (96 letters, [0] for regular, [1] for reverse)

**Note**: For the alarm character strings name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant. Use the same language code as the FS100, or the characters corrupt in case the client side does not correspond to its language code.

#### Alarm History Reading Command (for Applying the Sub Code Character Strings) (Command 0x30B)

**Request Structure:**

- **Command**: 0x30B
- **Instance**: Specifies which alarm history entry to read
  - `1 to 100`: Major failure
  - `1001 to 1100`: Monitor alarm
  - `2001 to 2100`: User alarm (system)
  - `3001 to 3100`: User alarm (user)
  - `4001 to 4100`: OFF line alarm
  - Note: This field specifies the alarm number within the history
- **Attribute**: Specifies the type of alarm information to retrieve
  - `1`: Alarm code (refers to the alarm number)
  - `2`: Alarm data (refers to the sub code that supports the alarm content; some alarms may not appear as a sub code)
  - `3`: Alarm type
  - `4`: Alarm occurring time
  - `5`: Alarm character strings name
  - `6`: Sub code data additional information character strings (numbers of the Servo circuit boards [SV#*] where the alarms occurred; \* denotes number)
  - `7`: Sub code data character strings
  - `8`: Sub code data character strings reverse display information (sets [1] when characters are reverse)
- **Service**:
  - `0x0E`: Get_Attribute_Single (Read out data of the specified element number)
  - `0x01`: Get_Attribute_All (Read out data of all elements; in this case, specify 0 to the element number)
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: 67 × 32-bit integers (268 bytes): Alarm history data
  - Integer 1: Alarm code (0x0001 to 0x270F, decimal value: 9999)
  - Integer 2: Alarm data (depends on alarm type, 0x0 if no sub-code)
  - Integer 3: Alarm type (0: No alarm, 1: Decimal UNSIGNED SHORT, 2: UNSIGNED CHAR bit pattern, etc.)
  - Integers 4-7: Alarm occurring time (16 letters, e.g., "2011/10/10 15:49")
  - Integers 8-15: Alarm character strings name (32 letters, mixed half- and full-width characters)
  - Integers 16-19: Sub code data additional information character strings (16 letters, e.g., "[SV#1]")
  - Integers 20-43: Sub code data character strings (96 letters)
  - Integers 44-67: Sub code data character strings reverse display information (96 letters, [0] for regular, [1] for reverse)

**Note**: For the alarm character strings name, it is transmitted in the form of the character strings whose language code was selected by the programming pendant. Use the same language code as the FS100, or the characters corrupt in case the client side does not correspond to its language code.

#### Move Instruction Command (Type Cartesian coordinates) (Command 0x8A)

**Request Structure:**

- **Command**: 0x8A
- **Instance**: Specifies the operation type
  - `1`: Link absolute position operation
  - `2`: Straight absolute position operation
  - `3`: Straight increment value operation
  - Note: Specify the operation number from 1 to 3
- **Attribute**: Fixed to 1
  - Note: Specify "1"
- **Service**: 0x02 (Set_Attribute_All)
  - Note: Write the data to the specified coordinate
- **Payload**: 17 × 32-bit integers (68 bytes): Move instruction data
  - Integer 1: Specifying control group (Robot) (1 to 2)
  - Integer 2: Specifying control group (Station) (1 to 3)
  - Integer 3: Specifying the classification in speed
    - `0`: % (Link operation)
    - `1`: V (Cartesian operation)
    - `2`: VR (Cartesian operation)
  - Integer 4: Specifying a speed
    - Link operation: 0.01%
    - Cartesian operation V speed: 0.1 mm/s
    - Cartesian operation VR speed: 0.1 degree/s
  - Integer 5: Specifying the operation coordinate
    - `16`: Base coordinate
    - `17`: Robot coordinate
    - `18`: User coordinate
    - `19`: Tool coordinate
  - Integer 6: X coordinate value (unit: µm)
  - Integer 7: Y coordinate value (unit: µm)
  - Integer 8: Z coordinate value (unit: µm)
  - Integer 9: Tx coordinate value (unit: 0.0001 degree)
  - Integer 10: Ty coordinate value (unit: 0.0001 degree)
  - Integer 11: Tz coordinate value (unit: 0.0001 degree)
  - Integer 12: Reservation
  - Integer 13: Reservation
  - Integer 14: Type (Refer to following data for details)
  - Integer 15: Expanded type
  - Integer 16: Tool No. (0 to 63)
  - Integer 17: User coordinate No. (1 to 63)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: No data part

**Note**: Robot and station cannot be operated simultaneously. If both operations are set at the same time, FS100 returns a control group setting error (0xB008).

#### Move Instruction Command (Type Pulse) (Command 0x8B)

**Request Structure:**

- **Command**: 0x8B
- **Instance**: Specifies the operation number
  - `1`: Link absolute position operation
  - `2`: Straight absolute position operation
- **Attribute**: Fixed to 1
- **Service**: 0x02 (Set_Attribute_All)
  - Note: Write the data to the specified coordinate
- **Payload**: 22 × 32-bit integers (88 bytes): Move instruction data
  - Integer 1: Specifying control group (Robot) (1 to 2)
  - Integer 2: Specifying control group (Station) (1 to 3)
  - Integer 3: Specifying the classification in speed
    - `0`: % (Link operation)
    - `1`: V (Cartesian operation)
    - `2`: VR (Cartesian operation)
  - Integer 4: Specifying a speed
    - Link operation: 0.01%
    - Cartesian operation V speed: 0.1 mm/s
    - Cartesian operation VR speed: 0.1 degree/s
  - Integer 5: Robot 1st axis pulse value
  - Integer 6: Robot 2nd axis pulse value
  - Integer 7: Robot 3rd axis pulse value
  - Integer 8: Robot 4th axis pulse value
  - Integer 9: Robot 5th axis pulse value
  - Integer 10: Robot 6th axis pulse value
  - Integer 11: Robot 7th axis pulse value
  - Integer 12: Robot 8th axis pulse value
  - Integer 13: Tool No. (0 to 63)
  - Integer 14: Base 1st axis position (Pulse value)
  - Integer 15: Base 2nd axis position (Pulse value)
  - Integer 16: Base 3rd axis position (Pulse value)
  - Integer 17: Station 1st axis position (pulse value)
  - Integer 18: Station 2nd axis position (pulse value)
  - Integer 19: Station 3rd axis position (pulse value)
  - Integer 20: Station 4th axis position (pulse value)
  - Integer 21: Station 5th axis position (pulse value)
  - Integer 22: Station 6th axis position (pulse value)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: No data part

**Note**: To move the base axis, specify the robot No. at the specifying control group, and input the each axis value. Robot and station cannot be operated simultaneously. If both operations are set at the same time, FS100 returns a control group setting error (0xB008).

#### 32-byte Character Variable (S) Read/Write Command (Command 0x8C)

**Request Structure:**

- **Command**: 0x8C
- **Instance**: Specifies the variable number (0-99 for standard settings)
  - Note: Variable extension is an optional feature, so follow the number of variables specified by the parameter
- **Attribute**: Fixed to 1
  - Note: Specify "1"
- **Service**: Specifies the data access method
  - `0x0E` (Get_Attribute_Single): Read data from the specified variable
  - `0x01` (Get_Attribute_All): Read data from the specified variable
  - `0x10` (Set_Attribute_Single): Write data to the specified variable
  - `0x02` (Set_Attribute_All): Write data to the specified variable
- **Payload**: 8 × 32-bit integers (32 bytes): S variable data (exists only during writing)
  - Integer 1: S variable (Byte 0)
  - Integer 2: S variable (Byte 1-4)
  - Integer 3: S variable (Byte 5-8)
  - Integer 4: S variable (Byte 9-12)
  - Integer 5: S variable (Byte 13-16)
  - Integer 6: S variable (Byte 17-20)
  - Integer 7: S variable (Byte 21-24)
  - Integer 8: S variable (Byte 25-28)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: 8 × 32-bit integers (32 bytes): S variable data (exists only during reading)
  - Integer 1: S variable (Byte 0)
  - Integer 2: S variable (Byte 1-4)
  - Integer 3: S variable (Byte 5-8)
  - Integer 4: S variable (Byte 9-12)
  - Integer 5: S variable (Byte 13-16)
  - Integer 6: S variable (Byte 17-20)
  - Integer 7: S variable (Byte 21-24)
  - Integer 8: S variable (Byte 25-28)

**Note**: Data part exists only when a read request is made from the client. For writing operations, the data part is included in the request payload.

#### 32-byte Character Variable (S) Multiple Read/Write Command (Command 0x30C)

**Request Structure:**

- **Command**: 0x30C
- **Instance**: Specifies the variable number (0-99 for standard settings)
  - Note: Variable extension is an optional feature, so follow the number of variables specified by the parameter
  - This is the starting number for read/write operations
- **Attribute**: Fixed to 0
  - Note: Must be "0". Only all-element batch access is possible
- **Service**: Specifies the data access method
  - `0x33`: Multiple read
  - `0x34`: Multiple write
- **Payload**: Variable data structure
  - Integer 1: Count (Maximum count is 14)
  - Integer 2-9: S Variable 1 (32 bytes)
  - Integer 10-17: S Variable 2 (32 bytes)
  - Integer 18-25: S Variable 3 (32 bytes)
  - Integer 26-33: S Variable 4 (32 bytes)
  - Integer 34-41: S Variable 5 (32 bytes)
  - Integer 42-49: S Variable 6 (32 bytes)
  - Integer 50-57: S Variable 7 (32 bytes)
  - Integer 58-65: S Variable 8 (32 bytes)
  - Integer 66-73: S Variable 9 (32 bytes)
  - Integer 74-81: S Variable 10 (32 bytes)
  - Integer 82-89: S Variable 11 (32 bytes)
  - Integer 90-97: S Variable 12 (32 bytes)
  - Integer 98-105: S Variable 13 (32 bytes)
  - Integer 106-113: S Variable 14 (32 bytes)

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: Variable data structure (same as request payload)
  - Integer 1: Count (Maximum count is 14)
  - Integer 2-113: S Variable data (up to 14 variables, 32 bytes each)

**Note**: The variable data part is valid only during writing. During reading, only the count data is valid. Each S variable occupies 8 × 32-bit integers (32 bytes).

#### Encoder Temperature Read Command (Command 0x0411)

**Request Structure:**

- **Command**: 0x0411
- **Instance**: Specifies the control group
  - `1-2`: R1-R2... Robot axis
  - `11-12`: B1-B2... Base axis
  - `21-23`: S1-S3... Station axis
- **Attribute**: Fixed to 1
  - Note: Specify "1"
- **Service**: Specifies the data access method
  - `0x0E` (Get_Attribute_Single): Read data for the specified robot axis
  - `0x01` (Get_Attribute_All): Read data for the specified robot axis
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: 8 × 32-bit integers (32 bytes): Encoder temperature data
  - Integer 1: 1st axis encoder temperature data
  - Integer 2: 2nd axis encoder temperature data
  - Integer 3: 3rd axis encoder temperature data
  - Integer 4: 4th axis encoder temperature data
  - Integer 5: 5th axis encoder temperature data
  - Integer 6: 6th axis encoder temperature data
  - Integer 7: 7th axis encoder temperature data
  - Integer 8: 8th axis encoder temperature data

**Note**: This command is available for YBS3.10-00 and later versions. The encoder temperature for each axis is set in the response payload.

#### Converter Temperature Read Command (Command 0x0413)

**Request Structure:**

- **Command**: 0x0413
- **Instance**: Specifies the servo board number
  - `1`: Servo board 1
  - `2`: Servo board 2
- **Attribute**: Fixed to 1
  - Note: Specify "1"
- **Service**: Specifies the data access method
  - `0x0E` (Get_Attribute_Single): Read converter temperature for the specified servo board
  - `0x01` (Get_Attribute_All): Read converter temperature for the specified servo board
- **Payload**: No data part

**Response Structure:**

- **Status**: Command execution result
  - `0x00`: Respond normally
  - Other than `0x00`: Respond abnormally
- **Added status size**: Size of additional status data
  - `0`: Not specified
  - `1`: 1 WORD of added status data
  - `2`: 2 WORD of added status data
- **Added status**: Error code specified by the added status size
  - 1 WORD error code if added status size is "1"
  - 2 WORD error code if added status size is "2"
- **Payload**: 6 × 32-bit integers (24 bytes): Converter temperature data
  - Integer 1: Converter No. 1 data (converter temperature for the specified servo board)
  - Integer 2: Reserved
  - Integer 3: Reserved
  - Integer 4: Reserved
  - Integer 5: Reserved
  - Integer 6: Reserved

**Note**: This command is available for YBS4.10-00 and later versions. The converter temperature corresponding to the specified servo board is set in the response payload.

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

### Command details

#### File Deleting Command (Service 0x09)

**Request Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x09 (File deleting process)
- **Payload**: Job name to be deleted
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

#### File Loading Command (Service 0x15)

The File Loading Command is not a single message but a protocol involving multiple message exchanges.

**Communication Flow:**

1. **Loading Request (Request 1)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x00
   - **Service**: 0x15 (File loading process)
   - **Block No.**: 0x0000_0000
   - **Payload**: File name to be loaded
     - 32-bit integer format
     - Example: "TEST.JOB" (8 characters, 2 integers)
       - Integer 1: "TEST" (T, E, S, T)
       - Integer 2: "JOB." (J, O, B, .)

2. **Request Acknowledgment (ACK to Request)**

   - **Service**: 0x95 (ACK service)
   - **Status**: 0x00 (Normal response)
   - **Added status size**: 0x00
   - **Added status**: 0x0000

3. **File Data Transfer (Data 1, Data 2, ..., Data N)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x00
   - **Service**: 0x15
   - **Block No.**: Increment by 1 from previous block number (1, 2, ..., N)
   - **Payload**: File data blocks
     - Data 1: First data block
     - Data 2-N: Intermediate data blocks
     - Data N: Last data block (Add 0x8000_0000 to the previous block number)

4. **Data Acknowledgment (ACK 1, ACK 2, ..., ACK N)**

   - **Service**: 0x95 (ACK service)
   - **Status**: 0x00 (Normal response)
   - **Added status size**: 0x00
   - **Added status**: 0x0000
   - Individual ACK response for each data block

**Request Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x15 (File loading process)
- **Block No.**: 0x0000_0000 (for initial request)
- **Payload**: File name to be loaded
  - 32-bit integer format
  - Example: "TEST.JOB" (8 characters, 2 integers)
    - Integer 1: "TEST" (T, E, S, T)
    - Integer 2: "JOB." (J, O, B, .)

**Data Transfer Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x15
- **Block No.**: Increment by 1 from previous block number
  - Normal packets: 1, 2, ..., N
  - Last packet: Add 0x8000_0000 to the previous block number
- **Payload**: File data blocks
  - Variable size data blocks
  - Last block identified by adding 0x8000_0000 to the previous block number

**Response Structure:**

- **Service**: 0x95 (ACK service)
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

#### File Saving Command (Service 0x16)

The File Saving Command is not a single message but a protocol involving multiple message exchanges.

**Communication Flow:**

1. **Saving Request (Request 1)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x00
   - **Service**: 0x16 (File saving process)
   - **Block No.**: 0x0000_0000
   - **Payload**: File name to be saved
     - 32-bit integer format
     - Example: "TEST.JOB" (8 characters, 2 integers)
       - Integer 1: "TEST" (T, E, S, T)
       - Integer 2: "JOB." (J, O, B, .)

2. **File Data Transfer (Data 1, Data 2, ..., Data N)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x16
   - **Service**: 0x96
   - **Block No.**: Increment by 1 from previous block number (1, 2, ..., N)
   - **Payload**: File data blocks
     - Data 1: First data block
     - Data 2-N: Intermediate data blocks
     - Data N: Last data block (Add 0x8000_0000 to the previous block number)

3. **Data Acknowledgment (ACK 1, ACK 2, ..., ACK N)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x16
   - **Service**: 0x00
   - **Block No.**: Same as the corresponding data packet's block number
   - Individual ACK response for each data block

**Request Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x16 (File saving process)
- **Block No.**: 0x0000_0000 (for initial request)
- **Payload**: File name to be saved
  - 32-bit integer format
  - Example: "TEST.JOB" (8 characters, 2 integers)
    - Integer 1: "TEST" (T, E, S, T)
    - Integer 2: "JOB." (J, O, B, .)

**Data Transfer Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x16
- **Service**: 0x96
- **Block No.**: Increment by 1 from previous block number
  - Normal packets: 1, 2, ..., N
  - Last packet: Add 0x8000_0000 to the previous block number
- **Payload**: File data blocks
  - Variable size data blocks
  - Last block identified by adding 0x8000_0000 to the previous block number

**Response Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x16
- **Service**: 0x00
- **Block No.**: Same as the corresponding data packet's block number
- **Payload**: No data part

#### File List Acquiring Command (Service 0x32)

The File List Acquiring Command is not a single message but a protocol involving multiple message exchanges.

**Communication Flow:**

1. **File List Request (Request 1)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x00
   - **Service**: 0x32 (File list acquiring process)
   - **Block No.**: 0x0000_0000
   - **Payload**: File type specification
     - 32-bit integer format
     - Example: "\*" (wildcard for all JBI files)
       - Byte 0: "\*"
       - Byte 1: "."
       - Byte 2: "J"
       - Byte 3: "B"
       - Byte 4: "I"

2. **File List Data Transfer (Data 1, Data 2, ..., Data N)**

   - **Service**: 0xB2 (ACK service for 0x32)
   - **Status**: 0x00 (Normal response)
   - **Added status size**: 0x00
   - **Added status**: 0x0000
   - **Block No.**: Increment by 1 from previous block number (1, 2, ..., N)
   - **Payload**: File list data blocks
     - Data 1: First file list block
     - Data 2-N: Intermediate file list blocks
     - Data N: Last file list block (Add 0x8000_0000 to the previous block number)

3. **Data Acknowledgment (ACK 1, ACK 2, ..., ACK N)**

   - **Command**: 0x00
   - **Instance**: 0x00
   - **Attribute**: 0x00
   - **Service**: 0x32
   - **Block No.**: Same as the corresponding data packet's block number
   - Individual ACK response for each data block

**Request Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x32 (File list acquiring process)
- **Block No.**: 0x0000_0000 (for initial request)
- **Payload**: File type specification
  - 32-bit integer format
  - Example: "\*" (wildcard for all JBI files)
    - Byte 0: "\*"
    - Byte 1: "."
    - Byte 2: "J"
    - Byte 3: "B"
    - Byte 4: "I"

**File Type Specifications:**

- No specification: JBI list
- `**`: JBI list
- `*.JBI`: JBI list
- `*.DAT`: DAT file list
- `*.CND`: CND file list
- `*.PRM`: PRM file list
- `*.SYS`: SYS file list
- `*.LST`: LST file list

**Data Transfer Structure:**

- **Service**: 0xB2 (ACK service for 0x32)
- **Status**: 0x00 (Normal response)
- **Added status size**: 0x00
- **Added status**: 0x0000
- **Block No.**: Increment by 1 from previous block number
  - Normal packets: 1, 2, ..., N
  - Last packet: Add 0x8000_0000 to the previous block number
- **Payload**: File list data blocks
  - Variable size data blocks
  - File names terminated by `<CR><LF>`
  - 32-bit integer format for each file name
  - Last block identified by adding 0x8000_0000 to the previous block number

**Data Acknowledgment Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x32
- **Block No.**: Same as the corresponding data packet's block number
- **Payload**: No data part

#### File Saving Command (Batch Data Backup) (Service 0x16)

**Request Structure:**

- **Command**: 0x00
- **Instance**: 0x00
- **Attribute**: 0x00
- **Service**: 0x16 (File saving process)
- **Payload**: File path to be saved
  - 32-bit integer format
  - Example: "/SPDRV/CMOSBK.BIN"
    - Integer 1: "/SPD" (/, S, P, D)
    - Integer 2: "RV/C" (R, V, /, C)
    - Integer 3: "MOSB" (M, O, S, B)
    - Integer 4: "K.BI" (K, ., B, I)
    - Integer 5: "N" (N, padding, padding, padding)

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

**Notes:**

- To set the batch data backup function, set the device as "RAMDISK" in advance
- It takes about ten minutes to finish backing-up the data by using the batch data backup function
- Refer to chapter 2.5 "Setting of a Batch Data Backup Function" for more detail

## Variable Types

### Supported Variable Types

- `B variables`: Byte variables (Command 0x7A)
  - 32-bit integer (4 bytes)
  - Byte 0: B variable value
  - Byte 1-3: Reserved
  - Variable number range: 0 to 99
- `I variables`: Integer type variables (Command 0x7B)
  - 32-bit integer (4 bytes)
  - Byte 0-3: I variable value
  - Variable number range: 0 to 99
- `D variables`: Double precision integer type variables (Command 0x7C)
  - 32-bit integer (4 bytes)
  - Byte 0-3: D variable value
  - Variable number range: 0 to 99
- `R variables`: Real type variables (Command 0x7D)
  - 32-bit floating point (4 bytes)
  - Variable number range: 0 to 99
- `S variables`: Character type variables (Command 0x7E)
  - 32-bit integer (4 bytes) × 4 = 16 bytes total
  - Byte 0-15: S variable data (16-byte string)
  - Variable number range: 0 to 99
- `P variables`: Robot position type variables (Command 0x7F)
  - 13 × 32-bit integers (52 bytes): Position variable data
  - Variable number range: 0 to 127
- `Bp variables`: Base position type variables (Command 0x80)
  - 9 × 32-bit integers (36 bytes): Base position variable data
  - Variable number range: 0 to 127
- `Ex variables`: External axis type variables (Command 0x81)
  - 9 × 32-bit integers (36 bytes): External axis variable data
  - Variable number range: 0 to 127

### Variable Numbering

- Variables are numbered starting from 0
- Instance field contains the variable number
- Different variable types use different commands and have different number ranges
- Extended variables may have different ranges depending on the controller configuration

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
