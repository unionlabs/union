use core::future::Future;
use std::{any::Any, collections::HashMap, pin::Pin, sync::Arc};

use bech32::FromBase32;
use chain_utils::{
    cosmos_sdk::CosmosSdkChainExt,
    ethereum::{EthereumChain, IBCHandlerEvents},
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
    core::k256::ecdsa::{self, SigningKey},
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, Ws},
    signers::LocalWallet,
    types::{Address, Filter, H160},
    utils::secret_key_to_address,
};
use futures::StreamExt;
use hex::{decode as hex_decode, encode as hex_encode};
use protos::ibc::{applications::transfer::v1::MsgTransfer, core::channel};
use serde_json::{from_value, to_value};
use tendermint_rpc::{
    event::{Event, EventData},
    SubscriptionClient, WebSocketClient,
};
use tokio::{
    sync::Mutex,
    time::{interval, Duration},
};
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use ucs01_relay_api::types::{Ics20Ack, Ucs01Ack};
use unionlabs::{
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    encoding::Proto,
    ethereum::config::Minimal,
    events::{AcknowledgePacket, RecvPacket, SendPacket, WriteAcknowledgement},
    ibc::core::client::height::Height,
    id::ClientId,
    tendermint::abci::{event::Event as TendermintEvent, event_attribute::EventAttribute},
    uint::U256,
    validated::ValidateT,
};

use crate::{
    config::{Chain, ChainConfig, ChainId, Config, CosmosConfig, PacketStatus, Protocol},
    // datadog::{ log_builder, send_log_to_datadog },
    sql_helper::{
        delete_packet_status, get_packet_status, get_packet_statuses,
        insert_or_update_packet_status,
    },
};

pub type IbcEvent = unionlabs::events::IbcEvent<ClientId, String, ClientId>;

#[derive(Clone, Debug)]
pub struct Context {
    pub transfer_test_config: Config,
    pub union: Option<chain_utils::union::Union>,
    pub osmosis: Option<chain_utils::cosmos::Cosmos>,
    pub ethereum: Option<chain_utils::ethereum::Ethereum<Minimal>>,
    pub ethereum_config: Option<EthereumConfig>,
    pub union_txs: Arc<Mutex<HashMap<u64, uuid::Uuid>>>,
    pub osmosis_txs: Arc<Mutex<HashMap<u64, uuid::Uuid>>>,
    // pub datadog_data: DatadogData,
    pub packet_statuses: Arc<Mutex<HashMap<u64, PacketStatus>>>,
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

#[derive(Clone, Debug)]
pub struct EthereumConfig {
    // pub chain_id: u64,
    pub address: Address,
    // pub signer_middleware: Arc<
    //     SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>
    // >,
    // pub contract_address: Address,
    pub relay: UCS01Relay<SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>>,
    pub erc_contract:
        erc20::ERC20<SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>>,
    pub denom_address: ethers::types::H160,
}

// Define the IbcTransfer trait
pub trait IbcTransfer<S: IbcTransferMain, D: IbcTransferMain> {
    async fn send_ibc_transfer(&self, direction: TransferDirection<S, D>);
}

pub trait IbcTransferMain {
    async fn send_ibc_transfer(&self, amount: &str, receiver: &str);

    fn receiver(&self) -> String;
}

impl<T: CosmosIbcTransfer> IbcTransferMain for CosmosConfig<T> {
    async fn send_ibc_transfer(&self, amount: &str, receiver: &str) {
        self.chain_config
            .send_ibc_transfer(&self.protocol, amount, receiver)
            .await;
    }

