//! Command handlers for mock server
//!
//! This module contains all command handlers organized by functionality.

use crate::state::MockState;
use moto_hses_proto as proto;

/// Command handler trait
pub trait CommandHandler {
    /// Handle a command message
    ///
    /// # Errors
    ///
    /// Returns an error if command processing fails
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError>;
}

// Re-export all handler modules
pub mod alarm;
pub mod cycle_mode_switching;
pub mod file;
pub mod io;
pub mod job;
pub mod position;
pub mod registry;
pub mod system;
pub mod variable;

// Re-export the registry
pub use registry::CommandHandlerRegistry;
