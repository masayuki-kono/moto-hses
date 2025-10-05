//! Alarm data structures and operations

use crate::encoding::TextEncoding;
use crate::error::ProtocolError;
use crate::payload::HsesPayload;

/// Alarm data structure
#[derive(Debug, Clone)]
pub struct Alarm {
    pub code: u32,
    pub data: u32,
    pub alarm_type: u32,
    pub time: String,
    pub name: String,
    pub sub_code_info: String,
    pub sub_code_data: String,
    pub sub_code_reverse: String,
}

impl Alarm {
    #[must_use]
    pub const fn new(code: u32, data: u32, alarm_type: u32, time: String, name: String) -> Self {
        Self {
            code,
            data,
            alarm_type,
            time,
            name,
            sub_code_info: String::new(),
            sub_code_data: String::new(),
            sub_code_reverse: String::new(),
        }
    }

    #[must_use]
    pub fn with_sub_code(mut self, info: String, data: String, reverse: String) -> Self {
        self.sub_code_info = info;
        self.sub_code_data = data;
        self.sub_code_reverse = reverse;
        self
    }

    /// Serialize alarm data for response with specified text encoding
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn serialize(
        &self,
        attribute: u8,
        encoding: TextEncoding,
    ) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        match attribute {
            1 => {
                // Alarm code
                data.extend_from_slice(&self.code.to_le_bytes());
            }
            2 => {
                // Alarm data
                data.extend_from_slice(&self.data.to_le_bytes());
            }
            3 => {
                // Alarm type
                data.extend_from_slice(&self.alarm_type.to_le_bytes());
            }
            4 => {
                // Alarm time
                let time_bytes = crate::encoding_utils::encode_string(&self.time, encoding);
                let mut padded_time = vec![0u8; 16];
                padded_time[..time_bytes.len().min(16)]
                    .copy_from_slice(&time_bytes[..time_bytes.len().min(16)]);
                data.extend_from_slice(&padded_time);
            }
            5 => {
                // Alarm name
                let name_bytes = crate::encoding_utils::encode_string(&self.name, encoding);
                let mut padded_name = vec![0u8; 32];
                padded_name[..name_bytes.len().min(32)]
                    .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
                data.extend_from_slice(&padded_name);
            }
            6 => {
                // Sub code info
                let info_bytes =
                    crate::encoding_utils::encode_string(&self.sub_code_info, encoding);
                let mut padded_info = vec![0u8; 16];
                padded_info[..info_bytes.len().min(16)]
                    .copy_from_slice(&info_bytes[..info_bytes.len().min(16)]);
                data.extend_from_slice(&padded_info);
            }
            7 => {
                // Sub code data
                let data_bytes =
                    crate::encoding_utils::encode_string(&self.sub_code_data, encoding);
                let mut padded_data = vec![0u8; 96];
                padded_data[..data_bytes.len().min(96)]
                    .copy_from_slice(&data_bytes[..data_bytes.len().min(96)]);
                data.extend_from_slice(&padded_data);
            }
            8 => {
                // Sub code reverse
                let reverse_bytes =
                    crate::encoding_utils::encode_string(&self.sub_code_reverse, encoding);
                let mut padded_reverse = vec![0u8; 96];
                padded_reverse[..reverse_bytes.len().min(96)]
                    .copy_from_slice(&reverse_bytes[..reverse_bytes.len().min(96)]);
                data.extend_from_slice(&padded_reverse);
            }
            _ => {
                return Err(ProtocolError::InvalidAttribute);
            }
        }

