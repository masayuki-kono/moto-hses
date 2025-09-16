use log::info;
use moto_hses_client::{ClientConfig, ClientError, HsesClient};
use moto_hses_proto::ROBOT_CONTROL_PORT;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("HSES Client Connection Management Example");

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

    let controller_addr = format!("{host}:{robot_port}");
    info!("Target controller: {controller_addr}");

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
    info!("\n--- Connection Attempt ---");
    let client = match HsesClient::new_with_config(config).await {
        Ok(client) => {
            info!("✓ Successfully connected to controller");
            client
        }
        Err(ClientError::ConnectionFailed(retries)) => {
            info!("✗ Connection failed after {retries} retries");
            info!("  Please check:");
            info!("    - Controller is powered on");
            info!("    - Network connectivity");
            info!("    - HSES is enabled on controller");
            info!("    - Firewall settings");
            return Ok(());
        }
        Err(ClientError::SystemError(msg)) => {
            info!("✗ System error: {msg}");
            return Ok(());
        }
        Err(e) => {
            info!("✗ Unexpected error: {e}");
            return Ok(());
        }
    };

    // Verify connection status
    info!("\n--- Connection Status ---");
    info!("✓ Client created successfully");

    // Test basic communication
    info!("\n--- Communication Test ---");
    match client.read_status().await {
        Ok(status) => {
            info!("✓ Communication successful");
            info!("  Robot running: {}", status.is_running());
            info!("  Servo on: {}", status.is_servo_on());
        }
        Err(ClientError::TimeoutError(_)) => {
            info!("✗ Communication timeout - robot may be busy or network slow");
        }
        Err(ClientError::ProtocolError(e)) => {
            info!("✗ Protocol error: {e}");
        }
        Err(e) => {
            info!("✗ Communication failed: {e}");
        }
    }

    // Demonstrate reconnection
    info!("\n--- Reconnection Test ---");
    info!("UDP is connectionless, so reconnection is not applicable");
    info!("✓ Client is ready for communication");

    // Test connection with invalid address
    info!("\n--- Invalid Address Test ---");
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
            info!("✓ Unexpectedly connected to invalid address");
        }
        Err(ClientError::ConnectionFailed(_)) => {
            info!("✓ Correctly failed to connect to invalid address");
        }
        Err(ClientError::SystemError(_)) => {
            info!("✓ Correctly failed to parse invalid address");
        }
        Err(e) => {
            info!("✓ Failed as expected: {e}");
        }
    }

    info!("\n--- Connection management example completed ---");
    Ok(())
}
