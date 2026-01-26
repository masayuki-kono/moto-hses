//! Thread-safe wrapper for HSES client
//!
//! This module provides `SharedHsesClient`, a thread-safe wrapper around `HsesClient`
//! that can be safely shared across multiple tasks and threads.

use crate::traits::HsesClientOps;
use crate::types::{ClientError, HsesClient};
use moto_hses_proto::commands::JobSelectType;
use moto_hses_proto::{
    Alarm, AlarmAttribute, CycleMode, ExecutingJobInfo, Position, Status, StatusData1, StatusData2,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A thread-safe wrapper around `HsesClient`
///
/// `SharedHsesClient` wraps an `HsesClient` in `Arc<Mutex<_>>`, allowing it to be
/// safely shared and used across multiple async tasks and threads.
///
/// # Example
///
/// ```ignore
/// use moto_hses_client::{HsesClient, SharedHsesClient, HsesClientOps};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = HsesClient::new("192.168.1.1:10040").await?;
///     let shared_client = SharedHsesClient::new(client);
///
///     // Clone for use in multiple tasks
///     let client1 = shared_client.clone();
///     let client2 = shared_client.clone();
///
///     let handle1 = tokio::spawn(async move {
///         client1.read_status().await
///     });
///
///     let handle2 = tokio::spawn(async move {
///         client2.read_position(0).await
///     });
///
///     let (status, position) = tokio::try_join!(handle1, handle2)?;
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct SharedHsesClient {
    client: Arc<Mutex<HsesClient>>,
}

impl SharedHsesClient {
    /// Create a new thread-safe wrapper around an `HsesClient`
    #[must_use]
    pub fn new(client: HsesClient) -> Self {
        Self { client: Arc::new(Mutex::new(client)) }
    }

    /// Create a new `SharedHsesClient` from an existing `Arc<Mutex<HsesClient>>`
    ///
    /// This is useful when you already have an `Arc<Mutex<HsesClient>>` and want
    /// to use the `HsesClientOps` trait methods.
    #[must_use]
    pub const fn from_arc(client: Arc<Mutex<HsesClient>>) -> Self {
        Self { client }
    }

    /// Get a reference to the inner `Arc<Mutex<HsesClient>>`
    #[must_use]
    pub const fn inner(&self) -> &Arc<Mutex<HsesClient>> {
        &self.client
    }
}

#[async_trait::async_trait]
impl HsesClientOps for SharedHsesClient {
    // ========== Status Operations ==========

    async fn read_status(&self) -> Result<Status, ClientError> {
        let client = self.client.lock().await;
        client.read_status().await
    }

    async fn read_status_data1(&self) -> Result<StatusData1, ClientError> {
        let client = self.client.lock().await;
        client.read_status_data1().await
    }

    async fn read_status_data2(&self) -> Result<StatusData2, ClientError> {
        let client = self.client.lock().await;
        client.read_status_data2().await
    }

    // ========== Position Operations ==========

    async fn read_position(&self, control_group: u8) -> Result<Position, ClientError> {
        let client = self.client.lock().await;
        client.read_position(control_group).await
    }

    // ========== Alarm Operations ==========

    async fn read_alarm_data(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError> {
        let client = self.client.lock().await;
        client.read_alarm_data(instance, attribute).await
    }

    async fn read_alarm_history(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError> {
        let client = self.client.lock().await;
        client.read_alarm_history(instance, attribute).await
    }

    async fn reset_alarm(&self) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.reset_alarm().await
    }

    async fn cancel_error(&self) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.cancel_error().await
    }

    // ========== Control Operations ==========

    async fn set_hold(&self, enabled: bool) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.set_hold(enabled).await
    }

    async fn set_servo(&self, enabled: bool) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.set_servo(enabled).await
    }

    async fn set_hlock(&self, enabled: bool) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.set_hlock(enabled).await
    }

    async fn set_cycle_mode(&self, mode: CycleMode) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.set_cycle_mode(mode).await
    }

    // ========== Job Operations ==========

    async fn start_job(&self) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.start_job().await
    }

    async fn select_job(
        &self,
        select_type: JobSelectType,
        job_name: impl Into<String> + Send,
        line_number: u32,
    ) -> Result<(), ClientError> {
        let job_name = job_name.into();
        let client = self.client.lock().await;
        client.select_job(select_type, job_name, line_number).await
    }

    async fn read_executing_job_info(
        &self,
        task_type: u16,
        attribute: u8,
    ) -> Result<ExecutingJobInfo, ClientError> {
        let client = self.client.lock().await;
        client.read_executing_job_info(task_type, attribute).await
    }

    async fn read_executing_job_info_complete(
        &self,
        task_type: u16,
    ) -> Result<ExecutingJobInfo, ClientError> {
        let client = self.client.lock().await;
        client.read_executing_job_info_complete(task_type).await
    }

    // ========== Variable Operations (Typed) ==========

    async fn read_u8(&self, index: u16) -> Result<u8, ClientError> {
        let client = self.client.lock().await;
        client.read_u8(index).await
    }

    async fn write_u8(&self, index: u16, value: u8) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_u8(index, value).await
    }

    async fn read_i16(&self, index: u16) -> Result<i16, ClientError> {
        let client = self.client.lock().await;
        client.read_i16(index).await
    }

    async fn write_i16(&self, index: u16, value: i16) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_i16(index, value).await
    }

    async fn read_i32(&self, index: u16) -> Result<i32, ClientError> {
        let client = self.client.lock().await;
        client.read_i32(index).await
    }

    async fn write_i32(&self, index: u16, value: i32) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_i32(index, value).await
    }

    async fn read_f32(&self, index: u16) -> Result<f32, ClientError> {
        let client = self.client.lock().await;
        client.read_f32(index).await
    }

    async fn write_f32(&self, index: u16, value: f32) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_f32(index, value).await
    }

    async fn read_string(&self, index: u16) -> Result<String, ClientError> {
        let client = self.client.lock().await;
        client.read_string(index).await
    }

    async fn write_string(&self, index: u16, value: String) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_string(index, value).await
    }

    // ========== Multiple Variable Operations ==========

    async fn read_multiple_u8(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_u8(start_variable_number, count).await
    }

    async fn write_multiple_u8(
        &self,
        start_variable_number: u16,
        values: Vec<u8>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_u8(start_variable_number, values).await
    }

    async fn read_multiple_i16(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_i16(start_variable_number, count).await
    }

    async fn write_multiple_i16(
        &self,
        start_variable_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_i16(start_variable_number, values).await
    }

    async fn read_multiple_i32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i32>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_i32(start_variable_number, count).await
    }

    async fn write_multiple_i32(
        &self,
        start_variable_number: u16,
        values: Vec<i32>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_i32(start_variable_number, values).await
    }

    async fn read_multiple_f32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<f32>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_f32(start_variable_number, count).await
    }

    async fn write_multiple_f32(
        &self,
        start_variable_number: u16,
        values: Vec<f32>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_f32(start_variable_number, values).await
    }

    async fn read_multiple_strings(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<String>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_strings(start_variable_number, count).await
    }

    async fn write_multiple_strings(
        &self,
        start_variable_number: u16,
        values: Vec<String>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_strings(start_variable_number, values).await
    }

    // ========== I/O Operations ==========

    async fn read_io(&self, io_number: u16) -> Result<u8, ClientError> {
        let client = self.client.lock().await;
        client.read_io(io_number).await
    }

    async fn write_io(&self, io_number: u16, value: u8) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_io(io_number, value).await
    }

    async fn read_multiple_io(
        &self,
        start_io_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_io(start_io_number, count).await
    }

    async fn write_multiple_io(
        &self,
        start_io_number: u16,
        io_data: Vec<u8>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_io(start_io_number, io_data).await
    }

    // ========== Register Operations ==========

    async fn read_register(&self, register_number: u16) -> Result<i16, ClientError> {
        let client = self.client.lock().await;
        client.read_register(register_number).await
    }

    async fn write_register(&self, register_number: u16, value: i16) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_register(register_number, value).await
    }

    async fn read_multiple_registers(
        &self,
        start_register_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        let client = self.client.lock().await;
        client.read_multiple_registers(start_register_number, count).await
    }

    async fn write_multiple_registers(
        &self,
        start_register_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.write_multiple_registers(start_register_number, values).await
    }

    // ========== File Operations ==========

    async fn read_file_list(&self, pattern: &str) -> Result<Vec<String>, ClientError> {
        let client = self.client.lock().await;
        client.read_file_list(pattern).await
    }

    async fn send_file(&self, filename: &str, content: &[u8]) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.send_file(filename, content).await
    }

    async fn receive_file(&self, filename: &str) -> Result<String, ClientError> {
        let client = self.client.lock().await;
        client.receive_file(filename).await
    }

    async fn delete_file(&self, filename: &str) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        client.delete_file(filename).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that SharedHsesClient is Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_shared_client_is_send_sync() {
        assert_send_sync::<SharedHsesClient>();
    }

    #[test]
    fn test_shared_client_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<SharedHsesClient>();
    }
}
