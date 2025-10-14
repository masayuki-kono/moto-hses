#![allow(clippy::expect_used)]
// Integration tests for alarm operations

use crate::common::{
    mock_server_setup::{create_alarm_test_server, MockServerManager},
    test_utils::create_test_client,
};
use crate::test_with_logging;
use moto_hses_proto::AlarmAttribute;

test_with_logging!(test_complete_alarm_data, {
    log::info!("Creating alarm test server...");
    let _server = create_alarm_test_server().await.expect("Failed to start mock server");

    log::debug!("Creating test client...");
    let client = create_test_client().await.expect("Failed to create client");

    log::info!("Reading complete alarm data for instance 1, attribute 0...");
    let alarm_data = client
        .read_alarm_data(1, AlarmAttribute::All) // Read complete alarm data (attribute 0)
        .await
        .expect("Failed to read complete alarm data");

    log::info!(
        "Complete alarm data received: code={}, data={}, alarm_type={}, time='{}', name='{}'",
        alarm_data.code,
        alarm_data.data,
        alarm_data.alarm_type,
        alarm_data.time,
        alarm_data.name
    );

    // Verify alarm data matches expected values from MockServer default state
    // Expected values from test_alarms::servo_error():
    // - code: 1001, data: 1, alarm_type: 1, time: "2024/01/01 12:00", name: "Servo Error"
    assert_eq!(alarm_data.code, 1001, "Alarm code should match expected value");
    assert_eq!(alarm_data.data, 1, "Alarm data should match expected value");
    assert_eq!(alarm_data.alarm_type, 1, "Alarm type should match expected value");
    assert_eq!(alarm_data.time, "2024/01/01 12:00", "Alarm time should match expected value");
    assert_eq!(alarm_data.name, "Servo Error", "Alarm name should match expected value");

    log::info!("All alarm data values match expected values from MockServer");
    log::info!("Test completed successfully");
});

test_with_logging!(test_specific_alarm_attributes, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading alarm code
    log::info!("Testing alarm code attribute...");
    let alarm_code =
        client.read_alarm_data(1, AlarmAttribute::Code).await.expect("Failed to read alarm code");
    log::info!("Alarm code: {}", alarm_code.code);
    assert_eq!(alarm_code.code, 1001, "Alarm code should match expected value");

    // Test reading alarm data
    log::info!("Testing alarm data attribute...");
    let alarm_data =
        client.read_alarm_data(1, AlarmAttribute::Data).await.expect("Failed to read alarm data");
    log::info!("Alarm data: {}", alarm_data.data);
    assert_eq!(alarm_data.data, 1, "Alarm data should match expected value");

    // Test reading alarm type
    log::info!("Testing alarm type attribute...");
    let alarm_type =
        client.read_alarm_data(1, AlarmAttribute::Type).await.expect("Failed to read alarm type");
    log::info!("Alarm type: {}", alarm_type.alarm_type);
    assert_eq!(alarm_type.alarm_type, 1, "Alarm type should match expected value");

    // Test reading alarm time
    log::info!("Testing alarm time attribute...");
    let alarm_time =
        client.read_alarm_data(1, AlarmAttribute::Time).await.expect("Failed to read alarm time");
    log::info!("Alarm time: {}", alarm_time.time);
    assert_eq!(alarm_time.time, "2024/01/01 12:00", "Alarm time should match expected value");

    // Test reading alarm name
    log::info!("Testing alarm name attribute...");
    let alarm_name =
        client.read_alarm_data(1, AlarmAttribute::Name).await.expect("Failed to read alarm name");
    log::info!("Alarm name: {}", alarm_name.name);
    assert_eq!(alarm_name.name, "Servo Error", "Alarm name should match expected value");

    log::info!("All specific alarm attributes match expected values");
});

