// Test logging utilities for integration tests

use log::{debug, error, info};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test logging to write to logs directory
pub fn init_test_logging() {
    INIT.call_once(|| {
        // Create logs directory if it doesn't exist
        std::fs::create_dir_all("logs").unwrap_or_else(|e| {
            eprintln!("Failed to create logs directory: {}", e);
        });

        // Configure env_logger to write to file with detailed logging
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug) // Enable debug level logging
            .format(|buf, record| {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                writeln!(
                    buf,
                    "{} [{}] {}: {}",
                    timestamp,
                    record.level(),
                    record.target(),
                    record.args()
                )
            })
            .target(env_logger::Target::Pipe(Box::new(
                OpenOptions::new()
                    .create(true)
                    .write(true) // Use write mode to clear the file
                    .truncate(true) // Truncate the file to clear previous content
                    .open("logs/integration_tests.log")
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to open log file: {}", e);
                        std::process::exit(1);
                    }),
            )))
            .try_init();
    });
}

/// Log test session start
pub fn log_test_session_start() {
    info!("=== Integration Test Session Started ===");
    info!("Timestamp: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
}

/// Log test start
pub fn log_test_start(test_name: &str) {
    info!("=== Starting test: {} ===", test_name);
}

/// Log test completion
pub fn log_test_completion(test_name: &str, success: bool) {
    if success {
        info!("=== Test completed successfully: {} ===", test_name);
    } else {
        error!("=== Test failed: {} ===", test_name);
    }
}

/// Log mock server startup
pub fn log_mock_server_startup(host: &str, port: u16) {
    debug!("Starting mock server on {}:{}", host, port);
}

/// Log mock server startup failure
pub fn log_mock_server_startup_failure(host: &str, port: u16, error: &str) {
    error!("Failed to start mock server on {}:{} - {}", host, port, error);
}

/// Log client connection attempt
pub fn log_client_connection_attempt(host: &str, port: u16) {
    info!("Attempting to connect client to {}:{}", host, port);
}

/// Log client connection result
pub fn log_client_connection_result(host: &str, port: u16, success: bool) {
    if success {
        info!("Client connected successfully to {}:{}", host, port);
    } else {
        error!("Client failed to connect to {}:{}", host, port);
    }
}

/// Macro to create a test with automatic logging
#[macro_export]
macro_rules! test_with_logging {
    ($test_name:ident, $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            $crate::common::test_logging::init_test_logging();
            $crate::common::test_logging::log_test_start(stringify!($test_name));

            $test_body

            $crate::common::test_logging::log_test_completion(stringify!($test_name), true);
        }
    };
}
