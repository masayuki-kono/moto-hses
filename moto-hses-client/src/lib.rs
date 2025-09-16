//! moto-hses-client - HSES (High Speed Ethernet Server) client implementation

#[macro_use]
extern crate log;

pub mod connection;
pub mod convenience;
pub mod protocol;
pub mod types;

// Re-export main types for convenience
pub use types::{ClientConfig, ClientError, HsesClient};

// Re-export protocol types that are commonly used
pub use moto_hses_proto::{
    Alarm, CoordinateSystemType, ExecutingJobInfo, Position, Status, VariableType,
};
