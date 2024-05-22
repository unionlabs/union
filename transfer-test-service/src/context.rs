use std::{
    collections::HashMap,
    fs::{ File, OpenOptions },
    io::Write,
    sync::Arc,
    time::{ SystemTime, UNIX_EPOCH },
};

use block_message::{ data::Data, AnyChainIdentified, BlockMessageTypes, Identified };
use hex::encode as hex_encode;
use chain_utils::{ cosmos_sdk::CosmosSdkChainExt, ethereum::Ethereum, Chains };

use futures::StreamExt;
use queue_msg::{ Engine, InMemoryQueue, Queue };
use tendermint_rpc::Client;
use tendermint_rpc::event::TxInfo;
use tokio::sync::Mutex;
use tokio::time::{ interval, Duration };
use tendermint_rpc::{ SubscriptionClient, WebSocketClient };
use tendermint_rpc::event::EventData;
use tendermint_rpc::event::Event;
use ucs01_relay::msg::{ ExecuteMsg, TransferMsg };

// use tendermint::abci::Event as TendermintEvent;
use ethers::providers::{ Middleware, Provider, ProviderError, Ws, WsClientError };
use unionlabs::{
    events::IbcEvent,
    traits::Chain,
    uint::U256,
    ethereum::config::Minimal,
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    google::protobuf::any::Any,
};
use unionlabs::tendermint::abci::event::Event as TendermintEvent;
use unionlabs::tendermint::abci::event_attribute::EventAttribute;
use unionlabs::id::ClientId;
use ethers::{ types::Filter, types::H160 };

use crate::{ config::Config, config::DatadogData }; //, events::{ EventType } };
use serde::{ Deserialize, Serialize };
use bech32::{ FromBase32 };

use crate::datadog::{ log_builder, send_log_to_datadog };

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

    pub async fn listen(&self) {
        let mut osmosis_subs = self.osmosis.tm_client
            .subscribe(tendermint_rpc::query::EventType::Tx.into()).await
            .unwrap();
        let mut union_subs = self.union.tm_client
            .subscribe(tendermint_rpc::query::EventType::Tx.into()).await
            .unwrap();

        loop {
            tokio::select! {
                Some(event_result) = osmosis_subs.next() => {
                    match event_result {
                        Ok(event) => {
                            // println!("Osmosis Event: {:?}", event);
                            self.handle_event(event).await;
                        }
                        Err(e) => {
                            tracing::error!("Error while receiving osmosis event: {:?}", e);
                        }
                    }
                },
                // Handle events from union
                Some(event_result) = union_subs.next() => {
                    match event_result {
                        Ok(event) => {
                            // println!("Union Event: {:?}", event);
                            self.handle_event(event).await;
                        }
                        Err(e) => {
                            tracing::error!("Error while receiving union event: {:?}", e);
                        }
                    }
                },


                else => break,
            }
        }
    }

    async fn handle_event(&self, event: Event) {
        match event.data {
            EventData::Tx { tx_result, .. } => {
                self.handle_tx_event(tx_result).await;
            }
            _ => {
                println!("Unhandled event type: {:?}", event);
            }
        }
    }

    pub async fn send_ibc_transfer_from_osmosis_to_union(&self) {
        println!("Sending IBC transfer from Osmosis to Union.");
        // Define the details of the transfer

        let channel = self.transfer_test_config.channel.to_string();
        let (_hrp, data, _variant) = bech32
            ::decode(&self.transfer_test_config.osmosis_contract)
            .expect("Invalid Bech32 address");
        let bytes = Vec::<u8>::from_base32(&data).expect("Invalid base32 data");
        let receiver = hex::encode(bytes);

        // panic!("{:?}", receiver);
        let amount = self.transfer_test_config.amount.to_string(); // Amount in uosmo
        let denom = self.transfer_test_config.osmosis.fee_denom.to_string(); // Denomination

        let uuid = uuid::Uuid::new_v4();

        // Create the transfer message
        let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
            channel,
            receiver,
            memo: uuid.to_string(),
            timeout: None,
        });

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Convert the message to JSON and then to bytes
        let transfer_msg_bytes = serde_json::to_string(&transfer_msg).unwrap().into_bytes();

        self.osmosis.signers.clone().with(|signer| async move {
            println!("Union: Sending Tx for {}.", signer.to_string());
            let msg = Any(MsgExecuteContract {
                sender: signer.to_string(),
                contract: self.transfer_test_config.osmosis_contract.clone(),
                msg: transfer_msg_bytes,
                funds: vec![Coin {
                    denom: denom,
                    amount: amount,
                }],
            }).into();

            match self.osmosis.broadcast_tx_commit(signer.clone(), [msg]).await {
                Ok(tx_hash) => {
                    println!("Union: Transaction sent successfully. Hash: {:?}", tx_hash);
                    // let mut attempts = 0;
                    // let max_attempts = 10;
                    // let delay_duration = Duration::from_secs(10);

                    // loop {
                    //     let tx_res_first: Result<
                    //         tendermint_rpc::endpoint::tx::Response,
                    //         tendermint_rpc::Error
                    //     > = self.union.tm_client.tx(
                    //         tx_hash.into_bytes().try_into().expect("Bytes are Hash"),
                    //         false
                    //     ).await;

                    //     match tx_res_first {
                    //         Ok(tx_res) => {
                    //             println!("Union: Transaction committed. Hash: {:?}", tx_res);
                    //             break;
                    //         }
                    //         Err(e) => {
                    //             if attempts >= max_attempts {
                    //                 println!(
                    //                     "Union: Failed to submit tx after {} attempts! {:?}",
                    //                     attempts,
                    //                     e
                    //                 );
                    //                 break;
                    //             } else {
                    //                 attempts += 1;
                    //                 println!("Union: Transaction not found yet, retrying... Attempt: {}", attempts);
                    //                 tokio::time::sleep(delay_duration).await;
                    //             }
                    //         }
                    //     }
                    // }
                }
                Err(e) => {
                    println!("Union: Failed to submit tx!{:?}", e);
                }
            }
        }).await;
    }

    async fn handle_tx_event(&self, tx_result: TxInfo) {
        for event in tx_result.result.events {
            let Some(my_event) = IbcEvent::<ClientId, String, String>::try_from_tendermint_event(
                TendermintEvent {
                    ty: event.kind,
                    attributes: event.attributes
                        .into_iter()
                        .map(|attr| EventAttribute {
                            key: attr.key,
                            value: attr.value,
                            index: attr.index,
                        })
                        .collect(),
                }
            ) else {
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
                        status.acknowledge_packet = Some(IbcEvent::AcknowledgePacket(e.clone()));
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
                        None
                    );
                    send_log_to_datadog(
                        &datadog_data.datadog_api_key,
                        &log_info,
                        &datadog_data.datadog_log_host
                    ).await.unwrap();
                }
                if !sequences_to_remove.is_empty() {
                    println!("What is happening here?: {:?}", sequences_to_remove);
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
