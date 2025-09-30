//! Servo related commands (0x83)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Hold/Servo On/off Command (0x83)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldServoControl {
    pub control_type: HoldServoType,
    pub value: HoldServoValue,
}

/// Type of Hold/Servo control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldServoType {
    Hold = 1,
    ServoOn = 2,
    HLock = 3,
}

/// ON/OFF value for Hold/Servo control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldServoValue {
    On = 1,
    Off = 2,
}

impl HoldServoControl {
    /// Create a new Hold/Servo control command
    #[must_use]
    pub const fn new(control_type: HoldServoType, value: HoldServoValue) -> Self {
        Self { control_type, value }
    }

    /// Create HOLD ON command
    #[must_use]
    pub const fn hold_on() -> Self {
        Self::new(HoldServoType::Hold, HoldServoValue::On)
    }

    /// Create HOLD OFF command
    #[must_use]
    pub const fn hold_off() -> Self {
        Self::new(HoldServoType::Hold, HoldServoValue::Off)
    }

    /// Create Servo ON command
    #[must_use]
    pub const fn servo_on() -> Self {
        Self::new(HoldServoType::ServoOn, HoldServoValue::On)
    }

    /// Create Servo OFF command
    #[must_use]
    pub const fn servo_off() -> Self {
        Self::new(HoldServoType::ServoOn, HoldServoValue::Off)
    }

    /// Create HLOCK ON command
    #[must_use]
    pub const fn hlock_on() -> Self {
        Self::new(HoldServoType::HLock, HoldServoValue::On)
    }

    /// Create HLOCK OFF command
    #[must_use]
    pub const fn hlock_off() -> Self {
        Self::new(HoldServoType::HLock, HoldServoValue::Off)
    }
}

impl Command for HoldServoControl {
    type Response = ();

    fn command_id() -> u16 {
        0x83
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&(self.value as i32).to_le_bytes());
        Ok(payload)
    }

    fn instance(&self) -> u16 {
        self.control_type as u16
    }

    fn attribute(&self) -> u8 {
        1
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_hold_servo_control_serialization() {
        let hold_on = HoldServoControl::hold_on();
        assert_eq!(hold_on.instance(), 1);
        assert_eq!(hold_on.attribute(), 1);
        assert_eq!(hold_on.service(), 0x10);
        let serialized = hold_on.serialize().unwrap();
        assert_eq!(serialized, vec![1, 0, 0, 0]);

        let servo_off = HoldServoControl::servo_off();
        assert_eq!(servo_off.instance(), 2);
        let serialized = servo_off.serialize().unwrap();
        assert_eq!(serialized, vec![2, 0, 0, 0]);

        let hlock_on = HoldServoControl::hlock_on();
        assert_eq!(hlock_on.instance(), 3);
        let serialized = hlock_on.serialize().unwrap();
        assert_eq!(serialized, vec![1, 0, 0, 0]);
    }
}
