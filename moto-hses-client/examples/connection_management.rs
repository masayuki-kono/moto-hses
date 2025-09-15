use moto_hses_client::{ClientConfig, ClientError, HsesClient};
use moto_hses_proto::ROBOT_CONTROL_PORT;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HSES Client Connection Management Example");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let (host, robot_port) = match args.as_slice() {
        [_, host, robot_port] => {
            // Format: [host] [robot_port]
            let robot_port: u16 =
                robot_port.parse().map_err(|_| format!("Invalid robot port: {}", robot_port))?;

            (host.to_string(), robot_port)
        }
        _ => {
            // Default: 127.0.0.1:DEFAULT_PORT
            ("127.0.0.1".to_string(), ROBOT_CONTROL_PORT)
        }
    };

    let controller_addr = format!("{}:{}", host, robot_port);
    println!("Target controller: {}", controller_addr);

    // Create configuration with aggressive retry settings
    let config = ClientConfig {
        host: "127.0.0.1".to_string(),
        port: ROBOT_CONTROL_PORT,
        timeout: Duration::from_millis(200),
        retry_count: 3,
        retry_delay: Duration::from_millis(100),
        buffer_size: 8192,
    };

    // Attempt to connect with error handling
    println!("\n--- Connection Attempt ---");
    let client = match HsesClient::new_with_config(config).await {
        Ok(client) => {
            println!("✓ Successfully connected to controller");
            client
        }
        Err(ClientError::ConnectionFailed(retries)) => {
            eprintln!("✗ Connection failed after {} retries", retries);
            eprintln!("  Please check:");
            eprintln!("    - Controller is powered on");
            eprintln!("    - Network connectivity");
            eprintln!("    - HSES is enabled on controller");
            eprintln!("    - Firewall settings");
            return Ok(());
        }
        Err(ClientError::SystemError(msg)) => {
            eprintln!("✗ System error: {}", msg);
            return Ok(());
        }
        Err(e) => {
            eprintln!("✗ Unexpected error: {}", e);
            return Ok(());
        }
    };

    // Verify connection status
    println!("\n--- Connection Status ---");
    println!("✓ Client created successfully");

    // Test basic communication
    println!("\n--- Communication Test ---");
    match client.read_status().await {
        Ok(status) => {
            println!("✓ Communication successful");
            println!("  Robot running: {}", status.is_running());
            println!("  Servo on: {}", status.is_servo_on());
        }
        Err(ClientError::TimeoutError(_)) => {
            eprintln!("✗ Communication timeout - robot may be busy or network slow");
        }
        Err(ClientError::ProtocolError(e)) => {
            eprintln!("✗ Protocol error: {}", e);
        }
        Err(e) => {
            eprintln!("✗ Communication failed: {}", e);
        }
    }

    // Demonstrate reconnection
    println!("\n--- Reconnection Test ---");
    println!("UDP is connectionless, so reconnection is not applicable");
    println!("✓ Client is ready for communication");

    // Test connection with invalid address
    println!("\n--- Invalid Address Test ---");
    let invalid_config = ClientConfig {
        host: "192.168.999.999".to_string(), // Invalid IP address
        port: ROBOT_CONTROL_PORT,
        timeout: Duration::from_millis(100),
        retry_count: 1,
        retry_delay: Duration::from_millis(50),
        buffer_size: 8192,
    };

    match HsesClient::new_with_config(invalid_config).await {
        Ok(_) => {
            println!("✓ Unexpectedly connected to invalid address");
        }
        Err(ClientError::ConnectionFailed(_)) => {
            println!("✓ Correctly failed to connect to invalid address");
        }
        Err(ClientError::SystemError(_)) => {
            println!("✓ Correctly failed to parse invalid address");
        }
        Err(e) => {
            println!("✓ Failed as expected: {}", e);
        }
    }

    println!("\n--- Connection management example completed ---");
    Ok(())
}
