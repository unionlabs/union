use std::collections::HashMap;

use crate::{
    log_path,
    process_compose::{
        LogConfiguration, Probe, Process, ProcessDependency, RestartPolicy, ShutdownConfig,
    },
};

// Constants
const CIRCUIT_BASE_PATH: &str = "./.devnet/circuit/";
const GALIOSD_PORT: &str = "localhost:9999";

/// Creates a process for downloading the circuit files.
pub fn download_circuit_process() -> Process {
    let name = "galois-download-circuit";
    Process {
        name: name.into(),
        disabled: None,
        is_daemon: None,
        command: "nix run .#download-circuit-devnet .".into(),
        depends_on: None,
        liveliness_probe: None,
        readiness_probe: None,
        log_configuration: LogConfiguration::default(),
        log_location: log_path(name),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::on_failure(2)),
    }
}

/// Creates the main `galoisd` process.
pub fn galoisd_process() -> Process {
    let name = "galoisd";
    let command = format!(
        "nix run .#galoisd -- serve {GALIOSD_PORT} \
        --cs-path={CIRCUIT_BASE_PATH}r1cs.bin \
        --pk-path={CIRCUIT_BASE_PATH}pk.bin \
        --vk-path={CIRCUIT_BASE_PATH}vk.bin"
    );

    let readiness_probe = Probe::exec(format!("nix run .#galoisd -- query-stats {GALIOSD_PORT}"));

    Process {
        name: name.into(),
        disabled: None,
        is_daemon: None,
        command,
        depends_on: Some(HashMap::from([(
            download_circuit_process().name,
            ProcessDependency::completed_successfully(),
        )])),
        liveliness_probe: None,
        readiness_probe: Some(readiness_probe),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(name),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::always(5)),
    }
}
