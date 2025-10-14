//! Example: Read executing job information using 0x73 command
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
            let robot_port: u16 =
                robot_port.parse().map_err(|_| format!("Invalid robot port: {robot_port}"))?;

            (host.to_string(), robot_port)
        }
        _ => {
            // Default: 127.0.0.1:DEFAULT_PORT
            ("127.0.0.1".to_string(), ROBOT_CONTROL_PORT)
        }
    };

    // Create custom configuration with Shift_JIS encoding
    let config = ClientConfig {
        host: host.to_string(),
        port: robot_port,
        timeout: Duration::from_millis(3000),
        retry_count: 0,
        retry_delay: Duration::from_millis(200),
        buffer_size: 8192,
        text_encoding: TextEncoding::ShiftJis,
    };

    let controller_addr = format!("{host}:{robot_port}");
    info!("Connecting to controller at {controller_addr}...");
    let client = HsesClient::new_with_config(config).await?;

    info!("Reading executing job information...");

    // Read complete executing job information (all attributes)
    match client.read_executing_job_info_complete(1).await {
        Ok(job_info) => {
            info!("Complete job information:");
            info!("  Job name: {}", job_info.job_name);
            info!("  Line number: {}", job_info.line_number);
            info!("  Step number: {}", job_info.step_number);
            info!("  Speed override value: {}", job_info.speed_override_value);
        }
        Err(e) => {
            info!("Failed to read complete job information: {e}");
        }
    }

    // Read specific attributes
    info!("\nReading specific attributes:");

    // Read job name only
    match client.read_executing_job_info(1, 1).await {
        Ok(job_info) => {
            info!("  Job name: {}", job_info.job_name);
        }
        Err(e) => {
            info!("  Failed to read job name: {e}");
        }
    }

    // Read line number only
    match client.read_executing_job_info(1, 2).await {
        Ok(job_info) => {
            info!("  Line number: {}", job_info.line_number);
        }
        Err(e) => {
            info!("  Failed to read line number: {e}");
        }
    }

    // Read step number only
    match client.read_executing_job_info(1, 3).await {
        Ok(job_info) => {
            info!("  Step number: {}", job_info.step_number);
        }
        Err(e) => {
            info!("  Failed to read step number: {e}");
        }
    }

    // Read speed override value only
    match client.read_executing_job_info(1, 4).await {
        Ok(job_info) => {
            info!("  Speed override value: {}", job_info.speed_override_value);
        }
        Err(e) => {
            info!("  Failed to read speed override value: {e}");
        }
    }

    Ok(())
}
