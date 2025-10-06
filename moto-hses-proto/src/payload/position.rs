//! Position data structures and operations

use crate::error::ProtocolError;
use crate::payload::HsesPayload;
use bytes::Buf;

// Form bit field definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J1Placement {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J3Placement {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J5Placement {
    Flip,
    NoFlip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J4TurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J6TurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J1TurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedundantJ1Placement {
    RedundantFront,
    RedundantBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverseConversion {
    PreviousStepRegarded,
    FormRegarded,
}

// Extended form bit field definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedJ2TurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedJ3TurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedJ5TurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedEAxisTurnNum {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedWAxisTurnNum {
    Single,
    Double,
}

// Form structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Form {
    pub j1_placement: J1Placement,
    pub j3_placement: J3Placement,
    pub j5_placement: J5Placement,
    pub j4_turn_num: J4TurnNum,
    pub j6_turn_num: J6TurnNum,
    pub j1_turn_num: J1TurnNum,
    pub redundant_j1_placement: RedundantJ1Placement,
    pub reverse_conversion_select: ReverseConversion,
}

// Extended form structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedForm {
    pub bit0: ExtendedJ2TurnNum,
    pub bit1: ExtendedJ3TurnNum,
    pub bit2: ExtendedJ5TurnNum,
    pub bit3: ExtendedEAxisTurnNum,
    pub bit4: ExtendedWAxisTurnNum,
    pub bit5: bool, // Reserve
    pub bit6: bool, // Reserve
    pub bit7: bool, // Reserve
}

impl Form {
    /// Create Form from raw u8 value
    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        Self {
            j1_placement: if value & 0x01 == 0 { J1Placement::Front } else { J1Placement::Back },
            j3_placement: if value & 0x02 == 0 { J3Placement::Up } else { J3Placement::Down },
            j5_placement: if value & 0x04 == 0 { J5Placement::Flip } else { J5Placement::NoFlip },
            j4_turn_num: if value & 0x08 == 0 { J4TurnNum::Single } else { J4TurnNum::Double },
            j6_turn_num: if value & 0x10 == 0 { J6TurnNum::Single } else { J6TurnNum::Double },
            j1_turn_num: if value & 0x20 == 0 { J1TurnNum::Single } else { J1TurnNum::Double },
            redundant_j1_placement: if value & 0x40 == 0 {
                RedundantJ1Placement::RedundantFront
            } else {
                RedundantJ1Placement::RedundantBack
            },
            reverse_conversion_select: if value & 0x80 == 0 {
                ReverseConversion::PreviousStepRegarded
            } else {
                ReverseConversion::FormRegarded
            },
        }
    }

    /// Convert Form to raw u8 value
    #[must_use]
    pub const fn to_raw(self) -> u8 {
        let mut value = 0u8;
        if matches!(self.j1_placement, J1Placement::Back) {
            value |= 0x01;
        }
        if matches!(self.j3_placement, J3Placement::Down) {
            value |= 0x02;
        }
        if matches!(self.j5_placement, J5Placement::NoFlip) {
            value |= 0x04;
        }
        if matches!(self.j4_turn_num, J4TurnNum::Double) {
            value |= 0x08;
        }
        if matches!(self.j6_turn_num, J6TurnNum::Double) {
            value |= 0x10;
        }
        if matches!(self.j1_turn_num, J1TurnNum::Double) {
            value |= 0x20;
        }
        if matches!(self.redundant_j1_placement, RedundantJ1Placement::RedundantBack) {
            value |= 0x40;
        }
        if matches!(self.reverse_conversion_select, ReverseConversion::FormRegarded) {
            value |= 0x80;
        }
        value
    }
}

