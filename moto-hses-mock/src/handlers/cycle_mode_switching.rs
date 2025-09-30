//! Cycle mode switching command handler (0x84)

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for cycle mode switching command (0x84)
pub struct CycleModeSwitchingHandler;

impl CommandHandler for CycleModeSwitchingHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        // Validate instance (must be 2)
        if message.sub_header.instance != 2 {
            return Err(proto::ProtocolError::InvalidCommand);
        }

        // Validate attribute (must be 1)
        if message.sub_header.attribute != 1 {
            return Err(proto::ProtocolError::InvalidAttribute);
        }

        // Validate service (must be 0x10)
        if message.sub_header.service != 0x10 {
            return Err(proto::ProtocolError::InvalidService);
        }

        // Parse cycle mode from payload
        if message.payload.len() < 4 {
            return Err(proto::ProtocolError::Deserialization(
                "Insufficient payload length".to_string(),
            ));
        }

        let mode_value = u32::from_le_bytes([
            message.payload[0],
            message.payload[1],
            message.payload[2],
            message.payload[3],
        ]);

        let mode = match mode_value {
            1 => proto::CycleMode::Step,
            2 => proto::CycleMode::OneCycle,
            3 => proto::CycleMode::Continuous,
            _ => return Err(proto::ProtocolError::InvalidAttribute),
        };

        // Update state
        state.set_cycle_mode(mode);

        // Return empty payload (success response)
        Ok(vec![])
    }
}