    fn receiver(&self) -> String {
        self.chain_config.receiver()
    }
}

impl IbcTransferMain
    for ChainConfig<
        (
            PrivateKey<ecdsa::SigningKey>,
            chain_utils::ethereum::Ethereum<Minimal>,
        ),
        H160,
    >
{
    // TODO(aeryz): do most of these right at the beginning in the config phase
    async fn send_ibc_transfer(&self, amount: &str, receiver: &str) {
        let denom = format!(
            "{}/{}/muno",
            self.address.to_string().to_lowercase(),
            self.counterparty_channel,
        );

        let provider: Arc<Provider<Ws>> = self.chain_config.1.provider().clone();
        let wallet = LocalWallet::new_with_signer(
            self.chain_config.0.clone().value(),
            self.address,
            self.chain_config.1.chain_id.try_into().unwrap(),
        );
        let signer_middleware = Arc::new(SignerMiddleware::new(
            NonceManagerMiddleware::new(provider.clone(), self.address),
            wallet.clone(),
        ));
        let relay = UCS01Relay::new(self.address, signer_middleware.clone());
        let denom_address = relay
            .get_denom_address(self.counterparty_channel.clone(), denom.clone())
            .call()
            .await
            .unwrap();
        println!("denom address: {}", denom_address);

        relay
            .send(
                self.channel.clone(),
                hex_decode(receiver).unwrap().into(),
                [LocalToken {
                    denom: denom_address,
                    amount: amount.parse().unwrap(),
                }]
                .into(),
                Default::default(),
                Height {
                    revision_number: 0,
                    revision_height: u32::MAX as u64,
                }
                .into(),
                u64::MAX,
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
    }

    fn receiver(&self) -> String {
        self.address.to_string()
    }
}

pub trait CosmosIbcTransfer {
    async fn send_ibc_transfer(&self, protocol: &Protocol, amount: &str, receiver: &str);

    fn receiver(&self) -> String;
}

impl CosmosIbcTransfer for ChainConfig<chain_utils::union::Union, String> {
    async fn send_ibc_transfer(&self, protocol: &Protocol, amount: &str, receiver: &str) {
        self.chain_config
            .signers
            .with(|signer| async move {
                let msg = protocol.transfer_message(
                    &signer.to_string(),
                    &self.channel,
                    &self.chain_config.fee_denom,
                    amount,
                    receiver,
                );
                self.chain_config
                    .broadcast_tx_commit(signer, [msg])
                    .await
                    .unwrap();
            })
            .await;
    }

    fn receiver(&self) -> String {
        self.address.clone()
    }
}

impl CosmosIbcTransfer for ChainConfig<chain_utils::cosmos::Cosmos, String> {
    async fn send_ibc_transfer(&self, protocol: &Protocol, amount: &str, receiver: &str) {
        self.chain_config
            .signers
            .with(|signer| async move {
                let msg = protocol.transfer_message(
                    &signer.to_string(),
                    &self.channel,
                    &self.chain_config.fee_denom,
                    amount,
                    receiver,
                );

                self.chain_config
                    .broadcast_tx_commit(signer, [msg])
                    .await
                    .unwrap();
            })
            .await;
    }

    fn receiver(&self) -> String {
        self.address.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TransferDirection<S: IbcTransferMain, D: IbcTransferMain> {
    pub source_chain: S,
    pub destination_chain: D,
}

pub trait TendermintClient {
    fn tm_client(&self) -> &WebSocketClient;
}

pub trait EthereumClient {}

impl TendermintClient for chain_utils::cosmos::Cosmos {
    fn tm_client(&self) -> &WebSocketClient {
        &self.tm_client
    }
}

impl TendermintClient for chain_utils::union::Union {
    fn tm_client(&self) -> &WebSocketClient {
        &self.tm_client
    }
}

impl EthereumClient for chain_utils::ethereum::Ethereum<Minimal> {}

pub trait ChainListener: Sync + Send {
    fn listen<'a>(
        &'a self,
        context: &'a Context,
        source_chain: &'a str,
        target_chain: &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

impl<T> ChainListener for T
where
    T: TendermintClient + Sync + Send + 'static,
{
    fn listen<'a>(
        &'a self,
        context: &'a Context,
        source_chain: &'a str,
        target_chain: &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            tracing::info!("Listening for events on {}.", source_chain);
            let mut subs = self
                .tm_client()
                .subscribe(tendermint_rpc::query::EventType::Tx.into())
                .await
                .unwrap();
            loop {
                tokio::select! {
                    Some(event_result) = subs.next() => {
                        match event_result {
                            Ok(event) => {
                                context.handle_tendermint_tx_event(event, source_chain, target_chain).await;
                            }
                            Err(e) => {
                                tracing::error!("Error while receiving event: {:?}", e);
                            }
                        }
                    },
                    else => break,
                }
            }
        })
    }
}

impl ChainListener for chain_utils::ethereum::Ethereum<Minimal> {
    fn listen<'a>(
        &'a self,
        context: &'a Context,
        source_chain: &'a str,
        target_chain: &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        tracing::info!("Listening for events on {:?}.", source_chain);
        Box::pin(async move {
            let provider: Arc<Provider<Ws>> = self.provider().clone();
            let mut latest_block = provider.get_block_number().await.unwrap().as_u64();
            tracing::info!("latest_block {:?}.", latest_block);
            tracing::info!("ibc_handler_address {:?}.", self.ibc_handler_address());
            loop {
                // Update the filter to fetch logs from the latest block processed + 1
                let filter = Filter::new()
                    .address(ethers::types::H160::from(self.ibc_handler_address()))
                    .from_block(latest_block)
                    .to_block(latest_block);

                let logs = provider.get_logs(&filter).await.unwrap();

                let logs_clone = logs.clone(); // Clone logs for processing
                futures::stream::iter(logs_clone)
                    .filter_map(|log| async move {
                        let raw_log = RawLog {
                            topics: log.topics.clone(),
                            data: log.data.clone().to_vec(),
                        };

                        Some(raw_log)
                    })
                    .for_each_concurrent(None, |raw_log| async move {
                        context
                            .handle_ethereum_log_event(raw_log, source_chain, target_chain)
                            .await;
                    })
                    .await;

                // Update the latest block to the most recent block fetched
                if let Some(last_log) = logs.last() {
                    latest_block = last_log.block_number.unwrap().as_u64() + 1;
                } else {
                    // If no logs were found, just increment the block number to avoid getting stuck
                    latest_block += 1;
                }

                // TODO: 6 seconds for new block fetching?
                tokio::time::sleep(Duration::from_secs(6)).await;
            }
        })
        // unimplemented!()
    }
}

impl Context {
    pub async fn new(transfer_test_config: Config, pool: sqlx::Pool<sqlx::Postgres>) -> Context {
        let mut union = None;
        let mut osmosis = None;
        let mut ethereum = None;
        let mut ethereum_config = None;

        for connection in &transfer_test_config.connections {
            match connection.source_chain.as_str() {
                "union" if union.is_none() => {
                    union = Some(
                        chain_utils::union::Union::new(
                            transfer_test_config.clone().union.chain_config.chain_config,
                        )
                        .await
                        .unwrap(),
                    );
                    tracing::debug!("Created Union instance.");
                }
                "osmosis" if osmosis.is_none() => {
                    osmosis = Some(
                        chain_utils::cosmos::Cosmos::new(
                            transfer_test_config
                                .clone()
                                .osmosis
                                .chain_config
                                .chain_config,
                        )
                        .await
                        .unwrap(),
                    );
                    tracing::debug!("Created Osmosis instance.");
                }
                "ethereum" if ethereum.is_none() => {
                    ethereum = Some(
                        chain_utils::ethereum::Ethereum::new(
                            transfer_test_config.clone().ethereum.chain_config,
                        )
                        .await
                        .unwrap(),
                    );
                    tracing::info!("Created Ethereum instance.");
                    // Initialize Ethereum-specific configurations
                    let provider: Arc<Provider<Ws>> = ethereum.as_ref().unwrap().provider().clone();
                    let private_key_hex: &str = "&transfer_test_config.ethereum_priv_key";
                    let private_key_bytes =
                        hex_decode(private_key_hex).expect("Invalid private key hex");
                    let private_key = SigningKey::from_slice(&private_key_bytes)
                        .expect("Invalid private key bytes");
                    let address = secret_key_to_address(&private_key);
                    let chain_id = provider
                        .get_chainid()
                        .await
                        .expect("Failed to get chain ID")
                        .as_u64();
                    let wallet = LocalWallet::new_with_signer(private_key, address, chain_id);
                    let signer_middleware = Arc::new(SignerMiddleware::new(
                        NonceManagerMiddleware::new(provider.clone(), address),
                        wallet.clone(),
                    ));
                    let contract_address: Address = transfer_test_config
                        .ethereum
                        .address
                        .to_string()
                        .parse()
                        .expect("Invalid contract address");
                    let relay = UCS01Relay::new(contract_address, signer_middleware.clone());

                    let denom = format!(
                        "{}/{}/{}",
                        transfer_test_config
                            .ethereum
                            .address
                            .to_string()
                            .to_lowercase(),
                        transfer_test_config.ethereum.counterparty_channel,
                        transfer_test_config
                            .union
                            .chain_config
                            .chain_config
                            .fee_denom
                    );

                    let denom_address = relay
                        .get_denom_address(
                            transfer_test_config.ethereum.counterparty_channel.clone(),
                            denom.clone(),
                        )
                        .call()
                        .await
                        .unwrap();
                    tracing::info!("Corresponding ERC20 address: {}", denom_address);

                    let erc_contract = erc20::ERC20::new(denom_address, signer_middleware.clone());

                    erc_contract
                        .approve(contract_address, (U256::MAX / U256::from(2)).into())
                        .send()
                        .await
                        .unwrap()
                        .await
                        .unwrap()
                        .unwrap();

                    ethereum_config = Some(EthereumConfig {
                        // chain_id,
                        address,
                        // signer_middleware,
                        // contract_address,
                        relay,
                        erc_contract,
                        denom_address,
                    });
                    tracing::debug!("Created Ethereum config.");
                }
                _ => {}
            }
        }

        // let datadog_data = transfer_test_config.datadog_data.clone();
        Context {
            transfer_test_config,
            union: union,
            osmosis: osmosis,
            ethereum: ethereum,
            ethereum_config: ethereum_config,
            union_txs: Arc::new(Mutex::new(HashMap::new())),
            osmosis_txs: Arc::new(Mutex::new(HashMap::new())),
            // datadog_data,
            packet_statuses: Arc::new(Mutex::new(HashMap::new())),
            pool,
        }
    }

