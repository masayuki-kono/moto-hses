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

    // Read variables
    info!("\n--- Variable Reading Operations ---");

    // Read byte variable (B variable)
    match client.read_u8(0).await {
        Ok(value) => {
            info!("✓ B000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read B000: {e}");
        }
    }

    // Read 16-bit integer variable (I variable)
    match client.read_i16(0).await {
        Ok(value) => {
            info!("✓ I000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read I000: {e}");
        }
    }

    // Read 32-bit integer variable (D variable)
    match client.read_i32(0).await {
        Ok(value) => {
            info!("✓ D000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read D000: {e}");
        }
    }

    // Read float variable (R variable)
    match client.read_f32(0).await {
        Ok(value) => {
            info!("✓ R000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read R000: {e}");
        }
    }

    // Write variables
    info!("\n--- Variable Writing Operations ---");

    // Write byte variable (B variable)
    match client.write_u8(0, 255).await {
        Ok(()) => {
            info!("✓ Wrote 255 to B000");
        }
        Err(e) => {
            info!("✗ Failed to write to B000: {e}");
        }
    }

    // Write 16-bit integer variable (I variable)
    match client.write_i16(0, 4660).await {
        Ok(()) => {
            info!("✓ Wrote 4660 (0x1234) to I000");
        }
        Err(e) => {
            info!("✗ Failed to write to I000: {e}");
        }
    }

    // Write 32-bit integer variable (D variable)
    match client.write_i32(0, 305_419_896).await {
        Ok(()) => {
            info!("✓ Wrote 305_419_896 (0x12345678) to D000");
        }
        Err(e) => {
            info!("✗ Failed to write to D000: {e}");
        }
    }

    // Write float variable (R variable)
    match client.write_f32(0, std::f32::consts::PI).await {
        Ok(()) => {
            info!("✓ Wrote π to R000");
        }
        Err(e) => {
            info!("✗ Failed to write to R000: {e}");
        }
    }

    // Verify written values
    info!("\n--- Verifying Written Values ---");

    match client.read_u8(0).await {
        Ok(value) => {
            info!("✓ B000 = {value} (expected: 255)");
        }
        Err(e) => {
            info!("✗ Failed to read B000: {e}");
        }
    }

    match client.read_i16(0).await {
        Ok(value) => {
            info!("✓ I000 = {value} (expected: 4660)");
        }
        Err(e) => {
            info!("✗ Failed to read I000: {e}");
        }
    }

    match client.read_i32(0).await {
        Ok(value) => {
            info!("✓ D000 = {value} (expected: 305_419_896)");
        }
        Err(e) => {
            info!("✗ Failed to read D000: {e}");
        }
    }

    match client.read_f32(0).await {
        Ok(value) => {
            info!("✓ R000 = {value} (expected: 3.14159)");
        }
        Err(e) => {
            info!("✗ Failed to read R003: {e}");
        }
    }

    // Test string variables (0x7E command)
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

    // Multiple byte variable operations (0x302 command)
    info!("\n=== Multiple Byte Variable Operations ===");

    // Write multiple byte variables
    let byte_values = vec![10, 20, 30, 40, 50, 60];
    info!("Writing multiple byte variables B40-B45: {byte_values:?}");
    client.write_multiple_byte_variables(40, byte_values.clone()).await?;

    // Read back multiple byte variables
    let read_byte_values = client.read_multiple_byte_variables(40, 6).await?;
    info!("Read back multiple byte variables B40-B45: {read_byte_values:?}");

    // Verify the values match
    assert_eq!(byte_values, read_byte_values, "Multiple byte variable values should match");

    // Demonstrate efficiency with larger batch
    let large_batch: Vec<u8> = (0..20).map(|i| (i * 10) as u8).collect();
    info!("Writing large batch of byte variables B50-B69 (count=20): First few values: {:?}...", &large_batch[..5]);
    client.write_multiple_byte_variables(50, large_batch.clone()).await?;

    let read_large_batch = client.read_multiple_byte_variables(50, 20).await?;
    info!("Read back large batch: First few values: {:?}...", &read_large_batch[..5]);
    assert_eq!(large_batch, read_large_batch, "Large batch values should match");

    info!("Multiple byte variable operations completed successfully!");

    info!("\n--- Variable Operations Example completed successfully ---");
    Ok(())
}
