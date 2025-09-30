//! Payload data type definitions for HSES protocol

pub mod alarm;
pub mod job;
pub mod payload_trait;
pub mod position;
pub mod status;
pub mod variable;

// Re-export commonly used payload types
pub use alarm::Alarm;
pub use job::ExecutingJobInfo;
pub use payload_trait::HsesPayload;
pub use position::{
    CartesianPosition, ControlGroupPositionType, CoordinateSystemType, Position, PulsePosition,
};
pub use status::{Status, StatusData1, StatusData2};
