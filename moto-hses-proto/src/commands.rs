//! HSES protocol commands and core traits

use crate::error::ProtocolError;
use crate::position::ControlGroupPositionType;
use std::marker::PhantomData;

// Core traits for type-safe commands
pub trait Command {
    type Response;
    fn command_id() -> u16;
    /// Serialize the command to byte data
    ///
    /// # Errors
    /// Returns `ProtocolError` if serialization fails
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;
    fn instance(&self) -> u16;
    fn attribute(&self) -> u8;
    fn service(&self) -> u8;
}

pub trait VariableType: Send + Sync + 'static {
    fn command_id() -> u16;
    /// Serialize the variable to byte data with specified text encoding
    ///
    /// # Errors
    /// Returns `ProtocolError` if serialization fails
    fn serialize(&self, encoding: crate::encoding::TextEncoding) -> Result<Vec<u8>, ProtocolError>;
    /// Deserialize the variable from byte data with specified text encoding
    ///
    /// # Errors
    /// Returns `ProtocolError` if deserialization fails
    fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError>
    where
        Self: Sized;
}

// Basic enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Division {
    Robot = 1,
    File = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    GetSingle = 0x0e,
    SetSingle = 0x10,
    GetAll = 0x01,
    SetAll = 0x02,
    ReadMultiple = 0x33,
    WriteMultiple = 0x34,
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

pub struct ReadIo {
    pub io_number: u16,
}

pub struct WriteIo {
    pub io_number: u16,
    pub value: bool,
}

pub struct ReadRegister {
    pub register_number: u16,
}

