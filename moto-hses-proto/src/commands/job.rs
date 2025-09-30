//! Job related commands (0x73)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Task types for job info reading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    MasterTask, // 1
    SubTask1,   // 2
    SubTask2,   // 3
    SubTask3,   // 4
    SubTask4,   // 5
    SubTask5,   // 6
    Invalid,
}

/// Command for reading executing job information (0x73)
#[derive(Debug, Clone)]
pub struct ReadExecutingJobInfo {
    pub instance: u16,
    pub attribute: u8,
}

impl ReadExecutingJobInfo {
    #[must_use]
    pub const fn new(instance: u16, attribute: u8) -> Self {
        Self { instance, attribute }
    }

    /// Validate instance range for job info reading
    #[must_use]
    pub const fn is_valid_instance(&self) -> bool {
        matches!(self.instance, 1..=6)
    }

    /// Get task type from instance
    #[must_use]
    pub const fn get_task_type(&self) -> TaskType {
        match self.instance {
            1 => TaskType::MasterTask,
            2 => TaskType::SubTask1,
            3 => TaskType::SubTask2,
            4 => TaskType::SubTask3,
            5 => TaskType::SubTask4,
            6 => TaskType::SubTask5,
            _ => TaskType::Invalid,
        }
    }

    /// Validate attribute range for job info reading
    #[must_use]
    pub const fn is_valid_attribute(&self) -> bool {
        matches!(self.attribute, 0..=4)
    }
}

impl Command for ReadExecutingJobInfo {
    type Response = crate::payload::job::ExecutingJobInfo;

    fn command_id() -> u16 {
        0x73
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // For 0x73 command, payload is typically empty
        // instance and attribute are specified in the sub-header
        Ok(vec![])
    }

    fn instance(&self) -> u16 {
        self.instance
    }

    fn attribute(&self) -> u8 {
        self.attribute
    }

    fn service(&self) -> u8 {
        if self.attribute == 0 {
            0x01 // Get_Attribute_All
        } else {
            0x0e // Get_Attribute_Single
        }
    }
}
