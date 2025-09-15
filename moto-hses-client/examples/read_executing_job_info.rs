//! Example: Read executing job information using 0x73 command

use moto_hses_client::HsesClient;
use moto_hses_proto::ROBOT_CONTROL_PORT;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let (host, robot_port) = match args.as_slice() {
        [_, host, robot_port] => {
            // Format: [host] [robot_port]
            let robot_port: u16 =
                robot_port.parse().map_err(|_| format!("Invalid robot port: {}", robot_port))?;

            (host.to_string(), robot_port)
        }
        _ => {
            // Default: 127.0.0.1:DEFAULT_PORT
            ("127.0.0.1".to_string(), ROBOT_CONTROL_PORT)
        }
    };

    let controller_addr = format!("{}:{}", host, robot_port);
    println!("Connecting to controller at {}...", controller_addr);
    let client = HsesClient::new(&controller_addr).await?;

    println!("Reading executing job information...");

    // Read complete executing job information (all attributes)
    match client.read_executing_job_info_complete(1).await {
        Ok(job_info) => {
            println!("Complete job information:");
            println!("  Job name: {}", job_info.job_name);
            println!("  Line number: {}", job_info.line_number);
            println!("  Step number: {}", job_info.step_number);
            println!("  Speed override value: {}", job_info.speed_override_value);
        }
        Err(e) => {
            println!("Failed to read complete job information: {}", e);
        }
    }

    // Read specific attributes
    println!("\nReading specific attributes:");

    // Read job name only
    match client.read_executing_job_info(1, 1).await {
        Ok(job_info) => {
            println!("  Job name: {}", job_info.job_name);
        }
        Err(e) => {
            println!("  Failed to read job name: {}", e);
        }
    }

    // Read line number only
    match client.read_executing_job_info(1, 2).await {
        Ok(job_info) => {
            println!("  Line number: {}", job_info.line_number);
        }
        Err(e) => {
            println!("  Failed to read line number: {}", e);
        }
    }

    // Read step number only
    match client.read_executing_job_info(1, 3).await {
        Ok(job_info) => {
            println!("  Step number: {}", job_info.step_number);
        }
        Err(e) => {
            println!("  Failed to read step number: {}", e);
        }
    }

    // Read speed override value only
    match client.read_executing_job_info(1, 4).await {
        Ok(job_info) => {
            println!("  Speed override value: {}", job_info.speed_override_value);
        }
        Err(e) => {
            println!("  Failed to read speed override value: {}", e);
        }
    }

    // Test different task types
    println!("\nTesting different task types:");

    for task_type in 1..=6 {
        match client.read_executing_job_info(task_type, 1).await {
            Ok(job_info) => {
                let task_name = match task_type {
                    1 => "Master Task",
                    2 => "Sub Task 1",
                    3 => "Sub Task 2",
                    4 => "Sub Task 3",
                    5 => "Sub Task 4",
                    6 => "Sub Task 5",
                    _ => "Unknown",
                };
                println!("  {} ({}): {}", task_name, task_type, job_info.job_name);
            }
            Err(e) => {
                println!("  Task {}: Error - {}", task_type, e);
            }
        }
    }

    Ok(())
}