    async fn handle_ethereum_log_event(&self, log: RawLog, source_chain: &str, target_chain: &str) {
        let ibc_event = self.ibchandler_events_to_ibc_event(log).await;
        if let Some(ibc_event) = ibc_event {
            self.handle_ibc_event(ibc_event, source_chain, target_chain)
                .await;
        }
    }
    async fn handle_tendermint_tx_event(
        &self,
        event: Event,
        source_chain: &str,
        target_chain: &str,
    ) {
        match event.data {
            EventData::Tx { tx_result, .. } => {
                for event in tx_result.result.events {
                    let Some(my_event) = IbcEvent::try_from_tendermint_event(TendermintEvent {
                        ty: event.kind,
                        attributes: event
                            .attributes
                            .into_iter()
                            .map(|attr| EventAttribute {
                                key: attr.key,
                                value: attr.value,
                                index: attr.index,
                            })
                            .collect(),
                    }) else {
                        continue;
                    };
                    let ibc_event = my_event.unwrap();
                    self.handle_ibc_event(ibc_event, source_chain, target_chain)
                        .await;
                }
            }
            _ => {
                tracing::error!("Unhandled event type: {:?}", event);
            }
        }
    }

    async fn handle_ibc_event(&self, ibc_event: IbcEvent, source_chain: &str, target_chain: &str) {
        let packet_sequence = match ibc_event {
            IbcEvent::SendPacket(ref e) => Some(e.packet_sequence),
            IbcEvent::RecvPacket(ref e) => Some(e.packet_sequence),
            IbcEvent::WriteAcknowledgement(ref e) => Some(e.packet_sequence),
            IbcEvent::AcknowledgePacket(ref e) => Some(e.packet_sequence),
            _ => None,
        };
        if let Some(sequence) = packet_sequence {
            let mut packet_statuses = self.packet_statuses.lock().await;
            let mut sequences_to_remove: Vec<u64> = Vec::new();

            let protocol = if source_chain == "osmosis" || target_chain == "osmosis" {
                "Ics20"
            } else {
                "Ucs01"
            };

            let status = packet_statuses.entry(sequence.get()).or_insert_with(|| {
                PacketStatus::new(
                    source_chain,
                    target_chain,
                    protocol,
                    sequence.get().try_into().unwrap(),
                )
            });

            let mut should_insert_or_update = true;

            match ibc_event {
                IbcEvent::SendPacket(ref e) => {
                    status.send_packet = Some(
                        to_value(IbcEvent::SendPacket(e.clone())).expect("Serialization failed"),
                    );
                    status.last_update = chrono::Utc::now();
                    tracing::info!(
                        "SendPacket event. Sequence: {}. Direction: {}->{}", //, event: {:?}",
                        sequence,
                        source_chain,
                        target_chain
                    );
                }
                IbcEvent::RecvPacket(ref e) => {
                    if status.send_packet.is_some() {
                        status.recv_packet = Some(
                            to_value(IbcEvent::RecvPacket(e.clone()))
                                .expect("Serialization failed"),
                        );
                        tracing::info!(
                            "RecvPacket event. Sequence: {}. Direction: {}->{}",
                            sequence,
                            source_chain,
                            target_chain
                        );
                    } else {
                        tracing::warn!(
                            "RecvPacket without SendPacket. Sequence: {}. {} -> {}",
                            sequence,
                            source_chain,
                            target_chain
                        );
                        should_insert_or_update = false;
                    }
                }
                IbcEvent::WriteAcknowledgement(ref e) => {
                    if status.recv_packet.is_some() {
                        if self.write_handler_packet_ack_hex_controller(
                            e.packet_ack_hex.clone(),
                            &status.protocol,
                        ) {
                            status.write_ack = Some(
                                to_value(IbcEvent::WriteAcknowledgement(e.clone()))
                                    .expect("Serialization failed"),
                            );
                            tracing::info!(
                                "WriteAcknowledgement event. Sequence: {}. Direction: {}->{}",
                                sequence,
                                source_chain,
                                target_chain
                            );
                        } else {
                            tracing::error!(
                                "WriteAcknowledgement indicates failure. Sequence: {}.",
                                sequence
                            );
                            self.remove_packet_status(sequence.into(), &status).await;
                            should_insert_or_update = false;
                        }
                    } else {
                        tracing::warn!(
                            "WriteAcknowledgement without RecvPacket. Sequence: {}.",
                            sequence
                        );
                        should_insert_or_update = false;
                    }
                }
                IbcEvent::AcknowledgePacket(ref e) => {
                    status.acknowledge_packet = Some(
                        to_value(IbcEvent::AcknowledgePacket(e.clone()))
                            .expect("Serialization failed"),
                    );
                    tracing::info!(
                        "AcknowledgePacket event. Sequence: {}. Direction: {}->{}",
                        sequence,
                        source_chain,
                        target_chain
                    );

                    self.remove_packet_status(sequence.into(), &status).await;
                    tracing::info!(
                        "Packet status with sequence number {} deleted successfully. {} => {}",
                        status.sequence_number,
                        status.source_chain_id,
                        status.target_chain_id
                    );
                }
                _ => {
                    should_insert_or_update = false;
                }
            }

            if should_insert_or_update {
                insert_or_update_packet_status(&self.pool, status.clone())
                    .await
                    .unwrap();
            }
        }
    }

