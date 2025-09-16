//! Register operations example for 0x79 register command

use log::info;
use moto_hses_client::HsesClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Register operations example - 0x79 register command");

    // Create client
    let client =
        HsesClient::new(&format!("127.0.0.1:{}", moto_hses_proto::ROBOT_CONTROL_PORT)).await?;
    info!("Connected to mock server");

    // Test reading a register
    info!("Reading register 0...");
    let value = client.read_register(0).await?;
    info!("Register 0 value: {value}");

    // Test writing to a register
    info!("Writing value 42 to register 0...");
    client.write_register(0, 42).await?;
    info!("Write successful");

    // Test reading the register again to verify the write
    info!("Reading register 0 again...");
    let new_value = client.read_register(0).await?;
    info!("Register 0 value after write: {new_value}");

    // Test with different register numbers
    for i in 1..5 {
        let test_value = i16::try_from(i * 10).map_err(|_| "Invalid value for i16")?;
        info!("Writing {test_value} to register {i}...");
        client.write_register(i, test_value).await?;

        let read_value = client.read_register(i).await?;
        info!("Register {i} value: {read_value}");

        if read_value == test_value {
            info!("✓ Register {i} test passed");
        } else {
            info!("✗ Register {i} test failed: expected {test_value}, got {read_value}");
        }
    }

    // Test error handling
    info!("\n--- Error Handling Tests ---");

    // Test invalid register number
    match client.read_register(65535).await {
        Ok(value) => {
            info!("✗ Invalid register number succeeded unexpectedly: {value}");
        }
        Err(e) => {
            info!("✓ Invalid register number correctly failed: {e}");
        }
    }

    // Test invalid register number for write
    match client.write_register(65535, 42).await {
        Ok(()) => {
            info!("✗ Invalid register number write succeeded unexpectedly");
        }
        Err(e) => {
            info!("✓ Invalid register number write correctly failed: {e}");
        }
    }

    info!("Register operations example completed!");
    Ok(())
}
