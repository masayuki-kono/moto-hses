use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
use std::time::Duration;

#[tokio::main]
#[allow(clippy::too_many_lines, clippy::similar_names)]
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
        Ok(value) => info!("✓ S000 = '{value}'"),
        Err(e) => info!("✗ Failed to read S000: {e}"),
    }

    // Write string variable
    let test_string = "Hello, Robot!";
    match client.write_string(0, test_string.to_string()).await {
        Ok(()) => info!("✓ Wrote '{test_string}' to S000"),
        Err(e) => info!("✗ Failed to write to S000: {e}"),
    }

    // Verify written string
    match client.read_string(0).await {
        Ok(value) => {
            if value == test_string {
                info!("✓ S000 = '{value}' (expected: '{test_string}')");
            } else {
                info!("✗ S000 = '{value}' (expected: '{test_string}')");
            }
        }
        Err(e) => info!("✗ Failed to read S000: {e}"),
    }

    // Multiple Character Variable Operations (0x306 command)
    info!("\n--- Multiple Character Variable Operations (0x306) ---");

    // Write multiple character type variables
    let character_values = vec!["Hello".to_string(), "World".to_string(), "Test1234".to_string()];

    match client.write_multiple_strings(0, character_values.clone()).await {
        Ok(()) => info!("✓ Wrote {} character variables to S000-S002", character_values.len()),
        Err(e) => info!("✗ Failed to write multiple character variables: {e}"),
    }

    // Read multiple character type variables and verify
    match client.read_multiple_strings(0, 3).await {
        Ok(read_values) => {
            info!("✓ Read {} character variables from S000-S002:", read_values.len());
            for (i, value) in read_values.iter().enumerate() {
                info!("  S{i:03} = '{value}'");
            }

            // Verify that read values match written values
            if read_values == character_values {
                info!("✓ Verification successful: Read values match written values");
            } else {
                info!("✗ Verification failed: Read values do not match written values");
                for (i, (written, read)) in
                    character_values.iter().zip(read_values.iter()).enumerate()
                {
                    if written != read {
                        info!("  S{i:03}: written='{written}', read='{read}'");
                    }
                }
            }
        }
        Err(e) => info!("✗ Failed to read multiple character variables: {e}"),
    }

    info!("\n--- String Variable Operations Example completed successfully ---");
    Ok(())
}