    async fn remove_packet_status(&self, sequence: u64, status: &PacketStatus) {
        delete_packet_status(
            &self.pool,
            status.source_chain_id,
            status.target_chain_id,
            status.sequence_number,
        )
        .await
        .unwrap();
        let mut packet_statuses = self.packet_statuses.lock().await;
        packet_statuses.remove(&sequence);
    }

    pub fn get_chain_listener(&self, chain_id: &ChainId) -> Option<&dyn ChainListener> {
        match chain_id {
            ChainId::Union => self.union.as_ref().map(|u| u as &dyn ChainListener),
            ChainId::Osmosis => self.osmosis.as_ref().map(|o| o as &dyn ChainListener),
            // Add other chain mappings as needed
            ChainId::Ethereum => self.ethereum.as_ref().map(|o| o as &dyn ChainListener), // Ethereum listener is not implemented yet
        }
    }

    pub async fn listen(&self, source_chain: &str, target_chain: &str) {
        let source_chain_id: ChainId =
            ChainId::from_str(source_chain).expect("Invalid source chain");
        tracing::debug!("Source chain: {}", source_chain);
        if let Some(listener) = self.get_chain_listener(&source_chain_id) {
            listener.listen(self, source_chain, target_chain).await;
        } else {
            tracing::warn!("No listener available for chain: {}", source_chain);
        }
    }

