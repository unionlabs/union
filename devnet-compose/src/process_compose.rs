use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
    pub log_configuration: Option<LogConfiguration>,
    pub log_location: Option<String>,
    // entrypoint: Option<Vec<String>>,
    // availability: AvailabilityConfig,
    pub depends_on: Option<HashMap<String, ProcessDependency>>,
    pub liveliness_probe: Option<Probe>,
    pub readiness_probe: Option<Probe>,
    // shutdown: ShutdownParams,
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
pub struct ProcessDependency {
    pub condition: Option<String>,
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ExecProbe {
    pub command: String,
    pub working_dir: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct HttpProbe {
    pub host: String,
    pub path: String,
    pub scheme: String,
    pub port: usize,
}
