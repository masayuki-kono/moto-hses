//! Command handlers for mock server
//!
//! This module contains all command handlers organized by functionality.

use crate::state::MockState;
use moto_hses_proto as proto;

/// Command handler trait
pub trait CommandHandler {
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError>;
}

// Re-export all handler modules
pub mod alarm;
pub mod file;
pub mod io;
pub mod job;
pub mod position;
pub mod registry;
pub mod system;
pub mod variable;

// Re-export the registry
pub use registry::CommandHandlerRegistry;
