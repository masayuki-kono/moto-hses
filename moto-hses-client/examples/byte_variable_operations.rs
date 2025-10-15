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

    // Single Byte Variable Operations (0x7A command)
    info!("\n--- Single Byte Variable Operations (0x7A) ---");

    // Read byte variable
    match client.read_u8(0).await {
        Ok(value) => {
            info!("✓ B000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read B000: {e}");
        }
    }

    // Write byte variable
    match client.write_u8(0, 255).await {
        Ok(()) => {
            info!("✓ Wrote 255 to B000");
        }
        Err(e) => {
            info!("✗ Failed to write to B000: {e}");
        }
    }

    // Verify written value
    match client.read_u8(0).await {
        Ok(value) => {
            info!("✓ B000 = {value} (expected: 255)");
        }
        Err(e) => {
            info!("✗ Failed to read B000: {e}");
        }
    }

    // Plural Byte Variable Operations (0x302 command)
    info!("\n--- Plural Byte Variable Operations (0x302) ---");

    // Read multiple byte variables
    match client.read_multiple_u8(0, 4).await {
        Ok(values) => {
            info!("✓ Read 4 byte variables starting from B000:");
            for (i, value) in values.iter().enumerate() {
                info!("  B{i:03} = {value}");
            }
        }
        Err(e) => {
            info!("✗ Failed to read multiple byte variables: {e}");
        }
    }

    // Write multiple byte variables
    let test_values = vec![100, 200, 50, 75];
    match client.write_multiple_u8(0, test_values.clone()).await {
        Ok(()) => {
            info!("✓ Wrote {} byte variables starting from B000:", test_values.len());
            for (i, value) in test_values.iter().enumerate() {
                info!("  B{i:03} = {value}");
            }
        }
        Err(e) => {
            info!("✗ Failed to write multiple byte variables: {e}");
        }
    }

    // Verify written values
    #[allow(clippy::expect_used)]
    match client
        .read_multiple_u8(0, u32::try_from(test_values.len()).expect("Count should fit in u32"))
        .await
    {
        Ok(values) => {
            info!("✓ Verification - Read back {} byte variables:", values.len());
            let mut all_match = true;
            for (i, (expected, actual)) in test_values.iter().zip(values.iter()).enumerate() {
                if expected == actual {
                    info!("  B{i:03} = {actual} ✓");
                } else {
                    info!("  B{i:03} = {actual} ✗ (expected: {expected})");
                    all_match = false;
                }
            }
            if all_match {
                info!("✓ All values match!");
            } else {
                info!("✗ Some values don't match!");
            }
        }
        Err(e) => {
            info!("✗ Failed to verify written values: {e}");
        }
    }

    info!("\n--- Byte Variable Operations Example completed successfully ---");
    Ok(())
}
