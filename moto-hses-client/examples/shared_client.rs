//! Example demonstrating thread-safe `SharedHsesClient` usage
//!
//! This example shows how to use `SharedHsesClient` to safely access
//! the HSES client from multiple concurrent tasks.

use log::info;
use moto_hses_client::{ClientConfig, HsesClient, HsesClientOps, SharedHsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let (host, robot_port) = match args.as_slice() {
        [_, host, robot_port] => {
            let robot_port: u16 = robot_port
                .parse()
                .map_err(|e| format!("Invalid robot port: {robot_port} - {e}"))?;
            (host.to_string(), robot_port)
        }
        _ => ("127.0.0.1".to_string(), ROBOT_CONTROL_PORT),
    };

    // Create custom configuration
    let config = ClientConfig {
        host: host.to_string(),
        port: robot_port,
        timeout: Duration::from_millis(3000),
        retry_count: 0,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
        text_encoding: TextEncoding::ShiftJis,
    };

    // Create HsesClient and wrap it with SharedHsesClient for thread-safe access
    let client = HsesClient::new_with_config(config).await?;
    let shared_client = SharedHsesClient::new(client);

    info!("=== SharedHsesClient Usage Example ===\n");

    // Example 1: Basic usage (same as HsesClient)
    info!("1. Basic usage (single task):");
    let status = shared_client.read_status().await?;
    info!("  Status: Running={}, Servo={}", status.is_running(), status.is_servo_on());

    // Example 2: Concurrent access from multiple tasks
    info!("\n2. Concurrent access from multiple tasks:");

    // Clone the shared client for each task
    let client1 = shared_client.clone();
    let client2 = shared_client.clone();
    let client3 = shared_client.clone();

    // Spawn multiple concurrent tasks
    let handle1 = tokio::spawn(async move {
        info!("  Task 1: Reading status...");
        let result = client1.read_status().await;
        info!("  Task 1: Done");
        result
    });

    let handle2 = tokio::spawn(async move {
        info!("  Task 2: Reading position...");
        let result = client2.read_position(0).await;
        info!("  Task 2: Done");
        result
    });

    let handle3 = tokio::spawn(async move {
        info!("  Task 3: Reading status data1...");
        let result = client3.read_status_data1().await;
        info!("  Task 3: Done");
        result
    });

    // Wait for all tasks to complete
    let (result1, result2, result3) = tokio::try_join!(handle1, handle2, handle3)?;

    info!("\n3. Results from concurrent tasks:");
    match result1 {
        Ok(status) => info!("  Task 1 result: Running={}", status.is_running()),
        Err(e) => info!("  Task 1 error: {e}"),
    }
    match result2 {
        Ok(_position) => info!("  Task 2 result: Position data received"),
        Err(e) => info!("  Task 2 error: {e}"),
    }
    match result3 {
        Ok(data1) => info!("  Task 3 result: Play mode={}", data1.play),
        Err(e) => info!("  Task 3 error: {e}"),
    }

    // Example 3: Using the trait for abstraction
    info!("\n4. Using HsesClientOps trait for abstraction:");
    print_status(&shared_client).await?;

    info!("\n=== Example completed ===");

    Ok(())
}

/// Example function that accepts any type implementing `HsesClientOps`
async fn print_status(client: &impl HsesClientOps) -> Result<(), moto_hses_client::ClientError> {
    let status = client.read_status().await?;
    info!("  Status via trait: Alarm={}, Error={}", status.has_alarm(), status.has_error());
    Ok(())
}
