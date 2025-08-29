use moto_hses_client::{HsesClient, ClientConfig, ClientError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HSES Client Connection Management Example");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let controller_addr = args.get(1).unwrap_or(&"127.0.0.1:12222".to_string()).clone();
    
    println!("Target controller: {}", controller_addr);
    
    // Create configuration with aggressive retry settings
    let config = ClientConfig {
        timeout: Duration::from_millis(200),
        retry_count: 3,
        retry_delay: Duration::from_millis(100),
        buffer_size: 8192,
        connection_timeout: Duration::from_secs(3),
    };
    
    // Attempt to connect with error handling
    println!("\n--- Connection Attempt ---");
    let client = match HsesClient::new_with_config(&controller_addr, config).await {
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
    if client.is_connected() {
        println!("✓ Client reports connected");
    } else {
        println!("✗ Client reports not connected");
        return Ok(());
    }
    
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
    println!("Attempting to reconnect...");
    match client.reconnect().await {
        Ok(()) => {
            println!("✓ Reconnection successful");
        }
        Err(e) => {
            eprintln!("✗ Reconnection failed: {}", e);
        }
    }
    
    // Test connection with invalid address
    println!("\n--- Invalid Address Test ---");
    let invalid_config = ClientConfig {
        timeout: Duration::from_millis(100),
        retry_count: 1,
        retry_delay: Duration::from_millis(50),
        buffer_size: 8192,
        connection_timeout: Duration::from_millis(1),
    };
    
    match HsesClient::new_with_config("192.168.1.999:10040", invalid_config).await {
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
