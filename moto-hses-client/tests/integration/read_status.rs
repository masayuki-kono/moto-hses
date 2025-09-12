// Integration tests for read status operations

use crate::common::{
    mock_server_setup::{
        create_status_all_false_server, create_status_all_true_server, create_status_test_server,
    },
    test_utils::create_test_client,
};
use crate::test_with_logging;

test_with_logging!(test_read_complete_status, {
    let _server = create_status_test_server()
        .await
        .expect("Failed to start status test server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client
        .read_status()
        .await
        .expect("Failed to read complete status");

    // Verify status matches expected values from MockServer configuration
    // Data1 expectations
    assert_eq!(status.data1.step, false);
    assert_eq!(status.data1.one_cycle, false);
    assert_eq!(status.data1.continuous, true);
    assert_eq!(status.data1.running, true);
    assert_eq!(status.data1.speed_limited, false);
    assert_eq!(status.data1.teach, false);
    assert_eq!(status.data1.play, true);
    assert_eq!(status.data1.remote, false);

    // Data2 expectations
    assert_eq!(status.data2.teach_pendant_hold, false);
    assert_eq!(status.data2.external_hold, false);
    assert_eq!(status.data2.command_hold, false);
    assert_eq!(status.data2.alarm, true);
    assert_eq!(status.data2.error, false);
    assert_eq!(status.data2.servo_on, true);

    // Test convenience methods
    assert_eq!(status.is_running(), true);
    assert_eq!(status.is_servo_on(), true);
    assert_eq!(status.has_alarm(), true);
    assert_eq!(status.has_error(), false);
    assert_eq!(status.is_teach_mode(), false);
    assert_eq!(status.is_play_mode(), true);
    assert_eq!(status.is_remote_mode(), false);
});

test_with_logging!(test_read_complete_status_all_true, {
    let _server = create_status_all_true_server()
        .await
        .expect("Failed to start status all true test server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client
        .read_status()
        .await
        .expect("Failed to read complete status");

    // Verify all status values are true
    // Data1 expectations
    assert_eq!(status.data1.step, true);
    assert_eq!(status.data1.one_cycle, true);
    assert_eq!(status.data1.continuous, true);
    assert_eq!(status.data1.running, true);
    assert_eq!(status.data1.speed_limited, true);
    assert_eq!(status.data1.teach, true);
    assert_eq!(status.data1.play, true);
    assert_eq!(status.data1.remote, true);

    // Data2 expectations
    assert_eq!(status.data2.teach_pendant_hold, true);
    assert_eq!(status.data2.external_hold, true);
    assert_eq!(status.data2.command_hold, true);
    assert_eq!(status.data2.alarm, true);
    assert_eq!(status.data2.error, true);
    assert_eq!(status.data2.servo_on, true);

    // Test convenience methods
    assert_eq!(status.is_running(), true);
    assert_eq!(status.is_servo_on(), true);
    assert_eq!(status.has_alarm(), true);
    assert_eq!(status.has_error(), true);
    assert_eq!(status.is_teach_mode(), true);
    assert_eq!(status.is_play_mode(), true);
    assert_eq!(status.is_remote_mode(), true);
});

test_with_logging!(test_read_complete_status_all_false, {
    let _server = create_status_all_false_server()
        .await
        .expect("Failed to start status all false test server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client
        .read_status()
        .await
        .expect("Failed to read complete status");

    // Verify all status values are false
    // Data1 expectations
    assert_eq!(status.data1.step, false);
    assert_eq!(status.data1.one_cycle, false);
    assert_eq!(status.data1.continuous, false);
    assert_eq!(status.data1.running, false);
    assert_eq!(status.data1.speed_limited, false);
    assert_eq!(status.data1.teach, false);
    assert_eq!(status.data1.play, false);
    assert_eq!(status.data1.remote, false);

    // Data2 expectations
    assert_eq!(status.data2.teach_pendant_hold, false);
    assert_eq!(status.data2.external_hold, false);
    assert_eq!(status.data2.command_hold, false);
    assert_eq!(status.data2.alarm, false);
    assert_eq!(status.data2.error, false);
    assert_eq!(status.data2.servo_on, false);

    // Test convenience methods
    assert_eq!(status.is_running(), false);
    assert_eq!(status.is_servo_on(), false);
    assert_eq!(status.has_alarm(), false);
    assert_eq!(status.has_error(), false);
    assert_eq!(status.is_teach_mode(), false);
    assert_eq!(status.is_play_mode(), false);
    assert_eq!(status.is_remote_mode(), false);
});

test_with_logging!(test_read_status_data1_mixed, {
    let _server = create_status_test_server()
        .await
        .expect("Failed to start status test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client
        .read_status_data1()
        .await
        .expect("Failed to read status data1");

    // Verify data1 matches expected values from MockServer configuration
    assert_eq!(data1.step, false);
    assert_eq!(data1.one_cycle, false);
    assert_eq!(data1.continuous, true);
    assert_eq!(data1.running, true);
    assert_eq!(data1.speed_limited, false);
    assert_eq!(data1.teach, false);
    assert_eq!(data1.play, true);
    assert_eq!(data1.remote, false);
});

test_with_logging!(test_read_status_data1_all_true, {
    let _server = create_status_all_true_server()
        .await
        .expect("Failed to start status all true test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client
        .read_status_data1()
        .await
        .expect("Failed to read status data1");

    // Verify all data1 values are true
    assert_eq!(data1.step, true);
    assert_eq!(data1.one_cycle, true);
    assert_eq!(data1.continuous, true);
    assert_eq!(data1.running, true);
    assert_eq!(data1.speed_limited, true);
    assert_eq!(data1.teach, true);
    assert_eq!(data1.play, true);
    assert_eq!(data1.remote, true);
});

test_with_logging!(test_read_status_data1_all_false, {
    let _server = create_status_all_false_server()
        .await
        .expect("Failed to start status all false test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client
        .read_status_data1()
        .await
        .expect("Failed to read status data1");

    // Verify all data1 values are false
    assert_eq!(data1.step, false);
    assert_eq!(data1.one_cycle, false);
    assert_eq!(data1.continuous, false);
    assert_eq!(data1.running, false);
    assert_eq!(data1.speed_limited, false);
    assert_eq!(data1.teach, false);
    assert_eq!(data1.play, false);
    assert_eq!(data1.remote, false);
});

test_with_logging!(test_read_status_data2_mixed, {
    let _server = create_status_test_server()
        .await
        .expect("Failed to start status test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client
        .read_status_data2()
        .await
        .expect("Failed to read status data2");

    // Verify data2 matches expected values from MockServer configuration
    assert_eq!(data2.teach_pendant_hold, false);
    assert_eq!(data2.external_hold, false);
    assert_eq!(data2.command_hold, false);
    assert_eq!(data2.alarm, true);
    assert_eq!(data2.error, false);
    assert_eq!(data2.servo_on, true);
});

test_with_logging!(test_read_status_data2_all_true, {
    let _server = create_status_all_true_server()
        .await
        .expect("Failed to start status all true test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client
        .read_status_data2()
        .await
        .expect("Failed to read status data2");

    // Verify all data2 values are true
    assert_eq!(data2.teach_pendant_hold, true);
    assert_eq!(data2.external_hold, true);
    assert_eq!(data2.command_hold, true);
    assert_eq!(data2.alarm, true);
    assert_eq!(data2.error, true);
    assert_eq!(data2.servo_on, true);
});

test_with_logging!(test_read_status_data2_all_false, {
    let _server = create_status_all_false_server()
        .await
        .expect("Failed to start status all false test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client
        .read_status_data2()
        .await
        .expect("Failed to read status data2");

    // Verify all data2 values are false
    assert_eq!(data2.teach_pendant_hold, false);
    assert_eq!(data2.external_hold, false);
    assert_eq!(data2.command_hold, false);
    assert_eq!(data2.alarm, false);
    assert_eq!(data2.error, false);
    assert_eq!(data2.servo_on, false);
});
