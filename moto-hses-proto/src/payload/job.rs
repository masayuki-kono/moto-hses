//! Job information data structures and operations

use crate::error::ProtocolError;
use crate::payload::HsesPayload;

/// Executing job information data structure
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn serialize(
        &self,
        attribute: u8,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        match attribute {
            1 => {
                // Job name (32 bytes)
                let name_bytes = crate::encoding_utils::encode_string(&self.job_name, encoding);
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
                // Speed override value (4 bytes) - convert from % to 0.01%
                let raw_speed_override_value = self.speed_override_value * 100;
                data.extend_from_slice(&raw_speed_override_value.to_le_bytes());
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
    pub fn serialize_complete(
        &self,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        // Job name (32 bytes)
        let name_bytes = crate::encoding_utils::encode_string(&self.job_name, encoding);
        let mut padded_name = vec![0u8; 32];
        padded_name[..name_bytes.len().min(32)]
            .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
        data.extend_from_slice(&padded_name);

        // Line number (4 bytes)
        data.extend_from_slice(&self.line_number.to_le_bytes());

        // Step number (4 bytes)
        data.extend_from_slice(&self.step_number.to_le_bytes());

        // Speed override value (4 bytes) - convert from % to 0.01%
        let raw_speed_override_value = self.speed_override_value * 100;
        data.extend_from_slice(&raw_speed_override_value.to_le_bytes());

        Ok(data)
    }
}

impl Default for ExecutingJobInfo {
    fn default() -> Self {
        Self {
            job_name: "NO_JOB".to_string(),
            line_number: 0,
            step_number: 0,
            speed_override_value: 100, // 100% (will be converted to 10000 in 0.01% units)
        }
    }
}

impl ExecutingJobInfo {
    /// Deserialize job info data from response
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        if data.len() < 44 {
            return Err(ProtocolError::Deserialization("Insufficient data length".to_string()));
        }

        // Extract job name (32 bytes, null-terminated) and decode with specified encoding
        let name_end = data[0..32].iter().position(|&b| b == 0).unwrap_or(32);
        let name_bytes = &data[0..name_end];
        let job_name = crate::encoding_utils::decode_string_with_fallback(name_bytes, encoding);

        // Extract line number (4 bytes)
        let line_number = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);

        // Extract step number (4 bytes)
        let step_number = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);

        // Extract speed override value (4 bytes) and convert from 0.01% to %
        let raw_speed_override_value = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);
        let speed_override_value = raw_speed_override_value / 100;

        Ok(Self { job_name, line_number, step_number, speed_override_value })
    }

    /// Deserialize job info data from response for specific attribute
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize_attribute(
        data: &[u8],
        attribute: u8,
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        match attribute {
            1 => {
                // Job name (32 bytes)
                if data.len() < 32 {
                    return Err(ProtocolError::Deserialization(
                        "Insufficient data length for job name".to_string(),
                    ));
                }
                let name_end = data[0..32].iter().position(|&b| b == 0).unwrap_or(32);
                let name_bytes = &data[0..name_end];
                let job_name =
                    crate::encoding_utils::decode_string_with_fallback(name_bytes, encoding);
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
                // Speed override value (4 bytes) and convert from 0.01% to %
                if data.len() < 4 {
                    return Err(ProtocolError::Deserialization(
                        "Insufficient data length for speed override value".to_string(),
                    ));
                }
                let raw_speed_override_value =
                    u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let speed_override_value = raw_speed_override_value / 100;
                Ok(Self {
                    job_name: String::new(),
                    line_number: 0,
                    step_number: 0,
                    speed_override_value,
                })
            }
            _ => {
                // Default to complete deserialization
                Self::deserialize(data, encoding)
            }
        }
    }
}

impl HsesPayload for ExecutingJobInfo {
    fn serialize(&self, encoding: crate::encoding::TextEncoding) -> Result<Vec<u8>, ProtocolError> {
        self.serialize_complete(encoding)
    }

