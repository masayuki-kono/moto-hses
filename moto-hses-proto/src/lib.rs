//! moto-hses-proto - HSES (High Speed Ethernet Server) protocol implementation

pub mod error;
pub mod types;
pub mod position;
pub mod status;
pub mod message;
pub mod variables;
pub mod alarm;
#[cfg(test)]
pub mod tests;

// Re-export commonly used items for convenience
pub use error::ProtocolError;
pub use types::{
    Command, VariableType, VarType, Variable,
    Division, Service, CoordinateSystem, CoordinateSystemType,
    ReadVar, WriteVar, ReadStatus, ReadCurrentPosition,
    DEFAULT_PORT, FILE_PORT,
};
pub use position::{Position, PulsePosition, CartesianPosition};
pub use status::{Status, StatusWrapper};
pub use message::{HsesMessage, HsesHeader, HsesSubHeader};
pub use alarm::{Alarm, AlarmAttribute};
