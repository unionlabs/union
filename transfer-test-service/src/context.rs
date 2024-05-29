use core::future::Future;
use std::{
    collections::HashMap,
    fs::{ File, OpenOptions },
    pin::Pin,
    sync::Arc,
    time::{ SystemTime },
};

use bech32::FromBase32;
use chain_utils::{ cosmos_sdk::CosmosSdkChainExt, ethereum::Ethereum };
use chrono::{ Utc };
// use tendermint::abci::Event as TendermintEvent;
// use ethers::providers::{ Middleware, ProviderError };
use futures::StreamExt;
use hex::encode as hex_encode;
use serde_json::{ from_value, to_value };
use tendermint_rpc::{ event::{ Event, EventData }, SubscriptionClient, WebSocketClient };
use tokio::{ sync::Mutex, time::{ interval, Duration } };
use ucs01_relay::msg::{ ExecuteMsg, TransferMsg };
use unionlabs::{
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    events::IbcEvent,
    google::protobuf::any::Any,
    id::ClientId,
    tendermint::abci::{ event::Event as TendermintEvent, event_attribute::EventAttribute },
    ClientType,
};

use crate::{
    config::{ ChainId, Config, DatadogData, PacketStatus },
    datadog::{ log_builder, send_log_to_datadog },
    sql_helper::{ delete_packet_status, get_packet_statuses, insert_or_update_packet_status },
}; //, events::{ EventType } };

