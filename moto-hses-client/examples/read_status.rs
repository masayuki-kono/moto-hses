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

    println!("Reading robot status...");
    let status = client.read_status().await?;

    println!("Status: {:?}", status);
    println!("Running: {}", status.is_running());
    println!("Servo on: {}", status.is_servo_on());
    println!("Alarm: {}", status.has_alarm());
    println!("Error: {}", status.error);
    println!("Play mode: {}", status.is_play_mode());
    println!("Teach mode: {}", status.is_teach_mode());

    Ok(())
}
