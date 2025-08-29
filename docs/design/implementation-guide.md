# Implementation Guide

## Overview

This document provides a step-by-step guide for implementing the Rust HSES client library, based on the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet) and incorporating design insights from the Python implementation [fs100](https://github.com/fih-mobile/fs100).

## Implementation Phases

### Phase 1: Protocol Layer (moto-hses-proto)

#### Step 1: Core Types and Traits

```rust
// src/lib.rs
use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;
use std::marker::PhantomData;

// Core traits for type-safe commands
pub trait Command {
    type Response;
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
}

pub trait VariableType: Send + Sync + 'static {
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError>;
}

// Variable type definitions incorporating design insights from Python implementation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableType {
    Io = 0x78,           // 1 byte
    Register = 0x79,     // 2 bytes
    Byte = 0x7a,         // 1 byte
    Integer = 0x7b,      // 2 bytes
    Double = 0x7c,       // 4 bytes
    Real = 0x7d,         // 4 bytes
    String = 0x7e,       // 16 bytes
    RobotPosition = 0x7f, // Position data
    BasePosition = 0x80,  // Base position data
    ExternalAxis = 0x81,  // External axis data
}

// Variable object for type-safe operations
#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub var_type: VariableType,
    pub index: u8,
    pub value: T,
}

impl<T> Variable<T> {
    pub fn new(var_type: VariableType, index: u8, value: T) -> Self {
        Self { var_type, index, value }
    }

    pub fn with_default(var_type: VariableType, index: u8) -> Self
    where T: Default {
        Self { var_type, index, value: T::default() }
    }
}

// Basic enums
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Division {
    Robot = 1,
    File = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Service {
    GetSingle = 0x0e,
    SetSingle = 0x10,
    GetAll = 0x01,
    SetAll = 0x02,
    ReadMultiple = 0x33,
    WriteMultiple = 0x34,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    Base,
    Robot,
    Tool,
    User(u8), // User1-User16
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystemType {
    RobotPulse = 0,
    BasePulse = 1,
    StationPulse = 3,
    RobotCartesian = 4,
}

// Pose configuration with bit flags
#[derive(Debug, Clone, Copy)]
pub struct PoseConfiguration {
    pub no_flip: bool,
    pub lower_arm: bool,
    pub back: bool,
    pub high_r: bool,
    pub high_t: bool,
    pub high_s: bool,
}

impl Default for PoseConfiguration {
    fn default() -> Self {
        Self {
            no_flip: true,
            lower_arm: false,
            back: false,
            high_r: false,
            high_t: false,
            high_s: false,
        }
    }
}

impl From<u8> for PoseConfiguration {
    fn from(value: u8) -> Self {
        Self {
            no_flip: (value & 0x01) != 0,
            lower_arm: (value & 0x02) != 0,
            back: (value & 0x04) != 0,
            high_r: (value & 0x08) != 0,
            high_t: (value & 0x10) != 0,
            high_s: (value & 0x20) != 0,
        }
    }
}
```

#### Step 2: Enhanced Command System

```rust
// Type-safe command definitions with improved variable support
pub struct ReadVar<T> {
    pub index: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVar<T> {
    pub index: u8,
    pub value: T,
}

// Multiple variable operations incorporating efficient design patterns
pub struct ReadVars<T> {
    pub start_index: u8,
    pub count: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVars<T> {
    pub start_index: u8,
    pub values: Vec<T>,
}

// Status and position operations
pub struct ReadStatus;
pub struct ReadCurrentPosition {
    pub control_group: u8,
    pub coordinate_system: CoordinateSystemType,
}

pub struct ReadPosition {
    pub robot_no: u8,
}

pub struct ReadPositionError {
    pub robot_no: u8,
}

pub struct ReadTorque {
    pub robot_no: u8,
}

// File operations
pub struct ReadFileList {
    pub extension: String,
}

pub struct ReadFile {
    pub filename: String,
}

pub struct WriteFile {
    pub filename: String,
    pub data: Vec<u8>,
}

pub struct DeleteFile {
    pub filename: String,
}

// System information
pub struct ReadSystemInfo {
    pub system_type: u8,
    pub info_type: u8,
}

pub struct ReadManagementTime {
    pub time_type: u8,
}

// Power and control operations
pub struct SwitchPower {
    pub power_type: u8,
    pub switch_action: u8,
}

pub struct SelectCycle {
    pub cycle_type: u8,
}

pub struct Move {
    pub move_type: u8,
    pub coordinate: u8,
    pub speed_class: u8,
    pub speed: f32,
    pub position: Position,
    pub form: u8,
    pub extended_form: u8,
    pub robot_no: u8,
    pub station_no: u8,
    pub tool_no: u8,
    pub user_coor_no: u8,
}
```

#### Step 3: Enhanced Error Handling

```rust
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("buffer underflow")]
    Underflow,
    #[error("invalid header")]
    InvalidHeader,
    #[error("unknown command 0x{0:04X}")]
    UnknownCommand(u16),
    #[error("unsupported operation")]
    Unsupported,
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("deserialization error: {0}")]
    Deserialization(String),
    #[error("invalid variable type")]
    InvalidVariableType,
    #[error("invalid coordinate system")]
    InvalidCoordinateSystem,
    #[error("position data error: {0}")]
    PositionError(String),
    #[error("file operation error: {0}")]
    FileError(String),
    #[error("system info error: {0}")]
    SystemInfoError(String),
}

// Enhanced status structure
#[derive(Debug, Clone)]
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

impl Status {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 2 {
            return Err(ProtocolError::Underflow);
        }

        let status_word = u16::from_le_bytes([data[0], data[1]]);

        Ok(Self {
            step: (status_word & 0x0001) != 0,
            one_cycle: (status_word & 0x0002) != 0,
            continuous: (status_word & 0x0004) != 0,
            running: (status_word & 0x0008) != 0,
            speed_limited: (status_word & 0x0010) != 0,
            teach: (status_word & 0x0020) != 0,
            play: (status_word & 0x0040) != 0,
            remote: (status_word & 0x0080) != 0,
            teach_pendant_hold: (status_word & 0x0100) != 0,
            external_hold: (status_word & 0x0200) != 0,
            command_hold: (status_word & 0x0400) != 0,
            alarm: (status_word & 0x0800) != 0,
            error: (status_word & 0x1000) != 0,
            servo_on: (status_word & 0x2000) != 0,
        })
    }

    // Convenience methods
    pub fn is_running(&self) -> bool { self.running }
    pub fn is_servo_on(&self) -> bool { self.servo_on }
    pub fn has_alarm(&self) -> bool { self.alarm }
    pub fn is_teach_mode(&self) -> bool { self.teach }
    pub fn is_play_mode(&self) -> bool { self.play }
    pub fn is_remote_mode(&self) -> bool { self.remote }
}
```

#### Step 4: Efficient Variable Operations

The library incorporates efficient design patterns for multiple variable operations, including:

1. **Consecutive Variable Grouping**: Automatically groups consecutive variables of the same type for efficient batch operations
2. **Plural Commands**: Uses HSE plural commands (0x33) for reading multiple variables in a single network call
3. **Automatic Optimization**: Handles padding requirements for 1-byte variable types

```rust
// Implementation of efficient variable operations
impl HsesClient {
    // Groups consecutive variables for optimal batch operations
    fn group_consecutive_variables<T>(vars: &[Variable<T>]) -> Vec<Vec<&Variable<T>>> {
        // Implementation that groups consecutive variables for efficient batch operations
        // Similar to Python implementation's _group_nums method
    }

    // Reads consecutive variables using plural command (0x33)
    async fn read_consecutive_variables<T>(&mut self, vars: &mut [Variable<T>]) -> Result<(), ClientError>
    where T: VariableType {
        // Uses plural command (0x33) for efficient batch reading
        // Automatically handles padding for 1-byte variable types
        // Similar to Python implementation's _read_consecutive_variables method
    }

    // Public API for reading multiple variables with automatic optimization
    pub async fn read_variables<T>(&mut self, vars: &mut [Variable<T>]) -> Result<(), ClientError>
    where T: VariableType {
        // Automatically groups variables and uses optimal reading strategy
        // Similar to Python implementation's read_variables method
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
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use thiserror::Error;

use moto_hses_proto::{
    Command, VariableType, Position, Status, ReadStatus, ReadCurrentPosition,
    ReadVar, WriteVar, CoordinateSystemType, ProtocolError
};

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

struct InnerClient {
    socket: UdpSocket,
    remote_addr: SocketAddr,
    request_id: AtomicU8,
    pending_requests: Arc<Mutex<HashMap<u8, PendingRequest>>>,
}

struct PendingRequest {
    start_time: std::time::Instant,
    on_reply: Box<dyn FnOnce(Result<Vec<u8>, ClientError>) + Send>,
}

pub struct HsesClient {
    inner: Arc<InnerClient>,
    config: ClientConfig,
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
    pub async fn connect(addr: &str) -> Result<Self, ClientError> {
        Self::connect_with_config(addr, ClientConfig::default()).await
    }

    pub async fn connect_with_config(addr: &str, config: ClientConfig) -> Result<Self, ClientError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let remote_addr: SocketAddr = addr.parse()
            .map_err(|e| ClientError::SystemError(format!("Invalid address: {}", e)))?;
        socket.connect(remote_addr).await?;

        Ok(Self {
            inner: Arc::new(InnerClient {
                socket,
                remote_addr,
                request_id: AtomicU8::new(1),
                pending_requests: Arc::new(Mutex::new(HashMap::new())),
            }),
            config,
        })
    }

    // High-level API methods
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        let command = ReadVar::<T> { index, _phantom: PhantomData };
        self.send_command(command).await
    }

    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: VariableType,
    {
        let command = WriteVar::<T> { index, value };
        self.send_command(command).await
    }

    pub async fn read_status(&self) -> Result<Status, ClientError> {
        self.send_command(ReadStatus).await
    }

    pub async fn read_position(&self, control_group: u8, coord_system: CoordinateSystemType) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition { control_group, coordinate_system: coord_system };
        self.send_command(command).await
    }

    // Generic command sending
    async fn send_command<C: Command>(&self, command: C) -> Result<C::Response, ClientError> {
        let request_id = self.inner.request_id.fetch_add(1, Ordering::Relaxed);
        let payload = command.serialize()?;

        // Create and send message
        let message = self.create_message(command.command_id(), request_id, payload)?;
        self.inner.socket.send(&message).await?;

        // Wait for response
        let response = self.wait_for_response(request_id).await?;

        // Deserialize response
        self.deserialize_response::<C::Response>(&response)
    }

    fn create_message(&self, command: u16, request_id: u8, payload: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        let mut message = Vec::new();

        // Magic bytes "YERC"
        message.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        message.extend_from_slice(&0x20u16.to_le_bytes());

        // Payload size
        message.extend_from_slice(&(payload.len() as u16).to_le_bytes());

        // Reserved magic constant
        message.push(0x03);

        // Division (Robot)
        message.push(0x01);

        // ACK (Request)
        message.push(0x00);

        // Request ID
        message.push(request_id);

        // Block number (0 for requests)
        message.extend_from_slice(&0u32.to_le_bytes());

        // Reserved (8 bytes of '9')
        message.extend_from_slice(b"99999999");

        // Command
        message.extend_from_slice(&command.to_le_bytes());

        // Instance (0 for most commands)
        message.extend_from_slice(&0u16.to_le_bytes());

        // Attribute (1 for most commands)
        message.push(1);

        // Service (Get_Attribute_All for reads, Set_Attribute_All for writes)
        message.push(0x01);

        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        message.extend(payload);

        Ok(message)
    }

    async fn wait_for_response(&self, request_id: u8) -> Result<Vec<u8>, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];

        loop {
            let (len, _addr) = timeout(self.config.timeout, self.inner.socket.recv_from(&mut buffer)).await
                .map_err(|_| ClientError::TimeoutError("Response timeout".to_string()))??;

            let response_data = &buffer[..len];

            // Parse response header
            if response_data.len() < 32 {
                continue;
            }

            let response_request_id = response_data[18];
            if response_request_id != request_id {
                continue;
            }

            let status = response_data[33];
            if status != 0 {
                return Err(ClientError::ProtocolError(ProtocolError::InvalidMessage(
                    format!("Robot returned error status: {}", status)
                )));
            }

            // Extract payload
            let payload_size = u16::from_le_bytes([response_data[6], response_data[7]]) as usize;
            let payload = response_data[32..32+payload_size].to_vec();

            return Ok(payload);
        }
    }

    fn deserialize_response<T>(&self, data: &[u8]) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        T::deserialize(data).map_err(ClientError::from)
    }
}
```

### Phase 3: Mock Server (moto-hses-mock)

#### Step 1: Mock Server Structure

```rust
// src/mock_server.rs
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::net::SocketAddr;

use moto_hses_proto::{Position, Status, VariableType};

pub struct MockHsesServer {
    variables: Arc<RwLock<HashMap<u8, Box<dyn std::any::Any + Send + Sync>>>>,
    positions: Arc<RwLock<HashMap<u8, Position>>>,
    status: Arc<RwLock<Status>>,
    handlers: Arc<RwLock<HashMap<u16, Box<dyn RequestHandler>>>>,
}

pub trait RequestHandler: Send + Sync {
    fn handle(&self, request: &[u8]) -> Result<Vec<u8>, MockError>;
}

#[derive(Debug, thiserror::Error)]
pub enum MockError {
    #[error("Handler not found for command: {0}")]
    HandlerNotFound(u16),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Server error: {0}")]
    ServerError(String),
}

impl MockHsesServer {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(Status {
                step: false,
                one_cycle: false,
                continuous: false,
                running: false,
                speed_limited: false,
                teach: false,
                play: false,
                remote: false,
                teach_pendant_hold: false,
                external_hold: false,
                command_hold: false,
                alarm: false,
                error: false,
                servo_on: true,
            })),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_variable<T: VariableType + 'static>(mut self, index: u8, value: T) -> Self {
        self.variables.write().unwrap().insert(index, Box::new(value));
        self
    }

    pub fn with_position(mut self, index: u8, position: Position) -> Self {
        self.positions.write().unwrap().insert(index, position);
        self
    }

    pub fn with_custom_handler<F>(mut self, command_id: u16, handler: F) -> Self
    where
        F: RequestHandler + 'static,
    {
        self.handlers.write().unwrap().insert(command_id, Box::new(handler));
        self
    }

    pub async fn start(self, addr: &str) -> Result<(), MockError> {
        let socket = UdpSocket::bind(addr).await
            .map_err(|e| MockError::ServerError(format!("Failed to bind: {}", e)))?;

        let server = Arc::new(self);

        loop {
            let mut buffer = vec![0u8; 1024];
            let (len, addr) = socket.recv_from(&mut buffer).await
                .map_err(|e| MockError::ServerError(format!("Failed to receive: {}", e)))?;

            let request = &buffer[..len];
            let server_clone = server.clone();

            tokio::spawn(async move {
                if let Err(e) = server_clone.handle_request(request, addr, &socket).await {
                    eprintln!("Error handling request: {}", e);
                }
            });
        }
    }

    async fn handle_request(&self, request: &[u8], addr: SocketAddr, socket: &UdpSocket) -> Result<(), MockError> {
        if request.len() < 32 {
            return Err(MockError::InvalidResponse("Request too short".to_string()));
        }

        let command = u16::from_le_bytes([request[30], request[31]]);
        let request_id = request[18];

        // Find handler
        let handler = {
            let handlers = self.handlers.read().unwrap();
            handlers.get(&command).cloned()
        };

        let response_payload = if let Some(handler) = handler {
            handler.handle(request)?
        } else {
            // Default handlers
            match command {
                0x7A..=0x7F => self.handle_variable_command(request)?,
                0x72 => self.handle_read_status()?,
                0x75 => self.handle_read_position(request)?,
                _ => return Err(MockError::HandlerNotFound(command)),
            }
        };

        // Send response
        let response = self.create_response(request_id, response_payload)?;
        socket.send_to(&response, addr).await
            .map_err(|e| MockError::ServerError(format!("Failed to send response: {}", e)))?;

        Ok(())
    }

    fn handle_variable_command(&self, request: &[u8]) -> Result<Vec<u8>, MockError> {
        // Implementation for variable read/write commands
        // This would parse the request and return appropriate response
        Ok(Vec::new())
    }

    fn handle_read_status(&self) -> Result<Vec<u8>, MockError> {
        // Implementation for status reading
        Ok(Vec::new())
    }

    fn handle_read_position(&self, request: &[u8]) -> Result<Vec<u8>, MockError> {
        // Implementation for position reading
        Ok(Vec::new())
    }

    fn create_response(&self, request_id: u8, payload: Vec<u8>) -> Result<Vec<u8>, MockError> {
        let mut response = Vec::new();

        // Magic bytes "YERC"
        response.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        response.extend_from_slice(&0x20u16.to_le_bytes());

        // Payload size
        response.extend_from_slice(&(payload.len() as u16).to_le_bytes());

        // Reserved magic constant
        response.push(0x03);

        // Division (Robot)
        response.push(0x01);

        // ACK (Response)
        response.push(0x01);

        // Request ID
        response.push(request_id);

        // Block number (0x80000000 for single response)
        response.extend_from_slice(&0x80000000u32.to_le_bytes());

        // Reserved (8 bytes of '9')
        response.extend_from_slice(b"99999999");

        // Service
        response.push(0x01);

        // Status (0 = success)
        response.push(0x00);

        // Added status size
        response.push(0x00);

        // Reserved
        response.push(0x00);

        // Added status (0 for success)
        response.extend_from_slice(&0u16.to_le_bytes());

        // Padding
        response.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        response.extend(payload);

        Ok(response)
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
    async fn test_read_variable() {
        let server = MockHsesServer::new()
            .with_variable(0, 42i32)
            .start("127.0.0.1:10040")
            .await?;

        let client = HsesClient::connect("127.0.0.1:10040").await?;
        let value: i32 = client.read_variable(0).await?;
        assert_eq!(value, 42);
    }

    #[test]
    fn test_position_serialization() {
        let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));
        let serialized = position.serialize().unwrap();
        let deserialized = Position::deserialize(&serialized).unwrap();

        assert_eq!(position, deserialized);
    }
}
```

## Performance Considerations

### Memory Management

1. **Zero-Copy Operations**: Use bytes crate for efficient buffer management
2. **Arc for Sharing**: Share client state across async tasks
3. **Pooled Buffers**: Reuse message buffers to reduce allocations
4. **Async I/O**: Non-blocking UDP operations with Tokio

### Concurrency

1. **Request ID Management**: Atomic operations for request ID allocation
2. **Pending Requests**: Thread-safe HashMap for tracking requests
3. **Connection Pooling**: Reuse connections when possible
4. **Batch Operations**: Support for multiple commands in single request
