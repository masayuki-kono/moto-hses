//! Mock server state management

use moto_hses_proto as proto;
use proto::commands::alarm::AlarmCategory;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Selected job information
#[derive(Debug, Clone)]
pub struct SelectedJobInfo {
    pub job_name: String,
    pub line_number: u32,
    pub select_type: u16, // Instance value
}

/// Mock server state
#[derive(Debug, Clone)]
pub struct MockState {
    pub text_encoding: proto::TextEncoding,
    pub status: proto::Status,
    pub position: proto::Position,
    pub variables: HashMap<u16, Vec<u8>>,
    pub io_states: HashMap<u16, bool>,
    pub registers: HashMap<u16, i16>,
    pub alarms: Vec<proto::Alarm>,
    pub alarm_history: AlarmHistory,
    pub executing_job: Option<proto::ExecutingJobInfo>,
    pub selected_job: Option<SelectedJobInfo>,
    pub servo_on: bool,
    pub hold_state: bool,
    pub hlock_state: bool,
    pub cycle_mode: proto::CycleMode,
    pub files: HashMap<String, Vec<u8>>,
}

/// Alarm history organized by categories
#[derive(Debug, Clone, Default)]
pub struct AlarmHistory {
    pub major_failure: Vec<proto::Alarm>,     // 1-100
    pub monitor_alarm: Vec<proto::Alarm>,     // 1001-1100
    pub user_alarm_system: Vec<proto::Alarm>, // 2001-2100
    pub user_alarm_user: Vec<proto::Alarm>,   // 3001-3100
    pub offline_alarm: Vec<proto::Alarm>,     // 4001-4100
}

impl AlarmHistory {
    /// Get alarm by category and index
    #[must_use]
    pub fn get_alarm(&self, category: AlarmCategory, index: usize) -> Option<&proto::Alarm> {
        match category {
            AlarmCategory::MajorFailure => self.major_failure.get(index),
            AlarmCategory::MonitorAlarm => self.monitor_alarm.get(index),
            AlarmCategory::UserAlarmSystem => self.user_alarm_system.get(index),
            AlarmCategory::UserAlarmUser => self.user_alarm_user.get(index),
            AlarmCategory::OfflineAlarm => self.offline_alarm.get(index),
            AlarmCategory::Invalid => None,
        }
    }

    /// Add alarm to specific category
    pub fn add_alarm(&mut self, category: AlarmCategory, alarm: proto::Alarm) {
        match category {
            AlarmCategory::MajorFailure => {
                if self.major_failure.len() < 100 {
                    self.major_failure.push(alarm);
                }
            }
            AlarmCategory::MonitorAlarm => {
                if self.monitor_alarm.len() < 100 {
                    self.monitor_alarm.push(alarm);
                }
            }
            AlarmCategory::UserAlarmSystem => {
                if self.user_alarm_system.len() < 100 {
                    self.user_alarm_system.push(alarm);
                }
            }
            AlarmCategory::UserAlarmUser => {
                if self.user_alarm_user.len() < 100 {
                    self.user_alarm_user.push(alarm);
                }
            }
            AlarmCategory::OfflineAlarm => {
                if self.offline_alarm.len() < 100 {
                    self.offline_alarm.push(alarm);
                }
            }
            AlarmCategory::Invalid => {}
        }
    }

    /// Clear all alarm history
    pub fn clear_all(&mut self) {
        self.major_failure.clear();
        self.monitor_alarm.clear();
        self.user_alarm_system.clear();
        self.user_alarm_user.clear();
        self.offline_alarm.clear();
    }
}

impl Default for MockState {
    fn default() -> Self {
        Self::new_with_test_data()
    }
}

