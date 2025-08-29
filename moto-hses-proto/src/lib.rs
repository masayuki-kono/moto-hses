//! moto-hses-proto - HSES (High Speed Ethernet Server) protocol implementation

use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;
use std::marker::PhantomData;

pub const DEFAULT_PORT: u16 = 10040;
pub const FILE_PORT: u16 = 10041;

// Core traits for type-safe commands
pub trait Command {
    type Response;
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
}

pub trait VariableType: Send + Sync + 'static {
    fn command_id() -> u16;
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> where Self: Sized;
}

// Variable type definitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarType {
    Io = 0x78,
    Register = 0x79,
    Byte = 0x7a,
    Integer = 0x7b,
    Double = 0x7c,
    Real = 0x7d,
    String = 0x7e,
    RobotPosition = 0x7f,
    BasePosition = 0x80,
    ExternalAxis = 0x81,
}

// Variable object for type-safe operations
#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub var_type: VarType,
    pub index: u8,
    pub value: T,
}

impl<T> Variable<T> {
    pub fn new(var_type: VarType, index: u8, value: T) -> Self {
        Self { var_type, index, value }
    }

    pub fn with_default(var_type: VarType, index: u8) -> Self
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
    User(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystemType {
    RobotPulse = 0,
    BasePulse = 1,
    StationPulse = 3,
    RobotCartesian = 4,
}

// Position data structures
#[derive(Debug, Clone, PartialEq)]
pub struct PulsePosition {
    pub joints: [i32; 8],
    pub control_group: u8,
}

impl PulsePosition {
    pub fn new(joints: [i32; 8], control_group: u8) -> Self {
        Self { joints, control_group }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CartesianPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    pub tool_no: u8,
    pub user_coord_no: u8,
    pub coordinate_system: CoordinateSystem,
}

impl CartesianPosition {
    pub fn new(x: f32, y: f32, z: f32, rx: f32, ry: f32, rz: f32, tool_no: u8, user_coord_no: u8, coordinate_system: CoordinateSystem) -> Self {
        Self { x, y, z, rx, ry, rz, tool_no, user_coord_no, coordinate_system }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    Pulse(PulsePosition),
    Cartesian(CartesianPosition),
}

impl Position {
    pub fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();
        
        match self {
            Position::Pulse(pulse) => {
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&(pulse.control_group as u32).to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                for joint in &pulse.joints {
                    data.extend_from_slice(&joint.to_le_bytes());
                }
            }
            Position::Cartesian(cart) => {
                data.extend_from_slice(&16u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&(cart.tool_no as u32).to_le_bytes());
                data.extend_from_slice(&(cart.user_coord_no as u32).to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&(cart.x * 1000.0).to_le_bytes());
                data.extend_from_slice(&(cart.y * 1000.0).to_le_bytes());
                data.extend_from_slice(&(cart.z * 1000.0).to_le_bytes());
                data.extend_from_slice(&(cart.rx * 1000.0).to_le_bytes());
                data.extend_from_slice(&(cart.ry * 1000.0).to_le_bytes());
                data.extend_from_slice(&(cart.rz * 1000.0).to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        
        Ok(data)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 52 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let position_type = buf.get_u32_le();
        
        match position_type {
            0 => {
                let _form = buf.get_u32_le();
                let control_group = buf.get_u32_le() as u8;
                let _user_coord = buf.get_u32_le();
                let _extended_form = buf.get_u32_le();
                
                let mut joints = [0i32; 8];
                for i in 0..8 {
                    joints[i] = buf.get_i32_le();
                }
                
                Ok(Position::Pulse(PulsePosition::new(joints, control_group)))
            }
            16 => {
                let _form = buf.get_u32_le();
                let tool_no = buf.get_u32_le() as u8;
                let user_coord_no = buf.get_u32_le() as u8;
                let _extended_form = buf.get_u32_le();
                
                let x = buf.get_f32_le() / 1000.0;
                let y = buf.get_f32_le() / 1000.0;
                let z = buf.get_f32_le() / 1000.0;
                let rx = buf.get_f32_le() / 1000.0;
                let ry = buf.get_f32_le() / 1000.0;
                let rz = buf.get_f32_le() / 1000.0;
                
                Ok(Position::Cartesian(CartesianPosition::new(
                    x, y, z, rx, ry, rz, tool_no, user_coord_no, CoordinateSystem::Base
                )))
            }
            _ => Err(ProtocolError::PositionError(format!("Unknown position type: {}", position_type))),
        }
    }
}

// Enhanced Error Handling
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
    #[error("invalid message: {0}")]
    InvalidMessage(String),
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
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let status_word1 = buf.get_u16_le();
        let status_word2 = buf.get_u16_le();

        Ok(Self {
            step: (status_word1 & 0x0001) != 0,
            one_cycle: (status_word1 & 0x0002) != 0,
            continuous: (status_word1 & 0x0004) != 0,
            running: (status_word1 & 0x0008) != 0,
            speed_limited: (status_word1 & 0x0010) != 0,
            teach: (status_word1 & 0x0020) != 0,
            play: (status_word1 & 0x0040) != 0,
            remote: (status_word1 & 0x0080) != 0,
            teach_pendant_hold: (status_word2 & 0x0002) != 0,
            external_hold: (status_word2 & 0x0004) != 0,
            command_hold: (status_word2 & 0x0008) != 0,
            alarm: (status_word2 & 0x0010) != 0,
            error: (status_word2 & 0x0020) != 0,
            servo_on: (status_word2 & 0x0040) != 0,
        })
    }

    pub fn is_running(&self) -> bool { self.running }
    pub fn is_servo_on(&self) -> bool { self.servo_on }
    pub fn has_alarm(&self) -> bool { self.alarm }
    pub fn is_teach_mode(&self) -> bool { self.teach }
    pub fn is_play_mode(&self) -> bool { self.play }
    pub fn is_remote_mode(&self) -> bool { self.remote }
}

impl VariableType for Status {
    fn command_id() -> u16 { 0x72 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();
        let mut status_word1 = 0u16;
        let mut status_word2 = 0u16;
        
        if self.step { status_word1 |= 0x0001; }
        if self.one_cycle { status_word1 |= 0x0002; }
        if self.continuous { status_word1 |= 0x0004; }
        if self.running { status_word1 |= 0x0008; }
        if self.speed_limited { status_word1 |= 0x0010; }
        if self.teach { status_word1 |= 0x0020; }
        if self.play { status_word1 |= 0x0040; }
        if self.remote { status_word1 |= 0x0080; }
        
        if self.teach_pendant_hold { status_word2 |= 0x0002; }
        if self.external_hold { status_word2 |= 0x0004; }
        if self.command_hold { status_word2 |= 0x0008; }
        if self.alarm { status_word2 |= 0x0010; }
        if self.error { status_word2 |= 0x0020; }
        if self.servo_on { status_word2 |= 0x0040; }
        
        data.extend_from_slice(&status_word1.to_le_bytes());
        data.extend_from_slice(&status_word2.to_le_bytes());
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Status::from_bytes(data)
    }
}

// HSES Message Structure
#[derive(Debug, Clone)]
pub struct HsesHeader {
    pub magic: [u8; 4],
    pub header_size: u16,
    pub payload_size: u16,
    pub reserved_magic: u8,
    pub division: u8,
    pub ack: u8,
    pub request_id: u8,
    pub block_number: u32,
    pub reserved: [u8; 8],
}

impl HsesHeader {
    pub fn new(division: u8, ack: u8, request_id: u8, payload_size: u16) -> Self {
        Self {
            magic: *b"YERC",
            header_size: 0x20,
            payload_size,
            reserved_magic: 0x03,
            division,
            ack,
            request_id,
            block_number: if ack == 0x01 { 0x80000000 } else { 0 },
            reserved: *b"99999999",
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.extend_from_slice(&self.magic);
        dst.put_u16_le(self.header_size);
        dst.put_u16_le(self.payload_size);
        dst.put_u8(self.reserved_magic);
        dst.put_u8(self.division);
        dst.put_u8(self.ack);
        dst.put_u8(self.request_id);
        dst.put_u32_le(self.block_number);
        dst.extend_from_slice(&self.reserved);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, ProtocolError> {
        if src.len() < 24 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = *src;
        let magic = [buf.get_u8(), buf.get_u8(), buf.get_u8(), buf.get_u8()];
        if magic != *b"YERC" {
            return Err(ProtocolError::InvalidHeader);
        }

        let header_size = buf.get_u16_le();
        let payload_size = buf.get_u16_le();
        let reserved_magic = buf.get_u8();
        let division = buf.get_u8();
        let ack = buf.get_u8();
        let request_id = buf.get_u8();
        let block_number = buf.get_u32_le();
        let mut reserved = [0u8; 8];
        buf.copy_to_slice(&mut reserved);

        *src = &buf[..];

        Ok(Self {
            magic,
            header_size,
            payload_size,
            reserved_magic,
            division,
            ack,
            request_id,
            block_number,
            reserved,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HsesSubHeader {
    pub command: u16,
    pub instance: u16,
    pub attribute: u8,
    pub service: u8,
    pub padding: u16,
}

impl HsesSubHeader {
    pub fn new(command: u16, instance: u16, attribute: u8, service: u8) -> Self {
        Self {
            command,
            instance,
            attribute,
            service,
            padding: 0,
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u16_le(self.command);
        dst.put_u16_le(self.instance);
        dst.put_u8(self.attribute);
        dst.put_u8(self.service);
        dst.put_u16_le(self.padding);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, ProtocolError> {
        if src.len() < 8 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = *src;
        let command = buf.get_u16_le();
        let instance = buf.get_u16_le();
        let attribute = buf.get_u8();
        let service = buf.get_u8();
        let padding = buf.get_u16_le();

        *src = &buf[..];

        Ok(Self {
            command,
            instance,
            attribute,
            service,
            padding,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HsesMessage {
    pub header: HsesHeader,
    pub sub_header: HsesSubHeader,
    pub payload: Vec<u8>,
}

impl HsesMessage {
    pub fn new(division: u8, ack: u8, request_id: u8, command: u16, instance: u16, attribute: u8, service: u8, payload: Vec<u8>) -> Self {
        let header = HsesHeader::new(division, ack, request_id, payload.len() as u16);
        let sub_header = HsesSubHeader::new(command, instance, attribute, service);
        Self { header, sub_header, payload }
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(32 + self.payload.len());
        self.header.encode(&mut buf);
        self.sub_header.encode(&mut buf);
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(src: &[u8]) -> Result<Self, ProtocolError> {
        let mut buf = src;
        let header = HsesHeader::decode(&mut buf)?;
        let sub_header = HsesSubHeader::decode(&mut buf)?;
        let payload = buf.to_vec();
        
        Ok(Self { header, sub_header, payload })
    }
}

// Type-safe command definitions
pub struct ReadVar<T> {
    pub index: u8,
    pub _phantom: PhantomData<T>,
}

pub struct WriteVar<T> {
    pub index: u8,
    pub value: T,
}

// Generic Command implementations for ReadVar and WriteVar
impl<T: VariableType> Command for ReadVar<T> {
    type Response = T;
    fn command_id() -> u16 { T::command_id() }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl<T: VariableType> Command for WriteVar<T> {
    type Response = ();
    fn command_id() -> u16 { T::command_id() }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.value.serialize()
    }
}

pub struct ReadStatus;
pub struct ReadCurrentPosition {
    pub control_group: u8,
    pub coordinate_system: CoordinateSystemType,
}

// Implementations for basic variable types
impl VariableType for u8 {
    fn command_id() -> u16 { 0x7a }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = vec![0u8; 4];
        data[0] = *self;
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        Ok(data[0])
    }
}

impl VariableType for i32 {
    fn command_id() -> u16 { 0x7b }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_i32_le())
    }
}

impl VariableType for f32 {
    fn command_id() -> u16 { 0x7d }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.to_le_bytes().to_vec())
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }
        let mut buf = data;
        Ok(buf.get_f32_le())
    }
}

impl VariableType for Position {
    fn command_id() -> u16 { 0x7f }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.serialize()
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Position::deserialize(data)
    }
}

// Unit type implementation for write operations
impl VariableType for () {
    fn command_id() -> u16 { 0x00 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn deserialize(_data: &[u8]) -> Result<Self, ProtocolError> {
        Ok(())
    }
}

// Wrapper types to avoid orphan rule violations
pub struct StatusWrapper(pub Status);

impl VariableType for StatusWrapper {
    fn command_id() -> u16 { 0x72 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.0.serialize()
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Status::from_bytes(data).map(StatusWrapper)
    }
}

impl From<StatusWrapper> for Status {
    fn from(wrapper: StatusWrapper) -> Self {
        wrapper.0
    }
}

// Command implementations
impl Command for ReadStatus {
    type Response = StatusWrapper;
    fn command_id() -> u16 { 0x72 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl Command for ReadCurrentPosition {
    type Response = Position;
    fn command_id() -> u16 { 0x75 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hses_header_creation() {
        let header = HsesHeader::new(1, 0, 1, 0);
        assert_eq!(header.magic, *b"YERC");
        assert_eq!(header.header_size, 0x20);
        assert_eq!(header.division, 1);
        assert_eq!(header.ack, 0);
        assert_eq!(header.request_id, 1);
    }

    #[test]
    fn test_hses_header_encode_decode() {
        let header = HsesHeader::new(1, 0, 1, 0);
        let mut buf = BytesMut::new();
        header.encode(&mut buf);
        
        let mut data = &buf[..];
        let decoded = HsesHeader::decode(&mut data).unwrap();
        
        assert_eq!(header.magic, decoded.magic);
        assert_eq!(header.header_size, decoded.header_size);
        assert_eq!(header.division, decoded.division);
        assert_eq!(header.ack, decoded.ack);
        assert_eq!(header.request_id, decoded.request_id);
    }

    #[test]
    fn test_status_from_bytes() {
        let data = vec![0x01, 0x00, 0x40, 0x00];
        let status = Status::from_bytes(&data).unwrap();
        assert!(status.step);
        assert!(status.servo_on);
        assert!(!status.running);
    }

    #[test]
    fn test_position_serialization() {
        let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));
        let serialized = position.serialize().unwrap();
        let deserialized = Position::deserialize(&serialized).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    fn test_variable_type_serialization() {
        let value: u8 = 42;
        let serialized = value.serialize().unwrap();
        let deserialized = u8::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
}
