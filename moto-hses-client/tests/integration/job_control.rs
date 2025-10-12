#![allow(clippy::expect_used)]
// Integration tests for job control operations

use crate::common::test_utils::{create_test_client, wait_for_operation};
use crate::test_with_logging;
use moto_hses_mock::server::MockServerBuilder;
use moto_hses_proto::{FILE_CONTROL_PORT, ROBOT_CONTROL_PORT, commands::JobSelectType};
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

test_with_logging!(test_job_select_command, {
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

    // Verify initial state (should be no selected job)
    let initial_selected_job = server.get_selected_job().await;
    assert!(initial_selected_job.is_none());
    log::info!("✓ Initial selected job state: None");

    // Test job select command for execution
    log::info!("Testing job select command for execution...");
    client
        .select_job(JobSelectType::InExecution, "TEST.JOB", 0)
        .await
        .expect("Failed to select job");
    wait_for_operation().await;

    // Verify state change
    let selected_job = server.get_selected_job().await;
    assert!(selected_job.is_some());
    let selected_job = selected_job.expect("Selected job should exist");
    assert_eq!(selected_job.job_name, "TEST.JOB");
    assert_eq!(selected_job.line_number, 0);
    assert_eq!(selected_job.select_type, 1);
    log::info!(
        "✓ Job select for execution verified: job_name={}, line_number={}, select_type={}",
        selected_job.job_name,
        selected_job.line_number,
        selected_job.select_type
    );

    // Test job select command for master task
    log::info!("Testing job select command for master task...");
    client
        .select_job(JobSelectType::MasterTask0, "MASTER.JOB", 123)
        .await
        .expect("Failed to select master job");
    wait_for_operation().await;

    // Verify state change
    let selected_job = server.get_selected_job().await;
    assert!(selected_job.is_some());
    let selected_job = selected_job.expect("Selected job should exist");
    assert_eq!(selected_job.job_name, "MASTER.JOB");
    assert_eq!(selected_job.line_number, 123);
    assert_eq!(selected_job.select_type, 10);
    log::info!(
        "✓ Job select for master task verified: job_name={}, line_number={}, select_type={}",
        selected_job.job_name,
        selected_job.line_number,
        selected_job.select_type
    );

    log::info!("✓ Job select command completed successfully");

    // Clean up
    server_handle.abort();
});

test_with_logging!(test_job_select_command_validation, {
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

    // Test job name length validation
    log::info!("Testing job name length validation...");
    let long_job_name = "A".repeat(33); // 33 characters, exceeds 32-byte limit
    let result = client.select_job(JobSelectType::InExecution, long_job_name, 0).await;
    assert!(result.is_err());
    log::info!("✓ Job name length validation passed");

    // Test line number validation
    log::info!("Testing line number validation...");
    let result = client.select_job(JobSelectType::InExecution, "TEST.JOB", 10000).await; // Exceeds 9999
    assert!(result.is_err());
    log::info!("✓ Line number validation passed");

    // Test valid job select with maximum line number
    log::info!("Testing valid job select with maximum line number...");
    client
        .select_job(JobSelectType::InExecution, "TEST.JOB", 9999)
        .await
        .expect("Failed to select job");
    wait_for_operation().await;

    let selected_job = server.get_selected_job().await;
    assert!(selected_job.is_some());
    let selected_job = selected_job.expect("Selected job should exist");
    assert_eq!(selected_job.line_number, 9999);
    log::info!("✓ Valid job select with maximum line number verified");

    log::info!("✓ Job select command validation completed successfully");

    // Clean up
    server_handle.abort();
});

test_with_logging!(test_job_select_command_all_types, {
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

    // Test all job select types
    let test_cases = vec![
        (JobSelectType::InExecution, 1, "EXEC.JOB"),
        (JobSelectType::MasterTask0, 10, "MASTER0.JOB"),
        (JobSelectType::MasterTask1, 11, "MASTER1.JOB"),
        (JobSelectType::MasterTask2, 12, "MASTER2.JOB"),
        (JobSelectType::MasterTask3, 13, "MASTER3.JOB"),
        (JobSelectType::MasterTask4, 14, "MASTER4.JOB"),
        (JobSelectType::MasterTask5, 15, "MASTER5.JOB"),
    ];

    for (select_type, expected_instance, job_name) in test_cases {
        log::info!("Testing job select type: {select_type:?}");
        client.select_job(select_type, job_name, 0).await.expect("Failed to select job");
        wait_for_operation().await;

        let selected_job = server.get_selected_job().await;
        assert!(selected_job.is_some());
        let selected_job = selected_job.expect("Selected job should exist");
        assert_eq!(selected_job.job_name, job_name);
        assert_eq!(selected_job.select_type, expected_instance);
        log::info!(
            "✓ Job select type {:?} verified: job_name={}, select_type={}",
            select_type,
            selected_job.job_name,
            selected_job.select_type
        );
    }

    log::info!("✓ All job select types completed successfully");

    // Clean up
    server_handle.abort();
});
