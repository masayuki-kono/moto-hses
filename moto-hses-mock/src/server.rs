//! Mock HSES server implementation

use std::net::SocketAddr;
use tokio::net::UdpSocket;
use moto_hses_proto as proto;
use crate::state::{MockState, SharedState};
use crate::handlers::CommandHandlerRegistry;

/// Mock HSES server
pub struct MockServer {
    socket: UdpSocket,
    state: SharedState,
    handlers: CommandHandlerRegistry,
}

impl MockServer {
    /// Create a new mock server
    pub async fn new(config: crate::MockConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let socket = UdpSocket::bind(config.bind_addr).await?;
        let state = SharedState::new(MockState::default());
        let handlers = CommandHandlerRegistry::default();
        
        eprintln!("Mock server listening on {}", config.bind_addr);
        
        Ok(Self {
            socket,
            state,
            handlers,
        })
    }
    
    /// Get the local address of the server
    pub fn local_addr(&self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.socket.local_addr()?)
    }
    
    /// Run the server
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = vec![0u8; 2048];
        
        loop {
            let (n, src) = self.socket.recv_from(&mut buf).await?;
            
            if n < 32 {
                eprintln!("Received message too short: {} bytes", n);
                continue;
            }
            
            // Parse HSES message
            let message = match proto::HsesMessage::decode(&buf[..n]) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Failed to decode message: {:?}", e);
                    continue;
                }
            };
            
            eprintln!("Received command: 0x{:04x} from {}", message.sub_header.command, src);
            
            // Handle the message
            let response = self.handle_message(&message).await;
            
            // Send response
            if let Ok(response_data) = response {
                if !response_data.is_empty() {
                    eprintln!("Sending response: {} bytes", response_data.len());
                    let _ = self.socket.send_to(&response_data, src).await?;
                }
            } else {
                eprintln!("Error handling message: {:?}", response.err());
            }
        }
    }
    
    /// Handle a single message
    async fn handle_message(&self, message: &proto::HsesMessage) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        
        // Handle the command
        let payload = self.handlers.handle(message, &mut state)?;
        
        // Create response message
        let response_message = proto::HsesMessage::new(
            message.header.division,
            0x01, // ACK
            message.header.request_id,
            message.sub_header.command,
            message.sub_header.instance,
            message.sub_header.attribute,
            message.sub_header.service + 0x80, // Add 0x80 to service for response
            payload,
        );
        
        Ok(response_message.encode().to_vec())
    }
    
    /// Get a reference to the shared state
    pub fn state(&self) -> &SharedState {
        &self.state
    }
    
    /// Add a test alarm to the server state
    pub async fn add_test_alarm(&self, alarm: proto::Alarm) {
        let mut state = self.state.write().await;
        state.add_alarm(alarm);
    }
    
    /// Set a variable in the server state
    pub async fn set_variable(&self, index: u8, value: Vec<u8>) {
        let mut state = self.state.write().await;
        state.set_variable(index, value);
    }
    
    /// Set an I/O state in the server state
    pub async fn set_io_state(&self, io_number: u16, state: bool) {
        let mut server_state = self.state.write().await;
        server_state.set_io_state(io_number, state);
    }
    
    /// Set the robot status
    pub async fn set_status(&self, status: proto::Status) {
        let mut state = self.state.write().await;
        state.status = status;
    }
    
    /// Set the robot position
    pub async fn set_position(&self, position: proto::Position) {
        let mut state = self.state.write().await;
        state.update_position(position);
    }
}

/// Server builder for easy configuration
pub struct MockServerBuilder {
    config: crate::MockConfig,
}

impl MockServerBuilder {
    pub fn new() -> Self {
        Self {
            config: crate::MockConfig::default(),
        }
    }
    
    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.config.bind_addr = addr;
        self
    }
    
    pub fn with_alarm(self, _alarm: proto::Alarm) -> Self {
        // Note: This would need to be applied after server creation
        // since we can't modify the config's state directly
        self
    }
    
    pub fn with_variable(mut self, index: u8, value: Vec<u8>) -> Self {
        self.config.variables.insert(index, value);
        self
    }
    
    pub fn with_io_state(mut self, io_number: u16, state: bool) -> Self {
        self.config.io_states.insert(io_number, state);
        self
    }
    
    pub async fn build(self) -> Result<MockServer, Box<dyn std::error::Error + Send + Sync>> {
        MockServer::new(self.config).await
    }
}

impl Default for MockServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
