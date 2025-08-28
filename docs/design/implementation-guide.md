# Implementation Guide

## Overview

This document provides a step-by-step guide for implementing the Rust HSES client library, based on the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet).

## Implementation Phases

### Phase 1: Protocol Layer (moto-hses-proto)

#### Step 1: Basic Types and Enums

```rust
// src/lib.rs
use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Division {
    Robot = 1,
    File = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Service {
    GetSingle = 0x0e,
    SetSingle = 0x10,
    GetAll = 0x01,
    SetAll = 0x02,
    ReadMultiple = 0x33,
    WriteMultiple = 0x34,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Byte = 0,
    Integer = 1,
    Double = 2,
    Real = 3,
    RobotPosition = 4,
    BasePosition = 5,
    StationPosition = 6,
    String = 7,
}

#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub payload_size: u16,
    pub division: Division,
    pub ack: bool,
    pub request_id: u8,
    pub block_number: u32,
    pub command: u16,
    pub instance: u16,
    pub attribute: u8,
    pub service: Service,
}

#[derive(Debug, Clone)]
pub struct ResponseHeader {
    pub payload_size: u16,
    pub division: Division,
    pub ack: bool,
    pub request_id: u8,
    pub block_number: u32,
    pub service: u8,
    pub status: u8,
    pub extra_status: u16,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub position_type: u32,
    pub joint_config: u32,
    pub tool_number: u32,
    pub user_coordinate: u32,
    pub extended_config: u32,
    pub joints: [i32; 8],
}

#[derive(Debug, Clone)]
pub struct CartesianPosition {
    pub position_type: u32,
    pub joint_config: u32,
    pub tool_number: u32,
    pub user_coordinate: u32,
    pub extended_config: u32,
    pub x: i32, // micrometers
    pub y: i32, // micrometers
    pub z: i32, // micrometers
    pub rx: i32, // millidegrees
    pub ry: i32, // millidegrees
    pub rz: i32, // millidegrees
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    #[error("Invalid magic bytes")]
    InvalidMagicBytes,
    #[error("Invalid header size")]
    InvalidHeaderSize,
}
```

#### Step 2: Serialization Implementation

```rust
// src/serializer.rs
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{RequestHeader, ResponseHeader, Division, Service, Position, CartesianPosition, ProtocolError};

pub struct Serializer;

impl Serializer {
    pub fn serialize_request_header(header: &RequestHeader) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();

        // Magic bytes "YERC"
        buf.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        buf.put_u16_le(0x20);

        // Payload size
        buf.put_u16_le(header.payload_size);

        // Reserved magic constant
        buf.put_u8(0x03);

        // Division
        buf.put_u8(header.division as u8);

        // ACK (always false for requests)
        buf.put_u8(0x00);

        // Request ID
        buf.put_u8(header.request_id);

        // Block number
        buf.put_u32_le(header.block_number);

        // Reserved (8 bytes of '9')
        buf.extend_from_slice(b"99999999");

        // Command
        buf.put_u16_le(header.command);

        // Instance
        buf.put_u16_le(header.instance);

        // Attribute
        buf.put_u8(header.attribute);

        // Service
        buf.put_u8(header.service as u8);

        // Padding (2 bytes)
        buf.put_u16_le(0x0000);

        Ok(buf.freeze())
    }

    pub fn serialize_int32_var(index: u8, value: i32) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();
        buf.put_i32_le(value);
        Ok(buf.freeze())
    }

    pub fn serialize_float32_var(index: u8, value: f32) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();
        buf.put_f32_le(value);
        Ok(buf.freeze())
    }

    pub fn serialize_string_var(index: u8, value: &str) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(value.as_bytes());
        Ok(buf.freeze())
    }

    pub fn serialize_position_var(index: u8, position: &Position) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();

        // Position type (pulse = 0)
        buf.put_u32_le(0x00);

        // Joint configuration
        buf.put_u32_le(position.joint_config);

        // Tool number
        buf.put_u32_le(position.tool_number);

        // User coordinate
        buf.put_u32_le(position.user_coordinate);

        // Extended joint configuration
        buf.put_u32_le(position.extended_config);

        // Joint values
        for joint in &position.joints {
            buf.put_i32_le(*joint);
        }

        Ok(buf.freeze())
    }

    pub fn serialize_cartesian_position_var(index: u8, position: &CartesianPosition) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();

        // Position type
        buf.put_u32_le(position.position_type);

        // Joint configuration
        buf.put_u32_le(position.joint_config);

        // Tool number
        buf.put_u32_le(position.tool_number);

        // User coordinate
        buf.put_u32_le(position.user_coordinate);

        // Extended joint configuration
        buf.put_u32_le(position.extended_config);

        // XYZ coordinates (in micrometers)
        buf.put_i32_le(position.x);
        buf.put_i32_le(position.y);
        buf.put_i32_le(position.z);

        // Rotation coordinates (in millidegrees)
        buf.put_i32_le(position.rx);
        buf.put_i32_le(position.ry);
        buf.put_i32_le(position.rz);

        // Padding
        buf.put_u32_le(0x00);
        buf.put_u32_le(0x00);

        Ok(buf.freeze())
    }
}
```

