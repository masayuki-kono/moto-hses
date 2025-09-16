//! Basic usage example for mock server

use log::info;
use moto_hses_mock::{MockConfig, MockServer};
use moto_hses_proto as proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    info!("Starting mock HSES server...");

    // Create server configuration
    let config = MockConfig::new(
        "127.0.0.1",
        moto_hses_proto::ROBOT_CONTROL_PORT,
        moto_hses_proto::FILE_CONTROL_PORT,
    );

    // Create and start server
    let server = MockServer::new(config).await?;
    let addr = server.local_addr()?;

    info!("Mock server listening on {addr}");
    info!("Supported commands:");
    info!("  0x70 - Alarm data reading");
    info!("  0x72 - Status information reading");
    info!("  0x75 - Robot position data reading");
    info!("  0x78 - I/O data reading/writing");
    info!("  0x79 - Register data reading/writing");
    info!("  0x7a - Byte variable reading/writing");
    info!("  0x7b - Integer variable reading/writing");
    info!("  0x7d - Real variable reading/writing");
    info!("  0x7f - Position variable reading/writing");
    info!("  0x82 - Alarm reset/error cancel");
    info!("  0x83 - HOLD/servo ON/OFF");
    info!("  0x86 - Job start");
    info!("  0x87 - Job select");

    // Add some test alarms
    server.add_test_alarm(proto::alarm::test_alarms::servo_error()).await;
    server.add_test_alarm(proto::alarm::test_alarms::emergency_stop()).await;

    // Set some test variables
    server.set_variable(10, vec![0x42, 0x00, 0x00, 0x00]).await; // D010 = 66
    server.set_variable(20, vec![0x00, 0x00, 0x48, 0x42]).await; // D020 = 50.0

    // Set some I/O states
    server.set_io_state(1, true).await; // Input 1 = ON
    server.set_io_state(1001, false).await; // Output 1 = OFF

    info!("Test data configured:");
    info!("  - 4 test alarms added (Instance 1-4)");
    info!("  - D010 = 66, D020 = 50.0");
    info!("  - Input 1 = ON, Output 1 = OFF");

    // Run the server
    info!("Server running. Press Ctrl+C to stop.");
    server.run().await?;

    Ok(())
}
