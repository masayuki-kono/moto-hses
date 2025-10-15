use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
use std::time::Duration;

#[tokio::main]
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

    // Real Variable Operations (0x7D command)
    info!("\n--- Real Variable Operations (0x7D) ---");

    // Read float variable
    match client.read_f32(0).await {
        Ok(value) => {
            info!("✓ R000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read R000: {e}");
        }
    }

    // Write float variable
    match client.write_f32(0, std::f32::consts::PI).await {
        Ok(()) => {
            info!("✓ Wrote π to R000");
        }
        Err(e) => {
            info!("✗ Failed to write to R000: {e}");
        }
    }

    // Verify written value
    match client.read_f32(0).await {
        Ok(value) => {
            info!("✓ R000 = {value} (expected: 3.14159)");
        }
        Err(e) => {
            info!("✗ Failed to read R000: {e}");
        }
    }

    info!("\n--- Real Variable Operations Example completed successfully ---");
    Ok(())
}
