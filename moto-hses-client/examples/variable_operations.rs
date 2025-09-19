use log::info;
use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::ROBOT_CONTROL_PORT;
use std::time::Duration;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // Parse command line arguments
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

    info!("HSES Client Variable Operations Example");
    info!("Connecting to controller at: {host}:{robot_port}");

    // Create custom configuration
    let config = ClientConfig {
        host: host.to_string(),
        port: robot_port,
        timeout: Duration::from_millis(500),
        retry_count: 5,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
        text_encoding: moto_hses_proto::TextEncoding::ShiftJis,
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

    // Read byte variable (B variable)
    match client.read_u8(0).await {
        Ok(value) => {
            info!("✓ B000 = {value}");
        }
        Err(e) => {
            info!("✗ Failed to read B000: {e}");
        }
    }

    // Write variables
    info!("\n--- Variable Writing Operations ---");

    // Write 16-bit integer variable (I variable)
    match client.write_i16(1, 42).await {
        Ok(()) => {
            info!("✓ Wrote 42 to I001");
        }
        Err(e) => {
            info!("✗ Failed to write to I001: {e}");
        }
    }

    // Write 32-bit integer variable (D variable)
    match client.write_i32(2, 12345).await {
        Ok(()) => {
            info!("✓ Wrote 12345 to D002");
        }
        Err(e) => {
            info!("✗ Failed to write to D002: {e}");
        }
    }

    // Write float variable (R variable)
    match client.write_f32(3, std::f32::consts::PI).await {
        Ok(()) => {
            info!("✓ Wrote π to R003");
        }
        Err(e) => {
            info!("✗ Failed to write to R003: {e}");
        }
    }

    // Write byte variable (B variable)
    match client.write_u8(4, 255).await {
        Ok(()) => {
            info!("✓ Wrote 255 to B004");
        }
        Err(e) => {
            info!("✗ Failed to write to B004: {e}");
        }
    }

    // Verify written values
    info!("\n--- Verifying Written Values ---");

    match client.read_i16(1).await {
        Ok(value) => {
            info!("✓ I001 = {value} (expected: 42)");
        }
        Err(e) => {
            info!("✗ Failed to read I001: {e}");
        }
    }

    match client.read_i32(2).await {
        Ok(value) => {
            info!("✓ D002 = {value} (expected: 12345)");
        }
        Err(e) => {
            info!("✗ Failed to read D002: {e}");
        }
    }

    match client.read_f32(3).await {
        Ok(value) => {
            info!("✓ R003 = {value} (expected: 3.14159)");
        }
        Err(e) => {
            info!("✗ Failed to read R003: {e}");
        }
    }

    match client.read_u8(4).await {
        Ok(value) => {
            info!("✓ B004 = {value} (expected: 255)");
        }
        Err(e) => {
            info!("✗ Failed to read B004: {e}");
        }
    }

    // Test multiple variable operations
    info!("\n--- Multiple Variable Operations ---");

    // Read multiple variables of different types
    let variables_to_read =
        vec![(0, "I000"), (1, "I001"), (0, "D000"), (2, "D002"), (0, "R000"), (3, "R003")];

    for (index, var_name) in variables_to_read {
        if var_name.starts_with('I') {
            match client.read_i16(index).await {
                Ok(value) => info!("✓ {var_name} = {value}"),
                Err(e) => info!("✗ Failed to read {var_name}: {e}"),
            }
        } else if var_name.starts_with('D') {
            match client.read_i32(index).await {
                Ok(value) => info!("✓ {var_name} = {value}"),
                Err(e) => info!("✗ Failed to read {var_name}: {e}"),
            }
        } else if var_name.starts_with('R') {
            match client.read_f32(index).await {
                Ok(value) => info!("✓ {var_name} = {value}"),
                Err(e) => info!("✗ Failed to read {var_name}: {e}"),
            }
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

    // Test another string variable
    let test_string2 = b"Test String 123";
    match client.write_string(1, test_string2.to_vec()).await {
        Ok(()) => info!("✓ Wrote '{}' to S001", String::from_utf8_lossy(test_string2)),
        Err(e) => info!("✗ Failed to write to S001: {e}"),
    }

    match client.read_string(1).await {
        Ok(value) => info!("✓ S001 = '{}'", String::from_utf8_lossy(&value)),
        Err(e) => info!("✗ Failed to read S001: {e}"),
    }

    // Test error handling
    info!("\n--- Error Handling Tests ---");

    // Test invalid variable index
    match client.read_i16(255).await {
        Ok(value) => {
            info!("✗ Invalid variable index succeeded unexpectedly: {value}");
        }
        Err(e) => {
            info!("✓ Invalid variable index correctly failed: {e}");
        }
    }

    // Test invalid variable index for write
    match client.write_i16(255, 42).await {
        Ok(()) => {
            info!("✗ Invalid variable index write succeeded unexpectedly");
        }
        Err(e) => {
            info!("✓ Invalid variable index write correctly failed: {e}");
        }
    }

    // Test invalid string variable index
    match client.read_string(255).await {
        Ok(value) => {
            info!(
                "✗ Invalid string variable index succeeded unexpectedly: '{}'",
                String::from_utf8_lossy(&value)
            );
        }
        Err(e) => {
            info!("✓ Invalid string variable index correctly failed: {e}");
        }
    }

    info!("\n--- Variable Operations Example completed successfully ---");
    Ok(())
}