#### Step 3: Deserialization Implementation

```rust
// src/deserializer.rs
use bytes::{Buf, Bytes};
use crate::{RequestHeader, ResponseHeader, Division, Service, Position, CartesianPosition, ProtocolError};

pub struct Deserializer;

impl Deserializer {
    pub fn deserialize_response_header(data: &mut Bytes) -> Result<ResponseHeader, ProtocolError> {
        if data.len() < 32 {
            return Err(ProtocolError::InvalidMessage("Message too short".to_string()));
        }

        // Check magic bytes
        let magic = data.copy_to_bytes(4);
        if magic != b"YERC" {
            return Err(ProtocolError::InvalidMagicBytes);
        }

        // Header size
        let header_size = data.get_u16_le();
        if header_size != 0x20 {
            return Err(ProtocolError::InvalidHeaderSize);
        }

        // Payload size
        let payload_size = data.get_u16_le();

        // Reserved magic constant
        let _reserved = data.get_u8();

        // Division
        let division = match data.get_u8() {
            1 => Division::Robot,
            2 => Division::File,
            _ => return Err(ProtocolError::InvalidMessage("Invalid division".to_string())),
        };

        // ACK
        let ack = data.get_u8() == 1;

        // Request ID
        let request_id = data.get_u8();

        // Block number
        let block_number = data.get_u32_le();

        // Reserved (8 bytes)
        data.advance(8);

        // Service
        let service = data.get_u8();

        // Status
        let status = data.get_u8();

        // Extra status
        data.advance(2); // Skip added status size
        let extra_status = data.get_u16_le();

        // Padding
        data.advance(2);

        Ok(ResponseHeader {
            payload_size,
            division,
            ack,
            request_id,
            block_number,
            service,
            status,
            extra_status,
        })
    }

    pub fn deserialize_int32_var(data: &mut Bytes) -> Result<i32, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::DeserializationError("Insufficient data for Int32".to_string()));
        }
        Ok(data.get_i32_le())
    }

    pub fn deserialize_float32_var(data: &mut Bytes) -> Result<f32, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::DeserializationError("Insufficient data for Float32".to_string()));
        }
        Ok(data.get_f32_le())
    }

    pub fn deserialize_string_var(data: &mut Bytes) -> Result<String, ProtocolError> {
        let string_data = data.to_vec();
        String::from_utf8(string_data)
            .map_err(|e| ProtocolError::DeserializationError(format!("Invalid UTF-8: {}", e)))
    }

    pub fn deserialize_position_var(data: &mut Bytes) -> Result<Position, ProtocolError> {
        if data.len() < 52 {
            return Err(ProtocolError::DeserializationError("Insufficient data for Position".to_string()));
        }

        let position_type = data.get_u32_le();
        let joint_config = data.get_u32_le();
        let tool_number = data.get_u32_le();
        let user_coordinate = data.get_u32_le();
        let extended_config = data.get_u32_le();

        let mut joints = [0i32; 8];
        for i in 0..8 {
            joints[i] = data.get_i32_le();
        }

        Ok(Position {
            position_type,
            joint_config,
            tool_number,
            user_coordinate,
            extended_config,
            joints,
        })
    }
}
```

