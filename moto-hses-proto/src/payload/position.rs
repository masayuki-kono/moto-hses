//! Position data structures and operations

use crate::error::ProtocolError;
use crate::payload::HsesPayload;
use bytes::Buf;

// Configuration bit field definitions
// S(Swing,J1) Axis Placement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SAxisPlacement {
    Front,
    Back,
}

// U(Upper arm,J3) Axis Placement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UAxisPlacement {
    Up,
    Down,
}

// B(Bend,J5) Axis Placement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BAxisPlacement {
    Flip,
    NoFlip,
}

// R(Rotate,J4) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RAxisTurnNum {
    // R < 180
    Single,
    // R ≥ 180
    Double,
}

// T(Twist,J6) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TAxisTurnNum {
    // T < 180
    Single,
    // T ≥ 180
    Double,
}

// S(Swing,J1) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SAxisTurnNum {
    // S < 180
    Single,
    // S ≥ 180
    Double,
}

// Redundant S Axis Placement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedundantSAxisPlacement {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IkSolutionBasis {
    // Select configuration that minimizes joint angle changes from the previous step
    PreviousStep,
    // Prioritize the configuration attached to the position
    Configuration,
}

// Extended configuration bit field definitions
// L(Lower arm,J2) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LAxisTurnNum {
    // L < 180
    Single,
    // L ≥ 180
    Double,
}

// U(Upper arm,J3) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UAxisTurnNum {
    // U < 180
    Single,
    // U ≥ 180
    Double,
}

// B(Bend,J5) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BAxisTurnNum {
    // B < 180
    Single,
    // B ≥ 180
    Double,
}

// E(External axis) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EAxisTurnNum {
    // E < 180
    Single,
    // E ≥ 180
    Double,
}

// W(Wrist-extension axis) Axis Turn Number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WAxisTurnNum {
    // W < 180
    Single,
    // W ≥ 180
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Configuration {
    pub s_placement: SAxisPlacement,
    pub u_placement: UAxisPlacement,
    pub b_placement: BAxisPlacement,
    pub r_turn_num: RAxisTurnNum,
    pub t_turn_num: TAxisTurnNum,
    pub s_turn_num: SAxisTurnNum,
    pub redundant_s_placement: RedundantSAxisPlacement,
    pub ik_solution_basis: IkSolutionBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedConfiguration {
    pub bit0: LAxisTurnNum,
    pub bit1: UAxisTurnNum,
    pub bit2: BAxisTurnNum,
    pub bit3: EAxisTurnNum,
    pub bit4: WAxisTurnNum,
    pub bit5: bool, // Reserve
    pub bit6: bool, // Reserve
    pub bit7: bool, // Reserve
}

impl Configuration {
    /// Create configuration from raw u8 value
    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        Self {
            s_placement: if value & 0x01 == 0 {
                SAxisPlacement::Front
            } else {
                SAxisPlacement::Back
            },
            u_placement: if value & 0x02 == 0 { UAxisPlacement::Up } else { UAxisPlacement::Down },
            b_placement: if value & 0x04 == 0 {
                BAxisPlacement::Flip
            } else {
                BAxisPlacement::NoFlip
            },
            r_turn_num: if value & 0x08 == 0 { RAxisTurnNum::Single } else { RAxisTurnNum::Double },
            t_turn_num: if value & 0x10 == 0 { TAxisTurnNum::Single } else { TAxisTurnNum::Double },
            s_turn_num: if value & 0x20 == 0 { SAxisTurnNum::Single } else { SAxisTurnNum::Double },
            redundant_s_placement: if value & 0x40 == 0 {
                RedundantSAxisPlacement::Front
            } else {
                RedundantSAxisPlacement::Back
            },
            ik_solution_basis: if value & 0x80 == 0 {
                IkSolutionBasis::PreviousStep
            } else {
                IkSolutionBasis::Configuration
            },
        }
    }

    /// Convert configuration to raw u8 value
    #[must_use]
    pub const fn to_raw(self) -> u8 {
        let mut value = 0u8;
        if matches!(self.s_placement, SAxisPlacement::Back) {
            value |= 0x01;
        }
        if matches!(self.u_placement, UAxisPlacement::Down) {
            value |= 0x02;
        }
        if matches!(self.b_placement, BAxisPlacement::NoFlip) {
            value |= 0x04;
        }
        if matches!(self.r_turn_num, RAxisTurnNum::Double) {
            value |= 0x08;
        }
        if matches!(self.t_turn_num, TAxisTurnNum::Double) {
            value |= 0x10;
        }
        if matches!(self.s_turn_num, SAxisTurnNum::Double) {
            value |= 0x20;
        }
        if matches!(self.redundant_s_placement, RedundantSAxisPlacement::Back) {
            value |= 0x40;
        }
        if matches!(self.ik_solution_basis, IkSolutionBasis::Configuration) {
            value |= 0x80;
        }
        value
    }
}

