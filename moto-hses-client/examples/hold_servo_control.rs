use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{TextEncoding, ROBOT_CONTROL_PORT};
use std::time::Duration;

// Specify the command to execute
const COMMAND_NAME: &str = "servo"; // "servo", "hold", "hlock"

// Control command execution function
async fn execute_control<F, Fut>(
    client: &HsesClient,
    command_name: &str,
    command_func: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(bool) -> Fut,
    Fut: std::future::Future<Output = Result<(), moto_hses_client::ClientError>>,
{
    info!("Setting {command_name} to ON");
    match command_func(true).await {
        Ok(()) => {
            info!("✓ Successfully set {command_name} to ON");
        }
        Err(e) => {
            info!("✗ Failed to set {command_name}: {e}");
            return Err(Box::new(e));
        }
    }

    // Read and log system status after ON command
    if let Ok(data2) = client.read_status_data2().await {
        info!("System status after {command_name} ON:");
        info!("  Command hold: {}", data2.command_hold);
        info!("  Servo on: {}", data2.servo_on);
    } else {
        info!("✗ Failed to read system status after {command_name} ON");
    }

    // Wait for 3 seconds
    tokio::time::sleep(Duration::from_secs(3)).await;

    info!("Setting {command_name} to OFF");
    match command_func(false).await {
        Ok(()) => {
            info!("✓ Successfully set {command_name} to OFF");
        }
        Err(e) => {
            info!("✗ Failed to set {command_name}: {e}");
            return Err(Box::new(e));
        }
    }

    // Read and log system status after OFF command
    if let Ok(data2) = client.read_status_data2().await {
        info!("System status after {command_name} OFF:");
        info!("  Command hold: {}", data2.command_hold);
        info!("  Servo on: {}", data2.servo_on);
    } else {
        info!("✗ Failed to read system status after {command_name} OFF");
    }

    Ok(())
}

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

    match COMMAND_NAME {
        "servo" => {
            if let Err(e) = execute_control(&client, "servo", |value| client.set_servo(value)).await
            {
                info!("✗ Failed to execute servo command: {e}");
                return Ok(());
            }
        }
        "hold" => {
            if let Err(e) = execute_control(&client, "hold", |value| client.set_hold(value)).await {
                info!("✗ Failed to execute hold command: {e}");
                return Ok(());
            }
        }
        "hlock" => {
            if let Err(e) = execute_control(&client, "hlock", |value| client.set_hlock(value)).await
            {
                info!("✗ Failed to execute hlock command: {e}");
                return Ok(());
            }
        }
        _ => {
            info!("✗ Invalid command: {COMMAND_NAME}. Valid command: servo, hold, hlock");
            return Ok(());
        }
    }

    Ok(())
}
