#![allow(clippy::expect_used)]
// Integration tests for register operations

use crate::common::{
    mock_server_setup::create_register_test_server,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_register_read_operations, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading registers with expected values from MockServer configuration
    assert_eq!(client.read_register(0).await.expect("Failed to read register 0"), 0);
    assert_eq!(client.read_register(1).await.expect("Failed to read register 1"), 100);
    assert_eq!(client.read_register(2).await.expect("Failed to read register 2"), 200);
    assert_eq!(client.read_register(3).await.expect("Failed to read register 3"), 300);
    assert_eq!(client.read_register(4).await.expect("Failed to read register 4"), 400);
});

test_with_logging!(test_register_write_operations, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing to register 0 (initial value is 0)
    let test_value = 42;
    client.write_register(0, test_value).await.expect("Failed to write to register 0");

    wait_for_operation().await;

    // Verify the write operation
    let register_value_after_write =
        client.read_register(0).await.expect("Failed to read register 0 after write");

    assert_eq!(register_value_after_write, test_value);
});

test_with_logging!(test_multiple_register_operations, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test operations on multiple registers with expected initial values
    let test_registers = [1, 2, 3, 4];
    let expected_initial_values = [100, 200, 300, 400];
    let test_values = [150, 250, 350, 450];

    // First, verify initial values
    for (register, expected_value) in test_registers.iter().zip(expected_initial_values.iter()) {
        let initial_value = client
            .read_register(*register)
            .await
            .expect("Failed to read initial value from register");
        assert_eq!(initial_value, *expected_value);
    }

    // Then test write operations
    for (register, value) in test_registers.iter().zip(test_values.iter()) {
        client.write_register(*register, *value).await.expect("Failed to write to register");

        wait_for_operation().await;

        let read_value = client.read_register(*register).await.expect("Failed to read register");

        assert_eq!(read_value, *value);
    }
});

test_with_logging!(test_register_boundary_values, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    let register = 0;
    let boundary_values = vec![0, 1, 255, 256, 32767, -32768];

    for value in boundary_values {
        // Write boundary value
        let write_result = client.write_register(register, value).await;

        if write_result.is_ok() {
            wait_for_operation().await;

            // Read and verify if write was successful
            let read_value = client
                .read_register(register)
                .await
                .expect("Failed to read register after successful write");
            assert_eq!(read_value, value);
        }
        // Note: Some boundary values might be invalid depending on register size
        // This test documents the behavior rather than enforcing specific limits
    }
});

test_with_logging!(test_register_error_handling, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid register number for read (65535)
    assert!(client.read_register(65535).await.is_err(), "Invalid register number read should fail");

    // Test invalid register number for write (65535)
    assert!(
        client.write_register(65535, 42).await.is_err(),
        "Invalid register number write should fail"
    );
});

test_with_logging!(test_read_multiple_registers, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading multiple registers
    log::info!("Reading multiple registers from 0 to 4...");
    let values =
        client.read_multiple_registers(0, 5).await.expect("Failed to read multiple registers");
    log::info!("Read {} register values", values.len());
    assert_eq!(values.len(), 5, "Should read exactly 5 register values");

    // Verify expected values from MockServer configuration
    assert_eq!(values[0], 0, "Register 0 should be 0");
    assert_eq!(values[1], 100, "Register 1 should be 100");
    assert_eq!(values[2], 200, "Register 2 should be 200");
    assert_eq!(values[3], 300, "Register 3 should be 300");
    assert_eq!(values[4], 400, "Register 4 should be 400");

    // Test reading from different range
    log::info!("Reading multiple registers from 1 to 3...");
    let values =
        client.read_multiple_registers(1, 3).await.expect("Failed to read multiple registers");
    assert_eq!(values.len(), 3, "Should read exactly 3 register values");
    assert_eq!(values[0], 100, "Register 1 should be 100");
    assert_eq!(values[1], 200, "Register 2 should be 200");
    assert_eq!(values[2], 300, "Register 3 should be 300");
});

