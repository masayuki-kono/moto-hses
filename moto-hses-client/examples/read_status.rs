use moto_hses_client::HsesClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:12222".to_string());
    let cli = HsesClient::connect(&controller).await?;
    let status = cli.read_status().await?;
    println!("status: {:?}", status);
    println!("running: {}", status.is_running());
    println!("servo on: {}", status.is_servo_on());
    println!("alarm: {}", status.has_alarm());
    Ok(())
}