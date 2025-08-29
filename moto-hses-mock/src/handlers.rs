//! Command handlers for mock server

use moto_hses_proto as proto;
use crate::state::MockState;

/// Command handler trait
pub trait CommandHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError>;
}

/// Handler for alarm data reading (0x70)
pub struct AlarmDataHandler;

impl CommandHandler for AlarmDataHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let instance = message.sub_header.instance as usize;
        let attribute = message.sub_header.attribute;
        
        if instance == 0 || instance > state.alarms.len() {
            return Ok(vec![]); // No alarm
        }
        
        let alarm = &state.alarms[instance - 1];
        alarm.serialize(attribute)
    }
}

/// Handler for status reading (0x72)
pub struct StatusHandler;

impl CommandHandler for StatusHandler {
    fn handle(&self, _message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        use moto_hses_proto::VariableType;
        state.status.serialize()
    }
}

/// Handler for current position reading (0x75)
pub struct PositionHandler;

impl CommandHandler for PositionHandler {
    fn handle(&self, _message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        state.position.serialize()
    }
}

/// Handler for I/O operations (0x78)
pub struct IoHandler;

impl CommandHandler for IoHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let io_number = message.sub_header.instance;
        let service = message.sub_header.service;
        
        match service {
            0x0e => { // Read
                let value = if state.get_io_state(io_number) { 1 } else { 0 };
                Ok(vec![value, 0, 0, 0])
            }
            0x10 => { // Write
                if message.payload.len() >= 4 {
                    let value = i32::from_le_bytes([
                        message.payload[0],
                        message.payload[1],
                        message.payload[2],
                        message.payload[3],
                    ]);
                    state.set_io_state(io_number, value != 0);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for register operations (0x79)
pub struct RegisterHandler;

impl CommandHandler for RegisterHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let reg_number = message.sub_header.instance;
        let service = message.sub_header.service;
        
        match service {
            0x0e => { // Read
                let value = state.get_register(reg_number);
                Ok(value.to_le_bytes().to_vec())
            }
            0x10 => { // Write
                if message.payload.len() >= 4 {
                    let value = i32::from_le_bytes([
                        message.payload[0],
                        message.payload[1],
                        message.payload[2],
                        message.payload[3],
                    ]);
                    state.set_register(reg_number, value);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for byte variable operations (0x7a)
pub struct ByteVarHandler;

impl CommandHandler for ByteVarHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;
        
        match service {
            0x0e => { // Read
                if let Some(value) = state.get_variable(var_index) {
                    Ok(value.clone())
                } else {
                    Ok(vec![0])
                }
            }
            0x10 => { // Write
                if !message.payload.is_empty() {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for integer variable operations (0x7b)
pub struct IntegerVarHandler;

impl CommandHandler for IntegerVarHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;
        
        match service {
            0x0e => { // Read
                if let Some(value) = state.get_variable(var_index) {
                    Ok(value.clone())
                } else {
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => { // Write
                if message.payload.len() >= 4 {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for real variable operations (0x7d)
pub struct RealVarHandler;

impl CommandHandler for RealVarHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;
        
        match service {
            0x0e => { // Read
                if let Some(value) = state.get_variable(var_index) {
                    Ok(value.clone())
                } else {
                    Ok(vec![0, 0, 0, 0])
                }
            }
            0x10 => { // Write
                if message.payload.len() >= 4 {
                    state.set_variable(var_index, message.payload.clone());
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for position variable operations (0x7f)
pub struct PositionVarHandler;

impl CommandHandler for PositionVarHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let _var_index = message.sub_header.instance as u8;
        let service = message.sub_header.service;
        
        match service {
            0x0e => { // Read
                state.position.serialize()
            }
            0x10 => { // Write
                if message.payload.len() >= 52 {
                    if let Ok(position) = proto::Position::deserialize(&message.payload) {
                        state.update_position(position);
                    }
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}

/// Handler for alarm reset/error cancel (0x82)
pub struct AlarmResetHandler;

impl CommandHandler for AlarmResetHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let reset_type = message.sub_header.instance;
        
        match reset_type {
            1 => { // RESET
                state.clear_alarms();
            }
            2 => { // CANCEL
                state.status.error = false;
            }
            _ => {}
        }
        
        Ok(vec![])
    }
}

/// Handler for hold/servo control (0x83)
pub struct HoldServoHandler;

impl CommandHandler for HoldServoHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let control_type = message.sub_header.instance;
        
        if message.payload.len() >= 4 {
            let value = i32::from_le_bytes([
                message.payload[0],
                message.payload[1],
                message.payload[2],
                message.payload[3],
            ]);
            
            match control_type {
                1 => { // HOLD
                    state.set_hold(value == 1);
                }
                2 => { // Servo ON
                    state.set_servo(value == 1);
                }
                _ => {}
            }
        }
        
        Ok(vec![])
    }
}

/// Handler for job start (0x86)
pub struct JobStartHandler;

impl CommandHandler for JobStartHandler {
    fn handle(&self, _message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        state.set_running(true);
        Ok(vec![])
    }
}

/// Handler for job select (0x87)
pub struct JobSelectHandler;

impl CommandHandler for JobSelectHandler {
    fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let select_type = message.sub_header.instance;
        
        match select_type {
            1 => { // Set execution job
                if message.payload.len() >= 4 {
                    // In a real implementation, this would parse the job name
                    state.set_current_job(Some("SELECTED.JOB".to_string()));
                }
            }
            _ => {}
        }
        
        Ok(vec![])
    }
}

/// Command handler registry
pub struct CommandHandlerRegistry {
    handlers: std::collections::HashMap<u16, Box<dyn CommandHandler + Send + Sync>>,
}

impl CommandHandlerRegistry {
    pub fn new() -> Self {
        let mut handlers = std::collections::HashMap::new();
        
        handlers.insert(0x70, Box::new(AlarmDataHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x72, Box::new(StatusHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x75, Box::new(PositionHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x78, Box::new(IoHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x79, Box::new(RegisterHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7a, Box::new(ByteVarHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7b, Box::new(IntegerVarHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7d, Box::new(RealVarHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7f, Box::new(PositionVarHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x82, Box::new(AlarmResetHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x83, Box::new(HoldServoHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x86, Box::new(JobStartHandler) as Box<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x87, Box::new(JobSelectHandler) as Box<dyn CommandHandler + Send + Sync>);
        
        Self { handlers }
    }
    
    pub fn handle(&self, message: &proto::HsesMessage, state: &mut MockState) -> Result<Vec<u8>, proto::ProtocolError> {
        let command = message.sub_header.command;
        
        if let Some(handler) = self.handlers.get(&command) {
            handler.handle(message, state)
        } else {
            eprintln!("Unknown command: 0x{:04x}", command);
            Err(proto::ProtocolError::InvalidCommand)
        }
    }
}

impl Default for CommandHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
