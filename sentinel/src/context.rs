use std::{collections::HashMap, sync::Arc, time::Duration};

use ecdsa::SigningKey;
use ethers::{core::k256::ecdsa, signers::LocalWallet, utils::secret_key_to_address};
use hex::{decode as hex_decode, encode as hex_encode, FromHex};
use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::{sync::Mutex, task::JoinHandle, time::interval};
use unionlabs::ethereum::config::{Mainnet, Minimal, PresetBaseKind};

use crate::{
    chains::{Chain, Cosmos, Ethereum, IbcListen as _, IbcTransfer as _},
    config::{AnyChainConfig, Config, IbcInteraction, KEY_ETHEREUM, KEY_OSMOSIS, KEY_UNION},
};
type InnerInnerMap = HashMap<i32, bool>;
type InnerMap = HashMap<i32, InnerInnerMap>;
pub type SharedMap = Arc<Mutex<HashMap<String, InnerMap>>>;

#[derive(Clone, Debug)]
pub struct Context {
    pub chains: HashMap<String, Chain>,
    pub interactions: Vec<IbcInteraction>,
    pub shared_map: SharedMap,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self, ()> {
        let mut chains = HashMap::new();

        for (chain_name, chain) in config.chain_configs {
            match chain {
                AnyChainConfig::Cosmos(cosmos) => {
                    chains.insert(chain_name, Chain::Cosmos(Cosmos::new(cosmos).await));
                }
                AnyChainConfig::Ethereum(ethereum) => {
                    chains.insert(chain_name, Chain::Ethereum(Ethereum::new(ethereum).await));
                }
            }
        }

        tracing::info!(
            "Initialized chains: {:?}",
            chains.keys().collect::<Vec<_>>()
        );

        // Initialize the shared hashmap
        let shared_map = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            chains,
            interactions: config.interactions,
            shared_map, // Set the shared_map field
        })
    }

    pub async fn listen(&self) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];

        for (chain_name, chain) in &self.chains {
            tracing::info!(%chain_name, "listening on chain");

            let shared_map = self.shared_map.clone();
            let chain = chain.clone();

            let handle = tokio::spawn(async move {
                chain.listen(&shared_map).await;
            });
            handles.push(handle);
        }
        handles
    }

    pub async fn do_transactions(self) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];

        for interaction in self.interactions {
            let source_chain = self.chains.get(&interaction.source.chain).cloned().unwrap();
            let destination_chain = self
                .chains
                .get(&interaction.destination.chain)
                .cloned()
                .unwrap();

            let handle = tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(interaction.send_packet_interval));

                loop {
                    interval.tick().await;
                    let mut rng = StdRng::from_entropy();

                    let amount = rng.gen_range(interaction.amount_min..=interaction.amount_max);

                    match &source_chain {
                        Chain::Ethereum(ethereum) => {
                            ethereum
                                .send_ibc_transfer(
                                    interaction.protocol.clone(),
                                    interaction.source.channel.clone(),
                                    interaction.destination.channel.clone(),
                                    "muno".to_string(),
                                    amount,
                                )
                                .await;
                        }
                        Chain::Cosmos(cosmos) => {
                            cosmos
                                .send_ibc_transfer(
                                    interaction.protocol.clone(),
                                    interaction.source.channel.clone(),
                                    interaction.destination.channel.clone(),
                                    "muno".to_string(),
                                    amount,
                                )
                                .await;
                        }
                    }
                }
            });
            handles.push(handle);
        }
        handles
    }
    pub async fn check_packet_sequence(&self, expect_full_cycle: u64, key: &str) -> JoinHandle<()> {
        let shared_map = Arc::clone(&self.shared_map); // Clone the Arc to extend its lifetime
        let key = key.to_string(); // Clone the key to extend its lifetime

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(expect_full_cycle));

            loop {
                interval.tick().await;

                let mut shared_map = shared_map.lock().await;

                if let Some(inner_map) = shared_map.get_mut(&key) {
                    // Use get_mut to get a mutable reference
                    tracing::info!("Checking packet sequences for chain flow: {}", key);

                    let sequences_to_remove: Vec<i32> = inner_map
                        .iter()
                        .filter_map(|(&sequence, event_map)| {
                            let mut all_events_received = true;
                            tracing::info!("    Sequence: {}", sequence);
                            for (event_index, status) in event_map.iter() {
                                let event_name = match event_index {
                                    0 => "SendPacket",
                                    1 => "RecvPacket",
                                    2 => "WriteAcknowledgement",
                                    3 => "AcknowledgePacket",
                                    _ => "UnknownEvent",
                                };
                                if !status {
                                    all_events_received = false;
                                }
                                tracing::info!("      {}: {}", event_name, status);
                            }

                            if all_events_received {
                                tracing::info!("All events received for sequence: {}", sequence);
                                Some(sequence)
                            } else {
                                tracing::error!(
                                    "Not all events received for sequence: {} after {} seconds",
                                    sequence,
                                    expect_full_cycle
                                );
                                None
                            }
                        })
                        .collect();

                    // Remove sequences with all events received
                    for sequence in sequences_to_remove {
                        inner_map.remove(&sequence);
                        tracing::info!("Removed sequence: {} from chain flow: {}", sequence, key);
                    }
                } else {
                    tracing::info!("No data found for chain flow: {}", key);
                }
            }
        })
    }

    pub async fn check_packet_sequences(&self) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];
        for interaction in &self.interactions {
            let key = format!(
                "{}->{}",
                interaction.destination.channel, interaction.source.channel
            );
            let expect_full_cycle = interaction.expect_full_cycle;
            tracing::info!("Calling check_packet_sequence for key: {}", key);
            let handle = self.check_packet_sequence(expect_full_cycle, &key).await;
            handles.push(handle);
        }
        handles
    }
}