test_with_logging!(test_write_multiple_registers, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing multiple registers
    log::info!("Writing multiple registers to 0-4...");
    let values = vec![1000, 2000, 3000, 4000, 5000];
    client
        .write_multiple_registers(0, values.clone())
        .await
        .expect("Failed to write multiple registers");
    log::info!("Successfully wrote {} register values", values.len());

    // Wait and verify the changes
    wait_for_operation().await;
    log::info!("Verifying written register values...");
    let read_values =
        client.read_multiple_registers(0, 5).await.expect("Failed to read back multiple registers");
    log::info!("Read back values: {read_values:?}");
    assert_eq!(read_values, values, "Read back values should match written values");

    // Test writing to different range
    log::info!("Writing multiple registers to 10-12...");
    let values = vec![111, 222, 333];
    client
        .write_multiple_registers(10, values.clone())
        .await
        .expect("Failed to write multiple registers");

    wait_for_operation().await;
    let read_values = client
        .read_multiple_registers(10, 3)
        .await
        .expect("Failed to read back multiple registers");
    assert_eq!(read_values, values, "Read back values should match written values");
});

test_with_logging!(test_multiple_registers_validation, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid count (0)
    match client.read_multiple_registers(0, 0).await {
        Ok(_) => {
            log::error!("✗ Read with count 0 succeeded unexpectedly");
            unreachable!("Read with count 0 should return error");
        }
        Err(e) => {
            log::debug!("✓ Read with count 0 correctly failed: {e}");
        }
    }

    // Test invalid count (238)
    match client.read_multiple_registers(0, 238).await {
        Ok(_) => {
            log::error!("✗ Read with count 238 succeeded unexpectedly");
            unreachable!("Read with count 238 should return error");
        }
        Err(e) => {
            log::debug!("✓ Read with count 238 correctly failed: {e}");
        }
    }

    // Test range overflow (start + count - 1 > 999)
    match client.read_multiple_registers(999, 2).await {
        Ok(_) => {
            log::error!("✗ Read with range overflow succeeded unexpectedly");
            unreachable!("Read with range overflow should return error");
        }
        Err(e) => {
            log::debug!("✓ Read with range overflow correctly failed: {e}");
        }
    }

    // Test write to non-writable range (560-999)
    match client.write_multiple_registers(560, vec![123]).await {
        Ok(()) => {
            log::error!("✗ Write to non-writable range succeeded unexpectedly");
            unreachable!("Write to non-writable range should return error");
        }
        Err(e) => {
            log::debug!("✓ Write to non-writable range correctly failed: {e}");
        }
    }

    // Test write with range crossing writable boundary
    match client.write_multiple_registers(559, vec![123, 456]).await {
        Ok(()) => {
            log::error!("✗ Write crossing writable boundary succeeded unexpectedly");
            unreachable!("Write crossing writable boundary should return error");
        }
        Err(e) => {
            log::debug!("✓ Write crossing writable boundary correctly failed: {e}");
        }
    }
});

test_with_logging!(test_multiple_registers_boundary_conditions, {
    let _server =
        create_register_test_server().await.expect("Failed to start register test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test minimum valid count (1)
    log::info!("Testing minimum valid count (1)...");
    let values = client.read_multiple_registers(0, 1).await.expect("Failed to read minimum count");
    assert_eq!(values.len(), 1, "Should read exactly 1 register value");

    // Test maximum valid count (237)
    log::info!("Testing maximum valid count (237)...");
    let values =
        client.read_multiple_registers(0, 237).await.expect("Failed to read maximum count");
    assert_eq!(values.len(), 237, "Should read exactly 237 register values");

    // Test writing maximum count to writable range
    log::info!("Testing write maximum count to writable range...");
    let large_values: Vec<i16> =
        (0..237).map(|i| i16::try_from(i + 1000).expect("i + 1000 should fit in i16")).collect();
    client
        .write_multiple_registers(0, large_values.clone())
        .await
        .expect("Failed to write maximum count");

    // Verify the write
    wait_for_operation().await;
    let read_values =
        client.read_multiple_registers(0, 237).await.expect("Failed to read back maximum count");
    assert_eq!(read_values, large_values, "Read back values should match written values");

    // Test writable range boundary (559 is last writable register)
    log::info!("Testing writable range boundary...");
    let boundary_values = vec![9999];
    client
        .write_multiple_registers(559, boundary_values.clone())
        .await
        .expect("Failed to write to boundary register");

    wait_for_operation().await;
    let read_values =
        client.read_multiple_registers(559, 1).await.expect("Failed to read boundary register");
    assert_eq!(read_values, boundary_values, "Boundary register should be writable");
});
