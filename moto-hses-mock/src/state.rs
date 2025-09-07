//! Mock server state management

use moto_hses_proto as proto;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock server state
#[derive(Debug, Clone)]
pub struct MockState {
    pub status: proto::Status,
    pub position: proto::Position,
    pub variables: HashMap<u8, Vec<u8>>,
    pub io_states: HashMap<u16, bool>,
    pub registers: HashMap<u16, i16>,
    pub alarms: Vec<proto::Alarm>,
    pub alarm_history: AlarmHistory,
    pub current_job: Option<String>,
    pub servo_on: bool,
    pub hold_state: bool,
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
    pub fn get_alarm(
        &self,
        category: proto::alarm::AlarmCategory,
        index: usize,
    ) -> Option<&proto::Alarm> {
        match category {
            proto::alarm::AlarmCategory::MajorFailure => self.major_failure.get(index),
            proto::alarm::AlarmCategory::MonitorAlarm => self.monitor_alarm.get(index),
            proto::alarm::AlarmCategory::UserAlarmSystem => self.user_alarm_system.get(index),
            proto::alarm::AlarmCategory::UserAlarmUser => self.user_alarm_user.get(index),
            proto::alarm::AlarmCategory::OfflineAlarm => self.offline_alarm.get(index),
            proto::alarm::AlarmCategory::Invalid => None,
        }
    }

    /// Add alarm to specific category
    pub fn add_alarm(&mut self, category: proto::alarm::AlarmCategory, alarm: proto::Alarm) {
        match category {
            proto::alarm::AlarmCategory::MajorFailure => {
                if self.major_failure.len() < 100 {
                    self.major_failure.push(alarm);
                }
            }
            proto::alarm::AlarmCategory::MonitorAlarm => {
                if self.monitor_alarm.len() < 100 {
                    self.monitor_alarm.push(alarm);
                }
            }
            proto::alarm::AlarmCategory::UserAlarmSystem => {
                if self.user_alarm_system.len() < 100 {
                    self.user_alarm_system.push(alarm);
                }
            }
            proto::alarm::AlarmCategory::UserAlarmUser => {
                if self.user_alarm_user.len() < 100 {
                    self.user_alarm_user.push(alarm);
                }
            }
            proto::alarm::AlarmCategory::OfflineAlarm => {
                if self.offline_alarm.len() < 100 {
                    self.offline_alarm.push(alarm);
                }
            }
            proto::alarm::AlarmCategory::Invalid => {}
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
        let mut variables = HashMap::new();
        variables.insert(0, vec![0x01, 0x00, 0x00, 0x00]); // D000 = 1
        variables.insert(1, vec![0x64, 0x00, 0x00, 0x00]); // D001 = 100
        variables.insert(2, vec![0x00, 0x00, 0x20, 0x41]); // D002 = 10.0

        let mut io_states = HashMap::new();
        io_states.insert(1, true); // Robot user input 1
        io_states.insert(1001, false); // Robot user output 1

        let mut registers = HashMap::new();
        registers.insert(0, 0);
        registers.insert(1, 100);

        let mut files = HashMap::new();
        files.insert("TEST.JOB".to_string(), b"/JOB\r\n//NAME TEST.JOB\r\n//POS\r\n///NPOS 0,0,0,0,0,0\r\n//INST\r\n///DATE 2022/12/23 15:58\r\n///ATTR SC,RW\r\n///GROUP1 RB1\r\nNOP\r\nEND\r\n".to_vec());

        // Add test alarms (4 alarms for HSES specification: Instance 1-4)
        let alarms = vec![
            proto::alarm::test_alarms::servo_error(), // Instance 1: Latest alarm
            proto::alarm::test_alarms::emergency_stop(), // Instance 2: Second alarm
            proto::alarm::test_alarms::safety_error(), // Instance 3: Third alarm
            proto::alarm::test_alarms::communication_error(), // Instance 4: Fourth alarm
        ];

        // Add test alarm history data
        let mut alarm_history = AlarmHistory::default();

        // Add some major failure alarms (instances 1-3)
        alarm_history.add_alarm(
            proto::alarm::AlarmCategory::MajorFailure,
            proto::alarm::test_alarms::servo_error(),
        );
        alarm_history.add_alarm(
            proto::alarm::AlarmCategory::MajorFailure,
            proto::alarm::test_alarms::emergency_stop(),
        );
        alarm_history.add_alarm(
            proto::alarm::AlarmCategory::MajorFailure,
            proto::alarm::test_alarms::safety_error(),
        );

        // Add some monitor alarms (instances 1001-1003)
        alarm_history.add_alarm(
            proto::alarm::AlarmCategory::MonitorAlarm,
            proto::alarm::test_alarms::communication_error(),
        );
        alarm_history.add_alarm(
            proto::alarm::AlarmCategory::MonitorAlarm,
            proto::alarm::test_alarms::servo_error(),
        );

        Self {
            status: proto::Status::new(
                proto::StatusData1 {
                    step: false,
                    one_cycle: false,
                    continuous: true,
                    running: true,
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
            position: proto::Position::Pulse(proto::PulsePosition::new(
                [0, 0, 0, 0, 0, 0, 0, 0],
                1,
            )),
            variables,
            io_states,
            registers,
            alarms,
            alarm_history,
            current_job: Some("TEST.JOB".to_string()),
            servo_on: true,
            hold_state: false,
            files,
        }
    }
}

impl MockState {
    /// Get variable value
    pub fn get_variable(&self, index: u8) -> Option<&Vec<u8>> {
        self.variables.get(&index)
    }

    /// Set variable value
    pub fn set_variable(&mut self, index: u8, value: Vec<u8>) {
        self.variables.insert(index, value);
    }

    /// Get I/O state
    pub fn get_io_state(&self, io_number: u16) -> bool {
        self.io_states.get(&io_number).copied().unwrap_or(false)
    }

    /// Set I/O state
    pub fn set_io_state(&mut self, io_number: u16, state: bool) {
        self.io_states.insert(io_number, state);
    }

    /// Get register value
    pub fn get_register(&self, reg_number: u16) -> i16 {
        self.registers.get(&reg_number).copied().unwrap_or(0)
    }

    /// Set register value
    pub fn set_register(&mut self, reg_number: u16, value: i16) {
        self.registers.insert(reg_number, value);
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
    pub fn set_servo(&mut self, on: bool) {
        self.servo_on = on;
        self.status.data2.servo_on = on;
    }

    /// Set hold state
    pub fn set_hold(&mut self, hold: bool) {
        self.hold_state = hold;
        self.status.data2.command_hold = hold;
    }

    /// Set running state
    pub fn set_running(&mut self, running: bool) {
        self.status.data1.running = running;
    }

    /// Set current job
    pub fn set_current_job(&mut self, job: Option<String>) {
        self.current_job = job;
    }

    /// Update position
    pub fn update_position(&mut self, position: proto::Position) {
        self.position = position;
    }

    /// Get file list
    pub fn get_file_list(&self, pattern: &str) -> Vec<String> {
        self.files
            .keys()
            .filter(|name| name.contains(pattern.trim_matches('*')))
            .cloned()
            .collect()
    }

    /// Get file content
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
}

/// Thread-safe state wrapper
#[derive(Debug)]
pub struct SharedState {
    inner: Arc<RwLock<MockState>>,
}

impl SharedState {
    pub fn new(state: MockState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(state)),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, MockState> {
        self.inner.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, MockState> {
        self.inner.write().await
    }

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
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
