//! Implementation of `HsesClientOps` trait for `HsesClient`

use crate::traits::HsesClientOps;
use crate::types::{ClientError, HsesClient};
use moto_hses_proto::commands::JobSelectType;
use moto_hses_proto::{
    Alarm, AlarmAttribute, CycleMode, ExecutingJobInfo, Position, Status, StatusData1, StatusData2,
};

#[async_trait::async_trait]
impl HsesClientOps for HsesClient {
    // ========== Status Operations ==========

    async fn read_status(&self) -> Result<Status, ClientError> {
        Self::read_status(self).await
    }

    async fn read_status_data1(&self) -> Result<StatusData1, ClientError> {
        Self::read_status_data1(self).await
    }

    async fn read_status_data2(&self) -> Result<StatusData2, ClientError> {
        Self::read_status_data2(self).await
    }

    // ========== Position Operations ==========

    async fn read_position(&self, control_group: u8) -> Result<Position, ClientError> {
        Self::read_position(self, control_group).await
    }

    // ========== Alarm Operations ==========

    async fn read_alarm_data(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError> {
        Self::read_alarm_data(self, instance, attribute).await
    }

    async fn read_alarm_history(
        &self,
        instance: u16,
        attribute: AlarmAttribute,
    ) -> Result<Alarm, ClientError> {
        Self::read_alarm_history(self, instance, attribute).await
    }

    async fn reset_alarm(&self) -> Result<(), ClientError> {
        Self::reset_alarm(self).await
    }

    async fn cancel_error(&self) -> Result<(), ClientError> {
        Self::cancel_error(self).await
    }

    // ========== Control Operations ==========

    async fn set_hold(&self, enabled: bool) -> Result<(), ClientError> {
        Self::set_hold(self, enabled).await
    }

    async fn set_servo(&self, enabled: bool) -> Result<(), ClientError> {
        Self::set_servo(self, enabled).await
    }

    async fn set_hlock(&self, enabled: bool) -> Result<(), ClientError> {
        Self::set_hlock(self, enabled).await
    }

    async fn set_cycle_mode(&self, mode: CycleMode) -> Result<(), ClientError> {
        Self::set_cycle_mode(self, mode).await
    }

    // ========== Job Operations ==========

    async fn start_job(&self) -> Result<(), ClientError> {
        Self::start_job(self).await
    }

    async fn select_job(
        &self,
        select_type: JobSelectType,
        job_name: impl Into<String> + Send,
        line_number: u32,
    ) -> Result<(), ClientError> {
        Self::select_job(self, select_type, job_name, line_number).await
    }

    async fn read_executing_job_info(
        &self,
        task_type: u16,
        attribute: u8,
    ) -> Result<ExecutingJobInfo, ClientError> {
        Self::read_executing_job_info(self, task_type, attribute).await
    }

    async fn read_executing_job_info_complete(
        &self,
        task_type: u16,
    ) -> Result<ExecutingJobInfo, ClientError> {
        Self::read_executing_job_info_complete(self, task_type).await
    }

    // ========== Variable Operations (Typed) ==========

    async fn read_u8(&self, index: u16) -> Result<u8, ClientError> {
        Self::read_u8(self, index).await
    }

    async fn write_u8(&self, index: u16, value: u8) -> Result<(), ClientError> {
        Self::write_u8(self, index, value).await
    }

    async fn read_i16(&self, index: u16) -> Result<i16, ClientError> {
        Self::read_i16(self, index).await
    }

    async fn write_i16(&self, index: u16, value: i16) -> Result<(), ClientError> {
        Self::write_i16(self, index, value).await
    }

    async fn read_i32(&self, index: u16) -> Result<i32, ClientError> {
        Self::read_i32(self, index).await
    }

    async fn write_i32(&self, index: u16, value: i32) -> Result<(), ClientError> {
        Self::write_i32(self, index, value).await
    }

    async fn read_f32(&self, index: u16) -> Result<f32, ClientError> {
        Self::read_f32(self, index).await
    }

    async fn write_f32(&self, index: u16, value: f32) -> Result<(), ClientError> {
        Self::write_f32(self, index, value).await
    }

    async fn read_string(&self, index: u16) -> Result<String, ClientError> {
        Self::read_string(self, index).await
    }

    async fn write_string(&self, index: u16, value: String) -> Result<(), ClientError> {
        Self::write_string(self, index, value).await
    }

    // ========== Multiple Variable Operations ==========

    async fn read_multiple_u8(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        Self::read_multiple_u8(self, start_variable_number, count).await
    }

    async fn write_multiple_u8(
        &self,
        start_variable_number: u16,
        values: Vec<u8>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_u8(self, start_variable_number, values).await
    }

    async fn read_multiple_i16(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        Self::read_multiple_i16(self, start_variable_number, count).await
    }

    async fn write_multiple_i16(
        &self,
        start_variable_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_i16(self, start_variable_number, values).await
    }

    async fn read_multiple_i32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i32>, ClientError> {
        Self::read_multiple_i32(self, start_variable_number, count).await
    }

    async fn write_multiple_i32(
        &self,
        start_variable_number: u16,
        values: Vec<i32>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_i32(self, start_variable_number, values).await
    }

    async fn read_multiple_f32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<f32>, ClientError> {
        Self::read_multiple_f32(self, start_variable_number, count).await
    }

    async fn write_multiple_f32(
        &self,
        start_variable_number: u16,
        values: Vec<f32>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_f32(self, start_variable_number, values).await
    }

    async fn read_multiple_strings(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<String>, ClientError> {
        Self::read_multiple_strings(self, start_variable_number, count).await
    }

    async fn write_multiple_strings(
        &self,
        start_variable_number: u16,
        values: Vec<String>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_strings(self, start_variable_number, values).await
    }

    // ========== I/O Operations ==========

    async fn read_io(&self, io_number: u16) -> Result<u8, ClientError> {
        Self::read_io(self, io_number).await
    }

    async fn write_io(&self, io_number: u16, value: u8) -> Result<(), ClientError> {
        Self::write_io(self, io_number, value).await
    }

    async fn read_multiple_io(
        &self,
        start_io_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        Self::read_multiple_io(self, start_io_number, count).await
    }

    async fn write_multiple_io(
        &self,
        start_io_number: u16,
        io_data: Vec<u8>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_io(self, start_io_number, io_data).await
    }

    // ========== Register Operations ==========

    async fn read_register(&self, register_number: u16) -> Result<i16, ClientError> {
        Self::read_register(self, register_number).await
    }

    async fn write_register(&self, register_number: u16, value: i16) -> Result<(), ClientError> {
        Self::write_register(self, register_number, value).await
    }

    async fn read_multiple_registers(
        &self,
        start_register_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        Self::read_multiple_registers(self, start_register_number, count).await
    }

    async fn write_multiple_registers(
        &self,
        start_register_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        Self::write_multiple_registers(self, start_register_number, values).await
    }

    // ========== File Operations ==========

    async fn read_file_list(&self, pattern: &str) -> Result<Vec<String>, ClientError> {
        Self::read_file_list(self, pattern).await
    }

    async fn send_file(&self, filename: &str, content: &[u8]) -> Result<(), ClientError> {
        Self::send_file(self, filename, content).await
    }

    async fn receive_file(&self, filename: &str) -> Result<String, ClientError> {
        Self::receive_file(self, filename).await
    }

    async fn delete_file(&self, filename: &str) -> Result<(), ClientError> {
        Self::delete_file(self, filename).await
    }
}
