//! Register operations example for 0x79 register command

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
            let robot_port: u16 = robot_port
                .parse()
                .map_err(|e| format!("Invalid robot port: {robot_port} - {e}"))?;

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

    // Test reading a register
    info!("Reading register 0...");
    let value = client.read_register(0).await?;
    info!("Register 0 value: {value}");

    // Test writing to a register
    info!("Writing value 150 to register 0...");
    client.write_register(0, 150).await?;
    info!("Write successful");

    // Test reading the register again to verify the write
    info!("Reading register 0 again to verify write...");
    let new_value = client.read_register(0).await?;
    info!("Register 0 value after write: {new_value}");

    // Test reading multiple registers (0x301 command)
    info!("Reading multiple registers from 0 to 4...");
    let values = client.read_multiple_registers(0, 5).await?;
    info!("Read {} register values: {values:?}", values.len());

    // Test writing multiple registers (0x301 command)
    info!("Writing multiple registers to 10-14...");
    let values_to_write = vec![1000, 2000, 3000, 4000, 5000];
    client.write_multiple_registers(10, values_to_write.clone()).await?;
    info!("Successfully wrote {} register values", values_to_write.len());

    // Verify the multiple register write
    info!("Reading back multiple registers from 10-14 to verify write...");
    let read_values = client.read_multiple_registers(10, 5).await?;
    info!("Read back values: {read_values:?}");
    info!(
        "Write verification: {}",
        if read_values == values_to_write { "✓ PASSED" } else { "✗ FAILED" }
    );

    Ok(())
}