### Phase 2: Client Layer (moto-hses-client)

#### Step 1: Client Structure

```rust
// src/client.rs
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

use moto_hses_proto::{RequestHeader, ResponseHeader, Division, Service, ProtocolError};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub buffer_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(300),
            retry_count: 3,
            retry_delay: Duration::from_millis(100),
            buffer_size: 8192,
        }
    }
}

pub struct HsesClient {
    socket: Arc<UdpSocket>,
    remote_addr: SocketAddr,
    config: ClientConfig,
    request_id_counter: Arc<Mutex<u8>>,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("Invalid variable: {0}")]
    InvalidVariable(String),
    #[error("System error: {0}")]
    SystemError(String),
}
```

#### Step 2: Client Implementation

```rust
impl HsesClient {
    pub async fn new(addr: &str) -> Result<Self, ClientError> {
        let remote_addr: SocketAddr = addr.parse()
            .map_err(|e| ClientError::SystemError(format!("Invalid address: {}", e)))?;

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(remote_addr).await?;

        Ok(Self {
            socket: Arc::new(socket),
            remote_addr,
            config: ClientConfig::default(),
            request_id_counter: Arc::new(Mutex::new(0)),
        })
    }

    pub async fn with_config(addr: &str, config: ClientConfig) -> Result<Self, ClientError> {
        let mut client = Self::new(addr).await?;
        client.config = config;
        Ok(client)
    }

    async fn next_request_id(&self) -> u8 {
        let mut counter = self.request_id_counter.lock().await;
        *counter = counter.wrapping_add(1);
        *counter
    }

    async fn send_request(&self, header: RequestHeader, payload: Option<Bytes>) -> Result<ResponseHeader, ClientError> {
        let serialized_header = moto_hses_proto::Serializer::serialize_request_header(&header)?;

        let mut message = Vec::new();
        message.extend_from_slice(&serialized_header);
        if let Some(payload_data) = payload {
            message.extend_from_slice(&payload_data);
        }

        // Send with retry logic
        for attempt in 0..=self.config.retry_count {
            match timeout(self.config.timeout, self.socket.send(&message)).await {
                Ok(Ok(_)) => break,
                Ok(Err(e)) => {
                    if attempt == self.config.retry_count {
                        return Err(ClientError::ConnectionError(e));
                    }
                    tokio::time::sleep(self.config.retry_delay).await;
                }
                Err(_) => {
                    if attempt == self.config.retry_count {
                        return Err(ClientError::TimeoutError("Send timeout".to_string()));
                    }
                    tokio::time::sleep(self.config.retry_delay).await;
                }
            }
        }

        // Receive response
        let mut buffer = vec![0u8; self.config.buffer_size];
        match timeout(self.config.timeout, self.socket.recv(&mut buffer)).await {
            Ok(Ok(len)) => {
                let mut data = Bytes::copy_from_slice(&buffer[..len]);
                let response_header = moto_hses_proto::Deserializer::deserialize_response_header(&mut data)?;

                if response_header.request_id != header.request_id {
                    return Err(ClientError::ProtocolError(
                        ProtocolError::InvalidMessage("Request ID mismatch".to_string())
                    ));
                }

                if response_header.status != 0 {
                    return Err(ClientError::ProtocolError(
                        ProtocolError::InvalidMessage(format!("Robot error: status {}", response_header.status))
                    ));
                }

                Ok(response_header)
            }
            Ok(Err(e)) => Err(ClientError::ConnectionError(e)),
            Err(_) => Err(ClientError::TimeoutError("Receive timeout".to_string())),
        }
    }
}
```

