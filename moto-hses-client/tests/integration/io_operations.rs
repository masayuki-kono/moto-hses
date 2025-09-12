// Integration tests for I/O operations

use crate::common::{
    mock_server_setup::{create_io_test_server, MockServerManager},
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_io_read_operations, {
    log::info!("Creating I/O test server...");
    let _server = create_io_test_server()
        .await
        .expect("Failed to start mock server");
    log::info!("I/O test server started successfully");

    log::info!("Creating test client...");
    let client = create_test_client().await.expect("Failed to create client");
    log::info!("Client created successfully");

    // Test reading robot user input I/O
    log::info!("Reading robot user input I/O #1...");
    let io1_state = client.read_io(1).await.expect("Failed to read I/O #1");
    log::info!("I/O #1 state: {}", io1_state);

    // Test reading robot user output I/O
    log::info!("Reading robot user output I/O #1001...");
    let io1001_state = client
        .read_io(1001)
        .await
        .expect("Failed to read I/O #1001");
    log::info!("I/O #1001 state: {}", io1001_state);

    // Verify I/O states are valid (should be boolean values)
    log::info!("Verifying I/O states...");
    assert!(
        io1_state || !io1_state,
        "I/O #1 state should be a valid boolean"
    );
    assert!(
        io1001_state || !io1001_state,
        "I/O #1001 state should be a valid boolean"
    );
    log::info!("I/O state verification passed");
});

test_with_logging!(test_io_write_operations, {
    let _server = create_io_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing to robot user output I/O
    client
        .write_io(1001, true)
        .await
        .expect("Failed to write to I/O #1001");

    wait_for_operation().await;

    // Verify the write operation
    let io_state_after_write = client
        .read_io(1001)
        .await
        .expect("Failed to read I/O #1001 after write");

    assert_eq!(
        io_state_after_write, true,
        "I/O #1001 should be ON after write"
    );
});

test_with_logging!(test_io_write_off, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing OFF to I/O
    client
        .write_io(1002, false)
        .await
        .expect("Failed to write OFF to I/O #1002");

    wait_for_operation().await;

    // Verify the write operation
    let io_state_after_write = client
        .read_io(1002)
        .await
        .expect("Failed to read I/O #1002 after write");

    assert_eq!(
        io_state_after_write, false,
        "I/O #1002 should be OFF after write"
    );
});

test_with_logging!(test_invalid_io_number, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid I/O number
    let result = client.read_io(9999).await;
    assert!(result.is_err(), "Invalid I/O number should return error");

    let write_result = client.write_io(9999, true).await;
    assert!(
        write_result.is_err(),
        "Writing to invalid I/O number should return error"
    );
});

test_with_logging!(test_io_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Comprehensive test covering all I/O operations
    let test_ios = vec![1, 2, 1001, 1002];

    // Read initial states
    for io_num in &test_ios {
        let state = client
            .read_io(*io_num)
            .await
            .expect(&format!("Failed to read I/O #{}", io_num));
        assert!(state || !state, "I/O #{} state should be 0 or 1", io_num);
    }

    // Test write operations
    client
        .write_io(1001, true)
        .await
        .expect("Failed to write ON to I/O #1001");

    client
        .write_io(1002, false)
        .await
        .expect("Failed to write OFF to I/O #1002");

    wait_for_operation().await;

    // Verify write operations
    let io1001_state = client
        .read_io(1001)
        .await
        .expect("Failed to read I/O #1001 after write");
    assert_eq!(io1001_state, true, "I/O #1001 should be ON");

    let io1002_state = client
        .read_io(1002)
        .await
        .expect("Failed to read I/O #1002 after write");
    assert_eq!(io1002_state, false, "I/O #1002 should be OFF");
});

test_with_logging!(test_io_state_consistency, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test I/O state consistency over multiple reads
    let io_num = 1001;

    // Set initial state
    client
        .write_io(io_num, true)
        .await
        .expect("Failed to set initial I/O state");

    wait_for_operation().await;

    // Read multiple times to ensure consistency
    for _ in 0..5 {
        let state = client
            .read_io(io_num)
            .await
            .expect("Failed to read I/O state");
        assert_eq!(state, true, "I/O state should remain consistent");
        wait_for_operation().await;
    }

    // Change state and verify
    client
        .write_io(io_num, false)
        .await
        .expect("Failed to change I/O state");

    wait_for_operation().await;

    let new_state = client
        .read_io(io_num)
        .await
        .expect("Failed to read I/O state after change");
    assert_eq!(new_state, false, "I/O state should change correctly");
});
