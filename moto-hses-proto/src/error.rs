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
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("server error: {0}")]
    ServerError(String),
    #[error("invalid attribute")]
    InvalidAttribute,
    #[error("invalid service")]
    InvalidService,
    #[error("invalid command")]
    InvalidCommand,
}
