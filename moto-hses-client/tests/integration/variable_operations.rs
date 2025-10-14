#![allow(clippy::expect_used)]
#![allow(clippy::float_cmp)]
// Integration tests for variable operations

use crate::common::{
    mock_server_setup::create_variable_test_server,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_variable_read_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading different variable types with expected values from MockServer configuration
    assert_eq!(client.read_i16(0).await.expect("Failed to read I16 variable"), 100);
    assert_eq!(client.read_i16(1).await.expect("Failed to read I16 variable"), 200);

    assert_eq!(client.read_i32(10).await.expect("Failed to read I32 variable"), 1000);
    assert_eq!(client.read_i32(11).await.expect("Failed to read I32 variable"), 2000);

    assert_eq!(client.read_f32(20).await.expect("Failed to read F32 variable"), 1.5);
    assert_eq!(client.read_f32(21).await.expect("Failed to read F32 variable"), 2.5);

    assert_eq!(client.read_u8(30).await.expect("Failed to read U8 variable"), 10);
    assert_eq!(client.read_u8(31).await.expect("Failed to read U8 variable"), 20);

    // Test string variables
    let s0 = client.read_string(40).await.expect("Failed to read string variable");
    assert_eq!(String::from_utf8_lossy(&s0), "Hello");

    let s1 = client.read_string(41).await.expect("Failed to read string variable");
    assert_eq!(String::from_utf8_lossy(&s1), "World");
});

test_with_logging!(test_variable_write_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing different variable types
    client.write_i16(1, 42).await.expect("Failed to write i16 variable");

    client.write_i32(11, 12345).await.expect("Failed to write i32 variable");

    client.write_f32(21, std::f32::consts::PI).await.expect("Failed to write f32 variable");

    client.write_u8(31, 255).await.expect("Failed to write u8 variable");

    wait_for_operation().await;

    // Verify written values
    assert_eq!(client.read_i16(1).await.expect("Failed to read i16 after write"), 42);
    assert_eq!(client.read_i32(11).await.expect("Failed to read i32 after write"), 12345);
    assert!(
        (client.read_f32(21).await.expect("Failed to read f32 after write") - std::f32::consts::PI)
            .abs()
            < 0.001
    );
    assert_eq!(client.read_u8(31).await.expect("Failed to read u8 after write"), 255);
});

test_with_logging!(test_string_variable_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test string operations with expected initial values
    // Verify initial string values
    let s0 = client.read_string(40).await.expect("Failed to read initial string");
    assert_eq!(String::from_utf8_lossy(&s0), "Hello");

    let s1 = client.read_string(41).await.expect("Failed to read initial string");
    assert_eq!(String::from_utf8_lossy(&s1), "World");

    // Test writing new string
    let test_string = "Hello, Robot!";
    client
        .write_string(40, test_string.as_bytes().to_vec())
        .await
        .expect("Failed to write string variable");

    wait_for_operation().await;

    // Read string back and verify
    let read_string_bytes = client.read_string(40).await.expect("Failed to read string variable");

    let read_string = String::from_utf8_lossy(&read_string_bytes);
    assert_eq!(read_string.trim_end_matches('\0'), test_string);
});

test_with_logging!(test_invalid_variable_handling, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid variable index for read
    let result: Result<i16, _> = client.read_i16(255).await;
    assert!(result.is_err(), "Invalid variable index should return error");

    let result: Result<i32, _> = client.read_i32(255).await;
    assert!(result.is_err(), "Invalid variable index should return error");

    let result: Result<f32, _> = client.read_f32(255).await;
    assert!(result.is_err(), "Invalid variable index should return error");

    let result: Result<u8, _> = client.read_u8(255).await;
    assert!(result.is_err(), "Invalid variable index should return error");

    // Test invalid string variable index
    let result: Result<Vec<u8>, _> = client.read_string(255).await;
    assert!(result.is_err(), "Invalid string variable index should return error");

    // Test invalid variable index for write
    let result: Result<(), _> = client.write_i16(255, 42).await;
    assert!(result.is_err(), "Invalid variable index write should return error");

    let result: Result<(), _> = client.write_string(255, b"test".to_vec()).await;
    assert!(result.is_err(), "Invalid string variable index write should return error");
});

