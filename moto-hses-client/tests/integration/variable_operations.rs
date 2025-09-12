// Integration tests for variable operations

use crate::common::{
    mock_server_setup::create_variable_test_server,
    test_utils::{create_test_client, wait_for_operation},
};
use crate::test_with_logging;

test_with_logging!(test_variable_read_operations, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading different variable types with expected values from MockServer configuration
    assert_eq!(
        client
            .read_i16(0)
            .await
            .expect("Failed to read I16 variable"),
        100
    );
    assert_eq!(
        client
            .read_i16(1)
            .await
            .expect("Failed to read I16 variable"),
        200
    );

    assert_eq!(
        client
            .read_i32(10)
            .await
            .expect("Failed to read I32 variable"),
        1000
    );
    assert_eq!(
        client
            .read_i32(11)
            .await
            .expect("Failed to read I32 variable"),
        2000
    );

    assert_eq!(
        client
            .read_f32(20)
            .await
            .expect("Failed to read F32 variable"),
        1.5
    );
    assert_eq!(
        client
            .read_f32(21)
            .await
            .expect("Failed to read F32 variable"),
        2.5
    );

    assert_eq!(
        client
            .read_u8(30)
            .await
            .expect("Failed to read U8 variable"),
        10
    );
    assert_eq!(
        client
            .read_u8(31)
            .await
            .expect("Failed to read U8 variable"),
        20
    );

    // Test string variables
    let s0 = client
        .read_string(40)
        .await
        .expect("Failed to read string variable");
    assert_eq!(String::from_utf8_lossy(&s0), "Hello");

    let s1 = client
        .read_string(41)
        .await
        .expect("Failed to read string variable");
    assert_eq!(String::from_utf8_lossy(&s1), "World");
});

test_with_logging!(test_variable_write_operations, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing different variable types
    client
        .write_i16(1, 42)
        .await
        .expect("Failed to write i16 variable");

    client
        .write_i32(11, 12345)
        .await
        .expect("Failed to write i32 variable");

    client
        .write_f32(21, std::f32::consts::PI)
        .await
        .expect("Failed to write f32 variable");

    client
        .write_u8(31, 255)
        .await
        .expect("Failed to write u8 variable");

    wait_for_operation().await;

    // Verify written values
    assert_eq!(
        client
            .read_i16(1)
            .await
            .expect("Failed to read i16 after write"),
        42
    );
    assert_eq!(
        client
            .read_i32(11)
            .await
            .expect("Failed to read i32 after write"),
        12345
    );
    assert!(
        (client
            .read_f32(21)
            .await
            .expect("Failed to read f32 after write")
            - std::f32::consts::PI)
            .abs()
            < 0.001
    );
    assert_eq!(
        client
            .read_u8(31)
            .await
            .expect("Failed to read u8 after write"),
        255
    );
});

test_with_logging!(test_string_variable_operations, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test string operations with expected initial values
    // Verify initial string values
    let s0 = client
        .read_string(40)
        .await
        .expect("Failed to read initial string");
    assert_eq!(String::from_utf8_lossy(&s0), "Hello");

    let s1 = client
        .read_string(41)
        .await
        .expect("Failed to read initial string");
    assert_eq!(String::from_utf8_lossy(&s1), "World");

    // Test writing new string
    let test_string = "Hello, Robot!";
    client
        .write_string(40, test_string.as_bytes().to_vec())
        .await
        .expect("Failed to write string variable");

    wait_for_operation().await;

    // Read string back and verify
    let read_string_bytes = client
        .read_string(40)
        .await
        .expect("Failed to read string variable");

    let read_string = String::from_utf8_lossy(&read_string_bytes);
    assert_eq!(read_string.trim_end_matches('\0'), test_string);
});

test_with_logging!(test_invalid_variable_handling, {
    let _server = create_variable_test_server()
        .await
        .expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid variable index for read
    let result: Result<i16, _> = client.read_i16(255).await;
    assert!(
        result.is_err(),
        "Invalid variable index should return error"
    );

    let result: Result<i32, _> = client.read_i32(255).await;
    assert!(
        result.is_err(),
        "Invalid variable index should return error"
    );

    let result: Result<f32, _> = client.read_f32(255).await;
    assert!(
        result.is_err(),
        "Invalid variable index should return error"
    );

    let result: Result<u8, _> = client.read_u8(255).await;
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

    // Test invalid variable index for write
    let result: Result<(), _> = client.write_i16(255, 42).await;
    assert!(
        result.is_err(),
        "Invalid variable index write should return error"
    );

    let result: Result<(), _> = client.write_string(255, b"test".to_vec()).await;
    assert!(
        result.is_err(),
        "Invalid string variable index write should return error"
    );
});
