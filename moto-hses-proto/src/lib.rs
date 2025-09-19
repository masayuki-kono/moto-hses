//! moto-hses-proto - HSES (High Speed Ethernet Server) protocol implementation

pub mod alarm;
pub mod encoding;
pub mod encoding_utils;
pub mod error;
pub mod file;
pub mod job;
pub mod message;
pub mod position;
pub mod status;
pub mod types;
pub mod variables;

// Re-export commonly used items for convenience
pub use alarm::{Alarm, AlarmAttribute, ReadAlarmData};
pub use encoding::TextEncoding;
pub use error::ProtocolError;
pub use file::response::{parse_file_content, parse_file_list};
pub use file::{DeleteFile, ReadFileList, ReceiveFile, SendFile};
pub use job::{ExecutingJobInfo, JobInfoAttribute, ReadExecutingJobInfo, TaskType};
pub use message::{
    HsesCommonHeader, HsesRequestMessage, HsesRequestSubHeader, HsesResponseMessage,
    HsesResponseSubHeader,
};
pub use position::{CartesianPosition, Position, PulsePosition};
pub use status::{Status, StatusData1, StatusData2};
pub use types::{
    Command, CoordinateSystem, CoordinateSystemType, Division, FILE_CONTROL_PORT, HoldServoControl,
    HoldServoType, HoldServoValue, ROBOT_CONTROL_PORT, ReadCurrentPosition, ReadIo, ReadStatus,
    ReadStatusData1, ReadStatusData2, ReadVar, Service, VariableType, WriteIo, WriteVar,
};
