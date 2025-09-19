#![allow(clippy::expect_used)]
// Test utilities for integration tests

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::ROBOT_CONTROL_PORT;
use std::time::Duration;

/// Create a test client with default host and port
///
/// # Errors
///
/// Returns an error if the client fails to connect
pub async fn create_test_client() -> Result<HsesClient, Box<dyn std::error::Error>> {
    create_test_client_with_host_and_port("127.0.0.1", ROBOT_CONTROL_PORT).await
}

/// Create a test client with custom host and default port
///
/// # Errors
///
/// Returns an error if the client fails to connect
pub async fn create_test_client_with_host(
    host: &str,
) -> Result<HsesClient, Box<dyn std::error::Error>> {
    create_test_client_with_host_and_port(host, ROBOT_CONTROL_PORT).await
}

/// Create a test client with custom host and port
///
/// # Errors
///
/// Returns an error if the client fails to connect
pub async fn create_test_client_with_host_and_port(
    host: &str,
    port: u16,
) -> Result<HsesClient, Box<dyn std::error::Error>> {
    let config = ClientConfig {
        host: host.to_string(),
        port,
        timeout: Duration::from_millis(500),
        retry_count: 3,
        retry_delay: Duration::from_millis(100),
        buffer_size: 8192,
        text_encoding: moto_hses_proto::TextEncoding::Utf8,
    };

    let client = HsesClient::new_with_config(config).await?;
    Ok(client)
}

pub async fn wait_for_operation() {
    // Small delay to ensure operations complete
    tokio::time::sleep(Duration::from_millis(50)).await;
}
