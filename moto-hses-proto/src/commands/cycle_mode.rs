//! Cycle mode switching command (0x84)

use super::command_trait::Command;
use crate::error::ProtocolError;

/// Cycle mode switching command (0x84)
#[derive(Debug, Clone)]
pub struct CycleModeSwitchingCommand {
    pub mode: CycleMode,
}

impl CycleModeSwitchingCommand {
    #[must_use]
    pub const fn new(mode: CycleMode) -> Self {
        Self { mode }
    }
}

/// Cycle mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleMode {
    Step = 1,
    OneCycle = 2,
    Continuous = 3,
}

impl Command for CycleModeSwitchingCommand {
    type Response = ();

    fn command_id() -> u16 {
        0x84
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(vec![self.mode as u8, 0, 0, 0])
    }

    fn instance(&self) -> u16 {
        2 // Fixed according to specification
    }

    fn attribute(&self) -> u8 {
        1 // Fixed according to specification
    }

    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_mode_switching_command_new() {
        let command = CycleModeSwitchingCommand::new(CycleMode::Step);
        assert_eq!(command.mode, CycleMode::Step);
    }

    #[test]
    fn test_cycle_mode_switching_command_trait() {
        let command = CycleModeSwitchingCommand::new(CycleMode::OneCycle);

        assert_eq!(CycleModeSwitchingCommand::command_id(), 0x84);
        assert_eq!(command.instance(), 2);
        assert_eq!(command.attribute(), 1);
        assert_eq!(command.service(), 0x10);
    }

    #[test]
    fn test_cycle_mode_switching_command_serialize() {
        let command = CycleModeSwitchingCommand::new(CycleMode::Step);
        let data = command.serialize().unwrap();
        assert_eq!(data, vec![1, 0, 0, 0]);

        let command = CycleModeSwitchingCommand::new(CycleMode::OneCycle);
        let data = command.serialize().unwrap();
        assert_eq!(data, vec![2, 0, 0, 0]);

        let command = CycleModeSwitchingCommand::new(CycleMode::Continuous);
        let data = command.serialize().unwrap();
        assert_eq!(data, vec![3, 0, 0, 0]);
    }

    #[test]
    fn test_cycle_mode_enum_values() {
        assert_eq!(CycleMode::Step as u8, 1);
        assert_eq!(CycleMode::OneCycle as u8, 2);
        assert_eq!(CycleMode::Continuous as u8, 3);
    }

    #[test]
    fn test_cycle_mode_equality() {
        assert_eq!(CycleMode::Step, CycleMode::Step);
        assert_ne!(CycleMode::Step, CycleMode::OneCycle);
        assert_ne!(CycleMode::OneCycle, CycleMode::Continuous);
    }
}