impl MockState {
    /// Create a new `MockState` with test data
    #[allow(clippy::too_many_lines)]
    fn new_with_test_data() -> Self {
        let variables = HashMap::new();
        // Note: Variables are initialized as needed, not pre-populated
        // This avoids conflicts between different variable types (B, I, D, R, S) using the same indices

        let mut io_states = HashMap::new();
        io_states.insert(1, true); // Robot user input 1
        io_states.insert(1001, false); // Robot user output 1

        let mut registers = HashMap::new();
        registers.insert(0, 0);
        registers.insert(1, 100);

        let mut files = HashMap::new();
        files.insert("TEST.JBI".to_string(), b"/JOB\r\n//NAME TEST.JBI\r\n//POS\r\n///NPOS 0,0,0,0,0,0\r\n//INST\r\n///DATE 2022/12/23 15:58\r\n///ATTR SC,RW\r\n///GROUP1 RB1\r\nNOP\r\nEND\r\n".to_vec());

        // Add test alarms (4 alarms for HSES specification: Instance 1-4)
        let alarms = vec![
            proto::payload::alarm::test_alarms::servo_error(), // Instance 1: Latest alarm
            proto::payload::alarm::test_alarms::emergency_stop(), // Instance 2: Second alarm
            proto::payload::alarm::test_alarms::safety_error(), // Instance 3: Third alarm
            proto::payload::alarm::test_alarms::communication_error(), // Instance 4: Fourth alarm
        ];

        // Add test alarm history data
        let mut alarm_history = AlarmHistory::default();

        // Add some major failure alarms (instances 1-3)
        alarm_history.add_alarm(
            proto::commands::alarm::AlarmCategory::MajorFailure,
            proto::payload::alarm::test_alarms::servo_error(),
        );
        alarm_history.add_alarm(
            proto::commands::alarm::AlarmCategory::MajorFailure,
            proto::payload::alarm::test_alarms::emergency_stop(),
        );
        alarm_history.add_alarm(
            proto::commands::alarm::AlarmCategory::MajorFailure,
            proto::payload::alarm::test_alarms::safety_error(),
        );

        // Add some monitor alarms (instances 1001-1003)
        alarm_history.add_alarm(
            proto::commands::alarm::AlarmCategory::MonitorAlarm,
            proto::payload::alarm::test_alarms::communication_error(),
        );
        alarm_history.add_alarm(
            proto::commands::alarm::AlarmCategory::MonitorAlarm,
            proto::payload::alarm::test_alarms::servo_error(),
        );

        Self {
            text_encoding: proto::TextEncoding::Utf8,
            status: proto::Status::new(
                proto::StatusData1 {
                    step: false,
                    one_cycle: false,
                    continuous: true,
                    running: false,
                    speed_limited: false,
                    teach: false,
                    play: true,
                    remote: false,
                },
                proto::StatusData2 {
                    teach_pendant_hold: false,
                    external_hold: false,
                    command_hold: false,
                    alarm: true,
                    error: false,
                    servo_on: true,
                },
            ),
            position: proto::Position::Pulse(proto::PulsePosition::new(vec![
                0, 0, 0, 0, 0, 0, 0, 0,
            ])),
            variables,
            io_states,
            registers,
            alarms,
            alarm_history,
            executing_job: Some(proto::ExecutingJobInfo::new("TEST.JOB".to_string(), 2, 1, 100)),
            selected_job: None,
            servo_on: true,
            hold_state: false,
            hlock_state: false,
            cycle_mode: proto::CycleMode::Continuous,
            files,
        }
    }
    /// Get variable value
    #[must_use]
    pub fn get_variable(&self, index: u16) -> Option<&Vec<u8>> {
        self.variables.get(&index)
    }

    /// Set variable value
    pub fn set_variable(&mut self, index: u16, value: Vec<u8>) {
        self.variables.insert(index, value);
    }

