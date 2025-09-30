//! Command trait and related types for HSES protocol

use crate::error::ProtocolError;

/// Core trait for type-safe commands
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

/// Division types for HSES protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Division {
    Robot = 1,
    File = 2,
}

/// Service types for HSES protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    GetSingle = 0x0e,
    SetSingle = 0x10,
}
