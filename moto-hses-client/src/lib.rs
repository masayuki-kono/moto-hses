//! moto-hses-client - HSES (High Speed Ethernet Server) client implementation
//!
//! This crate provides an async UDP client for communicating with Yaskawa robot controllers
//! using the High-Speed Ethernet Server (HSES) protocol.
//!
//! # Thread Safety
//!
//! The crate provides two ways to use the client:
//!
//! - [`HsesClient`]: The basic client, suitable for single-task usage
//! - [`SharedHsesClient`]: A thread-safe wrapper that can be shared across multiple tasks
//!
//! Both implement the [`HsesClientOps`] trait, allowing generic code to work with either.
//!
//! # Example
//!
//! ```ignore
//! use moto_hses_client::{HsesClient, SharedHsesClient, HsesClientOps};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a basic client
//!     let client = HsesClient::new("192.168.1.1:10040").await?;
//!
//!     // Wrap it for thread-safe access
//!     let shared_client = SharedHsesClient::new(client);
//!
//!     // Use from multiple tasks
//!     let client1 = shared_client.clone();
//!     let client2 = shared_client.clone();
//!
//!     let handle1 = tokio::spawn(async move {
//!         client1.read_status().await
//!     });
//!
//!     let handle2 = tokio::spawn(async move {
//!         client2.read_position(0).await
//!     });
//!
//!     let (status, position) = tokio::try_join!(handle1, handle2)?;
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate log;

pub mod connection;
pub mod convenience;
mod impl_traits;
pub mod protocol;
pub mod shared;
pub mod traits;
pub mod types;

// Re-export main types for convenience
pub use shared::SharedHsesClient;
pub use traits::HsesClientOps;
pub use types::{ClientConfig, ClientError, HsesClient};

// Re-export protocol types that are commonly used
pub use moto_hses_proto::{Alarm, ExecutingJobInfo, HsesPayload, Position, Status, TextEncoding};