test_with_logging!(test_multiple_byte_variables_read_write, {
    let _server = create_variable_test_server().await.expect("Failed to start variable test server");
    let client = create_test_client().await.expect("Failed to create client");

    // Write multiple byte variables (count must be multiple of 2)
    let values = vec![10, 20, 30, 40];
    client.write_multiple_byte_variables(0, values.clone()).await.expect("Failed to write multiple byte variables");

    wait_for_operation().await;

    // Read back and verify
    let read_values = client.read_multiple_byte_variables(0, 4).await.expect("Failed to read multiple byte variables");
    assert_eq!(read_values, values);

    // Test different range
    let values2 = vec![255, 0, 100, 200];
    client.write_multiple_byte_variables(10, values2.clone()).await.expect("Failed to write multiple byte variables");

    wait_for_operation().await;

    let read_values2 = client.read_multiple_byte_variables(10, 4).await.expect("Failed to read multiple byte variables");
    assert_eq!(read_values2, values2);
});

test_with_logging!(test_multiple_byte_variables_boundary, {
    let _server = create_variable_test_server().await.expect("Failed to start variable test server");
    let client = create_test_client().await.expect("Failed to create client");

    // Test boundary: variables 98-99 (maximum range)
    let boundary_values = vec![99, 100];
    client.write_multiple_byte_variables(98, boundary_values.clone()).await.expect("Failed to write boundary variables");

    wait_for_operation().await;

    let read_boundary = client.read_multiple_byte_variables(98, 2).await.expect("Failed to read boundary variables");
    assert_eq!(read_boundary, boundary_values);

    // Test maximum safe range: variables 0-99 (100 variables)
    let max_values: Vec<u8> = (0..100).map(|i| (i % 256) as u8).collect();
    client.write_multiple_byte_variables(0, max_values.clone()).await.expect("Failed to write max range variables");

    wait_for_operation().await;

    let read_max = client.read_multiple_byte_variables(0, 100).await.expect("Failed to read max range variables");
    assert_eq!(read_max, max_values);
});

test_with_logging!(test_multiple_byte_variables_validation, {
    let _server = create_variable_test_server().await.expect("Failed to start variable test server");
    let client = create_test_client().await.expect("Failed to create client");

    // Test count must be multiple of 2 (should fail)
    match client.read_multiple_byte_variables(0, 3).await {
        Ok(_) => panic!("Should fail for odd count"),
        Err(_) => {} // Expected
    }

    match client.write_multiple_byte_variables(0, vec![10, 20, 30]).await {
        Ok(_) => panic!("Should fail for odd count"),
        Err(_) => {} // Expected
    }

    // Test range overflow (should fail)
    match client.read_multiple_byte_variables(99, 4).await {
        Ok(_) => panic!("Should fail for range overflow"),
        Err(_) => {} // Expected
    }

    match client.write_multiple_byte_variables(99, vec![10, 20, 30, 40]).await {
        Ok(_) => panic!("Should fail for range overflow"),
        Err(_) => {} // Expected
    }

    // Test invalid variable number (should fail)
    match client.read_multiple_byte_variables(100, 2).await {
        Ok(_) => panic!("Should fail for invalid variable number"),
        Err(_) => {} // Expected
    }

    match client.write_multiple_byte_variables(100, vec![10, 20]).await {
        Ok(_) => panic!("Should fail for invalid variable number"),
        Err(_) => {} // Expected
    }

    // Test zero count (should fail)
    match client.read_multiple_byte_variables(0, 0).await {
        Ok(_) => panic!("Should fail for zero count"),
        Err(_) => {} // Expected
    }

    match client.write_multiple_byte_variables(0, vec![]).await {
        Ok(_) => panic!("Should fail for empty values"),
        Err(_) => {} // Expected
    }
});

test_with_logging!(test_multiple_byte_variables_mixed_operations, {
    let _server = create_variable_test_server().await.expect("Failed to start variable test server");
    let client = create_test_client().await.expect("Failed to create client");

    // Write some individual variables first
    client.write_u8(5, 50).await.expect("Failed to write single byte variable");
    client.write_u8(6, 60).await.expect("Failed to write single byte variable");

    wait_for_operation().await;

    // Now write multiple variables that overlap
    let values = vec![55, 65];
    client.write_multiple_byte_variables(5, values.clone()).await.expect("Failed to write multiple byte variables");

    wait_for_operation().await;

    // Read back individual variables to verify
    let val5 = client.read_u8(5).await.expect("Failed to read single byte variable");
    let val6 = client.read_u8(6).await.expect("Failed to read single byte variable");
    assert_eq!(val5, 55);
    assert_eq!(val6, 65);

    // Read back using multiple variable read
    let read_values = client.read_multiple_byte_variables(5, 2).await.expect("Failed to read multiple byte variables");
    assert_eq!(read_values, values);
});
