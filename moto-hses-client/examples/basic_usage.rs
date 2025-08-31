use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::CoordinateSystemType;
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
            // Default: 127.0.0.1:10040
            ("127.0.0.1".to_string(), 10040)
        }
    };

    println!("HSES Client Basic Usage Example");
    println!("Connecting to controller at: {}:{}", host, robot_port);

    // Create custom configuration
    let config = ClientConfig {
        timeout: Duration::from_millis(500),
        retry_count: 5,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
    };

    // Connect to the controller
    let client =
        match HsesClient::new_with_config(&format!("{}:{}", host, robot_port), config).await {
            Ok(client) => {
                println!("✓ Successfully connected to controller");
                client
            }
            Err(e) => {
                eprintln!("✗ Failed to connect: {}", e);
                return Ok(());
            }
        };

    // Read robot status
    println!("\n--- Robot Status ---");
    match client.read_status().await {
        Ok(status) => {
            println!("✓ Status read successfully");
            println!("  Running: {}", status.is_running());
            println!("  Servo on: {}", status.is_servo_on());
            println!("  Alarm: {}", status.has_alarm());
            println!("  Error: {}", status.error);
            println!("  Play mode: {}", status.is_play_mode());
            println!("  Teach mode: {}", status.is_teach_mode());
        }
        Err(e) => {
            eprintln!("✗ Failed to read status: {}", e);
        }
    }

    // Read current position
    println!("\n--- Current Position ---");
    match client
        .read_position(1, CoordinateSystemType::RobotPulse)
        .await
    {
        Ok(position) => {
            println!("✓ Position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read position: {}", e);
        }
    }

    // Read variables
    println!("\n--- Variable Operations ---");

    // Read integer variable
    match client.read_int(0).await {
        Ok(value) => {
            println!("✓ D000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read D000: {}", e);
        }
    }

    // Read float variable
    match client.read_float(0).await {
        Ok(value) => {
            println!("✓ R000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read R000: {}", e);
        }
    }

    // Read byte variable
    match client.read_byte(0).await {
        Ok(value) => {
            println!("✓ B000 = {}", value);
        }
        Err(e) => {
            eprintln!("✗ Failed to read B000: {}", e);
        }
    }

    // Write variables (commented out for safety)
    /*
    println!("\n--- Writing Variables ---");

    // Write integer variable
    match client.write_int(1, 42).await {
        Ok(()) => {
            println!("✓ Wrote 42 to D001");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to D001: {}", e);
        }
    }

    // Write float variable
    match client.write_float(1, 3.14159).await {
        Ok(()) => {
            println!("✓ Wrote 3.14159 to R001");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to R001: {}", e);
        }
    }

    // Write byte variable
    match client.write_byte(1, 255).await {
        Ok(()) => {
            println!("✓ Wrote 255 to B001");
        }
        Err(e) => {
            eprintln!("✗ Failed to write to B001: {}", e);
        }
    }
    */

    // Convenience methods
    println!("\n--- Convenience Methods ---");

    match client.is_running().await {
        Ok(running) => {
            println!("✓ Robot running: {}", running);
        }
        Err(e) => {
            eprintln!("✗ Failed to check running status: {}", e);
        }
    }

    match client.is_servo_on().await {
        Ok(servo_on) => {
            println!("✓ Servo on: {}", servo_on);
        }
        Err(e) => {
            eprintln!("✗ Failed to check servo status: {}", e);
        }
    }

    match client.has_alarm().await {
        Ok(has_alarm) => {
            println!("✓ Has alarm: {}", has_alarm);
        }
        Err(e) => {
            eprintln!("✗ Failed to check alarm status: {}", e);
        }
    }

    println!("\n--- Example completed successfully ---");
    Ok(())
}
