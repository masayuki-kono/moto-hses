#![allow(clippy::expect_used)]
// Integration tests for job control operations

use crate::common::test_utils::{create_test_client, wait_for_operation};
use crate::test_with_logging;
use moto_hses_mock::server::MockServerBuilder;
use moto_hses_proto::{FILE_CONTROL_PORT, ROBOT_CONTROL_PORT};
use std::sync::Arc;

test_with_logging!(test_job_start_command, {
    // Create mock server
    let server = Arc::new(
        MockServerBuilder::new()
            .host("127.0.0.1")
            .robot_port(ROBOT_CONTROL_PORT)
            .file_port(FILE_CONTROL_PORT)
            .build()
            .await
            .expect("Failed to build mock server"),
    );

    // Start server in background
    let server_clone = Arc::clone(&server);
    let server_handle = tokio::spawn(async move {
        server_clone.run().await.expect("Failed to run mock server");
    });

    // Wait for server to be ready
    wait_for_operation().await;

    let client = create_test_client().await.expect("Failed to create client");

    // Verify initial state (should be not running)
    let initial_running = server.get_running().await;
    assert!(!initial_running);
    log::info!("✓ Initial running state: {initial_running}");

    // Test job start command
    log::info!("Testing job start command...");
    client.start_job().await.expect("Failed to start job");
    wait_for_operation().await;

    // Verify state change
    let current_running = server.get_running().await;
    assert!(current_running);
    log::info!("✓ Job start verified: running={current_running}");

    log::info!("✓ Job start command completed successfully");

    // Clean up
    server_handle.abort();
});
