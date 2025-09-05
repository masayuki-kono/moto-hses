//! Alarm-related command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for alarm data reading (0x70)
pub struct AlarmDataHandler;

impl CommandHandler for AlarmDataHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let instance = message.sub_header.instance as usize;
        let attribute = message.sub_header.attribute;

        // Python client expects 60 bytes: 4+4+4+16+32
        let mut data = vec![0u8; 60];

        if instance == 0 || instance > state.alarms.len() {
            // Default alarm data when no alarms exist
            data[0..4].copy_from_slice(&0u32.to_le_bytes()); // Default alarm code
            data[4..8].copy_from_slice(&0u32.to_le_bytes()); // Default alarm data
            data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                              // data[12..28] and data[28..60] remain 0 (default time and name)
        } else {
            let alarm = &state.alarms[instance - 1];

            if attribute == 0 {
                // For attribute 0, return complete alarm data
                let alarm_data = alarm.serialize_complete()?;

                // Copy complete alarm data to response
                if alarm_data.len() >= 60 {
                    data[..60].copy_from_slice(&alarm_data[..60]);
                } else {
                    data[..alarm_data.len()].copy_from_slice(&alarm_data);
                }
            } else {
                // For specific attributes, use the existing logic
                let alarm_data = alarm.serialize(attribute)?;

                // Copy alarm data to appropriate positions
                if alarm_data.len() >= 8 {
                    data[0..8].copy_from_slice(&alarm_data[0..8]);
                }
                // Fill remaining fields with default values
                data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                                  // data[12..28] and data[28..60] remain 0 (default time and name)
            }
        }

        Ok(data)
    }
}

/// Handler for alarm info reading (0x71)
pub struct AlarmInfoHandler;

impl CommandHandler for AlarmInfoHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let alarm_number = message.sub_header.instance as usize;
        let attribute = message.sub_header.attribute;

        // Python client expects 60 bytes: 4+4+4+16+32
        let mut data = vec![0u8; 60];

        if alarm_number == 0 || alarm_number > state.alarms.len() {
            // Default alarm data when no alarms exist
            data[0..4].copy_from_slice(&0u32.to_le_bytes()); // Default alarm code
            data[4..8].copy_from_slice(&0u32.to_le_bytes()); // Default alarm data
            data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                              // data[12..28] and data[28..60] remain 0 (default time and name)
        } else {
            let alarm = &state.alarms[alarm_number - 1];

            if attribute == 0 {
                // For attribute 0, return complete alarm data
                let alarm_data = alarm.serialize_complete()?;

                // Copy complete alarm data to response
                if alarm_data.len() >= 60 {
                    data[..60].copy_from_slice(&alarm_data[..60]);
                } else {
                    data[..alarm_data.len()].copy_from_slice(&alarm_data);
                }
            } else {
                // For specific attributes, use the existing logic
                let alarm_data = alarm.serialize(attribute)?;

                // Copy alarm data to appropriate positions
                if alarm_data.len() >= 8 {
                    data[0..8].copy_from_slice(&alarm_data[0..8]);
                }
                // Fill remaining fields with default values
                data[8..12].copy_from_slice(&0u32.to_le_bytes()); // Default alarm type
                                                                  // data[12..28] and data[28..60] remain 0 (default time and name)
            }
        }

        Ok(data)
    }
}

/// Handler for alarm reset/error cancel (0x82)
pub struct AlarmResetHandler;

impl CommandHandler for AlarmResetHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let reset_type = message.sub_header.instance;

        match reset_type {
            1 => {
                // RESET
                state.clear_alarms();
            }
            2 => {
                // CANCEL
                state.status.error = false;
            }
            _ => {}
        }

        Ok(vec![])
    }
}
