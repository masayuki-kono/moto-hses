// Integration tests for file operations

use crate::common::mock_server_setup::MockServerManager;
use crate::test_with_logging;
use moto_hses_proto::{HsesRequestMessage, HsesResponseMessage, FILE_CONTROL_PORT};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

// Helper functions for file operations (based on legacy implementation)
async fn create_file_socket() -> Result<UdpSocket, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok(socket)
}

async fn send_file_message(
    socket: &UdpSocket,
    addr: &SocketAddr,
    message: &HsesRequestMessage,
) -> Result<HsesResponseMessage, Box<dyn std::error::Error>> {
    let data = message.encode();
    socket.send_to(&data, addr).await?;

    let mut buf = vec![0u8; 2048];
    let (n, _) = socket.recv_from(&mut buf).await?;

    let response = HsesResponseMessage::decode(&buf[..n])?;
    Ok(response)
}

async fn get_file_list(
    socket: &UdpSocket,
    addr: &SocketAddr,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let message = HsesRequestMessage::new(
        0x02,   // Division: File control
        0x00,   // ACK: Request
        0x01,   // Request ID
        0x00,   // Command
        0x00,   // Instance
        0x00,   // Attribute
        0x32,   // Service: Get file list
        vec![], // Empty payload
    );

    let response = send_file_message(socket, addr, &message).await?;

    // Parse file list from response
    let content = String::from_utf8_lossy(&response.payload);
    let files: Vec<String> = content
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

async fn send_file(
    socket: &UdpSocket,
    addr: &SocketAddr,
    filename: &str,
    content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut payload = filename.as_bytes().to_vec();
    payload.push(0); // Null terminator
    payload.extend_from_slice(content);

    let message = HsesRequestMessage::new(
        0x02, // Division: File control
        0x00, // ACK: Request
        0x01, // Request ID
        0x00, // Command
        0x00, // Instance
        0x00, // Attribute
        0x15, // Service: Send file
        payload,
    );

    let _response = send_file_message(socket, addr, &message).await?;
    Ok(())
}

async fn receive_file(
    socket: &UdpSocket,
    addr: &SocketAddr,
    filename: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut payload = filename.as_bytes().to_vec();
    payload.push(0); // Null terminator

    let message = HsesRequestMessage::new(
        0x02, // Division: File control
        0x00, // ACK: Request
        0x01, // Request ID
        0x00, // Command
        0x00, // Instance
        0x00, // Attribute
        0x16, // Service: Receive file
        payload,
    );

    let response = send_file_message(socket, addr, &message).await?;

    // Extract file content from response
    if let Some(null_pos) = response.payload.iter().position(|&b| b == 0) {
        let content = response.payload[null_pos + 1..].to_vec();
        Ok(content)
    } else {
        Ok(response.payload)
    }
}

async fn delete_file(
    socket: &UdpSocket,
    addr: &SocketAddr,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut payload = filename.as_bytes().to_vec();
    payload.push(0); // Null terminator

    let message = HsesRequestMessage::new(
        0x02, // Division: File control
        0x00, // ACK: Request
        0x01, // Request ID
        0x00, // Command
        0x00, // Instance
        0x00, // Attribute
        0x09, // Service: Delete file
        payload,
    );

    let _response = send_file_message(socket, addr, &message).await?;
    Ok(())
}

test_with_logging!(test_file_list_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create file control socket
    let file_socket = create_file_socket()
        .await
        .expect("Failed to create file socket");
    let file_addr: SocketAddr = format!("127.0.0.1:{}", FILE_CONTROL_PORT)
        .parse()
        .expect("Invalid address");

    // Test file list retrieval
    log::info!("Testing file list retrieval...");
    let files = get_file_list(&file_socket, &file_addr)
        .await
        .expect("Failed to get file list");

    log::info!("✓ File list retrieved successfully");
    log::info!("  Number of files: {}", files.len());
    for file in files.iter() {
        log::info!("  - {}", file);
    }

    // Verify we got a response (even if empty)
    // Note: files.len() is always >= 0, so we just verify we got a response
    log::info!("File list operation completed successfully");
});

