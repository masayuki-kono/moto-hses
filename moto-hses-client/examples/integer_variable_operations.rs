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

    // Single Integer Variable Operations (0x7B command)
    info!("\n--- Single Integer Variable Operations (0x7B) ---");

    // Read 16-bit integer variable
    match client.read_i16(0).await {
        Ok(value) => {
            info!("✓ I000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read I000: {e}");
        }
    }

    // Write 16-bit integer variable
    match client.write_i16(0, 4660).await {
        Ok(()) => {
            info!("✓ Wrote 4660 (0x1234) to I000");
        }
        Err(e) => {
            info!("✗ Failed to write to I000: {e}");
        }
    }

    // Verify written value
    match client.read_i16(0).await {
        Ok(value) => {
            info!("✓ I000 = {value} (expected: 4660)");
        }
        Err(e) => {
            info!("✗ Failed to read I000: {e}");
        }
    }

    // Plural Integer Variable Operations (0x303 command)
    info!("\n--- Plural Integer Variable Operations (0x303) ---");

    // Read multiple integer variables
    match client.read_multiple_integer_variables(0, 4).await {
        Ok(values) => {
            info!("✓ Read 4 integer variables starting from I000:");
            for (i, value) in values.iter().enumerate() {
                info!("  I{i:03} = {value}");
            }
        }
        Err(e) => {
            info!("✗ Failed to read multiple integer variables: {e}");
        }
    }

    // Write multiple integer variables
    let test_int_values = vec![100, -200, 300, -400];
    match client.write_multiple_integer_variables(0, test_int_values.clone()).await {
        Ok(()) => {
            info!("✓ Wrote {} integer variables starting from I000:", test_int_values.len());
            for (i, value) in test_int_values.iter().enumerate() {
                info!("  I{i:03} = {value}");
            }
        }
        Err(e) => {
            info!("✗ Failed to write multiple integer variables: {e}");
        }
    }

    // Verify written values
    #[allow(clippy::expect_used)]
    match client
        .read_multiple_integer_variables(
            0,
            u32::try_from(test_int_values.len()).expect("Count should fit in u32"),
        )
        .await
    {
        Ok(values) => {
            info!("✓ Verification - Read back {} integer variables:", values.len());
            let mut all_match = true;
            for (i, (expected, actual)) in test_int_values.iter().zip(values.iter()).enumerate() {
                if expected == actual {
                    info!("  I{i:03} = {actual} ✓");
                } else {
                    info!("  I{i:03} = {actual} ✗ (expected: {expected})");
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

    info!("\n--- Integer Variable Operations Example completed successfully ---");
    Ok(())
}