    // pub async fn send_ibc_transfer_eth_to_eth(&self, direction: &TransferDirection) {
    //     if
    //         let TransferDirection::FromEth {
    //             source_chain,
    //             target_chain,
    //             channel: _,
    //             contract: _,
    //             receiver_addr: _,
    //             is_receiver_bech32: _,
    //             amount: _,
    //         } = direction
    //     {
    //         tracing::info!("Sending IBC transfer from {} to {}.", source_chain, target_chain);

    //         let ethereum_config = self.ethereum_config
    //             .as_ref()
    //             .expect("Ethereum config not initialized");

    //         let erc_contract = ethereum_config.erc_contract.clone();
    //         let balance = erc_contract.balance_of(ethereum_config.address).await.unwrap();
    //         tracing::info!("Balance: {:?}, addr: {:?}", balance, ethereum_config.address);

    //         let (_hrp, data, _variant) = bech32
    //             ::decode(&self.transfer_test_config.union_contract)
    //             .expect("Invalid Bech32 address");

    //         let bytes = Vec::<u8>::from_base32(&data).expect("Invalid base32 data");
    //         let receiver = hex::encode(bytes);
    //         let pub_amount: u128 = self.transfer_test_config.amount.parse().unwrap();

    //         let _tx_rcp: Option<ethers::types::TransactionReceipt> = match
    //             ethereum_config.relay
    //                 .send(
    //                     self.transfer_test_config.counterparty_channel.clone(),
    //                     hex::decode(receiver).unwrap().into(),
    //                     [
    //                         LocalToken {
    //                             denom: ethereum_config.denom_address,
    //                             amount: pub_amount,
    //                         },
    //                     ].into(),
    //                     "".to_string(),
    //                     (Height {
    //                         revision_number: 0,
    //                         revision_height: 0,
    //                     }).into(),
    //                     u64::MAX
    //                 )
    //                 .send().await
    //         {
    //             Ok(response) =>
    //                 match response.await {
    //                     Ok(receipt) => receipt,
    //                     Err(e) => {
    //                         tracing::error!("Transaction failed: {:?}", e);
    //                         return;
    //                     }
    //                 }
    //             Err(e) => {
    //                 tracing::error!("Failed to send transaction: {:?}", e);
    //                 return;
    //             }
    //         };
    //     } else {
    //         tracing::error!("Invalid transfer direction for Ethereum to Ethereum.");
    //     }
    // }

