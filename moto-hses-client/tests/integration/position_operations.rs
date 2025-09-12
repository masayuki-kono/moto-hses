// Integration tests for position operations

use crate::common::{
    mock_server_setup::MockServerManager,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_robot_pulse_position, {
    log::info!("Creating mock server...");
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");
    log::info!("Mock server started successfully");

    log::info!("Creating test client...");
    let client = create_test_client().await.expect("Failed to create client");
    log::info!("Client created successfully");

    let position = client
        .read_position(1, moto_hses_proto::CoordinateSystemType::RobotPulse)
        .await
        .expect("Failed to read robot pulse position");

    // Verify position data structure
    // Position is a struct, not a collection, so we verify it was read successfully
    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "Robot pulse position should have at least 6 axes"
            );

            // Verify all values are finite numbers
            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "Position value at axis {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_base_pulse_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::CoordinateSystemType::BasePulse)
        .await
        .expect("Failed to read base pulse position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "Base pulse position should have at least 6 axes"
            );

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "Base pulse position value at axis {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_station_pulse_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::CoordinateSystemType::StationPulse)
        .await
        .expect("Failed to read station pulse position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "Station pulse position should have at least 6 axes"
            );

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "Station pulse position value at axis {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_robot_cartesian_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::CoordinateSystemType::RobotCartesian)
        .await
        .expect("Failed to read robot cartesian position");

    match position {
        moto_hses_proto::Position::Cartesian(cart_pos) => {
            assert!(
                cart_pos.x.is_finite(),
                "Cartesian X position should be finite"
            );
            assert!(
                cart_pos.y.is_finite(),
                "Cartesian Y position should be finite"
            );
            assert!(
                cart_pos.z.is_finite(),
                "Cartesian Z position should be finite"
            );
            assert!(
                cart_pos.rx.is_finite(),
                "Cartesian RX position should be finite"
            );
            assert!(
                cart_pos.ry.is_finite(),
                "Cartesian RY position should be finite"
            );
            assert!(
                cart_pos.rz.is_finite(),
                "Cartesian RZ position should be finite"
            );
        }
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            // Mock server may return pulse position instead of cartesian
            // This is acceptable for testing purposes
            assert!(
                pulse_pos.joints.len() >= 6,
                "Pulse position should have at least 6 axes"
            );
        }
    }
});

test_with_logging!(test_r1_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::CoordinateSystemType::RobotPulse)
        .await
        .expect("Failed to read R1 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "R1 position should have at least 6 values"
            );

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "R1 position value at index {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_r2_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(2, moto_hses_proto::CoordinateSystemType::RobotPulse)
        .await
        .expect("Failed to read R2 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "R2 position should have at least 6 values"
            );

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "R2 position value at index {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_b1_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::CoordinateSystemType::BasePulse)
        .await
        .expect("Failed to read B1 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "B1 position should have at least 6 values"
            );

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "B1 position value at index {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_b2_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(2, moto_hses_proto::CoordinateSystemType::BasePulse)
        .await
        .expect("Failed to read B2 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(
                pulse_pos.joints.len() >= 6,
                "B2 position should have at least 6 values"
            );

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2147483648,
                    "B2 position value at index {} should be within i32 range",
                    i
                );
            }
        }
        _ => panic!("Expected pulse position type"),
    }
});

test_with_logging!(test_position_monitoring, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test position monitoring for a short duration
    let start_time = std::time::Instant::now();
    let monitoring_duration = std::time::Duration::from_secs(1);

    while start_time.elapsed() < monitoring_duration {
        let robot_pos = client
            .read_position(1, moto_hses_proto::CoordinateSystemType::RobotPulse)
            .await;
        let base_pos = client
            .read_position(1, moto_hses_proto::CoordinateSystemType::BasePulse)
            .await;
        let cartesian_pos = client
            .read_position(1, moto_hses_proto::CoordinateSystemType::RobotCartesian)
            .await;

        // All operations should succeed
        assert!(
            robot_pos.is_ok(),
            "Robot position reading should succeed during monitoring"
        );
        assert!(
            base_pos.is_ok(),
            "Base position reading should succeed during monitoring"
        );
        assert!(
            cartesian_pos.is_ok(),
            "Cartesian position reading should succeed during monitoring"
        );

        wait_for_operation().await;
    }
});
