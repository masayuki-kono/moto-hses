//! Type definitions for HSES client

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU8;
use std::time::Duration;
use thiserror::Error;
use tokio::net::UdpSocket;

use moto_hses_proto::ProtocolError;

/// Client configuration options
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub buffer_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: moto_hses_proto::ROBOT_CONTROL_PORT,
            timeout: Duration::from_millis(300),
            retry_count: 3,
            retry_delay: Duration::from_millis(100),
            buffer_size: 8192,
        }
    }
}

/// Internal client state
pub(crate) struct InnerClient {
    pub socket: UdpSocket,
    pub remote_addr: SocketAddr,
    pub request_id: AtomicU8,
    pub _pending_requests: Arc<Mutex<HashMap<u8, PendingRequest>>>,
}

/// Pending request tracking
pub(crate) struct PendingRequest {
    pub _start_time: std::time::Instant,
    pub _on_reply: Box<dyn FnOnce(Result<Vec<u8>, ClientError>) + Send>,
}

/// Main HSES client
pub struct HsesClient {
    pub(crate) inner: Arc<InnerClient>,
    pub config: ClientConfig,
}

/// Client-specific errors
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("Invalid variable: {0}")]
    InvalidVariable(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Connection failed after {0} retries")]
    ConnectionFailed(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout.as_millis(), 300);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_delay.as_millis(), 100);
        assert_eq!(config.buffer_size, 8192);
    }

    #[test]
    fn test_client_error_display() {
        let error = ClientError::TimeoutError("test timeout".to_string());
        assert_eq!(error.to_string(), "Timeout error: test timeout");

        let error = ClientError::SystemError("test error".to_string());
        assert_eq!(error.to_string(), "System error: test error");
    }
}
