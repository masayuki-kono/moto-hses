//! Register related commands (0x79)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Read register command (0x79)
#[derive(Debug, Clone)]
pub struct ReadRegister {
    pub register_number: u16,
}

impl ReadRegister {
    #[must_use]
    pub const fn new(register_number: u16) -> Self {
        Self { register_number }
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

/// Write register command (0x79)
#[derive(Debug, Clone)]
pub struct WriteRegister {
    pub register_number: u16,
    pub value: i16,
}

impl WriteRegister {
    #[must_use]
    pub const fn new(register_number: u16, value: i16) -> Self {
        Self { register_number, value }
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
