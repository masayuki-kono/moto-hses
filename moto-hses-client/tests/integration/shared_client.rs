#![allow(clippy::expect_used)]
// Integration tests for SharedHsesClient (thread-safe wrapper)

use crate::common::{
    mock_server_setup::create_status_test_server, test_utils::create_shared_test_client,
};
use crate::test_with_logging;
use moto_hses_client::HsesClientOps;

test_with_logging!(test_shared_client_concurrent_access, {
    let _server = create_status_test_server().await.expect("Failed to start status test server");

    let shared_client = create_shared_test_client().await.expect("Failed to create shared client");

    // Clone for multiple tasks
    let client1 = shared_client.clone();
    let client2 = shared_client.clone();
    let client3 = shared_client.clone();

    // Spawn concurrent tasks
    let handle1 = tokio::spawn(async move { client1.read_status().await });
    let handle2 = tokio::spawn(async move { client2.read_status_data1().await });
    let handle3 = tokio::spawn(async move { client3.read_status_data2().await });

    // Wait for all tasks
    let (result1, result2, result3) =
        tokio::try_join!(handle1, handle2, handle3).expect("Failed to join tasks");

    // All should succeed
    let status = result1.expect("Task 1 failed");
    let data1 = result2.expect("Task 2 failed");
    let data2 = result3.expect("Task 3 failed");

    // Verify results are consistent
    assert_eq!(status.data1, data1);
    assert_eq!(status.data2, data2);
});
