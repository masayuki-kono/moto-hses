//! File operations example for HSES client
//!
//! This example demonstrates file operations using the `HsesClient` API

use log::info;

use moto_hses_client::HsesClient;
use moto_hses_proto::FILE_CONTROL_PORT;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let (host, file_port) = match args.as_slice() {
        [_, host, file_port] => {
            // Format: [host] [file_port]
            let file_port: u16 =
                file_port.parse().map_err(|_| format!("Invalid file port: {file_port}"))?;

            (host.to_string(), file_port)
        }
        _ => {
            // Default: 127.0.0.1:FILE_CONTROL_PORT
            ("127.0.0.1".to_string(), FILE_CONTROL_PORT)
        }
    };

    info!("HSES Client File Operations Example");
    info!("File Control: {host}:{file_port}");

    // Create HsesClient for file operations
    let client = match HsesClient::new(&format!("{host}:{file_port}")).await {
        Ok(client) => {
            info!("✓ Successfully created HsesClient");
            client
        }
        Err(e) => {
            info!("✗ Failed to create HsesClient: {e}");
            return Ok(());
        }
    };

    // File operations demonstration
    info!("\n--- File Operations Test ---");

    // Step 1: Get file list
    info!("1. Getting file list...");
    let files = match client.read_file_list().await {
        Ok(files) => {
            info!("✓ File list retrieved successfully");
            for file in &files {
                info!("  - {file}");
            }
            files
        }
        Err(e) => {
            info!("✗ Failed to get file list: {e}");
            return Ok(());
        }
    };

    // Step 2: Process files or create test file
    if let Some(first_file) = files.first() {
        process_existing_file(&client, first_file).await?;
    } else {
        create_and_cleanup_test_file(&client).await?;
    }

    info!("\n--- File Operations Test Completed ---");

    Ok(())
}

async fn process_existing_file(
    client: &HsesClient,
    first_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n2. Getting content of file: {first_file}");

    let content = match client.receive_file(first_file).await {
        Ok(content) => {
            info!("✓ File content retrieved successfully");
            info!("  Content length: {} bytes", content.len());
            content
        }
        Err(e) => {
            info!("✗ Failed to get file content: {e}");
            return Ok(());
        }
    };

    let content_str = if let Ok(s) = String::from_utf8(content.clone()) {
        info!("  Original content: {s}");
        s
    } else {
        info!("✗ Failed to decode original content as UTF-8");
        return Ok(());
    };

    // Step 3: Modify content and create new file
    // Parse the JOB file and modify the NAME line
    let modified_content = modify_job_name(&content_str, &format!("MODIFIED_{first_file}"));
    info!("  Modified content: {modified_content}");
    let new_filename = format!("MODIFIED_{first_file}");

    info!("\n3. Creating modified file: {new_filename}");
    info!("  Modified content: {modified_content}");

    if let Err(e) = client.send_file(&new_filename, modified_content.as_bytes()).await {
        info!("✗ Failed to send modified file: {e}");
        return Ok(());
    }
    info!("✓ Modified file sent successfully");

    // Step 4: Verify file creation and content
    verify_file_creation_and_content(client, &new_filename, &modified_content).await?;

    // Step 5: Cleanup
    cleanup_file(client, &new_filename, "modified").await?;

    Ok(())
}

async fn verify_file_creation_and_content(
    client: &HsesClient,
    filename: &str,
    expected_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get updated file list
    info!("\n4. Getting updated file list...");
    let updated_files = match client.read_file_list().await {
        Ok(files) => {
            info!("✓ Updated file list retrieved successfully");
            for file in &files {
                info!("  - {file}");
            }
            files
        }
        Err(e) => {
            info!("✗ Failed to get updated file list: {e}");
            return Ok(());
        }
    };

    // Verify new file exists in list
    if !updated_files.contains(&filename.to_string()) {
        info!("✗ New file '{filename}' not found in updated list");
        return Ok(());
    }
    info!("✓ New file '{filename}' found in list");

    // Get content of new file and verify
    info!("\n5. Verifying new file content...");
    let received_content = match client.receive_file(filename).await {
        Ok(content) => {
            info!("✓ New file content retrieved successfully");
            content
        }
        Err(e) => {
            info!("✗ Failed to get new file content: {e}");
            return Ok(());
        }
    };

    let received_str = if let Ok(s) = String::from_utf8(received_content) {
        info!("  Received content: {s}");
        s
    } else {
        info!("✗ Failed to decode received content as UTF-8");
        return Ok(());
    };

    // Compare content
    if received_str == expected_content {
        info!("✓ Content verification successful!");
        info!("  Original modified: {expected_content}");
        info!("  Received content: {received_str}");
    } else {
        info!("✗ Content verification failed!");
        info!("  Expected: {expected_content}");
        info!("  Received: {received_str}");
    }

    Ok(())
}

async fn create_and_cleanup_test_file(
    client: &HsesClient,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n2. No files found, creating a test file...");
    let test_content = "This is a test file content";
    let test_filename = "TEST.JOB";

    info!("  Creating file: {test_filename}");
    info!("  Content: {test_content}");

    if let Err(e) = client.send_file(test_filename, test_content.as_bytes()).await {
        info!("✗ Failed to create test file: {e}");
        return Ok(());
    }
    info!("✓ Test file created successfully");

    // Verify the file was created
    info!("\n3. Verifying test file creation...");
    let updated_files = match client.read_file_list().await {
        Ok(files) => {
            info!("✓ Updated file list retrieved successfully");
            for file in &files {
                info!("  - {file}");
            }
            files
        }
        Err(e) => {
            info!("✗ Failed to get updated file list: {e}");
            return Ok(());
        }
    };

    if !updated_files.contains(&test_filename.to_string()) {
        info!("✗ Test file '{test_filename}' not found in list");
        return Ok(());
    }
    info!("✓ Test file '{test_filename}' found in list");

    // Cleanup
    cleanup_file(client, test_filename, "test").await?;

    Ok(())
}

async fn cleanup_file(
    client: &HsesClient,
    filename: &str,
    file_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n6. Cleaning up - Deleting {file_type} file: {filename}");

    if let Err(e) = client.delete_file(filename).await {
        info!("✗ Failed to delete {file_type} file: {e}");
        return Ok(());
    }
    info!("✓ {file_type} file deleted successfully");

    // Verify the file was deleted
    info!("\n7. Verifying cleanup...");
    let final_files = match client.read_file_list().await {
        Ok(files) => {
            info!("✓ Final file list retrieved successfully");
            for file in &files {
                info!("  - {file}");
            }
            files
        }
        Err(e) => {
            info!("✗ Failed to get final file list: {e}");
            return Ok(());
        }
    };

    if final_files.contains(&filename.to_string()) {
        info!("✗ {file_type} file '{filename}' still exists in list");
    } else {
        info!("✓ {file_type} file '{filename}' successfully removed from list");
    }

    Ok(())
}

/// Modify JOB file name in the content
fn modify_job_name(content: &str, new_name: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut modified_lines = Vec::new();

    for line in lines {
        if line.starts_with("//NAME ") {
            // Replace the NAME line with the new name
            modified_lines.push(format!("//NAME {new_name}"));
        } else {
            // Keep other lines unchanged
            modified_lines.push(line.to_string());
        }
    }

    // Join lines back together with proper line endings
    modified_lines.join("\r\n")
}
