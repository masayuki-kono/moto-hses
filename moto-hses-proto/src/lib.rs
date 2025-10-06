//! moto-hses-proto - HSES (High Speed Ethernet Server) protocol implementation

pub mod commands;
pub mod constants;
pub mod encoding;
pub mod encoding_utils;
pub mod error;
pub mod message;
pub mod payload;

// Re-export commonly used items for convenience
pub use commands::{
    AlarmAttribute, AlarmReset, Command, CycleMode, CycleModeSwitchingCommand, DeleteFile,
    Division, HoldServoControl, HoldServoType, HoldServoValue, ReadAlarmData, ReadAlarmHistory,
    ReadCurrentPosition, ReadExecutingJobInfo, ReadFileList, ReadIo, ReadRegister, ReadStatus,
    ReadStatusData1, ReadStatusData2, ReadVar, ReceiveFile, SendFile, Service, VariableCommandId,
    WriteIo, WriteRegister, WriteVar,
};
pub use constants::{FILE_CONTROL_PORT, ROBOT_CONTROL_PORT};
pub use encoding::TextEncoding;
pub use error::ProtocolError;
pub use message::{
    HsesCommonHeader, HsesRequestMessage, HsesRequestSubHeader, HsesResponseMessage,
    HsesResponseSubHeader,
};
pub use payload::{
    Alarm, CartesianPosition, ExecutingJobInfo, HsesPayload, Position, PulsePosition, Status,
    StatusData1, StatusData2,
};
