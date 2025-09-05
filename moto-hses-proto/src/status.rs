//! Status data structures and operations

use crate::error::ProtocolError;
use crate::types::VariableType;
use bytes::Buf;

// Enhanced status structure
#[derive(Debug, Clone)]
pub struct Status {
    pub data1: StatusData1,
    pub data2: StatusData2,
}

impl Status {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }

        let data1 = StatusData1::from_bytes(&data[0..2])?;
        let data2 = StatusData2::from_bytes(&data[2..4])?;

        Ok(Self { data1, data2 })
    }

    /// Create Status from Data 1 and Data 2 instances
    pub fn new(data1: StatusData1, data2: StatusData2) -> Self {
        Self { data1, data2 }
    }

    pub fn is_running(&self) -> bool {
        self.data1.running
    }
    pub fn is_servo_on(&self) -> bool {
        self.data2.servo_on
    }
    pub fn has_alarm(&self) -> bool {
        self.data2.alarm
    }
    pub fn is_teach_mode(&self) -> bool {
        self.data1.teach
    }
    pub fn is_play_mode(&self) -> bool {
        self.data1.play
    }
    pub fn is_remote_mode(&self) -> bool {
        self.data1.remote
    }
    pub fn has_error(&self) -> bool {
        self.data2.error
    }
}

impl VariableType for Status {
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();
        data.extend(self.data1.serialize()?);
        data.extend(self.data2.serialize()?);
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Status::from_bytes(data)
    }
}

// Attribute-specific status structures
#[derive(Debug, Clone)]
pub struct StatusData1 {
    pub step: bool,
    pub one_cycle: bool,
    pub continuous: bool,
    pub running: bool,
    pub speed_limited: bool,
    pub teach: bool,
    pub play: bool,
    pub remote: bool,
}

#[derive(Debug, Clone)]
pub struct StatusData2 {
    pub teach_pendant_hold: bool,
    pub external_hold: bool,
    pub command_hold: bool,
    pub alarm: bool,
    pub error: bool,
    pub servo_on: bool,
}

impl StatusData1 {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 2 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let status_word = buf.get_u16_le();

        Ok(Self {
            step: (status_word & 0x0001) != 0,
            one_cycle: (status_word & 0x0002) != 0,
            continuous: (status_word & 0x0004) != 0,
            running: (status_word & 0x0008) != 0,
            speed_limited: (status_word & 0x0010) != 0,
            teach: (status_word & 0x0020) != 0,
            play: (status_word & 0x0040) != 0,
            remote: (status_word & 0x0080) != 0,
        })
    }
}

impl StatusData2 {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 2 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let status_word = buf.get_u16_le();

        Ok(Self {
            teach_pendant_hold: (status_word & 0x0002) != 0,
            external_hold: (status_word & 0x0004) != 0,
            command_hold: (status_word & 0x0008) != 0,
            alarm: (status_word & 0x0010) != 0,
            error: (status_word & 0x0020) != 0,
            servo_on: (status_word & 0x0040) != 0,
        })
    }
}

impl VariableType for StatusData1 {
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();
        let mut status_word = 0u16;

        if self.step {
            status_word |= 0x0001;
        }
        if self.one_cycle {
            status_word |= 0x0002;
        }
        if self.continuous {
            status_word |= 0x0004;
        }
        if self.running {
            status_word |= 0x0008;
        }
        if self.speed_limited {
            status_word |= 0x0010;
        }
        if self.teach {
            status_word |= 0x0020;
        }
        if self.play {
            status_word |= 0x0040;
        }
        if self.remote {
            status_word |= 0x0080;
        }

        data.extend_from_slice(&status_word.to_le_bytes());
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        StatusData1::from_bytes(data)
    }
}

impl VariableType for StatusData2 {
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();
        let mut status_word = 0u16;

        if self.teach_pendant_hold {
            status_word |= 0x0002;
        }
        if self.external_hold {
            status_word |= 0x0004;
        }
        if self.command_hold {
            status_word |= 0x0008;
        }
        if self.alarm {
            status_word |= 0x0010;
        }
        if self.error {
            status_word |= 0x0020;
        }
        if self.servo_on {
            status_word |= 0x0040;
        }

        data.extend_from_slice(&status_word.to_le_bytes());
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        StatusData2::from_bytes(data)
    }
}

// Wrapper types to avoid orphan rule violations
pub struct StatusWrapper(pub Status);

impl VariableType for StatusWrapper {
    fn command_id() -> u16 {
        0x72
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        self.0.serialize()
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Status::from_bytes(data).map(StatusWrapper)
    }
}