    // pub async fn send_ibc_transfer_cosmos_to_cosmos(&self, direction: &TransferDirection) {
    //     match direction {
    //         TransferDirection::FromCosmos {
    //             source_chain,
    //             target_chain,
    //             channel,
    //             contract,
    //             receiver_bech32,
    //             is_receiver_eth,
    //             denom,
    //             amount,
    //         } => {
    //             tracing::info!("Sending IBC transfer from {} to {}.", source_chain, target_chain);
    //             let receiver: String;

    //             if !is_receiver_eth {
    //                 let (_hrp, data, _variant) = bech32
    //                     ::decode(&receiver_bech32)
    //                     .expect("Invalid Bech32 address");

    //                 let bytes = Vec::<u8>::from_base32(&data).expect("Invalid base32 data");
    //                 receiver = hex::encode(bytes);
    //             } else {
    //                 let ethereum_config = self.ethereum_config
    //                     .as_ref()
    //                     .expect("Ethereum config not initialized");
    //                 receiver = format!("{:?}", ethereum_config.address);
    //             }

    //             let uuid = uuid::Uuid::new_v4();

    //             // Create the transfer message
    //             let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
    //                 channel: channel.to_string(),
    //                 receiver,
    //                 memo: uuid.to_string(),
    //                 timeout: None,
    //             });

    //             let transfer_msg_bytes = serde_json::to_string(&transfer_msg).unwrap().into_bytes();

    //             if
    //                 let Some(signers) = (if source_chain == "osmosis" {
    //                     self.osmosis.as_ref().map(|o| o.signers.clone())
    //                 } else {
    //                     self.union.as_ref().map(|u| u.signers.clone())
    //                 })
    //             {
    //                 signers.with(|signer| async move {
    //                     tracing::info!("Sending Tx for {}.", signer.to_string());
    //                     let msg = Any(MsgExecuteContract {
    //                         sender: signer.to_string(),
    //                         contract: contract.clone(),
    //                         msg: transfer_msg_bytes,
    //                         funds: vec![Coin {
    //                             denom: denom.clone(),
    //                             amount: amount.clone(),
    //                         }],
    //                     }).into();

    //                     match (
    //                         if source_chain == "osmosis" {
    //                             self.osmosis
    //                                 .as_ref()
    //                                 .unwrap()
    //                                 .broadcast_tx_commit(signer.clone(), [msg]).await
    //                         } else {
    //                             self.union
    //                                 .as_ref()
    //                                 .unwrap()
    //                                 .broadcast_tx_commit(signer.clone(), [msg]).await
    //                         }
    //                     ) {
    //                         Ok(tx_hash) => {
    //                             tracing::info!(
    //                                 "Transaction sent successfully. Hash: {:?}",
    //                                 tx_hash
    //                             );
    //                         }
    //                         Err(e) => {
    //                             tracing::error!("Failed to submit tx!{:?}", e.to_string());
    //                         }
    //                     }
    //                 }).await;
    //             }
    //         }
    //         _ => {
    //             tracing::error!("Invalid transfer direction.");
    //         }
    //     }
    // }

