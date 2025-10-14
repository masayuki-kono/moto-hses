//! Alarm-related command handlers

use crate::{state::MockState, CommandHandler};
use moto_hses_proto::{
    encoding_utils, Alarm, AlarmAttribute, HsesRequestMessage, ProtocolError, ReadAlarmData,
    ReadAlarmHistory,
};

/// Common helper function to handle alarm attribute reading based on service type
fn handle_alarm_service_request(
    alarm: &Alarm,
    service: u8,
    attribute: u8,
    state: &MockState,
) -> Result<Vec<u8>, ProtocolError> {
    match service {
        0x01 => {
            // Service = 0x01 (Get_Attribute_All) - Return complete alarm data (60 bytes)
            alarm.serialize_complete(state.text_encoding)
        }
        0x0E => {
            // Service = 0x0E (Get_Attribute_Single) - Return specific attribute data
            Ok(get_alarm_attribute_data(alarm, attribute, state))
        }
        _ => {
            // Invalid service - return empty data
            Ok(vec![0u8; 4])
        }
    }
}

/// Common helper function to get specific alarm attribute data
fn get_alarm_attribute_data(alarm: &Alarm, attribute: u8, state: &MockState) -> Vec<u8> {
    match attribute {
        1 => {
            // Alarm code (4 bytes)
            alarm.code.to_le_bytes().to_vec()
        }
        2 => {
            // Alarm data (4 bytes)
            alarm.data.to_le_bytes().to_vec()
        }
        3 => {
            // Alarm type (4 bytes)
            alarm.alarm_type.to_le_bytes().to_vec()
        }
        4 => {
            // Alarm time (16 bytes)
            let time_bytes = encoding_utils::encode_string(&alarm.time, state.text_encoding);
            let mut padded_time = vec![0u8; 16];
            padded_time[..time_bytes.len().min(16)]
                .copy_from_slice(&time_bytes[..time_bytes.len().min(16)]);
            padded_time
        }
        5 => {
            // Alarm name (32 bytes)
            let name_bytes = encoding_utils::encode_string(&alarm.name, state.text_encoding);
            let mut padded_name = vec![0u8; 32];
            padded_name[..name_bytes.len().min(32)]
                .copy_from_slice(&name_bytes[..name_bytes.len().min(32)]);
            padded_name
        }
        _ => {
            // Invalid attribute - return empty data
            vec![0u8; 4]
        }
    }
}

/// Handler for alarm data reading (0x70)
pub struct AlarmDataHandler;

impl CommandHandler for AlarmDataHandler {
    fn handle(
        &self,
        message: &HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, ProtocolError> {
        let instance = message.sub_header.instance;
        let attribute = message.sub_header.attribute;
        let service = message.sub_header.service;

        // Create ReadAlarmData command to validate instance and attribute
        let alarm_data_cmd = ReadAlarmData::new(instance, AlarmAttribute::from(attribute));

        // Validate instance range
        if !alarm_data_cmd.is_valid_instance() {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid alarm instance: {instance} (valid range: 1-1000)"
            )));
        }

        let instance_usize = instance as usize;
        if instance_usize == 0 || instance_usize > state.alarms.len() {
            // No alarm found - return empty data
            return Ok(vec![0u8; 4]);
        }

        let alarm = &state.alarms[instance_usize - 1];
        handle_alarm_service_request(alarm, service, attribute, state)
    }
}

/// Handler for alarm history reading (0x71)
pub struct AlarmInfoHandler;

impl CommandHandler for AlarmInfoHandler {
    fn handle(
        &self,
        message: &HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, ProtocolError> {
        let instance = message.sub_header.instance;
        let attribute = message.sub_header.attribute;
        let service = message.sub_header.service;

        // Create ReadAlarmHistory command to validate instance
        let alarm_history_cmd = ReadAlarmHistory::new(instance, AlarmAttribute::from(attribute));

        // Validate instance range
        if !alarm_history_cmd.is_valid_instance() {
            return Err(ProtocolError::InvalidInstance(format!(
                "Invalid alarm history instance: {instance} (valid range: 1-1000)"
            )));
        }

        let category = alarm_history_cmd.get_alarm_category();
        let index = alarm_history_cmd.get_alarm_index();

        // Get alarm from history
        state.alarm_history.get_alarm(category, index).map_or_else(
            || {
                // No alarm found at this index - return empty data
                Ok(vec![0u8; 4])
            },
            |alarm| handle_alarm_service_request(alarm, service, attribute, state),
        )
    }
}

/// Handler for alarm reset/error cancel (0x82)
pub struct AlarmResetHandler;

impl CommandHandler for AlarmResetHandler {
    fn handle(
        &self,
        message: &HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, ProtocolError> {
        let reset_type = message.sub_header.instance;

        match reset_type {
            1 => {
                // RESET
                state.clear_alarms();
            }
            2 => {
                // CANCEL
                state.status.data2.error = false;
            }
            _ => {}
        }

        Ok(vec![])
    }
}
