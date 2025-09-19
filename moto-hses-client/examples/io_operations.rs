//! I/O operations example
//!
//! This example demonstrates how to read and write I/O data using the 0x78 command.

use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // Create client configuration
    let config = ClientConfig {
        host: "127.0.0.1".to_string(),
        port: moto_hses_proto::ROBOT_CONTROL_PORT,
        timeout: Duration::from_secs(5),
        retry_count: 3,
        retry_delay: Duration::from_millis(100),
        buffer_size: 8192,
        text_encoding: moto_hses_proto::TextEncoding::ShiftJis,
    };

    // Create client
    let client = HsesClient::new_with_config(config).await?;
    info!("Connected successfully!");

    // Example I/O operations
    info!("\n=== I/O Operations Example ===");

    // Read robot user input (I/O number 1-128)
    info!("Reading robot user input I/O #1...");
    match client.read_io(1).await {
        Ok(value) => info!("I/O #1 state: {}", if value { "ON" } else { "OFF" }),
        Err(e) => info!("Failed to read I/O #1: {e}"),
    }

    // Read robot user output (I/O number 1001-1128)
    info!("Reading robot user output I/O #1001...");
    match client.read_io(1001).await {
        Ok(value) => info!("I/O #1001 state: {}", if value { "ON" } else { "OFF" }),
        Err(e) => info!("Failed to read I/O #1001: {e}"),
    }

    // Write to robot user output (only network input signals are writable)
    info!("Writing to robot user output I/O #1001...");
    match client.write_io(1001, true).await {
        Ok(()) => info!("Successfully set I/O #1001 to ON"),
        Err(e) => info!("Failed to write I/O #1001: {e}"),
    }

    // Verify the write operation
    sleep(Duration::from_millis(100)).await;
    info!("Verifying I/O #1001 state...");
    match client.read_io(1001).await {
        Ok(value) => info!("I/O #1001 state after write: {}", if value { "ON" } else { "OFF" }),
        Err(e) => info!("Failed to read I/O #1001: {e}"),
    }

    // Additional I/O operations
    info!("\n=== Additional I/O Operations ===");
    match client.read_io(2).await {
        Ok(value) => info!("I/O #2 state: {}", if value { "ON" } else { "OFF" }),
        Err(e) => info!("Failed to read I/O #2: {e}"),
    }

    match client.write_io(1002, false).await {
        Ok(()) => info!("Successfully set I/O #1002 to OFF"),
        Err(e) => info!("Failed to write I/O #1002: {e}"),
    }

    // Test error handling
    info!("\n--- Error Handling Tests ---");

    // Test invalid I/O number
    match client.read_io(65535).await {
        Ok(value) => {
            info!("✗ Invalid I/O number succeeded unexpectedly: {value}");
        }
        Err(e) => {
            info!("✓ Invalid I/O number correctly failed: {e}");
        }
    }

    // Test invalid I/O number for write
    match client.write_io(65535, true).await {
        Ok(()) => {
            info!("✗ Invalid I/O number write succeeded unexpectedly");
        }
        Err(e) => {
            info!("✓ Invalid I/O number write correctly failed: {e}");
        }
    }

    info!("\nI/O operations completed.");

    Ok(())
}
