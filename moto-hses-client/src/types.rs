//! Type definitions for HSES client

use std::time::Duration;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use tokio::net::UdpSocket;
use thiserror::Error;

use moto_hses_proto::ProtocolError;

/// Client configuration options
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub buffer_size: usize,
    pub connection_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(300),
            retry_count: 3,
            retry_delay: Duration::from_millis(100),
            buffer_size: 8192,
            connection_timeout: Duration::from_secs(5),
        }
    }
}

/// Internal client state
pub(crate) struct InnerClient {
    pub socket: UdpSocket,
    pub remote_addr: SocketAddr,
    pub request_id: AtomicU8,
    pub _pending_requests: Arc<Mutex<HashMap<u8, PendingRequest>>>,
    pub connected: Arc<Mutex<bool>>,
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
    #[error("Not connected")]
    NotConnected,
    #[error("Connection failed after {0} retries")]
    ConnectionFailed(u32),
}