    pub async fn check_packet_sequences(
        &self,
        source_chain_name: &str,
        target_chain_name: &str,
        expect_full_circle: u64,
    ) {
        let source_chain_id: i32 = ChainId::from_str(source_chain_name).unwrap() as i32;
        let target_chain_id = ChainId::from_str(target_chain_name).unwrap() as i32;
        // let datadog_data = self.datadog_data.clone();
        let mut interval = interval(Duration::from_secs(expect_full_circle));
        loop {
            interval.tick().await;

            let statuses = get_packet_statuses(&self.pool, source_chain_id, target_chain_id)
                .await
                .unwrap();
            let mut packet_statuses = self.packet_statuses.lock().await;

            for status in statuses {
                tracing::info!(
                    "Checking packet: {}. {} -> {}",
                    status.sequence_number,
                    source_chain_id,
                    target_chain_id
                );

                let time_passed = Utc::now()
                    .signed_duration_since(status.last_update)
                    .num_seconds();

                if time_passed < (expect_full_circle as i64) {
                    continue;
                }

                let mut can_be_removed = false;
                let mut issue = String::new();
                match status.recv_packet {
                    None => {
                        issue = "RecvPacket is missing".to_string();
                    }
                    Some(serde_json::Value::Null) => {
                        issue = "RecvPacket is null".to_string();
                    }
                    _ => {}
                }

                // If issue is empty string here, then the RecvPacket is present.
                // We can check the WriteAcknowledgement and AcknowledgePacket fields.

                if issue.is_empty() {
                    match status.write_ack {
                        None => {
                            issue = "WriteAcknowledgement is missing".to_string();
                        }
                        Some(serde_json::Value::Null) => {
                            issue = "WriteAcknowledgement is null".to_string();
                        }
                        _ => {
                            if let Ok(IbcEvent::WriteAcknowledgement(ref ack_event)) =
                                from_value::<IbcEvent>(status.write_ack.clone().unwrap())
                            {
                                let encoded_ack_hex = hex_encode(&ack_event.packet_ack_hex);

                                if !self.write_handler_packet_ack_hex_controller(
                                    ack_event.packet_ack_hex.clone(),
                                    &status.protocol,
                                ) {
                                    tracing::warn!(
                                        "WriteAcknowledgement indicates failure ({}).",
                                        encoded_ack_hex
                                    );
                                    issue = format!(
                                        "WriteAcknowledgement indicates failure ({}).",
                                        encoded_ack_hex
                                    );
                                }
                            }
                        }
                    };
                }

                // If issue is still empty string here, then the WriteAcknowledgement is present and valid.
                // We can check the WriteAcknowledgement and AcknowledgePacket fields.

                // if issue.is_empty() {
                //     match status.acknowledge_packet {
                //         None => {
                //             issue = "AcknowledgePacket is missing".to_string();
                //         }
                //         Some(serde_json::Value::Null) => {
                //             issue = "AcknowledgePacket is null".to_string();
                //         }
                //         _ => {
                //             tracing::info!(
                //                 "Acknowledgementta geliyo, ee  o zaman silebiliriz?: {:?}",
                //                 status.sequence_number
                //             );
                //             can_be_removed = true;
                //         }
                //     };
                // }

                if issue.is_empty() {
                    match status.acknowledge_packet {
                        None => {
                            issue = "AcknowledgePacket is missing".to_string();
                        }
                        Some(serde_json::Value::Null) => {
                            issue = "AcknowledgePacket is null".to_string();
                        }
                        _ => {
                            can_be_removed = true;
                        }
                    }
                }
                if !issue.is_empty() {
                    tracing::error!(
                        "There is a problem with sequence {}: {}. After: {} seconds.",
                        status.sequence_number,
                        issue,
                        time_passed
                    );
                    can_be_removed = true;
                }

                if can_be_removed {
                    self.remove_packet_status(status.sequence_number as u64, &status)
                        .await;
                    tracing::info!(
                        "Packet status with sequence number {} removed. {} => {}",
                        status.sequence_number,
                        status.source_chain_id,
                        status.target_chain_id
                    );
                }
            }
        }
    }

