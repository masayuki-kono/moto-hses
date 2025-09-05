use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::AlarmAttribute;
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

    println!("HSES Client Alarm Operations Example");
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

    println!("\n--- Alarm operations example completed successfully ---");
    Ok(())
}