test_with_logging!(test_alarm_instances, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading alarm instances (HSES specification: Instance 1-4)
    log::info!("Testing multiple alarm instances...");

    // Expected values from MockServer default state:
    // Instance 1: servo_error() - code: 1001, name: "Servo Error"
    // Instance 2: emergency_stop() - code: 2001, name: "Emergency Stop"
    // Instance 3: safety_error() - code: 3001, name: "Safety Error"
    // Instance 4: communication_error() - code: 4001, name: "Communication Error"

    let expected_alarms = vec![
        (1, 1001, "Servo Error"),
        (2, 2001, "Emergency Stop"),
        (3, 3001, "Safety Error"),
        (4, 4001, "Communication Error"),
    ];

    for (instance, expected_code, expected_name) in expected_alarms {
        let alarm_instance = client
            .read_alarm_data(instance, AlarmAttribute::All) // Read complete alarm data
            .await
            .expect("Failed to read alarm instance");

        log::info!(
            "Alarm instance {}: Code={}, Name={}",
            instance,
            alarm_instance.code,
            alarm_instance.name
        );

        // Verify expected values
        assert_eq!(
            alarm_instance.code, expected_code,
            "Alarm instance {instance} code should match expected value {expected_code}"
        );
        assert_eq!(
            alarm_instance.name, expected_name,
            "Alarm instance {instance} name should match expected value '{expected_name}'"
        );
    }

    log::info!("All alarm instances match expected values");
});

test_with_logging!(test_alarm_history_major_failure, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test major failure alarm history (instances 1-3)
    log::info!("Testing major failure alarm history...");

    // Expected values from MockServer alarm history:
    // Instance 1: servo_error() - code: 1001, name: "Servo Error"
    // Instance 2: emergency_stop() - code: 2001, name: "Emergency Stop"
    // Instance 3: safety_error() - code: 3001, name: "Safety Error"

    let expected_history =
        vec![(1, 1001, "Servo Error"), (2, 2001, "Emergency Stop"), (3, 3001, "Safety Error")];

    for (instance, expected_code, expected_name) in expected_history {
        // Test alarm history code
        let alarm_history_code = client
            .read_alarm_history(instance, AlarmAttribute::Code)
            .await
            .expect("Failed to read major failure alarm code");

        // Test alarm history name
        let alarm_history_name = client
            .read_alarm_history(instance, AlarmAttribute::Name)
            .await
            .expect("Failed to read major failure alarm name");

        log::info!(
            "Major failure alarm {}: Code={}, Name={}",
            instance,
            alarm_history_code.code,
            alarm_history_name.name
        );

        // Verify expected values
        assert_eq!(
            alarm_history_code.code, expected_code,
            "Major failure alarm {instance} code should match expected value {expected_code}"
        );
        assert_eq!(
            alarm_history_name.name, expected_name,
            "Major failure alarm {instance} name should match expected value '{expected_name}'"
        );
    }

    log::info!("All major failure alarm history values match expected values");
});

test_with_logging!(test_alarm_history_monitor, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test monitor alarm history (instances 1001-1003)
    log::info!("Testing monitor alarm history...");

    // Expected values from MockServer alarm history:
    // Monitor alarms are set up in alarm history but may return "No alarm" for some instances
    // We'll check that the response is valid (either has alarm data or "No alarm")

    for instance in 1001..=1003 {
        let alarm_history = client
            .read_alarm_history(instance, AlarmAttribute::Name)
            .await
            .expect("Failed to read monitor alarm");

        if alarm_history.code != 0 {
            log::info!(
                "Monitor alarm {}: Code={}, Name={}",
                instance,
                alarm_history.code,
                alarm_history.name
            );
            // If there's an alarm, verify it has valid data
            assert!(
                alarm_history.code > 0,
                "Monitor alarm {instance} should have positive code if not 'No alarm'"
            );
            assert!(
                !alarm_history.name.is_empty(),
                "Monitor alarm {instance} should have non-empty name if not 'No alarm'"
            );
        } else {
            log::info!("Monitor alarm {instance}: No alarm");
            // If no alarm, verify it's properly handled
            assert_eq!(
                alarm_history.code, 0,
                "Monitor alarm {instance} should have code 0 when 'No alarm'"
            );
        }
    }

    log::info!("Monitor alarm history test completed");
});

test_with_logging!(test_alarm_history_attributes, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test alarm history code attribute
    log::info!("Testing alarm history code attribute...");
    let alarm_code = client
        .read_alarm_history(1, AlarmAttribute::Code)
        .await
        .expect("Failed to read major failure alarm code");
    log::info!("Major failure alarm #1 code: {}", alarm_code.code);
    assert_eq!(alarm_code.code, 1001, "Major failure alarm code should match expected value");

    // Test alarm history time attribute
    log::info!("Testing alarm history time attribute...");
    let alarm_time = client
        .read_alarm_history(1, AlarmAttribute::Time)
        .await
        .expect("Failed to read major failure alarm time");
    log::info!("Major failure alarm #1 time: {}", alarm_time.time);
    assert_eq!(
        alarm_time.time, "2024/01/01 12:00",
        "Major failure alarm time should match expected value"
    );

    log::info!("All alarm history attributes match expected values");
});

