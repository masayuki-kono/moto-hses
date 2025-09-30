//! Alarm data structures and operations

use crate::commands::{Command, VariableType};
use crate::encoding::TextEncoding;
use crate::error::ProtocolError;

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

impl VariableType for Alarm {
    fn command_id() -> u16 {
        0x70
    }

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

/// Command for reading alarm data (0x70)
#[derive(Debug, Clone)]
pub struct ReadAlarmData {
    pub instance: u16,
    pub attribute: u8,
}

/// Command for reading alarm history (0x71)
#[derive(Debug, Clone)]
pub struct ReadAlarmHistory {
    pub instance: u16,
    pub attribute: u8,
}

impl ReadAlarmHistory {
    #[must_use]
    pub const fn new(instance: u16, attribute: u8) -> Self {
        Self { instance, attribute }
    }

    /// Validate instance range for alarm history
    #[must_use]
    pub const fn is_valid_instance(&self) -> bool {
        matches!(
            self.instance,
            1..=100 | 1001..=1100 | 2001..=2100 | 3001..=3100 | 4001..=4100
        )
    }

    /// Get alarm category from instance
    #[must_use]
    pub const fn get_alarm_category(&self) -> AlarmCategory {
        match self.instance {
            1..=100 => AlarmCategory::MajorFailure,
            1001..=1100 => AlarmCategory::MonitorAlarm,
            2001..=2100 => AlarmCategory::UserAlarmSystem,
            3001..=3100 => AlarmCategory::UserAlarmUser,
            4001..=4100 => AlarmCategory::OfflineAlarm,
            _ => AlarmCategory::Invalid,
        }
    }

    /// Get alarm index within category
    #[must_use]
    pub const fn get_alarm_index(&self) -> usize {
        match self.get_alarm_category() {
            AlarmCategory::MajorFailure => (self.instance - 1) as usize,
            AlarmCategory::MonitorAlarm => (self.instance - 1001) as usize,
            AlarmCategory::UserAlarmSystem => (self.instance - 2001) as usize,
            AlarmCategory::UserAlarmUser => (self.instance - 3001) as usize,
            AlarmCategory::OfflineAlarm => (self.instance - 4001) as usize,
            AlarmCategory::Invalid => 0,
        }
    }
}

/// Alarm categories for history reading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmCategory {
    MajorFailure,    // 1-100
    MonitorAlarm,    // 1001-1100
    UserAlarmSystem, // 2001-2100
    UserAlarmUser,   // 3001-3100
    OfflineAlarm,    // 4001-4100
    Invalid,
}

impl ReadAlarmData {
    #[must_use]
    pub const fn new(instance: u16, attribute: u8) -> Self {
        Self { instance, attribute }
    }

    /// Validate instance range for alarm data
    #[must_use]
    pub const fn is_valid_instance(&self) -> bool {
        matches!(
            self.instance,
            1..=100 | 1001..=1100 | 2001..=2100 | 3001..=3100 | 4001..=4100
        )
    }

    /// Validate attribute range for alarm data
    #[must_use]
    pub const fn is_valid_attribute(&self) -> bool {
        matches!(self.attribute, 0..=8)
    }
}

impl Command for ReadAlarmData {
    type Response = Alarm;