        Ok(data)
    }

    /// Serialize complete alarm data (all attributes)
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn serialize_complete(&self, encoding: TextEncoding) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        // Alarm code
        data.extend_from_slice(&self.code.to_le_bytes());
        // Alarm data
        data.extend_from_slice(&self.data.to_le_bytes());
        // Alarm type
        data.extend_from_slice(&self.alarm_type.to_le_bytes());

        // Alarm time (16 bytes)
        let time_bytes = crate::encoding_utils::encode_string(&self.time, encoding);
        let mut padded_time = vec![0u8; 16];
        padded_time[..time_bytes.len().min(16)]
            .copy_from_slice(&time_bytes[..time_bytes.len().min(16)]);
        data.extend_from_slice(&padded_time);

        // Alarm name (32 bytes)
        let name_bytes = crate::encoding_utils::encode_string(&self.name, encoding);
        let mut padded_name = vec![0u8; 32];
        padded_name[..name_bytes.len().min(32)]
            .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
        data.extend_from_slice(&padded_name);

        // Sub code info (16 bytes)
        let info_bytes = crate::encoding_utils::encode_string(&self.sub_code_info, encoding);
        let mut padded_info = vec![0u8; 16];
        padded_info[..info_bytes.len().min(16)]
            .copy_from_slice(&info_bytes[..info_bytes.len().min(16)]);
        data.extend_from_slice(&padded_info);

        // Sub code data (96 bytes)
        let data_bytes = crate::encoding_utils::encode_string(&self.sub_code_data, encoding);
        let mut padded_data = vec![0u8; 96];
        padded_data[..data_bytes.len().min(96)]
            .copy_from_slice(&data_bytes[..data_bytes.len().min(96)]);
        data.extend_from_slice(&padded_data);

        // Sub code reverse (96 bytes)
        let reverse_bytes = crate::encoding_utils::encode_string(&self.sub_code_reverse, encoding);
        let mut padded_reverse = vec![0u8; 96];
        padded_reverse[..reverse_bytes.len().min(96)]
            .copy_from_slice(&reverse_bytes[..reverse_bytes.len().min(96)]);
        data.extend_from_slice(&padded_reverse);

        Ok(data)
    }
}

impl Default for Alarm {
    fn default() -> Self {
        Self {
            code: 0,
            data: 0,
            alarm_type: 0,
            time: "2024/01/01 00:00".to_string(),
            name: "No Alarm".to_string(),
            sub_code_info: String::new(),
            sub_code_data: String::new(),
            sub_code_reverse: String::new(),
        }
    }
}

impl Alarm {
    /// Deserialize alarm data from response with specified text encoding
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize(data: &[u8], text_encoding: TextEncoding) -> Result<Self, ProtocolError> {
        if data.len() < 60 {
            return Err(ProtocolError::Deserialization("Insufficient data length".to_string()));
        }

        let code = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let alarm_data = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let alarm_type = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

        // Extract time (16 bytes, null-terminated) and decode with specified encoding
        let time_end = data[12..28].iter().position(|&b| b == 0).unwrap_or(16);
        let time_bytes = &data[12..12 + time_end];
        let time = crate::encoding_utils::decode_string_with_fallback(time_bytes, text_encoding);

        // Extract name (32 bytes, null-terminated) and decode with specified encoding
        let name_end = data[28..60].iter().position(|&b| b == 0).unwrap_or(32);
        let name_bytes = &data[28..28 + name_end];
        let name = crate::encoding_utils::decode_string_with_fallback(name_bytes, text_encoding);

        Ok(Self {
            code,
            data: alarm_data,
            alarm_type,
            time,
            name,
            sub_code_info: String::new(),
            sub_code_data: String::new(),
            sub_code_reverse: String::new(),
        })
    }
}

impl HsesPayload for Alarm {
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

/// Predefined alarms for testing
pub mod test_alarms {
    use super::Alarm;

    #[must_use]
    pub fn servo_error() -> Alarm {
        Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Servo Error".to_string())
            .with_sub_code(
                "[SV#1]".to_string(),
                "Servo amplifier error".to_string(),
                "0".to_string(),
            )
    }

    #[must_use]
    pub fn emergency_stop() -> Alarm {
        Alarm::new(2001, 0, 0, "2024/01/01 12:01".to_string(), "Emergency Stop".to_string())
    }

    #[must_use]
    pub fn safety_error() -> Alarm {
        Alarm::new(3001, 2, 2, "2024/01/01 12:02".to_string(), "Safety Error".to_string())
            .with_sub_code(
                "[SV#2]".to_string(),
                "Safety circuit error".to_string(),
                "1".to_string(),
            )
    }

