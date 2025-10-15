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

    // String Variable Operations (0x7E command)
    info!("\n--- String Variable Operations (0x7E) ---");

    // Read string variable
    match client.read_string(0).await {
        Ok(value) => info!("✓ S000 = '{}'", String::from_utf8_lossy(&value)),
        Err(e) => info!("✗ Failed to read S000: {e}"),
    }

    // Write string variable
    let test_string = b"Hello, Robot!";
    match client.write_string(0, test_string.to_vec()).await {
        Ok(()) => info!("✓ Wrote '{}' to S000", String::from_utf8_lossy(test_string)),
        Err(e) => info!("✗ Failed to write to S000: {e}"),
    }

    // Verify written string
    match client.read_string(0).await {
        Ok(value) => {
            let expected = String::from_utf8_lossy(test_string);
            let actual = String::from_utf8_lossy(&value);
            if value == test_string {
                info!("✓ S000 = '{actual}' (expected: '{expected}')");
            } else {
                info!("✗ S000 = '{actual}' (expected: '{expected}')");
            }
        }
        Err(e) => info!("✗ Failed to read S000: {e}"),
    }

    info!("\n--- String Variable Operations Example completed successfully ---");
    Ok(())
}
