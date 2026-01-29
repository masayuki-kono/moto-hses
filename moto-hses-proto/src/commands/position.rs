//! Position related commands (0x75)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Read current position command (0x75)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadCurrentPosition {
    pub control_group: u8,
}

impl ReadCurrentPosition {
    #[must_use]
    pub const fn new(control_group: u8) -> Self {
        Self { control_group }
    }
}

impl Command for ReadCurrentPosition {
    type Response = crate::payload::position::Position;

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
        0
    }

    fn service(&self) -> u8 {
        0x01 // Get_Attribute_All
    }
}