#### Step 3: High-Level API Methods

```rust
impl HsesClient {
    pub async fn read_int32_var(&self, var_number: u16) -> Result<i32, ClientError> {
        let request_id = self.next_request_id().await;
        let header = RequestHeader {
            payload_size: 0,
            division: Division::Robot,
            ack: false,
            request_id,
            block_number: 0,
            command: 0x007c, // readwrite_int32_variable
            instance: var_number,
            attribute: 0,
            service: Service::GetSingle,
        };

        let _response = self.send_request(header, None).await?;

        // Parse response payload
        // Implementation depends on response format
        Ok(0) // Placeholder
    }

    pub async fn write_int32_var(&self, var_number: u16, value: i32) -> Result<(), ClientError> {
        let request_id = self.next_request_id().await;
        let payload = moto_hses_proto::Serializer::serialize_int32_var(var_number as u8, value)?;

        let header = RequestHeader {
            payload_size: payload.len() as u16,
            division: Division::Robot,
            ack: false,
            request_id,
            block_number: 0,
            command: 0x007c, // readwrite_int32_variable
            instance: var_number,
            attribute: 0,
            service: Service::SetSingle,
        };

        let _response = self.send_request(header, Some(payload)).await?;
        Ok(())
    }

    pub async fn read_status(&self) -> Result<Status, ClientError> {
        let request_id = self.next_request_id().await;
        let header = RequestHeader {
            payload_size: 0,
            division: Division::Robot,
            ack: false,
            request_id,
            block_number: 0,
            command: 0x0072, // read_status_information
            instance: 1,
            attribute: 0,
            service: Service::GetAll,
        };

        let _response = self.send_request(header, None).await?;

        // Parse status from response payload
        // Implementation depends on status format
        Ok(Status::default())
    }

    pub async fn execute_job(&self, job_number: u16) -> Result<(), ClientError> {
        let request_id = self.next_request_id().await;
        let header = RequestHeader {
            payload_size: 0,
            division: Division::Robot,
            ack: false,
            request_id,
            block_number: 0,
            command: 0x0073, // execute_job_information
            instance: job_number,
            attribute: 0,
            service: Service::SetAll,
        };

        let _response = self.send_request(header, None).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    pub step: bool,
    pub one_cycle: bool,
    pub continuous: bool,
    pub running: bool,
    pub speed_limited: bool,
    pub teach: bool,
    pub play: bool,
    pub remote: bool,
    pub teach_pendant_hold: bool,
    pub external_hold: bool,
    pub command_hold: bool,
    pub alarm: bool,
    pub error: bool,
    pub servo_on: bool,
}
```

### Phase 3: Mock Server (moto-hses-mock)

#### Step 1: Mock Server Structure

