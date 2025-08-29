//! Tests for HSES client

use crate::types::{ClientConfig, ClientError};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout.as_millis(), 300);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_delay.as_millis(), 100);
        assert_eq!(config.buffer_size, 8192);
    }

    #[test]
    fn test_client_error_display() {
        let error = ClientError::TimeoutError("test timeout".to_string());
        assert_eq!(error.to_string(), "Timeout error: test timeout");
        
        let error = ClientError::SystemError("test error".to_string());
        assert_eq!(error.to_string(), "System error: test error");
    }
}
