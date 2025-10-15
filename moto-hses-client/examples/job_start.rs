use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
use std::time::Duration;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let (host, robot_port) = match args.as_slice() {
        [_, host, robot_port] => {
            // Format: [host] [robot_port]
            let robot_port: u16 = robot_port
                .parse()
                .map_err(|e| format!("Invalid robot port: {robot_port} - {e}"))?;

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
        timeout: Duration::from_millis(3000),
        retry_count: 0,
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

    // Read initial status
    info!("Reading initial status...");
    let initial_status = match client.read_status_data1().await {
        Ok(status) => {
            info!("✓ Initial status read successfully");
            info!("  - Running: {}", status.running);
            info!("  - Step: {}", status.step);
            info!("  - One cycle: {}", status.one_cycle);
            info!("  - Continuous: {}", status.continuous);
            status
        }
        Err(e) => {
            info!("✗ Failed to read initial status: {e}");
            return Ok(());
        }
    };

    // Start job execution
    info!("Starting job execution...");
    match client.start_job().await {
        Ok(()) => {
            info!("✓ Job start command sent successfully");
        }
        Err(e) => {
            info!("✗ Failed to start job: {e}");
            return Ok(());
        }
    }

    // Wait a moment for the command to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Read status after job start
    info!("Reading status after job start...");
    match client.read_status_data1().await {
        Ok(status) => {
            info!("✓ Status after job start read successfully");
            info!("  - Running: {} (was: {})", status.running, initial_status.running);
            info!("  - Step: {}", status.step);
            info!("  - One cycle: {}", status.one_cycle);
            info!("  - Continuous: {}", status.continuous);

            if status.running {
                info!("✓ Job is now running successfully");
            } else {
                info!("⚠ Job start command was sent but robot is not running");
            }
        }
        Err(e) => {
            info!("✗ Failed to read status after job start: {e}");
        }
    }

    Ok(())
}
