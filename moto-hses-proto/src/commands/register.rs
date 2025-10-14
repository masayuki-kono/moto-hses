//! Register related commands (0x79)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Read register command (0x79)
#[derive(Debug, Clone)]
pub struct ReadRegister {
    pub register_number: u16,
}

impl ReadRegister {
    #[must_use]
    pub const fn new(register_number: u16) -> Self {
        Self { register_number }
    }
}

impl Command for ReadRegister {
    type Response = i16;

    fn command_id() -> u16 {
        0x79
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }

    fn instance(&self) -> u16 {
        self.register_number
    }

    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for register commands
    }

    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

/// Write register command (0x79)
#[derive(Debug, Clone)]
pub struct WriteRegister {
    pub register_number: u16,
    pub value: i16,
}

impl WriteRegister {
    #[must_use]
    pub const fn new(register_number: u16, value: i16) -> Self {
        Self { register_number, value }
    }
}

impl Command for WriteRegister {
    type Response = ();

    fn command_id() -> u16 {
        0x79
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Register data is 2 bytes (i16)
        Ok(self.value.to_le_bytes().to_vec())
    }

    fn instance(&self) -> u16 {
        self.register_number
    }

    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for register commands
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

/// Read multiple registers command (0x301)
#[derive(Debug, Clone)]
pub struct ReadMultipleRegisters {
    pub start_register_number: u16,
    pub count: u32, // Number of register data (max 237)
}

impl ReadMultipleRegisters {
    /// Create a new `ReadMultipleRegisters` command
    ///
    /// # Errors
    ///
    /// Returns an error if the register number is invalid or count is out of range
    pub fn new(start_register_number: u16, count: u32) -> Result<Self, ProtocolError> {
        // Validate register number (0-999)
        if start_register_number > 999 {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid register number: {start_register_number} (valid range: 0-999)"
            )));
        }
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-237)"
            )));
        }
        // Validate range doesn't exceed maximum register number
        let end_register = u32::from(start_register_number) + count - 1;
        if end_register > 999 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Register range exceeds maximum: {start_register_number}-{end_register} (max 999)"
            )));
        }
        Ok(Self { start_register_number, count })
    }
}

impl Command for ReadMultipleRegisters {
    type Response = Vec<i16>; // Array of register values
    fn command_id() -> u16 {
        0x301
    }
    fn instance(&self) -> u16 {
        self.start_register_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Different from 0x79 (which uses 1)
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple registers command (0x301)
#[derive(Debug, Clone)]
pub struct WriteMultipleRegisters {
    pub start_register_number: u16,
    pub values: Vec<i16>, // Register values to write
}

impl WriteMultipleRegisters {
    /// Create a new `WriteMultipleRegisters` command
    ///
    /// # Errors
    ///
    /// Returns an error if the register number is invalid, count is out of range, or range exceeds writable limit
    pub fn new(start_register_number: u16, values: Vec<i16>) -> Result<Self, ProtocolError> {
        let count = values.len();
        // Validate count (max 237, must be > 0)
        if count == 0 || count > 237 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Invalid count: {count} (must be 1-237)"
            )));
        }
        // Validate writable range (0-559 for writes)
        if start_register_number > 559 {
            return Err(ProtocolError::InvalidInstance(format!(
                "Register {start_register_number} is not writable (writable range: 0-559)"
            )));
        }
        let end_register = u32::from(start_register_number)
            + u32::try_from(count)
                .map_err(|_| ProtocolError::InvalidMessage("Count too large".to_string()))?
            - 1;
        if end_register > 559 {
            return Err(ProtocolError::InvalidMessage(format!(
                "Register range exceeds writable limit: {start_register_number}-{end_register} (max 559)"
            )));
        }
        Ok(Self { start_register_number, values })
    }
}

