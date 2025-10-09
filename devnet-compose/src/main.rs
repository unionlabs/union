use std::{collections::HashMap, fs};

use cliclack::{intro, multiselect};
use console::style;
use itertools::Itertools;
use process_compose::{
    LogConfiguration, Probe, Process, ProcessDependency, Project, ShutdownConfig,
};
use serde::{Deserialize, Serialize};

use crate::process_compose::RestartPolicy;

mod galois;
mod process_compose;
mod theme;
mod voyager;

const LOGS_BASE_PATH: &str = "./.devnet/logs/";

pub fn log_path(process_name: &str) -> String {
    format!("{LOGS_BASE_PATH}{process_name}.log")
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum Network {
    Union,
    Osmosis,
    Stargaze,
    Simd,
}

impl Network {
    fn to_process(self) -> Process {
        Process {
            name: self.network_id().clone(),
            command: format!("nix run .#{}", self.network_id()),
            is_daemon: None,
            disabled: None,
            depends_on: None,
            liveliness_probe: None,
            readiness_probe: Some(Probe::http_get(self.probe_port(), "/block?height=2")),
            log_configuration: LogConfiguration::default(),
            log_location: log_path(&self.network_id()),
            shutdown: ShutdownConfig::default(),
            availability: Some(RestartPolicy::always(10)),
        }
    }

    fn network_id(&self) -> String {
        format!("devnet-{}", self.to_string().to_lowercase())
    }

    // TODO: Put this in a config file
    fn probe_port(&self) -> usize {
        match self {
            Network::Union => 26657,
            Network::Stargaze => 26757,
            Network::Osmosis => 26857,
            Network::Simd => 26957,
        }
    }

    fn cometbls_light_client_config(&self) -> String {
        // TODO: this is a bit hacky, well need better Network types rather than an assertion here.
        assert!(
            self != &Network::Union,
            "Tried to get cometbls client id on union"
        );
        let cometbls_lightclient_checksum = fs::read_to_string(format!(
            "./.devnet/homes/{}/code-ids/cometbls_light_client",
            self.to_string().to_lowercase()
        ))
        .unwrap_or_else(|_| panic!("could not find code-id for cometbls_light_client on {self}"));

        let cometbls_lightclient_checksum = cometbls_lightclient_checksum.trim().to_string();

        format!("'{{\"checksum\":\"0x{cometbls_lightclient_checksum}\"}}'")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevnetConfig {
    pub networks: Vec<Network>,
    pub connections: Vec<(Network, Network)>,
}

pub fn connection_to_process((net_a, net_b): &(Network, Network)) -> Process {
    use Network::*;
    let name = format!(
        "connection-{}-{}",
        net_a.to_string().to_lowercase(),
        net_b.to_string().to_lowercase()
    );

    let (client_a_config, client_b_config) = match (net_a, net_b) {
        (Union, n) => ("null".to_string(), n.cometbls_light_client_config()),
        (n, Union) => (n.cometbls_light_client_config(), "null".to_string()),
        (_, _) => ("null".to_string(), "null".to_string()),
    };

    Process {
        name: name.clone(),
        disabled: None,
        is_daemon: Some(true),
        command: format!(
            "set -o pipefail; nix run .#voyager -- queue enqueue \"$(nix run -L .#voyager -- -c ./voyager-config.json handshake {} {} --client-a-config {} --client-b-config {} --create-clients --open-connection --connection-ordering unordered --init-fetch)\"",
            net_a.network_id(),
            net_b.network_id(),
            client_a_config,
            client_b_config
        ),

        log_configuration: LogConfiguration::default(),
        log_location: log_path(&name),
        depends_on: Some(HashMap::from([
            (net_a.to_process().name, ProcessDependency::healthy()),
            (net_b.to_process().name, ProcessDependency::healthy()),
            (
                voyager::relay_process(&[]).name,
                ProcessDependency::healthy(),
            ),
        ])),
        liveliness_probe: None,
        readiness_probe: None, // TODO
        shutdown: ShutdownConfig::default(),
        availability: None,
    }
}

impl DevnetConfig {
    pub fn to_process_compose(&self) -> Project {
        let mut project = Project::default();

        // Add a devnet for each network
        for network in self.networks.clone() {
            project.add_process(network.to_process());
        }

        if !self.connections.is_empty() {
            // There are connections, so we need voyager running with applied migrations
            project.add_process(voyager::queue_process());
            project.add_process(voyager::migrations_process());
            project.add_process(voyager::relay_process(&self.networks));

            if self.networks.contains(&Network::Union) {
                // There are connections to Union, so we need to prove Union consensus
                project.add_process(galois::download_circuit_process());
                project.add_process(galois::galoisd_process());
            }

            for conn in &self.connections {
                project.add_process(connection_to_process(conn))
            }
        }

        project
    }
}

fn main() {
    cliclack::set_theme(theme::UnionTheme);
    let _ = cliclack::clear_screen();

    intro(style(" Union Devnet Compose ").on_cyan().black().bold()).unwrap();
    let networks: Vec<Network> = multiselect("Which networks do you want to include?")
        .initial_values(vec![])
        .item(Union, "Union", "recommended")
        .item(Osmosis, "Osmosis", "")
        .item(Stargaze, "Stargaze", "")
        .item(Simd, "Simd", "")
        .interact()
        .unwrap();

    let mut connections: Vec<(Network, Network)> = Vec::new();
    if networks.len() > 1 {
        let connection_options: Vec<((Network, Network), String, String)> = networks
            .clone()
            .into_iter()
            .combinations(2)
            .map(|mut combo| (combo.remove(0), combo.remove(0)))
            .map(|combo @ (net_a, net_b)| {
                (
                    combo,
                    format!("{net_a} <-> {net_b}").to_string(),
                    if net_a == Union || net_b == Union {
                        "Will include Galois services".to_string()
                    } else {
                        "".to_string()
                    },
                )
            })
            .collect();
        connections = multiselect("Which IBC connections do you want to set up?")
            .items(&connection_options)
            .required(false)
            .interact()
            .unwrap();
    }

    let info_text = format!(
        "Tips:\n - Run {} in a second terminal tab to view logs.\n - You can restart single processes in the interface with ctrl+r.\n - You can view the generated process composition at `process-compose.yml` in the repo root.\n - Processes are designed to be overridden for a fast dev feedback cycle.\n   For example, add a `process-compose.override.yml` to the repo root with the following contents\n   to use a cargo debug build of voyager instead of the nix build.",
        style("`nix run .#devnet-logs`").cyan().bold()
    ) + r##"
    {
      "processes": {
        "voyager-relay" : {
          "command" : "RUST_LOG=info cargo run -p voyager"
        }
      }
    }
    "##;

    cliclack::note(
        style(" Devnet generated! ").on_cyan().black().bold(),
        info_text,
    )
    .unwrap();

    use Network::*;
    let config = DevnetConfig {
        networks,
        connections,
    };

    let project = config.to_process_compose();
    let project = serde_json::to_string_pretty(&project).expect("failed to serialize project");

    fs::write("process-compose.yml", project).expect("failed to write contents");

    let answer = cliclack::confirm("Ready to launch the devnet?")
        .initial_value(true)
        .interact()
        .unwrap();

    if answer {
        cliclack::outro("Launching the devnet").unwrap()
    } else {
        cliclack::outro_cancel("Generated a process-compose.yml but did not start the devnet")
            .unwrap()
    }
}
