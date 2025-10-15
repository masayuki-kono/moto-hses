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
    assert_eq!(s0, "Hello");

    let s1 = client.read_string(41).await.expect("Failed to read string variable");
    assert_eq!(s1, "World");
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
    assert_eq!(s0, "Hello");

    let s1 = client.read_string(41).await.expect("Failed to read initial string");
    assert_eq!(s1, "World");

    // Test writing new string
    let test_string = "Hello, Robot!";
    client
        .write_string(40, test_string.to_string())
        .await
        .expect("Failed to write string variable");

    wait_for_operation().await;

    // Read string back and verify
    let read_string = client.read_string(40).await.expect("Failed to read string variable");
    assert_eq!(read_string, test_string);
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
    let result: Result<String, _> = client.read_string(255).await;
    assert!(result.is_err(), "Invalid string variable index should return error");

    // Test invalid variable index for write
    let result: Result<(), _> = client.write_i16(255, 42).await;
    assert!(result.is_err(), "Invalid variable index write should return error");

    let result: Result<(), _> = client.write_string(255, "test".to_string()).await;
    assert!(result.is_err(), "Invalid string variable index write should return error");
});

test_with_logging!(test_multiple_byte_variables_read_write, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading multiple byte variables (count must be multiple of 2)
    let values = vec![10, 20, 30, 40];
    client
        .write_multiple_u8(0, values.clone())
        .await
        .expect("Failed to write multiple byte variables");

    wait_for_operation().await;

    // Read back and verify
    let read_values =
        client.read_multiple_u8(0, 4).await.expect("Failed to read multiple byte variables");
    assert_eq!(read_values, values);

    // Test boundary conditions
    let boundary_values = vec![99, 100];
    client
        .write_multiple_u8(98, boundary_values.clone())
        .await
        .expect("Failed to write boundary byte variables");

    wait_for_operation().await;

    let read_boundary =
        client.read_multiple_u8(98, 2).await.expect("Failed to read boundary byte variables");
    assert_eq!(read_boundary, boundary_values);
});

test_with_logging!(test_multiple_byte_variables_validation, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test count must be multiple of 2 (should fail)
    let result = client.read_multiple_u8(0, 3).await;
    assert!(result.is_err(), "Odd count should fail");

    let result = client.write_multiple_u8(0, vec![1, 2, 3]).await;
    assert!(result.is_err(), "Odd count write should fail");

    // Test count too large (should fail)
    let result = client.read_multiple_u8(0, 475).await;
    assert!(result.is_err(), "Count too large should fail");

    // Note: Instance range validation removed to support extended settings
    // The actual variable range is now configurable and not limited to 0-99

    // Test zero count (should fail)
    let result = client.read_multiple_u8(0, 0).await;
    assert!(result.is_err(), "Zero count should fail");
});

test_with_logging!(test_multiple_byte_variables_large_batch, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test maximum safe count (limited by variable number range 0-99)
    let large_values: Vec<u8> =
        (0..100).map(|i| u8::try_from(i % 256).expect("Should fit in u8")).collect();
    client
        .write_multiple_u8(0, large_values.clone())
        .await
        .expect("Failed to write large batch of byte variables");

    wait_for_operation().await;

    let read_large = client
        .read_multiple_u8(0, 100)
        .await
        .expect("Failed to read large batch of byte variables");
    assert_eq!(read_large, large_values);
});

test_with_logging!(test_multiple_integer_variables_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing multiple integer variables
    let values = vec![100, -200, 300, -400];
    client
        .write_multiple_i16(0, values.clone())
        .await
        .expect("Failed to write multiple integer variables");

    wait_for_operation().await;

    // Read back and verify
    let read_values =
        client.read_multiple_i16(0, 4).await.expect("Failed to read multiple integer variables");
    assert_eq!(read_values, values);

    // Test boundary conditions
    let boundary_values = vec![99, -100];
    client
        .write_multiple_i16(98, boundary_values.clone())
        .await
        .expect("Failed to write boundary integer variables");

    wait_for_operation().await;

    let read_boundary =
        client.read_multiple_i16(98, 2).await.expect("Failed to read boundary integer variables");
    assert_eq!(read_boundary, boundary_values);
});

