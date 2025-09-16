//! Mock HSES UDP server
//! Usage: cargo run -p moto-hses-mock -- [host] [`robot_port`] [`file_port`]
//! Examples:
//!   cargo run -p moto-hses-mock                    # Default: 127.0.0.1:10040, 127.0.0.1:10041
//!   cargo run -p moto-hses-mock -- 192.168.1.100 10040 10041
//!   cargo run -p moto-hses-mock -- 127.0.0.1 20000 20001

use log::info;
use moto_hses_mock::MockServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let (host, robot_port, file_port) = match args.as_slice() {
        [_, host, robot_port, file_port] => {
            // Format: [host] [robot_port] [file_port]
            let robot_port: u16 =
                robot_port.parse().map_err(|_| format!("Invalid robot port: {robot_port}"))?;
            let file_port: u16 =
                file_port.parse().map_err(|_| format!("Invalid file port: {file_port}"))?;

            (host.to_string(), robot_port, file_port)
        }
        _ => {
            // Default: 127.0.0.1:DEFAULT_PORT, 127.0.0.1:FILE_PORT
            (
                "127.0.0.1".to_string(),
                moto_hses_proto::ROBOT_CONTROL_PORT,
                moto_hses_proto::FILE_CONTROL_PORT,
            )
        }
    };

    info!("Starting HSES Mock Server:");
    info!("  Host: {host}");
    info!("  Robot Control Port: {robot_port}");
    info!("  File Control Port: {file_port}");

    // Create and run mock server
    let server =
        MockServer::new(moto_hses_mock::MockConfig::new(host, robot_port, file_port)).await?;

    info!(
        "Supported commands: 0x70, 0x72, 0x75, 0x78, 0x79, 0x7a, 0x7b, 0x7d, 0x7f, 0x82, 0x83, 0x86, 0x87"
    );

    // Run the server
    server.run().await?;

    Ok(())
}