impl ExtendedForm {
    /// Create `ExtendedForm` from raw u8 value
    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        Self {
            bit0: if value & 0x01 == 0 {
                ExtendedJ2TurnNum::Single
            } else {
                ExtendedJ2TurnNum::Double
            },
            bit1: if value & 0x02 == 0 {
                ExtendedJ3TurnNum::Single
            } else {
                ExtendedJ3TurnNum::Double
            },
            bit2: if value & 0x04 == 0 {
                ExtendedJ5TurnNum::Single
            } else {
                ExtendedJ5TurnNum::Double
            },
            bit3: if value & 0x08 == 0 {
                ExtendedEAxisTurnNum::Single
            } else {
                ExtendedEAxisTurnNum::Double
            },
            bit4: if value & 0x10 == 0 {
                ExtendedWAxisTurnNum::Single
            } else {
                ExtendedWAxisTurnNum::Double
            },
            bit5: value & 0x20 != 0, // Reserve
            bit6: value & 0x40 != 0, // Reserve
            bit7: value & 0x80 != 0, // Reserve
        }
    }

    /// Convert `ExtendedForm` to raw u8 value
    #[must_use]
    pub const fn to_raw(self) -> u8 {
        let mut value = 0u8;
        if matches!(self.bit0, ExtendedJ2TurnNum::Double) {
            value |= 0x01;
        }
        if matches!(self.bit1, ExtendedJ3TurnNum::Double) {
            value |= 0x02;
        }
        if matches!(self.bit2, ExtendedJ5TurnNum::Double) {
            value |= 0x04;
        }
        if matches!(self.bit3, ExtendedEAxisTurnNum::Double) {
            value |= 0x08;
        }
        if matches!(self.bit4, ExtendedWAxisTurnNum::Double) {
            value |= 0x10;
        }
        if self.bit5 {
            value |= 0x20;
        } // Reserve
        if self.bit6 {
            value |= 0x40;
        } // Reserve
        if self.bit7 {
            value |= 0x80;
        } // Reserve
        value
    }
}

// Position data structures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PulsePosition {
    pub joints: Vec<i32>,
}

