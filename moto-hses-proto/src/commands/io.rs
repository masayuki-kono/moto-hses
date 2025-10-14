//! I/O related commands (0x78)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// I/O categories according to HSES protocol specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoCategory {
    /// Robot user input (1-512)
    RobotUserInput,
    /// Robot user output (1001-1512)
    RobotUserOutput,
    /// External input (2001-2128)
    ExternalInput,
    /// Network input (2701-2956)
    NetworkInput,
    /// External output (3001-3128)
    ExternalOutput,
    /// Network output (3701-3956)
    NetworkOutput,
    /// Robot system input (4001-4256)
    RobotSystemInput,
    /// Robot system output (5001-5512)
    RobotSystemOutput,
    /// Interface panel input (6001-6064)
    InterfacePanelInput,
    /// Auxiliary relay (7001-7999)
    AuxiliaryRelay,
    /// Robot control status signal (8001-8512)
    RobotControlStatusSignal,
    /// Pseudo input (8701-8720)
    PseudoInput,
}

impl IoCategory {
    /// Get the I/O category for a given I/O number
    #[must_use]
    pub const fn from_io_number(io_number: u16) -> Option<Self> {
        match io_number {
            1..=512 => Some(Self::RobotUserInput),
            1001..=1512 => Some(Self::RobotUserOutput),
            2001..=2128 => Some(Self::ExternalInput),
            2701..=2956 => Some(Self::NetworkInput),
            3001..=3128 => Some(Self::ExternalOutput),
            3701..=3956 => Some(Self::NetworkOutput),
            4001..=4256 => Some(Self::RobotSystemInput),
            5001..=5512 => Some(Self::RobotSystemOutput),
            6001..=6064 => Some(Self::InterfacePanelInput),
            7001..=7999 => Some(Self::AuxiliaryRelay),
            8001..=8512 => Some(Self::RobotControlStatusSignal),
            8701..=8720 => Some(Self::PseudoInput),
            _ => None,
        }
    }

    /// Check if an I/O number is valid
    #[must_use]
    pub const fn is_valid_io_number(io_number: u16) -> bool {
        Self::from_io_number(io_number).is_some()
    }

    /// Get the range of I/O numbers for this category
    #[must_use]
    pub const fn range(&self) -> (u16, u16) {
        match self {
            Self::RobotUserInput => (1, 512),
            Self::RobotUserOutput => (1001, 1512),
            Self::ExternalInput => (2001, 2128),
            Self::NetworkInput => (2701, 2956),
            Self::ExternalOutput => (3001, 3128),
            Self::NetworkOutput => (3701, 3956),
            Self::RobotSystemInput => (4001, 4256),
            Self::RobotSystemOutput => (5001, 5512),
            Self::InterfacePanelInput => (6001, 6064),
            Self::AuxiliaryRelay => (7001, 7999),
            Self::RobotControlStatusSignal => (8001, 8512),
            Self::PseudoInput => (8701, 8720),
        }
    }
}

/// Read I/O command (0x78)
#[derive(Debug, Clone)]
pub struct ReadIo {
    pub io_number: u16,
}

impl ReadIo {
    #[must_use]
    pub const fn new(io_number: u16) -> Self {
        Self { io_number }
    }
}

impl Command for ReadIo {
    type Response = u8;

    fn command_id() -> u16 {
        0x78
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }

    fn instance(&self) -> u16 {
        self.io_number
    }

    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for I/O commands
    }

    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

/// Write I/O command (0x78)
#[derive(Debug, Clone)]
pub struct WriteIo {
    pub io_number: u16,
    pub value: u8,
}

impl WriteIo {
    #[must_use]
    pub const fn new(io_number: u16, value: u8) -> Self {
        Self { io_number, value }
    }
}

impl Command for WriteIo {
    type Response = ();

    fn command_id() -> u16 {
        0x78
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(vec![self.value, 0, 0, 0])
    }

    fn instance(&self) -> u16 {
        self.io_number
    }

    fn attribute(&self) -> u8 {
        1 // Fixed to 1 for I/O commands
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

/// Read multiple I/O data command (0x300)
#[derive(Debug, Clone)]
pub struct ReadMultipleIo {
    pub start_io_number: u16,
    pub count: u32, // Number of I/O data (max 474, must be multiple of 2)
}

impl ReadMultipleIo {
    /// Create a new `ReadMultipleIo` command
    ///
    /// # Errors
    ///
    /// Returns an error if the I/O number is invalid or count is invalid
    pub fn new(start_io_number: u16, count: u32) -> Result<Self, ProtocolError> {
        if !IoCategory::is_valid_io_number(start_io_number) {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid I/O number: {start_io_number} (valid range: 0-999)"
            )));
        }
        // Validate count (max 474, must be multiple of 2)
        if count == 0 || count > 474 || count % 2 != 0 {
            return Err(ProtocolError::InvalidMessage("Invalid count".to_string()));
        }
        Ok(Self { start_io_number, count })
    }
}

