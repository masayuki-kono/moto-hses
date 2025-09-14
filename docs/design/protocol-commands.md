# HSES Protocol Command Reference

## Overview

This document provides a comprehensive reference for HSES protocol commands and their corresponding API methods in the Rust client library. For detailed API documentation, refer to the generated Rust documentation.

## Protocol Command Mapping

This API is designed based on the following HSES protocol commands:

### Robot Commands (Division = 0x01)

| API Method                                                        | Command ID | Description                                                                 |
| ----------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------- |
| `read_alarm_data()`                                               | 0x70       | Alarm data reading command                                                  |
| `read_alarm_history()`                                            | 0x71       | Alarm history reading command                                               |
| `read_status()`                                                   | 0x72       | Status information reading command                                          |
| `get_executing_job_info()`                                        | 0x73       | Executing job information reading command                                   |
| `read_axis_config()`                                              | 0x74       | Axis configuration information reading command                              |
| `read_position()`                                                 | 0x75       | Robot position data reading command                                         |
| `read_position_error()`                                           | 0x76       | Position error reading command                                              |
| `read_torque()`                                                   | 0x77       | Torque data reading command                                                 |
| `read_io()`, `write_io()`                                         | 0x78       | I/O data reading / writing command                                          |
| `read_register()`, `write_register()`                             | 0x79       | Register data reading / writing command                                     |
| `read_variable<u8>()`, `write_variable()`                         | 0x7A       | Byte variable (B) reading / writing command                                 |
| `read_variable<i16>()`, `write_variable()`                        | 0x7B       | Integer type variable (I) reading / writing command                         |
| `read_variable<i32>()`, `write_variable()`                        | 0x7C       | Double precision integer type variable (D) reading / writing command        |
| `read_variable<f32>()`, `write_variable()`                        | 0x7D       | Real type variable (R) reading / writing command                            |
| `read_variable<String>()`, `write_variable()`                     | 0x7E       | Character type variable (S) reading / writing command                       |
| `read_variable<Position>()`, `write_variable()`                   | 0x7F       | Robot position type variable (P) reading / writing command                  |
| `read_variable<BasePosition>()`, `write_variable()`               | 0x80       | Base position type variable (BP) reading / writing command                  |
| `read_variable<ExternalAxis>()`, `write_variable()`               | 0x81       | External axis type variable (EX) reading / writing command                  |
| `reset_alarm()`                                                   | 0x82       | Alarm reset / error cancel command                                          |
| `set_hold()`, `set_servo()`                                       | 0x83       | HOLD / servo ON/OFF command                                                 |
| `set_execution_mode()`                                            | 0x84       | Step / cycle / continuous switching command                                 |
| `display_message()`                                               | 0x85       | Character string display command to the programming pendant                 |
| `start_job()`                                                     | 0x86       | Start-up (job START) command                                                |
| `select_job()`                                                    | 0x87       | Job select command                                                          |
| `get_management_time()`                                           | 0x88       | Management time acquiring command                                           |
| `get_system_info()`                                               | 0x89       | System information acquiring command                                        |
| `read_multiple_io()`                                              | 0x300      | Plural I/O data reading / writing command                                   |
| `read_multiple_registers()`                                       | 0x301      | Plural register data reading / writing command                              |
| `read_multiple_variables<u8>()`                                   | 0x302      | Plural byte type variable (B) reading / writing command                     |
| `read_multiple_variables<i16>()`                                  | 0x303      | Plural integer type variable (I) reading / writing command                  |
| `read_multiple_variables<i32>()`                                  | 0x304      | Plural double precision integer type variable (D) reading / writing command |
| `read_multiple_variables<f32>()`                                  | 0x305      | Plural real type variable (R) reading / writing command                     |
| `read_multiple_variables<String>()`                               | 0x306      | Plural character type variable (S) reading / writing command                |
| `read_multiple_variables<Position>()`                             | 0x307      | Plural robot position type variable (P) reading / writing command           |
| `read_multiple_variables<BasePosition>()`                         | 0x308      | Plural base position type variable (BP) reading / writing command           |
| `read_multiple_variables<ExternalAxis>()`                         | 0x309      | Plural external axis type variable (EX) reading / writing command           |
| `read_alarm_data_with_subcode()`                                  | 0x30A      | Alarm data reading command (for applying the sub code character strings)    |
| `read_alarm_history_with_subcode()`                               | 0x30B      | Alarm history reading command (for applying the sub character strings)      |
| `read_multiple_32byte_strings()`                                  | 0x30C      | 32-byte character type variable (S) multiple reading / writing command      |
| `move_cartesian()`                                                | 0x8A       | Move instruction command (Type Cartesian coordinates)                       |
| `move_pulse()`                                                    | 0x8B       | Move instruction command (Type Pulse)                                       |
| `read_32byte_string_variable()`, `write_32byte_string_variable()` | 0x8C       | 32-byte character type variable (S) reading / writing command               |
| `read_encoder_temperature()`                                      | 0x0411     | Encoder temperature reading command                                         |
| `read_converter_temperature()`                                    | 0x0413     | Converter temperature reading command                                       |

### File Commands (Division = 0x02)

| API Method         | Service | Description                        |
| ------------------ | ------- | ---------------------------------- |
| `delete_file()`    | 0x09    | File delete                        |
| `write_file()`     | 0x15    | File loading command (PC to FS100) |
| `read_file()`      | 0x16    | File saving command (FS100 to PC)  |
| `read_file_list()` | 0x32    | File list acquiring command        |
