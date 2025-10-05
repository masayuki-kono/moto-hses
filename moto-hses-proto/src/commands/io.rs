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
}