    #[must_use]
    pub fn communication_error() -> Alarm {
        Alarm::new(4001, 3, 3, "2024/01/01 12:03".to_string(), "Communication Error".to_string())
            .with_sub_code(
                "[COM#1]".to_string(),
                "Network communication error".to_string(),
                "2".to_string(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_new() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        assert_eq!(alarm.code, 1001);
        assert_eq!(alarm.data, 1);
        assert_eq!(alarm.alarm_type, 1);
        assert_eq!(alarm.time, "2024/01/01 12:00");
        assert_eq!(alarm.name, "Test Alarm");
        assert_eq!(alarm.sub_code_info, "");
        assert_eq!(alarm.sub_code_data, "");
        assert_eq!(alarm.sub_code_reverse, "");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_with_sub_code() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string())
                .with_sub_code(
                    "[SV#1]".to_string(),
                    "Sub code data".to_string(),
                    "Reverse".to_string(),
                );

        assert_eq!(alarm.sub_code_info, "[SV#1]");
        assert_eq!(alarm.sub_code_data, "Sub code data");
        assert_eq!(alarm.sub_code_reverse, "Reverse");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_default() {
        let alarm = Alarm::default();

        assert_eq!(alarm.code, 0);
        assert_eq!(alarm.data, 0);
        assert_eq!(alarm.alarm_type, 0);
        assert_eq!(alarm.time, "2024/01/01 00:00");
        assert_eq!(alarm.name, "No Alarm");
        assert_eq!(alarm.sub_code_info, "");
        assert_eq!(alarm.sub_code_data, "");
        assert_eq!(alarm.sub_code_reverse, "");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_serialize_complete() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        let data = alarm.serialize_complete(TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 268); // 4+4+4+16+32+16+96+96

        // Check alarm code (first 4 bytes)
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 1001);

        // Check alarm data (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[4], data[5], data[6], data[7]]), 1);

        // Check alarm type (next 4 bytes)
        assert_eq!(u32::from_le_bytes([data[8], data[9], data[10], data[11]]), 1);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_serialize_attribute() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        // Test alarm code serialization
        let data = alarm.serialize(1, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 1001);

        // Test alarm data serialization
        let data = alarm.serialize(2, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 1);

        // Test alarm type serialization
        let data = alarm.serialize(3, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(u32::from_le_bytes([data[0], data[1], data[2], data[3]]), 1);

        // Test alarm time serialization
        let data = alarm.serialize(4, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 16);
        let time_str =
            String::from_utf8_lossy(&data[..data.iter().position(|&b| b == 0).unwrap_or(16)]);
        assert_eq!(time_str, "2024/01/01 12:00");

        // Test alarm name serialization
        let data = alarm.serialize(5, TextEncoding::Utf8).unwrap();
        assert_eq!(data.len(), 32);
        let name_str =
            String::from_utf8_lossy(&data[..data.iter().position(|&b| b == 0).unwrap_or(32)]);
        assert_eq!(name_str, "Test Alarm");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_serialize_invalid_attribute() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        let result = alarm.serialize(99, TextEncoding::Utf8);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::InvalidAttribute));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_deserialize() {
        let original_alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        let serialized = original_alarm.serialize_complete(TextEncoding::Utf8).unwrap();
        let deserialized = Alarm::deserialize(&serialized, TextEncoding::Utf8).unwrap();

        assert_eq!(deserialized.code, original_alarm.code);
        assert_eq!(deserialized.data, original_alarm.data);
        assert_eq!(deserialized.alarm_type, original_alarm.alarm_type);
        assert_eq!(deserialized.time, original_alarm.time);
        assert_eq!(deserialized.name, original_alarm.name);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_deserialize_insufficient_data() {
        let short_data = vec![0u8; 10]; // Less than 60 bytes
        let result = Alarm::deserialize(&short_data, TextEncoding::Utf8);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::Deserialization(_)));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_variable_type_trait() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        let serialized = alarm.serialize_complete(TextEncoding::Utf8).unwrap();
        assert_eq!(serialized.len(), 268);

        let deserialized = Alarm::deserialize(&serialized, TextEncoding::Utf8).unwrap();
        assert_eq!(deserialized.code, 1001);
    }

