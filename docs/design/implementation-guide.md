# Implementation Guide

## Overview

This document provides a step-by-step guide for implementing the Rust HSES client library, based on the C++ reference implementation from [fizyr/yaskawa_ethernet](https://github.com/fizyr/yaskawa_ethernet).

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

impl From<PoseConfiguration> for u8 {
    fn from(config: PoseConfiguration) -> Self {
        (config.no_flip as u8) |
        ((config.lower_arm as u8) << 1) |
        ((config.back as u8) << 2) |
        ((config.high_r as u8) << 3) |
        ((config.high_t as u8) << 4) |
        ((config.high_s as u8) << 5)
    }
}
```

#### Step 2: Position Types

```rust
// src/position.rs
#[derive(Debug, Clone)]
pub struct PulsePosition {
    joints: [i32; 8],
    tool: i32,
    size: u8, // Number of active joints
}

impl PulsePosition {
    pub fn new(joints: [i32; 8], tool: i32) -> Self {
        Self { joints, tool, size: 8 }
    }

    pub fn new_with_size(joints: [i32; 8], size: u8, tool: i32) -> Self {
        Self { joints, tool, size }
    }

    pub fn joints(&self) -> &[i32] {
        &self.joints[..self.size as usize]
    }

    pub fn joints_mut(&mut self) -> &mut [i32] {
        &mut self.joints[..self.size as usize]
    }

    pub fn tool(&self) -> i32 {
        self.tool
    }

    pub fn set_tool(&mut self, tool: i32) {
        self.tool = tool;
    }

    pub fn size(&self) -> u8 {
        self.size
    }
}

#[derive(Debug, Clone)]
pub struct CartesianPosition {
    x: f64,
    y: f64,
    z: f64,
    rx: f64,
    ry: f64,
    rz: f64,
    frame: CoordinateSystem,
    configuration: PoseConfiguration,
    tool: i32,
}

impl CartesianPosition {
    pub fn new(
        x: f64, y: f64, z: f64,
        rx: f64, ry: f64, rz: f64,
        frame: CoordinateSystem,
        configuration: PoseConfiguration,
        tool: i32,
    ) -> Self {
        Self { x, y, z, rx, ry, rz, frame, configuration, tool }
    }

    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }
    pub fn rx(&self) -> f64 { self.rx }
    pub fn ry(&self) -> f64 { self.ry }
    pub fn rz(&self) -> f64 { self.rz }
    pub fn frame(&self) -> CoordinateSystem { self.frame }
    pub fn configuration(&self) -> PoseConfiguration { self.configuration }
    pub fn tool(&self) -> i32 { self.tool }

    pub fn set_x(&mut self, x: f64) { self.x = x; }
    pub fn set_y(&mut self, y: f64) { self.y = y; }
    pub fn set_z(&mut self, z: f64) { self.z = z; }
    pub fn set_rx(&mut self, rx: f64) { self.rx = rx; }
    pub fn set_ry(&mut self, ry: f64) { self.ry = ry; }
    pub fn set_rz(&mut self, rz: f64) { self.rz = rz; }
    pub fn set_frame(&mut self, frame: CoordinateSystem) { self.frame = frame; }
    pub fn set_configuration(&mut self, config: PoseConfiguration) { self.configuration = config; }
    pub fn set_tool(&mut self, tool: i32) { self.tool = tool; }
}

// Unified Position type
#[derive(Debug, Clone)]
pub enum Position {
    Pulse(PulsePosition),
    Cartesian(CartesianPosition),
}

impl Position {
    pub fn is_pulse(&self) -> bool {
        matches!(self, Position::Pulse(_))
    }

    pub fn is_cartesian(&self) -> bool {
        matches!(self, Position::Cartesian(_))
    }

    pub fn as_pulse(&self) -> Option<&PulsePosition> {
        match self {
            Position::Pulse(pos) => Some(pos),
            _ => None,
        }
    }

    pub fn as_pulse_mut(&mut self) -> Option<&mut PulsePosition> {
        match self {
            Position::Pulse(pos) => Some(pos),
            _ => None,
        }
    }

    pub fn as_cartesian(&self) -> Option<&CartesianPosition> {
        match self {
            Position::Cartesian(pos) => Some(pos),
            _ => None,
        }
    }

