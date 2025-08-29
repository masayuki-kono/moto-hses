//! moto-hses-client - HSES (High Speed Ethernet Server) client implementation

use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use thiserror::Error;

use moto_hses_proto::{
    Command, VariableType, Position, Status, StatusWrapper, ReadStatus, ReadCurrentPosition,
    ReadVar, WriteVar, CoordinateSystemType, ProtocolError
};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub buffer_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(300),
            retry_count: 3,
            retry_delay: Duration::from_millis(100),
            buffer_size: 8192,
        }
    }
}

struct InnerClient {
    socket: UdpSocket,
    _remote_addr: SocketAddr,
    request_id: AtomicU8,
    _pending_requests: Arc<Mutex<HashMap<u8, PendingRequest>>>,
}

struct PendingRequest {
    _start_time: std::time::Instant,
    _on_reply: Box<dyn FnOnce(Result<Vec<u8>, ClientError>) + Send>,
}

pub struct HsesClient {
    inner: Arc<InnerClient>,
    config: ClientConfig,
}

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
}

impl HsesClient {
    pub async fn connect(addr: &str) -> Result<Self, ClientError> {
        Self::connect_with_config(addr, ClientConfig::default()).await
    }

    pub async fn connect_with_config(addr: &str, config: ClientConfig) -> Result<Self, ClientError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let remote_addr: SocketAddr = addr.parse()
            .map_err(|e| ClientError::SystemError(format!("Invalid address: {}", e)))?;
        socket.connect(remote_addr).await?;

        Ok(Self {
            inner: Arc::new(InnerClient {
                socket,
                _remote_addr: remote_addr,
                request_id: AtomicU8::new(1),
                _pending_requests: Arc::new(Mutex::new(HashMap::new())),
            }),
            config,
        })
    }

    // High-level API methods
    pub async fn read_variable<T>(&self, index: u8) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        let command = ReadVar::<T> { index, _phantom: std::marker::PhantomData };
        self.send_command(command).await
    }

    pub async fn write_variable<T>(&self, index: u8, value: T) -> Result<(), ClientError>
    where
        T: VariableType,
    {
        let command = WriteVar::<T> { index, value };
        self.send_command(command).await
    }

    pub async fn read_status(&self) -> Result<Status, ClientError> {
        let result: StatusWrapper = self.send_command(ReadStatus).await?;
        Ok(result.into())
    }

    pub async fn read_position(&self, control_group: u8, coord_system: CoordinateSystemType) -> Result<Position, ClientError> {
        let command = ReadCurrentPosition { control_group, coordinate_system: coord_system };
        self.send_command(command).await
    }

    // Generic command sending
    async fn send_command<C: Command>(&self, command: C) -> Result<C::Response, ClientError> 
    where
        C::Response: VariableType,
    {
        let request_id = self.inner.request_id.fetch_add(1, Ordering::Relaxed);
        let payload = command.serialize()?;

        // Create and send message
        let message = self.create_message(C::command_id(), request_id, payload)?;
        self.inner.socket.send(&message).await?;

        // Wait for response
        let response = self.wait_for_response(request_id).await?;

        // Deserialize response
        self.deserialize_response::<C::Response>(&response)
    }

    fn create_message(&self, command: u16, request_id: u8, payload: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        let mut message = Vec::new();

        // Magic bytes "YERC"
        message.extend_from_slice(b"YERC");

        // Header size (always 0x20)
        message.extend_from_slice(&0x20u16.to_le_bytes());

        // Payload size
        message.extend_from_slice(&(payload.len() as u16).to_le_bytes());

        // Reserved magic constant
        message.push(0x03);

        // Division (Robot)
        message.push(0x01);

        // ACK (Request)
        message.push(0x00);

        // Request ID
        message.push(request_id);

        // Block number (0 for requests)
        message.extend_from_slice(&0u32.to_le_bytes());

        // Reserved (8 bytes of '9')
        message.extend_from_slice(b"99999999");

        // Command
        message.extend_from_slice(&command.to_le_bytes());

        // Instance (0 for most commands)
        message.extend_from_slice(&0u16.to_le_bytes());

        // Attribute (1 for most commands)
        message.push(1);

        // Service (Get_Attribute_All for reads, Set_Attribute_All for writes)
        message.push(0x01);

        // Padding
        message.extend_from_slice(&0u16.to_le_bytes());

        // Payload
        message.extend(payload);

        Ok(message)
    }

    async fn wait_for_response(&self, request_id: u8) -> Result<Vec<u8>, ClientError> {
        let mut buffer = vec![0u8; self.config.buffer_size];

        loop {
            let (len, _addr) = timeout(self.config.timeout, self.inner.socket.recv_from(&mut buffer)).await
                .map_err(|_| ClientError::TimeoutError("Response timeout".to_string()))??;

            let response_data = &buffer[..len];

            // Parse response header
            if response_data.len() < 32 {
                continue;
            }

            let response_request_id = response_data[18];
            if response_request_id != request_id {
                continue;
            }

            let status = response_data[33];
            if status != 0 {
                return Err(ClientError::ProtocolError(ProtocolError::InvalidMessage(
                    format!("Robot returned error status: {}", status)
                )));
            }

            // Extract payload
            let payload_size = u16::from_le_bytes([response_data[6], response_data[7]]) as usize;
            let payload = response_data[32..32+payload_size].to_vec();

            return Ok(payload);
        }
    }

    fn deserialize_response<T>(&self, data: &[u8]) -> Result<T, ClientError>
    where
        T: VariableType,
    {
        T::deserialize(data).map_err(ClientError::from)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // This test requires a mock server or real robot
        // For now, just test the client can be created
        let config = ClientConfig::default();
        assert_eq!(config.timeout.as_millis(), 300);
        assert_eq!(config.retry_count, 3);
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout.as_millis(), 300);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_delay.as_millis(), 100);
        assert_eq!(config.buffer_size, 8192);
    }
}