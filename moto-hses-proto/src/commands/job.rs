//! Job related commands (0x73, 0x86, 0x87)

use super::command_trait::Command;
use crate::encoding::TextEncoding;
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

/// Command for starting job execution (0x86)
#[derive(Debug, Clone)]
pub struct JobStartCommand;

impl JobStartCommand {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for JobStartCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for JobStartCommand {
    type Response = ();

    fn command_id() -> u16 {
        0x86
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Fixed value 1 as 32-bit integer (little-endian)
        Ok(vec![1, 0, 0, 0])
    }

    fn instance(&self) -> u16 {
        1 // Fixed according to specification
    }

    fn attribute(&self) -> u8 {
        1 // Fixed according to specification
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

/// Job select type (instance value for 0x87)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobSelectType {
    InExecution = 1,
    MasterTask0 = 10,
    MasterTask1 = 11,
    MasterTask2 = 12,
    MasterTask3 = 13,
    MasterTask4 = 14,
    MasterTask5 = 15,
}

/// Command for selecting job (0x87)
#[derive(Debug, Clone)]
pub struct JobSelectCommand {
    pub select_type: JobSelectType,
    pub job_name: String,
    pub line_number: u32,
    pub text_encoding: TextEncoding,
}

impl JobSelectCommand {
    #[must_use]
    pub const fn new(
        select_type: JobSelectType,
        job_name: String,
        line_number: u32,
        text_encoding: TextEncoding,
    ) -> Self {
        Self { select_type, job_name, line_number, text_encoding }
    }
}

impl Command for JobSelectCommand {
    type Response = ();

    fn command_id() -> u16 {
        0x87
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Job name: 32 bytes (8 x 4-byte integers)
        // Line number: 4 bytes (1 x 4-byte integer)
        // Total: 36 bytes

        // Encode job name using the client's configured text encoding
        let job_name_bytes =
            crate::encoding_utils::encode_string(&self.job_name, self.text_encoding);

        // Validate job name length (max 32 characters)
        if job_name_bytes.len() > 32 {
            return Err(ProtocolError::InvalidMessage("Job name exceeds 32 bytes".to_string()));
        }

        // Create 32-byte buffer for job name (pad with zeros)
        let mut payload = vec![0u8; 32];
        payload[..job_name_bytes.len()].copy_from_slice(&job_name_bytes);

        // Add line number as 4-byte little-endian integer
        payload.extend_from_slice(&self.line_number.to_le_bytes());

        Ok(payload)
    }

    fn instance(&self) -> u16 {
        self.select_type as u16
    }

    fn attribute(&self) -> u8 {
        0 // Fixed to 0(All attributes)
    }

    fn service(&self) -> u8 {
        0x02 // Set_Attribute_All
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_job_start_command_trait() {
        let command = JobStartCommand::new();

        assert_eq!(JobStartCommand::command_id(), 0x86);
        assert_eq!(command.instance(), 1);
        assert_eq!(command.attribute(), 1);
        assert_eq!(command.service(), 0x10);
    }

    #[test]
    fn test_job_start_command_serialize() {
        let command = JobStartCommand::new();
        let data = command.serialize().unwrap();
        assert_eq!(data, vec![1, 0, 0, 0]);
    }

    #[test]
    fn test_job_select_command_trait() {
        let command = JobSelectCommand::new(
            JobSelectType::InExecution,
            "TEST.JOB".to_string(),
            0,
            TextEncoding::Utf8,
        );

        assert_eq!(JobSelectCommand::command_id(), 0x87);
        assert_eq!(command.instance(), 1);
        assert_eq!(command.attribute(), 0);
        assert_eq!(command.service(), 0x02);
    }

    #[test]
    fn test_job_select_command_serialize() {
        let command = JobSelectCommand::new(
            JobSelectType::InExecution,
            "TEST.JOB".to_string(),
            123,
            TextEncoding::Utf8,
        );
        let data = command.serialize().unwrap();

        // Should be 36 bytes total
        assert_eq!(data.len(), 36);

        // First 8 bytes should be "TEST.JOB" (ASCII)
        assert_eq!(&data[0..8], b"TEST.JOB");

        // Remaining job name bytes should be zero-padded
        assert_eq!(&data[8..32], &[0u8; 24]);

        // Last 4 bytes should be line number (123) in little-endian
        assert_eq!(&data[32..36], &[123, 0, 0, 0]);
    }

    #[test]
    fn test_job_select_command_serialize_japanese() {
        let command = JobSelectCommand::new(
            JobSelectType::MasterTask0,
            "テスト.JOB".to_string(),
            456,
            TextEncoding::Utf8,
        );
        let data = command.serialize().unwrap();

        // Should be 36 bytes total
        assert_eq!(data.len(), 36);

        // Last 4 bytes should be line number (456) in little-endian
        assert_eq!(&data[32..36], &[200, 1, 0, 0]);
    }

    #[test]
    fn test_job_select_command_serialize_long_job_name() {
        let long_name = "A".repeat(33); // 33 characters, exceeds 32-byte limit
        let command =
            JobSelectCommand::new(JobSelectType::InExecution, long_name, 0, TextEncoding::Utf8);
        let result = command.serialize();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::InvalidMessage(_)));
    }

    #[test]
    fn test_job_select_command_serialize_shift_jis() {
        let command = JobSelectCommand::new(
            JobSelectType::InExecution,
            "テスト.JOB".to_string(),
            123,
            TextEncoding::ShiftJis,
        );
        let data = command.serialize().unwrap();

        // Should be 36 bytes total
        assert_eq!(data.len(), 36);

        // Last 4 bytes should be line number (123) in little-endian
        assert_eq!(&data[32..36], &[123, 0, 0, 0]);

        // First 32 bytes should contain Shift-JIS encoded job name
        // "テスト.JOB" in Shift-JIS is 12 bytes, so remaining 20 bytes should be zero-padded
        assert_eq!(&data[12..32], &[0u8; 20]);
    }
}