    fn deserialize(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Self, ProtocolError> {
        Self::deserialize(data, encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::TextEncoding;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_new() {
        let job_info = ExecutingJobInfo::new("TEST_JOB".to_string(), 10, 5, 80);

        assert_eq!(job_info.job_name, "TEST_JOB");
        assert_eq!(job_info.line_number, 10);
        assert_eq!(job_info.step_number, 5);
        assert_eq!(job_info.speed_override_value, 80);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
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
        let job_info = ExecutingJobInfo::new("TEST_JOB".to_string(), 10, 5, 80);

        let data = job_info.serialize_complete(TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 44); // 32 + 4 + 4 + 4

        // Check job name (first 32 bytes)
        let name_str = String::from_utf8_lossy(&data[0..32]);
        assert!(name_str.starts_with("TEST_JOB"));

        // Check line number (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[32], data[33], data[34], data[35]]), 10);

        // Check step number (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[36], data[37], data[38], data[39]]), 5);

        // Check speed override value (last 4 bytes) - should be 8000 (80 * 100)
        assert_eq!(u32::from_le_bytes([data[40], data[41], data[42], data[43]]), 8000);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_attribute() {
        let job_info = ExecutingJobInfo::new("TEST_JOB".to_string(), 10, 5, 80);

        // Test job name serialization
        let data = job_info.serialize(1, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 32);
        let name_str = String::from_utf8_lossy(&data[0..32]);
        assert!(name_str.starts_with("TEST_JOB"));

        // Test line number serialization
        let data = job_info.serialize(2, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 10);

        // Test step number serialization
        let data = job_info.serialize(3, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 5);

        // Test speed override value serialization
        let data = job_info.serialize(4, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 8000);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_invalid_attribute() {
        let job_info = ExecutingJobInfo::new("TEST_JOB".to_string(), 10, 5, 80);

        let result = job_info.serialize(99, TextEncoding::Utf8);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::InvalidAttribute));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize() {
        let original_job_info = ExecutingJobInfo::new("TEST_JOB".to_string(), 10, 5, 80);

        let serialized = original_job_info.serialize_complete(TextEncoding::Utf8).unwrap();
        let deserialized = ExecutingJobInfo::deserialize(&serialized, TextEncoding::Utf8).unwrap();

        assert_eq!(deserialized.job_name, original_job_info.job_name);
        assert_eq!(deserialized.line_number, original_job_info.line_number);
        assert_eq!(deserialized.step_number, original_job_info.step_number);
        assert_eq!(deserialized.speed_override_value, original_job_info.speed_override_value);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize_insufficient_data() {
        let short_data = vec![0u8; 10]; // Less than 44 bytes
        let result = ExecutingJobInfo::deserialize(&short_data, TextEncoding::Utf8);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::Deserialization(_)));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_deserialize_attribute() {
        let original_job_info = ExecutingJobInfo::new("TEST_JOB".to_string(), 10, 5, 80);

        // Test job name deserialization
        let name_data = original_job_info.serialize(1, TextEncoding::Utf8).unwrap();
        let deserialized =
            ExecutingJobInfo::deserialize_attribute(&name_data, 1, TextEncoding::Utf8).unwrap();
        assert_eq!(deserialized.job_name, "TEST_JOB");
        assert_eq!(deserialized.line_number, 0);
        assert_eq!(deserialized.step_number, 0);
        assert_eq!(deserialized.speed_override_value, 0);

        // Test line number deserialization
        let line_data = original_job_info.serialize(2, TextEncoding::Utf8).unwrap();
        let deserialized =
            ExecutingJobInfo::deserialize_attribute(&line_data, 2, TextEncoding::Utf8).unwrap();
        assert_eq!(deserialized.job_name, "");
        assert_eq!(deserialized.line_number, 10);
        assert_eq!(deserialized.step_number, 0);
        assert_eq!(deserialized.speed_override_value, 0);

        // Test step number deserialization
        let step_data = original_job_info.serialize(3, TextEncoding::Utf8).unwrap();
        let deserialized =
            ExecutingJobInfo::deserialize_attribute(&step_data, 3, TextEncoding::Utf8).unwrap();
        assert_eq!(deserialized.job_name, "");
        assert_eq!(deserialized.line_number, 0);
        assert_eq!(deserialized.step_number, 5);
        assert_eq!(deserialized.speed_override_value, 0);

        // Test speed override value deserialization
        let speed_data = original_job_info.serialize(4, TextEncoding::Utf8).unwrap();
        let deserialized =
            ExecutingJobInfo::deserialize_attribute(&speed_data, 4, TextEncoding::Utf8).unwrap();
        assert_eq!(deserialized.job_name, "");
        assert_eq!(deserialized.line_number, 0);
        assert_eq!(deserialized.step_number, 0);
        assert_eq!(deserialized.speed_override_value, 80);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_encoding_shift_jis() {
        // Create test job info with Japanese text
        let japanese_job_name = "テストジョブ";
        let job_info = ExecutingJobInfo::new(japanese_job_name.to_string(), 10, 5, 80);

        // Serialize with Shift_JIS encoding
        let data = job_info.serialize_complete(TextEncoding::ShiftJis).unwrap();

        // Deserialize with Shift_JIS encoding
        let deserialized_job_info =
            ExecutingJobInfo::deserialize(&data, TextEncoding::ShiftJis).unwrap();

        // Test Shift_JIS round-trip
        assert_eq!(deserialized_job_info.job_name, japanese_job_name);
        assert_eq!(deserialized_job_info.line_number, 10);
        assert_eq!(deserialized_job_info.step_number, 5);
        assert_eq!(deserialized_job_info.speed_override_value, 80);

        // Deserialize with UTF-8 encoding (should produce garbled text)
        let job_info_utf8 = ExecutingJobInfo::deserialize(&data, TextEncoding::Utf8).unwrap();

        // The UTF-8 decoding should produce garbled text for Japanese job name
        assert_ne!(job_info_utf8.job_name, japanese_job_name);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_executing_job_info_serialize_long_strings() {
        let long_job_name = "This is a very long job name that exceeds 32 bytes limit"; // Longer than 32 bytes
        let job_info = ExecutingJobInfo::new(long_job_name.to_string(), 10, 5, 80);

        let data = job_info.serialize_complete(TextEncoding::Utf8).unwrap();

        // Job name should be truncated to 32 bytes
        let name_str = String::from_utf8_lossy(&data[0..32]);
        assert!(name_str.len() <= 32);
        assert!(name_str.starts_with("This is a very long job name"));
    }
}
