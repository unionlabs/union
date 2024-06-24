use std::{collections::HashMap, sync::Arc, time::Duration};

use dashmap::DashMap;
use parking_lot::Mutex;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use tokio::{task::JoinHandle, time::interval};

use crate::{
    chains::{Chain, Cosmos, Ethereum, IbcListen as _, IbcTransfer as _},
    config::{AnyChainConfig, Config, EventTrackerConfig, IbcInteraction},
};

pub type EventStateMap = Arc<DashMap<String, HashMap<i32, HashMap<u64, EventTrackerConfig>>>>;

#[derive(Clone, Debug)]
pub struct Context {
    pub chains: HashMap<String, Chain>,
    pub interactions: Vec<IbcInteraction>,
    pub event_state_map: EventStateMap,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self, ()> {
        let mut chains = HashMap::new();

        for (chain_name, chain) in config.chain_configs {
            match chain {
                AnyChainConfig::Cosmos(cosmos) if cosmos.enabled => {
                    tracing::info!("Initializing Cosmos chain: {}", chain_name);
                    chains.insert(chain_name, Chain::Cosmos(Cosmos::new(cosmos).await));
                }
                AnyChainConfig::Ethereum(ethereum) if ethereum.enabled => {
                    tracing::info!("Initializing Ethereum chain: {}", chain_name);
                    chains.insert(chain_name, Chain::Ethereum(Ethereum::new(ethereum).await));
                }
                _ => {
                    // Handle cases where the chain is not enabled or other configurations
                }
            }
        }

        tracing::info!(
            "Initialized chains: {:?}",
            chains.keys().collect::<Vec<_>>()
        );

        // Initialize the shared hashmap
        let event_state_map = Arc::new(DashMap::new());

        Ok(Self {
            chains,
            interactions: config.interactions,
            event_state_map, // Set the event_state_map field
        })
    }

    pub async fn listen(&self) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];

        for (chain_name, chain) in &self.chains {
            tracing::info!(%chain_name, "listening on chain");

            let event_state_map = self.event_state_map.clone();
            let chain = chain.clone();

            let handle = tokio::spawn(async move {
                chain.listen(&event_state_map).await;
            });
            handles.push(handle);
        }
        handles
    }

    pub async fn do_single_transaction(&self, interaction: IbcInteraction) -> JoinHandle<()> {
        let source_chain = self.chains.get(&interaction.source.chain).cloned().unwrap();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interaction.send_packet_interval));

            interval.tick().await;
            let mut rng = StdRng::from_entropy();

            let amount = rng.gen_range(interaction.amount_min..=interaction.amount_max);

            let send_memo = rng.gen_bool(interaction.sending_memo_probability);
            let memo = if send_memo {
                interaction.memo.clone()
            } else {
                String::new()
            };

            // selecting denom randomly
            let denom = interaction.denoms.choose(&mut rng).unwrap().to_string();

            match &source_chain {
                Chain::Ethereum(ethereum) => {
                    ethereum
                        .send_ibc_transfer(
                            interaction.protocol.clone(),
                            interaction.source.channel.clone(),
                            interaction.destination.channel.clone(),
                            denom,
                            amount,
                            memo,
                        )
                        .await;
                }
                Chain::Cosmos(cosmos) => {
                    cosmos
                        .send_ibc_transfer(
                            interaction.protocol.clone(),
                            interaction.source.channel.clone(),
                            interaction.destination.channel.clone(),
                            denom,
                            amount,
                            memo,
                        )
                        .await;
                }
            }
        })
    }

    pub async fn do_transactions(self) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];

        for interaction in self.interactions {
            let source_chain = self.chains.get(&interaction.source.chain).cloned().unwrap();
            let _destination_chain = self
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

                    // Determine memo based on sending_memo_probability
                    let send_memo = rng.gen_bool(interaction.sending_memo_probability);
                    let memo = if send_memo {
                        interaction.memo.clone()
                    } else {
                        String::new()
                    };

                    // selecting denom randomly
                    let denom = interaction.denoms.choose(&mut rng).unwrap().to_string();

                    match &source_chain {
                        Chain::Ethereum(ethereum) => {
                            ethereum
                                .send_ibc_transfer(
                                    interaction.protocol.clone(),
                                    interaction.source.channel.clone(),
                                    interaction.destination.channel.clone(),
                                    denom,
                                    amount,
                                    memo,
                                )
                                .await;
                        }
                        Chain::Cosmos(cosmos) => {
                            cosmos
                                .send_ibc_transfer(
                                    interaction.protocol.clone(),
                                    interaction.source.channel.clone(),
                                    interaction.destination.channel.clone(),
                                    denom,
                                    amount,
                                    memo,
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
        let event_state_map = Arc::clone(&self.event_state_map);
        let key = key.to_string(); // Clone the key to extend its lifetime

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10)); // TODO: Make this configurable (?)

            loop {
                interval.tick().await;

                // let mut event_state_map = event_state_map.lock().await;

                if let Some(mut inner_map) = event_state_map.get_mut(&key) {
                    // Use get_mut to get a mutable reference
                    let sequences_to_remove: Vec<i32> = inner_map
                        .iter()
                        .filter_map(|(&sequence, event_map)| {
                            let mut all_events_received = true;
                            for event_data in event_map.values() {
                                if !event_data.arrived {
                                    all_events_received = false;
                                }
                            }

                            if all_events_received {
                                tracing::info!("All events received for sequence: {}", sequence);
                                Some(sequence)
                            } else {
                                if let Some(event_data) = event_map.get(&0) {
                                    if let Some(send_packet_time) = event_data.arrived_time {
                                        let now = chrono::Utc::now();
                                        let duration = now.signed_duration_since(send_packet_time);

                                        if duration.num_seconds() >= (expect_full_cycle as i64) {
                                            tracing::error!(
                                                "[TRANSFER FAILED] Not all events received for sequence: {} after {} seconds. Event map: {:?}. Removing due to timeout.",
                                                sequence,
                                                duration.num_seconds(),
                                                event_map
                                            );
                                            Some(sequence)
                                        } else {
                                            None
                                        }
                                    } else {
                                        tracing::error!(
                                            "Not all events received for sequence: {} and no SendPacket timestamp found",
                                            sequence
                                        );
                                        Some(sequence)
                                    }
                                } else {
                                    tracing::error!(
                                        "SendPacket event not found for sequence: {}",
                                        sequence
                                    );
                                    None
                                }
                            }
                        })
                        .collect();

                    // Remove sequences with all events received
                    for sequence in sequences_to_remove {
                        inner_map.remove(&sequence);
                        tracing::info!("Removed sequence: {} from chain flow: {}", sequence, key);
                    }
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
