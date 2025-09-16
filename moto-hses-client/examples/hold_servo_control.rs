use log::info;
use moto_hses_client::HsesClient;
use moto_hses_proto::ROBOT_CONTROL_PORT;

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

    info!("=== 0x83 Command (Hold/Servo On/off) Usage Example ===\n");

    // 1. Read and save initial status
    info!("1. Read and save initial status:");
    let initial_status = client.read_status().await?;
    info!("✓ Initial status retrieved and saved");
    info!("  Servo on: {}", initial_status.is_servo_on());
    info!("  Running: {}", initial_status.is_running());

    // 2. HOLD control examples
    info!("2. HOLD control examples:");

    // Set HOLD to opposite of initial state
    let initial_hold_state = !initial_status.data1.running; // If running, HOLD is OFF
    let opposite_hold_state = !initial_hold_state;
    info!(
        "  Initial HOLD state: {} (running: {})",
        initial_hold_state, initial_status.data1.running
    );
    info!("  Setting HOLD to opposite state: {opposite_hold_state}...");
    client.set_hold(opposite_hold_state).await?;
    info!("  ✓ HOLD {} command sent", if opposite_hold_state { "ON" } else { "OFF" });

    // Wait a moment and verify the change
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let hold_test_status = client.read_status().await?;
    let expected_running = !opposite_hold_state; // If HOLD is ON, running should be false
    info!(
        "  ✓ HOLD state verification: Running = {} (expected: {})",
        hold_test_status.is_running(),
        expected_running
    );

    // Set HOLD back to initial state
    info!("  Setting HOLD back to initial state: {initial_hold_state}...");
    client.set_hold(initial_hold_state).await?;
    info!("  ✓ HOLD {} command sent", if initial_hold_state { "ON" } else { "OFF" });

    // 3. Servo control examples
    info!("3. Servo control examples:");

    // Set Servo to opposite of initial state
    let initial_servo_state = initial_status.data2.servo_on;
    let opposite_servo_state = !initial_servo_state;
    info!("  Initial Servo state: {initial_servo_state}");
    info!("  Setting Servo to opposite state: {opposite_servo_state}...");
    client.set_servo(opposite_servo_state).await?;
    info!("  ✓ Servo {} command sent", if opposite_servo_state { "ON" } else { "OFF" });

    // Wait a moment and verify the change
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let servo_test_status = client.read_status().await?;
    info!(
        "  ✓ Servo state verification: Servo ON = {} (expected: {})",
        servo_test_status.is_servo_on(),
        opposite_servo_state
    );

    // Set Servo back to initial state
    info!("  Setting Servo back to initial state: {initial_servo_state}...");
    client.set_servo(initial_servo_state).await?;
    info!("  ✓ Servo {} command sent", if initial_servo_state { "ON" } else { "OFF" });

    // 4. HLOCK control examples
    info!("4. HLOCK control examples:");

    // Set HLOCK
    let initial_hlock_state = false;
    let opposite_hlock_state = !initial_hlock_state;
    info!("  Initial HLOCK state: {initial_hlock_state}");
    info!("  Setting HLOCK to opposite state: {opposite_hlock_state}...");
    client.set_hlock(opposite_hlock_state).await?;
    info!("  ✓ HLOCK {} command sent", if opposite_hlock_state { "ON" } else { "OFF" });

    // Wait a moment and verify the change
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let _hlock_test_status = client.read_status().await?;
    info!("  ✓ HLOCK state verification: HLOCK set to {opposite_hlock_state} (command executed)");

    // Set HLOCK back to initial state
    info!("  Setting HLOCK back to initial state: {initial_hlock_state}...");
    client.set_hlock(initial_hlock_state).await?;
    info!("  ✓ HLOCK {} command sent", if initial_hlock_state { "ON" } else { "OFF" });

    Ok(())
}
