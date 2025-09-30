//! Mock HSES server implementation

use crate::handlers::CommandHandlerRegistry;
use crate::state::{MockState, SharedState};
use moto_hses_proto as proto;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// Mock HSES server
pub struct MockServer {
    robot_socket: Arc<UdpSocket>,
    file_socket: Arc<UdpSocket>,
    state: SharedState,
    handlers: CommandHandlerRegistry,
}

impl MockServer {
    /// Create a new mock server
    ///
    /// # Errors
    ///
    /// Returns an error if socket binding fails
    pub async fn new(
        config: crate::MockConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let robot_addr = config.robot_addr()?;
        let file_addr = config.file_addr()?;

        let robot_socket = Arc::new(UdpSocket::bind(robot_addr).await?);
        let file_socket = Arc::new(UdpSocket::bind(file_addr).await?);

        let mut mock_state = MockState {
            text_encoding: config.text_encoding,
            status: config.default_status.clone(),
            position: config.default_position.clone(),
            registers: config.registers.clone(),
            variables: config.variables.clone(),
            cycle_mode: config.cycle_mode,
            ..Default::default()
        };

        // Apply configured job information
        if let Some(job) = &config.executing_job {
            mock_state.executing_job = Some(job.clone());
        }

        // Apply configured alarms if any
        if !config.alarms.is_empty() {
            mock_state.alarms.clone_from(&config.alarms);
        }

        // Apply configured alarm history if any
        if !config.alarm_history.is_empty() {
            // Add alarms to appropriate history categories
            for alarm in &config.alarm_history {
                // Determine category based on alarm code or use a default category
                // For simplicity, we'll add all to major_failure category
                mock_state
                    .alarm_history
                    .add_alarm(proto::alarm::AlarmCategory::MajorFailure, alarm.clone());
            }
        }

        let state = SharedState::new(mock_state);
        let handlers = CommandHandlerRegistry::default();

        info!("Mock server listening on {robot_addr}");
        info!("Mock server listening on {file_addr}");

        Ok(Self { robot_socket, file_socket, state, handlers })
    }

    /// Get the local address of the server
    /// # Errors
    ///
    /// Returns an error if local address cannot be obtained
    pub fn local_addr(&self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.robot_socket.local_addr()?)
    }

    /// Run the server
    /// # Errors
    ///
    /// Returns an error if server operation fails
    #[allow(clippy::too_many_lines)]
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
                            error!("Error receiving from robot socket: {e:?}");
                            continue;
                        }
                    };

                    if n < 32 {
                        debug!("Received message too short: {n} bytes");
                        continue;
                    }

                    // Parse HSES message as request (since server only receives requests)
                    let message = match proto::HsesRequestMessage::decode(&buf[..n]) {
                        Ok(msg) => msg,
                        Err(e) => {
                            error!("Failed to decode message: {e:?}");
                            continue;
                        }
                    };

                    debug!(
                        "Received packet from {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[command=0x{:04x}, instance={}, attribute={}, service={}], Payload[{} bytes: {:02x?}]",
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
                            if let Ok(response_message) =
                                proto::HsesResponseMessage::decode(&response_data)
                            {
                                debug!(
                                    "Sending response to {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[service={}, status={}, added_status_size={}, added_status={}], Payload[{} bytes: {:02x?}]",
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
                                debug!(
                                    "Sending response: {} bytes (failed to decode for detailed logging)",
                                    response_data.len()
                                );
                            }
                            if let Err(e) = robot_socket.send_to(&response_data, src).await {
                                debug!("Error sending response: {e:?}");
                            }
                        }
                    } else {
                        debug!("Error handling message: {:?}", response.err());
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
                            debug!("Error receiving from file socket: {e:?}");
                            continue;
                        }
                    };

                    if n < 32 {
                        debug!("Received file message too short: {n} bytes");
                        continue;
                    }

                    // Parse HSES message as request (since server only receives requests)
                    let message = match proto::HsesRequestMessage::decode(&buf[..n]) {
                        Ok(msg) => msg,
                        Err(e) => {
                            debug!("Failed to decode file message: {e:?}");
                            continue;
                        }
                    };

                    debug!(
                        "Received file packet from {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[command=0x{:04x}, instance={}, attribute={}, service={}], Payload[{} bytes: {:02x?}]",
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
                            if let Ok(response_message) =
                                proto::HsesResponseMessage::decode(&response_data)
                            {
                                debug!(
                                    "Sending file response to {}: Header[division={}, ack={}, request_id={}, payload_size={}], SubHeader[service={}, status={}, added_status_size={}, added_status={}], Payload[{} bytes: {:02x?}]",
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
                                debug!(
                                    "Sending file response: {} bytes (failed to decode for detailed logging)",
                                    response_data.len()
                                );
                            }
                            if let Err(e) = file_socket.send_to(&response_data, src).await {
                                debug!("Error sending file response: {e:?}");
                            }
                        }
                    } else {
                        debug!("Error handling file message: {:?}", response.err());
                    }
                }
            })
        };

        // Wait for either task to complete (they should run forever)
        tokio::select! {
            result = robot_task => {
                let _ = result; // Tasks should run forever, ignore result
            }
            result = file_task => {
                let _ = result; // Tasks should run forever, ignore result
            }
        }

        Ok(())
    }

    /// Internal message handler (static method for use in tasks)
    async fn handle_message_internal(
        message: &proto::HsesRequestMessage,
        state: &SharedState,
        handlers: &CommandHandlerRegistry,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut state = state.write().await;

        // Handle the command using new message format
        let (payload, status, added_status) = match handlers.handle(message, &mut state) {
            Ok(payload) => (payload, 0x00, 0x0000), // Success
            Err(proto::ProtocolError::InvalidCommand) => {
                // For unknown commands, return error status
                (vec![], 0x01, 0x0001) // Error status with command error code
            }
            Err(proto::ProtocolError::InvalidService) => {
                // For invalid service, return error status
                (vec![], 0x02, 0x0002) // Error status with service error code
            }
            Err(proto::ProtocolError::InvalidAttribute) => {
                // For invalid attribute, return error status
                (vec![], 0x03, 0x0003) // Error status with attribute error code
            }
            Err(_e) => {
                // For other errors, return generic error status
                (vec![], 0xFF, 0x00FF) // Generic error status
            }
        };

        // Create response message with proper structure using new message types
        let response_message = proto::HsesResponseMessage::new(
            message.header.division,
            0x01, // ACK
            message.header.request_id,
            message.sub_header.service,
            status,       // status: success (0x00) or error (non-zero)
            added_status, // added_status: error code if status is non-zero
            payload,
        )
        .map_err(|e| {
            error!("Failed to create response message: {e}");
            e
        })?;

        // Encode the response
        let response_data = response_message.encode();
        Ok(response_data.to_vec())
    }

    /// Get a reference to the shared state
    #[must_use]
    pub const fn state(&self) -> &SharedState {
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

    /// Get the current cycle mode
    pub async fn get_cycle_mode(&self) -> proto::CycleMode {
        let state = self.state.read().await;
        state.get_cycle_mode()
    }
}

