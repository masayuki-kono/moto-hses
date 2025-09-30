//! I/O related commands (0x78)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Read I/O command (0x78)
#[derive(Debug, Clone)]
pub struct ReadIo {
    pub io_number: u16,
}

impl ReadIo {
    #[must_use]
    pub const fn new(io_number: u16) -> Self {
        Self { io_number }
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

/// Write I/O command (0x78)
#[derive(Debug, Clone)]
pub struct WriteIo {
    pub io_number: u16,
    pub value: bool,
}

impl WriteIo {
    #[must_use]
    pub const fn new(io_number: u16, value: bool) -> Self {
        Self { io_number, value }
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
