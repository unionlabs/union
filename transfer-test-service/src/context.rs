use core::future::Future;
use std::{
    collections::HashMap,
    fs::{ File, OpenOptions },
    pin::Pin,
    sync::Arc,
    time::{ SystemTime, UNIX_EPOCH },
};

use bech32::FromBase32;
use chain_utils::{ cosmos_sdk::CosmosSdkChainExt, ethereum::Ethereum };
// use tendermint::abci::Event as TendermintEvent;
use ethers::providers::{ Middleware, ProviderError };
use futures::StreamExt;
use hex::encode as hex_encode;
use queue_msg::Queue;
use tendermint_rpc::{
    event::{ Event, EventData, TxInfo },
    Client,
    SubscriptionClient,
    WebSocketClient,
};
use tokio::{ sync::Mutex, time::{ interval, Duration } };
use ucs01_relay::msg::{ ExecuteMsg, TransferMsg };
use unionlabs::{
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    events::IbcEvent,
    google::protobuf::any::Any,
    id::ClientId,
    tendermint::abci::{ event::Event as TendermintEvent, event_attribute::EventAttribute },
    traits::Chain,
};

use crate::{ config::{ Config, DatadogData }, datadog::{ log_builder, send_log_to_datadog } }; //, events::{ EventType } };

// Define a struct to store events for a packet sequence
#[derive(Debug, Clone)]
struct PacketStatus {
    send_packet: Option<IbcEvent<ClientId, String, String>>,
    recv_packet: Option<IbcEvent<ClientId, String, String>>,
    write_ack: Option<IbcEvent<ClientId, String, String>>,
    acknowledge_packet: Option<IbcEvent<ClientId, String, String>>,
    last_update: SystemTime,
}
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

