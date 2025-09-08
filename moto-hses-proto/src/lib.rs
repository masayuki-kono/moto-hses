//! moto-hses-proto - HSES (High Speed Ethernet Server) protocol implementation

pub mod alarm;
pub mod error;
pub mod job;
pub mod message;
pub mod position;
pub mod status;
pub mod types;
pub mod variables;

// Re-export commonly used items for convenience
pub use alarm::{Alarm, AlarmAttribute, ReadAlarmData};
pub use error::ProtocolError;
pub use job::{ExecutingJobInfo, JobInfoAttribute, ReadExecutingJobInfo, TaskType};
pub use message::{
    HsesCommonHeader, HsesRequestMessage, HsesRequestSubHeader, HsesResponseMessage,
    HsesResponseSubHeader,
};
pub use position::{CartesianPosition, Position, PulsePosition};
pub use status::{Status, StatusData1, StatusData2};
pub use types::{
    Command, CoordinateSystem, CoordinateSystemType, Division, ReadCurrentPosition, ReadIo,
    ReadStatus, ReadStatusData1, ReadStatusData2, ReadVar, Service, VariableType, WriteIo,
    WriteVar, FILE_CONTROL_PORT, ROBOT_CONTROL_PORT,
};