impl PulsePosition {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(joints: Vec<i32>) -> Self {
        Self { joints }
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
    pub form: Form,
    pub extended_form: ExtendedForm,
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
        form: Form,
        extended_form: ExtendedForm,
    ) -> Self {
        Self { x, y, z, rx, ry, rz, tool_no, user_coord_no, form, extended_form }
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
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                for joint in &pulse.joints {
                    data.extend_from_slice(&joint.to_le_bytes());
                }
            }
            Self::Cartesian(cart) => {
                data.extend_from_slice(&16u32.to_le_bytes());

                // Use the form from the CartesianPosition
                data.extend_from_slice(&u32::from(cart.form.to_raw()).to_le_bytes());

                data.extend_from_slice(&u32::from(cart.tool_no).to_le_bytes());
                data.extend_from_slice(&u32::from(cart.user_coord_no).to_le_bytes());
                data.extend_from_slice(&u32::from(cart.extended_form.to_raw()).to_le_bytes());

                // Convert coordinates to proper units
                #[allow(clippy::cast_possible_truncation)]
                {
                    data.extend_from_slice(&((cart.x * 1000.0) as i32).to_le_bytes()); // mm to μm
                    data.extend_from_slice(&((cart.y * 1000.0) as i32).to_le_bytes()); // mm to μm
                    data.extend_from_slice(&((cart.z * 1000.0) as i32).to_le_bytes()); // mm to μm
                    data.extend_from_slice(&((cart.rx * 10000.0) as i32).to_le_bytes()); // deg to 0.0001deg
                    data.extend_from_slice(&((cart.ry * 10000.0) as i32).to_le_bytes()); // deg to 0.0001deg
                    data.extend_from_slice(&((cart.rz * 10000.0) as i32).to_le_bytes()); // deg to 0.0001deg
                }
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
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let position_type = buf.get_u32_le();

        match position_type {
            0 => {
                // Pulse position: minimum 44 bytes (20 bytes header + 24 bytes joints)
                if data.len() < 44 {
                    return Err(ProtocolError::Underflow);
                }

                let mut buf = data;
                let _position_type = buf.get_u32_le(); // Already read above
                let _form = buf.get_u32_le();
                #[allow(clippy::cast_possible_truncation)]
                let _tool_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let _user_coord_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let _extended_form_raw = buf.get_u32_le() as u8;

                // Read joints - determine the number of joints based on remaining data
                let mut joints = Vec::new();

                // Calculate how many joints we can read from the remaining buffer
                let remaining_bytes = buf.remaining();
                let joint_count = remaining_bytes / 4; // Each joint is 4 bytes (i32)

                // Read all available joints from the buffer
                for _ in 0..joint_count {
                    joints.push(buf.get_i32_le());
                }

                Ok(Self::Pulse(PulsePosition::new(joints)))
            }
            16 => {
                // Cartesian position: minimum 44 bytes (20 bytes header + 24 bytes coordinates)
                if data.len() < 44 {
                    return Err(ProtocolError::Underflow);
                }

                let mut buf = data;
                let _position_type = buf.get_u32_le(); // Already read above

                // Read header fields (4 bytes each)
                #[allow(clippy::cast_possible_truncation)]
                let form_raw = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let tool_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let user_coord_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let extended_form_raw = buf.get_u32_le() as u8;

                // Parse form and extended form
                let form = Form::from_raw(form_raw);
                let extended_form = ExtendedForm::from_raw(extended_form_raw);

                // Read coordinate data from the remaining bytes (24 bytes for 6 coordinates)
                let coord_data = &data[20..44];
                let mut coord_buf = coord_data;

                #[allow(clippy::cast_precision_loss)]
                let x = coord_buf.get_i32_le() as f32 / 1000.0; // μm to mm
                #[allow(clippy::cast_precision_loss)]
                let y = coord_buf.get_i32_le() as f32 / 1000.0; // μm to mm
                #[allow(clippy::cast_precision_loss)]
                let z = coord_buf.get_i32_le() as f32 / 1000.0; // μm to mm
                #[allow(clippy::cast_precision_loss)]
                let rx = coord_buf.get_i32_le() as f32 / 10000.0; // 0.0001deg to deg
                #[allow(clippy::cast_precision_loss)]
                let ry = coord_buf.get_i32_le() as f32 / 10000.0; // 0.0001deg to deg
                #[allow(clippy::cast_precision_loss)]
                let rz = coord_buf.get_i32_le() as f32 / 10000.0; // 0.0001deg to deg

                Ok(Self::Cartesian(CartesianPosition::new(
                    x,
                    y,
                    z,
                    rx,
                    ry,
                    rz,
                    tool_no,
                    user_coord_no,
                    form,
                    extended_form,
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
    use super::*;

    #[test]
    fn test_pulse_position_creation() {
        let joints = vec![1000, 2000, 3000, 0, 0, 0, 0, 0];
        let position = PulsePosition::new(joints.clone());
        assert_eq!(position.joints, joints);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_cartesian_position_creation() {
        let form = Form::from_raw(0);
        let extended_form = ExtendedForm::from_raw(0);
        let position =
            CartesianPosition::new(100.0, 200.0, 300.0, 0.0, 0.0, 0.0, 1, 0, form, extended_form);
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
        assert_eq!(position.z, 300.0);
        assert_eq!(position.tool_no, 1);
        assert_eq!(position.user_coord_no, 0);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_position_serialization() {
        let position = Position::Pulse(PulsePosition::new(vec![1000, 2000, 3000, 0, 0, 0, 0, 0]));
        let serialized = position.serialize().unwrap();
        let deserialized =
            Position::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cartesian_position_serialization() {
        let form = Form::from_raw(0);
        let extended_form = ExtendedForm::from_raw(0);
        let position = Position::Cartesian(CartesianPosition::new(
            100.0,
            200.0,
            300.0,
            0.0,
            0.0,
            0.0,
            1,
            0,
            form,
            extended_form,
        ));
        let serialized = position.serialize().unwrap();
        let deserialized =
            Position::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_position_variable_type_trait() {
        let position = Position::Pulse(PulsePosition::new(vec![1000, 2000, 3000, 0, 0, 0, 0, 0]));

        let serialized = position.serialize().unwrap();
        let deserialized =
            Position::deserialize(&serialized, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    fn test_form_serialization() {
        let form = Form::from_raw(0x55); // 01010101
        assert_eq!(form.j1_placement, J1Placement::Back);
        assert_eq!(form.j3_placement, J3Placement::Up);
        assert_eq!(form.j5_placement, J5Placement::NoFlip);
        assert_eq!(form.j4_turn_num, J4TurnNum::Single);
        assert_eq!(form.j6_turn_num, J6TurnNum::Double);
        assert_eq!(form.j1_turn_num, J1TurnNum::Single);
        assert_eq!(form.redundant_j1_placement, RedundantJ1Placement::RedundantBack);
        assert_eq!(form.reverse_conversion_select, ReverseConversion::PreviousStepRegarded);

        let raw = form.to_raw();
        assert_eq!(raw, 0x55);
    }

    #[test]
    fn test_extended_form_serialization() {
        let extended_form = ExtendedForm::from_raw(0x1F); // 00011111
        assert_eq!(extended_form.bit0, ExtendedJ2TurnNum::Double);
        assert_eq!(extended_form.bit1, ExtendedJ3TurnNum::Double);
        assert_eq!(extended_form.bit2, ExtendedJ5TurnNum::Double);
        assert_eq!(extended_form.bit3, ExtendedEAxisTurnNum::Double);
        assert_eq!(extended_form.bit4, ExtendedWAxisTurnNum::Double);
        assert!(!extended_form.bit5);
        assert!(!extended_form.bit6);
        assert!(!extended_form.bit7);

        let raw = extended_form.to_raw();
        assert_eq!(raw, 0x1F);
    }
}
