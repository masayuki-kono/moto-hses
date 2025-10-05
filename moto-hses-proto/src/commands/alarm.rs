//! Alarm related commands (0x70, 0x71, 0x82)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Alarm attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmAttribute {
    All = 0,
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
            0 => Self::All,
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

/// Command for reading alarm data (0x70)
#[derive(Debug, Clone)]
pub struct ReadAlarmData {
    pub instance: u16,
    pub attribute: AlarmAttribute,
}

impl ReadAlarmData {
    #[must_use]
    pub const fn new(instance: u16, attribute: AlarmAttribute) -> Self {
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
}

impl Command for ReadAlarmData {
    type Response = crate::payload::alarm::Alarm;

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
        self.attribute as u8
    }

    fn service(&self) -> u8 {
        if self.attribute == AlarmAttribute::All {
            0x01 // Get_Attribute_All
        } else {
            0x0e // Get_Attribute_Single
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

/// Command for reading alarm history (0x71)
#[derive(Debug, Clone)]
pub struct ReadAlarmHistory {
    pub instance: u16,
    pub attribute: AlarmAttribute,
}

impl ReadAlarmHistory {
    #[must_use]
    pub const fn new(instance: u16, attribute: AlarmAttribute) -> Self {
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

impl Command for ReadAlarmHistory {
    type Response = crate::payload::alarm::Alarm;

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
        self.attribute as u8
    }

    fn service(&self) -> u8 {
        if self.attribute == AlarmAttribute::All {
            0x01 // Get_Attribute_All
        } else {
            0x0e // Get_Attribute_Single
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alarm_attribute_from_u8() {
        assert_eq!(AlarmAttribute::from(0), AlarmAttribute::All);
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
    fn test_read_alarm_data_new() {
        let command = ReadAlarmData::new(1, AlarmAttribute::All);
        assert_eq!(command.instance, 1);
        assert_eq!(command.attribute, AlarmAttribute::All);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_alarm_data_command_trait() {
        let command = ReadAlarmData::new(2, AlarmAttribute::Name);

        assert_eq!(ReadAlarmData::command_id(), 0x70);
        assert_eq!(command.instance(), 2);
        assert_eq!(command.attribute(), 5);

        let serialized = command.serialize().unwrap();
        assert_eq!(serialized.len(), 0); // Empty payload for 0x70
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_alarm_history_command_trait() {
        let cmd = ReadAlarmHistory::new(1, AlarmAttribute::Code);
        assert_eq!(ReadAlarmHistory::command_id(), 0x71);
        assert_eq!(cmd.instance(), 1);
        assert_eq!(cmd.attribute(), 1);
        assert!(cmd.serialize().unwrap().is_empty());
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::cognitive_complexity)]
    fn test_read_alarm_history_instance_validation() {
        // Valid instances
        assert!(ReadAlarmHistory::new(1, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(50, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(100, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(1001, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(1050, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(1100, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(2001, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(2050, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(2100, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(3001, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(3050, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(3100, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(4001, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(4050, AlarmAttribute::Code).is_valid_instance());
        assert!(ReadAlarmHistory::new(4100, AlarmAttribute::Code).is_valid_instance());

        // Invalid instances
        assert!(!ReadAlarmHistory::new(0, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(101, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(500, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(1000, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(1101, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(2000, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(2101, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(3000, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(3101, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(4000, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(4101, AlarmAttribute::Code).is_valid_instance());
        assert!(!ReadAlarmHistory::new(5000, AlarmAttribute::Code).is_valid_instance());
    }

    #[test]
    fn test_read_alarm_history_category_detection() {
        // Major failure alarms (1-100)
        assert_eq!(
            ReadAlarmHistory::new(1, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::MajorFailure
        );
        assert_eq!(
            ReadAlarmHistory::new(50, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::MajorFailure
        );
        assert_eq!(
            ReadAlarmHistory::new(100, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::MajorFailure
        );

        // Monitor alarms (1001-1100)
        assert_eq!(
            ReadAlarmHistory::new(1001, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::MonitorAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(1050, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::MonitorAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(1100, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::MonitorAlarm
        );

        // User alarm (system) (2001-2100)
        assert_eq!(
            ReadAlarmHistory::new(2001, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::UserAlarmSystem
        );
        assert_eq!(
            ReadAlarmHistory::new(2050, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::UserAlarmSystem
        );
        assert_eq!(
            ReadAlarmHistory::new(2100, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::UserAlarmSystem
        );

        // User alarm (user) (3001-3100)
        assert_eq!(
            ReadAlarmHistory::new(3001, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::UserAlarmUser
        );
        assert_eq!(
            ReadAlarmHistory::new(3050, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::UserAlarmUser
        );
        assert_eq!(
            ReadAlarmHistory::new(3100, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::UserAlarmUser
        );

        // Offline alarm (4001-4100)
        assert_eq!(
            ReadAlarmHistory::new(4001, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::OfflineAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(4050, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::OfflineAlarm
        );
        assert_eq!(
            ReadAlarmHistory::new(4100, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::OfflineAlarm
        );

        // Invalid instances
        assert_eq!(
            ReadAlarmHistory::new(0, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::Invalid
        );
        assert_eq!(
            ReadAlarmHistory::new(5000, AlarmAttribute::Code).get_alarm_category(),
            AlarmCategory::Invalid
        );
    }

    #[test]
    fn test_read_alarm_history_index_calculation() {
        // Major failure alarms (1-100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(1, AlarmAttribute::Code).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(50, AlarmAttribute::Code).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(100, AlarmAttribute::Code).get_alarm_index(), 99);

        // Monitor alarms (1001-1100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(1001, AlarmAttribute::Code).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(1050, AlarmAttribute::Code).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(1100, AlarmAttribute::Code).get_alarm_index(), 99);

        // User alarm (system) (2001-2100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(2001, AlarmAttribute::Code).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(2050, AlarmAttribute::Code).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(2100, AlarmAttribute::Code).get_alarm_index(), 99);

        // User alarm (user) (3001-3100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(3001, AlarmAttribute::Code).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(3050, AlarmAttribute::Code).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(3100, AlarmAttribute::Code).get_alarm_index(), 99);

        // Offline alarm (4001-4100) -> index 0-99
        assert_eq!(ReadAlarmHistory::new(4001, AlarmAttribute::Code).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(4050, AlarmAttribute::Code).get_alarm_index(), 49);
        assert_eq!(ReadAlarmHistory::new(4100, AlarmAttribute::Code).get_alarm_index(), 99);

        // Invalid instances
        assert_eq!(ReadAlarmHistory::new(0, AlarmAttribute::Code).get_alarm_index(), 0);
        assert_eq!(ReadAlarmHistory::new(5000, AlarmAttribute::Code).get_alarm_index(), 0);
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
}