impl From<StatusWrapper> for Status {
    fn from(wrapper: StatusWrapper) -> Self {
        wrapper.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_bytes() {
        let data = vec![0x01, 0x00, 0x40, 0x00];
        let status = Status::from_bytes(&data).unwrap();
        assert!(status.data1.step);
        assert!(status.data2.servo_on);
        assert!(!status.data1.running);
        assert!(!status.data2.alarm);
    }

    #[test]
    fn test_status_serialization() {
        let data1 = StatusData1 {
            step: true,
            one_cycle: false,
            continuous: false,
            running: false,
            speed_limited: false,
            teach: false,
            play: false,
            remote: false,
        };
        let data2 = StatusData2 {
            teach_pendant_hold: false,
            external_hold: false,
            command_hold: false,
            alarm: false,
            error: false,
            servo_on: true,
        };
        let status = Status::new(data1, data2);

        let serialized = status.serialize().unwrap();
        let deserialized = Status::deserialize(&serialized).unwrap();
        assert_eq!(status.data1.step, deserialized.data1.step);
        assert_eq!(status.data2.servo_on, deserialized.data2.servo_on);
        assert_eq!(status.data1.running, deserialized.data1.running);
    }

    #[test]
    fn test_status_helper_methods() {
        let data1 = StatusData1 {
            step: false,
            one_cycle: false,
            continuous: false,
            running: true,
            speed_limited: false,
            teach: true,
            play: false,
            remote: false,
        };
        let data2 = StatusData2 {
            teach_pendant_hold: false,
            external_hold: false,
            command_hold: false,
            alarm: false,
            error: false,
            servo_on: true,
        };
        let status = Status::new(data1, data2);

        assert!(status.is_running());
        assert!(status.is_servo_on());
        assert!(!status.has_alarm());
        assert!(status.is_teach_mode());
        assert!(!status.is_play_mode());
        assert!(!status.is_remote_mode());
    }

    #[test]
    fn test_status_variable_type_trait() {
        assert_eq!(Status::command_id(), 0x72);

        let data1 = StatusData1 {
            step: true,
            one_cycle: false,
            continuous: false,
            running: false,
            speed_limited: false,
            teach: false,
            play: false,
            remote: false,
        };
        let data2 = StatusData2 {
            teach_pendant_hold: false,
            external_hold: false,
            command_hold: false,
            alarm: false,
            error: false,
            servo_on: false,
        };
        let status = Status::new(data1, data2);

        let serialized = status.serialize().unwrap();
        let deserialized = Status::deserialize(&serialized).unwrap();
        assert_eq!(status.data1.step, deserialized.data1.step);
    }

    #[test]
    fn test_status_wrapper() {
        let data1 = StatusData1 {
            step: true,
            one_cycle: false,
            continuous: false,
            running: false,
            speed_limited: false,
            teach: false,
            play: false,
            remote: false,
        };
        let data2 = StatusData2 {
            teach_pendant_hold: false,
            external_hold: false,
            command_hold: false,
            alarm: false,
            error: false,
            servo_on: false,
        };
        let status = Status::new(data1, data2);

        let wrapper = StatusWrapper(status.clone());
        assert_eq!(StatusWrapper::command_id(), 0x72);

        let serialized = wrapper.serialize().unwrap();
        let deserialized = StatusWrapper::deserialize(&serialized).unwrap();
        let deserialized_status: Status = deserialized.into();
        assert_eq!(status.data1.step, deserialized_status.data1.step);
    }

    #[test]
    fn test_status_data1() {
        let data = vec![0x01, 0x00]; // step bit set
        let status_data1 = StatusData1::from_bytes(&data).unwrap();
        assert!(status_data1.step);
        assert!(!status_data1.running);
        assert!(!status_data1.teach);

        let serialized = status_data1.serialize().unwrap();
        let deserialized = StatusData1::deserialize(&serialized).unwrap();
        assert_eq!(status_data1.step, deserialized.step);
    }

    #[test]
    fn test_status_data2() {
        let data = vec![0x40, 0x00]; // servo_on bit set
        let status_data2 = StatusData2::from_bytes(&data).unwrap();
        assert!(status_data2.servo_on);
        assert!(!status_data2.alarm);
        assert!(!status_data2.error);

        let serialized = status_data2.serialize().unwrap();
        let deserialized = StatusData2::deserialize(&serialized).unwrap();
        assert_eq!(status_data2.servo_on, deserialized.servo_on);
    }
}
