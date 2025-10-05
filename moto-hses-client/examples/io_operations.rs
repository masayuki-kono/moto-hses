//! I/O operations example
//!
//! This example demonstrates how to read and write I/O data using the 0x78 command.

use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let (host, robot_port) = match args.as_slice() {
        [_, host, robot_port] => {
            // Format: [host] [robot_port]
            let robot_port: u16 =
                robot_port.parse().map_err(|_| format!("Invalid robot port: {robot_port}"))?;

            (host.to_string(), robot_port)
        }
        _ => {
            // Default: 127.0.0.1:DEFAULT_PORT
            ("127.0.0.1".to_string(), ROBOT_CONTROL_PORT)
        }
    };

    // Create custom configuration
    let config = ClientConfig {
        host: host.to_string(),
        port: robot_port,
        timeout: Duration::from_millis(500),
        retry_count: 5,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
        text_encoding: TextEncoding::ShiftJis,
    };

    // Connect to the controller
    let client = match HsesClient::new_with_config(config).await {
        Ok(client) => {
            info!("✓ Successfully connected to controller");
            client
        }
        Err(e) => {
            info!("✗ Failed to connect: {e}");
            return Ok(());
        }
    };

    info!("=== 0x78 Command (I/O Operations) ===\n");

    // Read robot system input (I/O number 4001-4256)
    info!("Reading robot system input I/O #4004...");
    match client.read_io(4004).await {
        Ok(value) => info!("I/O #4004: 0b{value:08b}"),
        Err(e) => info!("Failed to read I/O #4004: {e}"),
    }

    // Read robot user output (I/O number 5001-5512)
    info!("Reading robot user output I/O #5032...");
    match client.read_io(5032).await {
        Ok(value) => info!("I/O #5032: 0b{value:08b}"),
        Err(e) => info!("Failed to read I/O #5032: {e}"),
    }

    // Read robot control status signal (I/O number 8001-8512)
    info!("Reading robot control status signal I/O #8002...");
    match client.read_io(8002).await {
        Ok(value) => info!("I/O #8002: 0b{value:08b}"),
        Err(e) => info!("Failed to read I/O #8002: {e}"),
    }

    // Read pseudo input (I/O number 8701-8720)
    info!("Reading pseudo input I/O #8701...");
    match client.read_io(8701).await {
        Ok(value) => info!("I/O #8701: 0b{value:08b}"),
        Err(e) => info!("Failed to read I/O #8701: {e}"),
    }

    // Write to network input (I/O number 2701-2956 only network input signals are writable)
    info!("Writing to network input I/O #2701...");
    match client.write_io(2701, 0b0100_0001).await {
        Ok(()) => info!("Successfully set I/O #2701 to 0b01000001"),
        Err(e) => info!("Failed to write I/O #2701: {e}"),
    }

    // Verify the write operation
    sleep(Duration::from_millis(100)).await;
    info!("Verifying I/O #2701 state...");
    match client.read_io(2701).await {
        Ok(value) => info!("I/O #2701 after write: 0b{value:08b}"),
        Err(e) => info!("Failed to read I/O #2701: {e}"),
    }

    info!("\nI/O operations completed.");

    Ok(())
}
