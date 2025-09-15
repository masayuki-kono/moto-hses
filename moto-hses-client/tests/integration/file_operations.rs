// Integration tests for file operations

use crate::common::mock_server_setup::MockServerManager;
use crate::test_with_logging;
use moto_hses_client::HsesClient;
use moto_hses_proto::FILE_CONTROL_PORT;

// Tests using HsesClient API

test_with_logging!(test_file_list_initial_state, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create client for file operations
    let client = HsesClient::new(&format!("127.0.0.1:{}", FILE_CONTROL_PORT))
        .await
        .expect("Failed to create client");

    // Test file list retrieval and verify initial state
    let files = client.read_file_list().await.expect("Failed to get file list");

    // MockServer starts with TEST.JOB file
    assert_eq!(files.len(), 1, "MockServer should start with 1 file");
    assert!(files.contains(&"TEST.JOB".to_string()), "TEST.JOB should be in initial file list");

    log::info!("✓ Initial file list verified: {:?}", files);
});

test_with_logging!(test_file_send_receive_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create client for file operations
    let client = HsesClient::new(&format!("127.0.0.1:{}", FILE_CONTROL_PORT))
        .await
        .expect("Failed to create client");

    // Test file send operation
    let test_filename = "API_TEST.JOB";
    let test_content = "//NAME API_TEST\r\n//TYPE JOB\r\n//END";

    client.send_file(test_filename, test_content.as_bytes()).await.expect("Failed to send file");

    // Verify file was added to list
    let files_after_send =
        client.read_file_list().await.expect("Failed to get file list after send");
    assert!(
        files_after_send.contains(&test_filename.to_string()),
        "New file should be in file list after sending"
    );
    assert_eq!(files_after_send.len(), 2, "Should have 2 files after adding one");

    // Test file receive operation
    let received_content =
        client.receive_file(test_filename).await.expect("Failed to receive file");

    let received_str = String::from_utf8_lossy(&received_content);
    assert_eq!(received_str, test_content, "Received content should match sent content");

    log::info!("✓ File send/receive operations verified successfully");
});

test_with_logging!(test_file_delete_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create client for file operations
    let client = HsesClient::new(&format!("127.0.0.1:{}", FILE_CONTROL_PORT))
        .await
        .expect("Failed to create client");

    // Verify initial state
    let initial_files = client.read_file_list().await.expect("Failed to get initial file list");
    assert_eq!(initial_files.len(), 1, "Should start with 1 file");
    assert!(
        initial_files.contains(&"TEST.JOB".to_string()),
        "TEST.JOB should be present initially"
    );

    // Delete the initial TEST.JOB file
    client.delete_file("TEST.JOB").await.expect("Failed to delete TEST.JOB file");

    // Verify file was deleted
    let files_after_delete =
        client.read_file_list().await.expect("Failed to get file list after delete");
    assert_eq!(files_after_delete.len(), 0, "Should have 0 files after deleting TEST.JOB");
    assert!(
        !files_after_delete.contains(&"TEST.JOB".to_string()),
        "TEST.JOB should not be in file list after deletion"
    );

    log::info!("✓ File delete operations verified successfully");
});

test_with_logging!(test_file_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create client for file operations
    let client = HsesClient::new(&format!("127.0.0.1:{}", FILE_CONTROL_PORT))
        .await
        .expect("Failed to create client");

    // 1. Verify initial state (TEST.JOB should exist)
    let initial_files = client.read_file_list().await.expect("Failed to get initial file list");
    assert_eq!(initial_files.len(), 1, "Should start with 1 file");
    assert!(
        initial_files.contains(&"TEST.JOB".to_string()),
        "TEST.JOB should be present initially"
    );

    // 2. Create a new test file
    let test_filename = "COMPREHENSIVE_TEST.JOB";
    let test_content = "//NAME COMPREHENSIVE_TEST\r\n//TYPE JOB\r\n//END";

    client
        .send_file(test_filename, test_content.as_bytes())
        .await
        .expect("Failed to create test file");

    // 3. Verify file exists in list (should have 2 files now)
    let files_after_send =
        client.read_file_list().await.expect("Failed to get file list after send");
    assert_eq!(files_after_send.len(), 2, "Should have 2 files after adding one");
    assert!(
        files_after_send.contains(&test_filename.to_string()),
        "New file should exist in file list"
    );
    assert!(
        files_after_send.contains(&"TEST.JOB".to_string()),
        "Original TEST.JOB should still exist"
    );

    // 4. Retrieve and verify file content
    let received_content =
        client.receive_file(test_filename).await.expect("Failed to receive file");

    let received_str = String::from_utf8_lossy(&received_content);
    assert_eq!(received_str, test_content, "Received content should match sent content");

    // 5. Delete the new test file
    client.delete_file(test_filename).await.expect("Failed to delete test file");

    // 6. Verify file is removed from list (back to 1 file)
    let final_files = client.read_file_list().await.expect("Failed to get final file list");
    assert_eq!(final_files.len(), 1, "Should have 1 file after deletion");
    assert!(
        !final_files.contains(&test_filename.to_string()),
        "Deleted file should not be in list"
    );
    assert!(final_files.contains(&"TEST.JOB".to_string()), "Original TEST.JOB should still exist");

    log::info!("✓ Comprehensive file operations test completed successfully");
});
