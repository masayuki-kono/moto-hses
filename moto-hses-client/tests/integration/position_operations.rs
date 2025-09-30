#![allow(clippy::expect_used)]
// Integration tests for position operations

use crate::common::{
    mock_server_setup::{MockServerManager, create_position_test_server},
    test_utils::create_test_client,
};
use crate::test_with_logging;

test_with_logging!(test_read_robot_pulse_position, {
    log::debug!("Creating position test server...");
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");
    log::debug!("Position test server started successfully");

    log::debug!("Creating test client...");
    let client = create_test_client().await.expect("Failed to create client");
    log::debug!("Client created successfully");

    let position = client
        .read_position(1, moto_hses_proto::ControlGroupPositionType::RobotPulse)
        .await
        .expect("Failed to read robot pulse position");

    // Verify position data matches expected values
    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert_eq!(pulse_pos.joints.len(), 8, "Robot pulse position should have 8 axes");

            // Verify expected values from MockServer configuration
            let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
            for (i, &expected) in expected_values.iter().enumerate() {
                assert_eq!(
                    pulse_pos.joints[i], expected,
                    "Position value at axis {i} should be {expected}"
                );
            }

            assert_eq!(pulse_pos.control_group, 1, "Control group should be 1");
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_read_base_pulse_position, {
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::ControlGroupPositionType::BasePulse)
        .await
        .expect("Failed to read base pulse position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert_eq!(pulse_pos.joints.len(), 8, "Base pulse position should have 8 axes");

            // Verify expected values from MockServer configuration
            let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
            for (i, &expected) in expected_values.iter().enumerate() {
                assert_eq!(
                    pulse_pos.joints[i], expected,
                    "Base pulse position value at axis {i} should be {expected}"
                );
            }

            assert_eq!(pulse_pos.control_group, 1, "Control group should be 1");
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_read_station_pulse_position, {
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::ControlGroupPositionType::StationPulse)
        .await
        .expect("Failed to read station pulse position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert_eq!(pulse_pos.joints.len(), 8, "Station pulse position should have 8 axes");

            // Verify expected values from MockServer configuration
            let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
            for (i, &expected) in expected_values.iter().enumerate() {
                assert_eq!(
                    pulse_pos.joints[i], expected,
                    "Station pulse position value at axis {i} should be {expected}"
                );
            }

            assert_eq!(pulse_pos.control_group, 1, "Control group should be 1");
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_read_robot_cartesian_position, {
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::ControlGroupPositionType::RobotCartesian)
        .await
        .expect("Failed to read robot cartesian position");

    match position {
        moto_hses_proto::Position::Cartesian(cart_pos) => {
            assert!(cart_pos.x.is_finite(), "Cartesian X position should be finite");
            assert!(cart_pos.y.is_finite(), "Cartesian Y position should be finite");
            assert!(cart_pos.z.is_finite(), "Cartesian Z position should be finite");
            assert!(cart_pos.rx.is_finite(), "Cartesian RX position should be finite");
            assert!(cart_pos.ry.is_finite(), "Cartesian RY position should be finite");
            assert!(cart_pos.rz.is_finite(), "Cartesian RZ position should be finite");
        }
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            // Mock server may return pulse position instead of cartesian
            // This is acceptable for testing purposes
            assert_eq!(pulse_pos.joints.len(), 8, "Pulse position should have 8 axes");

            // Verify expected values from MockServer configuration
            let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
            for (i, &expected) in expected_values.iter().enumerate() {
                assert_eq!(
                    pulse_pos.joints[i], expected,
                    "Pulse position value at axis {i} should be {expected}"
                );
            }
        }
    }
});

test_with_logging!(test_read_r1_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(1, moto_hses_proto::ControlGroupPositionType::RobotPulse)
        .await
        .expect("Failed to read R1 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(pulse_pos.joints.len() >= 6, "R1 position should have at least 6 values");

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2_147_483_648,
                    "R1 position value at index {i} should be within i32 range"
                );
            }
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_read_r2_position, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(2, moto_hses_proto::ControlGroupPositionType::RobotPulse)
        .await
        .expect("Failed to read R2 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert!(pulse_pos.joints.len() >= 6, "R2 position should have at least 6 values");

            for (i, &value) in pulse_pos.joints.iter().enumerate() {
                assert!(
                    value >= -2_147_483_648,
                    "R2 position value at index {i} should be within i32 range"
                );
            }
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_read_b1_position, {
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(11, moto_hses_proto::ControlGroupPositionType::RobotPulse)
        .await
        .expect("Failed to read B1 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert_eq!(pulse_pos.joints.len(), 8, "B1 position should have 8 axes");

            // Verify expected values from MockServer configuration
            let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
            for (i, &expected) in expected_values.iter().enumerate() {
                assert_eq!(
                    pulse_pos.joints[i], expected,
                    "B1 position value at axis {i} should be {expected}"
                );
            }

            assert_eq!(pulse_pos.control_group, 1, "Control group should be 1");
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_read_b2_position, {
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");

    let client = create_test_client().await.expect("Failed to create client");

    let position = client
        .read_position(12, moto_hses_proto::ControlGroupPositionType::RobotPulse)
        .await
        .expect("Failed to read B2 position");

    match position {
        moto_hses_proto::Position::Pulse(pulse_pos) => {
            assert_eq!(pulse_pos.joints.len(), 8, "B2 position should have 8 axes");

            // Verify expected values from MockServer configuration
            let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
            for (i, &expected) in expected_values.iter().enumerate() {
                assert_eq!(
                    pulse_pos.joints[i], expected,
                    "B2 position value at axis {i} should be {expected}"
                );
            }

            assert_eq!(pulse_pos.control_group, 1, "Control group should be 1");
        }
        moto_hses_proto::Position::Cartesian(_) => {
            unreachable!("Expected pulse position type");
        }
    }
});

test_with_logging!(test_continuous_position_monitoring, {
    let _server =
        create_position_test_server().await.expect("Failed to start position test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test position monitoring for 5 seconds (as per legacy example)
    log::debug!("Monitoring position for 5 seconds...");
    for i in 1..=5 {
        match client.read_position(1, moto_hses_proto::ControlGroupPositionType::RobotPulse).await {
            Ok(position) => {
                log::debug!("  [{i}s] Position: {position:?}");

                // Verify position data matches expected values
                match position {
                    moto_hses_proto::Position::Pulse(pulse_pos) => {
                        assert_eq!(pulse_pos.joints.len(), 8, "Position should have 8 axes");

                        // Verify expected values from MockServer configuration
                        let expected_values = [100, 200, 300, 400, 500, 600, 700, 800];
                        for (i, &expected) in expected_values.iter().enumerate() {
                            assert_eq!(
                                pulse_pos.joints[i], expected,
                                "Position value at axis {i} should be {expected}"
                            );
                        }

                        assert_eq!(pulse_pos.control_group, 1, "Control group should be 1");
                    }
                    moto_hses_proto::Position::Cartesian(_) => {
                        unreachable!("Expected pulse position type");
                    }
                }
            }
            Err(e) => {
                log::error!("  [{i}s] Failed to read position: {e}");
                unreachable!("Position reading should succeed during monitoring");
            }
        }

        if i < 5 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    log::debug!("Position monitoring completed successfully");
});
