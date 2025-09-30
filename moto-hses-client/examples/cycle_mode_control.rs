use log::info;
use moto_hses_client::HsesClient;
use moto_hses_proto::{CycleMode, ROBOT_CONTROL_PORT};

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

    let controller_addr = format!("{host}:{robot_port}");
    info!("Connecting to controller at {controller_addr}...");
    let client = HsesClient::new(&controller_addr).await?;

    info!("Successfully connected to controller");

    info!("=== 0x84 Command (Cycle Mode Switching) Usage Example ===\n");

    // 1. Test STEP mode
    info!("1. Setting cycle mode to STEP:");
    client.set_cycle_mode(CycleMode::Step).await?;
    info!("✓ STEP mode command sent");

    // 2. Test ONE CYCLE mode
    info!("2. Setting cycle mode to ONE CYCLE:");
    client.set_cycle_mode(CycleMode::OneCycle).await?;
    info!("✓ ONE CYCLE mode command sent");

    // 3. Test CONTINUOUS mode
    info!("3. Setting cycle mode to CONTINUOUS:");
    client.set_cycle_mode(CycleMode::Continuous).await?;
    info!("✓ CONTINUOUS mode command sent");

    // 4. Test mode sequence
    info!("4. Testing mode sequence:");
    let modes = [CycleMode::Step, CycleMode::OneCycle, CycleMode::Continuous];
    for (i, mode) in modes.iter().enumerate() {
        client.set_cycle_mode(*mode).await?;
        info!("  {}. Set to {:?} mode", i + 1, mode);
    }

    // 5. Test rapid mode changes
    info!("5. Testing rapid mode changes:");
    for i in 0..3 {
        client.set_cycle_mode(CycleMode::Step).await?;
        client.set_cycle_mode(CycleMode::OneCycle).await?;
        client.set_cycle_mode(CycleMode::Continuous).await?;
        info!("  Cycle {} completed", i + 1);
    }

    info!("\n✓ All cycle mode operations completed successfully");
    info!("Example usage:");
    info!("  cargo run --example cycle_mode_control");
    info!("  cargo run --example cycle_mode_control 192.168.1.100 10040");

    Ok(())
}
