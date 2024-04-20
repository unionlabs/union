// You need to bring the ToString trait into scope to use it
use std::{any::Any, collections::HashMap, fs};

use process_compose::{HttpProbe, LogConfiguration, LogRotationConfig, Probe, Process, Project};
use serde::{Deserialize, Serialize};

mod process_compose;
mod voyager;

const LOGS_BASE_PATH: &str = "./.devnet/logs/";

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum Network {
    Union,
    Osmosis,
    Stargaze,
}

impl Network {
    fn to_process(self) -> Process {
        Process {
            name: format!("{} Devnet", &self),
            command: format!("nix run .#{}", &self.network_id()),
            is_daemon: None,
            disabled: None,
            depends_on: None,
            liveliness_probe: None,
            readiness_probe: Some(Probe {
                exec: None,
                http_get: Some(HttpProbe {
                    host: "127.0.0.1".into(),
                    path: "/block?height=2".into(),
                    scheme: "http".into(),
                    port: self.probe_port(),
                }),
                initial_delay_seconds: 10,
                period_seconds: 10,
                timeout_seconds: 5,
                success_threshold: 1,
                failure_threshold: 1000,
            }),
            log_configuration: Some(LogConfiguration {
                rotation: None,
                disable_json: Some(true),
                timestamp_format: None,
                no_color: None,
                no_metadata: Some(true),
                add_timestamp: Some(false),
                flush_each_line: None,
            }),
            log_location: Some(format!("{LOGS_BASE_PATH}/{}.log", self.network_id())),
        }
    }

    fn network_id(&self) -> String {
        format!("devnet-{}", self.to_string().to_lowercase())
    }

    fn probe_port(&self) -> usize {
        match self {
            Network::Union => 26657,
            Network::Stargaze => 26757,
            Network::Osmosis => 26857,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevnetConfig {
    pub networks: Vec<Network>,
    pub connections: Vec<(Network, Network)>,
}

impl DevnetConfig {
    pub fn to_process_compose(&self) -> Project {
        let mut project = Project {
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
        };

        for network in self.networks.clone() {
            project
                .processes
                .insert(network.network_id(), network.to_process());
        }

        project
    }
}

fn main() {
    use Network::*;
    let config = DevnetConfig {
        networks: vec![Union, Osmosis, Stargaze],
        connections: vec![(Union, Osmosis)],
    };

    let project = config.to_process_compose();
    let project = serde_json::to_string_pretty(&project).expect("failed to serialize project");

    fs::write("process-compose.yml", project).expect("failed to write contents");
}
