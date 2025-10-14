use log::info;
use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding};
use std::time::Duration;

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

    info!("=== 0x72 Command (Status Information Reading) ===\n");

    // 1. Read complete status information (Data1 + Data2)
    info!("1. Read complete status information (Data1 + Data2):");
    let status = client.read_status().await?;
    info!("✓ Complete status information retrieved");
    info!("  Status: {status:?}");
    info!("  Running: {}", status.is_running());
    info!("  Servo on: {}", status.is_servo_on());
    info!("  Alarm: {}", status.has_alarm());
    info!("  Error: {}", status.has_error());
    info!("  Play mode: {}", status.is_play_mode());
    info!("  Teach mode: {}", status.is_teach_mode());

    // 2. Read Data1 only (basic status information)
    info!("2. Read Data1 only (basic status information):");
    let data1 = client.read_status_data1().await?;
    info!("✓ Data1 retrieved");
    info!("  Data1: {data1:?}");
    info!("  Step: {}", data1.step);
    info!("  One cycle: {}", data1.one_cycle);
    info!("  Continuous: {}", data1.continuous);
    info!("  Running: {}", data1.running);
    info!("  Speed limited: {}", data1.speed_limited);
    info!("  Teach: {}", data1.teach);
    info!("  Play: {}", data1.play);
    info!("  Remote: {}", data1.remote);

    // 3. Read Data2 only (additional status information)
    info!("3. Read Data2 only (additional status information):");
    let data2 = client.read_status_data2().await?;
    info!("✓ Data2 retrieved");
    info!("  Data2: {data2:?}");
    info!("  Teach pendant hold: {}", data2.teach_pendant_hold);
    info!("  External hold: {}", data2.external_hold);
    info!("  Command hold: {}", data2.command_hold);
    info!("  Alarm: {}", data2.alarm);
    info!("  Error: {}", data2.error);
    info!("  Servo on: {}", data2.servo_on);

    // 4. Convenience methods usage example
    info!("4. Convenience methods usage example:");
    info!("  is_running(): {}", status.is_running());
    info!("  is_servo_on(): {}", status.is_servo_on());
    info!("  has_alarm(): {}", status.has_alarm());
    info!("  has_error(): {}", status.has_error());
    info!("  is_teach_mode(): {}", status.is_teach_mode());
    info!("  is_play_mode(): {}", status.is_play_mode());
    info!("  is_remote_mode(): {}", status.is_remote_mode());

    info!("=== Usage example completed ===");

    Ok(())
}
