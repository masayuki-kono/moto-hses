use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{AlarmAttribute, ROBOT_CONTROL_PORT};
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

    println!("HSES Client Alarm Operations Example");
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

    // Read complete alarm data (attribute 0)
    println!("\n--- Complete Alarm Data (Instance 1) ---");
    match client.read_alarm_data(1, 0).await {
        Ok(alarm) => {
            println!("✓ Complete alarm data read successfully");
            println!("  Code: {}", alarm.code);
            println!("  Data: {}", alarm.data);
            println!("  Type: {}", alarm.alarm_type);
            println!("  Time: {}", alarm.time);
            println!("  Name: {}", alarm.name);
            println!("  Sub Code Info: {}", alarm.sub_code_info);
            println!("  Sub Code Data: {}", alarm.sub_code_data);
            println!("  Sub Code Reverse: {}", alarm.sub_code_reverse);
        }
        Err(e) => {
            eprintln!("✗ Failed to read complete alarm data: {}", e);
        }
    }

    // Read specific alarm attributes
    println!("\n--- Specific Alarm Attributes ---");

    // Read alarm code
    match client.read_alarm_data(1, AlarmAttribute::Code as u8).await {
        Ok(alarm) => {
            println!("✓ Alarm code: {}", alarm.code);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm code: {}", e);
        }
    }

    // Read alarm data
    match client.read_alarm_data(1, AlarmAttribute::Data as u8).await {
        Ok(alarm) => {
            println!("✓ Alarm data: {}", alarm.data);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm data: {}", e);
        }
    }

    // Read alarm type
    match client.read_alarm_data(1, AlarmAttribute::Type as u8).await {
        Ok(alarm) => {
            println!("✓ Alarm type: {}", alarm.alarm_type);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm type: {}", e);
        }
    }

    // Read alarm time
    match client.read_alarm_data(1, AlarmAttribute::Time as u8).await {
        Ok(alarm) => {
            println!("✓ Alarm time: {}", alarm.time);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm time: {}", e);
        }
    }

    // Read alarm name
    match client.read_alarm_data(1, AlarmAttribute::Name as u8).await {
        Ok(alarm) => {
            println!("✓ Alarm name: {}", alarm.name);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm name: {}", e);
        }
    }

    // Read multiple alarm instances (HSES specification: Instance 1-4)
    println!("\n--- Multiple Alarm Instances ---");
    for instance in 1..=4 {
        match client.read_alarm_data(instance, 0).await {
            Ok(alarm) => {
                if alarm.code != 0 {
                    println!(
                        "✓ Alarm instance {}: Code={}, Name={}",
                        instance, alarm.code, alarm.name
                    );
                } else {
                    println!("✓ Alarm instance {}: No alarm", instance);
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to read alarm instance {}: {}", instance, e);
            }
        }
    }

    // Test alarm history reading (0x71 command)
    println!("\n--- Alarm History Reading (0x71 Command) ---");

    // Test major failure alarm history (instances 1-3)
    println!("\n--- Major Failure Alarm History ---");
    for instance in 1..=3 {
        match client
            .read_alarm_history(instance, AlarmAttribute::Code as u8)
            .await
        {
            Ok(alarm) => {
                if alarm.code != 0 {
                    println!(
                        "✓ Major failure alarm {}: Code={}, Name={}",
                        instance, alarm.code, alarm.name
                    );
                } else {
                    println!("✓ Major failure alarm {}: No alarm", instance);
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to read major failure alarm {}: {}", instance, e);
            }
        }
    }

    // Test monitor alarm history (instances 1001-1003)
    println!("\n--- Monitor Alarm History ---");
    for instance in 1001..=1003 {
        match client
            .read_alarm_history(instance, AlarmAttribute::Name as u8)
            .await
        {
            Ok(alarm) => {
                if alarm.code != 0 {
                    println!(
                        "✓ Monitor alarm {}: Code={}, Name={}",
                        instance, alarm.code, alarm.name
                    );
                } else {
                    println!("✓ Monitor alarm {}: No alarm", instance);
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to read monitor alarm {}: {}", instance, e);
            }
        }
    }

    // Test different attributes for alarm history
    println!("\n--- Alarm History Attributes Test ---");
    match client
        .read_alarm_history(1, AlarmAttribute::Code as u8)
        .await
    {
        Ok(alarm) => {
            println!("✓ Major failure alarm #1 code: {}", alarm.code);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm history code: {}", e);
        }
    }

    match client
        .read_alarm_history(1, AlarmAttribute::Time as u8)
        .await
    {
        Ok(alarm) => {
            println!("✓ Major failure alarm #1 time: {}", alarm.time);
        }
        Err(e) => {
            eprintln!("✗ Failed to read alarm history time: {}", e);
        }
    }

    // Test invalid instance (should return empty data)
    println!("\n--- Invalid Instance Test ---");
    match client
        .read_alarm_history(5000, AlarmAttribute::Code as u8)
        .await
    {
        Ok(alarm) => {
            if alarm.code == 0 {
                println!("✓ Invalid instance correctly returned empty data");
            } else {
                println!(
                    "⚠ Invalid instance returned unexpected data: {}",
                    alarm.code
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to read invalid instance: {}", e);
        }
    }

    // Test error handling for alarm data
    println!("\n--- Error Handling Tests ---");

    // Test invalid alarm instance (5000 is outside all valid ranges)
    match client.read_alarm_data(5000, 1).await {
        Ok(alarm) => {
            println!(
                "✗ Invalid alarm instance succeeded unexpectedly: code={}",
                alarm.code
            );
        }
        Err(e) => {
            println!("✓ Invalid alarm instance correctly failed: {}", e);
        }
    }

    // Test invalid alarm attribute
    match client.read_alarm_data(1, 255).await {
        Ok(alarm) => {
            println!(
                "✗ Invalid alarm attribute succeeded unexpectedly: code={}",
                alarm.code
            );
        }
        Err(e) => {
            println!("✓ Invalid alarm attribute correctly failed: {}", e);
        }
    }

    println!("\n--- Alarm operations example completed successfully ---");
    Ok(())
}
