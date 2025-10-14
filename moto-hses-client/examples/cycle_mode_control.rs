use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{CycleMode, TextEncoding, ROBOT_CONTROL_PORT};
use std::time::Duration;

const TARGET_CYCLE_MODE: CycleMode = CycleMode::Step;

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

    info!("Setting cycle mode to: {TARGET_CYCLE_MODE:?}");
    match client.set_cycle_mode(TARGET_CYCLE_MODE).await {
        Ok(()) => {
            info!("✓ Successfully set cycle mode to {TARGET_CYCLE_MODE:?}");
        }
        Err(e) => {
            info!("✗ Failed to set cycle mode to {TARGET_CYCLE_MODE:?}: {e}");
            return Ok(());
        }
    }

    let data1 = client.read_status_data1().await?;
    info!(
        "Status:(step:{},one cycle:{},continuous:{})",
        data1.step, data1.one_cycle, data1.continuous
    );

    Ok(())
}
