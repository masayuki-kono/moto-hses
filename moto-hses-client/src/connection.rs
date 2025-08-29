//! Connection management for HSES client


use tokio::time::timeout;
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
        // For UDP, we need to test actual communication to verify the server exists
        // Send a simple ping message and wait for response
        let ping_message = self.create_ping_message()?;
        
        // Send ping with timeout
        match timeout(self.config.connection_timeout, 
            self.inner.socket.send_to(&ping_message, self.inner.remote_addr)).await {
            Ok(Ok(_)) => {
                // Try to receive response
                match timeout(self.config.connection_timeout, 
                    self.recv_ping_response()).await {
                    Ok(Ok(_)) => {
                        *self.inner.connected.lock().unwrap() = true;
                        Ok(())
                    }
                    Ok(Err(_)) | Err(_) => {
                        Err(ClientError::ConnectionFailed(1))
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("Failed to send ping: {}", e);
                Err(ClientError::ConnectionError(e))
            }
            Err(_) => {
                Err(ClientError::ConnectionFailed(1))
            }
        }
    }
    
    /// Create a simple ping message for connection testing
    fn create_ping_message(&self) -> Result<Vec<u8>, ClientError> {
        // Create a minimal HSES message for testing
        let mut message = Vec::new();
        
        // Magic bytes "YERC"
        message.extend_from_slice(b"YERC");
        // Header size (always 0x20)
        message.extend_from_slice(&0x20u16.to_le_bytes());
        // Payload size (0 for ping)
        message.extend_from_slice(&0u16.to_le_bytes());
        // Reserved magic constant
        message.push(0x03);
        // Division (Robot)
        message.push(0x01);
        // ACK (Request)
        message.push(0x00);
        // Request ID (0 for ping)
        message.push(0x00);
        // Block number (0 for requests)
        message.extend_from_slice(&0u32.to_le_bytes());
        // Reserved (8 bytes of '9')
        message.extend_from_slice(b"99999999");
        // Command (0x72 - Read Status, safe command)
        message.extend_from_slice(&0x72u16.to_le_bytes());
        // Instance (0)
        message.extend_from_slice(&0u16.to_le_bytes());
        // Attribute (1)
        message.push(1);
        // Service (Get_Attribute_Single)
        message.push(0x0e);
        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());
        
        Ok(message)
    }
    
    /// Receive ping response
    async fn recv_ping_response(&self) -> Result<(), ClientError> {
        let mut buffer = vec![0u8; 1024];
        
        loop {
            let (len, addr) = self.inner.socket.recv_from(&mut buffer).await?;
            
            // Verify it's from the expected server
            if addr.ip() == self.inner.remote_addr.ip() {
                let response_data = &buffer[..len];
                
                // Basic validation - check magic bytes and ACK
                if response_data.len() >= 11 && 
                   &response_data[0..4] == b"YERC" && 
                   response_data[10] == 0x01 {
                    return Ok(());
                }
            }
        }
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
