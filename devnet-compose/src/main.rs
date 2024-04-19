use std::{collections::HashSet, fs};

use process_compose::Project;
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
