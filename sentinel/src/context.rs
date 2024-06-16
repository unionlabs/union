use std::{ collections::HashMap, sync::Arc, time::Duration };

use tokio::{ sync::Mutex, time::interval };
use unionlabs::ethereum::config::{ Mainnet, Minimal, PresetBaseKind };

use hex::{ decode as hex_decode, encode as hex_encode };
use hex::FromHex;

use ecdsa::SigningKey;

use ethers::{ signers::LocalWallet, core::k256::ecdsa, utils::secret_key_to_address };
use crate::{
    chains::{ Chain, Ethereum, IbcTransfer as _, Cosmos, IbcListen as _ },
    config::{ Config, IbcInteraction, KEY_ETHEREUM, KEY_OSMOSIS, KEY_UNION },
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

        if config.ethereum.enable {
            // We can take the first signer as the default signer since we don't need couple signers.

            let eth = match config.ethereum.preset {
                PresetBaseKind::Minimal => {
                    Chain::EthereumMinimal(
                        Ethereum::new(config.ethereum, "hebele".to_string()).await
                    )
                }
                PresetBaseKind::Mainnet => {
                    Chain::EthereumMainnet(
                        Ethereum::new(config.ethereum, "hubele".to_string()).await
                    )
                }
            };
            chains.insert(KEY_ETHEREUM.to_string(), eth);
        }

        if config.osmosis.enable {
            chains.insert(
                KEY_OSMOSIS.to_string(),
                Chain::Osmosis(Cosmos::new(config.osmosis).await)
            );
        }

        if config.union.enable {
            chains.insert(KEY_UNION.to_string(), Chain::Union(Cosmos::new(config.union).await));
        }

        tracing::info!("Initialized chains: {:?}", chains.keys().collect::<Vec<_>>());

        // Initialize the shared hashmap
        let shared_map = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            chains,
            interactions: config.interactions,
            shared_map, // Set the shared_map field
        })
    }

    pub async fn listen(&self) {
        if let Some(Chain::Osmosis(osmosis)) = self.chains.get(KEY_OSMOSIS).cloned() {
            let shared_map = self.shared_map.clone();
            tokio::spawn(async move {
                osmosis.listen(&shared_map).await;
            });
        }

        if let Some(Chain::Union(union)) = self.chains.get(KEY_UNION).cloned() {
            let shared_map = self.shared_map.clone();
            tokio::spawn(async move {
                union.listen(&shared_map).await;
            });
        }

        if let Some(Chain::EthereumMinimal(ethereum)) = self.chains.get(KEY_ETHEREUM).cloned() {
            let shared_map = self.shared_map.clone();

            tokio::spawn(async move {
                ethereum.listen(&shared_map).await;
            });
        }

        // TODO(caglankaan): It will be %100 same with the above block. We can refactor it.
        if let Some(Chain::EthereumMainnet(ethereum)) = self.chains.get(KEY_ETHEREUM).cloned() {
            let shared_map = self.shared_map.clone();

            tracing::info!("Listening to EthereumMainnet chain");
            tokio::spawn(async move {
                ethereum.listen(&shared_map).await;
            });
        }
    }

    pub async fn do_transactions(self) {
        for interaction in self.interactions {
            let source_chain = self.chains.get(&interaction.source.chain).cloned().unwrap();
            let destination_chain = self.chains
                .get(&interaction.destination.chain)
                .cloned()
                .unwrap();

            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(interaction.send_packet_interval));

                loop {
                    interval.tick().await;
                    match source_chain {
                        Chain::EthereumMinimal(_) => {
                            source_chain.send_ibc_transfer(
                                interaction.protocol.clone(),
                                interaction.source.channel.clone(),
                                interaction.destination.channel.clone(),
                                "muno".to_string(),
                                interaction.amount
                            ).await;
                        }
                        Chain::EthereumMainnet(_) => {
                            source_chain.send_ibc_transfer(
                                interaction.protocol.clone(),
                                interaction.source.channel.clone(),
                                interaction.destination.channel.clone(),
                                "muno".to_string(),
                                interaction.amount
                            ).await;
                        }
                        Chain::Osmosis(_) => {
                            source_chain.send_ibc_transfer(
                                interaction.protocol.clone(),
                                interaction.source.channel.clone(),
                                interaction.destination.channel.clone(),
                                "muno".to_string(),
                                interaction.amount
                            ).await;
                        }
                        Chain::Union(_) => {
                            source_chain.send_ibc_transfer(
                                interaction.protocol.clone(),
                                interaction.source.channel.clone(),
                                interaction.destination.channel.clone(),
                                "muno".to_string(),
                                interaction.amount
                            ).await;
                        }
                    }
                }
            });
        }
    }
    pub async fn check_packet_sequence(&self, expect_full_cycle: u64, key: &str) {
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
        });
    }

    pub async fn check_packet_sequences(&self) {
        for interaction in &self.interactions {
            let key = format!(
                "{}->{}",
                interaction.destination.channel,
                interaction.source.channel
            );
            let expect_full_cycle = interaction.expect_full_cycle;
            tracing::info!("Calling check_packet_sequence for key: {}", key);
            self.check_packet_sequence(expect_full_cycle, &key).await;
        }
    }
}
