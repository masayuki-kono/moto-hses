// Mock server setup utilities for integration tests

use crate::common::test_logging;
use moto_hses_mock::{server::MockServerBuilder, MockConfig, MockServer};
use moto_hses_proto::{FILE_CONTROL_PORT, ROBOT_CONTROL_PORT};
use std::time::Duration;
use tokio::time::sleep;

/// Create a new MockServerManager with default settings
pub fn create_test_server() -> MockServerManager {
    MockServerManager::new()
}

/// Create a new MockServerManager with custom host
pub fn create_test_server_with_host(host: &str) -> MockServerManager {
    MockServerManager::new_with_host(host.to_string())
}

pub struct MockServerManager {
    handle: Option<tokio::task::JoinHandle<()>>,
    host: String,
    robot_port: u16,
    file_port: u16,
}

impl MockServerManager {
    /// Create a new MockServerManager with default host and ports
    pub fn new() -> Self {
        Self::new_with_host_and_ports(
            "127.0.0.1".to_string(),
            ROBOT_CONTROL_PORT,
            FILE_CONTROL_PORT,
        )
    }

    /// Create a new MockServerManager with custom host and default ports
    pub fn new_with_host(host: String) -> Self {
        Self::new_with_host_and_ports(host, ROBOT_CONTROL_PORT, FILE_CONTROL_PORT)
    }

