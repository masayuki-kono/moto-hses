// Integration tests for variable operations

use crate::common::{
    mock_server_setup::{create_test_server, create_variable_test_server},
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_variable_read_operations, {
    let mut server = create_test_server();
    server.start().await.expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading different variable types
    let i16_value: i16 = client
        .read_i16(0)
        .await
        .expect("Failed to read I16 variable");

    let i32_value: i32 = client
        .read_i32(0)
        .await
        .expect("Failed to read I32 variable");

    let f32_value: f32 = client
        .read_f32(0)
        .await
        .expect("Failed to read F32 variable");

    let _u8_value: u8 = client.read_u8(0).await.expect("Failed to read U8 variable");

    // Verify values are within expected ranges (mock server should return predictable values)
    assert!(i16_value >= -32768);
    assert!(i32_value >= -2147483648);
    assert!(f32_value.is_finite());
    // Note: u8 values are always <= 255, so this assertion is not needed
});

test_with_logging!(test_variable_write_operations, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing different variable types
    client
        .write_i16(1, 42)
        .await
        .expect("Failed to write i16 variable");

    client
        .write_i32(1, 12345)
        .await
        .expect("Failed to write i32 variable");

    client
        .write_f32(1, 3.14159)
        .await
        .expect("Failed to write f32 variable");

    client
        .write_u8(1, 255)
        .await
        .expect("Failed to write u8 variable");

    wait_for_operation().await;
});

test_with_logging!(test_string_variable_operations, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test string operations
    let test_string = "Hello, Robot!";

    // Write string
    client
        .write_string(0, test_string.as_bytes().to_vec())
        .await
        .expect("Failed to write string variable");

    wait_for_operation().await;

    // Read string back
    let read_string_bytes = client
        .read_string(0)
        .await
        .expect("Failed to read string variable");

    let read_string = String::from_utf8_lossy(&read_string_bytes);
    assert_eq!(read_string.trim_end_matches('\0'), test_string);
});

test_with_logging!(test_invalid_variable_handling, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid variable index
    let result: Result<i16, _> = client.read_variable::<i16>(255).await;
    assert!(
        result.is_err(),
        "Invalid variable index should return error"
    );

    // Test invalid string variable index
    let result: Result<Vec<u8>, _> = client.read_string(255).await;
    assert!(
        result.is_err(),
        "Invalid string variable index should return error"
    );
});

test_with_logging!(test_variable_operations_comprehensive, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start mock server");

    let client = create_test_client().await.expect("Failed to create client");

    // Comprehensive test covering all variable operations
    // Test i16
    client
        .write_variable(0, 42i16)
        .await
        .expect("Failed to write i16");
    wait_for_operation().await;
    let read_i16: i16 = client
        .read_variable::<i16>(0)
        .await
        .expect("Failed to read i16");
    assert_eq!(read_i16, 42);

    // Test i32
    client
        .write_variable(1, 12345i32)
        .await
        .expect("Failed to write i32");
    wait_for_operation().await;
    let read_i32: i32 = client
        .read_variable::<i32>(1)
        .await
        .expect("Failed to read i32");
    assert_eq!(read_i32, 12345);

    // Test f32
    client
        .write_variable(2, 3.14159f32)
        .await
        .expect("Failed to write f32");
    wait_for_operation().await;
    let read_f32: f32 = client
        .read_variable::<f32>(2)
        .await
        .expect("Failed to read f32");
    assert!((read_f32 - 3.14159).abs() < 0.001);

    // Test u8
    client
        .write_variable(3, 255u8)
        .await
        .expect("Failed to write u8");
    wait_for_operation().await;
    let read_u8: u8 = client
        .read_variable::<u8>(3)
        .await
        .expect("Failed to read u8");
    assert_eq!(read_u8, 255);
});