    fn command_id() -> u16 {
        0x70
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // For 0x70 command, payload is typically empty
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

impl Command for ReadAlarmHistory {
    type Response = Alarm;

    fn command_id() -> u16 {
        0x71
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // For 0x71 command, payload is typically empty
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

/// Alarm attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmAttribute {
    Code = 1,
    Data = 2,
    Type = 3,
    Time = 4,
    Name = 5,
    SubCodeInfo = 6,
    SubCodeData = 7,
    SubCodeReverse = 8,
}

impl From<u8> for AlarmAttribute {
    fn from(value: u8) -> Self {
        match value {
            2 => Self::Data,
            3 => Self::Type,
            4 => Self::Time,
            5 => Self::Name,
            6 => Self::SubCodeInfo,
            7 => Self::SubCodeData,
            8 => Self::SubCodeReverse,
            _ => Self::Code,
        }
    }
}

/// Alarm Reset / Error Cancel Command (0x82)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmResetType {
    Reset = 1,  // RESET (Alarm reset)
    Cancel = 2, // CANCEL (Error cancel)
}

impl From<u16> for AlarmResetType {
    fn from(value: u16) -> Self {
        match value {
            2 => Self::Cancel,
            _ => Self::Reset, // Default to Reset
        }
    }
}

/// Command for alarm reset / error cancel (0x82)
#[derive(Debug, Clone)]
pub struct AlarmReset {
    pub reset_type: AlarmResetType,
}

impl AlarmReset {
    #[must_use]
    pub const fn new(reset_type: AlarmResetType) -> Self {
        Self { reset_type }
    }

    /// Create a reset command
    #[must_use]
    pub const fn reset() -> Self {
        Self { reset_type: AlarmResetType::Reset }
    }

    /// Create a cancel command
    #[must_use]
    pub const fn cancel() -> Self {
        Self { reset_type: AlarmResetType::Cancel }
    }
}

impl Command for AlarmReset {
    type Response = ();

    fn command_id() -> u16 {
        0x82
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Payload: 32-bit integer (4 bytes) with fixed value 1
        // Byte 0: Data 1, Bytes 1-3: Reserved
        Ok(vec![1, 0, 0, 0])
    }

    fn instance(&self) -> u16 {
        self.reset_type as u16
    }

    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
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
    fn test_read_alarm_data_new() {
        let command = ReadAlarmData::new(1, 0);
        assert_eq!(command.instance, 1);
        assert_eq!(command.attribute, 0);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_alarm_data_command_trait() {
        let command = ReadAlarmData::new(2, 5);

        assert_eq!(ReadAlarmData::command_id(), 0x70);
        assert_eq!(command.instance(), 2);
        assert_eq!(command.attribute(), 5);

        let serialized = command.serialize().unwrap();
        assert_eq!(serialized.len(), 0); // Empty payload for 0x70
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_variable_type_trait() {
        assert_eq!(Alarm::command_id(), 0x70);

        let alarm =
            Alarm::new(1001, 1, 1, "2024/01/01 12:00".to_string(), "Test Alarm".to_string());

        let serialized = alarm.serialize_complete(TextEncoding::Utf8).unwrap();
        assert_eq!(serialized.len(), 268);

        let deserialized = Alarm::deserialize(&serialized, TextEncoding::Utf8).unwrap();
        assert_eq!(deserialized.code, 1001);
    }

    #[test]
    fn test_alarm_attribute_from_u8() {
        assert_eq!(AlarmAttribute::from(1), AlarmAttribute::Code);
        assert_eq!(AlarmAttribute::from(2), AlarmAttribute::Data);
        assert_eq!(AlarmAttribute::from(3), AlarmAttribute::Type);
        assert_eq!(AlarmAttribute::from(4), AlarmAttribute::Time);
        assert_eq!(AlarmAttribute::from(5), AlarmAttribute::Name);
        assert_eq!(AlarmAttribute::from(6), AlarmAttribute::SubCodeInfo);
        assert_eq!(AlarmAttribute::from(7), AlarmAttribute::SubCodeData);
        assert_eq!(AlarmAttribute::from(8), AlarmAttribute::SubCodeReverse);
        assert_eq!(AlarmAttribute::from(99), AlarmAttribute::Code); // Default
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
    #[allow(clippy::unwrap_used)]
    fn test_read_alarm_history_command_trait() {
        let cmd = ReadAlarmHistory::new(1, 1);
        assert_eq!(ReadAlarmHistory::command_id(), 0x71);
        assert_eq!(cmd.instance(), 1);
        assert_eq!(cmd.attribute(), 1);
        assert!(cmd.serialize().unwrap().is_empty());
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::cognitive_complexity)]
    fn test_read_alarm_history_instance_validation() {
        // Valid instances
        assert!(ReadAlarmHistory::new(1, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(50, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(100, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(1001, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(1050, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(1100, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(2001, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(2050, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(2100, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(3001, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(3050, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(3100, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(4001, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(4050, 1).is_valid_instance());
        assert!(ReadAlarmHistory::new(4100, 1).is_valid_instance());

        // Invalid instances
        assert!(!ReadAlarmHistory::new(0, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(101, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(500, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(1000, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(1101, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(2000, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(2101, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(3000, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(3101, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(4000, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(4101, 1).is_valid_instance());
        assert!(!ReadAlarmHistory::new(5000, 1).is_valid_instance());
    }

    #[test]
    fn test_read_alarm_history_category_detection() {
        // Major failure alarms (1-100)
        assert_eq!(ReadAlarmHistory::new(1, 1).get_alarm_category(), AlarmCategory::MajorFailure);
        assert_eq!(ReadAlarmHistory::new(50, 1).get_alarm_category(), AlarmCategory::MajorFailure);
        assert_eq!(ReadAlarmHistory::new(100, 1).get_alarm_category(), AlarmCategory::MajorFailure);

        // Monitor alarms (1001-1100)
        assert_eq!(
            ReadAlarmHistory::new(1001, 1).get_alarm_category(),
            AlarmCategory::MonitorAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(1050, 1).get_alarm_category(),
            AlarmCategory::MonitorAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(1100, 1).get_alarm_category(),
            AlarmCategory::MonitorAlarm
        );

        // User alarm (system) (2001-2100)
        assert_eq!(
            ReadAlarmHistory::new(2001, 1).get_alarm_category(),
            AlarmCategory::UserAlarmSystem
        );
        assert_eq!(
            ReadAlarmHistory::new(2050, 1).get_alarm_category(),
            AlarmCategory::UserAlarmSystem
        );
        assert_eq!(
            ReadAlarmHistory::new(2100, 1).get_alarm_category(),
            AlarmCategory::UserAlarmSystem
        );

        // User alarm (user) (3001-3100)
        assert_eq!(
            ReadAlarmHistory::new(3001, 1).get_alarm_category(),
            AlarmCategory::UserAlarmUser
        );
        assert_eq!(
            ReadAlarmHistory::new(3050, 1).get_alarm_category(),
            AlarmCategory::UserAlarmUser
        );
        assert_eq!(
            ReadAlarmHistory::new(3100, 1).get_alarm_category(),
            AlarmCategory::UserAlarmUser
        );

        // Offline alarm (4001-4100)
        assert_eq!(
            ReadAlarmHistory::new(4001, 1).get_alarm_category(),
            AlarmCategory::OfflineAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(4050, 1).get_alarm_category(),
            AlarmCategory::OfflineAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(4100, 1).get_alarm_category(),
            AlarmCategory::OfflineAlarm
        );

        // Invalid instances
        assert_eq!(ReadAlarmHistory::new(0, 1).get_alarm_category(), AlarmCategory::Invalid);
        assert_eq!(ReadAlarmHistory::new(5000, 1).get_alarm_category(), AlarmCategory::Invalid);
    }

    #[test]
    fn test_read_alarm_history_index_calculation() {
        // Major failure alarms (1-100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(1, 1).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(50, 1).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(100, 1).get_alarm_index(), 99);

        // Monitor alarms (1001-1100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(1001, 1).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(1050, 1).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(1100, 1).get_alarm_index(), 99);

        // User alarm (system) (2001-2100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(2001, 1).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(2050, 1).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(2100, 1).get_alarm_index(), 99);

        // User alarm (user) (3001-3100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(3001, 1).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(3050, 1).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(3100, 1).get_alarm_index(), 99);

        // Offline alarm (4001-4100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(4001, 1).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(4050, 1).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(4100, 1).get_alarm_index(), 99);

        // Invalid instances
        assert_eq!(ReadAlarmHistory::new(0, 1).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(5000, 1).get_alarm_index(), 0);
    }

    #[test]
    fn test_alarm_reset_type_enum() {
        assert_eq!(AlarmResetType::Reset as u16, 1);
        assert_eq!(AlarmResetType::Cancel as u16, 2);
    }

    #[test]
    fn test_alarm_reset_type_from_u16() {
        assert_eq!(AlarmResetType::from(1), AlarmResetType::Reset);
        assert_eq!(AlarmResetType::from(2), AlarmResetType::Cancel);
        assert_eq!(AlarmResetType::from(99), AlarmResetType::Reset); // Default
    }

    #[test]
    fn test_alarm_reset_new() {
        let reset_cmd = AlarmReset::new(AlarmResetType::Reset);
        assert_eq!(reset_cmd.reset_type, AlarmResetType::Reset);

        let cancel_cmd = AlarmReset::new(AlarmResetType::Cancel);
        assert_eq!(cancel_cmd.reset_type, AlarmResetType::Cancel);
    }

    #[test]
    fn test_alarm_reset_convenience_methods() {
        let reset_cmd = AlarmReset::reset();
        assert_eq!(reset_cmd.reset_type, AlarmResetType::Reset);

        let cancel_cmd = AlarmReset::cancel();
        assert_eq!(cancel_cmd.reset_type, AlarmResetType::Cancel);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_reset_command_trait() {
        let reset_cmd = AlarmReset::reset();
        let cancel_cmd = AlarmReset::cancel();

        // Command ID
        assert_eq!(AlarmReset::command_id(), 0x82);

        // Instance
        assert_eq!(reset_cmd.instance(), 1);
        assert_eq!(cancel_cmd.instance(), 2);

        // Attribute (fixed to 1)
        assert_eq!(reset_cmd.attribute(), 1);
        assert_eq!(cancel_cmd.attribute(), 1);

        // Service (Set_Attribute_Single)
        assert_eq!(reset_cmd.service(), 0x10);
        assert_eq!(cancel_cmd.service(), 0x10);

        // Serialization
        let reset_payload = reset_cmd.serialize().unwrap();
        let cancel_payload = cancel_cmd.serialize().unwrap();

        // Both should have the same payload: [1, 0, 0, 0]
        assert_eq!(reset_payload, vec![1, 0, 0, 0]);
        assert_eq!(cancel_payload, vec![1, 0, 0, 0]);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_alarm_reset_serialization() {
        let reset_cmd = AlarmReset::reset();
        let payload = reset_cmd.serialize().unwrap();

        // Payload should be 4 bytes: [1, 0, 0, 0]
        assert_eq!(payload.len(), 4);
        assert_eq!(payload[0], 1); // Data 1
        assert_eq!(payload[1], 0); // Reserved
        assert_eq!(payload[2], 0); // Reserved
        assert_eq!(payload[3], 0); // Reserved
    }

    #[test]
    fn test_alarm_reset_clone_and_debug() {
        let reset_cmd = AlarmReset::reset();
        let cloned_cmd = reset_cmd.clone();
        assert_eq!(reset_cmd.reset_type, cloned_cmd.reset_type);

        // Test Debug trait
        let debug_str = format!("{reset_cmd:?}");
        assert!(debug_str.contains("AlarmReset"));
        assert!(debug_str.contains("Reset"));
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