impl Context {
    pub async fn new(transfer_test_config: Config, output: String) -> Context {
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

        let chain_id = osmosis.chain_id();

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
        }
    }

    pub async fn listen_tendermint(&self, tm_client: WebSocketClient) {
        let mut subs = tm_client
            .subscribe(tendermint_rpc::query::EventType::Tx.into()).await
            .unwrap();
        loop {
            tokio::select! {
                Some(event_result) = subs.next() => {
                    match event_result {
                        Ok(event) => {
                            self.handle_tendermint_tx_event(event).await;
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

    pub async fn listen(&self, source_chain: &str, target_chain: &str) {
        tokio::select! {
            _ = self.listen_tendermint(self.osmosis.tm_client.clone()) => {
                println!("Listening for events on Osmosis.");
            },
            _ = self.listen_tendermint(self.union.tm_client.clone()) => {
                println!("Listening for events on Union.");
            },
        }
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
                println!("Sending IBC transfer from {} to {}.", source_chain, target_chain);
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

                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

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

    async fn handle_tendermint_tx_event(&self, event: Event) {
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
                        let status = packet_statuses.entry(sequence.get()).or_insert(PacketStatus {
                            send_packet: None,
                            recv_packet: None,
                            write_ack: None,
                            acknowledge_packet: None,
                            last_update: SystemTime::now(),
                        });
                        // status.last_update = SystemTime::now();
                        match unwrapped {
                            IbcEvent::SendPacket(ref e) => {
                                status.send_packet = Some(IbcEvent::SendPacket(e.clone()));
                                status.last_update = SystemTime::now();
                            }
                            IbcEvent::RecvPacket(ref e) => {
                                status.recv_packet = Some(IbcEvent::RecvPacket(e.clone()));
                            }
                            IbcEvent::WriteAcknowledgement(ref e) => {
                                status.write_ack = Some(IbcEvent::WriteAcknowledgement(e.clone()));
                            }
                            IbcEvent::AcknowledgePacket(ref e) => {
                                status.acknowledge_packet = Some(
                                    IbcEvent::AcknowledgePacket(e.clone())
                                );
                            }
                            _ => {}
                        }
                    }

                    match unwrapped {
                        IbcEvent::SendPacket(e) => {
                            println!("SendPacket event: {:?}\n", e);
                        }
                        IbcEvent::RecvPacket(e) => {
                            let packet_sequence = e.packet_sequence;

                            println!("RecvPacket event: {:?}\n", e);
                            // Just an example datadog usage. This can be used to log any event.
                        }
                        IbcEvent::AcknowledgePacket(e) => {
                            println!("AcknowledgePacket event: {:?}\n", e);
                        }
                        IbcEvent::WriteAcknowledgement(e) => {
                            println!("WriteAcknowledgement event: {:?}\n", e);
                        }
                        _ => {
                            // println!("Untracked event: {:?}", unwrapped);
                        }
                    }
                }
            }
            _ => {
                println!("Unhandled event type: {:?}", event);
            }
        }
    }

    pub async fn start_packet_monitoring(&self) {
        let packet_statuses = self.packet_statuses.clone();
        let datadog_data = self.datadog_data.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10)); // 10 minutes
            loop {
                interval.tick().await;
                let mut statuses_to_log = Vec::new();
                let mut sequences_to_remove = Vec::new();

                let mut packet_statuses_locked = packet_statuses.lock().await;
                let now = SystemTime::now();
                for (sequence, status) in packet_statuses_locked.iter() {
                    println!("Looking at sequence: {}", sequence);
                    if status.send_packet.is_none() {
                        println!("SendPacket is missing");
                        sequences_to_remove.push(*sequence);
                        continue; // Skip if SendPacket has not been received
                    }

                    let elapsed = now.duration_since(status.last_update).unwrap();
                    let recv_packet_missing = status.recv_packet.is_none();
                    let write_ack_missing = status.write_ack.is_none();
                    let acknowledge_packet_missing = status.acknowledge_packet.is_none();
                    let ack_failed = status.write_ack.as_ref().map_or(false, |event| {
                        if let IbcEvent::WriteAcknowledgement(ref ack_event) = event {
                            hex_encode(&ack_event.packet_ack_hex) == "00"
                        } else {
                            false
                        }
                    });

                    if
                        recv_packet_missing ||
                        write_ack_missing ||
                        acknowledge_packet_missing ||
                        ack_failed
                    {
                        if elapsed > Duration::from_secs(10) {
                            statuses_to_log.push((*sequence, status.clone()));
                        } else if ack_failed {
                            println!("Ack failed.");
                            statuses_to_log.push((*sequence, status.clone()));
                        } else {
                            println!("time elapsed: {:?}", elapsed.as_secs());
                            continue;
                        }
                    } else {
                        println!("Removing from list: {}", sequence);
                        sequences_to_remove.push(*sequence);
                    }
                }

                for (sequence, status) in statuses_to_log {
                    let issue = if status.recv_packet.is_none() {
                        "RecvPacket is missing"
                    } else if status.write_ack.is_none() {
                        "WriteAcknowledgement is missing"
                    } else if
                        let Some(IbcEvent::WriteAcknowledgement(ref ack_event)) = status.write_ack
                    {
                        if hex_encode(&ack_event.packet_ack_hex) == "00" {
                            "WriteAcknowledgement indicates failure (0x00)"
                        } else if status.acknowledge_packet.is_none() {
                            "AcknowledgePacket is missing"
                        } else {
                            "Unknown issue"
                        }
                    } else {
                        "Unknown issue"
                    };

                    println!("There is a problem with sequence {}: {}", sequence, issue);
                    let log_info = log_builder(
                        format!(
                            "Incomplete packet sequence {}: {}. Packet: {:?}",
                            sequence,
                            issue,
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
                }
                if !sequences_to_remove.is_empty() {
                    // let mut packet_statuses_locked = packet_statuses.lock().await;
                    for sequence in sequences_to_remove {
                        println!("It is time to remove that sequence from list.: {}", sequence);
                        packet_statuses_locked.remove(&sequence);
                    }
                }
            }
        });
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
                    // Implement the logic for Cosmos to Cosmos transfer here
                    println!("Cosmos to Cosmos transfer not implemented yet.");
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
