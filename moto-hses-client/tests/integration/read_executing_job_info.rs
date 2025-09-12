// Integration tests for read executing job info operations

use crate::common::{
    mock_server_setup::MockServerManager,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_read_complete_job_info, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info_complete(1)
        .await
        .expect("Failed to read complete job info");

    // Verify job info structure
    assert!(
        !job_info.job_name.is_empty(),
        "Job name should not be empty"
    );
    // Note: u32 values are always non-negative, so these assertions are not needed
});

test_with_logging!(test_read_job_name, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 1)
        .await
        .expect("Failed to read job name");

    // Verify job name format
    assert!(
        !job_info.job_name.is_empty(),
        "Job name should not be empty"
    );
});

test_with_logging!(test_read_line_number, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 2)
        .await
        .expect("Failed to read line number");
    let line_number = job_info.line_number;

    // Verify line number
    assert!(line_number > 0, "Line number should be positive");
    assert!(
        line_number <= 9999,
        "Line number should be within reasonable range"
    );
});

test_with_logging!(test_read_step_number, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 3)
        .await
        .expect("Failed to read step number");
    let step_number = job_info.step_number;

    // Verify step number
    // Note: u32 values are always non-negative, so this assertion is not needed
    assert!(
        step_number <= 999,
        "Step number should be within reasonable range"
    );
});

test_with_logging!(test_read_speed_override, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 4)
        .await
        .expect("Failed to read speed override");
    let speed_override = job_info.speed_override_value;

    // Verify speed override
    // Note: u32 values are always non-negative, so this assertion is not needed
    assert!(
        speed_override <= 100,
        "Speed override should not exceed 100%"
    );
});

test_with_logging!(test_read_task_types, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test different task types (1-6)
    for task_type in 1..=6 {
        let job_info = client
            .read_executing_job_info(task_type, 1)
            .await
            .expect(&format!("Failed to read task type {}", task_type));

        // Verify job info is valid
        assert!(
            !job_info.job_name.is_empty(),
            "Job name should not be empty for task type {}",
            task_type
        );
    }
});

test_with_logging!(test_job_info_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test all job info operations
    let complete_job_info = client
        .read_executing_job_info_complete(1)
        .await
        .expect("Failed to read complete job info");

    let job_name_info = client
        .read_executing_job_info(1, 1)
        .await
        .expect("Failed to read job name");

    let line_number_info = client
        .read_executing_job_info(1, 2)
        .await
        .expect("Failed to read line number");

    let step_number_info = client
        .read_executing_job_info(1, 3)
        .await
        .expect("Failed to read step number");

    let speed_override_info = client
        .read_executing_job_info(1, 4)
        .await
        .expect("Failed to read speed override");

    // Verify data consistency
    assert_eq!(
        complete_job_info.job_name, job_name_info.job_name,
        "Complete job info name should match individual job name"
    );
    assert_eq!(
        complete_job_info.line_number, line_number_info.line_number,
        "Complete job info line number should match individual line number"
    );
    assert_eq!(
        complete_job_info.step_number, step_number_info.step_number,
        "Complete job info step number should match individual step number"
    );
    assert_eq!(
        complete_job_info.speed_override_value, speed_override_info.speed_override_value,
        "Complete job info speed override should match individual speed override"
    );
});

test_with_logging!(test_job_info_monitoring, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test job info monitoring over time
    let start_time = std::time::Instant::now();
    let monitoring_duration = std::time::Duration::from_secs(1);

    while start_time.elapsed() < monitoring_duration {
        let job_info = client.read_executing_job_info_complete(1).await;
        assert!(
            job_info.is_ok(),
            "Job info reading should succeed during monitoring"
        );

        if let Ok(job_info) = job_info {
            assert!(
                !job_info.job_name.is_empty(),
                "Job name should not be empty during monitoring"
            );
            // Note: u32 values are always non-negative, so this assertion is not needed
        }

        wait_for_operation().await;
    }
});

test_with_logging!(test_job_info_data_consistency, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test that job info data remains consistent across multiple reads
    let mut previous_job_name = None;
    let mut previous_line_number = None;

    for _ in 0..5 {
        let job_info = client
            .read_executing_job_info_complete(1)
            .await
            .expect("Failed to read job info");
        let job_name = job_info.job_name;
        let line_number = job_info.line_number;

        if let Some(prev_name) = &previous_job_name {
            assert_eq!(job_name, *prev_name, "Job name should remain consistent");
        }

        if let Some(prev_line) = &previous_line_number {
            assert_eq!(
                line_number, *prev_line,
                "Line number should remain consistent"
            );
        }

        previous_job_name = Some(job_name);
        previous_line_number = Some(line_number);

        wait_for_operation().await;
    }
});
