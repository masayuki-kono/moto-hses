//! File operations example for HSES client
//! This example demonstrates file operations using the HsesClient API

use moto_hses_client::HsesClient;
use moto_hses_proto::FILE_CONTROL_PORT;

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
            // Default: 127.0.0.1:FILE_CONTROL_PORT
            ("127.0.0.1".to_string(), FILE_CONTROL_PORT)
        }
    };

    println!("HSES Client File Operations Example");
    println!("File Control: {}:{}", host, file_port);

    // Create HsesClient for file operations
    let client = match HsesClient::new(&format!("{}:{}", host, file_port)).await {
        Ok(client) => {
            println!("✓ Successfully created HsesClient");
            client
        }
        Err(e) => {
            eprintln!("✗ Failed to create HsesClient: {}", e);
            return Ok(());
        }
    };

    // File operations demonstration
    println!("\n--- File Operations Test ---");

    // Step 1: Get file list
    println!("1. Getting file list...");
    let files = match client.read_file_list().await {
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
        process_existing_file(&client, first_file).await?;
    } else {
        create_and_cleanup_test_file(&client).await?;
    }

    println!("\n--- File Operations Test Completed ---");

    Ok(())
}

async fn process_existing_file(
    client: &HsesClient,
    first_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Getting content of file: {}", first_file);

    let content = match client.receive_file(first_file).await {
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

    if let Err(e) = client
        .send_file(&new_filename, modified_content.as_bytes())
        .await
    {
        eprintln!("✗ Failed to send modified file: {}", e);
        return Ok(());
    }
    println!("✓ Modified file sent successfully");

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
    println!("\n4. Getting updated file list...");
    let updated_files = match client.read_file_list().await {
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
    let received_content = match client.receive_file(filename).await {
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
    client: &HsesClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. No files found, creating a test file...");
    let test_content = "This is a test file content";
    let test_filename = "TEST.JOB";

    println!("  Creating file: {}", test_filename);
    println!("  Content: {}", test_content);

    if let Err(e) = client
        .send_file(test_filename, test_content.as_bytes())
        .await
    {
        eprintln!("✗ Failed to create test file: {}", e);
        return Ok(());
    }
    println!("✓ Test file created successfully");

    // Verify the file was created
    println!("\n3. Verifying test file creation...");
    let updated_files = match client.read_file_list().await {
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
    cleanup_file(client, test_filename, "test").await?;

    Ok(())
}

async fn cleanup_file(
    client: &HsesClient,
    filename: &str,
    file_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n6. Cleaning up - Deleting {} file: {}",
        file_type, filename
    );

    if let Err(e) = client.delete_file(filename).await {
        eprintln!("✗ Failed to delete {} file: {}", file_type, e);
        return Ok(());
    }
    println!("✓ {} file deleted successfully", file_type);

    // Verify the file was deleted
    println!("\n7. Verifying cleanup...");
    let final_files = match client.read_file_list().await {
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
