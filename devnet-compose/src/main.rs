use std::{collections::HashMap, fs};

use process_compose::{LogConfiguration, Process, Project};
use serde::{Deserialize, Serialize};

mod process_compose;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub enum Network {
    Union,
    Osmosis,
    Stargaze,
    Ethereum,
    Scroll,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevnetConfig {
    pub networks: Vec<Network>,
    pub connections: Vec<(Network, Network)>,
}

impl DevnetConfig {
    pub fn to_process_compose(&self) -> Project {
        Project {
            version: "0.5".into(),
            log_location: ".devnet/logs/".into(),
            log_level: None,
            log_length: None,
            log_format: "plain".into(),
            is_strict: true,
            file_names: None,
            log_configuration: Some(LogConfiguration {
                disable_json: true,
                add_timestamp: false,
                timestamp_format: None,
                no_color: false,
                flush_each_line: false,
                no_metadata: true,
            }),
            processes: HashMap::from([(
                "union-devnet".into(),
                Process {
                    name: "Union Devnet".into(),
                    command: "nix run .#devnet-union".into(),
                    disabled: None,
                },
            )]),
        }
    }
}

fn main() {
    use Network::*;
    let config = DevnetConfig {
        networks: vec![Union, Osmosis],
        connections: vec![(Union, Osmosis)],
    };

    let project = config.to_process_compose();
    let project = serde_json::to_string_pretty(&project).expect("failed to serialize project");

    fs::write("process-compose.yml", project).expect("failed to write contents");
}
