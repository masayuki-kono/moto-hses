//! Basic types and traits for HSES protocol

use std::marker::PhantomData;
use crate::error::ProtocolError;

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

// Command implementations
impl Command for ReadStatus {
    type Response = crate::status::StatusWrapper;
    fn command_id() -> u16 { 0x72 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl Command for ReadCurrentPosition {
    type Response = crate::position::Position;
    fn command_id() -> u16 { 0x75 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}
