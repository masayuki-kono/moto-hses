//! Connection management for HSES client

use crate::types::{ClientConfig, ClientError, HsesClient, InnerClient};

impl HsesClient {
    /// Create a new client with default configuration
    pub async fn new(addr: &str) -> Result<Self, ClientError> {
        let mut config = ClientConfig::default();
        // Parse address and update config
        let addr_parts: Vec<&str> = addr.split(':').collect();
        if addr_parts.len() == 2 {
            config.host = addr_parts[0].to_string();
            config.port = addr_parts[1]
                .parse()
                .map_err(|e| ClientError::SystemError(format!("Invalid port: {}", e)))?;
        } else {
            return Err(ClientError::SystemError(
                "Invalid address format. Use 'host:port'".to_string(),
            ));
        }
        Self::new_with_config(config).await
    }

    /// Create a new client with custom configuration
    pub async fn new_with_config(config: ClientConfig) -> Result<Self, ClientError> {
        let addr = format!("{}:{}", config.host, config.port);
        let client = Self {
            inner: std::sync::Arc::new(InnerClient {
                socket: tokio::net::UdpSocket::bind("0.0.0.0:0").await?,
                remote_addr: addr
                    .parse()
                    .map_err(|e| ClientError::SystemError(format!("Invalid address: {}", e)))?,
                request_id: std::sync::atomic::AtomicU8::new(1),
                _pending_requests: std::sync::Arc::new(std::sync::Mutex::new(
                    std::collections::HashMap::new(),
                )),
            }),
            config,
        };

        Ok(client)
    }
}
