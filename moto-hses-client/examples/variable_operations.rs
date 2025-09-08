use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::ROBOT_CONTROL_PORT;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let (host, robot_port) = match args.as_slice() {
        [_, host, robot_port] => {
            // Format: [host] [robot_port]
            let robot_port: u16 = robot_port
                .parse()
                .map_err(|_| format!("Invalid robot port: {}", robot_port))?;

            (host.to_string(), robot_port)
        }
        _ => {
            // Default: 127.0.0.1:DEFAULT_PORT
            ("127.0.0.1".to_string(), ROBOT_CONTROL_PORT)
        }
    };

    println!("HSES Client Variable Operations Example");
    println!("Connecting to controller at: {}:{}", host, robot_port);

    // Create custom configuration
    let config = ClientConfig {
        host: host.to_string(),
        port: robot_port,
        timeout: Duration::from_millis(500),
        retry_count: 5,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
    };

    // Connect to the controller
    let client = match HsesClient::new_with_config(config).await {
        Ok(client) => {
            println!("✓ Successfully connected to controller");
            client
        }
        Err(e) => {
            eprintln!("✗ Failed to connect: {}", e);
            return Ok(());
        }
    };

    // Read variables
    println!("\n--- Variable Reading Operations ---");

    // Read 16-bit integer variable (I variable)
    match client.read_i16(0).await {
        Ok(value) => {
            println!("✓ I000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read I000: {}", e);
        }
    }

    // Read 32-bit integer variable (D variable)
    match client.read_i32(0).await {
        Ok(value) => {
            println!("✓ D000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read D000: {}", e);
        }
    }

    // Read float variable (R variable)
    match client.read_f32(0).await {
        Ok(value) => {
            println!("✓ R000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read R000: {}", e);
        }
    }

    // Read byte variable (B variable)
    match client.read_u8(0).await {
        Ok(value) => {
            println!("✓ B000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read B000: {}", e);
        }
    }

    // Write variables
    println!("\n--- Variable Writing Operations ---");

    // Write 16-bit integer variable (I variable)
    match client.write_i16(1, 42).await {
        Ok(()) => {
            println!("✓ Wrote 42 to I001");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to I001: {}", e);
        }
    }

    // Write 32-bit integer variable (D variable)
    match client.write_i32(2, 12345).await {
        Ok(()) => {
            println!("✓ Wrote 12345 to D002");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to D002: {}", e);
        }
    }

    // Write float variable (R variable)
    match client.write_f32(3, std::f32::consts::PI).await {
        Ok(()) => {
            println!("✓ Wrote π to R003");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to R003: {}", e);
        }
    }

    // Write byte variable (B variable)
    match client.write_u8(4, 255).await {
        Ok(()) => {
            println!("✓ Wrote 255 to B004");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to B004: {}", e);
        }
    }

    // Verify written values
    println!("\n--- Verifying Written Values ---");

    match client.read_i16(1).await {
        Ok(value) => {
            println!("✓ I001 = {} (expected: 42)", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read I001: {}", e);
        }
    }

    match client.read_i32(2).await {
        Ok(value) => {
            println!("✓ D002 = {} (expected: 12345)", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read D002: {}", e);
        }
    }

    match client.read_f32(3).await {
        Ok(value) => {
            println!("✓ R003 = {} (expected: 3.14159)", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read R003: {}", e);
        }
    }

    match client.read_u8(4).await {
        Ok(value) => {
            println!("✓ B004 = {} (expected: 255)", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read B004: {}", e);
        }
    }

    // Test multiple variable operations
    println!("\n--- Multiple Variable Operations ---");

    // Read multiple variables of different types
    let variables_to_read = vec![
        (0, "I000"),
        (1, "I001"),
        (0, "D000"),
        (2, "D002"),
        (0, "R000"),
        (3, "R003"),
    ];

    for (index, var_name) in variables_to_read {
        if var_name.starts_with("I") {
            match client.read_i16(index).await {
                Ok(value) => println!("✓ {} = {}", var_name, value),
                Err(e) => eprintln!("✗ Failed to read {}: {}", var_name, e),
            }
        } else if var_name.starts_with("D") {
            match client.read_i32(index).await {
                Ok(value) => println!("✓ {} = {}", var_name, value),
                Err(e) => eprintln!("✗ Failed to read {}: {}", var_name, e),
            }
        } else if var_name.starts_with("R") {
            match client.read_f32(index).await {
                Ok(value) => println!("✓ {} = {}", var_name, value),
                Err(e) => eprintln!("✗ Failed to read {}: {}", var_name, e),
            }
        }
    }

    // Test string variables (0x7E command)
    println!("\n--- String Variable Operations (0x7E) ---");

    // Read string variable
    match client.read_string(0).await {
        Ok(value) => println!("✓ S000 = '{}'", String::from_utf8_lossy(&value)),
        Err(e) => eprintln!("✗ Failed to read S000: {}", e),
    }

    // Write string variable
    let test_string = b"Hello, Robot!";
    match client.write_string(0, test_string.to_vec()).await {
        Ok(()) => println!("✓ Wrote '{}' to S000", String::from_utf8_lossy(test_string)),
        Err(e) => eprintln!("✗ Failed to write to S000: {}", e),
    }

    // Verify written string
    match client.read_string(0).await {
        Ok(value) => {
            let expected = String::from_utf8_lossy(test_string);
            let actual = String::from_utf8_lossy(&value);
            if value == test_string {
                println!("✓ S000 = '{}' (expected: '{}')", actual, expected);
            } else {
                println!("✗ S000 = '{}' (expected: '{}')", actual, expected);
            }
        }
        Err(e) => eprintln!("✗ Failed to read S000: {}", e),
    }

    // Test another string variable
    let test_string2 = b"Test String 123";
    match client.write_string(1, test_string2.to_vec()).await {
        Ok(()) => println!(
            "✓ Wrote '{}' to S001",
            String::from_utf8_lossy(test_string2)
        ),
        Err(e) => eprintln!("✗ Failed to write to S001: {}", e),
    }

    match client.read_string(1).await {
        Ok(value) => println!("✓ S001 = '{}'", String::from_utf8_lossy(&value)),
        Err(e) => eprintln!("✗ Failed to read S001: {}", e),
    }

    // Test error handling
    println!("\n--- Error Handling Tests ---");

    // Test invalid variable index
    match client.read_i16(255).await {
        Ok(value) => {
            println!("✗ Invalid variable index succeeded unexpectedly: {}", value);
        }
        Err(e) => {
            println!("✓ Invalid variable index correctly failed: {}", e);
        }
    }

    // Test invalid variable index for write
    match client.write_i16(255, 42).await {
        Ok(()) => {
            println!("✗ Invalid variable index write succeeded unexpectedly");
        }
        Err(e) => {
            println!("✓ Invalid variable index write correctly failed: {}", e);
        }
    }

    // Test invalid string variable index
    match client.read_string(255).await {
        Ok(value) => {
            println!(
                "✗ Invalid string variable index succeeded unexpectedly: '{}'",
                String::from_utf8_lossy(&value)
            );
        }
        Err(e) => {
            println!("✓ Invalid string variable index correctly failed: {}", e);
        }
    }

    println!("\n--- Variable Operations Example completed successfully ---");
    Ok(())
}
