//! File operations example for HSES client
//! This example demonstrates file operations using the file control port

use moto_hses_proto::{HsesRequestMessage, HsesResponseMessage};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let (host, file_port) = match args.as_slice() {
        [_, host, file_port] => {
            // Format: [host] [file_port]
            let file_port: u16 = file_port
                .parse()
                .map_err(|_| format!("Invalid file port: {}", file_port))?;

            (host.to_string(), file_port)
        }
        _ => {
            // Default: 127.0.0.1:10041
            ("127.0.0.1".to_string(), 10041)
        }
    };

    println!("HSES Client File Operations Example");
    println!("File Control: {}:{}", host, file_port);

    // Create file control socket
    let file_socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(socket) => {
            println!("✓ Successfully created file control socket");
            socket
        }
        Err(e) => {
            eprintln!("✗ Failed to create file control socket: {}", e);
            return Ok(());
        }
    };

    let file_addr: SocketAddr = format!("{}:{}", host, file_port).parse()?;

    // File operations demonstration
    println!("\n--- File Operations Test ---");

    // Step 1: Get file list
    println!("1. Getting file list...");
    let files = match get_file_list(&file_socket, &file_addr).await {
        Ok(files) => {
            println!("✓ File list retrieved successfully");
            for file in files.iter() {
                println!("  - {}", file);
            }
            files
        }
        Err(e) => {
            eprintln!("✗ Failed to get file list: {}", e);
            return Ok(());
        }
    };

    // Step 2: Process files or create test file
    if let Some(first_file) = files.first() {
        process_existing_file(&file_socket, &file_addr, first_file).await?;
    } else {
        create_and_cleanup_test_file(&file_socket, &file_addr).await?;
    }

    println!("\n--- File Operations Test Completed ---");

    Ok(())
}

async fn process_existing_file(
    file_socket: &UdpSocket,
    file_addr: &SocketAddr,
    first_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Getting content of file: {}", first_file);

    let content = match receive_file(file_socket, file_addr, first_file).await {
        Ok(content) => {
            println!("✓ File content retrieved successfully");
            println!("  Content length: {} bytes", content.len());
            content
        }
        Err(e) => {
            eprintln!("✗ Failed to get file content: {}", e);
            return Ok(());
        }
    };

    let content_str = match String::from_utf8(content.clone()) {
        Ok(s) => {
            println!("  Original content: {}", s);
            s
        }
        Err(_) => {
            eprintln!("✗ Failed to decode original content as UTF-8");
            return Ok(());
        }
    };

    // Step 3: Modify content and create new file
    // Parse the JOB file and modify the NAME line
    let modified_content = match modify_job_name(&content_str, &format!("MODIFIED_{}", first_file))
    {
        Ok(modified) => {
            println!("  Modified content: {}", modified);
            modified
        }
        Err(e) => {
            eprintln!("✗ Failed to modify JOB file: {}", e);
            return Ok(());
        }
    };
    let new_filename = format!("MODIFIED_{}", first_file);

    println!("\n3. Creating modified file: {}", new_filename);
    println!("  Modified content: {}", modified_content);

    if let Err(e) = send_file(
        file_socket,
        file_addr,
        &new_filename,
        modified_content.as_bytes(),
    )
    .await
    {
        eprintln!("✗ Failed to send modified file: {}", e);
        return Ok(());
    }
    println!("✓ Modified file sent successfully");

    // Step 4: Verify file creation and content
    verify_file_creation_and_content(file_socket, file_addr, &new_filename, &modified_content)
        .await?;

    // Step 5: Cleanup
    cleanup_file(file_socket, file_addr, &new_filename, "modified").await?;

    Ok(())
}

async fn verify_file_creation_and_content(
    file_socket: &UdpSocket,
    file_addr: &SocketAddr,
    filename: &str,
    expected_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get updated file list
    println!("\n4. Getting updated file list...");
    let updated_files = match get_file_list(file_socket, file_addr).await {
        Ok(files) => {
            println!("✓ Updated file list retrieved successfully");
            for file in files.iter() {
                println!("  - {}", file);
            }
            files
        }
        Err(e) => {
            eprintln!("✗ Failed to get updated file list: {}", e);
            return Ok(());
        }
    };

    // Verify new file exists in list
    if !updated_files.contains(&filename.to_string()) {
        eprintln!("✗ New file '{}' not found in updated list", filename);
        return Ok(());
    }
    println!("✓ New file '{}' found in list", filename);

    // Get content of new file and verify
    println!("\n5. Verifying new file content...");
    let received_content = match receive_file(file_socket, file_addr, filename).await {
        Ok(content) => {
            println!("✓ New file content retrieved successfully");
            content
        }
        Err(e) => {
            eprintln!("✗ Failed to get new file content: {}", e);
            return Ok(());
        }
    };

    let received_str = match String::from_utf8(received_content.clone()) {
        Ok(s) => {
            println!("  Received content: {}", s);
            s
        }
        Err(_) => {
            eprintln!("✗ Failed to decode received content as UTF-8");
            return Ok(());
        }
    };

    // Compare content
    if received_str == expected_content {
        println!("✓ Content verification successful!");
        println!("  Original modified: {}", expected_content);
        println!("  Received content: {}", received_str);
    } else {
        eprintln!("✗ Content verification failed!");
        eprintln!("  Expected: {}", expected_content);
        eprintln!("  Received: {}", received_str);
    }

    Ok(())
}

