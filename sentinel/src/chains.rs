use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use bech32::FromBase32;
use chain_utils::{
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChainExt},
    ethereum::{EthereumExecutionRpcs, EthereumExecutionRpcsExt, IBCHandlerEvents},
    private_key::PrivateKey,
};
use chrono::Utc;
use contracts::{
    erc20,
    ibc_packet::IBCPacketEvents,
    ucs01_relay::{LocalToken, UCS01Relay},
};
use ethers::{
    abi::RawLog,
    contract::EthLogDecode,
    core::k256::ecdsa,
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    prelude::*,
    providers::{Middleware, Provider, Ws},
    signers::LocalWallet,
    types::{Address, Filter, H256},
    utils::secret_key_to_address,
};
use futures::stream::{FuturesUnordered, StreamExt};
use hex::{self, encode as hex_encode};
use prost::Message;
use protos::{google::protobuf::Any, ibc::applications::transfer::v1::MsgTransfer};
use rand::{prelude::SliceRandom, rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tendermint_rpc::{event::EventData, SubscriptionClient};
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use ucs01_relay_api::types::{Ics20Ack, JsonWasm, Ucs01Ack};
use unionlabs::{
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    encoding::{self, DecodeAs},
    events::{AcknowledgePacket, RecvPacket, SendPacket, WriteAcknowledgement},
    google::protobuf::any,
    hash::H160,
    ibc::core::{
        channel::channel::{self, Channel},
        client::height::Height,
    },
    id::{ChannelId, ClientId},
    tendermint::abci::{event::Event as TendermintEvent, event_attribute::EventAttribute},
    uint::U256,
    validated::ValidateT,
};

use crate::{
    config::{CosmosConfig, EthereumConfig, EventTrackerConfig, TransferModule},
    context::EventStateMap,
};
pub type IbcEvent = unionlabs::events::IbcEvent<ClientId, String, ClientId>;

pub trait IbcTransfer: Send + Sync {
    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
        memo: String,
        max_retry: u64,
    );

    async fn native_token_distribution(&self);

    async fn token_distribution(&self, tokens: Vec<H160>);

    // async fn distribute_balance(&self);
}

pub trait IbcListen: Send + Sync {
    async fn listen(&self, event_state_map: &EventStateMap);

    // TODO(caglankaan): How can i know the protocol type here? On listen we don't know what is the destination chain
    // It can be anything, if i am listening on union since there is only one listener for union there could be 2 different
    // chains which are sending request to me 1- ethereum with ucs01 and 2- osmosis with ics20 so i am not sure how can i know
    // the protocol here. For know i'll try bruteforce but it's not a good solution.
    fn write_handler_packet_ack_hex_controller(
        &self,
        ack_hex: Vec<u8>, //protocol: Protocol
    ) -> bool {
        // match protocol {
        //     Protocol::Ics20 => {
        //         let val = Ics20Ack::try_from(cosmwasm_std::Binary::from(ack_hex)).unwrap();
        //         match val {
        //             Ics20Ack::Result(_) => {
        //                 return true;
        //             }
        //             Ics20Ack::Error(_) => {
        //                 return false;
        //             }
        //         }
        //     }
        //     Protocol::Ucs01 => {
        //         return (
        //             Ucs01Ack::try_from(cosmwasm_std::Binary::from(ack_hex)).unwrap() ==
        //             Ucs01Ack::Success
        //         );
        //     }
        //     _ => {
        //         tracing::error!("Unknown protocol {:?} -> {:?}", protocol, ack_hex);
        //         return false;
        //     }
        // }

        // Try to decode as Ics20Ack first;
        if let Ok(val) =
            Ics20Ack::decode_as::<JsonWasm>(cosmwasm_std::Binary::from(ack_hex.clone()).as_slice())
        {
            match val {
                Ics20Ack::Result(_) => {
                    return true;
                }
                Ics20Ack::Error(_) => {
                    tracing::warn!("Ics20Ack::Result failed decode.");
                }
            }
        }

        if let Ok(val) = Ucs01Ack::decode_as::<encoding::EthAbi>(
            cosmwasm_std::Binary::from(ack_hex.clone()).as_slice(),
        ) {
            return val == Ucs01Ack::Success;
        } else {
            tracing::warn!("Failed to decode ack_hex: {:?} ", ack_hex);
            return false;
        }
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        event_state_map: &EventStateMap,
        block_number: u64,
        tx_hash: Option<H256>,
        chain_id: String,
    );

    fn handle_ibc_event_boxed<'a>(
        &'a self,
        ibc_event: IbcEvent,
        event_state_map: &'a EventStateMap,
        _block_number: u64,
        tx_hash: Option<H256>,
        chain_id: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        let chain_type = get_chain_type(&chain_id);
        let mut formatted_tx: String = Default::default();
        if let Some(tx) = tx_hash {
            formatted_tx = match chain_type {
                ChainType::Ethereum => unionlabs::hash::H256::from(tx).to_string(),
                ChainType::Cosmos => {
                    unionlabs::hash::H256::from(tx).to_string()[2..].to_ascii_uppercase()
                }
            };
        }

