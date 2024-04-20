use std::collections::HashMap;

use crate::{
    log_path,
    process_compose::{LogConfiguration, Probe, ProcessDependency, ShutdownConfig},
    Process,
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
    }
}
