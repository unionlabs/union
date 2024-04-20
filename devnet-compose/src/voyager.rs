use std::collections::HashMap;

use crate::{
    log_path,
    process_compose::{LogConfiguration, Probe, ProcessDependency, RestartPolicy, ShutdownConfig},
    Network, Process,
};

pub fn queue_process() -> Process {
    let name = "voyager-queue".to_string();
    Process {
        name: name.clone(),
        disabled: None,
        is_daemon: None,
        command: "nix run .#voyager-queue -L".into(),
        depends_on: None,
        liveliness_probe: None,
        readiness_probe: Some(Probe::exec(
            "pg_isready -h 127.0.0.1 -p 5432 -d default -U postgres",
        )),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(&name),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::always(2)),
    }
}

pub fn migrations_process() -> Process {
    let name = "voyager-migrations".to_string();
    Process {
        name: name.clone(),
        disabled: None,
        is_daemon: None,
        command: "RUST_LOG=debug nix run -L .#voyager -- -c ./voyager-config.json run-migrations"
            .into(),
        depends_on: Some(HashMap::from([(
            queue_process().name,
            ProcessDependency::healthy(),
        )])),
        liveliness_probe: None,
        readiness_probe: None,
        log_configuration: LogConfiguration::default(),
        log_location: log_path(&name),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::on_failure(2)),
    }
}
pub fn relay_process(networks: &[Network]) -> Process {
    let name = "voyager-migrations".to_string();

    let mut depends_on = HashMap::from([
        (queue_process().name, ProcessDependency::healthy()),
        (
            migrations_process().name,
            ProcessDependency::completed_successfully(),
        ),
    ]);

    for network in networks {
        depends_on.insert(network.to_process().name, ProcessDependency::healthy());
    }

    Process {
        name: name.clone(),
        disabled: None,
        is_daemon: None,
        command: "RUST_LOG=info nix run -L .#voyager -- -c ./voyager-config.json relay".into(),
        depends_on: Some(depends_on),
        liveliness_probe: None,
        readiness_probe: Some(Probe::http_get(65534, "/health")),
        log_configuration: LogConfiguration::default(),
        log_location: log_path(&name),
        shutdown: ShutdownConfig::default(),
        availability: Some(RestartPolicy::always(2)),
    }
}
