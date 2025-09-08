use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{CoordinateSystemType, ROBOT_CONTROL_PORT};
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

    println!("HSES Client Position Operations Example");
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

    // Read current position in different coordinate systems
    println!("\n--- Current Position Reading ---");

    // Read position in robot pulse coordinates
    match client
        .read_position(1, CoordinateSystemType::RobotPulse)
        .await
    {
        Ok(position) => {
            println!("✓ Robot pulse position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read robot pulse position: {}", e);
        }
    }

    // Read position in base pulse coordinates
    match client
        .read_position(1, CoordinateSystemType::BasePulse)
        .await
    {
        Ok(position) => {
            println!("✓ Base pulse position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read base pulse position: {}", e);
        }
    }

    // Read position in station pulse coordinates
    match client
        .read_position(1, CoordinateSystemType::StationPulse)
        .await
    {
        Ok(position) => {
            println!("✓ Station pulse position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read station pulse position: {}", e);
        }
    }

    // Read position in robot cartesian coordinates
    match client
        .read_position(1, CoordinateSystemType::RobotCartesian)
        .await
    {
        Ok(position) => {
            println!("✓ Robot cartesian position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read robot cartesian position: {}", e);
        }
    }

    // Read position for different control groups
    println!("\n--- Different Control Groups ---");

    // Read position for control group 1 (R1)
    match client
        .read_position(1, CoordinateSystemType::RobotPulse)
        .await
    {
        Ok(position) => {
            println!("✓ R1 position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read R1 position: {}", e);
        }
    }

    // Read position for control group 2 (R2)
    match client
        .read_position(2, CoordinateSystemType::RobotPulse)
        .await
    {
        Ok(position) => {
            println!("✓ R2 position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read R2 position: {}", e);
        }
    }

    // Read position for base control group 1 (B1)
    match client
        .read_position(11, CoordinateSystemType::RobotPulse)
        .await
    {
        Ok(position) => {
            println!("✓ B1 position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read B1 position: {}", e);
        }
    }

    // Read position for base control group 2 (B2)
    match client
        .read_position(12, CoordinateSystemType::RobotPulse)
        .await
    {
        Ok(position) => {
            println!("✓ B2 position read successfully");
            println!("  Position: {:?}", position);
        }
        Err(e) => {
            eprintln!("✗ Failed to read B2 position: {}", e);
        }
    }

    // Continuous position monitoring example
    println!("\n--- Continuous Position Monitoring ---");
    println!("Monitoring position for 5 seconds...");

    for i in 1..=5 {
        match client
            .read_position(1, CoordinateSystemType::RobotPulse)
            .await
        {
            Ok(position) => {
                println!("  [{}s] Position: {:?}", i, position);
            }
            Err(e) => {
                eprintln!("  [{}s] Failed to read position: {}", i, e);
            }
        }

        if i < 5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    println!("\n--- Position Operations Example completed successfully ---");
    Ok(())
}