/// Server builder for easy configuration
pub struct MockServerBuilder {
    config: crate::MockConfig,
}

impl MockServerBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { config: crate::MockConfig::default() }
    }

    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    #[must_use]
    pub const fn robot_port(mut self, port: u16) -> Self {
        self.config.robot_port = port;
        self
    }

    #[must_use]
    pub const fn file_port(mut self, port: u16) -> Self {
        self.config.file_port = port;
        self
    }

    #[must_use]
    pub const fn text_encoding(mut self, encoding: proto::TextEncoding) -> Self {
        self.config.text_encoding = encoding;
        self
    }

    #[must_use]
    pub fn with_alarm(mut self, alarm: proto::Alarm) -> Self {
        self.config.alarms.push(alarm);
        self
    }

    #[must_use]
    pub fn with_alarm_history(mut self, alarm: proto::Alarm) -> Self {
        self.config.alarm_history.push(alarm);
        self
    }

    #[must_use]
    pub fn with_io_state(mut self, io_number: u16, state: bool) -> Self {
        self.config.io_states.insert(io_number, state);
        self
    }

    #[must_use]
    pub const fn with_position(mut self, position: proto::Position) -> Self {
        self.config.default_position = position;
        self
    }

    #[must_use]
    pub const fn with_status(mut self, status: proto::Status) -> Self {
        self.config.default_status = status;
        self
    }

    #[must_use]
    pub fn with_executing_job(mut self, job: proto::ExecutingJobInfo) -> Self {
        self.config.executing_job = Some(job);
        self
    }

    #[must_use]
    pub fn with_registers(mut self, registers: std::collections::HashMap<u16, i16>) -> Self {
        self.config.registers = registers;
        self
    }

    #[must_use]
    pub fn with_variables(mut self, variables: std::collections::HashMap<u8, Vec<u8>>) -> Self {
        self.config.variables = variables;
        self
    }

    #[must_use]
    pub const fn with_cycle_mode(mut self, mode: proto::CycleMode) -> Self {
        self.config.cycle_mode = mode;
        self
    }

    /// # Errors
    ///
    /// Returns an error if server creation fails
    pub async fn build(self) -> Result<MockServer, Box<dyn std::error::Error + Send + Sync>> {
        MockServer::new(self.config).await
    }
}

impl Default for MockServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
