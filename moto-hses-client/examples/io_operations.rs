//! I/O operations example
//!
//! This example demonstrates how to read and write I/O data using the 0x78 command.

use moto_hses_client::{ClientConfig, HsesClient};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let config = ClientConfig {
        host: "127.0.0.1".to_string(),
        port: 10040,
        timeout: Duration::from_secs(5),
        retry_count: 3,
        retry_delay: Duration::from_millis(100),
        buffer_size: 8192,
    };

    // Create client
    let client = HsesClient::new_with_config(config).await?;
    println!("Connected successfully!");

    // Example I/O operations
    println!("\n=== I/O Operations Example ===");

    // Read robot user input (I/O number 1-128)
    println!("Reading robot user input I/O #1...");
    match client.read_io(1).await {
        Ok(value) => println!("I/O #1 state: {}", if value { "ON" } else { "OFF" }),
        Err(e) => println!("Failed to read I/O #1: {}", e),
    }

    // Read robot user output (I/O number 1001-1128)
    println!("Reading robot user output I/O #1001...");
    match client.read_io(1001).await {
        Ok(value) => println!("I/O #1001 state: {}", if value { "ON" } else { "OFF" }),
        Err(e) => println!("Failed to read I/O #1001: {}", e),
    }

    // Write to robot user output (only network input signals are writable)
    println!("Writing to robot user output I/O #1001...");
    match client.write_io(1001, true).await {
        Ok(_) => println!("Successfully set I/O #1001 to ON"),
        Err(e) => println!("Failed to write I/O #1001: {}", e),
    }

    // Verify the write operation
    sleep(Duration::from_millis(100)).await;
    println!("Verifying I/O #1001 state...");
    match client.read_io(1001).await {
        Ok(value) => println!(
            "I/O #1001 state after write: {}",
            if value { "ON" } else { "OFF" }
        ),
        Err(e) => println!("Failed to read I/O #1001: {}", e),
    }

    // Additional I/O operations
    println!("\n=== Additional I/O Operations ===");
    match client.read_io(2).await {
        Ok(value) => println!("I/O #2 state: {}", if value { "ON" } else { "OFF" }),
        Err(e) => println!("Failed to read I/O #2: {}", e),
    }

    match client.write_io(1002, false).await {
        Ok(_) => println!("Successfully set I/O #1002 to OFF"),
        Err(e) => println!("Failed to write I/O #1002: {}", e),
    }

    println!("\nI/O operations completed.");

    Ok(())
}
