//! Command handler registry

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;
use std::sync::Arc;

// Import all handlers
use super::alarm::{AlarmDataHandler, AlarmInfoHandler, AlarmResetHandler};
use super::cycle_mode_switching::CycleModeSwitchingHandler;
use super::file::FileControlHandler;
use super::io::{IoHandler, PluralIoHandler};
use super::job::{
    ExecutingJobInfoHandler, JobSelectHandler, JobStartHandler, MovHandler, PmovHandler,
};
use super::position::{
    BasePositionVarHandler, ExternalAxisVarHandler, PositionErrorHandler, PositionHandler,
    PositionVarHandler,
};
use super::register::{PluralRegisterHandler, RegisterHandler};
use super::system::{
    AxisNameHandler, HoldServoHandler, ManagementTimeHandler, StatusHandler, SystemInfoHandler,
    TextDisplayHandler, TorqueHandler,
};
use super::variable::{
    ByteVarHandler, DoubleVarHandler, IntegerVarHandler, PluralByteVarHandler,
    PluralDoubleVarHandler, PluralIntegerVarHandler, PluralRealVarHandler, RealVarHandler,
    StringVarHandler,
};

/// Command handler registry
#[derive(Clone)]
pub struct CommandHandlerRegistry {
    handlers: std::collections::HashMap<u16, Arc<dyn CommandHandler + Send + Sync>>,
}

impl CommandHandlerRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut handlers = std::collections::HashMap::new();

        // File operations
        handlers
            .insert(0x00, Arc::new(FileControlHandler) as Arc<dyn CommandHandler + Send + Sync>);

        // Alarm handlers
        handlers.insert(0x70, Arc::new(AlarmDataHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x71, Arc::new(AlarmInfoHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x82, Arc::new(AlarmResetHandler) as Arc<dyn CommandHandler + Send + Sync>);

        // System information handlers
        handlers.insert(0x72, Arc::new(StatusHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(
            0x73,
            Arc::new(ExecutingJobInfoHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(0x74, Arc::new(AxisNameHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x77, Arc::new(TorqueHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers
            .insert(0x85, Arc::new(TextDisplayHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers
            .insert(0x88, Arc::new(ManagementTimeHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x89, Arc::new(SystemInfoHandler) as Arc<dyn CommandHandler + Send + Sync>);

        // Position handlers
        handlers.insert(0x75, Arc::new(PositionHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers
            .insert(0x76, Arc::new(PositionErrorHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers
            .insert(0x7f, Arc::new(PositionVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(
            0x80,
            Arc::new(BasePositionVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x81,
            Arc::new(ExternalAxisVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );

        // I/O handlers
        handlers.insert(0x78, Arc::new(IoHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x300, Arc::new(PluralIoHandler) as Arc<dyn CommandHandler + Send + Sync>);

        // Register handlers
        handlers.insert(0x79, Arc::new(RegisterHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(
            0x301,
            Arc::new(PluralRegisterHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );

        // Variable handlers
        handlers.insert(0x7a, Arc::new(ByteVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7b, Arc::new(IntegerVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7c, Arc::new(DoubleVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7d, Arc::new(RealVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x7e, Arc::new(StringVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers
            .insert(0x302, Arc::new(PluralByteVarHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(
            0x303,
            Arc::new(PluralIntegerVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(
            0x304,
            Arc::new(PluralDoubleVarHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers
            .insert(0x305, Arc::new(PluralRealVarHandler) as Arc<dyn CommandHandler + Send + Sync>);

        // Job and movement handlers
        handlers.insert(0x83, Arc::new(HoldServoHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(
            0x84,
            Arc::new(CycleModeSwitchingHandler) as Arc<dyn CommandHandler + Send + Sync>,
        );
        handlers.insert(0x86, Arc::new(JobStartHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x87, Arc::new(JobSelectHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x8a, Arc::new(MovHandler) as Arc<dyn CommandHandler + Send + Sync>);
        handlers.insert(0x8b, Arc::new(PmovHandler) as Arc<dyn CommandHandler + Send + Sync>);

        Self { handlers }
    }

    /// # Errors
    ///
    /// Returns an error if command handling fails
    pub fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let command = message.sub_header.command;

        self.handlers.get(&command).map_or_else(
            || {
                debug!("Unknown command: 0x{command:04x}");
                Err(proto::ProtocolError::InvalidCommand)
            },
            |handler| handler.handle(message, state),
        )
    }
}

impl Default for CommandHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
