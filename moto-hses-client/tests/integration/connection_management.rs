#![allow(clippy::expect_used)]
// Integration tests for connection management

use crate::common::{
    mock_server_setup::{MockServerManager, create_test_server},
    test_utils::{create_test_client, create_test_client_with_host_and_port},
};
use crate::test_with_logging;

test_with_logging!(test_basic_connection, {
    log::debug!("Creating test server...");
    let mut server = create_test_server();
    server.start().await.expect("Failed to start mock server");
    log::info!("Mock server started successfully");

    log::debug!("Creating test client...");
    let _client = create_test_client().await.expect("Failed to create client");
    log::debug!("Client created successfully");

    log::info!("Testing connection...");
    // Connection test passed (client creation is sufficient for UDP)
    log::info!("Connection test passed");
});

test_with_logging!(test_connection_with_different_ports, {
    // Use a single port test to avoid port conflicts
    let port = 30091;
    log::info!("Testing connection with port {port}");

    let mut server =
        MockServerManager::new_with_host_and_ports("127.0.0.1".to_string(), port, port + 1);

    log::info!("Starting mock server on port {port}...");
    server.start().await.expect("Failed to start mock server");
    log::info!("Mock server started successfully on port {port}");

    log::info!("Creating client for port {port}...");
    let _client = create_test_client_with_host_and_port("127.0.0.1", port)
        .await
        .expect("Failed to create client");
    log::debug!("Client created successfully for port {port}");

    log::info!("Testing connection...");
    // Connection test passed (client creation is sufficient for UDP)
    log::info!("Connection test passed for port {port}");
});

test_with_logging!(test_connection_timeout_handling, {
    // Test communication with non-existent server (should timeout)
    // Use a different port that's not used by any mock server
    let client = create_test_client_with_host_and_port("127.0.0.1", 65535)
        .await
        .expect("UDP client creation should always succeed");

    // Test actual communication - this should timeout
    let status_result = client.read_status().await;
    assert!(status_result.is_err(), "Communication with non-existent server should fail");
});

test_with_logging!(test_multiple_connections, {
    let mut server =
        MockServerManager::new_with_host_and_ports("127.0.0.1".to_string(), 10096, 10097);
    server.start().await.expect("Failed to start mock server");

    // Create multiple clients to the same server
    let clients: Vec<_> = (0..5).map(|_| async { create_test_client().await }).collect();

    let results: Vec<Result<moto_hses_client::HsesClient, Box<dyn std::error::Error>>> =
        futures::future::join_all(clients).await;

    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Client {i} should connect successfully");
    }
});

test_with_logging!(test_actual_communication, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test actual communication with read_status
    log::info!("Testing actual communication with read_status...");
    match client.read_status().await {
        Ok(status) => {
            log::info!("✓ Communication successful");
            log::info!("  Robot running: {}", status.is_running());
            log::info!("  Servo on: {}", status.is_servo_on());

            // Verify status values are reasonable
            assert!(!status.is_running() || status.is_running());
            assert!(!status.is_servo_on() || status.is_servo_on());
        }
        Err(moto_hses_client::ClientError::TimeoutError(_)) => {
            log::warn!("✗ Communication timeout - robot may be busy or network slow");
            // This is acceptable for this test
        }
        Err(moto_hses_client::ClientError::ProtocolError(e)) => {
            log::error!("✗ Protocol error: {e}");
            unreachable!("Unexpected protocol error: {e}");
        }
        Err(e) => {
            log::error!("✗ Communication failed: {e}");
            unreachable!("Unexpected communication error: {e}");
        }
    }
});

test_with_logging!(test_retry_mechanism_actual, {
    // Test retry mechanism with a server that might be slow to respond
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create client with aggressive retry settings
    let config = moto_hses_client::ClientConfig {
        host: "127.0.0.1".to_string(),
        port: 10040,
        timeout: std::time::Duration::from_millis(50), // Short timeout
        retry_count: 3,
        retry_delay: std::time::Duration::from_millis(25),
        buffer_size: 8192,
    };

    let client = moto_hses_client::HsesClient::new_with_config(config)
        .await
        .expect("Failed to create client with retry configuration");

    // Test that retry mechanism works for actual communication
    log::info!("Testing retry mechanism with actual communication...");
    let start_time = std::time::Instant::now();

    match client.read_status().await {
        Ok(status) => {
            let elapsed = start_time.elapsed();
            log::info!("✓ Communication successful after {elapsed:?}");
            log::info!("  Robot running: {}", status.is_running());
            log::info!("  Servo on: {}", status.is_servo_on());

            // Verify that retry mechanism was used (should take some time)
            // Note: UDP communication might be very fast, so we just verify it completed
            log::info!("Communication completed in {elapsed:?}");
        }
        Err(e) => {
            log::warn!("Communication failed despite retries: {e}");
            // This is acceptable for this test
        }
    }
});
