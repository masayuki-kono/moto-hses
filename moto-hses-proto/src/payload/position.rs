//! Position data structures and operations

use crate::error::ProtocolError;
use crate::payload::HsesPayload;
use bytes::Buf;

// Coordinate system types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSystemType {
    Base,
    Robot,
    Tool,
    User(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlGroupPositionType {
    RobotPulse = 0,
    BasePulse = 1,
    StationPulse = 3,
    RobotCartesian = 4,
}

// Position data structures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PulsePosition {
    pub joints: [i32; 8],
    pub control_group: u8,
}

impl PulsePosition {
    #[must_use]
    pub const fn new(joints: [i32; 8], control_group: u8) -> Self {
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
    pub coordinate_system: CoordinateSystemType,
}

impl CartesianPosition {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        x: f32,
        y: f32,
        z: f32,
        rx: f32,
        ry: f32,
        rz: f32,
        tool_no: u8,
        user_coord_no: u8,
        coordinate_system: CoordinateSystemType,
    ) -> Self {
        Self { x, y, z, rx, ry, rz, tool_no, user_coord_no, coordinate_system }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    Pulse(PulsePosition),
    Cartesian(CartesianPosition),
}

impl Position {
    /// Serialize position to byte data
    ///
    /// # Errors
    /// Returns `ProtocolError::PositionError` if serialization fails
    pub fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        match self {
            Self::Pulse(pulse) => {
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&u32::from(pulse.control_group).to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                for joint in &pulse.joints {
                    data.extend_from_slice(&joint.to_le_bytes());
                }
            }
            Self::Cartesian(cart) => {
                data.extend_from_slice(&16u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&u32::from(cart.tool_no).to_le_bytes());
                data.extend_from_slice(&u32::from(cart.user_coord_no).to_le_bytes());
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

    /// Deserialize position from byte data
    ///
    /// # Errors
    /// Returns `ProtocolError::Underflow` if data is insufficient
    /// Returns `ProtocolError::PositionError` if data format is invalid
    pub fn deserialize(
        data: &[u8],
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        if data.len() < 52 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let position_type = buf.get_u32_le();

        match position_type {
            0 => {
                let _form = buf.get_u32_le();
                let control_group = u8::try_from(buf.get_u32_le()).map_err(|_| {
                    ProtocolError::PositionError("Invalid control group value".to_string())
                })?;
                let _user_coord = buf.get_u32_le();
                let _extended_form = buf.get_u32_le();

                let mut joints = [0i32; 8];
                for joint in &mut joints {
                    *joint = buf.get_i32_le();
                }

                Ok(Self::Pulse(PulsePosition::new(joints, control_group)))
            }
            16 => {
                let _form = buf.get_u32_le();
                let tool_no = u8::try_from(buf.get_u32_le()).map_err(|_| {
                    ProtocolError::PositionError("Invalid tool number value".to_string())
                })?;
                let user_coord_no = u8::try_from(buf.get_u32_le()).map_err(|_| {
                    ProtocolError::PositionError("Invalid user coordinate number value".to_string())
                })?;
                let _extended_form = buf.get_u32_le();

                let x = buf.get_f32_le() / 1000.0;
                let y = buf.get_f32_le() / 1000.0;
                let z = buf.get_f32_le() / 1000.0;
                let rx = buf.get_f32_le() / 1000.0;
                let ry = buf.get_f32_le() / 1000.0;
                let rz = buf.get_f32_le() / 1000.0;

                Ok(Self::Cartesian(CartesianPosition::new(
                    x,
                    y,
                    z,
                    rx,
                    ry,
                    rz,
                    tool_no,
                    user_coord_no,
                    CoordinateSystemType::Base,
                )))
            }
            _ => {
                Err(ProtocolError::PositionError(format!("Unknown position type: {position_type}")))
            }
        }
    }
}

impl HsesPayload for Position {
    fn serialize(
        &self,
        _encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        self.serialize()
    }
    fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        Self::deserialize(data, encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::CoordinateSystemType;
    use super::*;

    #[test]
    fn test_coordinate_system_enum() {
        assert_eq!(CoordinateSystemType::Base, CoordinateSystemType::Base);
        assert_eq!(CoordinateSystemType::Robot, CoordinateSystemType::Robot);
        assert_eq!(CoordinateSystemType::Tool, CoordinateSystemType::Tool);
        assert_eq!(CoordinateSystemType::User(1), CoordinateSystemType::User(1));
    }

    #[test]
    fn test_control_group_position_type_enum() {
        assert_eq!(ControlGroupPositionType::RobotPulse as u8, 0);
        assert_eq!(ControlGroupPositionType::BasePulse as u8, 1);
        assert_eq!(ControlGroupPositionType::StationPulse as u8, 3);
        assert_eq!(ControlGroupPositionType::RobotCartesian as u8, 4);
    }

    #[test]
    fn test_pulse_position_creation() {
        let joints = [1000, 2000, 3000, 0, 0, 0, 0, 0];
        let position = PulsePosition::new(joints, 1);
        assert_eq!(position.joints, joints);
        assert_eq!(position.control_group, 1);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_cartesian_position_creation() {
        let position = CartesianPosition::new(
            100.0,
            200.0,
            300.0,
            0.0,
            0.0,
            0.0,
            1,
            0,
            CoordinateSystemType::Base,
        );
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
        assert_eq!(position.z, 300.0);
        assert_eq!(position.tool_no, 1);
        assert_eq!(position.user_coord_no, 0);
        assert_eq!(position.coordinate_system, CoordinateSystemType::Base);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_position_serialization() {
        let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));
        let serialized = position.serialize().unwrap();
        let deserialized =
            Position::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cartesian_position_serialization() {
        let position = Position::Cartesian(CartesianPosition::new(
            100.0,
            200.0,
            300.0,
            0.0,
            0.0,
            0.0,
            1,
            0,
            CoordinateSystemType::Base,
        ));
        let serialized = position.serialize().unwrap();
        let deserialized =
            Position::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_position_variable_type_trait() {
        let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));

        let serialized = position.serialize().unwrap();
        let deserialized =
            Position::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(position, deserialized);
    }
}
