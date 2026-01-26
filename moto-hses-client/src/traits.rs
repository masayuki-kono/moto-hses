//! Trait definitions for HSES client operations
//!
//! This module provides the `HsesClientOps` trait which abstracts all client operations,
//! enabling thread-safe wrappers and mock implementations.

use crate::types::ClientError;
use moto_hses_proto::commands::JobSelectType;
use moto_hses_proto::{
    Alarm, AlarmAttribute, CycleMode, ExecutingJobInfo, Position, Status, StatusData1, StatusData2,
};

/// Trait for HSES client operations
///
/// This trait abstracts all client operations, allowing for:
/// - Thread-safe wrappers (e.g., `SharedHsesClient`)
/// - Mock implementations for testing
/// - Dependency injection in applications
///
/// # Example
///
/// ```ignore
/// use moto_hses_client::{HsesClientOps, SharedHsesClient, HsesClient};
///
/// async fn use_client(client: &impl HsesClientOps) -> Result<(), ClientError> {
///     let status = client.read_status().await?;
///     println!("Status: {:?}", status);
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait HsesClientOps: Send + Sync {
    // ========== Status Operations ==========

    /// Read complete status information (both Data 1 and Data 2)
    async fn read_status(&self) -> Result<Status, ClientError>;

    /// Read status data 1 (basic status information)
    async fn read_status_data1(&self) -> Result<StatusData1, ClientError>;

    /// Read status data 2 (additional status information)
    async fn read_status_data2(&self) -> Result<StatusData2, ClientError>;

    // ========== Position Operations ==========

    /// Read current position for a control group
    async fn read_position(&self, control_group: u8) -> Result<Position, ClientError>;

    // ========== Alarm Operations ==========

    /// Read alarm data
    async fn read_alarm_data(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError>;

    /// Read alarm history
    async fn read_alarm_history(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError>;

    /// Reset alarm
    async fn reset_alarm(&self) -> Result<(), ClientError>;

    /// Cancel error
    async fn cancel_error(&self) -> Result<(), ClientError>;

    // ========== Control Operations ==========

    /// Set HOLD state
    async fn set_hold(&self, enabled: bool) -> Result<(), ClientError>;

    /// Set Servo power state
    async fn set_servo(&self, enabled: bool) -> Result<(), ClientError>;

    /// Set HLOCK state
    async fn set_hlock(&self, enabled: bool) -> Result<(), ClientError>;

    /// Set cycle mode
    async fn set_cycle_mode(&self, mode: CycleMode) -> Result<(), ClientError>;

    // ========== Job Operations ==========

    /// Start job execution
    async fn start_job(&self) -> Result<(), ClientError>;

    /// Select job for execution
    async fn select_job(
        &self,
        select_type: JobSelectType,
        job_name: impl Into<String> + Send,
        line_number: u32,
    ) -> Result<(), ClientError>;

    /// Read executing job information
    async fn read_executing_job_info(
        &self,
        task_type: u16,
        attribute: u8,
    ) -> Result<ExecutingJobInfo, ClientError>;

    /// Read complete executing job information (all attributes)
    async fn read_executing_job_info_complete(
        &self,
        task_type: u16,
    ) -> Result<ExecutingJobInfo, ClientError>;

    // ========== Variable Operations (Typed) ==========

    /// Read an 8-bit unsigned integer variable (B variable)
    async fn read_u8(&self, index: u16) -> Result<u8, ClientError>;

    /// Write an 8-bit unsigned integer variable (B variable)
    async fn write_u8(&self, index: u16, value: u8) -> Result<(), ClientError>;

    /// Read a 16-bit integer variable (I variable)
    async fn read_i16(&self, index: u16) -> Result<i16, ClientError>;

    /// Write a 16-bit integer variable (I variable)
    async fn write_i16(&self, index: u16, value: i16) -> Result<(), ClientError>;

    /// Read a 32-bit integer variable (D variable)
    async fn read_i32(&self, index: u16) -> Result<i32, ClientError>;

    /// Write a 32-bit integer variable (D variable)
    async fn write_i32(&self, index: u16, value: i32) -> Result<(), ClientError>;

    /// Read a 32-bit float variable (R variable)
    async fn read_f32(&self, index: u16) -> Result<f32, ClientError>;

    /// Write a 32-bit float variable (R variable)
    async fn write_f32(&self, index: u16, value: f32) -> Result<(), ClientError>;

    /// Read a string variable (S variable)
    async fn read_string(&self, index: u16) -> Result<String, ClientError>;

    /// Write a string variable (S variable)
    async fn write_string(&self, index: u16, value: String) -> Result<(), ClientError>;

    // ========== Multiple Variable Operations ==========

    /// Read multiple u8 variables (B)
    async fn read_multiple_u8(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError>;

    /// Write multiple u8 variables (B)
    async fn write_multiple_u8(
        &self,
        start_variable_number: u16,
        values: Vec<u8>,
    ) -> Result<(), ClientError>;

    /// Read multiple i16 variables (I)
    async fn read_multiple_i16(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError>;

    /// Write multiple i16 variables (I)
    async fn write_multiple_i16(
        &self,
        start_variable_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError>;

    /// Read multiple i32 variables (D)
    async fn read_multiple_i32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i32>, ClientError>;

    /// Write multiple i32 variables (D)
    async fn write_multiple_i32(
        &self,
        start_variable_number: u16,
        values: Vec<i32>,
    ) -> Result<(), ClientError>;

    /// Read multiple f32 variables (R)
    async fn read_multiple_f32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<f32>, ClientError>;

    /// Write multiple f32 variables (R)
    async fn write_multiple_f32(
        &self,
        start_variable_number: u16,
        values: Vec<f32>,
    ) -> Result<(), ClientError>;

    /// Read multiple string variables (S)
    async fn read_multiple_strings(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<String>, ClientError>;

    /// Write multiple string variables (S)
    async fn write_multiple_strings(
        &self,
        start_variable_number: u16,
        values: Vec<String>,
    ) -> Result<(), ClientError>;

    // ========== I/O Operations ==========

    /// Read single I/O
    async fn read_io(&self, io_number: u16) -> Result<u8, ClientError>;

    /// Write single I/O
    async fn write_io(&self, io_number: u16, value: u8) -> Result<(), ClientError>;

    /// Read multiple I/O data
    async fn read_multiple_io(
        &self,
        start_io_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError>;

    /// Write multiple I/O data
    async fn write_multiple_io(
        &self,
        start_io_number: u16,
        io_data: Vec<u8>,
    ) -> Result<(), ClientError>;

    // ========== Register Operations ==========

    /// Read single register
    async fn read_register(&self, register_number: u16) -> Result<i16, ClientError>;

    /// Write single register
    async fn write_register(&self, register_number: u16, value: i16) -> Result<(), ClientError>;

    /// Read multiple registers
    async fn read_multiple_registers(
        &self,
        start_register_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError>;

    /// Write multiple registers
    async fn write_multiple_registers(
        &self,
        start_register_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError>;

    // ========== File Operations ==========

    /// Get file list from controller
    async fn read_file_list(&self, pattern: &str) -> Result<Vec<String>, ClientError>;

    /// Send file to controller
    async fn send_file(&self, filename: &str, content: &[u8]) -> Result<(), ClientError>;

    /// Receive file from controller
    async fn receive_file(&self, filename: &str) -> Result<String, ClientError>;

    /// Delete file from controller
    async fn delete_file(&self, filename: &str) -> Result<(), ClientError>;
}
