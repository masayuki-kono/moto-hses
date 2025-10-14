//! Error handling for HSES protocol

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("buffer underflow")]
    Underflow,
    #[error("invalid header")]
    InvalidHeader,
    #[error("unknown command 0x{0:04X}")]
    UnknownCommand(u16),
    #[error("unsupported operation")]
    Unsupported,
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("deserialization error: {0}")]
    Deserialization(String),
    #[error("invalid variable type")]
    InvalidVariableType,
    #[error("invalid coordinate system type")]
    InvalidCoordinateSystemType,
    #[error("position data error: {0}")]
    PositionError(String),
    #[error("file operation error: {0}")]
    FileError(String),
    #[error("system info error: {0}")]
    SystemInfoError(String),
    /// Invalid message content or payload (e.g., invalid count, range overflow, malformed data)
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("server error: {0}")]
    ServerError(String),
    /// Invalid attribute value in message header
    #[error("invalid attribute")]
    InvalidAttribute,
    /// Invalid service code in message header
    #[error("invalid service")]
    InvalidService,
    /// Invalid command ID
    #[error("invalid command")]
    InvalidCommand,
    /// Invalid instance parameter (e.g., register number, I/O number out of range)
    #[error("invalid instance: {0}")]
    InvalidInstance(String),
}
