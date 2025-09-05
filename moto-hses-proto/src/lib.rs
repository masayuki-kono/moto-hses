//! moto-hses-proto - HSES (High Speed Ethernet Server) protocol implementation

pub mod alarm;
pub mod error;
pub mod message;
pub mod position;
pub mod status;
pub mod types;
pub mod variables;

// Re-export commonly used items for convenience
pub use alarm::{Alarm, AlarmAttribute, ReadAlarmData};
pub use error::ProtocolError;
pub use message::{
    HsesCommonHeader, HsesRequestMessage, HsesRequestSubHeader, HsesResponseMessage,
    HsesResponseSubHeader,
};
pub use position::{CartesianPosition, Position, PulsePosition};
pub use status::{Status, StatusData1, StatusData2, StatusWrapper};
pub use types::{
    Command, CoordinateSystem, CoordinateSystemType, Division, ReadCurrentPosition, ReadStatus,
    ReadStatusData1, ReadStatusData2, ReadVar, Service, VarType, Variable, VariableType, WriteVar, DEFAULT_PORT, FILE_PORT,
};
