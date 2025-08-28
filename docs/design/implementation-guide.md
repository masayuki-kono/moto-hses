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
pub enum VariableType {
    Int32 = 0x0001,
    Float32 = 0x0002,
    String = 0x0003,
    Position = 0x0004,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ReadVariable { var_type: VariableType, var_number: u16 },
    WriteVariable { var_type: VariableType, var_number: u16, data: VariableData },
    ExecuteJob { job_number: u16 },
    GetStatus,
}

#[derive(Debug, Clone)]
pub enum VariableData {
    Int32(i32),
    Float32(f32),
    String(String),
    Position(Position),
}

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub message_id: u16,
    pub command_id: u16,
    pub data_length: u16,
    pub data: Bytes,
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}
```

#### Step 2: Serialization Implementation

```rust
// src/serializer.rs
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{Message, Command, VariableType, VariableData, Position, ProtocolError};

pub struct Serializer;

impl Serializer {
    pub fn serialize_message(message: &Message) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();

        // Header (8 bytes)
        buf.put_u16(message.message_id);
        buf.put_u16(message.command_id);
        buf.put_u16(message.data_length);
        buf.put_u16(0); // Reserved

        // Data
        buf.extend_from_slice(&message.data);

        Ok(buf.freeze())
    }

    pub fn serialize_command(command: &Command) -> Result<Bytes, ProtocolError> {
        match command {
            Command::ReadVariable { var_type, var_number } => {
                let mut buf = BytesMut::new();
                buf.put_u16(*var_type as u16);
                buf.put_u16(*var_number);
                Ok(buf.freeze())
            }
            Command::WriteVariable { var_type, var_number, data } => {
                let mut buf = BytesMut::new();
                buf.put_u16(*var_type as u16);
                buf.put_u16(*var_number);
                buf.extend_from_slice(&Self::serialize_variable_data(data)?);
                Ok(buf.freeze())
            }
            Command::ExecuteJob { job_number } => {
                let mut buf = BytesMut::new();
                buf.put_u16(*job_number);
                buf.put_u16(0); // Reserved
                Ok(buf.freeze())
            }
            Command::GetStatus => {
                Ok(BytesMut::new().freeze())
            }
        }
    }

    fn serialize_variable_data(data: &VariableData) -> Result<Bytes, ProtocolError> {
        let mut buf = BytesMut::new();
        match data {
            VariableData::Int32(value) => buf.put_i32(*value),
            VariableData::Float32(value) => buf.put_f32(*value),
            VariableData::String(value) => {
                let bytes = value.as_bytes();
                buf.put_u16(bytes.len() as u16);
                buf.extend_from_slice(bytes);
            }
            VariableData::Position(pos) => {
                buf.put_f32(pos.x);
                buf.put_f32(pos.y);
                buf.put_f32(pos.z);
                buf.put_f32(pos.rx);
                buf.put_f32(pos.ry);
                buf.put_f32(pos.rz);
            }
        }
        Ok(buf.freeze())
    }
}
```

#### Step 3: Deserialization Implementation

```rust
// src/deserializer.rs
use bytes::{Buf, Bytes};
use crate::{Message, Command, VariableType, VariableData, Position, ProtocolError};

pub struct Deserializer;

impl Deserializer {
    pub fn deserialize_message(data: Bytes) -> Result<Message, ProtocolError> {
        if data.len() < 8 {
            return Err(ProtocolError::InvalidMessage("Message too short".to_string()));
        }

        let mut buf = data;
        let message_id = buf.get_u16();
        let command_id = buf.get_u16();
        let data_length = buf.get_u16();
        let _reserved = buf.get_u16();

        let message_data = buf.copy_to_bytes(data_length as usize);

        Ok(Message {
            message_id,
            command_id,
            data_length,
            data: message_data,
        })
    }

