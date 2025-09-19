use log::info;
use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
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

    info!("HSES Client Alarm Operations Example");
    info!("Connecting to controller at: {host}:{robot_port}");

    // Create custom configuration
    let config = ClientConfig {
        host: host.to_string(),
        port: robot_port,
        timeout: Duration::from_millis(500),
        retry_count: 5,
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

    // Test alarm data reading (0x70 command)
    info!("\n--- Complete Alarm Data (Instance 1) ---");
    match client.read_alarm_data(1, 0).await {
        Ok(alarm) => {
            info!("✓ Complete alarm data read successfully");
            info!("  Code: {}", alarm.code);
            info!("  Data: {}", alarm.data);
            info!("  Type: {}", alarm.alarm_type);
            info!("  Time: {}", alarm.time);
            info!("  Name: {}", alarm.name);
            info!("  Raw alarm data: {alarm:?}");
        }
        Err(e) => {
            info!("✗ Failed to read complete alarm data: {e}");
        }
    }

    // Test alarm history reading (0x71 command)
    info!("\n--- Alarm History Reading (0x71 Command) ---");

    // Test major failure alarm history
    test_alarm_history(&client, "major failure", 1..=10).await;

    // Test monitor alarm history
    test_alarm_history(&client, "monitor", 1001..=1010).await;

    // Test alarm reset command
    info!("\n--- Alarm Reset Command (0x82, Instance 1) ---");
    match client.reset_alarm().await {
        Ok(()) => {
            info!("✓ Alarm reset command executed successfully");
        }
        Err(e) => {
            info!("✗ Alarm reset command failed: {e}");
        }
    }

    // Test error cancel command
    info!("\n--- Error Cancel Command (0x82, Instance 2) ---");
    match client.cancel_error().await {
        Ok(()) => {
            info!("✓ Error cancel command executed successfully");
        }
        Err(e) => {
            info!("✗ Error cancel command failed: {e}");
        }
    }

    Ok(())
}

/// Test alarm history reading for a given alarm type and instance range
async fn test_alarm_history(
    client: &HsesClient,
    alarm_type: &str,
    instances: std::ops::RangeInclusive<u16>,
) {
    info!("Testing {alarm_type} alarm history(max 10 instances)...");
    for instance in instances {
        match client.read_alarm_history(instance, 0).await {
            Ok(alarm) => {
                if alarm.code != 0 {
                    info!(
                        "✓ {} alarm {instance}: Code={}, Name={}",
                        alarm_type, alarm.code, alarm.name
                    );
                    info!("  Full alarm data for instance {instance}: {alarm:?}");
                } else {
                    break;
                }
            }
            Err(e) => {
                info!("✗ Failed to read {alarm_type} alarm {instance}: {e}");
            }
        }
    }
}