impl Command for ReadMultipleIo {
    type Response = Vec<u8>; // Array of I/O data bytes
    fn command_id() -> u16 {
        0x300
    }
    fn instance(&self) -> u16 {
        self.start_io_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Different from 0x78 (which uses 1)
    fn service(&self) -> u8 {
        0x33
    } // Read plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // Only send count (4 bytes, little-endian)
        Ok(self.count.to_le_bytes().to_vec())
    }
}

/// Write multiple I/O data command (0x300)
#[derive(Debug, Clone)]
pub struct WriteMultipleIo {
    pub start_io_number: u16,
    pub io_data: Vec<u8>, // Each byte contains 8 I/O states
}

impl WriteMultipleIo {
    /// Create a new `WriteMultipleIo` command
    ///
    /// # Errors
    ///
    /// Returns an error if the I/O number is invalid or `io_data` is invalid
    pub fn new(start_io_number: u16, io_data: Vec<u8>) -> Result<Self, ProtocolError> {
        if !IoCategory::is_valid_io_number(start_io_number) {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid I/O number: {start_io_number} (valid range: 0-999)"
            )));
        }
        let count = io_data.len();
        // Validate count (max 474, must be multiple of 2)
        if count == 0 || count > 474 || count % 2 != 0 {
            return Err(ProtocolError::InvalidMessage("Invalid count".to_string()));
        }
        Ok(Self { start_io_number, io_data })
    }
}

impl Command for WriteMultipleIo {
    type Response = ();
    fn command_id() -> u16 {
        0x300
    }
    fn instance(&self) -> u16 {
        self.start_io_number
    }
    fn attribute(&self) -> u8 {
        0
    } // Different from 0x78 (which uses 1)
    fn service(&self) -> u8 {
        0x34
    } // Write plural data
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let count = u32::try_from(self.io_data.len())
            .map_err(|_| ProtocolError::InvalidMessage("I/O data too large".to_string()))?;
        let mut payload = count.to_le_bytes().to_vec();
        payload.extend_from_slice(&self.io_data);
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_category_from_io_number() {
        assert_eq!(IoCategory::from_io_number(1), Some(IoCategory::RobotUserInput));
        assert_eq!(IoCategory::from_io_number(512), Some(IoCategory::RobotUserInput));
        assert_eq!(IoCategory::from_io_number(1001), Some(IoCategory::RobotUserOutput));
        assert_eq!(IoCategory::from_io_number(1512), Some(IoCategory::RobotUserOutput));
        assert_eq!(IoCategory::from_io_number(2701), Some(IoCategory::NetworkInput));
        assert_eq!(IoCategory::from_io_number(2956), Some(IoCategory::NetworkInput));
        assert_eq!(IoCategory::from_io_number(0), None);
        assert_eq!(IoCategory::from_io_number(513), None);
        assert_eq!(IoCategory::from_io_number(1000), None);
        assert_eq!(IoCategory::from_io_number(1513), None);
    }

    #[test]
    fn test_io_category_is_valid_io_number() {
        assert!(IoCategory::is_valid_io_number(1));
        assert!(IoCategory::is_valid_io_number(512));
        assert!(IoCategory::is_valid_io_number(1001));
        assert!(IoCategory::is_valid_io_number(1512));
        assert!(IoCategory::is_valid_io_number(2701));
        assert!(IoCategory::is_valid_io_number(2956));
        assert!(!IoCategory::is_valid_io_number(0));
        assert!(!IoCategory::is_valid_io_number(513));
        assert!(!IoCategory::is_valid_io_number(1000));
        assert!(!IoCategory::is_valid_io_number(1513));
    }

    #[test]
    fn test_io_category_range() {
        assert_eq!(IoCategory::RobotUserInput.range(), (1, 512));
        assert_eq!(IoCategory::RobotUserOutput.range(), (1001, 1512));
        assert_eq!(IoCategory::NetworkInput.range(), (2701, 2956));
        assert_eq!(IoCategory::PseudoInput.range(), (8701, 8720));
    }

