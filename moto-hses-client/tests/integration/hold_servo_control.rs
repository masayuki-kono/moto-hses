// Integration tests for hold/servo control operations

use crate::common::{
    mock_server_setup::MockServerManager,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_hold_control_commands, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Read initial status
    log::info!("Reading initial status...");
    let initial_status = client
        .read_status()
        .await
        .expect("Failed to read initial status");

    log::info!("✓ Initial status retrieved");
    log::info!("  Servo on: {}", initial_status.is_servo_on());
    log::info!("  Running: {}", initial_status.is_running());

    // Test HOLD control
    let initial_hold_state = !initial_status.data1.running; // If running, HOLD is OFF
    let opposite_hold_state = !initial_hold_state;

    log::info!("Testing HOLD control...");
    log::info!(
        "  Initial HOLD state: {} (running: {})",
        initial_hold_state,
        initial_status.data1.running
    );
    log::info!(
        "  Setting HOLD to opposite state: {}...",
        opposite_hold_state
    );

    client
        .set_hold(opposite_hold_state)
        .await
        .expect("Failed to set HOLD");

    log::info!(
        "  ✓ HOLD {} command sent",
        if opposite_hold_state { "ON" } else { "OFF" }
    );

    // Wait and verify the change
    wait_for_operation().await;
    let hold_test_status = client
        .read_status()
        .await
        .expect("Failed to read HOLD test status");

    let expected_running = !opposite_hold_state; // If HOLD is ON, running should be false
    log::info!(
        "  ✓ HOLD state verification: Running = {} (expected: {})",
        hold_test_status.is_running(),
        expected_running
    );

    // Verify the state change
    assert_eq!(
        hold_test_status.is_running(),
        expected_running,
        "HOLD state should affect running state"
    );

    // Set HOLD back to initial state
    log::info!(
        "  Setting HOLD back to initial state: {}...",
        initial_hold_state
    );
    client
        .set_hold(initial_hold_state)
        .await
        .expect("Failed to set HOLD back");

    log::info!(
        "  ✓ HOLD {} command sent",
        if initial_hold_state { "ON" } else { "OFF" }
    );
});

test_with_logging!(test_servo_control_commands, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Read initial status
    log::info!("Reading initial status...");
    let initial_status = client
        .read_status()
        .await
        .expect("Failed to read initial status");

    log::info!("✓ Initial status retrieved");
    log::info!("  Servo on: {}", initial_status.is_servo_on());

    // Test Servo control
    let initial_servo_state = initial_status.data2.servo_on;
    let opposite_servo_state = !initial_servo_state;

    log::info!("Testing Servo control...");
    log::info!("  Initial Servo state: {}", initial_servo_state);
    log::info!(
        "  Setting Servo to opposite state: {}...",
        opposite_servo_state
    );

    client
        .set_servo(opposite_servo_state)
        .await
        .expect("Failed to set Servo");

    log::info!(
        "  ✓ Servo {} command sent",
        if opposite_servo_state { "ON" } else { "OFF" }
    );

    // Wait and verify the change
    wait_for_operation().await;
    let servo_test_status = client
        .read_status()
        .await
        .expect("Failed to read Servo test status");

    log::info!(
        "  ✓ Servo state verification: Servo ON = {} (expected: {})",
        servo_test_status.is_servo_on(),
        opposite_servo_state
    );

    // Verify the state change
    assert_eq!(
        servo_test_status.is_servo_on(),
        opposite_servo_state,
        "Servo state should match the set value"
    );

    // Set Servo back to initial state
    log::info!(
        "  Setting Servo back to initial state: {}...",
        initial_servo_state
    );
    client
        .set_servo(initial_servo_state)
        .await
        .expect("Failed to set Servo back");

    log::info!(
        "  ✓ Servo {} command sent",
        if initial_servo_state { "ON" } else { "OFF" }
    );
});

test_with_logging!(test_hlock_control_commands, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test HLOCK control
    let initial_hlock_state = false;
    let opposite_hlock_state = !initial_hlock_state;

    log::info!("Testing HLOCK control...");
    log::info!("  Initial HLOCK state: {}", initial_hlock_state);
    log::info!(
        "  Setting HLOCK to opposite state: {}...",
        opposite_hlock_state
    );

    client
        .set_hlock(opposite_hlock_state)
        .await
        .expect("Failed to set HLOCK");

    log::info!(
        "  ✓ HLOCK {} command sent",
        if opposite_hlock_state { "ON" } else { "OFF" }
    );

    // Wait and verify the change
    wait_for_operation().await;
    let _hlock_test_status = client
        .read_status()
        .await
        .expect("Failed to read HLOCK test status");

    log::info!(
        "  ✓ HLOCK state verification: HLOCK set to {} (command executed)",
        opposite_hlock_state
    );

    // Set HLOCK back to initial state
    log::info!(
        "  Setting HLOCK back to initial state: {}...",
        initial_hlock_state
    );
    client
        .set_hlock(initial_hlock_state)
        .await
        .expect("Failed to set HLOCK back");

    log::info!(
        "  ✓ HLOCK {} command sent",
        if initial_hlock_state { "ON" } else { "OFF" }
    );
});
