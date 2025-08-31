//! Basic types and traits for HSES protocol

use crate::error::ProtocolError;
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
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError>
    where
        Self: Sized;
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
        Self {
            var_type,
            index,
            value,
        }
    }

    pub fn with_default(var_type: VarType, index: u8) -> Self
    where
        T: Default,
    {
        Self {
            var_type,
            index,
            value: T::default(),
        }
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
    fn command_id() -> u16 {
        T::command_id()
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl<T: VariableType> Command for WriteVar<T> {
    type Response = ();
    fn command_id() -> u16 {
        T::command_id()
    }
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
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

impl Command for ReadCurrentPosition {
    type Response = crate::position::Position;
    fn command_id() -> u16 {
        0x75
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_creation() {
        let var = Variable::new(VarType::Byte, 1, 42u8);
        assert_eq!(var.var_type, VarType::Byte);
        assert_eq!(var.index, 1);
        assert_eq!(var.value, 42);
    }

    #[test]
    fn test_variable_with_default() {
        let var = Variable::<u8>::with_default(VarType::Byte, 1);
        assert_eq!(var.var_type, VarType::Byte);
        assert_eq!(var.index, 1);
        assert_eq!(var.value, 0u8);
    }

    #[test]
    fn test_division_enum() {
        assert_eq!(Division::Robot as u8, 1);
        assert_eq!(Division::File as u8, 2);
    }

    #[test]
    fn test_service_enum() {
        assert_eq!(Service::GetSingle as u8, 0x0e);
        assert_eq!(Service::SetSingle as u8, 0x10);
        assert_eq!(Service::GetAll as u8, 0x01);
        assert_eq!(Service::SetAll as u8, 0x02);
    }

    #[test]
    fn test_coordinate_system_enum() {
        assert_eq!(CoordinateSystem::Base, CoordinateSystem::Base);
        assert_eq!(CoordinateSystem::Robot, CoordinateSystem::Robot);
        assert_eq!(CoordinateSystem::Tool, CoordinateSystem::Tool);
        assert_eq!(CoordinateSystem::User(1), CoordinateSystem::User(1));
    }

    #[test]
    fn test_coordinate_system_type_enum() {
        assert_eq!(CoordinateSystemType::RobotPulse as u8, 0);
        assert_eq!(CoordinateSystemType::BasePulse as u8, 1);
        assert_eq!(CoordinateSystemType::StationPulse as u8, 3);
        assert_eq!(CoordinateSystemType::RobotCartesian as u8, 4);
    }

    #[test]
    fn test_read_var_command() {
        let read_cmd = ReadVar::<u8> {
            index: 1,
            _phantom: PhantomData,
        };
        assert_eq!(ReadVar::<u8>::command_id(), 0x7a); // ByteVar command ID
        let serialized = read_cmd.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    fn test_write_var_command() {
        let write_cmd = WriteVar::<u8> {
            index: 1,
            value: 42,
        };
        assert_eq!(WriteVar::<u8>::command_id(), 0x7a); // ByteVar command ID
        let serialized = write_cmd.serialize().unwrap();
        assert_eq!(serialized, vec![42, 0, 0, 0]); // u8 is serialized as 4 bytes in HSES protocol
    }

    #[test]
    fn test_read_status_command() {
        let read_status = ReadStatus;
        assert_eq!(ReadStatus::command_id(), 0x72);
        let serialized = read_status.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    fn test_read_current_position_command() {
        let read_pos = ReadCurrentPosition {
            control_group: 1,
            coordinate_system: CoordinateSystemType::RobotPulse,
        };
        assert_eq!(ReadCurrentPosition::command_id(), 0x75);
        let serialized = read_pos.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    fn test_variable_type_serialization() {
        let value: u8 = 42;
        let serialized = value.serialize().unwrap();
        let deserialized = u8::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_variable_type_serialization_i32() {
        let value: i32 = 12345;
        let serialized = value.serialize().unwrap();
        let deserialized = i32::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_variable_type_serialization_f32() {
        let value: f32 = 3.14159;
        let serialized = value.serialize().unwrap();
        let deserialized = f32::deserialize(&serialized).unwrap();
        assert!((value - deserialized).abs() < f32::EPSILON);
    }
}