```rust
// src/mock_server.rs
use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use std::sync::Arc;

use moto_hses_proto::{RequestHeader, ResponseHeader, Division, Service, ProtocolError};

pub struct MockHsesServer {
    socket: UdpSocket,
    variables: Arc<Mutex<HashMap<u16, i32>>>,
    running: Arc<Mutex<bool>>,
}

impl MockHsesServer {
    pub async fn new(addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            socket,
            variables: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn with_variable(mut self, var_number: u16, value: i32) -> Self {
        let mut variables = self.variables.lock().await;
        variables.insert(var_number, value);
        self
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        let mut running = self.running.lock().await;
        *running = true;
        drop(running);

        let socket = self.socket.try_clone().await?;
        let variables = self.variables.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 8192];

            while *running.lock().await {
                match socket.recv_from(&mut buffer).await {
                    Ok((len, src_addr)) => {
                        let mut data = bytes::Bytes::copy_from_slice(&buffer[..len]);
                        if let Ok(request_header) = moto_hses_proto::Deserializer::deserialize_request_header(&mut data) {
                            if let Ok(response) = Self::handle_request(request_header, data, &variables).await {
                                let serialized = moto_hses_proto::Serializer::serialize_response_header(&response).unwrap();
                                let _ = socket.send_to(&serialized, src_addr).await;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    async fn handle_request(
        request_header: RequestHeader,
        payload: bytes::Bytes,
        variables: &Arc<Mutex<HashMap<u16, i32>>>,
    ) -> Result<ResponseHeader, ProtocolError> {
        match request_header.command {
            0x007c => { // readwrite_int32_variable
                match request_header.service {
                    Service::GetSingle => {
                        let variables = variables.lock().await;
                        if let Some(value) = variables.get(&request_header.instance) {
                            // Create response with variable value
                            Ok(ResponseHeader {
                                payload_size: 4,
                                division: Division::Robot,
                                ack: true,
                                request_id: request_header.request_id,
                                block_number: 0,
                                service: 0x0e,
                                status: 0x00,
                                extra_status: 0x0000,
                            })
                        } else {
                            Ok(ResponseHeader {
                                payload_size: 0,
                                division: Division::Robot,
                                ack: true,
                                request_id: request_header.request_id,
                                block_number: 0,
                                service: 0x0e,
                                status: 0x02, // Invalid instance
                                extra_status: 0x0000,
                            })
                        }
                    }
                    Service::SetSingle => {
                        let mut variables = variables.lock().await;
                        let value = moto_hses_proto::Deserializer::deserialize_int32_var(&mut payload.clone())?;
                        variables.insert(request_header.instance, value);
                        Ok(ResponseHeader {
                            payload_size: 0,
                            division: Division::Robot,
                            ack: true,
                            request_id: request_header.request_id,
                            block_number: 0,
                            service: 0x10,
                            status: 0x00,
                            extra_status: 0x0000,
                        })
                    }
                    _ => Err(ProtocolError::InvalidMessage("Unsupported service".to_string())),
                }
            }
            _ => Err(ProtocolError::InvalidMessage("Unsupported command".to_string())),
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serialization_deserialization() {
        let header = RequestHeader {
            payload_size: 0,
            division: Division::Robot,
            ack: false,
            request_id: 1,
            block_number: 0,
            command: 0x007c,
            instance: 1,
            attribute: 0,
            service: Service::GetSingle,
        };

        let serialized = Serializer::serialize_request_header(&header).unwrap();
        assert_eq!(serialized.len(), 32); // Header size

        let mut data = serialized.clone();
        let deserialized = Deserializer::deserialize_response_header(&mut data).unwrap();
        assert_eq!(deserialized.request_id, 1);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = HsesClient::new("127.0.0.1:10040").await;
        assert!(client.is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_client_mock_server_communication() {
    let server = MockHsesServer::new("127.0.0.1:10041")
        .await
        .unwrap()
        .with_variable(1, 42i32)
        .await;

    server.start().await.unwrap();

    let client = HsesClient::new("127.0.0.1:10041").await.unwrap();
    let value = client.read_int32_var(1).await.unwrap();

    assert_eq!(value, 42);
}
```

## Performance Considerations

1. **Zero-Copy Operations**: Use `Bytes` for efficient memory management
2. **Connection Reuse**: Reuse UDP sockets when possible
3. **Batch Operations**: Implement batch read/write operations
4. **Async I/O**: Use Tokio for non-blocking operations
5. **Memory Pooling**: Implement connection and buffer pooling

## Security Considerations

1. **Input Validation**: Validate all input parameters
2. **Buffer Management**: Prevent buffer overflows
3. **Resource Limits**: Limit concurrent connections
4. **Error Information**: Avoid leaking sensitive information in errors

## Next Steps

1. Implement comprehensive error handling
2. Add logging and metrics
3. Implement connection pooling
4. Add configuration management
5. Create examples and documentation
6. Add performance benchmarks
7. Implement security features
