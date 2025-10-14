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

    // Test multiple byte variable operations (0x302 command)
    info!("\n--- Multiple Byte Variable Operations (0x302) ---");

    // Write multiple byte variables (demonstrating efficiency gains)
    let values = vec![10, 20, 30, 40, 50, 60];
    match client.write_multiple_byte_variables(10, values.clone()).await {
        Ok(()) => {
            info!("✓ Wrote {} byte variables (B010-B015) in one operation", values.len());
            info!("  Values: {:?}", values);
        }
        Err(e) => {
            info!("✗ Failed to write multiple byte variables: {e}");
        }
    }

    // Read multiple byte variables back
    match client.read_multiple_byte_variables(10, 6).await {
        Ok(read_values) => {
            info!("✓ Read {} byte variables (B010-B015) in one operation", read_values.len());
            info!("  Values: {:?}", read_values);
            if read_values == values {
                info!("  ✓ Values match expected");
            } else {
                info!("  ✗ Values don't match expected");
            }
        }
        Err(e) => {
            info!("✗ Failed to read multiple byte variables: {e}");
        }
    }

    // Demonstrate efficiency: Compare single vs multiple operations
    info!("\n--- Performance Comparison: Single vs Multiple Operations ---");

    // Individual operations (less efficient)
    let start = std::time::Instant::now();
    for i in 0..6 {
        if let Err(e) = client.write_u8(20 + i, 100 + i as u8).await {
            info!("✗ Failed individual write to B{:03}: {e}", 20 + i);
        }
    }
    let individual_time = start.elapsed();
    info!("✓ Individual writes took: {:?}", individual_time);

    // Multiple operation (more efficient)
    let start = std::time::Instant::now();
    let batch_values: Vec<u8> = (100..106).collect();
    match client.write_multiple_byte_variables(30, batch_values).await {
        Ok(()) => {
            let batch_time = start.elapsed();
            info!("✓ Batch write took: {:?}", batch_time);
            if batch_time < individual_time {
                info!("  ✓ Batch operation was {} faster!", 
                      individual_time.as_nanos() / batch_time.as_nanos().max(1));
            }
        }
        Err(e) => {
            info!("✗ Failed batch write: {e}");
        }
    }

    // Verify both approaches produced same results
    match (client.read_multiple_byte_variables(20, 6).await, 
           client.read_multiple_byte_variables(30, 6).await) {
        (Ok(individual_results), Ok(batch_results)) => {
            if individual_results == batch_results {
                info!("✓ Both approaches produced identical results: {:?}", individual_results);
            } else {
                info!("✗ Results differ - Individual: {:?}, Batch: {:?}", 
                      individual_results, batch_results);
            }
        }
        (Err(e), _) => info!("✗ Failed to read individual results: {e}"),
        (_, Err(e)) => info!("✗ Failed to read batch results: {e}"),
    }

    // Demonstrate boundary operations
    info!("\n--- Boundary Operations ---");

    // Test maximum range (variables 95-99, just within limit)
    let boundary_values = vec![95, 96, 97, 98];
    match client.write_multiple_byte_variables(96, boundary_values.clone()).await {
        Ok(()) => {
            info!("✓ Successfully wrote to boundary variables B096-B099");
        }
        Err(e) => {
            info!("✗ Failed boundary write: {e}");
        }
    }

    match client.read_multiple_byte_variables(96, 4).await {
        Ok(read_boundary) => {
            info!("✓ Read boundary variables: {:?}", read_boundary);
        }
        Err(e) => {
            info!("✗ Failed boundary read: {e}");
        }
    }

    info!("\n--- Multiple Byte Variable Operations demonstrate:");
    info!("  • Efficient batch processing of byte variables");
    info!("  • Reduced network round-trips compared to individual operations");
    info!("  • Support for up to 474 variables per operation (limited by variable range 0-99)");
    info!("  • Count must be multiple of 2 as per HSES protocol specification");
    info!("  • Proper boundary handling and validation");

    info!("\n--- Variable Operations Example completed successfully ---");
    Ok(())
}