impl ExtendedConfiguration {
    /// Create `ExtendedConfiguration` from raw u8 value
    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        Self {
            bit0: if value & 0x01 == 0 { LAxisTurnNum::Single } else { LAxisTurnNum::Double },
            bit1: if value & 0x02 == 0 { UAxisTurnNum::Single } else { UAxisTurnNum::Double },
            bit2: if value & 0x04 == 0 { BAxisTurnNum::Single } else { BAxisTurnNum::Double },
            bit3: if value & 0x08 == 0 { EAxisTurnNum::Single } else { EAxisTurnNum::Double },
            bit4: if value & 0x10 == 0 { WAxisTurnNum::Single } else { WAxisTurnNum::Double },
            bit5: value & 0x20 != 0, // Reserve
            bit6: value & 0x40 != 0, // Reserve
            bit7: value & 0x80 != 0, // Reserve
        }
    }

    /// Convert `ExtendedConfiguration` to raw u8 value
    #[must_use]
    pub const fn to_raw(self) -> u8 {
        let mut value = 0u8;
        if matches!(self.bit0, LAxisTurnNum::Double) {
            value |= 0x01;
        }
        if matches!(self.bit1, UAxisTurnNum::Double) {
            value |= 0x02;
        }
        if matches!(self.bit2, BAxisTurnNum::Double) {
            value |= 0x04;
        }
        if matches!(self.bit3, EAxisTurnNum::Double) {
            value |= 0x08;
        }
        if matches!(self.bit4, WAxisTurnNum::Double) {
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
    // X [mm]
    pub x: f32,
    // Y [mm]
    pub y: f32,
    // Z [mm]
    pub z: f32,
    // RX [deg]
    pub rx: f32,
    // RY [deg]
    pub ry: f32,
    // RZ [deg]
    pub rz: f32,
    // Tool number
    pub tool_no: u8,
    // User coordinate number
    pub user_coord_no: u8,
    // Configuration
    pub configuration: Configuration,
    // Extended configuration
    pub extended_configuration: ExtendedConfiguration,
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
        configuration: Configuration,
        extended_configuration: ExtendedConfiguration,
    ) -> Self {
        Self { x, y, z, rx, ry, rz, tool_no, user_coord_no, configuration, extended_configuration }
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

                // Use the configuration from the CartesianPosition
                data.extend_from_slice(&u32::from(cart.configuration.to_raw()).to_le_bytes());

                data.extend_from_slice(&u32::from(cart.tool_no).to_le_bytes());
                data.extend_from_slice(&u32::from(cart.user_coord_no).to_le_bytes());
                data.extend_from_slice(
                    &u32::from(cart.extended_configuration.to_raw()).to_le_bytes(),
                );

                // Convert coordinates to proper units
                #[allow(clippy::cast_possible_truncation)]
                {
                    data.extend_from_slice(&((cart.x * 1000.0) as i32).to_le_bytes()); // mm to μm
                    data.extend_from_slice(&((cart.y * 1000.0) as i32).to_le_bytes()); // mm to μm
                    data.extend_from_slice(&((cart.z * 1000.0) as i32).to_le_bytes()); // mm to μm
                    data.extend_from_slice(&((cart.rx * 10000.0) as i32).to_le_bytes()); // deg to 0.0001deg
                    data.extend_from_slice(&((cart.ry * 10000.0) as i32).to_le_bytes()); // deg to 0.0001deg
                    data.extend_from_slice(&((cart.rz * 10000.0) as i32).to_le_bytes());
                    // deg to 0.0001deg
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
                let _configuration = buf.get_u32_le();
                #[allow(clippy::cast_possible_truncation)]
                let _tool_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let _user_coord_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let _extended_configuration_raw = buf.get_u32_le() as u8;

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
                let configuration_raw = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let tool_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let user_coord_no = buf.get_u32_le() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let extended_configuration_raw = buf.get_u32_le() as u8;

                // Parse configuration and extended configuration
                let configuration = Configuration::from_raw(configuration_raw);
                let extended_configuration =
                    ExtendedConfiguration::from_raw(extended_configuration_raw);

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
                    configuration,
                    extended_configuration,
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
        let configuration = Configuration::from_raw(0);
        let extended_configuration = ExtendedConfiguration::from_raw(0);
        let position = CartesianPosition::new(
            100.0,
            200.0,
            300.0,
            0.0,
            0.0,
            0.0,
            1,
            0,
            configuration,
            extended_configuration,
        );
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
        let configuration = Configuration::from_raw(0);
        let extended_configuration = ExtendedConfiguration::from_raw(0);
        let position = Position::Cartesian(CartesianPosition::new(
            100.0,
            200.0,
            300.0,
            0.0,
            0.0,
            0.0,
            1,
            0,
            configuration,
            extended_configuration,
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
    fn test_configuration_serialization() {
        let configuration = Configuration::from_raw(0x55); // 01010101
        assert_eq!(configuration.s_placement, SAxisPlacement::Back);
        assert_eq!(configuration.u_placement, UAxisPlacement::Up);
        assert_eq!(configuration.b_placement, BAxisPlacement::NoFlip);
        assert_eq!(configuration.r_turn_num, RAxisTurnNum::Single);
        assert_eq!(configuration.t_turn_num, TAxisTurnNum::Double);
        assert_eq!(configuration.s_turn_num, SAxisTurnNum::Single);
        assert_eq!(configuration.redundant_s_placement, RedundantSAxisPlacement::Back);
        assert_eq!(configuration.ik_solution_basis, IkSolutionBasis::PreviousStep);

        let raw = configuration.to_raw();
        assert_eq!(raw, 0x55);
    }

    #[test]
    fn test_extended_configuration_serialization() {
        let extended_configuration = ExtendedConfiguration::from_raw(0x1F); // 00011111
        assert_eq!(extended_configuration.bit0, LAxisTurnNum::Double);
        assert_eq!(extended_configuration.bit1, UAxisTurnNum::Double);
        assert_eq!(extended_configuration.bit2, BAxisTurnNum::Double);
        assert_eq!(extended_configuration.bit3, EAxisTurnNum::Double);
        assert_eq!(extended_configuration.bit4, WAxisTurnNum::Double);
        assert!(!extended_configuration.bit5);
        assert!(!extended_configuration.bit6);
        assert!(!extended_configuration.bit7);

        let raw = extended_configuration.to_raw();
        assert_eq!(raw, 0x1F);
    }
}
