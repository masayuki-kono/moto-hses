//! Connection management for HSES client



use crate::types::{HsesClient, ClientError, ClientConfig, InnerClient};

impl HsesClient {
    /// Create a new client with default configuration
    pub async fn new(addr: &str) -> Result<Self, ClientError> {
        Self::new_with_config(addr, ClientConfig::default()).await
    }

    /// Create a new client with custom configuration
    pub async fn new_with_config(addr: &str, config: ClientConfig) -> Result<Self, ClientError> {
        let client = Self {
            inner: std::sync::Arc::new(InnerClient {
                socket: tokio::net::UdpSocket::bind("0.0.0.0:0").await?,
                remote_addr: addr.parse()
                    .map_err(|e| ClientError::SystemError(format!("Invalid address: {}", e)))?,
                request_id: std::sync::atomic::AtomicU8::new(1),
                _pending_requests: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            }),
            config,
        };

        Ok(client)
    }








}
