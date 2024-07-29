use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use chain_utils::keyring::ChainKeyring;
use contracts::erc20;
use dashmap::DashMap;
use ethers::{
    providers::{Middleware, Provider, Ws},
    types::U256,
};
use parking_lot::Mutex;
use protos::ibc::core::client::v1::query_client::QueryClient;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
// use protos::ibc::core::client::v1::Quer
use serde_json::json;
use tokio::{task::JoinHandle, time::interval};
use unionlabs::hash::H160;

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
                            interaction.max_retry,
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
                            interaction.max_retry,
                        )
                        .await;
                }
            }
        })
    }

    pub async fn balances(&self) -> HashMap<String, serde_json::Value> {
        let mut balances = HashMap::new();

        for (chain_name, chain) in self.chains.iter() {
            let mut account_index = 1;
            tracing::info!("Checking balance for chain: {}", chain_name);
            tracing::info!("Checking balance for chain: {:?}", chain);
            match chain {
                Chain::Ethereum(ethereum) => {
                    let provider = ethereum.rpc.provider.clone();
                    let mut chain_balances = HashMap::new();

                    for signer_middleware in &*ethereum.signer_middlewares {
                        let signer = signer_middleware.lock().await;
                        let address = signer.address();
                        match provider.get_balance(address, None).await {
                            Ok(balance) => {
                                let account_key = format!("account-{}", account_index);
                                chain_balances.insert(
                                    account_key,
                                    json!({
                                        "address": format!("{:?}", address),
                                        "balance": balance.to_string(),
                                    }),
                                );
                                account_index += 1;
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Error fetching balance for address {:?}: {:?}",
                                    address,
                                    e
                                );
                            }
                        }
                    }
                    balances.insert(chain_name.clone(), json!(chain_balances));
                }
                Chain::Cosmos(cosmos) => {
                    let mut chain_balances = HashMap::new();
                    let chain_id = cosmos.chain.keyring.name.clone();

                    let balances_list = cosmos.chain.balances().await;
                    for balance_info in balances_list {
                        chain_balances.insert(
                            balance_info.key_name.clone(),
                            json!({
                                "address": balance_info.address,
                                "balance": balance_info.balance.to_string(),
                                "denom": balance_info.denom,
                            }),
                        );
                    }

                    balances.insert(chain_id.to_string(), json!(chain_balances));
                }
            }
        }

        balances
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
                                    interaction.max_retry,
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
                                    interaction.max_retry,
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

    pub async fn perform_token_distribution(&self) -> JoinHandle<()> {
        let mut chain_tokens: HashMap<String, Vec<H160>> = HashMap::new();

        // Extract tokens from interactions based on the source chain
        for interaction in &self.interactions {
            if let Chain::Ethereum(_) = self.chains[&interaction.source.chain] {
                let tokens = chain_tokens
                    .entry(interaction.source.chain.clone())
                    .or_insert_with(Vec::new);

                for denom in &interaction.denoms {
                    if let Ok(token) = H160::from_str(denom) {
                        if !tokens.contains(&token) {
                            tokens.push(token);
                        }
                    } else {
                        tracing::warn!("Invalid token address: {}", denom);
                    }
                }
            }
        }

        let chains = self.chains.clone();
        tokio::spawn(async move {
            for (chain_name, tokens) in chain_tokens {
                if let Some(Chain::Ethereum(ethereum)) = chains.get(&chain_name) {
                    ethereum.token_distribution(tokens).await;
                }
            }
        })
    }

    pub async fn perform_native_token_distribution(&self) -> JoinHandle<()> {
        let ethereum_chains: Vec<_> = self
            .chains
            .values()
            .filter_map(|chain| {
                if let Chain::Ethereum(eth) = chain {
                    Some(eth)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        tokio::spawn(async move {
            for ethereum in &ethereum_chains {
                ethereum.native_token_distribution().await;
            }
        })
    }
}
