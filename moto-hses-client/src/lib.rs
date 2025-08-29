//! moto-hses-client - HSES (High Speed Ethernet Server) client implementation

pub mod types;
pub mod connection;
pub mod protocol;
pub mod convenience;
#[cfg(test)]
pub mod tests;

// Re-export main types for convenience
pub use types::{HsesClient, ClientConfig, ClientError};

// Re-export protocol types that are commonly used
pub use moto_hses_proto::{
    VariableType, Position, Status, CoordinateSystemType
};