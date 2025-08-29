//! moto-hses-mock - Local mock HSES UDP server for testing

use std::collections::HashMap;
use std::net::SocketAddr;
use moto_hses_proto as proto;

pub mod server;
pub mod state;
pub mod handlers;

pub use server::MockServer;
pub use state::MockState;
pub use handlers::CommandHandler;

/// Mock server configuration
#[derive(Debug, Clone)]
pub struct MockConfig {
    pub bind_addr: SocketAddr,
    pub default_status: proto::Status,
    pub default_position: proto::Position,
    pub variables: HashMap<u8, Vec<u8>>,
    pub io_states: HashMap<u16, bool>,
}

impl Default for MockConfig {
    fn default() -> Self {
        let mut variables = HashMap::new();
        variables.insert(0, vec![0x01, 0x00, 0x00, 0x00]); // D000 = 1
        variables.insert(1, vec![0x64, 0x00, 0x00, 0x00]); // D001 = 100
        variables.insert(2, vec![0x00, 0x00, 0x20, 0x41]); // D002 = 10.0

        let mut io_states = HashMap::new();
        io_states.insert(1, true);   // Robot user input 1
        io_states.insert(1001, false); // Robot user output 1

        Self {
            bind_addr: "127.0.0.1:12222".parse().unwrap(),
            default_status: proto::Status {
                step: false,
                one_cycle: false,
                continuous: true,
                running: true,
                speed_limited: false,
                teach: false,
                play: true,
                remote: false,
                teach_pendant_hold: false,
                external_hold: false,
                command_hold: false,
                alarm: false,
                error: false,
                servo_on: true,
            },
            default_position: proto::Position::Pulse(proto::PulsePosition::new(
                [0, 0, 0, 0, 0, 0, 0, 0],
                1
            )),
            variables,
            io_states,
        }
    }
}

/// Test utilities for mock server
pub mod test_utils {
    use super::*;
    use tokio::time::{sleep, Duration};

    /// Start a mock server for testing
    pub async fn start_test_server() -> Result<(SocketAddr, tokio::task::JoinHandle<()>), Box<dyn std::error::Error + Send + Sync>> {
        // Use a random port to avoid conflicts
        let config = MockConfig {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        };
        let server = MockServer::new(config).await?;
        let addr = server.local_addr()?;
        
        let handle = tokio::spawn(async move {
            if let Err(e) = server.run().await {
                eprintln!("Mock server error: {}", e);
            }
        });

        // Give server time to start
        sleep(Duration::from_millis(100)).await;
        
        Ok((addr, handle))
    }

    /// Create a test client connected to mock server
    pub async fn create_test_client(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Note: This would require the client to be available
        // For now, just return a placeholder
        eprintln!("Test client would connect to {}", addr);
        Ok(())
    }
}
