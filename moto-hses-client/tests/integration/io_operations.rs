#![allow(clippy::expect_used)]
// Integration tests for I/O operations

use crate::common::{
    mock_server_setup::{create_io_test_server, MockServerManager},
    test_utils::create_test_client,
};
use crate::test_with_logging;

test_with_logging!(test_read_io, {
    log::debug!("Creating I/O test server...");
    let _server = create_io_test_server().await.expect("Failed to start mock server");
    log::debug!("I/O test server started successfully");

    log::debug!("Creating test client...");
    let client = create_test_client().await.expect("Failed to create client");
    log::debug!("Client created successfully");

    // Test reading robot user input I/O
    log::info!("Reading robot user input I/O #1...");
    let io1_state = client.read_io(1).await.expect("Failed to read I/O #1");
    log::info!("I/O #1 state: 0b{io1_state:08b}");
    assert_eq!(io1_state, 0b0000_0001, "I/O #1 should be ON (initial state)");

    // Test reading robot user output I/O
    log::info!("Reading robot user output I/O #1001...");
    let io1001_state = client.read_io(1001).await.expect("Failed to read I/O #1001");
    log::info!("I/O #1001 state: 0b{io1001_state:08b}");
    assert_eq!(io1001_state, 0b0000_0000, "I/O #1001 should be OFF (initial state)");

    // Test reading additional I/O as per legacy example
    log::info!("Reading robot user input I/O #2...");
    let io2_state = client.read_io(2).await.expect("Failed to read I/O #2");
    log::info!("I/O #2 state: 0b{io2_state:08b}");
    assert_eq!(io2_state, 0b0000_0000, "I/O #2 should be OFF (initial state)");

    log::info!("I/O state verification passed");
});

test_with_logging!(test_write_io, {
    let _server = create_io_test_server().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing to robot user output I/O (as per legacy example)
    log::info!("Writing to robot user output I/O #1001...");
    client.write_io(1001, 0b0000_0001).await.expect("Failed to write to I/O #1001");
    log::info!("Successfully set I/O #1001 to ON");

    // Wait a moment and verify the change (as per legacy example)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    log::info!("Verifying I/O #1001 state...");
    let io_state_after_write =
        client.read_io(1001).await.expect("Failed to read I/O #1001 after write");

    log::info!("I/O #1001 state after write: 0b{io_state_after_write:08b}");
    assert_eq!(io_state_after_write, 0b0000_0001, "I/O #1001 should be ON after write");

    // Additional I/O operations (as per legacy example)
    log::info!("Writing to robot user output I/O #1002...");
    client.write_io(1002, 0b0000_0000).await.expect("Failed to write OFF to I/O #1002");
    log::info!("Successfully set I/O #1002 to OFF");

    // Wait a moment and verify the change
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    log::info!("Verifying I/O #1002 state...");
    let io1002_state_after_write =
        client.read_io(1002).await.expect("Failed to read I/O #1002 after write");

    log::info!("I/O #1002 state after write: 0b{io1002_state_after_write:08b}");
    assert_eq!(io1002_state_after_write, 0b0000_0000, "I/O #1002 should be OFF after write");
});

test_with_logging!(test_read_and_write_io_with_invalid_number, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid I/O number (as per legacy example)
    log::info!("Testing invalid I/O number read...");
    match client.read_io(65535).await {
        Ok(value) => {
            log::error!("✗ Invalid I/O number succeeded unexpectedly: {value}");
            unreachable!("Invalid I/O number should return error");
        }
        Err(e) => {
            log::debug!("✓ Invalid I/O number correctly failed: {e}");
        }
    }

    // Test invalid I/O number for write
    log::info!("Testing invalid I/O number write...");
    match client.write_io(65535, 0b0000_0001).await {
        Ok(()) => {
            log::error!("✗ Invalid I/O number write succeeded unexpectedly");
            unreachable!("Writing to invalid I/O number should return error");
        }
        Err(e) => {
            log::debug!("✓ Invalid I/O number write correctly failed: {e}");
        }
    }
});

test_with_logging!(test_read_multiple_io, {
    let _server = create_io_test_server().await.expect("Failed to start mock server");
    let client = create_test_client().await.expect("Failed to create client");

    // Test reading multiple I/O data
    log::info!("Reading multiple I/O data from robot user input...");
    let io_data = client.read_multiple_io(1, 4).await.expect("Failed to read multiple I/O");
    log::info!("Read {} I/O data bytes", io_data.len());
    assert_eq!(io_data.len(), 4, "Should read exactly 4 I/O data bytes");
    // Verify that I/O #1 is ON (bit 0 of first byte)
    assert_eq!(io_data[0] & 0b0000_0001, 0b0000_0001, "I/O #1 should be ON");

    // Test reading from network input range
    log::info!("Reading multiple I/O data from network input...");
    let io_data = client.read_multiple_io(2701, 2).await.expect("Failed to read multiple I/O");
    log::info!("Read {} I/O data bytes from network input", io_data.len());
    assert_eq!(io_data.len(), 2, "Should read exactly 2 I/O data bytes");
});