async fn create_and_cleanup_test_file(
    file_socket: &UdpSocket,
    file_addr: &SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. No files found, creating a test file...");
    let test_content = "This is a test file content";
    let test_filename = "TEST.JOB";

    println!("  Creating file: {}", test_filename);
    println!("  Content: {}", test_content);

    if let Err(e) = send_file(
        file_socket,
        file_addr,
        test_filename,
        test_content.as_bytes(),
    )
    .await
    {
        eprintln!("✗ Failed to create test file: {}", e);
        return Ok(());
    }
    println!("✓ Test file created successfully");

    // Verify the file was created
    println!("\n3. Verifying test file creation...");
    let updated_files = match get_file_list(file_socket, file_addr).await {
        Ok(files) => {
            println!("✓ Updated file list retrieved successfully");
            for file in files.iter() {
                println!("  - {}", file);
            }
            files
        }
        Err(e) => {
            eprintln!("✗ Failed to get updated file list: {}", e);
            return Ok(());
        }
    };

    if !updated_files.contains(&test_filename.to_string()) {
        eprintln!("✗ Test file '{}' not found in list", test_filename);
        return Ok(());
    }
    println!("✓ Test file '{}' found in list", test_filename);

    // Cleanup
    cleanup_file(file_socket, file_addr, test_filename, "test").await?;

    Ok(())
}

async fn cleanup_file(
    file_socket: &UdpSocket,
    file_addr: &SocketAddr,
    filename: &str,
    file_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n6. Cleaning up - Deleting {} file: {}",
        file_type, filename
    );

    if let Err(e) = delete_file(file_socket, file_addr, filename).await {
        eprintln!("✗ Failed to delete {} file: {}", file_type, e);
        return Ok(());
    }
    println!("✓ {} file deleted successfully", file_type);

    // Verify the file was deleted
    println!("\n7. Verifying cleanup...");
    let final_files = match get_file_list(file_socket, file_addr).await {
        Ok(files) => {
            println!("✓ Final file list retrieved successfully");
            for file in files.iter() {
                println!("  - {}", file);
            }
            files
        }
        Err(e) => {
            eprintln!("✗ Failed to get final file list: {}", e);
            return Ok(());
        }
    };

    if !final_files.contains(&filename.to_string()) {
        println!(
            "✓ {} file '{}' successfully removed from list",
            file_type, filename
        );
    } else {
        eprintln!("✗ {} file '{}' still exists in list", file_type, filename);
    }

    Ok(())
}

/// Get file list from controller
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

    let response = send_message(socket, addr, &message).await?;

    // Parse file list from response
    let content = String::from_utf8_lossy(&response.payload);
    let files: Vec<String> = content
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

/// Send file to controller
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

    let _response = send_message(socket, addr, &message).await?;
    Ok(())
}

/// Receive file from controller
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

    let response = send_message(socket, addr, &message).await?;

    // Extract file content from response
    // Response format: filename\0content
    if let Some(null_pos) = response.payload.iter().position(|&b| b == 0) {
        let content = response.payload[null_pos + 1..].to_vec();
        Ok(content)
    } else {
        Ok(response.payload)
    }
}

/// Delete file from controller
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

    let _response = send_message(socket, addr, &message).await?;
    Ok(())
}

/// Send message and wait for response
async fn send_message(
    socket: &UdpSocket,
    addr: &SocketAddr,
    message: &HsesRequestMessage,
) -> Result<HsesResponseMessage, Box<dyn std::error::Error>> {
    let data = message.encode();

    // Send message
    socket.send_to(&data, addr).await?;

    // Wait for response
    let mut buf = vec![0u8; 2048];
    let (n, _) = socket.recv_from(&mut buf).await?;

    // Parse response
    let response = HsesResponseMessage::decode(&buf[..n])?;
    Ok(response)
}

/// Modify JOB file name in the content
fn modify_job_name(content: &str, new_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut modified_lines = Vec::new();

    for line in lines {
        if line.starts_with("//NAME ") {
            // Replace the NAME line with the new name
            modified_lines.push(format!("//NAME {}", new_name));
        } else {
            // Keep other lines unchanged
            modified_lines.push(line.to_string());
        }
    }

    // Join lines back together with proper line endings
    let modified_content = modified_lines.join("\r\n");
    Ok(modified_content)
}
