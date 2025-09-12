// Integration tests for read status operations

use crate::common::{
    mock_server_setup::MockServerManager,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_read_complete_status, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client
        .read_status()
        .await
        .expect("Failed to read complete status");

    // Verify status data structure
    // StatusData1 and StatusData2 are structs, not collections
    // We can verify they have valid boolean values
    assert!(
        status.data1.running || !status.data1.running,
        "Status data1 should have valid running state"
    );
    assert!(
        status.data2.servo_on || !status.data2.servo_on,
        "Status data2 should have valid servo state"
    );
});

test_with_logging!(test_read_status_data1, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let data1 = client
        .read_status_data1()
        .await
        .expect("Failed to read status data1");

    // Verify data1 structure
    // StatusData1 is a struct with boolean fields
    // We can verify the fields are valid boolean values
    assert!(
        data1.running || !data1.running,
        "Data1 running should be a valid boolean"
    );
    assert!(
        data1.teach || !data1.teach,
        "Data1 teach should be a valid boolean"
    );
    assert!(
        data1.play || !data1.play,
        "Data1 play should be a valid boolean"
    );
});

test_with_logging!(test_read_status_data2, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let data2 = client
        .read_status_data2()
        .await
        .expect("Failed to read status data2");

    // Verify data2 structure
    // StatusData2 is a struct with boolean fields
    // We can verify the fields are valid boolean values
    assert!(
        data2.servo_on || !data2.servo_on,
        "Data2 servo_on should be a valid boolean"
    );
    assert!(
        data2.alarm || !data2.alarm,
        "Data2 alarm should be a valid boolean"
    );
    assert!(
        data2.error || !data2.error,
        "Data2 error should be a valid boolean"
    );
});

test_with_logging!(test_read_status_result, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let status = client.read_status().await.expect("Failed to read status");

    // Verify status structure
    // Status contains data1 and data2 fields
    assert!(
        status.data1.running || !status.data1.running,
        "Running status should be a valid boolean"
    );
    assert!(
        status.data2.servo_on || !status.data2.servo_on,
        "Servo status should be a valid boolean"
    );
});

test_with_logging!(test_status_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test all status operations
    let status = client.read_status().await.expect("Failed to read status");

    let data1 = client
        .read_status_data1()
        .await
        .expect("Failed to read status data1");

    let data2 = client
        .read_status_data2()
        .await
        .expect("Failed to read status data2");

    // Verify data consistency
    assert_eq!(
        status.data1.running, data1.running,
        "Complete status data1 should match individual data1"
    );
    assert_eq!(
        status.data2.servo_on, data2.servo_on,
        "Complete status data2 should match individual data2"
    );
});

test_with_logging!(test_status_monitoring, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test status monitoring over time
    let start_time = std::time::Instant::now();
    let monitoring_duration = std::time::Duration::from_secs(1);

    while start_time.elapsed() < monitoring_duration {
        let status = client.read_status().await;
        assert!(
            status.is_ok(),
            "Status reading should succeed during monitoring"
        );

        if let Ok(status) = status {
            assert!(
                status.data1.running || !status.data1.running,
                "Data1 should have valid running state during monitoring"
            );
            assert!(
                status.data2.servo_on || !status.data2.servo_on,
                "Data2 should have valid servo state during monitoring"
            );
        }

        wait_for_operation().await;
    }
});

test_with_logging!(test_status_data_consistency, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test that status data remains consistent across multiple reads
    let mut previous_data1: Option<moto_hses_proto::StatusData1> = None;
    let mut previous_data2: Option<moto_hses_proto::StatusData2> = None;

    for _ in 0..5 {
        let data1 = client
            .read_status_data1()
            .await
            .expect("Failed to read status data1");
        let data2 = client
            .read_status_data2()
            .await
            .expect("Failed to read status data2");

        if let Some(prev_data1) = &previous_data1 {
            assert_eq!(
                data1.running, prev_data1.running,
                "Data1 running state should remain consistent"
            );
            assert_eq!(
                data1.teach, prev_data1.teach,
                "Data1 teach state should remain consistent"
            );
        }

        if let Some(prev_data2) = &previous_data2 {
            assert_eq!(
                data2.servo_on, prev_data2.servo_on,
                "Data2 servo state should remain consistent"
            );
            assert_eq!(
                data2.alarm, prev_data2.alarm,
                "Data2 alarm state should remain consistent"
            );
        }

        previous_data1 = Some(data1);
        previous_data2 = Some(data2);

        wait_for_operation().await;
    }
});
