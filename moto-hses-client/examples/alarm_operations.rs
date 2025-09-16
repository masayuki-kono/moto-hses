use log::info;
use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{AlarmAttribute, ROBOT_CONTROL_PORT};
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

    // Read complete alarm data (attribute 0)
    info!("\n--- Complete Alarm Data (Instance 1) ---");
    match client.read_alarm_data(1, 0).await {
        Ok(alarm) => {
            info!("✓ Complete alarm data read successfully");
            info!("  Code: {}", alarm.code);
            info!("  Data: {}", alarm.data);
            info!("  Type: {}", alarm.alarm_type);
            info!("  Time: {}", alarm.time);
            info!("  Name: {}", alarm.name);
            info!("  Sub Code Info: {}", alarm.sub_code_info);
            info!("  Sub Code Data: {}", alarm.sub_code_data);
            info!("  Sub Code Reverse: {}", alarm.sub_code_reverse);
        }
        Err(e) => {
            info!("✗ Failed to read complete alarm data: {e}");
        }
    }

    // Read specific alarm attributes
    info!("\n--- Specific Alarm Attributes ---");

    // Read alarm code
    match client.read_alarm_data(1, AlarmAttribute::Code as u8).await {
        Ok(alarm) => {
            info!("✓ Alarm code: {}", alarm.code);
        }
        Err(e) => {
            info!("✗ Failed to read alarm code: {e}");
        }
    }

    // Read alarm data
    match client.read_alarm_data(1, AlarmAttribute::Data as u8).await {
        Ok(alarm) => {
            info!("✓ Alarm data: {}", alarm.data);
        }
        Err(e) => {
            info!("✗ Failed to read alarm data: {e}");
        }
    }

    // Read alarm type
    match client.read_alarm_data(1, AlarmAttribute::Type as u8).await {
        Ok(alarm) => {
            info!("✓ Alarm type: {}", alarm.alarm_type);
        }
        Err(e) => {
            info!("✗ Failed to read alarm type: {e}");
        }
    }

    // Read alarm time
    match client.read_alarm_data(1, AlarmAttribute::Time as u8).await {
        Ok(alarm) => {
            info!("✓ Alarm time: {}", alarm.time);
        }
        Err(e) => {
            info!("✗ Failed to read alarm time: {e}");
        }
    }

    // Read alarm name
    match client.read_alarm_data(1, AlarmAttribute::Name as u8).await {
        Ok(alarm) => {
            info!("✓ Alarm name: {}", alarm.name);
        }
        Err(e) => {
            info!("✗ Failed to read alarm name: {e}");
        }
    }

    // Read multiple alarm instances (HSES specification: Instance 1-4)
    info!("\n--- Multiple Alarm Instances ---");
    for instance in 1..=4 {
        match client.read_alarm_data(instance, 0).await {
            Ok(alarm) => {
                if alarm.code != 0 {
                    info!(
                        "✓ Alarm instance {}: Code={}, Name={}",
                        instance, alarm.code, alarm.name
                    );
                } else {
                    info!("✓ Alarm instance {instance}: No alarm");
                }
            }
            Err(e) => {
                info!("✗ Failed to read alarm instance {instance}: {e}");
            }
        }
    }

    // Test alarm history reading (0x71 command)
    info!("\n--- Alarm History Reading (0x71 Command) ---");

    // Test major failure alarm history (instances 1-3)
    info!("\n--- Major Failure Alarm History ---");
    for instance in 1..=3 {
        match client.read_alarm_history(instance, AlarmAttribute::Code as u8).await {
            Ok(alarm) => {
                if alarm.code != 0 {
                    info!(
                        "✓ Major failure alarm {}: Code={}, Name={}",
                        instance, alarm.code, alarm.name
                    );
                } else {
                    info!("✓ Major failure alarm {instance}: No alarm");
                }
            }
            Err(e) => {
                info!("✗ Failed to read major failure alarm {instance}: {e}");
            }
        }
    }

    // Test monitor alarm history (instances 1001-1003)
    info!("\n--- Monitor Alarm History ---");
    for instance in 1001..=1003 {
        match client.read_alarm_history(instance, AlarmAttribute::Name as u8).await {
            Ok(alarm) => {
                if alarm.code != 0 {
                    info!("✓ Monitor alarm {}: Code={}, Name={}", instance, alarm.code, alarm.name);
                } else {
                    info!("✓ Monitor alarm {instance}: No alarm");
                }
            }
            Err(e) => {
                info!("✗ Failed to read monitor alarm {instance}: {e}");
            }
        }
    }

    // Test different attributes for alarm history
    info!("\n--- Alarm History Attributes Test ---");
    match client.read_alarm_history(1, AlarmAttribute::Code as u8).await {
        Ok(alarm) => {
            info!("✓ Major failure alarm #1 code: {}", alarm.code);
        }
        Err(e) => {
            info!("✗ Failed to read alarm history code: {e}");
        }
    }

    match client.read_alarm_history(1, AlarmAttribute::Time as u8).await {
        Ok(alarm) => {
            info!("✓ Major failure alarm #1 time: {}", alarm.time);
        }
        Err(e) => {
            info!("✗ Failed to read alarm history time: {e}");
        }
    }

    // Test invalid instance (should return empty data)
    info!("\n--- Invalid Instance Test ---");
    match client.read_alarm_history(5000, AlarmAttribute::Code as u8).await {
        Ok(alarm) => {
            if alarm.code == 0 {
                info!("✓ Invalid instance correctly returned empty data");
            } else {
                info!("⚠ Invalid instance returned unexpected data: {}", alarm.code);
            }
        }
        Err(e) => {
            info!("✗ Failed to read invalid instance: {e}");
        }
    }

    // Test error handling for alarm data
    info!("\n--- Error Handling Tests ---");

    // Test invalid alarm instance (5000 is outside all valid ranges)
    match client.read_alarm_data(5000, 1).await {
        Ok(alarm) => {
            info!("✗ Invalid alarm instance succeeded unexpectedly: code={}", alarm.code);
        }
        Err(e) => {
            info!("✓ Invalid alarm instance correctly failed: {e}");
        }
    }

    // Test invalid alarm attribute
    match client.read_alarm_data(1, 255).await {
        Ok(alarm) => {
            info!("✗ Invalid alarm attribute succeeded unexpectedly: code={}", alarm.code);
        }
        Err(e) => {
            info!("✓ Invalid alarm attribute correctly failed: {e}");
        }
    }

    // Test 0x82 Alarm Reset / Error Cancel Commands
    info!("\n--- 0x82 Alarm Reset / Error Cancel Commands ---");

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