    /// Get multiple byte variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[must_use]
    #[allow(clippy::panic)]
    pub fn get_multiple_byte_variables(&self, start_variable: u16, count: usize) -> Vec<u8> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            let var_data = self.get_variable(var_num);
            values.push(var_data.map_or(0, |data| data.first().copied().unwrap_or(0)));
        }
        values
    }

    /// Set multiple byte variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[allow(clippy::panic)]
    pub fn set_multiple_byte_variables(&mut self, start_variable: u16, values: &[u8]) {
        for (i, &value) in values.iter().enumerate() {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            self.set_variable(var_num, vec![value]);
        }
    }

    /// Get multiple integer variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[must_use]
    #[allow(clippy::panic)]
    pub fn get_multiple_integer_variables(&self, start_variable: u16, count: usize) -> Vec<i16> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            let var_data = self.get_variable(var_num);
            // I variable is 2 bytes (i16)
            let value = var_data.map_or(0_i16, |data| {
                if data.len() >= 2 { i16::from_le_bytes([data[0], data[1]]) } else { 0 }
            });
            values.push(value);
        }
        values
    }

    /// Set multiple integer variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[allow(clippy::panic)]
    pub fn set_multiple_integer_variables(&mut self, start_variable: u16, values: &[i16]) {
        for (i, &value) in values.iter().enumerate() {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            self.set_variable(var_num, value.to_le_bytes().to_vec());
        }
    }

    /// Get multiple double precision integer variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[must_use]
    #[allow(clippy::panic)]
    pub fn get_multiple_double_variables(&self, start_variable: u16, count: usize) -> Vec<i32> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            let var_data = self.get_variable(var_num);
            // D variable is 4 bytes (i32)
            let value = var_data.map_or(0_i32, |data| {
                if data.len() >= 4 {
                    i32::from_le_bytes([data[0], data[1], data[2], data[3]])
                } else {
                    0
                }
            });
            values.push(value);
        }
        values
    }

    /// Set multiple double precision integer variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[allow(clippy::panic)]
    pub fn set_multiple_double_variables(&mut self, start_variable: u16, values: &[i32]) {
        for (i, &value) in values.iter().enumerate() {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            self.set_variable(var_num, value.to_le_bytes().to_vec());
        }
    }

    /// Get multiple real type variable values
    ///
    /// # Arguments
    ///
    /// * `start_variable` - Starting variable number
    /// * `count` - Number of variables to read
    ///
    /// # Returns
    ///
    /// Vector of real variable values (f32)
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[must_use]
    #[allow(clippy::panic)]
    pub fn get_multiple_real_variables(&self, start_variable: u16, count: usize) -> Vec<f32> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            let var_data = self.get_variable(var_num);
            // R variable is 4 bytes (f32)
            let value = var_data.map_or(0.0_f32, |data| {
                if data.len() >= 4 {
                    f32::from_le_bytes([data[0], data[1], data[2], data[3]])
                } else {
                    0.0
                }
            });
            values.push(value);
        }
        values
    }

    /// Set multiple real type variable values
    ///
    /// # Arguments
    ///
    /// * `start_variable` - Starting variable number
    /// * `values` - Real variable values to set
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[allow(clippy::panic)]
    pub fn set_multiple_real_variables(&mut self, start_variable: u16, values: &[f32]) {
        for (i, &value) in values.iter().enumerate() {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            self.set_variable(var_num, value.to_le_bytes().to_vec());
        }
    }

    /// Get multiple character type variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[must_use]
    #[allow(clippy::panic)]
    pub fn get_multiple_character_variables(
        &self,
        start_variable: u16,
        count: usize,
    ) -> Vec<[u8; 16]> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            let var_data = self.get_variable(var_num);
            // S variable is 16 bytes
            let mut value = [0u8; 16];
            if let Some(data) = var_data {
                let copy_len = data.len().min(16);
                value[..copy_len].copy_from_slice(&data[..copy_len]);
            }
            values.push(value);
        }
        values
    }

    /// Set multiple character type variable values
    ///
    /// # Panics
    ///
    /// Panics if the variable index exceeds `u16::MAX`
    #[allow(clippy::panic)]
    pub fn set_multiple_character_variables(&mut self, start_variable: u16, values: &[[u8; 16]]) {
        for (i, value) in values.iter().enumerate() {
            let var_num = start_variable
                + u16::try_from(i).unwrap_or_else(|_| {
                    panic!("Variable index {i} (start_variable: {start_variable}) exceeds u16::MAX")
                });
            self.set_variable(var_num, value.to_vec());
        }
    }

    /// Get I/O state
    #[must_use]
    pub fn get_io_state(&self, io_number: u16) -> bool {
        self.io_states.get(&io_number).copied().unwrap_or(false)
    }

    /// Set I/O state
    pub fn set_io_state(&mut self, io_number: u16, state: bool) {
        self.io_states.insert(io_number, state);
    }

    /// Get multiple I/O states
    ///
    /// # Errors
    ///
    /// Returns an error if the I/O offset exceeds `u16::MAX`
    pub fn get_multiple_io_states(
        &self,
        start_io_number: u16,
        count: usize,
    ) -> Result<Vec<u8>, String> {
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let mut byte_value = 0u8;
            for bit in 0..8 {
                let offset = u16::try_from(i * 8 + bit)
                    .map_err(|_| format!("I/O offset {} exceeds u16::MAX", i * 8 + bit))?;
                let io_number = start_io_number.checked_add(offset).ok_or_else(|| {
                    format!("I/O number {start_io_number} + {offset} overflows u16")
                })?;
                let state = self.get_io_state(io_number);
                if state {
                    byte_value |= 1 << bit;
                }
            }
            result.push(byte_value);
        }
        Ok(result)
    }

    /// Set multiple I/O states
    ///
    /// # Errors
    ///
    /// Returns an error if the I/O offset exceeds `u16::MAX`
    pub fn set_multiple_io_states(
        &mut self,
        start_io_number: u16,
        io_data: &[u8],
    ) -> Result<(), String> {
        for (i, &byte) in io_data.iter().enumerate() {
            for bit in 0..8 {
                let offset = u16::try_from(i * 8 + bit)
                    .map_err(|_| format!("I/O offset {} exceeds u16::MAX", i * 8 + bit))?;
                let io_number = start_io_number.checked_add(offset).ok_or_else(|| {
                    format!("I/O number {start_io_number} + {offset} overflows u16")
                })?;
                let state = (byte & (1 << bit)) != 0;
                self.set_io_state(io_number, state);
            }
        }
        Ok(())
    }

    /// Get register value
    #[must_use]
    pub fn get_register(&self, reg_number: u16) -> i16 {
        self.registers.get(&reg_number).copied().unwrap_or(0)
    }

    /// Set register value
    pub fn set_register(&mut self, reg_number: u16, value: i16) {
        self.registers.insert(reg_number, value);
    }

    /// Get multiple register values
    ///
    /// # Panics
    ///
    /// Panics if the count is too large to fit in a u16
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn get_multiple_registers(&self, start_register: u16, count: usize) -> Vec<i16> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let reg_num = start_register + u16::try_from(i).expect("i should fit in u16");
            values.push(self.get_register(reg_num));
        }
        values
    }

    /// Set multiple register values
    ///
    /// # Panics
    ///
    /// Panics if the count is too large to fit in a u16
    #[allow(clippy::expect_used)]
    pub fn set_multiple_registers(&mut self, start_register: u16, values: &[i16]) {
        for (i, &value) in values.iter().enumerate() {
            let reg_num = start_register + u16::try_from(i).expect("i should fit in u16");
            self.set_register(reg_num, value);
        }
    }

    /// Add alarm
    pub fn add_alarm(&mut self, alarm: proto::Alarm) {
        self.alarms.push(alarm);
        self.status.data2.alarm = true;
    }

    /// Clear alarms
    pub fn clear_alarms(&mut self) {
        self.alarms.clear();
        self.status.data2.alarm = false;
    }

    /// Set servo state
    pub const fn set_servo(&mut self, on: bool) {
        self.servo_on = on;
        self.status.data2.servo_on = on;
    }

    /// Set hold state
    pub const fn set_hold(&mut self, hold: bool) {
        self.hold_state = hold;
        self.status.data2.command_hold = hold;
        // If HOLD is ON, running should be false
        if hold {
            self.status.data1.running = false;
        } else {
            // If HOLD is OFF, running should be true (assuming no other holds)
            self.status.data1.running = true;
        }
    }

    /// Set running state
    pub const fn set_running(&mut self, running: bool) {
        self.status.data1.running = running;
    }

    /// Get running state
    #[must_use]
    pub const fn get_running(&self) -> bool {
        self.status.data1.running
    }

    /// Set executing job
    pub fn set_executing_job(&mut self, job: Option<proto::ExecutingJobInfo>) {
        self.executing_job = job;
    }

    /// Set selected job
    pub fn set_selected_job(&mut self, job_name: String, line_number: u32, select_type: u16) {
        self.selected_job = Some(SelectedJobInfo { job_name, line_number, select_type });
    }

    /// Get selected job
    #[must_use]
    pub const fn get_selected_job(&self) -> Option<&SelectedJobInfo> {
        self.selected_job.as_ref()
    }

    /// Update position
    pub fn update_position(&mut self, position: proto::Position) {
        self.position = position;
    }

    /// Get file list
    #[must_use]
    pub fn get_file_list(&self, pattern: &str) -> Vec<String> {
        if pattern == "*" || pattern.is_empty() {
            // Return all files
            self.files.keys().cloned().collect()
        } else if pattern.starts_with("*.") {
            // Pattern like "*.JBI" - match by extension
            let extension = &pattern[1..]; // Remove the "*"
            self.files.keys().filter(|name| name.ends_with(extension)).cloned().collect()
        } else {
            // Exact match or other patterns
            self.files
                .keys()
                .filter(|name| name.contains(pattern.trim_matches('*')))
                .cloned()
                .collect()
        }
    }

    /// Get file content
    #[must_use]
    pub fn get_file(&self, filename: &str) -> Option<&Vec<u8>> {
        self.files.get(filename)
    }

    /// Set file content
    pub fn set_file(&mut self, filename: String, content: Vec<u8>) {
        self.files.insert(filename, content);
    }

    /// Delete file
    pub fn delete_file(&mut self, filename: &str) -> bool {
        self.files.remove(filename).is_some()
    }

    /// Set HLOCK state
    pub const fn set_hlock(&mut self, enabled: bool) {
        self.hlock_state = enabled;
    }

    /// Get HLOCK state
    #[must_use]
    pub const fn is_hlock_enabled(&self) -> bool {
        self.hlock_state
    }

    /// Set cycle mode
    pub const fn set_cycle_mode(&mut self, mode: proto::CycleMode) {
        self.cycle_mode = mode;
    }

    /// Get cycle mode
    #[must_use]
    pub const fn get_cycle_mode(&self) -> proto::CycleMode {
        self.cycle_mode
    }
}

/// Thread-safe state wrapper
#[derive(Debug)]
pub struct SharedState {
    inner: Arc<RwLock<MockState>>,
}

impl SharedState {
    #[must_use]
    pub fn new(state: MockState) -> Self {
        Self { inner: Arc::new(RwLock::new(state)) }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, MockState> {
        self.inner.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, MockState> {
        self.inner.write().await
    }

    #[must_use]
    pub fn clone_inner(&self) -> Arc<RwLock<MockState>> {
        Arc::clone(&self.inner)
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new(MockState::default())
    }
}

impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}
