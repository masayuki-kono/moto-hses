//! Status data structures and operations

use bytes::Buf;
use crate::error::ProtocolError;
use crate::types::VariableType;

// Enhanced status structure
#[derive(Debug, Clone)]
pub struct Status {
    pub step: bool,
    pub one_cycle: bool,
    pub continuous: bool,
    pub running: bool,
    pub speed_limited: bool,
    pub teach: bool,
    pub play: bool,
    pub remote: bool,
    pub teach_pendant_hold: bool,
    pub external_hold: bool,
    pub command_hold: bool,
    pub alarm: bool,
    pub error: bool,
    pub servo_on: bool,
}

impl Status {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = data;
        let status_word1 = buf.get_u16_le();
        let status_word2 = buf.get_u16_le();

        Ok(Self {
            step: (status_word1 & 0x0001) != 0,
            one_cycle: (status_word1 & 0x0002) != 0,
            continuous: (status_word1 & 0x0004) != 0,
            running: (status_word1 & 0x0008) != 0,
            speed_limited: (status_word1 & 0x0010) != 0,
            teach: (status_word1 & 0x0020) != 0,
            play: (status_word1 & 0x0040) != 0,
            remote: (status_word1 & 0x0080) != 0,
            teach_pendant_hold: (status_word2 & 0x0002) != 0,
            external_hold: (status_word2 & 0x0004) != 0,
            command_hold: (status_word2 & 0x0008) != 0,
            alarm: (status_word2 & 0x0010) != 0,
            error: (status_word2 & 0x0020) != 0,
            servo_on: (status_word2 & 0x0040) != 0,
        })
    }

    pub fn is_running(&self) -> bool { self.running }
    pub fn is_servo_on(&self) -> bool { self.servo_on }
    pub fn has_alarm(&self) -> bool { self.alarm }
    pub fn is_teach_mode(&self) -> bool { self.teach }
    pub fn is_play_mode(&self) -> bool { self.play }
    pub fn is_remote_mode(&self) -> bool { self.remote }
}

impl VariableType for Status {
    fn command_id() -> u16 { 0x72 }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();
        let mut status_word1 = 0u16;
        let mut status_word2 = 0u16;
        
        if self.step { status_word1 |= 0x0001; }
        if self.one_cycle { status_word1 |= 0x0002; }
        if self.continuous { status_word1 |= 0x0004; }
        if self.running { status_word1 |= 0x0008; }
        if self.speed_limited { status_word1 |= 0x0010; }
        if self.teach { status_word1 |= 0x0020; }
        if self.play { status_word1 |= 0x0040; }
        if self.remote { status_word1 |= 0x0080; }
        
        if self.teach_pendant_hold { status_word2 |= 0x0002; }
        if self.external_hold { status_word2 |= 0x0004; }
        if self.command_hold { status_word2 |= 0x0008; }
        if self.alarm { status_word2 |= 0x0010; }
        if self.error { status_word2 |= 0x0020; }
        if self.servo_on { status_word2 |= 0x0040; }
        
        data.extend_from_slice(&status_word1.to_le_bytes());
        data.extend_from_slice(&status_word2.to_le_bytes());
        Ok(data)
    }
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError> {
        Status::from_bytes(data)
    }
}

// Wrapper types to avoid orphan rule violations
pub struct StatusWrapper(pub Status);

impl VariableType for StatusWrapper {
    fn command_id() -> u16 { 0x72 }
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
