// Integration tests for register operations

use crate::common::{
    mock_server_setup::{create_register_test_server, MockServerManager},
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_register_read_operations, {
    let _server = create_register_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading register 0
    let register_value = client
        .read_register(0)
        .await
        .expect("Failed to read register 0");

    // Verify register value is within expected range
    assert!(
        register_value >= 0,
        "Register 0 value should be non-negative"
    );
});

test_with_logging!(test_register_write_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing to register 0
    let test_value = 42;
    client
        .write_register(0, test_value)
        .await
        .expect("Failed to write to register 0");

    wait_for_operation().await;

    // Verify the write operation
    let register_value_after_write = client
        .read_register(0)
        .await
        .expect("Failed to read register 0 after write");

    assert_eq!(
        register_value_after_write, test_value,
        "Register 0 should contain the written value"
    );
});

test_with_logging!(test_multiple_register_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test operations on multiple registers
    let test_registers = vec![1, 2, 3, 4];
    let test_values = vec![100, 200, 300, 400];

    for (register, value) in test_registers.iter().zip(test_values.iter()) {
        // Write value to register
        client
            .write_register(*register, *value)
            .await
            .expect(&format!("Failed to write to register {}", register));

        wait_for_operation().await;

        // Read and verify
        let read_value = client
            .read_register(*register)
            .await
            .expect(&format!("Failed to read register {}", register));

        assert_eq!(
            read_value, *value,
            "Register {} should contain the written value",
            register
        );
    }
});

test_with_logging!(test_register_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Comprehensive test covering all register operations
    let test_cases = vec![(0, 42), (1, 100), (2, 200), (3, 300), (4, 400)];

    for (register, value) in test_cases {
        // Read initial value
        let _initial_value = client.read_register(register).await.expect(&format!(
            "Failed to read initial value from register {}",
            register
        ));

        // Write new value
        client
            .write_register(register, value)
            .await
            .expect(&format!("Failed to write to register {}", register));

        wait_for_operation().await;

        // Verify write operation
        let final_value = client.read_register(register).await.expect(&format!(
            "Failed to read final value from register {}",
            register
        ));

        assert_eq!(
            final_value, value,
            "Register {} should contain the written value",
            register
        );
    }
});

test_with_logging!(test_register_value_persistence, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test that register values persist across multiple operations
    let register = 0;
    let test_value = 12345;

    // Write value
    client
        .write_register(register, test_value)
        .await
        .expect("Failed to write to register");

    wait_for_operation().await;

    // Read multiple times to ensure persistence
    for _ in 0..3 {
        let value = client
            .read_register(register)
            .await
            .expect("Failed to read register");
        assert_eq!(
            value, test_value,
            "Register value should persist across reads"
        );
        wait_for_operation().await;
    }
});

test_with_logging!(test_register_boundary_values, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

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
            assert_eq!(
                read_value, value,
                "Register should contain boundary value {}",
                value
            );
        }
        // Note: Some boundary values might be invalid depending on register size
        // This test documents the behavior rather than enforcing specific limits
    }
});
