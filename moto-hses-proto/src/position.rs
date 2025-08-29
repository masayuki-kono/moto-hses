//! Position data structures and operations

use bytes::Buf;
use crate::error::ProtocolError;
use crate::types::{CoordinateSystem, VariableType};

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

impl VariableType for Position {
    fn command_id() -> u16 { 0x7f }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.serialize()
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Position::deserialize(data)
    }
}
