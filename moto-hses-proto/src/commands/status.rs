//! Status related commands (0x72)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Read status command (0x72) - reads all status data
#[derive(Debug, Clone)]
pub struct ReadStatus;

impl Default for ReadStatus {
    fn default() -> Self {
        Self
    }
}

impl ReadStatus {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Command for ReadStatus {
    type Response = crate::payload::status::Status;

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

/// Read status data 1 command (0x72)
#[derive(Debug, Clone)]
pub struct ReadStatusData1;

impl Default for ReadStatusData1 {
    fn default() -> Self {
        Self
    }
}

impl ReadStatusData1 {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Command for ReadStatusData1 {
    type Response = crate::payload::status::StatusData1;

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

/// Read status data 2 command (0x72)
#[derive(Debug, Clone)]
pub struct ReadStatusData2;

impl Default for ReadStatusData2 {
    fn default() -> Self {
        Self
    }
}

impl ReadStatusData2 {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Command for ReadStatusData2 {
    type Response = crate::payload::status::StatusData2;

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
