//! Job information data structures and operations

use crate::error::ProtocolError;
use crate::types::{Command, VariableType};

/// Executing job information data structure
#[derive(Debug, Clone)]
pub struct ExecutingJobInfo {
    pub job_name: String,
    pub line_number: u32,
    pub step_number: u32,
    pub speed_override_value: u32,
}

impl ExecutingJobInfo {
    #[must_use]
    pub const fn new(
        job_name: String,
        line_number: u32,
        step_number: u32,
        speed_override_value: u32,
    ) -> Self {
        Self { job_name, line_number, step_number, speed_override_value }
    }

    /// Serialize job info data for response
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn serialize(&self, attribute: u8) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        match attribute {
            1 => {
                // Job name (32 bytes)
                let name_bytes = self.job_name.as_bytes();
                let mut padded_name = vec![0u8; 32];
                padded_name[..name_bytes.len().min(32)]
                    .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
                data.extend_from_slice(&padded_name);
            }
            2 => {
                // Line number (4 bytes)
                data.extend_from_slice(&self.line_number.to_le_bytes());
            }
            3 => {
                // Step number (4 bytes)
                data.extend_from_slice(&self.step_number.to_le_bytes());
            }
            4 => {
                // Speed override value (4 bytes)
                data.extend_from_slice(&self.speed_override_value.to_le_bytes());
            }
            _ => {
                return Err(ProtocolError::InvalidAttribute);
            }
        }

        Ok(data)
    }

    /// Serialize complete job info data (all attributes)
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn serialize_complete(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        // Job name (32 bytes)
        let name_bytes = self.job_name.as_bytes();
        let mut padded_name = vec![0u8; 32];
        padded_name[..name_bytes.len().min(32)]
            .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
        data.extend_from_slice(&padded_name);

        // Line number (4 bytes)
        data.extend_from_slice(&self.line_number.to_le_bytes());

        // Step number (4 bytes)
        data.extend_from_slice(&self.step_number.to_le_bytes());

        // Speed override value (4 bytes)
        data.extend_from_slice(&self.speed_override_value.to_le_bytes());

        Ok(data)
    }
}

impl Default for ExecutingJobInfo {
    fn default() -> Self {
        Self {
            job_name: "NO_JOB".to_string(),
            line_number: 0,
            step_number: 0,
            speed_override_value: 100,
        }
    }
}

impl ExecutingJobInfo {
    /// Deserialize job info data from response
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 44 {
            return Err(ProtocolError::Deserialization("Insufficient data length".to_string()));
        }

        // Extract job name (32 bytes, null-terminated)
        let name_end = data[0..32].iter().position(|&b| b == 0).unwrap_or(32);
        let job_name = String::from_utf8_lossy(&data[0..name_end]).to_string();

        // Extract line number (4 bytes)
        let line_number = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);

        // Extract step number (4 bytes)
        let step_number = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);

        // Extract speed override value (4 bytes)
        let speed_override_value = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);

        Ok(Self { job_name, line_number, step_number, speed_override_value })
    }

    /// Deserialize job info data from response for specific attribute
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize_attribute(data: &[u8], attribute: u8) -> Result<Self, ProtocolError> {
        match attribute {
            1 => {
                // Job name (32 bytes)
                if data.len() < 32 {
                    return Err(ProtocolError::Deserialization(
                        "Insufficient data length for job name".to_string(),
                    ));
                }
                let name_end = data[0..32].iter().position(|&b| b == 0).unwrap_or(32);
                let job_name = String::from_utf8_lossy(&data[0..name_end]).to_string();
                Ok(Self { job_name, line_number: 0, step_number: 0, speed_override_value: 0 })
            }
            2 => {
                // Line number (4 bytes)
                if data.len() < 4 {
                    return Err(ProtocolError::Deserialization(
                        "Insufficient data length for line number".to_string(),
                    ));
                }
                let line_number = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Self {
                    job_name: String::new(),
                    line_number,
                    step_number: 0,
                    speed_override_value: 0,
                })
            }
            3 => {
                // Step number (4 bytes)
                if data.len() < 4 {
                    return Err(ProtocolError::Deserialization(
                        "Insufficient data length for step number".to_string(),
                    ));
                }
                let step_number = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Self {
                    job_name: String::new(),
                    line_number: 0,
                    step_number,
                    speed_override_value: 0,
                })
            }
            4 => {
                // Speed override value (4 bytes)
                if data.len() < 4 {
                    return Err(ProtocolError::Deserialization(
                        "Insufficient data length for speed override value".to_string(),
                    ));
                }
                let speed_override_value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Self {
                    job_name: String::new(),
                    line_number: 0,
                    step_number: 0,
                    speed_override_value,
                })
            }
            _ => {
                // Default to complete deserialization
                Self::deserialize(data)
            }
        }
    }
}