    fn write_handler_packet_ack_hex_controller(&self, ack_hex: Vec<u8>, protocol: &str) -> bool {
        match protocol {
            "Ucs01" => {
                return (Ucs01Ack::try_from(cosmwasm_std::Binary::from(ack_hex)).unwrap()
                    == Ucs01Ack::Success);
            }
            "Ics20" => {
                let val = Ics20Ack::try_from(cosmwasm_std::Binary::from(ack_hex)).unwrap();
                match val {
                    Ics20Ack::Result(_) => {
                        return true;
                    }
                    Ics20Ack::Error(_) => {
                        return false;
                    }
                }
            }
            _ => {
                tracing::error!("Unknown protocol {:?} -> {:?}", protocol, ack_hex);
                return false;
            }
        }
    }

    // TODO: Are there any other similar function to this? It's not good.
    async fn ibchandler_events_to_ibc_event(&self, log: RawLog) -> Option<IbcEvent> {
        match IBCHandlerEvents::decode_log(&log) {
            Ok(event) => {
                // Handle the decoded event similarly to Tendermint events
                let ibc_event: Option<IbcEvent> = match event {
                    IBCHandlerEvents::PacketEvent(packet_event) => match packet_event {
                        IBCPacketEvents::SendPacketFilter(event) => {
                            Some(IbcEvent::SendPacket(SendPacket {
                                packet_sequence: event.sequence.try_into().unwrap(),
                                packet_src_port: event.source_port.parse().unwrap(),
                                packet_src_channel: event.source_channel.parse().unwrap(),
                                packet_dst_port: "RANDOM_VALUE".to_string().parse().unwrap(),
                                packet_dst_channel: "RANDOM_VALUE".to_string().parse().unwrap(),
                                packet_timeout_height: event.timeout_height.into(),
                                packet_timeout_timestamp: event.timeout_timestamp,
                                packet_data_hex: hex_encode(event.data).into(),
                                packet_channel_ordering:
                                    unionlabs::ibc::core::channel::order::Order::NoneUnspecified,
                                connection_id: "connection-0".to_string().validate().unwrap(),
                            }))
                        }
                        IBCPacketEvents::RecvPacketFilter(event) => {
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
                                packet_channel_ordering:
                                    unionlabs::ibc::core::channel::order::Order::NoneUnspecified,
                                connection_id: "connection-0".to_string().validate().unwrap(),
                            }))
                        }
                        IBCPacketEvents::AcknowledgePacketFilter(event) => {
                            Some(IbcEvent::AcknowledgePacket(AcknowledgePacket {
                                packet_sequence: event.packet.sequence.try_into().unwrap(),
                                packet_src_port: "RANDOM_VALUE".to_string().parse().unwrap(),
                                packet_src_channel: "RANDOM_VALUE".to_string().parse().unwrap(),
                                packet_dst_port: event.packet.destination_port.parse().unwrap(),
                                packet_dst_channel: event
                                    .packet
                                    .destination_channel
                                    .parse()
                                    .unwrap(),
                                packet_timeout_height: event.packet.timeout_height.into(),
                                packet_timeout_timestamp: event.packet.timeout_timestamp,
                                packet_channel_ordering:
                                    unionlabs::ibc::core::channel::order::Order::NoneUnspecified,
                                connection_id: "connection-0".to_string().validate().unwrap(),
                            }))
                        }
                        IBCPacketEvents::WriteAcknowledgementFilter(event) => {
                            Some(IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
                                packet_sequence: event.packet.sequence.try_into().unwrap(),
                                packet_src_port: "RANDOM_VALUE".to_string().parse().unwrap(),
                                packet_src_channel: "RANDOM_VALUE".to_string().parse().unwrap(),
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
                                packet_ack_hex: hex_encode(event.acknowledgement).into(),
                                packet_data_hex: hex_encode("RANDOM_VALUE").into(),
                                packet_timeout_timestamp: 0,
                                connection_id: "connection-0".to_string().validate().unwrap(),
                            }))
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
                };

                return ibc_event;
            }
            Err(e) => {
                tracing::warn!("Could not decode Ethereum log event: {}", e);
            }
        }
        return None;
    }
}

impl<S: IbcTransferMain, D: IbcTransferMain> IbcTransfer<S, D> for Context {
    async fn send_ibc_transfer(&self, direction: TransferDirection<S, D>) {
        direction
            .source_chain
            .send_ibc_transfer(
                &self.transfer_test_config.amount,
                &direction.destination_chain.receiver(),
            )
            .await;
    }
}