impl Command for WriteMultipleRegisters {
    type Response = ();
    fn command_id() -> u16 {
        0x301
    }
    fn instance(&self) -> u16 {
        self.start_register_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Different from 0x79 (which uses 1)
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.values.len())
            .map_err(|_| ProtocolError::InvalidMessage("Values count too large".to_string()))?;
        let mut payload = count.to_le_bytes().to_vec();
        for value in &self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test_read_multiple_registers_new() {
        // Valid cases
        assert!(ReadMultipleRegisters::new(0, 1).is_ok());
        assert!(ReadMultipleRegisters::new(500, 100).is_ok());
        assert!(ReadMultipleRegisters::new(999, 1).is_ok());
        assert!(ReadMultipleRegisters::new(0, 237).is_ok());

        // Invalid register number
        assert!(ReadMultipleRegisters::new(1000, 1).is_err());

        // Invalid count
        assert!(ReadMultipleRegisters::new(0, 0).is_err());
        assert!(ReadMultipleRegisters::new(0, 238).is_err());

        // Range overflow
        assert!(ReadMultipleRegisters::new(999, 2).is_err());
        assert!(ReadMultipleRegisters::new(500, 500).is_err());
    }

    #[test]
    fn test_read_multiple_registers_command_trait() {
        let cmd = ReadMultipleRegisters::new(100, 5).expect("Valid command should not fail");

        assert_eq!(ReadMultipleRegisters::command_id(), 0x301);
        assert_eq!(cmd.instance(), 100);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    fn test_read_multiple_registers_serialize() {
        let cmd = ReadMultipleRegisters::new(100, 5).expect("Valid command should not fail");
        let payload = cmd.serialize().expect("Serialization should not fail");

        assert_eq!(payload.len(), 4);
        assert_eq!(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]), 5);
    }

    #[test]
    fn test_write_multiple_registers_new() {
        // Valid cases
        assert!(WriteMultipleRegisters::new(0, vec![100]).is_ok());
        assert!(WriteMultipleRegisters::new(500, vec![100, 200, 300]).is_ok());
        assert!(WriteMultipleRegisters::new(559, vec![999]).is_ok());
        assert!(WriteMultipleRegisters::new(0, vec![0; 237]).is_ok());

        // Invalid register number (readable but not writable)
        assert!(WriteMultipleRegisters::new(560, vec![100]).is_err());
        assert!(WriteMultipleRegisters::new(999, vec![100]).is_err());

        // Invalid count
        assert!(WriteMultipleRegisters::new(0, vec![]).is_err());
        assert!(WriteMultipleRegisters::new(0, vec![0; 238]).is_err());

        // Range overflow
        assert!(WriteMultipleRegisters::new(559, vec![100, 200]).is_err());
        assert!(WriteMultipleRegisters::new(500, vec![0; 61]).is_err()); // 500 + 61 - 1 = 560 > 559
    }

    #[test]
    fn test_write_multiple_registers_command_trait() {
        let cmd = WriteMultipleRegisters::new(100, vec![100, 200, 300])
            .expect("Valid command should not fail");

        assert_eq!(WriteMultipleRegisters::command_id(), 0x301);
        assert_eq!(cmd.instance(), 100);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    fn test_write_multiple_registers_serialize() {
        let cmd = WriteMultipleRegisters::new(100, vec![100, 200, 300])
            .expect("Valid command should not fail");
        let payload = cmd.serialize().expect("Serialization should not fail");

        assert_eq!(payload.len(), 4 + 6); // 4 bytes count + 6 bytes data (3 * 2 bytes)

        // Check count
        assert_eq!(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]), 3);

        // Check data
        assert_eq!(i16::from_le_bytes([payload[4], payload[5]]), 100);
        assert_eq!(i16::from_le_bytes([payload[6], payload[7]]), 200);
        assert_eq!(i16::from_le_bytes([payload[8], payload[9]]), 300);
    }

    #[test]
    fn test_write_multiple_registers_serialize_empty() {
        let cmd = WriteMultipleRegisters::new(0, vec![]);
        assert!(cmd.is_err());
    }

    #[test]
    fn test_write_multiple_registers_serialize_large() {
        let large_values: Vec<i16> =
            (0..237).map(|i| i16::try_from(i).expect("i should fit in i16")).collect();
        let cmd =
            WriteMultipleRegisters::new(0, large_values).expect("Valid command should not fail");
        let payload = cmd.serialize().expect("Serialization should not fail");

        assert_eq!(payload.len(), 4 + 474); // 4 bytes count + 474 bytes data (237 * 2 bytes)
        assert_eq!(u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]), 237);
    }
}