test_with_logging!(test_file_send_receive_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create file control socket
    let file_socket = create_file_socket()
        .await
        .expect("Failed to create file socket");
    let file_addr: SocketAddr = format!("127.0.0.1:{}", FILE_CONTROL_PORT)
        .parse()
        .expect("Invalid address");

    // Test file send operation
    let test_filename = "TEST.JOB";
    let test_content = "//NAME TEST_JOB\r\n//TYPE JOB\r\n//END";

    log::info!("Testing file send operation...");
    log::info!("  Filename: {}", test_filename);
    log::info!("  Content: {}", test_content);

    send_file(
        &file_socket,
        &file_addr,
        test_filename,
        test_content.as_bytes(),
    )
    .await
    .expect("Failed to send file");

    log::info!("✓ File sent successfully");

    // Test file receive operation
    log::info!("Testing file receive operation...");
    let received_content = receive_file(&file_socket, &file_addr, test_filename)
        .await
        .expect("Failed to receive file");

    let received_str = String::from_utf8_lossy(&received_content);
    log::info!("✓ File received successfully");
    log::info!("  Received content: {}", received_str);

    // Verify content matches
    assert_eq!(
        received_str, test_content,
        "Received content should match sent content"
    );
});

test_with_logging!(test_file_delete_operations, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create file control socket
    let file_socket = create_file_socket()
        .await
        .expect("Failed to create file socket");
    let file_addr: SocketAddr = format!("127.0.0.1:{}", FILE_CONTROL_PORT)
        .parse()
        .expect("Invalid address");

    // First, send a test file
    let test_filename = "DELETE_TEST.JOB";
    let test_content = "//NAME DELETE_TEST\r\n//TYPE JOB\r\n//END";

    log::info!("Creating test file for deletion...");
    send_file(
        &file_socket,
        &file_addr,
        test_filename,
        test_content.as_bytes(),
    )
    .await
    .expect("Failed to send test file");

    log::info!("✓ Test file created");

    // Test file deletion
    log::info!("Testing file delete operation...");
    delete_file(&file_socket, &file_addr, test_filename)
        .await
        .expect("Failed to delete file");

    log::info!("✓ File deleted successfully");
});

test_with_logging!(test_file_operations_comprehensive, {
    let mut server = MockServerManager::new();
    server.start().await.expect("Failed to start mock server");

    // Create file control socket
    let file_socket = create_file_socket()
        .await
        .expect("Failed to create file socket");
    let file_addr: SocketAddr = format!("127.0.0.1:{}", FILE_CONTROL_PORT)
        .parse()
        .expect("Invalid address");

    // Comprehensive file operations test
    log::info!("Starting comprehensive file operations test...");

    // 1. Get initial file list
    log::info!("1. Getting initial file list...");
    let initial_files = get_file_list(&file_socket, &file_addr)
        .await
        .expect("Failed to get initial file list");
    log::info!("  Initial files count: {}", initial_files.len());

    // 2. Create a test file
    let test_filename = "COMPREHENSIVE_TEST.JOB";
    let test_content = "//NAME COMPREHENSIVE_TEST\r\n//TYPE JOB\r\n//END";

    log::info!("2. Creating test file: {}", test_filename);
    send_file(
        &file_socket,
        &file_addr,
        test_filename,
        test_content.as_bytes(),
    )
    .await
    .expect("Failed to create test file");
    log::info!("  ✓ Test file created");

    // 3. Verify file exists in list
    log::info!("3. Verifying file exists in updated list...");
    let updated_files = get_file_list(&file_socket, &file_addr)
        .await
        .expect("Failed to get updated file list");
    assert!(
        updated_files.contains(&test_filename.to_string()),
        "Test file should exist in file list"
    );
    log::info!("  ✓ File found in list");

    // 4. Retrieve and verify file content
    log::info!("4. Retrieving and verifying file content...");
    let received_content = receive_file(&file_socket, &file_addr, test_filename)
        .await
        .expect("Failed to receive file");

    let received_str = String::from_utf8_lossy(&received_content);
    assert_eq!(
        received_str, test_content,
        "Received content should match sent content"
    );
    log::info!("  ✓ File content verified");

    // 5. Delete the test file
    log::info!("5. Deleting test file...");
    delete_file(&file_socket, &file_addr, test_filename)
        .await
        .expect("Failed to delete test file");
    log::info!("  ✓ Test file deleted");

    // 6. Verify file is removed from list
    log::info!("6. Verifying file removal...");
    let final_files = get_file_list(&file_socket, &file_addr)
        .await
        .expect("Failed to get final file list");
    assert!(
        !final_files.contains(&test_filename.to_string()),
        "Test file should be removed from list"
    );
    log::info!("  ✓ File removal verified");

    log::info!("✓ Comprehensive file operations test completed successfully");
});