test_with_logging!(test_multiple_integer_variables_validation, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test count too large (should fail)
    let result = client.read_multiple_i16(0, 238).await;
    assert!(result.is_err(), "Count too large should fail");

    // Test zero count (should fail)
    let result = client.read_multiple_i16(0, 0).await;
    assert!(result.is_err(), "Zero count should fail");

    // Test empty values write (should fail)
    let result = client.write_multiple_i16(0, vec![]).await;
    assert!(result.is_err(), "Empty values should fail");
});

test_with_logging!(test_multiple_integer_variables_large_batch, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test maximum safe count (limited by variable number range 0-99)
    let large_values: Vec<i16> = (0..100).map(|i| i16::try_from(i % 1000).unwrap_or(0)).collect();
    client
        .write_multiple_i16(0, large_values.clone())
        .await
        .expect("Failed to write large batch of integer variables");

    wait_for_operation().await;

    let read_large = client
        .read_multiple_i16(0, 100)
        .await
        .expect("Failed to read large batch of integer variables");
    assert_eq!(read_large, large_values);
});

test_with_logging!(test_multiple_integer_variables_maximum_count, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test maximum count (237) - this tests the protocol limit
    let max_values: Vec<i16> = (0..237).map(|i| i16::try_from(i % 1000).unwrap_or(0)).collect();
    client
        .write_multiple_i16(0, max_values.clone())
        .await
        .expect("Failed to write maximum count integer variables");

    wait_for_operation().await;

    let read_max = client
        .read_multiple_i16(0, 237)
        .await
        .expect("Failed to read maximum count integer variables");
    assert_eq!(read_max, max_values);
});

test_with_logging!(test_plural_double_variable_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading multiple double precision integer variables
    let read_values =
        client.read_multiple_i32(0, 4).await.expect("Failed to read multiple double variables");
    assert_eq!(read_values.len(), 4);
    assert_eq!(read_values, vec![0, 0, 0, 0]); // Default values

    // Test writing multiple double precision integer variables
    let values = vec![1_000_000, -2_000_000, 2_147_483_647, -2_147_483_648];
    client
        .write_multiple_i32(0, values.clone())
        .await
        .expect("Failed to write multiple double variables");

    wait_for_operation().await;

    // Read back and verify
    let read_values = client
        .read_multiple_i32(0, 4)
        .await
        .expect("Failed to read back multiple double variables");
    assert_eq!(read_values, values);

    // Test with different start variable
    let values2 = vec![500_000, -1_500_000];
    client
        .write_multiple_i32(10, values2.clone())
        .await
        .expect("Failed to write multiple double variables at offset 10");

    wait_for_operation().await;

    let read_values2 = client
        .read_multiple_i32(10, 2)
        .await
        .expect("Failed to read multiple double variables at offset 10");
    assert_eq!(read_values2, values2);
});

test_with_logging!(test_plural_double_variable_validation, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid count: zero
    let result: Result<Vec<i32>, _> = client.read_multiple_i32(0, 0).await;
    assert!(result.is_err(), "Count 0 should return error");

    let result: Result<(), _> = client.write_multiple_i32(0, vec![]).await;
    assert!(result.is_err(), "Empty values should return error");

    // Test invalid count: too large
    let result: Result<Vec<i32>, _> = client.read_multiple_i32(0, 119).await;
    assert!(result.is_err(), "Count 119 should return error");

    let large_values: Vec<i32> = vec![0; 119];
    let result: Result<(), _> = client.write_multiple_i32(0, large_values).await;
    assert!(result.is_err(), "119 values should return error");
});