#[derive(Clone)]
pub struct Context {
    pub output_file: String,
    pub transfer_test_config: Config,
    pub writer: Arc<Mutex<File>>,
    pub union: chain_utils::union::Union,
    pub osmosis: chain_utils::cosmos::Cosmos,
    // pub ethereum: chain_utils::ethereum::Ethereum<Minimal>,
    pub union_txs: Arc<Mutex<HashMap<u64, uuid::Uuid>>>,
    pub osmosis_txs: Arc<Mutex<HashMap<u64, uuid::Uuid>>>,
    pub datadog_data: DatadogData,
    pub packet_statuses: Arc<Mutex<HashMap<u64, PacketStatus>>>,
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

// Define the IbcTransfer trait
pub trait IbcTransfer {
    fn send_ibc_transfer(
        &self,
        direction: TransferDirection
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

#[derive(Debug, Clone)]
pub enum TransferDirection {
    CosmosToCosmos {
        source_chain: String,
        target_chain: String,
        channel: String,
        contract: String,
        receiver_bech32: String,
        denom: String,
        amount: String,
    },
    EthToCosmos {
        // Define necessary fields for Eth to Cosmos
    },
    CosmosToEth {
        // Define necessary fields for Cosmos to Eth
    },
}

pub trait TendermintClient {
    fn tm_client(&self) -> &WebSocketClient;
}

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

pub trait ChainListener: Sync + Send {
    fn listen<'a>(
        &'a self,
        context: &'a Context,
        source_chain: &'a str,
        target_chain: &'a str
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

impl<T> ChainListener for T where T: TendermintClient + Sync + Send + 'static {
    fn listen<'a>(
        &'a self,
        context: &'a Context,
        source_chain: &'a str,
        target_chain: &'a str
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            println!("Listening for events on {}.", source_chain);
            let mut subs = self
                .tm_client()
                .subscribe(tendermint_rpc::query::EventType::Tx.into()).await
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

impl Context {
    pub async fn new(
        transfer_test_config: Config,
        output: String,
        pool: sqlx::Pool<sqlx::Postgres>
    ) -> Context {
        let writer = OpenOptions::new().create(true).append(true).open(output.clone()).unwrap();
        tracing::debug!("Created writer.");
        let union = chain_utils::union::Union
            ::new(transfer_test_config.clone().union).await
            .unwrap();
        tracing::debug!("Created Union instance.");
        let osmosis = chain_utils::cosmos::Cosmos
            ::new(transfer_test_config.clone().osmosis).await
            .unwrap();
        tracing::debug!("Created Osmosis instance.");
        // let ethereum = chain_utils::ethereum::Ethereum
        //     ::new(transfer_test_config.clone().ethereum).await
        //     .unwrap();
        // tracing::debug!("Created Ethereum instance.");

        let datadog_data = transfer_test_config.datadog_data.clone();
        Context {
            output_file: output,
            transfer_test_config,
            writer: Arc::new(Mutex::new(writer)),
            union,
            osmosis,
            // ethereum,
            union_txs: Arc::new(Mutex::new(HashMap::new())),
            osmosis_txs: Arc::new(Mutex::new(HashMap::new())),
            datadog_data,
            packet_statuses: Arc::new(Mutex::new(HashMap::new())),
            pool,
        }
    }

    pub async fn listen_tendermint(
        &self,
        tm_client: WebSocketClient,
        source_chain: &str,
        target_chain: &str
    ) {
        let mut subs = tm_client
            .subscribe(tendermint_rpc::query::EventType::Tx.into()).await
            .unwrap();
        loop {
            tokio::select! {
                Some(event_result) = subs.next() => {
                    match event_result {
                        Ok(event) => {
                            self.handle_tendermint_tx_event(event, source_chain, target_chain).await;
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

    pub fn get_chain_listener(&self, chain_id: &ChainId) -> &dyn ChainListener {
        match chain_id {
            ChainId::Union => &self.union as &dyn ChainListener,
            ChainId::Osmosis => &self.osmosis as &dyn ChainListener,
            // Add other chain mappings as needed
            ChainId::Ethereum => unimplemented!("Ethereum listener is not implemented yet"),
        }
    }

    pub async fn listen(&self, source_chain: &str, target_chain: &str) {
        let source_chain_id = ChainId::from_str(source_chain).expect("Invalid source chain");
        let listener = self.get_chain_listener(&source_chain_id);
        listener.listen(self, source_chain, target_chain).await;
    }

    pub async fn send_ibc_transfer_cosmos_to_cosmos(&self, direction: &TransferDirection) {
        match direction {
            TransferDirection::CosmosToCosmos {
                source_chain,
                target_chain,
                channel,
                contract,
                receiver_bech32,
                denom,
                amount,
            } => {
                let time_now: SystemTime = SystemTime::now();
                println!(
                    "TIME: {:?}\tSending IBC transfer from {} to {}.",
                    time_now,
                    source_chain,
                    target_chain
                );

                let (_hrp, data, _variant) = bech32
                    ::decode(&receiver_bech32)
                    .expect("Invalid Bech32 address");

                let bytes = Vec::<u8>::from_base32(&data).expect("Invalid base32 data");
                let receiver = hex::encode(bytes);

                let uuid = uuid::Uuid::new_v4();

                // Create the transfer message
                let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                    channel: channel.to_string(),
                    receiver,
                    memo: uuid.to_string(),
                    timeout: None,
                });

                let transfer_msg_bytes = serde_json::to_string(&transfer_msg).unwrap().into_bytes();

                let signers = if source_chain == "osmosis" {
                    self.osmosis.signers.clone()
                } else {
                    self.union.signers.clone()
                };

                signers.with(|signer| async move {
                    println!("Sending Tx for {}.", signer.to_string());
                    let msg = Any(MsgExecuteContract {
                        sender: signer.to_string(),
                        contract: contract.clone(),
                        msg: transfer_msg_bytes,
                        funds: vec![Coin {
                            denom: denom.clone(),
                            amount: amount.clone(),
                        }],
                    }).into();

                    match (
                        if source_chain == "osmosis" {
                            self.osmosis.broadcast_tx_commit(signer.clone(), [msg]).await
                        } else {
                            self.union.broadcast_tx_commit(signer.clone(), [msg]).await
                        }
                    ) {
                        Ok(tx_hash) => {
                            println!("Transaction sent successfully. Hash: {:?}", tx_hash);
                        }
                        Err(e) => {
                            println!("Failed to submit tx!{:?}", e);
                        }
                    }
                }).await;
            }
            _ => {
                println!("Invalid transfer direction.");
            }
        }
    }

    async fn handle_tendermint_tx_event(
        &self,
        event: Event,
        source_chain: &str,
        target_chain: &str
    ) {
        match event.data {
            EventData::Tx { tx_result, .. } => {
                for event in tx_result.result.events {
                    let Some(my_event) = IbcEvent::<
                        ClientId,
                        String,
                        String
                    >::try_from_tendermint_event(TendermintEvent {
                        ty: event.kind,
                        attributes: event.attributes
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
                    let unwrapped = my_event.unwrap();

                    let packet_sequence = match unwrapped {
                        IbcEvent::SendPacket(ref e) => Some(e.packet_sequence),
                        IbcEvent::RecvPacket(ref e) => Some(e.packet_sequence),
                        IbcEvent::WriteAcknowledgement(ref e) => Some(e.packet_sequence),
                        IbcEvent::AcknowledgePacket(ref e) => Some(e.packet_sequence),
                        _ => None,
                    };
                    if let Some(sequence) = packet_sequence {
                        let mut packet_statuses = self.packet_statuses.lock().await;
                        let mut sequences_to_remove: Vec<u64> = Vec::new();

                        let status = packet_statuses
                            .entry(sequence.get())
                            .or_insert_with(|| {
                                PacketStatus::new(
                                    source_chain,
                                    target_chain,
                                    sequence.get().try_into().unwrap()
                                )
                            });

                        let mut should_insert_or_update = true;
                        let mut issue = String::new();

                        match unwrapped {
                            IbcEvent::SendPacket(ref e) => {
                                status.send_packet = Some(
                                    to_value(
                                        IbcEvent::<ClientId, ClientType, ClientId>::SendPacket(
                                            e.clone()
                                        )
                                    ).expect("Serialization failed")
                                );
                                status.last_update = chrono::Utc::now();
                                println!("SendPacket event. Sequence: {}", sequence);
                            }
                            IbcEvent::RecvPacket(ref e) => {
                                if status.send_packet.is_none() {
                                    issue = "RecvPacket received without SendPacket".to_string();
                                } else {
                                    status.recv_packet = Some(
                                        to_value(
                                            IbcEvent::<ClientId, ClientType, ClientId>::RecvPacket(
                                                e.clone()
                                            )
                                        ).expect("Serialization failed")
                                    );
                                    println!("RecvPacket event. Sequence: {}", sequence);
                                }
                            }
                            IbcEvent::WriteAcknowledgement(ref e) => {
                                if status.recv_packet.is_none() {
                                    issue =
                                        "WriteAcknowledgement received without RecvPacket".to_string();
                                } else {
                                    status.write_ack = Some(
                                        to_value(
                                            IbcEvent::<
                                                ClientId,
                                                ClientType,
                                                ClientId
                                            >::WriteAcknowledgement(e.clone())
                                        ).expect("Serialization failed")
                                    );
                                    println!("WriteAcknowledgement event. Sequence: {}", sequence);
                                }
                            }
                            IbcEvent::AcknowledgePacket(ref e) => {
                                if status.write_ack.is_none() {
                                    issue =
                                        "AcknowledgePacket received without WriteAcknowledgement".to_string();
                                } else {
                                    status.acknowledge_packet = Some(
                                        to_value(
                                            IbcEvent::<
                                                ClientId,
                                                ClientType,
                                                ClientId
                                            >::AcknowledgePacket(e.clone())
                                        ).expect("Serialization failed")
                                    );
                                    println!("AcknowledgePacket event. Sequence: {}", sequence);
                                    delete_packet_status(
                                        &self.pool,
                                        status.source_chain_id,
                                        status.target_chain_id,
                                        status.sequence_number
                                    ).await.unwrap();
                                    sequences_to_remove.push(sequence.get());
                                    should_insert_or_update = false;
                                }
                            }
                            _ => {
                                should_insert_or_update = false;
                            }
                        }
                        if !issue.is_empty() {
                            println!(
                                "Incomplete packet sequence {}: {}. Packet: {:?}",
                                sequence,
                                issue,
                                status
                            );
                            let log_info = log_builder(
                                format!(
                                    "Incomplete packet sequence {} from chain {} -> {}: {}. Packet: {:?}",
                                    status.sequence_number,
                                    ChainId::from_i32(&status.source_chain_id),
                                    ChainId::from_i32(&status.target_chain_id),
                                    issue,
                                    status
                                ),
                                None,
                                None,
                                None,
                                Some("error".to_string())
                            );
                            send_log_to_datadog(
                                &self.datadog_data.datadog_api_key,
                                &log_info,
                                &self.datadog_data.datadog_log_host
                            ).await.unwrap();
                            delete_packet_status(
                                &self.pool,
                                status.source_chain_id,
                                status.target_chain_id,
                                status.sequence_number
                            ).await.unwrap();
                            sequences_to_remove.push(sequence.get());

                            should_insert_or_update = false;
                        }

                        if should_insert_or_update {
                            insert_or_update_packet_status(
                                &self.pool,
                                status.clone()
                            ).await.unwrap();
                        }
                        // Remove collected sequences from the HashMap
                        for sequence in sequences_to_remove {
                            packet_statuses.remove(&sequence);
                        }
                    }
                }
            }
            _ => {
                println!("Unhandled event type: {:?}", event);
            }
        }
    }

    pub async fn check_packet_sequences(
        &self,
        source_chain_name: &str,
        target_chain_name: &str,
        expect_full_circle: u64
    ) {
        let source_chain_id: i32 = ChainId::from_str(source_chain_name).unwrap() as i32;
        let target_chain_id = ChainId::from_str(target_chain_name).unwrap() as i32;
        let datadog_data = self.datadog_data.clone();
        let mut interval = interval(Duration::from_secs(expect_full_circle));
        loop {
            interval.tick().await;

            let statuses = get_packet_statuses(
                &self.pool,
                source_chain_id,
                target_chain_id
            ).await.unwrap();
            let mut packet_statuses = self.packet_statuses.lock().await;

            for status in statuses {
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
                            if
                                let Ok(IbcEvent::WriteAcknowledgement(ref ack_event)) =
                                    from_value::<IbcEvent<ClientId, ClientType, ClientId>>(
                                        status.write_ack.clone().unwrap()
                                    )
                            {
                                let encoded_ack_hex = hex_encode(&ack_event.packet_ack_hex);
                                if encoded_ack_hex != "01" {
                                    issue =
                                        format!("WriteAcknowledgement indicates failure ({}).", encoded_ack_hex);
                                }
                            }
                        }
                    };
                }

                // If issue is still empty string here, then the WriteAcknowledgement is present and valid.
                // We can check the WriteAcknowledgement and AcknowledgePacket fields.

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
                    };
                }

                if issue != "" {
                    println!(
                        "There is a problem with sequence {}: {}. After: {} seconds.",
                        status.sequence_number,
                        issue,
                        time_passed
                    );
                    let log_info = log_builder(
                        format!(
                            "Incomplete packet sequence {} from chain {} -> {}: {}. After: {} seconds. Packet: {:?}",
                            status.sequence_number,
                            ChainId::from_i32(&status.source_chain_id),
                            ChainId::from_i32(&status.target_chain_id),
                            issue,
                            time_passed,
                            status
                        ),
                        None,
                        None,
                        None,
                        Some("error".to_string())
                    );
                    send_log_to_datadog(
                        &datadog_data.datadog_api_key,
                        &log_info,
                        &datadog_data.datadog_log_host
                    ).await.unwrap();
                    can_be_removed = true; // already sent that as an error.
                }

                if can_be_removed {
                    println!("Deleting packet: {}", status.sequence_number);
                    delete_packet_status(
                        &self.pool,
                        status.source_chain_id,
                        status.target_chain_id,
                        status.sequence_number
                    ).await.unwrap();
                    packet_statuses.remove(&(status.sequence_number as u64));
                }
            }
        }
    }
}

impl IbcTransfer for Context {
    fn send_ibc_transfer(
        &self,
        direction: TransferDirection
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            match direction {
                TransferDirection::CosmosToCosmos {
                    source_chain: _,
                    target_chain: _,
                    channel: _,
                    contract: _,
                    receiver_bech32: _,
                    denom: _,
                    amount: _,
                } => {
                    self.send_ibc_transfer_cosmos_to_cosmos(&direction).await;
                }
                TransferDirection::EthToCosmos {
                    // Define necessary fields for Eth to Cosmos
                } => {
                    // Implement the logic for Eth to Cosmos transfer here
                    println!("Eth to Cosmos transfer not implemented yet.");
                }
                TransferDirection::CosmosToEth {
                    // Define necessary fields for Cosmos to Eth
                } => {
                    // Implement the logic for Cosmos to Eth transfer here
                    println!("Cosmos to Eth transfer not implemented yet.");
                }
            }
        })
    }
}
