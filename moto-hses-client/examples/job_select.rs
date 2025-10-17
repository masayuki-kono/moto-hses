#![allow(clippy::print_stdout, clippy::uninlined_format_args)]

use log::info;

use moto_hses_client::{ClientConfig, HsesClient};
use moto_hses_proto::{ROBOT_CONTROL_PORT, TextEncoding, commands::JobSelectType};
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

    // Job name does not require extension (.JOB)
    let job_name = "TEST".to_string();
    let line_number = 2;
    match client.select_job(JobSelectType::InExecution, job_name.clone(), line_number).await {
        Ok(()) => println!("✓ Successfully selected {job_name} for execution"),
        Err(e) => println!("✗ Failed to select job: {}", e),
    }

    Ok(())
}
