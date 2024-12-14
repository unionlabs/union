use std::collections::HashMap;

use crate::{
    log_path,
    process_compose::{LogConfiguration, Probe, ProcessDependency, RestartPolicy, ShutdownConfig},
    Network, Process,
};

// Static names and constants
const VOYAGER_QUEUE_NAME: &str = "voyager-queue";
const VOYAGER_MIGRATIONS_NAME: &str = "voyager-migrations";
const VOYAGER_RELAY_NAME: &str = "voyager-relay";
const CONFIG_PATH: &str = "./voyager-config.json";

pub fn queue_process() -> Process {
    Process {
        name: VOYAGER_QUEUE_NAME.into(),
        disabled: None,
        is_daemon: None,
        command: "nix run .#voyager-queue -L".into(),
        depends_on: None,
        liveliness_probe: None,
        readiness_probe: Some(exec_probe(
            "pg_isready -h 127.0.0.1 -p 5432 -d default -U postgres",
        )),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(VOYAGER_QUEUE_NAME),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::always(2)),
    }
}

pub fn migrations_process() -> Process {
    Process {
        name: VOYAGER_MIGRATIONS_NAME.into(),
        disabled: None,
        is_daemon: None,
        command: format!(
            "RUST_LOG=debug nix run -L .#voyager -- -c {} run-migrations",
            CONFIG_PATH
        )
        .into(),
        depends_on: Some(HashMap::from([(
            VOYAGER_QUEUE_NAME.into(),
            ProcessDependency::healthy(),
        )])),
        liveliness_probe: None,
        readiness_probe: None,
        log_configuration: LogConfiguration::default(),
        log_location: log_path(VOYAGER_MIGRATIONS_NAME),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::on_failure(2)),
    }
}

pub fn relay_process(networks: &[Network]) -> Process {
    let mut depends_on = HashMap::from([
        (VOYAGER_QUEUE_NAME.into(), ProcessDependency::healthy()),
        (
            VOYAGER_MIGRATIONS_NAME.into(),
            ProcessDependency::completed_successfully(),
        ),
    ]);

    depends_on.extend(networks.iter().map(|network| {
        (
            network.to_process().name,
            ProcessDependency::healthy(),
        )
    }));

    Process {
        name: VOYAGER_RELAY_NAME.into(),
        disabled: None,
        is_daemon: None,
        command: format!(
            "RUST_LOG=info nix run -L .#voyager -- -c {} relay",
            CONFIG_PATH
        )
        .into(),
        depends_on: Some(depends_on),
        liveliness_probe: None,
        readiness_probe: Some(http_probe(65534, "/health")),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(VOYAGER_RELAY_NAME),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::always(2)),
    }
}

// Helper functions for probes
fn exec_probe(command: &str) -> Probe {
    Probe::exec(command)
}

fn http_probe(port: usize, path: &str) -> Probe {
    Probe::http_get(port, path)
}
