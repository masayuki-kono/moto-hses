// Integration tests for read status operations

use crate::common::{
    mock_server_setup::{
        create_status_all_false_server, create_status_all_true_server, create_status_test_server,
    },
    test_utils::create_test_client,
};
use crate::test_with_logging;

test_with_logging!(test_read_complete_status, {
    let _server = create_status_test_server().await.expect("Failed to start status test server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client.read_status().await.expect("Failed to read complete status");

    // Verify status matches expected values from MockServer configuration
    // Data1 expectations
    assert!(!status.data1.step);
    assert!(!status.data1.one_cycle);
    assert!(status.data1.continuous);
    assert!(status.data1.running);
    assert!(!status.data1.speed_limited);
    assert!(!status.data1.teach);
    assert!(status.data1.play);
    assert!(!status.data1.remote);

    // Data2 expectations
    assert!(!status.data2.teach_pendant_hold);
    assert!(!status.data2.external_hold);
    assert!(!status.data2.command_hold);
    assert!(status.data2.alarm);
    assert!(!status.data2.error);
    assert!(status.data2.servo_on);

    // Test convenience methods
    assert!(status.is_running());
    assert!(status.is_servo_on());
    assert!(status.has_alarm());
    assert!(!status.has_error());
    assert!(!status.is_teach_mode());
    assert!(status.is_play_mode());
    assert!(!status.is_remote_mode());
});

test_with_logging!(test_read_complete_status_all_true, {
    let _server =
        create_status_all_true_server().await.expect("Failed to start status all true test server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client.read_status().await.expect("Failed to read complete status");

    // Verify all status values are true
    // Data1 expectations
    assert!(status.data1.step);
    assert!(status.data1.one_cycle);
    assert!(status.data1.continuous);
    assert!(status.data1.running);
    assert!(status.data1.speed_limited);
    assert!(status.data1.teach);
    assert!(status.data1.play);
    assert!(status.data1.remote);

    // Data2 expectations
    assert!(status.data2.teach_pendant_hold);
    assert!(status.data2.external_hold);
    assert!(status.data2.command_hold);
    assert!(status.data2.alarm);
    assert!(status.data2.error);
    assert!(status.data2.servo_on);

    // Test convenience methods
    assert!(status.is_running());
    assert!(status.is_servo_on());
    assert!(status.has_alarm());
    assert!(status.has_error());
    assert!(status.is_teach_mode());
    assert!(status.is_play_mode());
    assert!(status.is_remote_mode());
});

test_with_logging!(test_read_complete_status_all_false, {
    let _server = create_status_all_false_server()
        .await
        .expect("Failed to start status all false test server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client.read_status().await.expect("Failed to read complete status");

    // Verify all status values are false
    // Data1 expectations
    assert!(!status.data1.step);
    assert!(!status.data1.one_cycle);
    assert!(!status.data1.continuous);
    assert!(!status.data1.running);
    assert!(!status.data1.speed_limited);
    assert!(!status.data1.teach);
    assert!(!status.data1.play);
    assert!(!status.data1.remote);

    // Data2 expectations
    assert!(!status.data2.teach_pendant_hold);
    assert!(!status.data2.external_hold);
    assert!(!status.data2.command_hold);
    assert!(!status.data2.alarm);
    assert!(!status.data2.error);
    assert!(!status.data2.servo_on);

    // Test convenience methods
    assert!(!status.is_running());
    assert!(!status.is_servo_on());
    assert!(!status.has_alarm());
    assert!(!status.has_error());
    assert!(!status.is_teach_mode());
    assert!(!status.is_play_mode());
    assert!(!status.is_remote_mode());
});

test_with_logging!(test_read_status_data1_mixed, {
    let _server = create_status_test_server().await.expect("Failed to start status test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client.read_status_data1().await.expect("Failed to read status data1");

    // Verify data1 matches expected values from MockServer configuration
    assert!(!data1.step);
    assert!(!data1.one_cycle);
    assert!(data1.continuous);
    assert!(data1.running);
    assert!(!data1.speed_limited);
    assert!(!data1.teach);
    assert!(data1.play);
    assert!(!data1.remote);
});

test_with_logging!(test_read_status_data1_all_true, {
    let _server =
        create_status_all_true_server().await.expect("Failed to start status all true test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client.read_status_data1().await.expect("Failed to read status data1");

    // Verify all data1 values are true
    assert!(data1.step);
    assert!(data1.one_cycle);
    assert!(data1.continuous);
    assert!(data1.running);
    assert!(data1.speed_limited);
    assert!(data1.teach);
    assert!(data1.play);
    assert!(data1.remote);
});

test_with_logging!(test_read_status_data1_all_false, {
    let _server = create_status_all_false_server()
        .await
        .expect("Failed to start status all false test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client.read_status_data1().await.expect("Failed to read status data1");

    // Verify all data1 values are false
    assert!(!data1.step);
    assert!(!data1.one_cycle);
    assert!(!data1.continuous);
    assert!(!data1.running);
    assert!(!data1.speed_limited);
    assert!(!data1.teach);
    assert!(!data1.play);
    assert!(!data1.remote);
});

test_with_logging!(test_read_status_data2_mixed, {
    let _server = create_status_test_server().await.expect("Failed to start status test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client.read_status_data2().await.expect("Failed to read status data2");

    // Verify data2 matches expected values from MockServer configuration
    assert!(!data2.teach_pendant_hold);
    assert!(!data2.external_hold);
    assert!(!data2.command_hold);
    assert!(data2.alarm);
    assert!(!data2.error);
    assert!(data2.servo_on);
});

test_with_logging!(test_read_status_data2_all_true, {
    let _server =
        create_status_all_true_server().await.expect("Failed to start status all true test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client.read_status_data2().await.expect("Failed to read status data2");

    // Verify all data2 values are true
    assert!(data2.teach_pendant_hold);
    assert!(data2.external_hold);
    assert!(data2.command_hold);
    assert!(data2.alarm);
    assert!(data2.error);
    assert!(data2.servo_on);
});

test_with_logging!(test_read_status_data2_all_false, {
    let _server = create_status_all_false_server()
        .await
        .expect("Failed to start status all false test server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client.read_status_data2().await.expect("Failed to read status data2");

    // Verify all data2 values are false
    assert!(!data2.teach_pendant_hold);
    assert!(!data2.external_hold);
    assert!(!data2.command_hold);
    assert!(!data2.alarm);
    assert!(!data2.error);
    assert!(!data2.servo_on);
});
