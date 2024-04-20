use std::collections::HashMap;

use crate::{
    log_path,
    process_compose::{LogConfiguration, Probe, Process, ProcessDependency, ShutdownConfig},
};

const CIRCUIT_BASE_PATH: &str = "./.devnet/circuit/";

pub fn download_circuit_process() -> Process {
    let name = "galois-download-circuit".to_string();
    Process {
        name: name.clone(),
        disabled: None,
        is_daemon: None,
        command: r##"nix run .#download-circuit-devnet ."##.into(),
        depends_on: None,
        liveliness_probe: None,
        readiness_probe: Some(Probe::exec(
            "nix run .#galoisd -- query-stats localhost:9999",
        )),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(&name),
        shutdown: ShutdownConfig::default(),
    }
}

pub fn galoisd_process() -> Process {
    let name = "galoisd".to_string();
    Process {
        name: name.clone(),
        disabled: None,
        is_daemon: None,
        command: format!("nix run .#galoisd -- serve localhost:9999 --cs-path={CIRCUIT_BASE_PATH}r1cs.bin --pk-path={CIRCUIT_BASE_PATH}pk.bin --vk-path={CIRCUIT_BASE_PATH}vk.bin"),
        depends_on: Some(HashMap::from([(download_circuit_process().name, ProcessDependency::completed_successfully())])),
        liveliness_probe: None,
        readiness_probe: Some(Probe::exec("nix run .#galoisd -- query-stats localhost:9999")),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(&name),
        shutdown: ShutdownConfig::default(),
    }
}