pub struct WriteRegister {
    pub register_number: u16,
    pub value: i16,
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
    fn instance(&self) -> u16 {
        u16::from(self.index) // Variable number (0-99 for byte, 0-999 for int/real)
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

impl<T: VariableType> Command for WriteVar<T> {
    type Response = ();
    fn command_id() -> u16 {
        T::command_id()
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // WriteVar requires encoding, but Command trait doesn't support it
        // This is a design limitation - we'll use UTF-8 as default
        self.value.serialize(crate::encoding::TextEncoding::Utf8)
    }
    fn instance(&self) -> u16 {
        u16::from(self.index) // Variable number (0-99 for byte, 0-999 for int/real)
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

impl Command for ReadIo {
    type Response = bool;
    fn command_id() -> u16 {
        0x78
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        self.io_number
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for I/O commands
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

impl Command for WriteIo {
    type Response = ();
    fn command_id() -> u16 {
        0x78
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let value = u8::from(self.value);
        Ok(vec![value, 0, 0, 0])
    }
    fn instance(&self) -> u16 {
        self.io_number
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for I/O commands
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

impl Command for ReadRegister {
    type Response = i16;
    fn command_id() -> u16 {
        0x79
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        self.register_number
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for register commands
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

impl Command for WriteRegister {
    type Response = ();
    fn command_id() -> u16 {
        0x79
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Register data is 2 bytes (i16) + 2 bytes reserved = 4 bytes total
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.value.to_le_bytes()); // 2 bytes
        payload.extend_from_slice(&[0u8, 0u8]); // 2 bytes reserved
        Ok(payload)
    }
    fn instance(&self) -> u16 {
        self.register_number
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for register commands
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

pub struct ReadStatus;
pub struct ReadStatusData1;
pub struct ReadStatusData2;
pub struct ReadCurrentPosition {
    pub control_group: u8,
    pub coordinate_system: ControlGroupPositionType,
}

// Command implementations
impl Command for ReadStatus {
    type Response = crate::status::Status;
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        1 // Fixed to 1 according to specification
    }
    fn attribute(&self) -> u8 {
        0 // Use 0 to get all attributes (Data 1 and Data 2) with Get_Attribute_All
    }
    fn service(&self) -> u8 {
        0x01 // Get_Attribute_All
    }
}

impl Command for ReadStatusData1 {
    type Response = crate::status::StatusData1;
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        1 // Fixed to 1 according to specification
    }
    fn attribute(&self) -> u8 {
        1 // Data 1
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

impl Command for ReadStatusData2 {
    type Response = crate::status::StatusData2;
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        1 // Fixed to 1 according to specification
    }
    fn attribute(&self) -> u8 {
        2 // Data 2
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
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
    fn instance(&self) -> u16 {
        u16::from(self.control_group) // Control group (1-2 for R1-R2, 11-12 for B1-B2, etc.)
    }
    fn attribute(&self) -> u8 {
        1 // Data type (default)
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[allow(clippy::unwrap_used)]
    fn test_read_var_command() {
        let read_cmd = ReadVar::<u8> { index: 1, _phantom: PhantomData };
        assert_eq!(ReadVar::<u8>::command_id(), 0x7a); // ByteVar command ID
        let serialized = read_cmd.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_write_var_command() {
        let write_cmd = WriteVar::<u8> { index: 1, value: 42 };
        assert_eq!(WriteVar::<u8>::command_id(), 0x7a); // ByteVar command ID
        let serialized = write_cmd.serialize().unwrap();
        assert_eq!(serialized, vec![42, 0, 0, 0]); // u8 is serialized as 4 bytes in HSES protocol
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_status_command() {
        let read_status = ReadStatus;
        assert_eq!(ReadStatus::command_id(), 0x72);
        let serialized = read_status.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_current_position_command() {
        let read_pos = ReadCurrentPosition {
            control_group: 1,
            coordinate_system: ControlGroupPositionType::RobotPulse,
        };
        assert_eq!(ReadCurrentPosition::command_id(), 0x75);
        let serialized = read_pos.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_variable_type_serialization() {
        let value: u8 = 42;
        let serialized = value.serialize(crate::encoding::TextEncoding::Utf8).unwrap();
        let deserialized =
            u8::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_variable_type_serialization_i32() {
        let value: i32 = 12345;
        let serialized = value.serialize(crate::encoding::TextEncoding::Utf8).unwrap();
        let deserialized =
            i32::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_variable_type_serialization_f32() {
        let value: f32 = std::f32::consts::PI;
        let serialized = value.serialize(crate::encoding::TextEncoding::Utf8).unwrap();
        let deserialized =
            f32::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert!((value - deserialized).abs() < f32::EPSILON);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_status_data1_command() {
        let read_status_data1 = ReadStatusData1;
        assert_eq!(ReadStatusData1::command_id(), 0x72);
        assert_eq!(read_status_data1.instance(), 1);
        assert_eq!(read_status_data1.attribute(), 1);
        let serialized = read_status_data1.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_status_data2_command() {
        let read_status_data2 = ReadStatusData2;
        assert_eq!(ReadStatusData2::command_id(), 0x72);
        assert_eq!(read_status_data2.instance(), 1);
        assert_eq!(read_status_data2.attribute(), 2);
        let serialized = read_status_data2.serialize().unwrap();
        assert_eq!(serialized, Vec::<u8>::new());
    }
}

/// Hold/Servo On/off Command (0x83)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldServoControl {
    pub control_type: HoldServoType,
    pub value: HoldServoValue,
}

/// Type of Hold/Servo control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldServoType {
    Hold = 1,
    ServoOn = 2,
    HLock = 3,
}

/// ON/OFF value for Hold/Servo control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldServoValue {
    On = 1,
    Off = 2,
}

impl HoldServoControl {
    /// Create a new Hold/Servo control command
    #[must_use]
    pub const fn new(control_type: HoldServoType, value: HoldServoValue) -> Self {
        Self { control_type, value }
    }

    /// Create HOLD ON command
    #[must_use]
    pub const fn hold_on() -> Self {
        Self::new(HoldServoType::Hold, HoldServoValue::On)
    }

    /// Create HOLD OFF command
    #[must_use]
    pub const fn hold_off() -> Self {
        Self::new(HoldServoType::Hold, HoldServoValue::Off)
    }

    /// Create Servo ON command
    #[must_use]
    pub const fn servo_on() -> Self {
        Self::new(HoldServoType::ServoOn, HoldServoValue::On)
    }

    /// Create Servo OFF command
    #[must_use]
    pub const fn servo_off() -> Self {
        Self::new(HoldServoType::ServoOn, HoldServoValue::Off)
    }

    /// Create HLOCK ON command
    #[must_use]
    pub const fn hlock_on() -> Self {
        Self::new(HoldServoType::HLock, HoldServoValue::On)
    }

    /// Create HLOCK OFF command
    #[must_use]
    pub const fn hlock_off() -> Self {
        Self::new(HoldServoType::HLock, HoldServoValue::Off)
    }
}

impl Command for HoldServoControl {
    type Response = ();

    fn command_id() -> u16 {
        0x83
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&(self.value as i32).to_le_bytes());
        Ok(payload)
    }

    fn instance(&self) -> u16 {
        self.control_type as u16
    }

    fn attribute(&self) -> u8 {
        1
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

#[cfg(test)]
mod hold_servo_tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_hold_servo_control_serialization() {
        let hold_on = HoldServoControl::hold_on();
        assert_eq!(hold_on.instance(), 1);
        assert_eq!(hold_on.attribute(), 1);
        assert_eq!(hold_on.service(), 0x10);
        let serialized = hold_on.serialize().unwrap();
        assert_eq!(serialized, vec![1, 0, 0, 0]);

        let servo_off = HoldServoControl::servo_off();
        assert_eq!(servo_off.instance(), 2);
        let serialized = servo_off.serialize().unwrap();
        assert_eq!(serialized, vec![2, 0, 0, 0]);

        let hlock_on = HoldServoControl::hlock_on();
        assert_eq!(hlock_on.instance(), 3);
        let serialized = hlock_on.serialize().unwrap();
        assert_eq!(serialized, vec![1, 0, 0, 0]);
    }
}
