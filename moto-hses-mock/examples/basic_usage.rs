//! Basic usage example for mock server

use moto_hses_mock::{MockServer, MockConfig};
use moto_hses_proto as proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting mock HSES server...");
    
    // Create server configuration
    let config = MockConfig {
        bind_addr: "127.0.0.1:10040".parse().unwrap(),
        ..Default::default()
    };
    
    // Create and start server
    let server = MockServer::new(config).await?;
    let addr = server.local_addr()?;
    
    println!("Mock server listening on {}", addr);
    println!("Supported commands:");
    println!("  0x70 - Alarm data reading");
    println!("  0x72 - Status information reading");
    println!("  0x75 - Robot position data reading");
    println!("  0x78 - I/O data reading/writing");
    println!("  0x79 - Register data reading/writing");
    println!("  0x7a - Byte variable reading/writing");
    println!("  0x7b - Integer variable reading/writing");
    println!("  0x7d - Real variable reading/writing");
    println!("  0x7f - Position variable reading/writing");
    println!("  0x82 - Alarm reset/error cancel");
    println!("  0x83 - HOLD/servo ON/OFF");
    println!("  0x86 - Job start");
    println!("  0x87 - Job select");
    
    // Add some test alarms
    server.add_test_alarm(proto::alarm::test_alarms::servo_error()).await;
    server.add_test_alarm(proto::alarm::test_alarms::emergency_stop()).await;
    
    // Set some test variables
    server.set_variable(10, vec![0x42, 0x00, 0x00, 0x00]).await; // D010 = 66
    server.set_variable(20, vec![0x00, 0x00, 0x48, 0x42]).await; // D020 = 50.0
    
    // Set some I/O states
    server.set_io_state(1, true).await;   // Input 1 = ON
    server.set_io_state(1001, false).await; // Output 1 = OFF
    
    println!("Test data configured:");
    println!("  - 2 test alarms added");
    println!("  - D010 = 66, D020 = 50.0");
    println!("  - Input 1 = ON, Output 1 = OFF");
    
    // Run the server
    println!("Server running. Press Ctrl+C to stop.");
    server.run().await?;
    
    Ok(())
}