    #[test]
    fn test_io_ranges_consistency() {
        // Test that all ranges are properly defined
        for category in [
            IoCategory::RobotUserInput,
            IoCategory::RobotUserOutput,
            IoCategory::ExternalInput,
            IoCategory::NetworkInput,
            IoCategory::ExternalOutput,
            IoCategory::NetworkOutput,
            IoCategory::RobotSystemInput,
            IoCategory::RobotSystemOutput,
            IoCategory::InterfacePanelInput,
            IoCategory::AuxiliaryRelay,
            IoCategory::RobotControlStatusSignal,
            IoCategory::PseudoInput,
        ] {
            let (start, end) = category.range();
            assert!(start <= end, "Range {category:?}: start ({start}) should be <= end ({end})");
            assert!(start > 0, "Range {category:?}: start ({start}) should be > 0");
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_multiple_io_construction() {
        // Valid construction
        let cmd = ReadMultipleIo::new(1, 2).unwrap();
        assert_eq!(cmd.start_io_number, 1);
        assert_eq!(cmd.count, 2);

        // Valid construction with maximum count
        let cmd = ReadMultipleIo::new(2701, 474).unwrap();
        assert_eq!(cmd.start_io_number, 2701);
        assert_eq!(cmd.count, 474);
    }

    #[test]
    fn test_read_multiple_io_validation() {
        // Invalid I/O number
        assert!(ReadMultipleIo::new(0, 2).is_err());
        assert!(ReadMultipleIo::new(65535, 2).is_err());

        // Invalid count - zero
        assert!(ReadMultipleIo::new(1, 0).is_err());

        // Invalid count - odd number
        assert!(ReadMultipleIo::new(1, 1).is_err());
        assert!(ReadMultipleIo::new(1, 3).is_err());

        // Invalid count - too large
        assert!(ReadMultipleIo::new(1, 475).is_err());
        assert!(ReadMultipleIo::new(1, 1000).is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_multiple_io_command_trait() {
        let cmd = ReadMultipleIo::new(1, 2).unwrap();
        assert_eq!(ReadMultipleIo::command_id(), 0x300);
        assert_eq!(cmd.instance(), 1);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x33);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_multiple_io_serialization() {
        let cmd = ReadMultipleIo::new(1, 2).unwrap();
        let payload = cmd.serialize().unwrap();
        assert_eq!(payload, vec![2, 0, 0, 0]); // 2 in little-endian

        let cmd = ReadMultipleIo::new(2701, 474).unwrap();
        let payload = cmd.serialize().unwrap();
        assert_eq!(payload, vec![218, 1, 0, 0]); // 474 in little-endian
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    #[allow(clippy::redundant_clone)]
    fn test_write_multiple_io_construction() {
        // Valid construction
        let io_data = vec![0b1010_1010, 0b0101_0101];
        let cmd = WriteMultipleIo::new(2701, io_data.clone()).unwrap();
        assert_eq!(cmd.start_io_number, 2701);
        assert_eq!(cmd.io_data, io_data);

        // Valid construction with maximum count
        let io_data = vec![0u8; 474];
        let cmd = WriteMultipleIo::new(2701, io_data.clone()).unwrap();
        assert_eq!(cmd.start_io_number, 2701);
        assert_eq!(cmd.io_data.len(), 474);
    }

    #[test]
    fn test_write_multiple_io_validation() {
        // Invalid I/O number
        assert!(WriteMultipleIo::new(0, vec![0, 0]).is_err());
        assert!(WriteMultipleIo::new(65535, vec![0, 0]).is_err());

        // Invalid count - empty
        assert!(WriteMultipleIo::new(1, vec![]).is_err());

        // Invalid count - odd number
        assert!(WriteMultipleIo::new(1, vec![0]).is_err());
        assert!(WriteMultipleIo::new(1, vec![0, 0, 0]).is_err());

        // Invalid count - too large
        let large_data = vec![0u8; 475];
        assert!(WriteMultipleIo::new(1, large_data).is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_write_multiple_io_command_trait() {
        let io_data = vec![0b1010_1010, 0b0101_0101];
        let cmd = WriteMultipleIo::new(1, io_data).unwrap();
        assert_eq!(WriteMultipleIo::command_id(), 0x300);
        assert_eq!(cmd.instance(), 1);
        assert_eq!(cmd.attribute(), 0);
        assert_eq!(cmd.service(), 0x34);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_write_multiple_io_serialization() {
        let io_data = vec![0b1010_1010, 0b0101_0101];
        let cmd = WriteMultipleIo::new(1, io_data.clone()).unwrap();
        let payload = cmd.serialize().unwrap();

        // Expected: count (4 bytes) + io_data
        let mut expected = vec![2, 0, 0, 0]; // count = 2 in little-endian
        expected.extend_from_slice(&io_data);
        assert_eq!(payload, expected);
    }
}
