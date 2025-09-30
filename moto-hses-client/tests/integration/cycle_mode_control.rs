#![allow(clippy::expect_used)]
// Integration tests for cycle mode control operations

use crate::common::test_utils::{create_test_client, wait_for_operation};
use crate::test_with_logging;
use moto_hses_mock::server::MockServerBuilder;
use moto_hses_proto::{CycleMode, FILE_CONTROL_PORT, ROBOT_CONTROL_PORT};
use std::sync::Arc;

test_with_logging!(test_cycle_mode_control_commands, {
    // Create mock server with initial cycle mode
    let server = Arc::new(
        MockServerBuilder::new()
            .host("127.0.0.1")
            .robot_port(ROBOT_CONTROL_PORT)
            .file_port(FILE_CONTROL_PORT)
            .with_cycle_mode(CycleMode::Continuous)
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

    // Verify initial state
    let initial_mode = server.get_cycle_mode().await;
    assert_eq!(initial_mode, CycleMode::Continuous);
    log::info!("✓ Initial cycle mode: {initial_mode:?}");

    // Test STEP mode
    log::info!("Testing STEP mode...");
    client.set_cycle_mode(CycleMode::Step).await.expect("Failed to set STEP mode");
    wait_for_operation().await;

    let current_mode = server.get_cycle_mode().await;
    assert_eq!(current_mode, CycleMode::Step);
    log::info!("✓ STEP mode verified: {current_mode:?}");

    // Test ONE CYCLE mode
    log::info!("Testing ONE CYCLE mode...");
    client.set_cycle_mode(CycleMode::OneCycle).await.expect("Failed to set ONE CYCLE mode");
    wait_for_operation().await;

    let current_mode = server.get_cycle_mode().await;
    assert_eq!(current_mode, CycleMode::OneCycle);
    log::info!("✓ ONE CYCLE mode verified: {current_mode:?}");

    // Test CONTINUOUS mode
    log::info!("Testing CONTINUOUS mode...");
    client.set_cycle_mode(CycleMode::Continuous).await.expect("Failed to set CONTINUOUS mode");
    wait_for_operation().await;

    let current_mode = server.get_cycle_mode().await;
    assert_eq!(current_mode, CycleMode::Continuous);
    log::info!("✓ CONTINUOUS mode verified: {current_mode:?}");

    log::info!("✓ All cycle mode commands completed successfully");

    // Clean up
    server_handle.abort();
});

test_with_logging!(test_cycle_mode_sequence, {
    // Create mock server with initial cycle mode
    let server = Arc::new(
        MockServerBuilder::new()
            .host("127.0.0.1")
            .robot_port(ROBOT_CONTROL_PORT)
            .file_port(FILE_CONTROL_PORT)
            .with_cycle_mode(CycleMode::Step)
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

    // Test a sequence of mode changes
    log::info!("Testing cycle mode sequence...");

    // Verify initial state
    let initial_mode = server.get_cycle_mode().await;
    assert_eq!(initial_mode, CycleMode::Step);
    log::info!("✓ Initial mode: {initial_mode:?}");

    // Switch to CONTINUOUS
    client.set_cycle_mode(CycleMode::Continuous).await.expect("Failed to set CONTINUOUS mode");
    wait_for_operation().await;
    assert_eq!(server.get_cycle_mode().await, CycleMode::Continuous);
    log::info!("✓ Switched to CONTINUOUS mode");

    // Switch to STEP
    client.set_cycle_mode(CycleMode::Step).await.expect("Failed to set STEP mode");
    wait_for_operation().await;
    assert_eq!(server.get_cycle_mode().await, CycleMode::Step);
    log::info!("✓ Switched to STEP mode");

    // Switch to ONE CYCLE
    client.set_cycle_mode(CycleMode::OneCycle).await.expect("Failed to set ONE CYCLE mode");
    wait_for_operation().await;
    assert_eq!(server.get_cycle_mode().await, CycleMode::OneCycle);
    log::info!("✓ Switched to ONE CYCLE mode");

    // Back to CONTINUOUS
    client.set_cycle_mode(CycleMode::Continuous).await.expect("Failed to set CONTINUOUS mode");
    wait_for_operation().await;
    assert_eq!(server.get_cycle_mode().await, CycleMode::Continuous);
    log::info!("✓ Switched back to CONTINUOUS mode");

    log::info!("✓ Cycle mode sequence completed successfully");

    // Clean up
    server_handle.abort();
});

test_with_logging!(test_cycle_mode_error_handling, {
    // Create mock server with initial cycle mode
    let server = Arc::new(
        MockServerBuilder::new()
            .host("127.0.0.1")
            .robot_port(ROBOT_CONTROL_PORT)
            .file_port(FILE_CONTROL_PORT)
            .with_cycle_mode(CycleMode::Continuous)
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

    // Test normal operation should succeed
    log::info!("Testing error handling...");
    client.set_cycle_mode(CycleMode::Step).await.expect("Failed to set STEP mode");
    wait_for_operation().await;
    assert_eq!(server.get_cycle_mode().await, CycleMode::Step);
    log::info!("✓ Normal operation succeeded");

    // Test rapid mode changes
    log::info!("Testing rapid mode changes...");
    for i in 0..5 {
        client.set_cycle_mode(CycleMode::Step).await.expect("Failed to set STEP mode");
        wait_for_operation().await;
        assert_eq!(server.get_cycle_mode().await, CycleMode::Step);

        client.set_cycle_mode(CycleMode::OneCycle).await.expect("Failed to set ONE CYCLE mode");
        wait_for_operation().await;
        assert_eq!(server.get_cycle_mode().await, CycleMode::OneCycle);

        client.set_cycle_mode(CycleMode::Continuous).await.expect("Failed to set CONTINUOUS mode");
        wait_for_operation().await;
        assert_eq!(server.get_cycle_mode().await, CycleMode::Continuous);

        log::info!("✓ Rapid change iteration {} completed", i + 1);
    }
    log::info!("✓ Rapid mode changes completed");

    log::info!("✓ Error handling test completed");

    // Clean up
    server_handle.abort();
});