        Box::pin(async move {
            let (packet_sequence, key) = match &ibc_event {
                IbcEvent::SendPacket(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                IbcEvent::RecvPacket(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                IbcEvent::WriteAcknowledgement(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                IbcEvent::AcknowledgePacket(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                // Handle other events if necessary,
                _ => {
                    return;
                }
            };

            let sequence: i32 = packet_sequence.get() as i32;

            let mut entry = event_state_map
                .entry(key.clone())
                .or_insert_with(HashMap::new);

            let sequence_entry = entry.entry(sequence).or_insert_with(|| {
                let mut event_map = HashMap::new();
                for idx in 0..4 {
                    event_map.insert(
                        idx,
                        EventTrackerConfig {
                            idx,
                            arrived: false,
                            arrived_time: None,
                            tx_hash: None,
                        },
                    );
                }
                event_map
            });

            match ibc_event {
                IbcEvent::SendPacket(event) => {
                    if let Some(event_data) = sequence_entry.get_mut(&0) {
                        event_data.arrived = true;
                        event_data.arrived_time = Some(chrono::Utc::now());
                        event_data.tx_hash = Some(formatted_tx.clone());

                        tracing::info!(
                            sequence = sequence,
                            chain_id = chain_id,
                            key = key,
                            tx_hash = formatted_tx.clone(),
                            "SendPacket event recorded."
                        );
                    } else {
                        tracing::warn!(
                            "Unexpected error: Could not find event data for SendPacket."
                        );
                    }
                }
                IbcEvent::RecvPacket(_) => {
                    if sequence_entry.get(&0).map_or(false, |e| e.arrived) {
                        if let Some(event_data) = sequence_entry.get_mut(&1) {
                            event_data.arrived = true;
                            event_data.arrived_time = Some(chrono::Utc::now());
                            event_data.tx_hash = Some(formatted_tx.clone());
                            tracing::info!(
                                sequence = sequence,
                                chain_id = chain_id,
                                key = key,
                                tx_hash = formatted_tx.clone(),
                                "RecvPacket event recorded."
                            );
                        } else {
                            tracing::warn!(
                                "Unexpected error: Could not find event data for RecvPacket."
                            );
                        }
                    } else {
                        tracing::warn!(
                            sequence = sequence,
                            chain_id = chain_id,
                            key = key,
                            tx_hash = formatted_tx.clone(),
                            "RecvPacket event received without SendPacket."
                        );
                        entry.remove(&sequence);
                    }
                }

                IbcEvent::WriteAcknowledgement(ref e) => {
                    if sequence_entry.get(&0).map_or(false, |e| e.arrived) {
                        if self.write_handler_packet_ack_hex_controller(e.packet_ack_hex.clone()) {
                            if let Some(event_data) = sequence_entry.get_mut(&2) {
                                event_data.arrived = true;
                                event_data.arrived_time = Some(chrono::Utc::now());
                                event_data.tx_hash = Some(formatted_tx.clone());
                                tracing::info!(
                                    sequence = sequence,
                                    chain_id = chain_id,
                                    key = key,
                                    tx_hash = formatted_tx.clone(),
                                    "WriteAcknowledgement event recorded."
                                );
                            } else {
                                tracing::warn!(
                                    "Unexpected error: Could not find event data for WriteAcknowledgement."
                                );
                            }
                        } else {
                            let initial_tx_hash =
                                sequence_entry.get(&0).and_then(|e| e.tx_hash.clone());
                            tracing::error!(
                                sequence = sequence,
                                chain_id = chain_id,
                                key = key,
                                tx_hash = formatted_tx.clone(),
                                initial_tx_hash = initial_tx_hash,
                                "[TRANSFER FAILED] WriteAcknowledgement indicates failure. packet_ack_hex: {:?}.",
                                e.packet_ack_hex.clone()
                            );
                            entry.remove(&sequence);
                        }
                    } else {
                        tracing::warn!(
                            sequence = sequence,
                            chain_id = chain_id,
                            key = key,
                            tx_hash = formatted_tx.clone(),
                            "WriteAcknowledgement event received without SendPacket. "
                        );
                        entry.remove(&sequence);
                    }
                }
                IbcEvent::AcknowledgePacket(_) => {
                    if sequence_entry.get(&0).map_or(false, |e| e.arrived)
                        && sequence_entry.get(&1).map_or(false, |e| e.arrived)
                        && sequence_entry.get(&2).map_or(false, |e| e.arrived)
                    {
                        if let Some(event_data) = sequence_entry.get_mut(&3) {
                            event_data.arrived = true;
                            event_data.arrived_time = Some(chrono::Utc::now());
                            event_data.tx_hash = Some(formatted_tx.clone());
                            tracing::info!(
                                sequence = sequence,
                                chain_id = chain_id,
                                key = key,
                                tx_hash = formatted_tx.clone(),
                                "AcknowledgePacket event recorded."
                            );

                            if sequence_entry.values().all(|event_data| event_data.arrived) {
                                tracing::info!(
                                    sequence = sequence,
                                    chain_id = chain_id,
                                    key = key,
                                    tx_hash = formatted_tx.clone(),
                                    "All events completed. sequence_entry: {:?}.",
                                    sequence_entry
                                );
                                entry.remove(&sequence);
                            }
                        } else {
                            tracing::warn!(
                                "Unexpected error: Could not find event data for AcknowledgePacket."
                            );
                        }
                    } else {
                        tracing::warn!(
                            sequence = sequence,
                            chain_id = chain_id,
                            key = key,
                            tx_hash = formatted_tx.clone(),
                            "AcknowledgePacket event received out of order. "
                        );
                        entry.remove(&sequence);
                    }
                }
                _ => {
                    return;
                }
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Protocol {
    Ics20 {
        receivers: Vec<String>, // Changed to Vec<String>
        module: TransferModule,
    },
    Ucs01 {
        receivers: Vec<String>, //Vec<Vec<u8>>,
        contract: String,
    },
}

#[derive(Debug, Clone)]
pub enum Chain {
    Ethereum(Ethereum),
    Cosmos(Cosmos),
}

impl IbcListen for Chain {
    async fn listen(&self, event_state_map: &EventStateMap) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum.listen(event_state_map).await;
            }
            Chain::Cosmos(cosmos) => {
                cosmos.listen(event_state_map).await;
            }
        }
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        event_state_map: &EventStateMap,
        block_number: u64,
        tx_hash: Option<H256>,
        chain_id: String,
    ) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum
                    .handle_ibc_event(ibc_event, event_state_map, block_number, tx_hash, chain_id)
                    .await;
            }
            Chain::Cosmos(cosmos) => {
                cosmos
                    .handle_ibc_event(ibc_event, event_state_map, block_number, tx_hash, chain_id)
                    .await;
            }
        }
    }

    fn write_handler_packet_ack_hex_controller(
        &self,
        ack_hex: Vec<u8>, // protocol: Protocol // TODO: Add it after find a way
    ) -> bool {
        IbcListen::write_handler_packet_ack_hex_controller(self, ack_hex /* , protocol*/)
    }
}

impl IbcTransfer for Chain {
    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
        memo: String,
        max_retry: u64,
    ) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum
                    .send_ibc_transfer(
                        protocol,
                        channel,
                        destination_channel,
                        denom,
                        amount,
                        memo,
                        max_retry,
                    )
                    .await;
            }
            Chain::Cosmos(cosmos) => {
                cosmos
                    .send_ibc_transfer(
                        protocol,
                        channel,
                        destination_channel,
                        denom,
                        amount,
                        memo,
                        max_retry,
                    )
                    .await;
            }
        }
    }
    async fn native_token_distribution(&self) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum.native_token_distribution().await;
            }
            Chain::Cosmos(cosmos) => {
                cosmos.native_token_distribution().await;
            }
        }
    }

    async fn token_distribution(&self, tokens: Vec<H160>) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum.token_distribution(tokens).await;
            }
            Chain::Cosmos(cosmos) => {
                cosmos.token_distribution(tokens).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ethereum {
    pub rpc: EthereumRpc,
    pub relays:
        Vec<UCS01Relay<SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>>>,
    pub signer_middlewares: Arc<
        Vec<
            tokio::sync::Mutex<
                Arc<SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>>,
            >,
        >,
    >,
    pub ucs01_contract: String,
    pub msg_senders: Vec<ethers::types::H160>,
    pub relay_addr: ethers::types::H160,
    pub master_account: PrivateKey<ecdsa::SigningKey>,
}

#[derive(Debug, Clone)]
pub struct EthereumRpc {
    pub provider: Arc<Provider<Ws>>,
    pub ibc_handler_address: H160,
}

impl EthereumExecutionRpcs for EthereumRpc {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl IbcListen for Ethereum {
    async fn listen(&self, event_state_map: &EventStateMap) {
        let mut latest_checked_block = 0;
        loop {
            let provider = self.rpc.provider.clone();

            let latest_block: u64 = provider.get_block_number().await.unwrap().as_u64();
            if latest_checked_block >= latest_block {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            latest_checked_block = latest_block;
            let chain_id = provider
                .get_chainid()
                .await
                .expect("Failed to get chain ID")
                .as_u64();
            // tracing::info!(
            //     block = latest_block,
            //     chain_id = chain_id,
            //     "Fetching Ethereum latest_block."
            // );
            // Update the filter to fetch logs from the latest block processed + 1
            let filter = Filter::new()
                .address(ethers::types::H160::from(self.rpc.ibc_handler_address))
                .from_block(latest_block)
                .to_block(latest_block);

            let logs = provider.get_logs(&filter).await.unwrap();

            let logs_clone = logs.clone(); // Clone logs for processing
            futures::stream::iter(logs_clone.clone())
                .filter_map(|log| async move {
                    let raw_log = RawLog {
                        topics: log.topics.clone(),
                        data: log.data.clone().to_vec(),
                    };
                    let transaction_hash = log.transaction_hash;
                    Some((raw_log, transaction_hash))
                })
                .for_each_concurrent(None, |(raw_log, transaction_hash)| {
                    let value = logs_clone.clone();
                    async move {
                        let ibc_event: Option<
                            unionlabs::events::IbcEvent<
                                unionlabs::validated::Validated<
                                    String,
                                    (
                                        unionlabs::id::Bounded<9, 64>,
                                        unionlabs::id::Ics024IdentifierCharacters,
                                    ),
                                >,
                                String,
                                unionlabs::validated::Validated<
                                    String,
                                    (
                                        unionlabs::id::Bounded<9, 64>,
                                        unionlabs::id::Ics024IdentifierCharacters,
                                    ),
                                >,
                            >,
                        > = ibchandler_events_to_ibc_event(raw_log, &self.rpc, latest_block).await;

                        if let Some(ibc_event) = ibc_event {
                            self.handle_ibc_event(
                                ibc_event,
                                &event_state_map,
                                latest_block,
                                transaction_hash,
                                chain_id.to_string(),
                            )
                            .await;
                        }
                    }
                })
                .await;
        }
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        event_state_map: &EventStateMap,
        block_number: u64,
        tx_hash: Option<H256>,
        chain_id: String,
    ) {
        IbcListen::handle_ibc_event_boxed(
            self,
            ibc_event,
            event_state_map,
            block_number,
            tx_hash,
            chain_id,
        )
        .await;
    }
}
impl IbcListen for Cosmos {
    async fn listen(&self, event_state_map: &EventStateMap) {
        tracing::info!("Listening to Cosmos chain events");
        let mut subs = self
            .chain
            .tm_client
            .subscribe(tendermint_rpc::query::EventType::Tx.into())
            .await
            .unwrap();
        loop {
            tokio::select! {
                Some(event_result) = subs.next() => {
                    match event_result {
                        Ok(event) => {
                            if let Some(ref events) = event.events {
                                if let Some(heights) = events.get("tx.height") {
                                    if let Some(height) = heights.first() {
                                        let block_number: u64 = height.parse().expect("Failed to parse block number");
                                        // tracing::info!("Fetched cosmos Block number: {}", block_number);

                                        if let Some(tx_hashes) = events.get("tx.hash") {
                                            if let Some(tx_hash) = tx_hashes.first() {
                                                let tx_hash = H256::from_str(tx_hash).expect("Failed to parse transaction hash");

                                                match event.data {
                                                    EventData::Tx { tx_result, .. } => {
                                                        for event in tx_result.result.events {
                                                            let some_event = IbcEvent::try_from_tendermint_event(TendermintEvent {
                                                                ty: event.kind,
                                                                attributes: event.attributes
                                                                    .into_iter()
                                                                    .map(|attr| EventAttribute {
                                                                        key: attr.key,
                                                                        value: attr.value,
                                                                        index: attr.index,
                                                                    })
                                                                    .collect(),
                                                            });

                                                            if let Some(Ok(ibc_event)) = some_event {
                                                                self.handle_ibc_event(ibc_event, &event_state_map, block_number, tx_hash.into(), self.chain.chain_id.clone()).await;
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        tracing::error!("Unhandled event type: {:?}", event);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error while receiving event: {:?}", e);
                        }
                    }
                },
                else => break,
            }
        }
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        event_state_map: &EventStateMap,
        block_number: u64,
        tx_hash: Option<H256>,
        chain_id: String,
    ) {
        IbcListen::handle_ibc_event_boxed(
            self,
            ibc_event,
            event_state_map,
            block_number,
            tx_hash,
            chain_id,
        )
        .await;
    }
}

impl IbcTransfer for Ethereum {
    async fn token_distribution(&self, erc20_contracts: Vec<H160>) {
        let provider = self.rpc.provider.clone();
        let master_acc = self.master_account.clone().value();
        let addr_of_master: ethers::types::H160 = secret_key_to_address(&master_acc);

        let chain_id = match provider.get_chainid().await {
            Ok(chain_id) => chain_id.as_u64(),
            Err(e) => {
                tracing::error!("Failed to get chain ID: {:?}", e);
                return;
            }
        };

        tracing::info!(
            "Token distribution for tokens: {:?} on chain: {}",
            erc20_contracts,
            chain_id
        );
        let master_wallet =
            LocalWallet::new_with_signer(master_acc.clone(), addr_of_master, chain_id);

        let master_middleware = Arc::new(SignerMiddleware::new(
            NonceManagerMiddleware::new(provider.clone(), addr_of_master),
            master_wallet.clone(),
        ));

        for contract_address in erc20_contracts.iter() {
            let erc20 = erc20::ERC20::new(*contract_address, provider.clone());

            // Collect tokens from signers
            let mut total_collected = erc20.balance_of(addr_of_master).await.unwrap();
            tracing::info!(
                "Gathering from users started for contract: {:?}",
                contract_address
            );

            let mut futures: FuturesUnordered<_> = self.signer_middlewares
                .iter()
                .map(|signer_middleware| {
                    let provider = provider.clone();
                    let signer_middleware = signer_middleware.clone();
                    let addr_of_master = addr_of_master.clone();
                    let erc20 = erc20.clone();

                    async move {
                        let signer_middleware = signer_middleware.lock().await;
                        let address = signer_middleware.address();
                        let balance = match erc20.balance_of(address).call().await {
                            Ok(balance) => balance,
                            Err(e) => {
                                tracing::error!(
                                    "Failed to get token balance for address {:?}: {:?}",
                                    address,
                                    e
                                );
                                return ethers::types::U256::from(0);
                            }
                        };

                        if balance > ethers::types::U256::from(1e6 as u64) {
                            // Adjust threshold as needed
                            let amount_to_send = balance - ethers::types::U256::from(1e6 as u64); // Keep 1 token
                            let tx = erc20.transfer(addr_of_master, amount_to_send).tx;

                            for _ in 0..3 {
                                let pending_tx = signer_middleware.send_transaction(
                                    tx.clone(),
                                    None
                                ).await;
                                match pending_tx {
                                    Ok(pending_tx) => {
                                        if pending_tx.await.is_ok() {
                                            return amount_to_send;
                                        } else {
                                            tracing::error!(
                                                "Failed to send tokens from {:?} to master, retrying...",
                                                address
                                            );
                                            tokio::time::sleep(Duration::from_secs(10)).await;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to send tokens from {:?} to master: {:?}, retrying...",
                                            address,
                                            e
                                        );
                                        tokio::time::sleep(Duration::from_secs(10)).await;
                                    }
                                }
                            }
                        }
                        ethers::types::U256::from(0)
                    }
                })
                .collect();

            while let Some(amount_to_send) = futures.next().await {
                total_collected += amount_to_send;
            }

            // Just to be sure all transactions are mined
            tokio::time::sleep(Duration::from_secs(10)).await; // TODO: Adjust sleep duration as needed

            tracing::info!(
                "Gathering from users end for contract: {:?}, total collected: {:?}. Distribution from master started.",
                contract_address,
                total_collected
            );

            // Distribute collected tokens back to signers, keeping 1 token in master account
            if total_collected > ethers::types::U256::from(1e6 as u64) {
                // Adjust threshold as needed
                let amount_to_distribute = total_collected - ethers::types::U256::from(1e6 as u64); // Keep 1 token
                let num_signers = self.signer_middlewares.len();
                let amount_per_signer =
                    amount_to_distribute / ethers::types::U256::from(num_signers as u64);

                for signer_middleware in self.signer_middlewares.iter() {
                    let provider = provider.clone();
                    let address = signer_middleware.lock().await.address();
                    let tx = erc20.transfer(address, amount_per_signer).tx;

                    for _ in 0..3 {
                        let pending_tx = master_middleware.send_transaction(tx.clone(), None).await;
                        match pending_tx {
                            Ok(pending_tx) => {
                                if pending_tx.await.is_ok() {
                                    tracing::info!(
                                        chain_id = chain_id,
                                        "Distributed {:?} tokens to {:?}",
                                        amount_per_signer,
                                        address
                                    );
                                    tokio::time::sleep(Duration::from_secs(10)).await;
                                    break;
                                } else {
                                    tracing::error!(
                                        "Failed to distribute tokens to {:?}, retrying...",
                                        address
                                    );
                                    tokio::time::sleep(Duration::from_secs(10)).await;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to distribute tokens to {:?}: {:?}, retrying...",
                                    address,
                                    e
                                );
                                tokio::time::sleep(Duration::from_secs(10)).await;
                            }
                        }
                    }
                }
            } else {
            }
        }
        tracing::info!("Distribution token from master finalized.");
    }

    async fn native_token_distribution(&self) {
        let provider = self.rpc.provider.clone();
        let master_acc = self.master_account.clone().value();
        let addr_of_master: ethers::types::H160 = secret_key_to_address(&master_acc);

        let chain_id = match provider.get_chainid().await {
            Ok(chain_id) => chain_id.as_u64(),
            Err(e) => {
                tracing::error!("Failed to get chain ID: {:?}", e);
                return;
            }
        };

        let master_wallet =
            LocalWallet::new_with_signer(master_acc.clone(), addr_of_master, chain_id);

        let master_middleware = Arc::new(SignerMiddleware::new(
            NonceManagerMiddleware::new(provider.clone(), addr_of_master),
            master_wallet.clone(),
        ));

        // Collect ETH from signers
        let mut total_collected = match provider.get_balance(addr_of_master, None).await {
            Ok(balance) => balance,
            Err(e) => {
                tracing::error!("Failed to get master account balance: {:?}", e);
                return;
            }
        };
        tracing::info!(
            chain_id = chain_id,
            "Gathering from users started for native token."
        );

        let mut futures: FuturesUnordered<_> = self.signer_middlewares
            .iter()
            .map(|signer_middleware| {
                let provider = provider.clone();
                let signer_middleware = signer_middleware.clone();
                let addr_of_master = addr_of_master.clone();

                async move {
                    let signer_middleware = signer_middleware.lock().await;
                    let address = signer_middleware.address();
                    let balance = match provider.get_balance(address, None).await {
                        Ok(balance) => balance,
                        Err(e) => {
                            tracing::error!(
                                "Failed to get balance for address {:?}: {:?}",
                                address,
                                e
                            );
                            return ethers::types::U256::from(0);
                        }
                    };

                    if balance > ethers::types::U256::from(1e16 as u64) {
                        // 0.01 ETH in wei
                        let amount_to_send = balance - ethers::types::U256::from(1e16 as u64); // Keep 0.01 ETH
                        let tx = TransactionRequest::new().to(addr_of_master).value(amount_to_send);

                        for _ in 0..2 {
                            let pending_tx = signer_middleware.send_transaction(
                                tx.clone(),
                                None
                            ).await;
                            match pending_tx {
                                Ok(pending_tx) => {
                                    if pending_tx.await.is_ok() {
                                        return amount_to_send;
                                    } else {
                                        tracing::error!(
                                            "Failed to send transaction from {:?} to master, retrying...",
                                            address
                                        );
                                        tokio::time::sleep(Duration::from_secs(10)).await;
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to send transaction from {:?} to master: {:?}, retrying...",
                                        address,
                                        e
                                    );
                                    tokio::time::sleep(Duration::from_secs(10)).await;
                                }
                            }
                        }
                    }
                    ethers::types::U256::from(0)
                }
            })
            .collect();

        while let Some(amount_to_send) = futures.next().await {
            total_collected += amount_to_send;
        }

        // Just to be sure all transactions are mined
        tokio::time::sleep(Duration::from_secs(10)).await; // TODO(caglankaan): Do we really need to wait 10 sec? idk

        tracing::info!(
            "Gathering from users end, total collected: {:?}. Distribution from master started.",
            total_collected
        );

        // Distribute collected ETH back to signers, keeping 1 ETH in master account
        if total_collected > ethers::types::U256::from(1e17 as u64) {
            // More than 1 ETH collected
            let amount_to_distribute = total_collected - ethers::types::U256::from(1e17 as u64); // Keep 1 ETH
            let num_signers = self.signer_middlewares.len();
            let amount_per_signer =
                amount_to_distribute / ethers::types::U256::from(num_signers as u64);

            for signer_middleware in self.signer_middlewares.iter() {
                let provider = provider.clone();
                let address = signer_middleware.lock().await.address();

                let tx = TransactionRequest::new()
                    .to(address)
                    .value(amount_per_signer);

                for _ in 0..2 {
                    let pending_tx = master_middleware.send_transaction(tx.clone(), None).await;
                    match pending_tx {
                        Ok(pending_tx) => {
                            if pending_tx.await.is_ok() {
                                tracing::info!(
                                    chain_id = chain_id,
                                    "Distributed {:?} wei to {:?}",
                                    amount_per_signer,
                                    address
                                );
                                tokio::time::sleep(Duration::from_secs(10)).await;
                                break;
                            } else {
                                tracing::error!(
                                    "Failed to distribute to {:?}, retrying...",
                                    address
                                );
                                tokio::time::sleep(Duration::from_secs(10)).await;
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to distribute to {:?}: {:?}, retrying...",
                                address,
                                e
                            );
                            tokio::time::sleep(Duration::from_secs(10)).await;
                        }
                    }
                }
            }
        }
        tracing::info!("Distribution native token from master finalized.");
    }

    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
        memo: String,
        max_retry: u64,
    ) {
        let mut rng = StdRng::from_entropy();
        let index = rng.gen_range(0..self.relays.len());
        let relay = &self.relays[index];
        let signer_middleware = self.signer_middlewares[index].lock().await;
        let msg_sender = self.msg_senders[index];

        let denom_address = match ethers::types::H160::from_str(&denom) {
            Ok(address) => address,
            Err(_) => {
                let formatted_denom = format!(
                    "{}/{}/{}",
                    self.ucs01_contract.to_lowercase(),
                    destination_channel,
                    denom
                );

                relay
                    .get_denom_address(
                        destination_channel.clone().to_string(),
                        formatted_denom.clone(),
                    )
                    .call()
                    .await
                    .unwrap()
            }
        };

        if denom_address == ethers::types::H160::zero() {
            tracing::error!("Denom address not found");
            return;
        }
        let erc_contract = erc20::ERC20::new(denom_address, signer_middleware.clone());
        tracing::info!(
            "ERC20 contract address: {:?}, denom: {}, channel: {}",
            denom_address,
            denom,
            destination_channel
        );
        let balance = erc_contract.balance_of(msg_sender).await.unwrap();
        tracing::info!(
            "ETH Token({:?}) balance: {} of account: {:?}. Sending amount: {}",
            denom_address,
            balance,
            msg_sender,
            amount
        );

        // if balance < ethers::types::U256::exp10(9) {
        //     tracing::error!(
        //         "[INSUFFICIENT ERC20 BALANCE] Current balance is: {}. It should always be higher than: {}. Address: {:?}, ERC20 Address: {:?}",
        //         balance,
        //         ethers::types::U256::exp10(9).to_string(),
        //         msg_sender,
        //         erc_contract
        //     );
        // }

        if balance < amount.into() {
            tracing::warn!(
                "Insufficient balance: {}. Requested amount: {}",
                balance,
                amount
            );
            return;
        }

        let allowance = erc_contract
            .allowance(msg_sender, self.relay_addr)
            .await
            .unwrap();
        if allowance < amount.into() {
            let _ = erc_contract
                .approve(self.relay_addr, (U256::MAX / U256::from(2)).into())
                .send()
                .await;
            tracing::info!("We can not transfer after approve, returning now.");
            return;
        }

        let mut debug_msg;
        match protocol {
            Protocol::Ucs01 {
                ref receivers,
                ref contract,
            } => {
                let mut rng = StdRng::from_entropy();
                let index = rng.gen_range(0..receivers.len()); // Select a random index

                let receiver = &receivers[index];

                let mut final_receiver = receiver.encode_to_vec().into();

                if memo.is_empty() {
                    let (_hrp, data, _variant) =
                        bech32::decode(&receiver).expect("Invalid Bech32 address");

                    let bytes: Vec<u8> =
                        Vec::<u8>::from_base32(&data).expect("Invalid base32 data");

                    final_receiver = bytes.into();
                }

                debug_msg = format!(
                    "[Ethereum] -> Sent IBC transfer. memo: {}. Sending denom: {}. To: {:?}. Amount: {}, contract: {}, from {:?}",
                    memo,
                    denom,
                    final_receiver,
                    amount,
                    contract,
                    msg_sender
                );

                let tx_rcp: Option<ethers::types::TransactionReceipt> = match relay
                    .send(
                        destination_channel.clone().to_string(),
                        final_receiver,
                        [LocalToken {
                            denom: denom_address,
                            amount: amount as u128,
                        }]
                        .into(),
                        memo.clone(),
                        (Height {
                            revision_number: 0,
                            revision_height: 0,
                        })
                        .into(),
                        u64::MAX,
                    )
                    .send()
                    .await
                {
                    Ok(response) => match response.await {
                        Ok(receipt) => Some(receipt.expect("Failed to get transaction receipt")),
                        Err(e) => {
                            tracing::error!("Failed to get transaction receipt: {:?}", e);
                            return;
                        }
                    },
                    Err(ethers::contract::ContractError::MiddlewareError { e }) => {
                        if e.to_string().contains("replacement transaction underprice") {
                            if max_retry == 0 {
                                tracing::warn!(
                                    "Replacement transaction underprice. No more retrying"
                                );
                            } else {
                                tracing::info!("Retrying transaction.");
                                self.send_ibc_transfer(
                                    protocol,
                                    channel,
                                    destination_channel,
                                    denom,
                                    amount,
                                    memo,
                                    max_retry - 1,
                                );
                            }
                        } else {
                            tracing::error!(
                                "MiddlewareError Failed to send transaction eth->union: {:?}",
                                e.to_string()
                            );
                        }
                        return;
                    }
                    Err(e) => {
                        if e.to_string().contains("nonce too low")
                            || e.to_string()
                                .contains("Contract call reverted with data: 0x")
                        {
                            if max_retry == 0 {
                                tracing::warn!("No more retrying");
                            } else {
                                tracing::info!(
                                    "Retrying transaction. msg_sender: {:?}",
                                    msg_sender
                                );
                                self.send_ibc_transfer(
                                    protocol,
                                    channel,
                                    destination_channel,
                                    denom,
                                    amount,
                                    memo,
                                    max_retry - 1,
                                );
                            }
                        } else {
                            tracing::error!("Failed to send transaction eth->union: {:?}", e);
                        }
                        return;
                    }
                };
                let tx_hash = format!("{:?}", tx_rcp.unwrap().transaction_hash);
                debug_msg.push_str(&format!(" Tx Hash: {}", tx_hash));

                tracing::info!(debug_msg);
            }
            Protocol::Ics20 {
                receivers: _,
                module: _,
            } => {
                unimplemented!("Ics20 protocol not implemented"); // TODO: Do we even have this case?
            }
        }
    }
}

impl Ethereum {
    pub async fn new(config: EthereumConfig) -> Self {
        let ethereum_rpc = EthereumRpc {
            provider: Arc::new(Provider::new(
                Ws::connect(config.eth_rpc_api.clone()).await.unwrap(),
            )),
            ibc_handler_address: config.ibc_handler_address,
        };

        let mut relays = Vec::new();
        let mut signers_middleware = Vec::new();
        let mut msg_senders = Vec::new();

        let (relay_addr, ucs01_contract) = match config.transfer_module {
            TransferModule::Contract { ref address } => {
                let relay_addr: Address = address.parse().expect("Invalid contract address");
                (relay_addr, address.clone())
            }
            TransferModule::Native => {
                panic!("Native transfer module is not supported in this context")
            }
        };
        for signer in config.signers.clone() {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address_of_privkey: ethers::types::H160 = secret_key_to_address(&signing_key);
            tracing::info!("address: {:?}", address_of_privkey);

            let provider: Arc<Provider<Ws>> = ethereum_rpc.provider.clone();

            let chain_id = provider
                .get_chainid()
                .await
                .expect("Failed to get chain ID")
                .as_u64();
            let wallet = LocalWallet::new_with_signer(signing_key, address_of_privkey, chain_id);

            let signer_middleware = Arc::new(SignerMiddleware::new(
                NonceManagerMiddleware::new(provider.clone(), address_of_privkey),
                wallet.clone(),
            ));

            let relay = UCS01Relay::new(relay_addr, signer_middleware.clone());

            relays.push(relay);
            signers_middleware.push(tokio::sync::Mutex::new(signer_middleware));
            msg_senders.push(address_of_privkey);
        }
        Ethereum {
            rpc: ethereum_rpc,
            relays,
            signer_middlewares: Arc::new(signers_middleware),
            ucs01_contract,
            msg_senders,
            relay_addr,
            master_account: config.master_account,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cosmos {
    pub chain: chain_utils::cosmos::Cosmos,
}

#[derive(Debug, Clone)]
pub struct Union {
    pub chain: chain_utils::union::Union,
}

impl IbcTransfer for Cosmos {
    async fn native_token_distribution(&self) {
        unimplemented!()
    }

    async fn token_distribution(&self, tokens: Vec<H160>) {
        unimplemented!()
    }

    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
        memo: String,
        max_retry: u64,
    ) {
        self.chain.keyring.with(|signer| async move {
            let mut debug_msg;
            let transfer_msg = match protocol {
                Protocol::Ics20 { ref receivers, ref module } => {
                    let mut rng = StdRng::from_entropy();

                    let receiver = match receivers.choose(&mut rng) {
                        Some(receiver) => receiver,
                        None => {
                            tracing::error!("No receiver found.");
                            return;
                        }
                    };

                    let msg = MsgTransfer {
                        source_port: "transfer".into(),
                        source_channel: destination_channel.to_string(),
                        token: Some(
                            (Coin {
                                denom: denom.to_string(),
                                amount: amount as u128,
                            }).into()
                        ),
                        sender: signer.to_string(),
                        receiver: receiver.to_string(),
                        timeout_height: None,
                        timeout_timestamp: u64::MAX / 2,
                        memo: memo.clone(),
                    };

                    debug_msg = format!(
                        "[Cosmos Ics20] -> SENT IBC transfer from: {}. memo: {}. denom: {}. To: {}. Amount: {}, module: {:?}",
                        signer.to_string(),
                        memo,
                        denom,
                        receiver,
                        amount,
                        module
                    );

                    Any {
                        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                        value: msg.encode_to_vec().into(),
                    }
                }
                Protocol::Ucs01 { ref receivers, ref contract } => {
                    let mut rng = StdRng::from_entropy();
                    let receiver = match receivers.choose(&mut rng) {
                        Some(receiver) => receiver,
                        None => {
                            tracing::error!("No receiver found.");
                            return;
                        }
                    };

                    let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                        channel: destination_channel.to_string(),
                        receiver: receiver[2..].to_string(),
                        memo: memo.clone(),
                        timeout: None,
                    });

                    // TODO(caglankaan): This part is not clear right now. For the first version
                    // It would be better to get denom directly as smth like
                    // `factory/union1m37cxl0ld4uaw3r4lv9nt2uw69xxf8xfjrf7a4w9hamv6xvp6ddqqfaaaa/0xe619529b4396a62ab6d88ff2bb195e83c11e909ad9`
                    // for USDC for example. The code below works but it would be hard to check if its native or smth else.
                    // For next version(s) we can add a feature to calculate this with like
                    // "token": {
                    //     "protocol": "union",
                    //     "type": "native",
                    //     "denom": "muno"
                    // }

                    // let foreign_denom = format!(
                    //     "wasm.{}/{}/{}",
                    //     contract,
                    //     destination_channel.to_string(),
                    //     denom.to_lowercase()
                    // );
                    // let hashed_foreign_denom = hash_denom_str(&foreign_denom);

                    // let final_denom = format!(
                    //     "factory/{}/{}",
                    //     contract.to_string(),
                    //     hashed_foreign_denom
                    // );

                    let transfer_msg_bytes = serde_json::to_vec(&transfer_msg).unwrap();

                    debug_msg = format!(
                        "[Cosmos] -> SENT IBC transfer from: {}. memo: {}. denom: {}. To: {}. Amount: {}, contract: {}",
                        signer.to_string(),
                        memo,
                        denom,
                        receiver,
                        amount,
                        contract
                    );

                    any::Any(MsgExecuteContract {
                        sender: signer.to_string(),
                        contract: contract.clone(),
                        msg: transfer_msg_bytes,
                        funds: vec![Coin {
                            denom: denom.to_string(),
                            amount: amount as u128,
                        }],
                    }).into()
                }
            };

            match self.chain.broadcast_tx_commit(signer, [transfer_msg]).await {
                Ok(tx_hash) => {
                    debug_msg.push_str(
                        &format!(" Tx Hash: {}", tx_hash.to_string()[2..].to_ascii_uppercase())
                    );
                    tracing::info!(debug_msg);
                }
                Err(BroadcastTxCommitError::SimulateTx(e)) => {
                    if e.contains("account sequence mismatch") {
                        tracing::warn!("Account sequence mismatch.");
                        if max_retry == 0 {
                            tracing::warn!("Account sequence mismatch. No more retrying");
                        } else {
                            tracing::info!("Retrying transaction.");
                            self.send_ibc_transfer(
                                protocol,
                                channel,
                                destination_channel,
                                denom,
                                amount,
                                memo,
                                max_retry - 1
                            );
                        }
                    } else {
                        tracing::error!("Failed to simulate tx!{:?}", e.to_string());
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to submit tx!{:?}", e.to_string());
                }
            }
        }).await;
    }
}

impl Cosmos {
    pub async fn new(config: CosmosConfig) -> Self {
        let cosmos = chain_utils::cosmos::Cosmos::new(config.chain_config)
            .await
            .unwrap();

        Cosmos { chain: cosmos }
    }
}

async fn get_channel_for_eth_ack_packet(
    eth_rpcs: &EthereumRpc,
    port_id: String,
    channel_id: String,
    block_number: u64,
) -> Option<Channel> {
    let channel_result = eth_rpcs
        .ibc_handler()
        .get_channel(port_id.clone(), channel_id.clone())
        .block(block_number)
        .await;

    match channel_result {
        Ok(channel) => channel.try_into().ok(),
        Err(e) => {
            tracing::error!(
                "Failed to fetch channel for port: {}, channel: {}. Error: {}",
                port_id,
                channel_id,
                e
            );
            return None;
        }
    }
}

async fn ibchandler_events_to_ibc_event(
    log: RawLog,
    eth_rpcs: &EthereumRpc,
    block_number: u64,
) -> Option<IbcEvent> {
    match IBCHandlerEvents::decode_log(&log) {
        Ok(event) => {
            // Handle the decoded event similarly to Tendermint events
            match event {
                IBCHandlerEvents::PacketEvent(packet_event) => match packet_event {
                    IBCPacketEvents::SendPacketFilter(event) => {
                        if let Some(channel) = get_channel_for_eth_ack_packet(
                            eth_rpcs,
                            event.source_port.clone(),
                            event.source_channel.clone(),
                            block_number,
                        )
                        .await
                        {
                            Some(IbcEvent::SendPacket(SendPacket {
                                packet_sequence: event.sequence.try_into().unwrap(),
                                packet_src_port: event.source_port.parse().unwrap(),
                                packet_src_channel: event.source_channel.parse().unwrap(),
                                packet_dst_port: channel.counterparty.port_id,
                                packet_dst_channel: channel
                                    .counterparty
                                    .channel_id
                                    .to_string()
                                    .parse()
                                    .unwrap(),
                                packet_timeout_height: event.timeout_height.into(),
                                packet_timeout_timestamp: event.timeout_timestamp,
                                packet_data_hex: hex_encode(event.data).into(),
                                packet_channel_ordering: channel.ordering,
                                connection_id: channel.connection_hops[0].clone(),
                            }))
                        } else {
                            None
                        }
                    }
                    IBCPacketEvents::RecvPacketFilter(event) => {
                        if let Some(channel) = get_channel_for_eth_ack_packet(
                            eth_rpcs,
                            event.packet.destination_port.clone(),
                            event.packet.destination_channel.clone(),
                            block_number,
                        )
                        .await
                        {
                            Some(IbcEvent::RecvPacket(RecvPacket {
                                packet_sequence: event.packet.sequence.try_into().unwrap(),
                                packet_src_port: event.packet.source_port.parse().unwrap(),
                                packet_src_channel: event.packet.source_channel.parse().unwrap(),
                                packet_dst_port: event.packet.destination_port.parse().unwrap(),
                                packet_dst_channel: event
                                    .packet
                                    .destination_channel
                                    .parse()
                                    .unwrap(),
                                packet_timeout_height: event.packet.timeout_height.into(),
                                packet_timeout_timestamp: event.packet.timeout_timestamp,
                                packet_data_hex: hex_encode(event.packet.data).into(),
                                packet_channel_ordering: channel.ordering,
                                connection_id: channel.connection_hops[0].clone(),
                            }))
                        } else {
                            tracing::error!(
                                "Could not find channel for packet: {:?}",
                                event.packet
                            );
                            None
                        }
                    }
                    IBCPacketEvents::AcknowledgePacketFilter(event) => {
                        if let Some(channel) = get_channel_for_eth_ack_packet(
                            eth_rpcs,
                            event.packet.source_port.clone(),
                            event.packet.source_channel.clone(),
                            block_number,
                        )
                        .await
                        {
                            Some(IbcEvent::AcknowledgePacket(AcknowledgePacket {
                                packet_sequence: event.packet.sequence.try_into().unwrap(),
                                packet_src_port: event.packet.source_port.parse().unwrap(),
                                packet_src_channel: event.packet.source_channel.parse().unwrap(),
                                packet_dst_port: event.packet.destination_port.parse().unwrap(),
                                packet_dst_channel: event
                                    .packet
                                    .destination_channel
                                    .parse()
                                    .unwrap(),
                                packet_timeout_height: event.packet.timeout_height.into(),
                                packet_timeout_timestamp: event.packet.timeout_timestamp,
                                packet_channel_ordering: channel.ordering,
                                connection_id: channel.connection_hops[0].clone(),
                            }))
                        } else {
                            None
                        }
                    }
                    IBCPacketEvents::WriteAcknowledgementFilter(event) => {
                        if let Some(channel) = get_channel_for_eth_ack_packet(
                            eth_rpcs,
                            event.packet.destination_port.clone(),
                            event.packet.destination_channel.clone(),
                            block_number,
                        )
                        .await
                        {
                            Some(IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
                                packet_sequence: event.packet.sequence.try_into().unwrap(),
                                packet_src_port: event
                                    .packet
                                    .source_port
                                    .to_string()
                                    .parse()
                                    .unwrap(),
                                packet_src_channel: event.packet.source_channel.parse().unwrap(),
                                packet_dst_port: event.packet.destination_port.parse().unwrap(),
                                packet_dst_channel: event
                                    .packet
                                    .destination_channel
                                    .parse()
                                    .unwrap(),
                                packet_timeout_height: Height {
                                    revision_number: 0,
                                    revision_height: 0,
                                },
                                packet_ack_hex: event.acknowledgement.to_vec(),
                                packet_data_hex: hex_encode(event.packet.data).into(),
                                packet_timeout_timestamp: 0,
                                connection_id: channel.connection_hops[0].clone(),
                            }))
                        } else {
                            None
                        }
                    }
                    _ => {
                        tracing::warn!("Unhandled packet event type.");
                        None
                    }
                },
                _ => {
                    // tracing::warn!("Unhandled event type.");
                    None
                }
            }
        }
        Err(_) => {
            tracing::warn!("Could not decode Ethereum log event.");
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ChainType {
    Ethereum,
    Cosmos,
}

fn get_chain_type(chain_id: &str) -> ChainType {
    // TODO fill these with other chain ids
    let ethereum_chain_ids = vec!["11155111"];
    let cosmos_chain_ids = vec!["union-testnet-8"];

    if ethereum_chain_ids.contains(&chain_id) {
        ChainType::Ethereum
    } else if cosmos_chain_ids.contains(&chain_id) {
        ChainType::Cosmos
    } else if chain_id.parse::<u64>().is_ok() {
        ChainType::Ethereum
    } else {
        ChainType::Cosmos
    }
}
