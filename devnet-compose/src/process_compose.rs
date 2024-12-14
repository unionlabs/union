use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::LOGS_BASE_PATH;

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    pub log_location: String,
    pub log_level: Option<String>,
    pub log_length: Option<usize>,
    pub log_configuration: Option<LogConfiguration>,
    pub log_format: String,
    pub processes: HashMap<String, Process>,
    pub is_strict: bool,
    pub file_names: Option<Vec<String>>,
}

impl Project {
    pub fn add_process(&mut self, process: Process) {
        self.processes
            .entry(process.name.clone())
            .or_insert(process);
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: "0.5".to_string(),
            log_location: LOGS_BASE_PATH.to_string(),
            log_level: None,
            log_length: None,
            log_format: "plain".to_string(),
            is_strict: true,
            file_names: None,
            log_configuration: Some(LogConfiguration::default()),
            processes: HashMap::new(),
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct LogConfiguration {
    pub rotation: Option<LogRotationConfig>,
    pub disable_json: Option<bool>,
    pub timestamp_format: Option<String>,
    pub no_color: Option<bool>,
    pub no_metadata: Option<bool>,
    pub add_timestamp: Option<bool>,
    pub flush_each_line: Option<bool>,
}

impl Default for LogConfiguration {
    fn default() -> Self {
        Self {
            rotation: Some(LogRotationConfig::default()),
            disable_json: Some(true),
            timestamp_format: None,
            no_color: None,
            no_metadata: Some(true),
            add_timestamp: Some(false),
            flush_each_line: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
pub struct LogRotationConfig {
    pub directory: Option<String>,
    pub filename: Option<String>,
    pub max_size_mb: Option<usize>,
    pub max_backups: Option<usize>,
    pub max_age_days: Option<usize>,
    pub compress: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub disabled: Option<bool>,
    pub is_daemon: Option<bool>,
    pub command: String,
    pub log_configuration: LogConfiguration,
    pub log_location: String,
    pub availability: Option<RestartPolicy>,
    pub depends_on: Option<HashMap<String, ProcessDependency>>,
    pub liveliness_probe: Option<Probe>,
    pub readiness_probe: Option<Probe>,
    pub shutdown: ShutdownConfig,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct RestartPolicy {
    pub restart: RestartType,
    pub backoff_seconds: usize,
    pub max_restarts: Option<usize>,
    pub exit_on_end: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub enum RestartType {
    Always,
    OnFailure,
}

impl RestartPolicy {
    pub fn always(backoff_seconds: usize) -> Self {
        assert!(backoff_seconds > 0, "backoff_seconds must be positive");
        Self {
            restart: RestartType::Always,
            backoff_seconds,
            max_restarts: None,
            exit_on_end: None,
        }
    }

    pub fn on_failure(backoff_seconds: usize) -> Self {
        assert!(backoff_seconds > 0, "backoff_seconds must be positive");
        Self {
            restart: RestartType::OnFailure,
            backoff_seconds,
            max_restarts: None,
            exit_on_end: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ShutdownConfig {
    pub command: Option<String>,
    pub timeout_seconds: Option<usize>,
    pub signal: Option<usize>,
    pub parent_only: Option<bool>,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            command: None,
            timeout_seconds: None,
            signal: Some(2),
            parent_only: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ProcessDependency {
    pub condition: String,
}

impl ProcessDependency {
    pub fn completed_successfully() -> Self {
        Self {
            condition: "process_completed_successfully".to_string(),
        }
    }

    pub fn healthy() -> Self {
        Self {
            condition: "process_healthy".to_string(),
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct Probe {
    pub exec: Option<ExecProbe>,
    pub http_get: Option<HttpProbe>,
    pub initial_delay_seconds: usize,
    pub period_seconds: usize,
    pub timeout_seconds: usize,
    pub success_threshold: usize,
    pub failure_threshold: usize,
}

impl Probe {
    pub fn exec(command: &str) -> Self {
        Self {
            exec: Some(ExecProbe {
                command: command.to_string(),
                working_dir: None,
            }),
            http_get: None,
            initial_delay_seconds: 5,
            period_seconds: 10,
            timeout_seconds: 5,
            success_threshold: 1,
            failure_threshold: 1000,
        }
    }

    pub fn http_get(port: usize, path: &str) -> Self {
        Self {
            exec: None,
            http_get: Some(HttpProbe {
                host: "127.0.0.1".to_string(),
                path: path.to_string(),
                scheme: "http".to_string(),
                port,
            }),
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            success_threshold: 1,
            failure_threshold: 1000,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ExecProbe {
    pub command: String,
    pub working_dir: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct HttpProbe {
    pub host: String,
    pub path: String,
    pub scheme: String,
    pub port: usize,
}