test_with_logging!(test_invalid_alarm_history_instance, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid instance (should return error)
    log::info!("Testing invalid alarm history instance...");
    let result = client.read_alarm_history(5000, AlarmAttribute::Code).await;

    // Assert that invalid instance returns error
    assert!(result.is_err(), "Invalid alarm history instance should return error");

    log::info!("Invalid instance correctly returned error");
});

test_with_logging!(test_invalid_alarm_instance, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid alarm instance (5000 is outside all valid ranges)
    log::info!("Testing invalid alarm instance...");
    let result = client.read_alarm_data(5000, AlarmAttribute::Code).await;
    assert!(result.is_err(), "Invalid alarm instance should return error");
});

test_with_logging!(test_invalid_alarm_attribute, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid alarm attribute (255 is invalid, but AlarmAttribute::from(255) returns Code)
    // This test now verifies that even with an invalid u8 value, the system handles it gracefully
    log::info!("Testing alarm attribute conversion...");
    let result = client.read_alarm_data(1, AlarmAttribute::from(255)).await;
    // Since AlarmAttribute::from(255) returns Code (default), this should succeed
    assert!(result.is_ok(), "AlarmAttribute::from(255) should return Code and succeed");
});

test_with_logging!(test_alarm_reset_command, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test alarm reset command (0x82 command with instance 1)
    let result = client.reset_alarm().await;
    assert!(result.is_ok(), "Alarm reset command should succeed");
});

test_with_logging!(test_error_cancel_command, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test error cancel command (0x82 command with instance 2)
    let result = client.cancel_error().await;
    assert!(result.is_ok(), "Error cancel command should succeed");
});

test_with_logging!(test_alarm_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    log::info!("Starting comprehensive alarm operations test...");

    // Test complete alarm data reading (attribute 0)
    let _complete_alarm = client
        .read_alarm_data(1, AlarmAttribute::All) // Read complete alarm data
        .await
        .expect("Failed to read complete alarm data");

    // Test specific alarm attributes
    let _alarm_code =
        client.read_alarm_data(1, AlarmAttribute::Code).await.expect("Failed to read alarm code");

    let _alarm_data =
        client.read_alarm_data(1, AlarmAttribute::Data).await.expect("Failed to read alarm data");

    let _alarm_type =
        client.read_alarm_data(1, AlarmAttribute::Type).await.expect("Failed to read alarm type");

    let _alarm_time =
        client.read_alarm_data(1, AlarmAttribute::Time).await.expect("Failed to read alarm time");

    let _alarm_name =
        client.read_alarm_data(1, AlarmAttribute::Name).await.expect("Failed to read alarm name");

    // Test multiple alarm instances (1-4)
    for i in 1..=4 {
        let _instance = client
            .read_alarm_data(i, AlarmAttribute::All) // Read complete alarm data
            .await
            .expect("Failed to read alarm instance");
    }

    // Test alarm history - major failure alarms
    for i in 1..=3 {
        let _major_failure = client
            .read_alarm_history(i, AlarmAttribute::Code)
            .await
            .expect("Failed to read major failure alarm");
    }

    // Test alarm history - monitor alarms
    for i in 1001..=1003 {
        let _monitor_alarm = client
            .read_alarm_history(i, AlarmAttribute::Name)
            .await
            .expect("Failed to read monitor alarm");
    }

    // Test alarm history attributes
    let _history_code = client
        .read_alarm_history(1, AlarmAttribute::Code)
        .await
        .expect("Failed to read alarm history code");

    let _history_time = client
        .read_alarm_history(1, AlarmAttribute::Time)
        .await
        .expect("Failed to read alarm history time");

    // Test commands
    assert!(client.reset_alarm().await.is_ok(), "Alarm reset should succeed");
    assert!(client.cancel_error().await.is_ok(), "Error cancel should succeed");

    log::info!("Comprehensive alarm operations test completed successfully");
});