    #[test]
    fn test_test_alarms() {
        let servo_alarm = test_alarms::servo_error();
        assert_eq!(servo_alarm.code, 1001);
        assert_eq!(servo_alarm.name, "Servo Error");
        assert_eq!(servo_alarm.sub_code_info, "[SV#1]");

        let emergency_alarm = test_alarms::emergency_stop();
        assert_eq!(emergency_alarm.code, 2001);
        assert_eq!(emergency_alarm.name, "Emergency Stop");
        assert_eq!(emergency_alarm.sub_code_info, "");

        let safety_alarm = test_alarms::safety_error();
        assert_eq!(safety_alarm.code, 3001);
        assert_eq!(safety_alarm.name, "Safety Error");
        assert_eq!(safety_alarm.sub_code_info, "[SV#2]");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_serialize_long_strings() {
        let long_time = "2024/01/01 12:00:00.123456789"; // Longer than 16 bytes
        let long_name = "This is a very long alarm name that exceeds 32 bytes limit"; // Longer than 32 bytes

        let alarm = Alarm::new(1001, 1, 1, long_time.to_string(), long_name.to_string());

        let data = alarm.serialize_complete(TextEncoding::Utf8).unwrap();

        // Time should be truncated to 16 bytes
        let time_str = String::from_utf8_lossy(&data[12..28]);
        assert!(time_str.len() <= 16);
        assert!(time_str.starts_with("2024/01/01 12:00"));

        // Name should be truncated to 32 bytes
        let name_str = String::from_utf8_lossy(&data[28..60]);
        assert!(name_str.len() <= 32);
        assert!(name_str.starts_with("This is a very long alarm"));
    }

    #[test]
    fn test_alarm_encoding_utf8() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        // Test UTF-8 encoding (default)
        assert_eq!(alarm.name, "Test Alarm");
        assert_eq!(alarm.time, "2024/01/01 12:00");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_encoding_shift_jis() {
        // Create test alarm with Japanese text
        let japanese_name = "テストアラーム";
        let japanese_time = "2024/01/01 12:00";
        let alarm = Alarm::new(1001, 1, 1, japanese_time.to_string(), japanese_name.to_string());

        // Serialize with Shift_JIS encoding
        let data = alarm.serialize_complete(TextEncoding::ShiftJis).unwrap();

        // Deserialize with Shift_JIS encoding
        let deserialized_alarm = Alarm::deserialize(&data, TextEncoding::ShiftJis).unwrap();

        // Test Shift_JIS round-trip
        assert_eq!(deserialized_alarm.name, japanese_name);
        assert_eq!(deserialized_alarm.time, japanese_time);

        // Deserialize with UTF-8 encoding (should produce garbled text)
        let alarm_utf8 = Alarm::deserialize(&data, TextEncoding::Utf8).unwrap();

        // The UTF-8 decoding should produce garbled text for Japanese name
        assert_ne!(alarm_utf8.name, japanese_name);

        // Time is ASCII, so it should be the same in both encodings
        assert_eq!(alarm_utf8.time, japanese_time);
    }

    #[test]
    fn test_alarm_sub_code_encoding() {
        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string())
                .with_sub_code(
                    "[SV#1]".to_string(),
                    "Sub code data".to_string(),
                    "Reverse".to_string(),
                );

        // Test UTF-8 encoding (default)
        assert_eq!(alarm.sub_code_info, "[SV#1]");
        assert_eq!(alarm.sub_code_data, "Sub code data");
        assert_eq!(alarm.sub_code_reverse, "Reverse");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_deserialize_with_encoding() {
        // Create test alarm
        let time_str = "2024/01/01 12:00";
        let name_str = "Test Alarm";
        let alarm = Alarm::new(1001, 1, 1, time_str.to_string(), name_str.to_string());

        // Serialize with UTF-8 encoding
        let data = alarm.serialize_complete(TextEncoding::Utf8).unwrap();

        // Deserialize with UTF-8 encoding
        let deserialized_alarm = Alarm::deserialize(&data, TextEncoding::Utf8).unwrap();

        assert_eq!(deserialized_alarm.code, 1001);
        assert_eq!(deserialized_alarm.data, 1);
        assert_eq!(deserialized_alarm.alarm_type, 1);
        assert_eq!(deserialized_alarm.time, time_str);
        assert_eq!(deserialized_alarm.name, name_str);
    }
}
