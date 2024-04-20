use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::LOGS_BASE_PATH;

/// https://github.com/F1bonacc1/process-compose/blob/5a7b83ed8a0f6be58efa9e4940ff41517892eca2/src/types/project.go#L11-L12
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
    // pub environment: Environment,
    pub is_strict: bool,
    // pub vars: Vars,
    pub file_names: Option<Vec<String>>,
}

impl Project {
    pub fn add_process(&mut self, process: Process) {
        let name = process.name.clone();
        self.processes.insert(name, process);
    }
}

impl Default for Project {
    fn default() -> Self {
        Project {
            version: "0.5".into(),
            log_location: LOGS_BASE_PATH.into(),
            log_level: None,
            log_length: None,
            log_format: "plain".into(),
            is_strict: true,
            file_names: None,
            log_configuration: Some(LogConfiguration {
                rotation: Some(LogRotationConfig {
                    directory: Some(LOGS_BASE_PATH.into()),
                    filename: None,
                    max_size_mb: None,
                    max_backups: None,
                    max_age_days: None,
                    compress: Some(false),
                }),
                disable_json: Some(true),
                add_timestamp: Some(false),
                timestamp_format: None,
                no_color: Some(false),
                flush_each_line: Some(false),
                no_metadata: Some(true),
            }),
            processes: HashMap::new(),
        }
    }
}

// /// https://github.com/F1bonacc1/process-compose/blob/5a7b83ed8a0f6be58efa9e4940ff41517892eca2/src/types/logger.go
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct LogConfiguration {
    pub rotation: Option<LogRotationConfig>,
    // fields_order: Vec<String>,
    pub disable_json: Option<bool>,
    pub timestamp_format: Option<String>,
    pub no_color: Option<bool>,
    pub no_metadata: Option<bool>,
    pub add_timestamp: Option<bool>,
    pub flush_each_line: Option<bool>,
}

impl Default for LogConfiguration {
    fn default() -> Self {
        LogConfiguration {
            rotation: None,
            disable_json: Some(true),
            timestamp_format: None,
            no_color: None,
            no_metadata: Some(true),
            add_timestamp: Some(false),
            flush_each_line: None,
        }
    }
}

// /// https://github.com/F1bonacc1/process-compose/blob/5a7b83ed8a0f6be58efa9e4940ff41517892eca2/src/types/logger.go
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub directory: Option<String>,
    pub filename: Option<String>,
    pub max_size_mb: Option<usize>,
    pub max_backups: Option<usize>,
    pub max_age_days: Option<usize>,
    pub compress: Option<bool>,
}

/// https://github.com/F1bonacc1/process-compose/blob/5a7b83ed8a0f6be58efa9e4940ff41517892eca2/src/types/process.go#L15
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub disabled: Option<bool>,
    pub is_daemon: Option<bool>,
    pub command: String,
    pub log_configuration: LogConfiguration,
    pub log_location: String,
    // entrypoint: Option<Vec<String>>,
    pub availability: Option<RestartPolicy>,
    pub depends_on: Option<HashMap<String, ProcessDependency>>,
    pub liveliness_probe: Option<Probe>,
    pub readiness_probe: Option<Probe>,
    pub shutdown: ShutdownConfig,
    // disable_ansi_colors: bool,
    // working_dir: String,
    // namespace: String,
    // replicas: usize,
    // description: String,
    // vars: Vars,
    // is_foreground: bool,
    // is_tty: bool,
    // replica_num: usize,
    // replica_name: string,
    // executable: string,
    // args: Vec<string>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct RestartPolicy {
    pub restart: String,
    pub backoff_seconds: usize,
    pub max_restarts: Option<usize>,
    pub exit_on_end: Option<bool>,
}

impl RestartPolicy {
    pub fn always(backoff_seconds: usize) -> Self {
        RestartPolicy {
            restart: "always".into(),
            backoff_seconds,
            max_restarts: None,
            exit_on_end: None,
        }
    }

    pub fn on_failure(backoff_seconds: usize) -> Self {
        RestartPolicy {
            restart: "on_failure".into(),
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
        ShutdownConfig {
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
            condition: "process_completed_successfully".into(),
        }
    }

    pub fn healthy() -> Self {
        Self {
            condition: "process_healthy".into(),
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
    pub fn exec(command: &str) -> Probe {
        Probe {
            exec: Some(ExecProbe {
                command: command.into(),
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

    pub fn http_get(port: usize, path: &str) -> Probe {
        Probe {
            exec: None,
            http_get: Some(HttpProbe {
                host: "127.0.0.1".into(),
                path: path.into(),
                scheme: "http".into(),
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
