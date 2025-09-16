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
