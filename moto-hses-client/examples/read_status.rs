use std::net::SocketAddr;
use moto_hses_client::HsesClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller: SocketAddr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:12222".to_string()).parse()?;
    let mut cli = HsesClient::connect(controller).await?;
    let status = cli.read_status().await?;
    println!("raw status bytes ({}): {:02X?}", status.len(), status);
    Ok(())
}