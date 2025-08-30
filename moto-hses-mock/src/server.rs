//! Mock HSES server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use moto_hses_proto as proto;
use crate::state::{MockState, SharedState};
use crate::handlers::CommandHandlerRegistry;

/// Mock HSES server
pub struct MockServer {
    robot_socket: Arc<UdpSocket>,
    file_socket: Arc<UdpSocket>,
    state: SharedState,
    handlers: CommandHandlerRegistry,
}

impl MockServer {
    /// Create a new mock server
    pub async fn new(config: crate::MockConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let robot_socket = Arc::new(UdpSocket::bind(config.bind_addr).await?);
        
        // Use configured file port or fallback to bind_addr.port() + 1
        let file_port = config.file_port.unwrap_or(config.bind_addr.port() + 1);
        let file_addr: SocketAddr = format!("127.0.0.1:{}", file_port).parse()?;
        let file_socket = Arc::new(UdpSocket::bind(file_addr).await?);
        
        let state = SharedState::new(MockState::default());
        let handlers = CommandHandlerRegistry::default();
        
        eprintln!("Mock server listening on {}", config.bind_addr);
        eprintln!("Mock server listening on {}", file_addr);
        
        Ok(Self {
            robot_socket,
            file_socket,
            state,
            handlers,
        })
    }
    
    /// Get the local address of the server
    pub fn local_addr(&self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.robot_socket.local_addr()?)
    }
    
    /// Run the server
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Create a task for each socket
        let robot_socket = Arc::clone(&self.robot_socket);
        let file_socket = Arc::clone(&self.file_socket);
        
        let robot_task = {
            let state = self.state.clone();
            let handlers = self.handlers.clone();
            tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            loop {
                let (n, src) = match robot_socket.recv_from(&mut buf).await {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("Error receiving from robot socket: {:?}", e);
                        continue;
                    }
                };
                
                if n < 32 {
                    eprintln!("Received message too short: {} bytes", n);
                    continue;
                }
                
                // Parse HSES message as request (since server only receives requests)
                let message = match proto::HsesRequestMessage::decode(&buf[..n]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Failed to decode message: {:?}", e);
                        continue;
                    }
                };
                
                eprintln!("Received packet from {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[command=0x{:04x}, instance={}, attribute={}, service={}], Payload[{} bytes: {:02x?}]", 
                    src,
                    message.header.division,
                    message.header.ack,
                    message.header.request_id,
                    message.header.payload_size,
                    message.sub_header.command,
                    message.sub_header.instance,
                    message.sub_header.attribute,
                    message.sub_header.service,
                    message.payload.len(),
                    message.payload
                );
                
                // Handle the message
                let response = Self::handle_message_internal(&message, &state, &handlers).await;
                
                // Send response
                if let Ok(response_data) = response {
                    if !response_data.is_empty() {
                        // Decode response message for detailed logging
                        if let Ok(response_message) = proto::HsesResponseMessage::decode(&response_data) {
                            eprintln!("Sending response to {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[service={}, status={}, added_status_size={}, added_status={}], Payload[{} bytes: {:02x?}]", 
                                src,
                                response_message.header.division,
                                response_message.header.ack,
                                response_message.header.request_id,
                                response_message.header.payload_size,
                                response_message.sub_header.service,
                                response_message.sub_header.status,
                                response_message.sub_header.added_status_size,
                                response_message.sub_header.added_status,
                                response_message.payload.len(),
                                response_message.payload
                            );
                        } else {
                            eprintln!("Sending response: {} bytes (failed to decode for detailed logging)", response_data.len());
                        }
                        if let Err(e) = robot_socket.send_to(&response_data, src).await {
                            eprintln!("Error sending response: {:?}", e);
                        }
                    }
                } else {
                    eprintln!("Error handling message: {:?}", response.err());
                }
            }
        })
        };
        
        let file_task = {
            let state = self.state.clone();
            let handlers = self.handlers.clone();
            tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            loop {
                let (n, src) = match file_socket.recv_from(&mut buf).await {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("Error receiving from file socket: {:?}", e);
                        continue;
                    }
                };
                
                if n < 32 {
                    eprintln!("Received file message too short: {} bytes", n);
                    continue;
                }
                
                // Parse HSES message as request (since server only receives requests)
                let message = match proto::HsesRequestMessage::decode(&buf[..n]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Failed to decode file message: {:?}", e);
                        continue;
                    }
                };
                
                eprintln!("Received file packet from {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[command=0x{:04x}, instance={}, attribute={}, service={}], Payload[{} bytes: {:02x?}]", 
                    src,
                    message.header.division,
                    message.header.ack,
                    message.header.request_id,
                    message.header.payload_size,
                    message.sub_header.command,
                    message.sub_header.instance,
                    message.sub_header.attribute,
                    message.sub_header.service,
                    message.payload.len(),
                    message.payload
                );
                
                // Handle the message
                let response = Self::handle_message_internal(&message, &state, &handlers).await;
                
                // Send response
                if let Ok(response_data) = response {
                    if !response_data.is_empty() {
                        // Decode response message for detailed logging
                        if let Ok(response_message) = proto::HsesResponseMessage::decode(&response_data) {
                            eprintln!("Sending file response to {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[service={}, status={}, added_status_size={}, added_status={}], Payload[{} bytes: {:02x?}]", 
                                src,
                                response_message.header.division,
                                response_message.header.ack,
                                response_message.header.request_id,
                                response_message.header.payload_size,
                                response_message.sub_header.service,
                                response_message.sub_header.status,
                                response_message.sub_header.added_status_size,
                                response_message.sub_header.added_status,
                                response_message.payload.len(),
                                response_message.payload
                            );
                        } else {
                            eprintln!("Sending file response: {} bytes (failed to decode for detailed logging)", response_data.len());
                        }
                        if let Err(e) = file_socket.send_to(&response_data, src).await {
                            eprintln!("Error sending file response: {:?}", e);
                        }
                    }
                } else {
                    eprintln!("Error handling file message: {:?}", response.err());
                }
            }
        })
        };
        
        // Wait for either task to complete (they should run forever)
        tokio::select! {
            result = robot_task => {
                if let Err(e) = result {
                    eprintln!("Robot task error: {:?}", e);
                }
            }
            result = file_task => {
                if let Err(e) = result {
                    eprintln!("File task error: {:?}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Internal message handler (static method for use in tasks)
    async fn handle_message_internal(
        message: &proto::HsesRequestMessage, 
        state: &SharedState, 
        handlers: &CommandHandlerRegistry
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut state = state.write().await;
        
        // Handle the command using new message format
        let payload = match handlers.handle(message, &mut state) {
            Ok(payload) => payload,
            Err(proto::ProtocolError::InvalidCommand) => {
                // For unknown commands, return empty payload but still send response
                vec![]
            }
            Err(e) => {
                // For other errors, propagate them
                return Err(Box::new(e));
            }
        };
        
        // Create response message with proper structure using new message types
        let response_message = proto::HsesResponseMessage::new(
            message.header.division,
            0x01, // ACK
            message.header.request_id,
            message.sub_header.service,
            0x00, // status: success
            0x0000, // added_status: no error
            payload,
        );
        
        // Encode the response
        let response_data = response_message.encode();
        Ok(response_data.to_vec())
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
    
    pub fn file_port(mut self, port: u16) -> Self {
        self.config.file_port = Some(port);
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
