//! Alarm data structures and operations

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
    pub fn new(code: u32, data: u32, alarm_type: u32, time: String, name: String) -> Self {
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

    pub fn with_sub_code(mut self, info: String, data: String, reverse: String) -> Self {
        self.sub_code_info = info;
        self.sub_code_data = data;
        self.sub_code_reverse = reverse;
        self
    }

    /// Serialize alarm data for response
    pub fn serialize(&self, attribute: u8) -> Result<Vec<u8>, ProtocolError> {
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
                let time_bytes = self.time.as_bytes();
                let mut padded_time = vec![0u8; 16];
                padded_time[..time_bytes.len().min(16)]
                    .copy_from_slice(&time_bytes[..time_bytes.len().min(16)]);
                data.extend_from_slice(&padded_time);
            }
            5 => {
                // Alarm name
                let name_bytes = self.name.as_bytes();
                let mut padded_name = vec![0u8; 32];
                padded_name[..name_bytes.len().min(32)]
                    .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
                data.extend_from_slice(&padded_name);
            }
            6 => {
                // Sub code info
                let info_bytes = self.sub_code_info.as_bytes();
                let mut padded_info = vec![0u8; 16];
                padded_info[..info_bytes.len().min(16)]
                    .copy_from_slice(&info_bytes[..info_bytes.len().min(16)]);
                data.extend_from_slice(&padded_info);
            }
            7 => {
                // Sub code data
                let data_bytes = self.sub_code_data.as_bytes();
                let mut padded_data = vec![0u8; 96];
                padded_data[..data_bytes.len().min(96)]
                    .copy_from_slice(&data_bytes[..data_bytes.len().min(96)]);
                data.extend_from_slice(&padded_data);
            }
            8 => {
                // Sub code reverse
                let reverse_bytes = self.sub_code_reverse.as_bytes();
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
    pub fn serialize_complete(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        // Alarm code
        data.extend_from_slice(&self.code.to_le_bytes());
        // Alarm data
        data.extend_from_slice(&self.data.to_le_bytes());
        // Alarm type
        data.extend_from_slice(&self.alarm_type.to_le_bytes());

        // Alarm time (16 bytes)
        let time_bytes = self.time.as_bytes();
        let mut padded_time = vec![0u8; 16];
        padded_time[..time_bytes.len().min(16)]
            .copy_from_slice(&time_bytes[..time_bytes.len().min(16)]);
        data.extend_from_slice(&padded_time);

        // Alarm name (32 bytes)
        let name_bytes = self.name.as_bytes();
        let mut padded_name = vec![0u8; 32];
        padded_name[..name_bytes.len().min(32)]
            .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
        data.extend_from_slice(&padded_name);

        // Sub code info (16 bytes)
        let info_bytes = self.sub_code_info.as_bytes();
        let mut padded_info = vec![0u8; 16];
        padded_info[..info_bytes.len().min(16)]
            .copy_from_slice(&info_bytes[..info_bytes.len().min(16)]);
        data.extend_from_slice(&padded_info);

        // Sub code data (96 bytes)
        let data_bytes = self.sub_code_data.as_bytes();
        let mut padded_data = vec![0u8; 96];
        padded_data[..data_bytes.len().min(96)]
            .copy_from_slice(&data_bytes[..data_bytes.len().min(96)]);
        data.extend_from_slice(&padded_data);

        // Sub code reverse (96 bytes)
        let reverse_bytes = self.sub_code_reverse.as_bytes();
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

/// Alarm attribute types
#[derive(Debug, Clone, Copy, PartialEq)]
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
            1 => AlarmAttribute::Code,
            2 => AlarmAttribute::Data,
            3 => AlarmAttribute::Type,
            4 => AlarmAttribute::Time,
            5 => AlarmAttribute::Name,
            6 => AlarmAttribute::SubCodeInfo,
            7 => AlarmAttribute::SubCodeData,
            8 => AlarmAttribute::SubCodeReverse,
            _ => AlarmAttribute::Code,
        }
    }
}

/// Predefined alarms for testing
pub mod test_alarms {
    use super::*;

    pub fn servo_error() -> Alarm {
        Alarm::new(
            1001,
            1,
            1,
            "2024/01/01 12:00".to_string(),
            "Servo Error".to_string(),
        )
        .with_sub_code(
            "[SV#1]".to_string(),
            "Servo amplifier error".to_string(),
            "0".to_string(),
        )
    }

    pub fn emergency_stop() -> Alarm {
        Alarm::new(
            2001,
            0,
            0,
            "2024/01/01 12:01".to_string(),
            "Emergency Stop".to_string(),
        )
    }

    pub fn safety_error() -> Alarm {
        Alarm::new(
            3001,
            2,
            2,
            "2024/01/01 12:02".to_string(),
            "Safety Error".to_string(),
        )
        .with_sub_code(
            "[SV#2]".to_string(),
            "Safety circuit error".to_string(),
            "1".to_string(),
        )
    }
}
