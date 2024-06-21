use std::{collections::HashMap, sync::Arc, time::Duration};

use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use tokio::{sync::Mutex, task::JoinHandle, time::interval};

use crate::{
    chains::{Chain, Cosmos, Ethereum, IbcListen as _, IbcTransfer as _},
    config::{AnyChainConfig, Config, IbcInteraction},
};
type InnerInnerMap = HashMap<i32, (bool, Option<chrono::DateTime<chrono::Utc>>)>;
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
        let shared_map = Arc::clone(&self.shared_map);
        let key = key.to_string(); // Clone the key to extend its lifetime

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10)); // TODO: Make this configurable (?)

            loop {
                interval.tick().await;

                let mut shared_map = shared_map.lock().await;

                if let Some(inner_map) = shared_map.get_mut(&key) {
                    // Use get_mut to get a mutable reference

                    let sequences_to_remove: Vec<i32> = inner_map
                        .iter()
                        .filter_map(|(&sequence, event_map)| {
                            let mut all_events_received = true;
                            for (event_index, (status, _date)) in event_map.iter() {
                                let _event_name = match event_index {
                                    0 => "SendPacket",
                                    1 => "RecvPacket",
                                    2 => "WriteAcknowledgement",
                                    3 => "AcknowledgePacket",
                                    _ => "UnknownEvent",
                                };
                                if !status {
                                    all_events_received = false;
                                }
                            }

                            if all_events_received {
                                tracing::info!("All events received for sequence: {}", sequence);
                                Some(sequence)
                            } else {
                                if let Some((_, Some(send_packet_time))) = event_map.get(&0) {
                                    let now = chrono::Utc::now();
                                    let duration = now.signed_duration_since(*send_packet_time);

                                    if duration.num_seconds() >= (expect_full_cycle as i64) {
                                        tracing::error!(
                                            "[SENTINEL ERROR] Not all events received for sequence: {} after {} seconds. Event map: {:?}. Removing due to timeout.",
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
