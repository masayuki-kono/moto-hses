use moto_hses_client::HsesClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let controller_addr = format!("{}:{}", host, robot_port);
    println!("Connecting to controller at {}...", controller_addr);
    let client = HsesClient::new(&controller_addr).await?;

    println!("Successfully connected to controller");

    println!("=== 0x72 Command (Status Information Reading) Usage Example ===\n");

    // 1. Read complete status information (Data1 + Data2)
    println!("1. Read complete status information (Data1 + Data2):");
    let status = client.read_status().await?;
    println!("✓ Complete status information retrieved");
    println!("  Status: {:?}", status);
    println!("  Running: {}", status.is_running());
    println!("  Servo on: {}", status.is_servo_on());
    println!("  Alarm: {}", status.has_alarm());
    println!("  Error: {}", status.has_error());
    println!("  Play mode: {}", status.is_play_mode());
    println!("  Teach mode: {}", status.is_teach_mode());
    println!();

    // 2. Read Data1 only (basic status information)
    println!("2. Read Data1 only (basic status information):");
    let data1 = client.read_status_data1().await?;
    println!("✓ Data1 retrieved");
    println!("  Data1: {:?}", data1);
    println!("  Step: {}", data1.step);
    println!("  One cycle: {}", data1.one_cycle);
    println!("  Continuous: {}", data1.continuous);
    println!("  Running: {}", data1.running);
    println!("  Speed limited: {}", data1.speed_limited);
    println!("  Teach: {}", data1.teach);
    println!("  Play: {}", data1.play);
    println!("  Remote: {}", data1.remote);
    println!();

    // 3. Read Data2 only (additional status information)
    println!("3. Read Data2 only (additional status information):");
    let data2 = client.read_status_data2().await?;
    println!("✓ Data2 retrieved");
    println!("  Data2: {:?}", data2);
    println!("  Teach pendant hold: {}", data2.teach_pendant_hold);
    println!("  External hold: {}", data2.external_hold);
    println!("  Command hold: {}", data2.command_hold);
    println!("  Alarm: {}", data2.alarm);
    println!("  Error: {}", data2.error);
    println!("  Servo on: {}", data2.servo_on);
    println!();

    // 4. Convenience methods usage example
    println!("4. Convenience methods usage example:");
    println!("  is_running(): {}", status.is_running());
    println!("  is_servo_on(): {}", status.is_servo_on());
    println!("  has_alarm(): {}", status.has_alarm());
    println!("  has_error(): {}", status.has_error());
    println!("  is_teach_mode(): {}", status.is_teach_mode());
    println!("  is_play_mode(): {}", status.is_play_mode());
    println!("  is_remote_mode(): {}", status.is_remote_mode());
    println!();

    println!("=== Usage example completed ===");

    Ok(())
}