    pub fn as_cartesian_mut(&mut self) -> Option<&mut CartesianPosition> {
        match self {
            Position::Cartesian(pos) => Some(pos),
            _ => None,
        }
    }
}
```

#### Step 3: Status Types

```rust
// src/status.rs
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
    pub fn is_running(&self) -> bool { self.running }
    pub fn is_servo_on(&self) -> bool { self.servo_on }
    pub fn has_alarm(&self) -> bool { self.alarm }
    pub fn has_error(&self) -> bool { self.error }
    pub fn is_teach_mode(&self) -> bool { self.teach }
    pub fn is_play_mode(&self) -> bool { self.play }
    pub fn is_remote_mode(&self) -> bool { self.remote }
    pub fn is_step_mode(&self) -> bool { self.step }
    pub fn is_continuous_mode(&self) -> bool { self.continuous }
    pub fn is_speed_limited(&self) -> bool { self.speed_limited }
    pub fn is_hold(&self) -> bool {
        self.teach_pendant_hold || self.external_hold || self.command_hold
    }
}
```

#### Step 4: Command Definitions

```rust
// src/commands.rs
use std::marker::PhantomData;

// Type-safe command definitions
pub struct ReadVar<T> {
    pub index: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVar<T> {
    pub index: u8,
    pub value: T,
}

pub struct ReadVars<T> {
    pub index: u8,
    pub count: u8,
    _phantom: PhantomData<T>,
}

pub struct WriteVars<T> {
    pub index: u8,
    pub values: Vec<T>,
}

// Simple commands
pub struct ReadStatus;
pub struct ReadCurrentPosition {
    pub control_group: u8,
    pub coordinate_system: CoordinateSystemType,
}

pub struct MoveL {
    pub control_group: u8,
    pub target: CartesianPosition,
    pub speed: Speed,
}

pub struct MoveP {
    pub control_group: u8,
    pub target: PulsePosition,
    pub speed: Speed,
}

// Speed and configuration
#[derive(Debug, Clone, Copy)]
pub enum SpeedType {
    Joint,       // 0.01% of max speed
    Translation, // 0.1 mm/s
    Rotation     // 0.1 degrees/s
}

#[derive(Debug, Clone)]
pub struct Speed {
    pub speed_type: SpeedType,
    pub value: u32,
}

impl Speed {
    pub fn new(speed_type: SpeedType, value: u32) -> Self {
        Self { speed_type, value }
    }
}

// Command implementations
impl<T: VariableType> Command for ReadVar<T> {
    type Response = T;

    fn command_id() -> u16 {
        T::command_id()
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.index.to_le_bytes());
        Ok(buf)
    }
}

impl<T: VariableType> Command for WriteVar<T> {
    type Response = ();

    fn command_id() -> u16 {
        T::command_id()
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.index.to_le_bytes());
        buf.extend(self.value.serialize()?);
        Ok(buf)
    }
}

impl Command for ReadStatus {
    type Response = Status;

    fn command_id() -> u16 {
        0x72
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl Command for ReadCurrentPosition {
    type Response = Position;

    fn command_id() -> u16 {
        0x75
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.control_group.to_le_bytes());
        buf.extend_from_slice(&(self.coordinate_system as u8).to_le_bytes());
        Ok(buf)
    }
}
```

#### Step 5: Variable Type Implementations

```rust
// src/variables.rs
impl VariableType for u8 {
    fn command_id() -> u16 { 0x7A }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.to_le_bytes());
        buf.extend_from_slice(&[0u8; 3]); // Reserved bytes
        Ok(buf)
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::DeserializationError("Insufficient data for u8".to_string()));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&data[..4]);
        Ok(u8::from_le_bytes(bytes))
    }
}

impl VariableType for i16 {
    fn command_id() -> u16 { 0x7B }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.to_le_bytes());
        buf.extend_from_slice(&[0u8; 2]); // Reserved bytes
        Ok(buf)
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::DeserializationError("Insufficient data for i16".to_string()));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&data[..4]);
        Ok(i32::from_le_bytes(bytes) as i16)
    }
}

impl VariableType for i32 {
    fn command_id() -> u16 { 0x7C }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(buf)
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::DeserializationError("Insufficient data for i32".to_string()));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&data[..4]);
        Ok(i32::from_le_bytes(bytes))
    }
}

impl VariableType for f32 {
    fn command_id() -> u16 { 0x7D }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(buf)
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::DeserializationError("Insufficient data for f32".to_string()));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&data[..4]);
        Ok(f32::from_le_bytes(bytes))
    }
}