test_with_logging!(test_plural_double_variable_maximum_count, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test maximum count (118)
    let max_values: Vec<i32> = (0..118).map(|i| i * 1000).collect();
    client
        .write_multiple_i32(0, max_values.clone())
        .await
        .expect("Failed to write maximum count double variables");

    wait_for_operation().await;

    let read_max = client
        .read_multiple_i32(0, 118)
        .await
        .expect("Failed to read maximum count double variables");
    assert_eq!(read_max, max_values);
});

test_with_logging!(test_plural_real_variable_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test reading multiple real type variables
    let read_values =
        client.read_multiple_f32(0, 4).await.expect("Failed to read multiple real variables");
    assert_eq!(read_values.len(), 4);
    assert_eq!(read_values, vec![0.0, 0.0, 0.0, 0.0]); // Default values

    // Test writing multiple real type variables
    let values = vec![1.5, -2.75, std::f32::consts::PI, -4.0];
    client
        .write_multiple_f32(0, values.clone())
        .await
        .expect("Failed to write multiple real variables");

    wait_for_operation().await;

    // Read back and verify
    let read_values =
        client.read_multiple_f32(0, 4).await.expect("Failed to read back multiple real variables");
    assert_eq!(read_values, values);

    // Test with different start variable
    let values2 = vec![0.5, -1.25];
    client
        .write_multiple_f32(10, values2.clone())
        .await
        .expect("Failed to write multiple real variables at offset 10");

    wait_for_operation().await;

    let read_values2 = client
        .read_multiple_f32(10, 2)
        .await
        .expect("Failed to read multiple real variables at offset 10");
    assert_eq!(read_values2, values2);
});

test_with_logging!(test_plural_real_variable_validation, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid count: zero
    let result: Result<Vec<f32>, _> = client.read_multiple_f32(0, 0).await;
    assert!(result.is_err(), "Zero count should fail");

    // Test invalid count: too large
    let result: Result<Vec<f32>, _> = client.read_multiple_f32(0, 119).await;
    assert!(result.is_err(), "Count too large should fail");

    // Test empty values write (should fail)
    let result: Result<(), _> = client.write_multiple_f32(0, vec![]).await;
    assert!(result.is_err(), "Empty values should fail");

    // Test values count too large (should fail)
    let large_values: Vec<f32> = vec![0.0; 119];
    let result: Result<(), _> = client.write_multiple_f32(0, large_values).await;
    assert!(result.is_err(), "Values count too large should fail");
});

test_with_logging!(test_plural_real_variable_large_batch, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test large batch of real variables
    #[allow(clippy::cast_precision_loss)]
    let large_values: Vec<f32> = (0..50).map(|i| i as f32 * 1.5).collect();
    client
        .write_multiple_f32(0, large_values.clone())
        .await
        .expect("Failed to write large batch of real variables");

    wait_for_operation().await;

    let read_large = client
        .read_multiple_f32(0, 50)
        .await
        .expect("Failed to read large batch of real variables");
    assert_eq!(read_large, large_values);
});

test_with_logging!(test_plural_real_variable_maximum_count, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test maximum count (118) - this tests the protocol limit
    #[allow(clippy::cast_precision_loss)]
    let max_values: Vec<f32> = (0..118).map(|i| i as f32 * 0.1).collect();
    client
        .write_multiple_f32(0, max_values.clone())
        .await
        .expect("Failed to write maximum count real variables");

    wait_for_operation().await;

    let read_max = client
        .read_multiple_f32(0, 118)
        .await
        .expect("Failed to read maximum count real variables");
    assert_eq!(read_max, max_values);
});

