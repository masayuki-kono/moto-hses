//! Register operations example for 0x79 register command

use moto_hses_client::HsesClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Register operations example - 0x79 register command");

    // Create client
    let client = HsesClient::new("127.0.0.1:10040").await?;
    println!("Connected to mock server");

    // Test reading a register
    println!("Reading register 0...");
    let value = client.read_register(0).await?;
    println!("Register 0 value: {}", value);

    // Test writing to a register
    println!("Writing value 42 to register 0...");
    client.write_register(0, 42).await?;
    println!("Write successful");

    // Test reading the register again to verify the write
    println!("Reading register 0 again...");
    let new_value = client.read_register(0).await?;
    println!("Register 0 value after write: {}", new_value);

    // Test with different register numbers
    for i in 1..5 {
        let test_value = (i * 10) as i16;
        println!("Writing {} to register {}...", test_value, i);
        client.write_register(i, test_value).await?;

        let read_value = client.read_register(i).await?;
        println!("Register {} value: {}", i, read_value);

        if read_value == test_value {
            println!("✓ Register {} test passed", i);
        } else {
            println!(
                "✗ Register {} test failed: expected {}, got {}",
                i, test_value, read_value
            );
        }
    }

    println!("Register operations example completed!");
    Ok(())
}