test_with_logging!(test_write_multiple_io, {
    let _server = create_io_test_server().await.expect("Failed to start mock server");
    let client = create_test_client().await.expect("Failed to create client");

    // Test writing multiple I/O data to network input signals
    log::info!("Writing multiple I/O data to network input signals...");
    let io_data = vec![0b1010_1010, 0b0101_0101];
    client.write_multiple_io(2701, io_data.clone()).await.expect("Failed to write multiple I/O");
    log::info!("Successfully wrote {} I/O data bytes", io_data.len());

    // Wait a moment and verify the change
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    log::info!("Verifying written I/O data...");
    let read_data = client.read_multiple_io(2701, 2).await.expect("Failed to read back I/O data");
    log::info!("Read back data: {read_data:?}");
    assert_eq!(read_data, io_data, "Read back data should match written data");
});

test_with_logging!(test_multiple_io_validation, {
    let _server = create_io_test_server().await.expect("Failed to start mock server");
    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid count (odd number)
    log::info!("Testing invalid count (odd number)...");
    match client.read_multiple_io(1, 3).await {
        Ok(_) => {
            log::error!("✗ Odd count succeeded unexpectedly");
            unreachable!("Odd count should return error");
        }
        Err(e) => {
            log::debug!("✓ Odd count correctly failed: {e}");
        }
    }

    // Test invalid count (too large)
    log::info!("Testing invalid count (too large)...");
    match client.read_multiple_io(1, 475).await {
        Ok(_) => {
            log::error!("✗ Too large count succeeded unexpectedly");
            unreachable!("Too large count should return error");
        }
        Err(e) => {
            log::debug!("✓ Too large count correctly failed: {e}");
        }
    }

    // Test write to non-writable range
    log::info!("Testing write to non-writable range...");
    let io_data = vec![0b1010_1010, 0b0101_0101];
    match client.write_multiple_io(1, io_data).await {
        Ok(()) => {
            log::error!("✗ Write to non-writable range succeeded unexpectedly");
            unreachable!("Write to non-writable range should return error");
        }
        Err(e) => {
            log::debug!("✓ Write to non-writable range correctly failed: {e}");
        }
    }

    // Test write that exceeds network input range
    log::info!("Testing write that exceeds network input range...");
    let large_io_data = vec![0u8; 258]; // This would exceed the 2701..=2956 range (256 bytes)
    match client.write_multiple_io(2701, large_io_data).await {
        Ok(()) => {
            log::error!("✗ Write exceeding range succeeded unexpectedly");
            unreachable!("Write exceeding range should return error");
        }
        Err(e) => {
            log::debug!("✓ Write exceeding range correctly failed: {e}");
        }
    }

    // Test invalid I/O number (out of range - too low)
    log::info!("Testing invalid I/O number (0 - too low)...");
    match client.read_multiple_io(0, 2).await {
        Ok(_) => {
            log::error!("✗ Invalid I/O number (0) succeeded unexpectedly");
            unreachable!("Invalid I/O number should return error");
        }
        Err(e) => {
            log::debug!("✓ Invalid I/O number (0) correctly failed: {e}");
        }
    }

    // Test invalid I/O number (out of range - too high)
    log::info!("Testing invalid I/O number (9000 - too high)...");
    match client.read_multiple_io(9000, 2).await {
        Ok(_) => {
            log::error!("✗ Invalid I/O number (9000) succeeded unexpectedly");
            unreachable!("Invalid I/O number should return error");
        }
        Err(e) => {
            log::debug!("✓ Invalid I/O number (9000) correctly failed: {e}");
        }
    }

    // Test invalid I/O number for write operation
    log::info!("Testing invalid I/O number for write (10000)...");
    let io_data = vec![0b1010_1010, 0b0101_0101];
    match client.write_multiple_io(10000, io_data).await {
        Ok(()) => {
            log::error!("✗ Invalid I/O number write succeeded unexpectedly");
            unreachable!("Invalid I/O number write should return error");
        }
        Err(e) => {
            log::debug!("✓ Invalid I/O number write correctly failed: {e}");
        }
    }
});

test_with_logging!(test_multiple_io_boundary_conditions, {
    let _server = create_io_test_server().await.expect("Failed to start mock server");
    let client = create_test_client().await.expect("Failed to create client");

    // Test minimum valid count (2)
    log::info!("Testing minimum valid count (2)...");
    let io_data = client.read_multiple_io(1, 2).await.expect("Failed to read minimum count");
    assert_eq!(io_data.len(), 2, "Should read exactly 2 I/O data bytes");

    // Test maximum valid count (474)
    log::info!("Testing maximum valid count (474)...");
    let io_data = client.read_multiple_io(1, 474).await.expect("Failed to read maximum count");
    assert_eq!(io_data.len(), 474, "Should read exactly 474 I/O data bytes");

    // Test writing maximum count to network input (within valid range)
    log::info!("Testing write maximum count to network input...");
    // Calculate maximum safe count: (2956 - 2701 + 1) / 8 = 32 bytes
    let max_safe_count = (2956 - 2701 + 1) / 8;
    let large_io_data = vec![0u8; max_safe_count as usize];
    client
        .write_multiple_io(2701, large_io_data.clone())
        .await
        .expect("Failed to write maximum safe count");

    // Verify the write
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let read_data = client
        .read_multiple_io(2701, max_safe_count)
        .await
        .expect("Failed to read back maximum safe count");
    assert_eq!(read_data, large_io_data, "Read back data should match written data");
});