    /// Create a new MockServerManager with custom host and ports
    pub fn new_with_host_and_ports(host: String, robot_port: u16, file_port: u16) -> Self {
        Self {
            handle: None,
            host,
            robot_port,
            file_port,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create mock server with default configuration
        let config = MockConfig::new(&self.host, self.robot_port, self.file_port);
        let server = match MockServer::new(config).await {
            Ok(server) => server,
            Err(e) => {
                test_logging::log_mock_server_startup_failure(
                    &self.host,
                    self.robot_port,
                    &e.to_string(),
                );
                return Err(e);
            }
        };

        // Start server in background task
        let handle = tokio::spawn(async move {
            if let Err(e) = server.run().await {
                eprintln!("Mock server error: {}", e);
            }
        });

        self.handle = Some(handle);

        // Wait for server to be ready
        match self.wait_for_server().await {
            Ok(_) => {
                test_logging::log_mock_server_startup(&self.host, self.robot_port);
                Ok(())
            }
            Err(e) => {
                test_logging::log_mock_server_startup_failure(
                    &self.host,
                    self.robot_port,
                    &e.to_string(),
                );
                Err(e)
            }
        }
    }

    pub async fn start_with_builder<F>(
        &mut self,
        builder_fn: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(MockServerBuilder) -> MockServerBuilder,
    {
        // Create mock server with custom configuration
        let builder = MockServerBuilder::new()
            .host(&self.host)
            .robot_port(self.robot_port)
            .file_port(self.file_port);

        let server = builder_fn(builder).build().await?;

        // Start server in background task
        let handle = tokio::spawn(async move {
            if let Err(e) = server.run().await {
                eprintln!("Mock server error: {}", e);
            }
        });

        self.handle = Some(handle);

        // Wait for server to be ready
        self.wait_for_server().await?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    async fn wait_for_server(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let max_attempts = 30;
        for _ in 0..max_attempts {
            if self.is_server_ready().await {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }
        Err("Mock server failed to start within timeout".into())
    }

    async fn is_server_ready(&self) -> bool {
        // Try to connect to the server
        match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
            Ok(socket) => {
                // Send a simple ping to check if server is responding
                let test_data = b"test";
                match socket
                    .send_to(test_data, format!("{}:{}", self.host, self.robot_port))
                    .await
                {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Get the robot port number
    pub fn robot_port(&self) -> u16 {
        self.robot_port
    }

    /// Get the file port number
    pub fn file_port(&self) -> u16 {
        self.file_port
    }
}

impl Drop for MockServerManager {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            // Give a moment for the server to fully shut down
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

// Helper functions for common test configurations

pub async fn create_variable_test_server(
) -> Result<MockServerManager, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = MockServerManager::new();

    manager
        .start_with_builder(|builder| {
            builder
                .with_variable(0, vec![0x2A, 0x00, 0x00, 0x00]) // I000 = 42
                .with_variable(1, vec![0x39, 0x30, 0x00, 0x00]) // I001 = 12345
                .with_variable(2, vec![0xDB, 0x0F, 0x49, 0x40]) // R002 = 3.14159
                .with_variable(3, vec![0xFF, 0x00, 0x00, 0x00]) // B003 = 255
        })
        .await?;

    Ok(manager)
}

pub async fn create_io_test_server(
) -> Result<MockServerManager, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = MockServerManager::new();

    manager
        .start_with_builder(|builder| {
            builder
                .with_io_state(1, true) // Input 1 = ON
                .with_io_state(2, false) // Input 2 = OFF
                .with_io_state(1001, false) // Output 1 = OFF
                .with_io_state(1002, true) // Output 2 = ON
        })
        .await?;

    Ok(manager)
}

pub async fn create_alarm_test_server(
) -> Result<MockServerManager, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = MockServerManager::new();

    manager
        .start_with_builder(|builder| {
            builder
                // Add test alarms for instances 1-4
                .with_alarm(moto_hses_proto::alarm::test_alarms::servo_error()) // Instance 1
                .with_alarm(moto_hses_proto::alarm::test_alarms::emergency_stop()) // Instance 2
                .with_alarm(moto_hses_proto::alarm::test_alarms::safety_error()) // Instance 3
                .with_alarm(moto_hses_proto::alarm::test_alarms::communication_error()) // Instance 4
                // Add alarm history for testing
                .with_alarm_history(moto_hses_proto::alarm::test_alarms::servo_error()) // Major failure #1
                .with_alarm_history(moto_hses_proto::alarm::test_alarms::emergency_stop()) // Major failure #2
                .with_alarm_history(moto_hses_proto::alarm::test_alarms::safety_error())
            // Major failure #3
        })
        .await?;

    Ok(manager)
}

pub async fn create_register_test_server(
) -> Result<MockServerManager, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = MockServerManager::new();

    manager
        .start_with_builder(|builder| {
            builder
                .with_variable(0, vec![0x00, 0x00, 0x00, 0x00]) // Register 0 = 0
                .with_variable(1, vec![0x64, 0x00, 0x00, 0x00]) // Register 1 = 100
                .with_variable(2, vec![0xC8, 0x00, 0x00, 0x00]) // Register 2 = 200
        })
        .await?;

    Ok(manager)
}

pub async fn create_position_test_server(
) -> Result<MockServerManager, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = MockServerManager::new();

    manager
        .start_with_builder(|builder| {
            // Set up position data with known values for testing
            let test_position =
                moto_hses_proto::Position::Pulse(moto_hses_proto::PulsePosition::new(
                    [100, 200, 300, 400, 500, 600, 700, 800], // Known test values
                    1,                                        // control_group = 1
                ));
            builder.with_position(test_position)
        })
        .await?;

    Ok(manager)
}

pub async fn create_job_info_test_server(
) -> Result<MockServerManager, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = MockServerManager::new();

    manager
        .start_with_builder(|builder| {
            // Set up job information with known values for testing
            builder.with_executing_job(moto_hses_proto::job::ExecutingJobInfo::new(
                "TEST_JOB".to_string(),
                2,
                1,
                100,
            ))
        })
        .await?;

    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_startup() {
        test_logging::init_test_logging();
        test_logging::log_test_start("test_mock_server_startup");

        let mut server = MockServerManager::new();
        let result = server.start().await.is_ok();

        test_logging::log_test_completion("test_mock_server_startup", result);
        assert!(result);
    }

    #[tokio::test]
    async fn test_variable_test_server() {
        let server = create_variable_test_server().await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_io_test_server() {
        let server = create_io_test_server().await;
        assert!(server.is_ok());
    }
}
