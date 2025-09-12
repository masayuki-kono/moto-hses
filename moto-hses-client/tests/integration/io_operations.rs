// Integration tests for I/O operations

use crate::common::{
    mock_server_setup::{create_io_test_server, MockServerManager},
    test_utils::create_test_client,
};
use crate::test_with_logging;

test_with_logging!(test_read_io, {
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
    assert_eq!(io1_state, true, "I/O #1 should be ON (initial state)");

    // Test reading robot user output I/O
    log::info!("Reading robot user output I/O #1001...");
    let io1001_state = client
        .read_io(1001)
        .await
        .expect("Failed to read I/O #1001");
    log::info!("I/O #1001 state: {}", io1001_state);
    assert_eq!(
        io1001_state, false,
        "I/O #1001 should be OFF (initial state)"
    );

    // Test reading additional I/O as per legacy example
    log::info!("Reading robot user input I/O #2...");
    let io2_state = client.read_io(2).await.expect("Failed to read I/O #2");
    log::info!("I/O #2 state: {}", io2_state);
    assert_eq!(io2_state, false, "I/O #2 should be OFF (initial state)");

    log::info!("I/O state verification passed");
});

test_with_logging!(test_write_io, {
    let _server = create_io_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing to robot user output I/O (as per legacy example)
    log::info!("Writing to robot user output I/O #1001...");
    client
        .write_io(1001, true)
        .await
        .expect("Failed to write to I/O #1001");
    log::info!("Successfully set I/O #1001 to ON");

    // Wait a moment and verify the change (as per legacy example)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    log::info!("Verifying I/O #1001 state...");
    let io_state_after_write = client
        .read_io(1001)
        .await
        .expect("Failed to read I/O #1001 after write");

    log::info!("I/O #1001 state after write: {}", io_state_after_write);
    assert_eq!(
        io_state_after_write, true,
        "I/O #1001 should be ON after write"
    );

    // Additional I/O operations (as per legacy example)
    log::info!("Writing to robot user output I/O #1002...");
    client
        .write_io(1002, false)
        .await
        .expect("Failed to write OFF to I/O #1002");
    log::info!("Successfully set I/O #1002 to OFF");

    // Wait a moment and verify the change
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    log::info!("Verifying I/O #1002 state...");
    let io1002_state_after_write = client
        .read_io(1002)
        .await
        .expect("Failed to read I/O #1002 after write");

    log::info!("I/O #1002 state after write: {}", io1002_state_after_write);
    assert_eq!(
        io1002_state_after_write, false,
        "I/O #1002 should be OFF after write"
    );
});

test_with_logging!(test_read_and_write_io_with_invalid_number, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid I/O number (as per legacy example)
    log::info!("Testing invalid I/O number read...");
    match client.read_io(65535).await {
        Ok(value) => {
            log::error!("✗ Invalid I/O number succeeded unexpectedly: {}", value);
            panic!("Invalid I/O number should return error");
        }
        Err(e) => {
            log::info!("✓ Invalid I/O number correctly failed: {}", e);
        }
    }

    // Test invalid I/O number for write
    log::info!("Testing invalid I/O number write...");
    match client.write_io(65535, true).await {
        Ok(()) => {
            log::error!("✗ Invalid I/O number write succeeded unexpectedly");
            panic!("Writing to invalid I/O number should return error");
        }
        Err(e) => {
            log::info!("✓ Invalid I/O number write correctly failed: {}", e);
        }
    }
});
