// Integration tests for read executing job info operations

use crate::common::{
    mock_server_setup::create_job_info_test_server, test_utils::create_test_client,
};
use crate::test_with_logging;

test_with_logging!(test_read_complete_job_info, {
    let _server = create_job_info_test_server()
        .await
        .expect("Failed to start job info test server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info_complete(1)
        .await
        .expect("Failed to read complete job info");

    // Verify job info matches expected values from MockServer configuration
    assert_eq!(
        job_info.job_name, "TEST_JOB",
        "Job name should match expected value"
    );
    assert_eq!(
        job_info.line_number, 2,
        "Line number should match expected value"
    );
    assert_eq!(
        job_info.step_number, 1,
        "Step number should match expected value"
    );
    assert_eq!(
        job_info.speed_override_value, 100,
        "Speed override should match expected value"
    );
});

test_with_logging!(test_read_job_name, {
    let _server = create_job_info_test_server()
        .await
        .expect("Failed to start job info test server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 1)
        .await
        .expect("Failed to read job name");

    // Verify job name matches expected value
    assert_eq!(
        job_info.job_name, "TEST_JOB",
        "Job name should match expected value"
    );
});

test_with_logging!(test_read_line_number, {
    let _server = create_job_info_test_server()
        .await
        .expect("Failed to start job info test server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 2)
        .await
        .expect("Failed to read line number");

    // Verify line number matches expected value
    assert_eq!(
        job_info.line_number, 2,
        "Line number should match expected value"
    );
});

test_with_logging!(test_read_step_number, {
    let _server = create_job_info_test_server()
        .await
        .expect("Failed to start job info test server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 3)
        .await
        .expect("Failed to read step number");

    // Verify step number matches expected value
    assert_eq!(
        job_info.step_number, 1,
        "Step number should match expected value"
    );
});

test_with_logging!(test_read_speed_override, {
    let _server = create_job_info_test_server()
        .await
        .expect("Failed to start job info test server");

    let client = create_test_client().await.expect("Failed to create client");

    let job_info = client
        .read_executing_job_info(1, 4)
        .await
        .expect("Failed to read speed override");

    // Verify speed override matches expected value
    assert_eq!(
        job_info.speed_override_value, 100,
        "Speed override should match expected value"
    );
});

test_with_logging!(test_read_task_types, {
    let _server = create_job_info_test_server()
        .await
        .expect("Failed to start job info test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test different task types (1-6)
    for task_type in 1..=6 {
        let job_info = client
            .read_executing_job_info(task_type, 1)
            .await
            .expect(&format!("Failed to read task type {}", task_type));

        // Verify job name matches expected value for all task types
        assert_eq!(
            job_info.job_name, "TEST_JOB",
            "Job name should match expected value for task type {}",
            task_type
        );
    }
});
