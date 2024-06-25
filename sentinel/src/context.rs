use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use contracts::erc20;
use dashmap::DashMap;
use ethers::{
    providers::{Middleware, Provider, Ws},
    types::U256,
};
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

    pub async fn check_balances(self) -> Vec<JoinHandle<()>> {
        let min_eth_balance = U256::exp10(17); // 0.1 ETH
                                               // let mut min_erc20_balances = HashMap::new();
                                               // min_erc20_balances.insert(
                                               //     "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238".to_string(), // todo change to actual token address.
                                               //     U256::exp10(9) // 1000 Token
                                               // ); // 1 token

        let mut handles = vec![];

        handles.push(
            self.check_ethereum_balances(min_eth_balance /*min_erc20_balances*/)
                .await,
        );

        handles
    }

    pub async fn check_ethereum_balances(
        &self,
        min_balance: U256, /*min_erc20_balances: HashMap<String, U256>*/
    ) -> JoinHandle<()> {
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
            let mut interval = interval(Duration::from_secs(600)); // TODO: Make this configurable (?)

            loop {
                interval.tick().await;
                for ethereum in &ethereum_chains {
                    for signer_middleware in &*ethereum.signer_middlewares {
                        let signer_middleware = signer_middleware.lock().await;

                        let address = signer_middleware.address();
                        let provider: Arc<Provider<Ws>> = ethereum.rpc.provider.clone();

                        let chain_id = provider
                            .get_chainid()
                            .await
                            .expect("Failed to get chain ID")
                            .as_u64();

                        // Check native balance
                        match provider.get_balance(address, None).await {
                            Ok(balance) => {
                                if balance < min_balance {
                                    tracing::error!(
                                        "[INSUFFICIENT BALANCE] Insufficient native balance for address {:?}. Balance: {}, Required: {}. Chain ID: {}",
                                        address,
                                        balance,
                                        min_balance,
                                        chain_id
                                    );
                                } else {
                                    tracing::info!(
                                        "Sufficient ETH balance for address {:?}. Balance: {}, Required: {}",
                                        address,
                                        balance,
                                        min_balance
                                    );
                                }
                            }
                            Err(e) =>
                                tracing::error!(
                                    "Error checking native balance for address {:?}. Required: {}. Error: {:?}",
                                    address,
                                    min_balance,
                                    e
                                ),
                        }
                        // TODO (caglankaan): I think checking them here is not necessary and makes thing more complicated
                        // We can check the erc20 balances in the send_ibc message anyway, right?

                        // Check ERC20 balances
                        // for (erc20_address, min_balance) in &min_erc20_balances {
                        //     let erc20_contract = erc20::ERC20::new(
                        //         ethers::types::H160
                        //             ::from_str(erc20_address)
                        //             .expect("Failed to parse transaction hash"),
                        //         signer_middleware.clone()
                        //     );
                        //     match erc20_contract.balance_of(address).await {
                        //         Ok(balance) => {
                        //             if balance < *min_balance {
                        //                 tracing::error!(
                        //                     "[INSUFFICIENT BALANCE] Insufficient ERC20 balance for token {} on address {}. Balance: {}, Required: {}",
                        //                     erc20_address,
                        //                     address,
                        //                     balance,
                        //                     min_balance
                        //                 );
                        //             } else {
                        //                 tracing::info!(
                        //                     "Sufficient ERC20 balance for token {} on address {}. Balance: {}, Required: {}",
                        //                     erc20_address,
                        //                     address,
                        //                     balance,
                        //                     min_balance
                        //                 );
                        //             }
                        //         }
                        //         Err(e) =>
                        //             tracing::error!(
                        //                 "Error checking ERC20 balance for token {} on address {}. Required: {}. Error: {:?}",
                        //                 erc20_address,
                        //                 address,
                        //                 min_balance,
                        //                 e
                        //             ),
                        //     }
                        // }
                    }
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
}