    pub fn deserialize_variable_data(var_type: VariableType, data: Bytes) -> Result<VariableData, ProtocolError> {
        let mut buf = data;
        match var_type {
            VariableType::Int32 => {
                if buf.len() < 4 {
                    return Err(ProtocolError::DeserializationError("Insufficient data for Int32".to_string()));
                }
                Ok(VariableData::Int32(buf.get_i32()))
            }
            VariableType::Float32 => {
                if buf.len() < 4 {
                    return Err(ProtocolError::DeserializationError("Insufficient data for Float32".to_string()));
                }
                Ok(VariableData::Float32(buf.get_f32()))
            }
            VariableType::String => {
                if buf.len() < 2 {
                    return Err(ProtocolError::DeserializationError("Insufficient data for String length".to_string()));
                }
                let length = buf.get_u16() as usize;
                if buf.len() < length {
                    return Err(ProtocolError::DeserializationError("Insufficient data for String".to_string()));
                }
                let string_data = buf.copy_to_bytes(length);
                let string = String::from_utf8(string_data.to_vec())
                    .map_err(|e| ProtocolError::DeserializationError(format!("Invalid UTF-8: {}", e)))?;
                Ok(VariableData::String(string))
            }
            VariableType::Position => {
                if buf.len() < 24 {
                    return Err(ProtocolError::DeserializationError("Insufficient data for Position".to_string()));
                }
                Ok(VariableData::Position(Position {
                    x: buf.get_f32(),
                    y: buf.get_f32(),
                    z: buf.get_f32(),
                    rx: buf.get_f32(),
                    ry: buf.get_f32(),
                    rz: buf.get_f32(),
                }))
            }
        }
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

use moto_hses_proto::{Message, Command, VariableType, VariableData, ProtocolError};

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
    message_id_counter: Arc<Mutex<u16>>,
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
            message_id_counter: Arc::new(Mutex::new(0)),
        })
    }

    pub async fn with_config(addr: &str, config: ClientConfig) -> Result<Self, ClientError> {
        let mut client = Self::new(addr).await?;
        client.config = config;
        Ok(client)
    }

    async fn next_message_id(&self) -> u16 {
        let mut counter = self.message_id_counter.lock().await;
        *counter = counter.wrapping_add(1);
        *counter
    }

    async fn send_command(&self, command: Command) -> Result<Message, ClientError> {
        let message_id = self.next_message_id().await;
        let command_id = match &command {
            Command::ReadVariable { .. } => 0x0001,
            Command::WriteVariable { .. } => 0x0002,
            Command::ExecuteJob { .. } => 0x0003,
            Command::GetStatus => 0x0004,
        };

        let data = moto_hses_proto::Serializer::serialize_command(&command)?;
        let data_length = data.len() as u16;

        let message = Message {
            message_id,
            command_id,
            data_length,
            data,
        };

        let serialized = moto_hses_proto::Serializer::serialize_message(&message)?;

        // Send with retry logic
        for attempt in 0..=self.config.retry_count {
            match timeout(self.config.timeout, self.socket.send(&serialized)).await {
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

        Ok(message)
    }

    async fn receive_response(&self, expected_message_id: u16) -> Result<Message, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];

        match timeout(self.config.timeout, self.socket.recv(&mut buffer)).await {
            Ok(Ok(len)) => {
                let data = Bytes::copy_from_slice(&buffer[..len]);
                let message = moto_hses_proto::Deserializer::deserialize_message(data)?;

                if message.message_id != expected_message_id {
                    return Err(ClientError::ProtocolError(
                        ProtocolError::InvalidMessage("Message ID mismatch".to_string())
                    ));
                }

                Ok(message)
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
    pub async fn read_variable<T>(&self, var_number: u16, var_type: VariableType) -> Result<T, ClientError>
    where
        T: From<VariableData>,
    {
        let command = Command::ReadVariable { var_type, var_number };
        let request = self.send_command(command).await?;
        let response = self.receive_response(request.message_id).await?;

        // Parse response data
        let variable_data = moto_hses_proto::Deserializer::deserialize_variable_data(var_type, response.data)?;
        Ok(T::from(variable_data))
    }

    pub async fn write_variable<T>(&self, var_number: u16, value: T) -> Result<(), ClientError>
    where
        T: Into<VariableData>,
    {
        let var_type = match &value.into() {
            VariableData::Int32(_) => VariableType::Int32,
            VariableData::Float32(_) => VariableType::Float32,
            VariableData::String(_) => VariableType::String,
            VariableData::Position(_) => VariableType::Position,
        };

        let command = Command::WriteVariable {
            var_type,
            var_number,
            data: value.into(),
        };

        let request = self.send_command(command).await?;
        let _response = self.receive_response(request.message_id).await?;

        Ok(())
    }

    pub async fn execute_job(&self, job_number: u16) -> Result<(), ClientError> {
        let command = Command::ExecuteJob { job_number };
        let request = self.send_command(command).await?;
        let _response = self.receive_response(request.message_id).await?;

        Ok(())
    }

    pub async fn get_status(&self) -> Result<RobotStatus, ClientError> {
        let command = Command::GetStatus;
        let request = self.send_command(command).await?;
        let response = self.receive_response(request.message_id).await?;

        // Parse status from response data
        // Implementation depends on status format
        Ok(RobotStatus::default())
    }
}

#[derive(Debug, Clone)]
pub struct RobotStatus {
    pub is_ready: bool,
    pub is_error: bool,
    pub error_code: Option<u16>,
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

use moto_hses_proto::{Message, Command, VariableType, VariableData, ProtocolError};

pub struct MockHsesServer {
    socket: UdpSocket,
    variables: Arc<Mutex<HashMap<u16, VariableData>>>,
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

    pub async fn with_variable(mut self, var_number: u16, value: impl Into<VariableData>) -> Self {
        let mut variables = self.variables.lock().await;
        variables.insert(var_number, value.into());
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
                        let data = bytes::Bytes::copy_from_slice(&buffer[..len]);
                        if let Ok(message) = moto_hses_proto::Deserializer::deserialize_message(data) {
                            if let Ok(response) = Self::handle_message(message, &variables).await {
                                let serialized = moto_hses_proto::Serializer::serialize_message(&response).unwrap();
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

    async fn handle_message(
        message: Message,
        variables: &Arc<Mutex<HashMap<u16, VariableData>>>,
    ) -> Result<Message, ProtocolError> {
        // Parse command from message data
        let command = Self::parse_command(message.command_id, message.data)?;

        match command {
            Command::ReadVariable { var_type, var_number } => {
                let variables = variables.lock().await;
                if let Some(value) = variables.get(&var_number) {
                    let data = moto_hses_proto::Serializer::serialize_variable_data(value)?;
                    Ok(Message {
                        message_id: message.message_id,
                        command_id: message.command_id,
                        data_length: data.len() as u16,
                        data,
                    })
                } else {
                    Err(ProtocolError::InvalidMessage("Variable not found".to_string()))
                }
            }
            Command::WriteVariable { var_type, var_number, data } => {
                let mut variables = variables.lock().await;
                variables.insert(var_number, data);
                Ok(Message {
                    message_id: message.message_id,
                    command_id: message.command_id,
                    data_length: 0,
                    data: bytes::Bytes::new(),
                })
            }
            _ => {
                // Handle other commands
                Ok(Message {
                    message_id: message.message_id,
                    command_id: message.command_id,
                    data_length: 0,
                    data: bytes::Bytes::new(),
                })
            }
        }
    }

    fn parse_command(command_id: u16, data: bytes::Bytes) -> Result<Command, ProtocolError> {
        // Implementation to parse command from data
        // This is a simplified version
        match command_id {
            0x0001 => {
                let mut buf = data;
                let var_type = match buf.get_u16() {
                    0x0001 => VariableType::Int32,
                    0x0002 => VariableType::Float32,
                    0x0003 => VariableType::String,
                    0x0004 => VariableType::Position,
                    _ => return Err(ProtocolError::InvalidMessage("Invalid variable type".to_string())),
                };
                let var_number = buf.get_u16();
                Ok(Command::ReadVariable { var_type, var_number })
            }
            _ => Err(ProtocolError::InvalidMessage("Unknown command".to_string())),
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
        let command = Command::ReadVariable {
            var_type: VariableType::Int32,
            var_number: 1,
        };

        let data = Serializer::serialize_command(&command).unwrap();
        let parsed = Deserializer::parse_command(0x0001, data).unwrap();

        assert!(matches!(parsed, Command::ReadVariable { var_type: VariableType::Int32, var_number: 1 }));
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
    let value: i32 = client.read_variable(1, VariableType::Int32).await.unwrap();

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
