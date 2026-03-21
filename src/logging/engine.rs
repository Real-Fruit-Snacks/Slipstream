use std::fs;
use std::path::PathBuf;
use chrono::Utc;

use crate::logging::writer::{atomic_write, locked_write};

pub struct LogEngine {
    session_dir: PathBuf,
    command_counter: u32,
    current_command: Option<String>,
    current_output: String,
    _boundary_detection: bool,
}

impl LogEngine {
    pub fn new(session_dir: PathBuf, boundary_detection: bool) -> Self {
        // Create the session directory if it doesn't exist
        fs::create_dir_all(&session_dir).expect("failed to create session dir");
        LogEngine {
            session_dir,
            command_counter: 0,
            current_command: None,
            current_output: String::new(),
            _boundary_detection: boundary_detection,
        }
    }

    pub fn session_log_path(&self) -> PathBuf {
        self.session_dir.join("session.log")
    }

    pub fn log_session_start(&mut self, user: &str, hostname: &str, ip: &str) {
        let ts = Utc::now().to_rfc3339();
        let content = format!(
            "[{}] session start — user={} hostname={} ip={}\n",
            ts, user, hostname, ip
        );
        self.append_session_log(&content);
    }

    pub fn log_session_end(&mut self) {
        let ts = Utc::now().to_rfc3339();
        let content = format!("[{}] session end\n", ts);
        self.append_session_log(&content);
    }

    pub fn log_event(&mut self, event: &str) {
        let ts = Utc::now().to_rfc3339();
        let content = format!("[{}] event: {}\n", ts, event);
        self.append_session_log(&content);
    }

    pub fn start_command(&mut self, command: &str) {
        self.command_counter += 1;
        self.current_command = Some(command.to_string());
        self.current_output = String::new();
    }

    pub fn append_output(&mut self, data: &str) {
        self.current_output.push_str(data);
    }

    pub fn end_command(&mut self, exit_code: i32) {
        let cmd = match self.current_command.take() {
            Some(c) => c,
            None => return,
        };

        let ts = Utc::now().format("%Y%m%dT%H%M%S").to_string();
        // Sanitize command for use in filename
        let cmd_slug: String = cmd
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .take(32)
            .collect();

        let filename = format!("{:03}_{}_{}.log", self.command_counter, ts, cmd_slug);
        let path = self.session_dir.join(&filename);

        let content = format!(
            "command: {}\nexit_code: {}\noutput:\n{}",
            cmd, exit_code, self.current_output
        );
        atomic_write(&path, content.as_bytes()).expect("failed to write command log");
        self.current_output = String::new();
    }

    pub fn append_continuous_output(&mut self, data: &str) {
        let path = self.session_dir.join("session_continuous.log");
        let ts = Utc::now().to_rfc3339();
        let line = format!("[{}] {}", ts, data);
        // Use locked_write for shared resource
        locked_write(&path, line.as_bytes()).expect("failed to write continuous log");
    }

    fn append_session_log(&self, content: &str) {
        let path = self.session_log_path();
        locked_write(&path, content.as_bytes()).expect("failed to write session log");
    }
}
