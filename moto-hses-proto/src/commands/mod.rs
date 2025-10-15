//! HSES protocol commands module
//!
//! This module contains all command implementations organized by functionality.

pub mod alarm;
pub mod command_trait;
pub mod cycle_mode;
pub mod file;
pub mod io;
pub mod job;
pub mod position;
pub mod register;
pub mod servo;
pub mod status;
pub mod variable;

// Re-export core traits and common types
pub use alarm::{AlarmAttribute, AlarmReset, ReadAlarmData, ReadAlarmHistory};
pub use command_trait::{Command, Division, Service};
pub use cycle_mode::{CycleMode, CycleModeSwitchingCommand};
pub use file::response::{parse_file_content, parse_file_list};
pub use file::{DeleteFile, ReadFileList, ReceiveFile, SendFile};
pub use io::{ReadIo, ReadMultipleIo, WriteIo, WriteMultipleIo};
pub use job::{JobSelectCommand, JobSelectType, JobStartCommand, ReadExecutingJobInfo, TaskType};
pub use position::ReadCurrentPosition;
pub use register::{ReadMultipleRegisters, ReadRegister, WriteMultipleRegisters, WriteRegister};
pub use servo::{HoldServoControl, HoldServoType, HoldServoValue};
pub use status::{ReadStatus, ReadStatusData1, ReadStatusData2};
pub use variable::{
    ReadMultipleByteVariables, ReadMultipleIntegerVariables, ReadVar, VariableCommandId,
    WriteMultipleByteVariables, WriteMultipleIntegerVariables, WriteVar,
};
