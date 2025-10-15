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

    // Double Variable Operations (0x7C command)
    info!("\n--- Double Variable Operations (0x7C) ---");

    // Read 32-bit integer variable
    match client.read_i32(0).await {
        Ok(value) => {
            info!("✓ D000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read D000: {e}");
        }
    }

    // Write 32-bit integer variable
    match client.write_i32(0, 305_419_896).await {
        Ok(()) => {
            info!("✓ Wrote 305_419_896 (0x12345678) to D000");
        }
        Err(e) => {
            info!("✗ Failed to write to D000: {e}");
        }
    }

    // Verify written value
    match client.read_i32(0).await {
        Ok(value) => {
            info!("✓ D000 = {value} (expected: 305_419_896)");
        }
        Err(e) => {
            info!("✗ Failed to read D000: {e}");
        }
    }

    // Plural Double Variable Operations (0x304 command)
    info!("\n--- Plural Double Variable Operations (0x304) ---");

    // Read multiple double precision integer variables
    match client.read_multiple_double_variables(0, 4).await {
        Ok(values) => {
            info!("✓ Read multiple D variables: {values:?}");
        }
        Err(e) => {
            info!("✗ Failed to read multiple D variables: {e}");
        }
    }

    // Write multiple double precision integer variables
    let values = vec![1_000_000, -2_000_000, 2_147_483_647, -2_147_483_648];
    match client.write_multiple_double_variables(0, values.clone()).await {
        Ok(()) => {
            info!("✓ Wrote multiple D variables: {values:?}");
        }
        Err(e) => {
            info!("✗ Failed to write multiple D variables: {e}");
        }
    }

    // Verify written values
    match client.read_multiple_double_variables(0, 4).await {
        Ok(read_values) => {
            info!("✓ Read back multiple D variables: {read_values:?}");
            if read_values == values {
                info!("✓ Values match expected values");
            } else {
                info!("✗ Values do not match expected values");
            }
        }
        Err(e) => {
            info!("✗ Failed to read back multiple D variables: {e}");
        }
    }

    info!("\n--- Double Variable Operations Example completed successfully ---");
    Ok(())
}
