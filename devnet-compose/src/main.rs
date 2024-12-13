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

// Configuration Constants
const LOGS_BASE_PATH: &str = "./.devnet/logs/";
const CONFIG_PATH: &str = "./voyager-config.json";

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
        ProcessBuilder::new(
            &self.network_id(),
            &format!("nix run .#{}", self.network_id()),
        )
        .readiness_probe(Probe::http_get(self.probe_port(), "/block?height=2"))
        .availability(RestartPolicy::always(10))
        .build()
    }

    fn network_id(&self) -> String {
        format!("devnet-{}", self.to_string().to_lowercase())
    }

    fn probe_port(&self) -> usize {
        match self {
            Network::Union => 26657,
            Network::Stargaze => 26757,
            Network::Osmosis => 26857,
            Network::Simd => 26957,
        }
    }

    fn cometbls_light_client_config(&self) -> String {
        assert!(
            self != &Network::Union,
            "Tried to get cometbls client id on union"
        );
        let cometbls_lightclient_checksum = fs::read_to_string(format!(
            "./.devnet/homes/{}/code-ids/cometbls_light_client",
            self.to_string().to_lowercase()
        ))
        .unwrap_or_else(|_| panic!(
            "could not find code-id for cometbls_light_client on {self}"
        ));

        format!(
            "'{{\"checksum\":\"0x{}\"}}'",
            cometbls_lightclient_checksum.trim()
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DevnetConfig {
    pub networks: Vec<Network>,
    pub connections: Vec<(Network, Network)>,
}

pub fn connection_to_process((net_a, net_b): &(Network, Network)) -> Process {
    let name = format!(
        "connection-{}-{}",
        net_a.to_string().to_lowercase(),
        net_b.to_string().to_lowercase()
    );

    let (client_a_config, client_b_config) = match (net_a, net_b) {
        (Network::Union, n) => ("null".to_string(), n.cometbls_light_client_config()),
        (n, Network::Union) => (n.cometbls_light_client_config(), "null".to_string()),
        (_, _) => ("null".to_string(), "null".to_string()),
    };

    ProcessBuilder::new(
        &name,
        &format!(
            "set -o pipefail; nix run .#voyager -- queue enqueue \"$(nix run -L .#voyager -- -c {CONFIG_PATH} handshake {} {} --client-a-config {} --client-b-config {} --create-clients --open-connection --connection-ordering unordered --init-fetch)\"",
            net_a.network_id(), net_b.network_id(), client_a_config, client_b_config
        ),
    )
    .is_daemon(true)
    .depends_on(HashMap::from([
        (net_a.to_process().name, ProcessDependency::healthy()),
        (net_b.to_process().name, ProcessDependency::healthy()),
        (voyager::relay_process(&[]).name, ProcessDependency::healthy()),
    ]))
    .build()
}

impl DevnetConfig {
    pub fn to_process_compose(&self) -> Project {
        let mut project = Project::default();

        for network in self.networks.clone() {
            project.add_process(network.to_process());
        }

        if !self.connections.is_empty() {
            project.add_process(voyager::queue_process());
            project.add_process(voyager::migrations_process());
            project.add_process(voyager::relay_process(&self.networks));

            if self.networks.contains(&Network::Union) {
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

fn setup_network_selection() -> Vec<Network> {
    multiselect("Which networks do you want to include?")
        .initial_values(vec![])
        .item(Network::Union, "Union", "recommended")
        .item(Network::Osmosis, "Osmosis", "")
        .item(Network::Stargaze, "Stargaze", "")
        .item(Network::Simd, "Simd", "")
        .interact()
        .unwrap()
}

fn setup_connections(networks: &[Network]) -> Vec<(Network, Network)> {
    if networks.len() > 1 {
        let connection_options = networks
            .iter()
            .combinations(2)
            .map(|combo| {
                let (net_a, net_b) = (combo[0], combo[1]);
                (
                    (*net_a, *net_b),
                    format!("{net_a} <-> {net_b}"),
                    if *net_a == Network::Union || *net_b == Network::Union {
                        "Will include Galois services".to_string()
                    } else {
                        "".to_string()
                    },
                )
            })
            .collect::<Vec<_>>();

        multiselect("Which IBC connections do you want to set up?")
            .items(&connection_options)
            .required(false)
            .interact()
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_project_to_file(project: &Project) {
    let project_str = serde_json::to_string_pretty(project).expect("failed to serialize project");
    fs::write("process-compose.yml", project_str).expect("failed to write process-compose.yml");
}

fn main() {
    cliclack::set_theme(theme::UnionTheme);
    let _ = cliclack::clear_screen();

    intro(style(" Union Devnet Compose ").on_cyan().black().bold()).unwrap();
    let networks = setup_network_selection();
    let connections = setup_connections(&networks);

    cliclack::note(
        style(" Devnet generated! ").on_cyan().black().bold(),
        format!(
            "Tips:\n - Run {} in a second terminal tab to view logs.\n - Restart single processes with ctrl+r.\n - View the generated process composition in process-compose.yml.\n",
            style("`nix run .#devnet-logs`").cyan().bold()
        ),
    )
    .unwrap();

    let config = DevnetConfig { networks, connections };
    let project = config.to_process_compose();

    save_project_to_file(&project);

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
