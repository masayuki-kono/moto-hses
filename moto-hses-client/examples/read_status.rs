use moto_hses_client::HsesClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:12222".to_string());
    
    println!("Connecting to controller at {}...", controller);
    let client = HsesClient::new(&controller).await?;
    
    if client.is_connected() {
        println!("Successfully connected to controller");
    } else {
        println!("Failed to connect to controller");
        return Ok(());
    }

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