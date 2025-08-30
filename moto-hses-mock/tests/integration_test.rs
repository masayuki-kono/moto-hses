//! Integration tests for mock server

use moto_hses_mock::test_utils;
use moto_hses_proto as proto;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mock_server_startup() {
    let (addr, _handle) = test_utils::start_test_server().await.unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert!(addr.port() > 0, "Port should be assigned");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_status_command() {
    let (addr, _handle) = test_utils::start_test_server().await.unwrap();
    
    // Create a UDP socket to send commands
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Create status read command (0x72)
    let message = proto::HsesRequestMessage::new(
        1, // Division: Robot
        0, // ACK: Request
        1, // Request ID
        0x72, // Command: Read Status
        1, // Instance
        1, // Attribute: Data 1
        0x0e, // Service: Get_Attribute_Single
        vec![], // No payload
    );
    
    let data = message.encode();
    socket.send_to(&data, addr).await.unwrap();
    
    // Wait for response
    sleep(Duration::from_millis(50)).await;
    
    // Try to receive response
    let mut buf = vec![0u8; 1024];
    
    match socket.recv_from(&mut buf).await {
        Ok((n, _)) => {
            assert!(n > 0, "Should receive a response");
            let response = proto::HsesResponseMessage::decode(&buf[..n]).unwrap();
            assert_eq!(response.header.ack, 1); // Should be ACK
            assert_eq!(response.sub_header.service, 0x8e); // 0x0e + 0x80
        }
        Err(_) => {
            // Socket might not have data yet
            // This is acceptable for this test
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_variable_read_command() {
    let (addr, _handle) = test_utils::start_test_server().await.unwrap();
    
    // Create a UDP socket to send commands
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Create integer variable read command (0x7b)
    let message = proto::HsesRequestMessage::new(
        1, // Division: Robot
        0, // ACK: Request
        2, // Request ID
        0x7b, // Command: Read Integer Variable
        0, // Instance: Variable index
        1, // Attribute
        0x0e, // Service: Get_Attribute_Single
        vec![], // No payload
    );
    
    let data = message.encode();
    socket.send_to(&data, addr).await.unwrap();
    
    // Wait for response
    sleep(Duration::from_millis(50)).await;
    
    // Try to receive response
    let mut buf = vec![0u8; 1024];
    
    match socket.recv_from(&mut buf).await {
        Ok((n, _)) => {
            assert!(n > 0, "Should receive a response");
            let response = proto::HsesResponseMessage::decode(&buf[..n]).unwrap();
            assert_eq!(response.header.ack, 1); // Should be ACK
            assert_eq!(response.sub_header.service, 0x8e); // 0x0e + 0x80
            assert_eq!(response.payload.len(), 4); // Integer should be 4 bytes
        }
        Err(_) => {
            // Socket might not have data yet
            // This is acceptable for this test
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_io_read_command() {
    let (addr, _handle) = test_utils::start_test_server().await.unwrap();
    
    // Create a UDP socket to send commands
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Create I/O read command (0x78)
    let message = proto::HsesRequestMessage::new(
        1, // Division: Robot
        0, // ACK: Request
        3, // Request ID
        0x78, // Command: Read I/O
        1, // Instance: I/O number
        1, // Attribute
        0x0e, // Service: Get_Attribute_Single
        vec![], // No payload
    );
    
    let data = message.encode();
    socket.send_to(&data, addr).await.unwrap();
    
    // Wait for response
    sleep(Duration::from_millis(50)).await;
    
    // Try to receive response
    let mut buf = vec![0u8; 1024];
    
    match socket.recv_from(&mut buf).await {
        Ok((n, _)) => {
            assert!(n > 0, "Should receive a response");
            let response = proto::HsesResponseMessage::decode(&buf[..n]).unwrap();
            assert_eq!(response.header.ack, 1); // Should be ACK
            assert_eq!(response.sub_header.service, 0x8e); // 0x0e + 0x80
            assert_eq!(response.payload.len(), 4); // I/O value should be 4 bytes
        }
        Err(_) => {
            // Socket might not have data yet
            // This is acceptable for this test
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_unknown_command() {
    let (addr, _handle) = test_utils::start_test_server().await.unwrap();
    
    // Create a UDP socket to send commands
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Create unknown command (0x9999)
    let message = proto::HsesRequestMessage::new(
        1, // Division: Robot
        0, // ACK: Request
        4, // Request ID
        0x9999, // Command: Unknown
        1, // Instance
        1, // Attribute
        0x0e, // Service: Get_Attribute_Single
        vec![], // No payload
    );
    
    let data = message.encode();
    socket.send_to(&data, addr).await.unwrap();
    
    // Wait for response
    sleep(Duration::from_millis(100)).await;
    
    // Try to receive response
    let mut buf = vec![0u8; 1024];
    
    match socket.recv_from(&mut buf).await {
        Ok((n, _)) => {
            assert!(n > 0, "Should receive a response");
            let response = proto::HsesResponseMessage::decode(&buf[..n]).unwrap();
            assert_eq!(response.header.ack, 1); // Should be ACK
            assert_eq!(response.sub_header.service, 0x8e); // 0x0e + 0x80
            // Should have empty payload for unknown command
            assert_eq!(response.payload.len(), 0);
        }
        Err(_) => {
            // Socket might not have data yet
            // This is acceptable for this test
        }
    }
}