test_with_logging!(test_plural_real_variable_floating_point_precision, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test floating point precision with various values
    let precision_values = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        std::f32::consts::PI,
        std::f32::consts::E,
        1.0e-6,
        1.0e6,
        f32::MAX,
        f32::MIN_POSITIVE,
    ];

    client
        .write_multiple_f32(0, precision_values.clone())
        .await
        .expect("Failed to write precision test real variables");

    wait_for_operation().await;

    let read_precision = client
        .read_multiple_f32(0, u32::try_from(precision_values.len()).expect("Should fit in u32"))
        .await
        .expect("Failed to read precision test real variables");

    // Compare with epsilon tolerance for floating point values
    for (expected, actual) in precision_values.iter().zip(read_precision.iter()) {
        if expected.is_finite() && actual.is_finite() {
            assert!(
                (expected - actual).abs() < f32::EPSILON,
                "Precision mismatch: expected {expected}, got {actual}"
            );
        } else {
            assert_eq!(expected, actual, "Non-finite value mismatch");
        }
    }
});

test_with_logging!(test_multiple_character_variables_operations, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test writing multiple character type variables
    let values = vec!["Hello".to_string(), "World".to_string(), "Test1234".to_string()];

    client
        .write_multiple_strings(0, values.clone())
        .await
        .expect("Failed to write multiple character variables");

    wait_for_operation().await;

    // Read back and verify
    let read_values = client
        .read_multiple_strings(0, 3)
        .await
        .expect("Failed to read multiple character variables");
    assert_eq!(read_values, values);

    // Test boundary conditions (count = 1)
    let single_values = vec!["Test".to_string()];

    client
        .write_multiple_strings(10, single_values.clone())
        .await
        .expect("Failed to write single character variable");

    wait_for_operation().await;

    let read_single = client
        .read_multiple_strings(10, 1)
        .await
        .expect("Failed to read single character variable");
    assert_eq!(read_single, single_values);

    // Test maximum count (29)
    let max_values: Vec<String> = (0..29).map(|i| format!("Test{i:02}")).collect();

    client
        .write_multiple_strings(20, max_values.clone())
        .await
        .expect("Failed to write maximum count character variables");

    wait_for_operation().await;

    let read_max = client
        .read_multiple_strings(20, 29)
        .await
        .expect("Failed to read maximum count character variables");
    assert_eq!(read_max, max_values);

    // Test with various string patterns
    let pattern_values =
        vec!["ASCII_STRING".to_string(), "こんにちは".to_string(), "Binary123".to_string()];

    client
        .write_multiple_strings(50, pattern_values.clone())
        .await
        .expect("Failed to write pattern character variables");

    wait_for_operation().await;

    let read_patterns = client
        .read_multiple_strings(50, 3)
        .await
        .expect("Failed to read pattern character variables");
    assert_eq!(read_patterns, pattern_values);
});

test_with_logging!(test_multiple_character_variables_validation, {
    let _server =
        create_variable_test_server().await.expect("Failed to start variable test server");

    let client = create_test_client().await.expect("Failed to create client");

    // Test invalid count: 0
    let result = client.read_multiple_strings(0, 0).await;
    assert!(result.is_err());
    assert!(result.expect_err("Should be error").to_string().contains("Invalid count: 0"));

    // Test invalid count: > 29
    let result = client.read_multiple_strings(0, 30).await;
    assert!(result.is_err());
    assert!(result.expect_err("Should be error").to_string().contains("Invalid count: 30"));

    // Test invalid count for write: 0
    let result = client.write_multiple_strings(0, vec![]).await;
    assert!(result.is_err());
    assert!(result.expect_err("Should be error").to_string().contains("Invalid count: 0"));

    // Test invalid count for write: > 29
    let large_values: Vec<String> = (0..30).map(|i| format!("Test{i}")).collect();
    let result = client.write_multiple_strings(0, large_values).await;
    assert!(result.is_err());
    assert!(result.expect_err("Should be error").to_string().contains("Invalid count: 30"));

    // Test string too long when encoded
    let long_string = "This is a very long string that exceeds 16 bytes when encoded";
    let long_values = vec![long_string.to_string()];
    let result = client.write_multiple_strings(0, long_values).await;
    assert!(result.is_err());
    assert!(
        result.expect_err("Should be error").to_string().contains("exceeds 16 bytes when encoded")
    );
});
