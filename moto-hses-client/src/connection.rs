//! Connection management for HSES client

use tokio::time::{timeout, sleep};
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
                connected: std::sync::Arc::new(std::sync::Mutex::new(false)),
            }),
            config,
        };

        client.connect().await?;
        Ok(client)
    }

    /// Establish connection to the robot
    async fn connect(&self) -> Result<(), ClientError> {
        let mut retries = 0;
        let max_retries = self.config.retry_count;

        while retries < max_retries {
            match timeout(self.config.connection_timeout, self.inner.socket.connect(self.inner.remote_addr)).await {
                Ok(Ok(())) => {
                    // Mark as connected - actual communication test will be done on first operation
                    *self.inner.connected.lock().unwrap() = true;
                    return Ok(());
                }
                Ok(Err(e)) => {
                    eprintln!("Connection attempt {} failed: {}", retries + 1, e);
                }
                Err(_) => {
                    eprintln!("Connection attempt {} timed out", retries + 1);
                }
            }

            retries += 1;
            if retries < max_retries {
                sleep(self.config.retry_delay).await;
            }
        }

        Err(ClientError::ConnectionFailed(max_retries))
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        *self.inner.connected.lock().unwrap()
    }

    /// Reconnect to the robot
    pub async fn reconnect(&self) -> Result<(), ClientError> {
        *self.inner.connected.lock().unwrap() = false;
        self.connect().await
    }

    /// Ensure client is connected before operations
    pub(crate) fn ensure_connected(&self) -> Result<(), ClientError> {
        if !self.is_connected() {
            return Err(ClientError::NotConnected);
        }
        Ok(())
    }
}