impl VariableType for Position {
    fn command_id() -> u16 { 0x7F }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        match self {
            Position::Pulse(pos) => {
                // Position type (0x00 for pulse)
                buf.extend_from_slice(&0u32.to_le_bytes());
                // Joint configuration
                buf.extend_from_slice(&0u32.to_le_bytes());
                // Tool number
                buf.extend_from_slice(&pos.tool().to_le_bytes());
                // User coordinate
                buf.extend_from_slice(&0u32.to_le_bytes());
                // Extended joint configuration
                buf.extend_from_slice(&0u32.to_le_bytes());
                // Joint values
                for joint in pos.joints() {
                    buf.extend_from_slice(&joint.to_le_bytes());
                }
                // Pad remaining joints with zeros
                for _ in pos.joints().len()..8 {
                    buf.extend_from_slice(&0i32.to_le_bytes());
                }
            }
            Position::Cartesian(pos) => {
                // Position type (0x10 for base frame)
                buf.extend_from_slice(&0x10u32.to_le_bytes());
                // Joint configuration
                buf.extend_from_slice(&(pos.configuration() as u8 as u32).to_le_bytes());
                // Tool number
                buf.extend_from_slice(&pos.tool().to_le_bytes());
                // User coordinate number
                buf.extend_from_slice(&0u32.to_le_bytes());
                // Extended joint configuration
                buf.extend_from_slice(&0u32.to_le_bytes());
                // XYZ coordinates (in micrometers)
                buf.extend_from_slice(&(pos.x() * 1000.0) as i32).to_le_bytes());
                buf.extend_from_slice(&(pos.y() * 1000.0) as i32).to_le_bytes());
                buf.extend_from_slice(&(pos.z() * 1000.0) as i32).to_le_bytes());
                // Rotation coordinates (in millidegrees)
                buf.extend_from_slice(&(pos.rx() * 1000.0) as i32).to_le_bytes());
                buf.extend_from_slice(&(pos.ry() * 1000.0) as i32).to_le_bytes());
                buf.extend_from_slice(&(pos.rz() * 1000.0) as i32).to_le_bytes());
                // Padding
                buf.extend_from_slice(&0u32.to_le_bytes());
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        Ok(buf)
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 52 {
            return Err(ProtocolError::DeserializationError("Insufficient data for Position".to_string()));
        }

        let mut cursor = std::io::Cursor::new(data);
        let position_type = cursor.get_u32_le();

        match position_type {
            0x00 => {
                // Pulse position
                let _joint_config = cursor.get_u32_le();
                let tool = cursor.get_i32_le();
                let _user_coordinate = cursor.get_u32_le();
                let _extended_config = cursor.get_u32_le();

                let mut joints = [0i32; 8];
                for i in 0..8 {
                    joints[i] = cursor.get_i32_le();
                }

                Ok(Position::Pulse(PulsePosition::new(joints, tool)))
            }
            0x10..=0x22 => {
                // Cartesian position
                let joint_config = cursor.get_u32_le();
                let tool = cursor.get_i32_le();
                let _user_coordinate = cursor.get_u32_le();
                let _extended_config = cursor.get_u32_le();

                let x = cursor.get_i32_le() as f64 / 1000.0; // micrometers to mm
                let y = cursor.get_i32_le() as f64 / 1000.0;
                let z = cursor.get_i32_le() as f64 / 1000.0;
                let rx = cursor.get_i32_le() as f64 / 1000.0; // millidegrees to degrees
                let ry = cursor.get_i32_le() as f64 / 1000.0;
                let rz = cursor.get_i32_le() as f64 / 1000.0;

                let frame = match position_type {
                    0x10 => CoordinateSystem::Base,
                    0x11 => CoordinateSystem::Robot,
                    0x12 => CoordinateSystem::Tool,
                    n if n >= 0x13 && n <= 0x22 => CoordinateSystem::User((n - 0x13 + 1) as u8),
                    _ => return Err(ProtocolError::DeserializationError("Invalid position type".to_string())),
                };

                let configuration = PoseConfiguration::from(joint_config as u8);

                Ok(Position::Cartesian(CartesianPosition::new(
                    x, y, z, rx, ry, rz, frame, configuration, tool
                )))
            }
            _ => Err(ProtocolError::DeserializationError("Unknown position type".to_string())),
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

## Migration from Current Design

### Key Changes

1. **Template-Based Commands**: Replace enum-based commands with trait-based system
2. **Unified Position Type**: Use enum instead of separate types
3. **Enhanced Type Safety**: Leverage Rust's type system more effectively
4. **Async-First API**: Improve async patterns and error handling
5. **Better Mock Support**: Enhanced testing capabilities

### Backward Compatibility

- Maintain existing API where possible
- Provide migration guides for breaking changes
- Support both old and new patterns during transition
