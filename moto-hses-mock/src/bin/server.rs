//! Mock HSES UDP server
//! Usage: cargo run -p moto-hses-mock -- [addr:port]
//! Default: 127.0.0.1:10040

use std::net::SocketAddr;
use moto_hses_mock::MockServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind: SocketAddr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:10040".to_string()).parse()?;
    
    // Create and run mock server
    let server = MockServer::new(moto_hses_mock::MockConfig {
        bind_addr: bind,
        ..Default::default()
    }).await?;
    
    eprintln!("Mock HSES server listening on {}", bind);
    eprintln!("Supported commands: 0x70, 0x72, 0x75, 0x78, 0x79, 0x7a, 0x7b, 0x7d, 0x7f, 0x82, 0x83, 0x86, 0x87");
    
    // Run the server
    server.run().await?;
    
    Ok(())
}