impl VariableType for ExecutingJobInfo {
    fn command_id() -> u16 {
        0x73
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.serialize_complete()
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Self::deserialize(data)
    }
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

impl Command for ReadExecutingJobInfo {
    type Response = ExecutingJobInfo;

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

/// Job information attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobInfoAttribute {
    All = 0,
    JobName = 1,
    LineNumber = 2,
    StepNumber = 3,
    SpeedOverrideValue = 4,
}

impl From<u8> for JobInfoAttribute {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::JobName,
            2 => Self::LineNumber,
            3 => Self::StepNumber,
            4 => Self::SpeedOverrideValue,
            _ => Self::All,
        }
    }
}

/// Predefined job info for testing
pub mod test_job_info {
    use super::ExecutingJobInfo;

    #[must_use]
    pub fn default_job() -> ExecutingJobInfo {
        ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100)
    }

    #[must_use]
    pub fn running_job() -> ExecutingJobInfo {
        ExecutingJobInfo::new("PRODUCTION.JOB".to_string(), 2500, 15, 80)
    }

    #[must_use]
    pub fn paused_job() -> ExecutingJobInfo {
        ExecutingJobInfo::new("MAINTENANCE.JOB".to_string(), 500, 3, 50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executing_job_info_new() {
        let job_info = ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100);

        assert_eq!(job_info.job_name, "TEST.JOB");
        assert_eq!(job_info.line_number, 1000);
        assert_eq!(job_info.step_number, 1);
        assert_eq!(job_info.speed_override_value, 100);
    }

    #[test]
    fn test_executing_job_info_default() {
        let job_info = ExecutingJobInfo::default();

        assert_eq!(job_info.job_name, "NO_JOB");
        assert_eq!(job_info.line_number, 0);
        assert_eq!(job_info.step_number, 0);
        assert_eq!(job_info.speed_override_value, 100);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_complete() {
        let job_info = ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100);

        let data = job_info.serialize_complete().unwrap();
        assert_eq!(data.len(), 44); // 32 + 4 + 4 + 4

        // Check job name (first 32 bytes)
        let name_str = String::from_utf8_lossy(&data[0..32]);
        assert!(name_str.starts_with("TEST.JOB"));

        // Check line number (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[32], data[33], data[34], data[35]]), 1000);

        // Check step number (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[36], data[37], data[38], data[39]]), 1);

        // Check speed override value (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[40], data[41], data[42], data[43]]), 100);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_attribute() {
        let job_info = ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100);

        // Test job name serialization
        let data = job_info.serialize(1).unwrap();
        assert_eq!(data.len(), 32);
        let name_str = String::from_utf8_lossy(&data[0..32]);
        assert!(name_str.starts_with("TEST.JOB"));

        // Test line number serialization
        let data = job_info.serialize(2).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 1000);

        // Test step number serialization
        let data = job_info.serialize(3).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 1);

        // Test speed override value serialization
        let data = job_info.serialize(4).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 100);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_invalid_attribute() {
        let job_info = ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100);

        let result = job_info.serialize(99);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::InvalidAttribute));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize() {
        let original_job_info = ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100);

        let serialized = original_job_info.serialize_complete().unwrap();
        let deserialized = ExecutingJobInfo::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.job_name, original_job_info.job_name);
        assert_eq!(deserialized.line_number, original_job_info.line_number);
        assert_eq!(deserialized.step_number, original_job_info.step_number);
        assert_eq!(deserialized.speed_override_value, original_job_info.speed_override_value);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize_insufficient_data() {
        let short_data = vec![0u8; 10]; // Less than 44 bytes
        let result = ExecutingJobInfo::deserialize(&short_data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::Deserialization(_)));
    }

    #[test]
    fn test_read_executing_job_info_new() {
        let command = ReadExecutingJobInfo::new(1, 0);
        assert_eq!(command.instance, 1);
        assert_eq!(command.attribute, 0);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_executing_job_info_command_trait() {
        let command = ReadExecutingJobInfo::new(1, 1);

        assert_eq!(ReadExecutingJobInfo::command_id(), 0x73);
        assert_eq!(command.instance(), 1);
        assert_eq!(command.attribute(), 1);

        let serialized = command.serialize().unwrap();
        assert_eq!(serialized.len(), 0); // Empty payload for 0x73
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_variable_type_trait() {
        assert_eq!(ExecutingJobInfo::command_id(), 0x73);

        let job_info = ExecutingJobInfo::new("TEST.JOB".to_string(), 1000, 1, 100);

        let serialized = job_info.serialize_complete().unwrap();
        assert_eq!(serialized.len(), 44);

        let deserialized = ExecutingJobInfo::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.job_name, "TEST.JOB");
    }

    #[test]
    fn test_read_executing_job_info_instance_validation() {
        // Valid instances
        assert!(ReadExecutingJobInfo::new(1, 1).is_valid_instance());
        assert!(ReadExecutingJobInfo::new(2, 1).is_valid_instance());
        assert!(ReadExecutingJobInfo::new(3, 1).is_valid_instance());
        assert!(ReadExecutingJobInfo::new(4, 1).is_valid_instance());
        assert!(ReadExecutingJobInfo::new(5, 1).is_valid_instance());
        assert!(ReadExecutingJobInfo::new(6, 1).is_valid_instance());

        // Invalid instances
        assert!(!ReadExecutingJobInfo::new(0, 1).is_valid_instance());
        assert!(!ReadExecutingJobInfo::new(7, 1).is_valid_instance());
        assert!(!ReadExecutingJobInfo::new(100, 1).is_valid_instance());
    }

    #[test]
    fn test_read_executing_job_info_attribute_validation() {
        // Valid attributes
        assert!(ReadExecutingJobInfo::new(1, 0).is_valid_attribute());
        assert!(ReadExecutingJobInfo::new(1, 1).is_valid_attribute());
        assert!(ReadExecutingJobInfo::new(1, 2).is_valid_attribute());
        assert!(ReadExecutingJobInfo::new(1, 3).is_valid_attribute());
        assert!(ReadExecutingJobInfo::new(1, 4).is_valid_attribute());

        // Invalid attributes
        assert!(!ReadExecutingJobInfo::new(1, 5).is_valid_attribute());
        assert!(!ReadExecutingJobInfo::new(1, 99).is_valid_attribute());
    }

    #[test]
    fn test_read_executing_job_info_task_type_detection() {
        // Master task
        assert_eq!(ReadExecutingJobInfo::new(1, 1).get_task_type(), TaskType::MasterTask);

        // Sub tasks
        assert_eq!(ReadExecutingJobInfo::new(2, 1).get_task_type(), TaskType::SubTask1);
        assert_eq!(ReadExecutingJobInfo::new(3, 1).get_task_type(), TaskType::SubTask2);
        assert_eq!(ReadExecutingJobInfo::new(4, 1).get_task_type(), TaskType::SubTask3);
        assert_eq!(ReadExecutingJobInfo::new(5, 1).get_task_type(), TaskType::SubTask4);
        assert_eq!(ReadExecutingJobInfo::new(6, 1).get_task_type(), TaskType::SubTask5);

        // Invalid instances
        assert_eq!(ReadExecutingJobInfo::new(0, 1).get_task_type(), TaskType::Invalid);
        assert_eq!(ReadExecutingJobInfo::new(7, 1).get_task_type(), TaskType::Invalid);
    }

    #[test]
    fn test_job_info_attribute_from_u8() {
        assert_eq!(JobInfoAttribute::from(0), JobInfoAttribute::All);
        assert_eq!(JobInfoAttribute::from(1), JobInfoAttribute::JobName);
        assert_eq!(JobInfoAttribute::from(2), JobInfoAttribute::LineNumber);
        assert_eq!(JobInfoAttribute::from(3), JobInfoAttribute::StepNumber);
        assert_eq!(JobInfoAttribute::from(4), JobInfoAttribute::SpeedOverrideValue);
        assert_eq!(JobInfoAttribute::from(99), JobInfoAttribute::All); // Default
    }

    #[test]
    fn test_test_job_info() {
        let default_job = test_job_info::default_job();
        assert_eq!(default_job.job_name, "TEST.JOB");
        assert_eq!(default_job.line_number, 1000);
        assert_eq!(default_job.step_number, 1);
        assert_eq!(default_job.speed_override_value, 100);

        let running_job = test_job_info::running_job();
        assert_eq!(running_job.job_name, "PRODUCTION.JOB");
        assert_eq!(running_job.line_number, 2500);
        assert_eq!(running_job.step_number, 15);
        assert_eq!(running_job.speed_override_value, 80);

        let paused_job = test_job_info::paused_job();
        assert_eq!(paused_job.job_name, "MAINTENANCE.JOB");
        assert_eq!(paused_job.line_number, 500);
        assert_eq!(paused_job.step_number, 3);
        assert_eq!(paused_job.speed_override_value, 50);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_long_job_name() {
        let long_job_name = "This is a very long job name that exceeds 32 bytes limit"; // Longer than 32 bytes

        let job_info = ExecutingJobInfo::new(long_job_name.to_string(), 1000, 1, 100);

        let data = job_info.serialize_complete().unwrap();

        // Job name should be truncated to 32 bytes
        let name_str = String::from_utf8_lossy(&data[0..32]);
        assert!(name_str.len() <= 32);
        assert!(name_str.starts_with("This is a very long job"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize_attribute() {
        // Test job name deserialization
        let job_name_data = b"TEST.JOB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let job_info = ExecutingJobInfo::deserialize_attribute(job_name_data, 1).unwrap();
        assert_eq!(job_info.job_name, "TEST.JOB");
        assert_eq!(job_info.line_number, 0);
        assert_eq!(job_info.step_number, 0);
        assert_eq!(job_info.speed_override_value, 0);

        // Test line number deserialization
        let line_number_data = [232, 3, 0, 0]; // 1000 in little-endian
        let job_info = ExecutingJobInfo::deserialize_attribute(&line_number_data, 2).unwrap();
        assert_eq!(job_info.job_name, "");
        assert_eq!(job_info.line_number, 1000);
        assert_eq!(job_info.step_number, 0);
        assert_eq!(job_info.speed_override_value, 0);

        // Test step number deserialization
        let step_number_data = [1, 0, 0, 0]; // 1 in little-endian
        let job_info = ExecutingJobInfo::deserialize_attribute(&step_number_data, 3).unwrap();
        assert_eq!(job_info.job_name, "");
        assert_eq!(job_info.line_number, 0);
        assert_eq!(job_info.step_number, 1);
        assert_eq!(job_info.speed_override_value, 0);

        // Test speed override value deserialization
        let speed_override_data = [100, 0, 0, 0]; // 100 in little-endian
        let job_info = ExecutingJobInfo::deserialize_attribute(&speed_override_data, 4).unwrap();
        assert_eq!(job_info.job_name, "");
        assert_eq!(job_info.line_number, 0);
        assert_eq!(job_info.step_number, 0);
        assert_eq!(job_info.speed_override_value, 100);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize_attribute_insufficient_data() {
        // Test insufficient data for job name
        let short_data = vec![0u8; 10];
        let result = ExecutingJobInfo::deserialize_attribute(&short_data, 1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::Deserialization(_)));

        // Test insufficient data for line number
        let short_data = vec![0u8; 2];
        let result = ExecutingJobInfo::deserialize_attribute(&short_data, 2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::Deserialization(_)));
    }
}
