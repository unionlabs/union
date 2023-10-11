use std::{
    collections::{BTreeMap, HashMap},
    fmt::Binary,
    fs::{read_to_string, OpenOptions},
    io::Write,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use chain_utils::EventSource;
use clap::Parser;
use cli::AppArgs;
use ethers::utils::get_create2_address_from_hash;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use ucs01_relay_api::types::Ucs01TransferPacket;
use unionlabs::{
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    ethereum_consts_traits::Minimal,
    events::{RecvPacket, SendPacket},
    ibc::google::protobuf::any::Any,
    IntoProto,
};

pub mod cli;
pub mod config;

/// A timestamped event originating from `chain_id`.
pub struct TimedEvent<T> {
    pub time: u64,
    pub event: T,
    pub chain_id: String,
}

impl<T> TimedEvent<T> {
    pub fn new(chain_id: String, event: T) -> Self {
        Self {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            chain_id,
            event,
        }
    }
}

/// Event types tracked by Zerg when exporting to CSV
enum EventType {
    SendEvent(TimedEvent<SendPacket>),
    ReceiveEvent(TimedEvent<RecvPacket>),
}

/// Event information recorded to the output CSV.
pub struct Event {
    sender: String,
    stamped_event: EventType,
    uuid: String,
}

/// Creates an `Event` originating from `chain_id` from the `SendPacket` event data.
///
/// Constructs a unique ID from packet information in the form of:
/// `<src_port>-<src_channel>-<sequence>`
fn create_send_event(chain_id: String, e: SendPacket) -> Event {
    let transfer =
        Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

    let uuid = format!(
        "{}-{}-{}",
        e.packet_src_port.clone(),
        e.packet_src_channel,
        e.packet_sequence
    );

    Event {
        sender: transfer.sender().to_owned(),
        stamped_event: EventType::SendEvent(TimedEvent::new(chain_id, e)),
        uuid,
    }
}

/// Creates an `Event` originating from `chain_id` from the `RecvPacket` event data.
///
/// Constructs a unique ID from packet information in the form of:
/// `<src_port>-<src_channel>-<sequence>`
fn create_recv_event(chain_id: String, e: RecvPacket) -> Event {
    let transfer =
        Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

    let uuid = format!(
        "{}-{}-{}",
        e.packet_src_port.clone(),
        e.packet_src_channel,
        e.packet_sequence
    );

    Event {
        sender: transfer.sender().to_owned(),
        stamped_event: EventType::ReceiveEvent(TimedEvent::new(chain_id, e)),
        uuid,
    }
}

pub struct Context {
    pub output_file: String,
}

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    do_main(args).await
}

impl Context {
    async fn listen_union(&self, union: chain_utils::union::Union) -> ! {
        let mut events = Box::pin(union.events(()));

        loop {
            let event = events.next().await.unwrap().unwrap();

            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    self.append_record(create_send_event(event.chain_id, e))
                }
                unionlabs::events::IbcEvent::RecvPacket(e) => {
                    self.append_record(create_recv_event(event.chain_id, e))
                }
                _ => (),
            }
        }
    }

    async fn listen_eth(&self, eth: chain_utils::evm::Evm<Minimal>) {
        let mut events = Box::pin(eth.events(()));

        loop {
            let event = events.next().await.unwrap().unwrap();

            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    self.append_record(create_send_event(event.chain_id.to_string(), e))
                }
                unionlabs::events::IbcEvent::RecvPacket(e) => {
                    self.append_record(create_recv_event(event.chain_id.to_string(), e))
                }
                _ => (),
            }
        }
    }

    /// Appends a comma seperated line to the `output_file` provided by the context.
    ///
    /// Line Format:
    /// `<uuid>, <address>, <timestamp>, <EVENT_TYPE>, <chain_id>`
    /// Where `EVENT_TYPE` is either `"SentFrom"` or `"ReceivedOn"`.
    fn append_record(&self, event: Event) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(self.output_file.as_str())
            .unwrap();

        match event.stamped_event {
            EventType::SendEvent(e) => {
                writeln!(
                    file,
                    "{},{},{},SentFrom,{}",
                    event.uuid, event.sender, e.time, e.chain_id
                )
                .unwrap();
            }
            EventType::ReceiveEvent(e) => {
                writeln!(
                    file,
                    "{},{},{},ReceivedOn,{}",
                    event.uuid, event.sender, e.time, e.chain_id
                )
                .unwrap();
            }
        }
    }
}

async fn do_main(args: AppArgs) {
    let zerg_config: config::Config =
        serde_json::from_str(&read_to_string(args.config_file_path).unwrap()).unwrap();

    let union = chain_utils::union::Union::new(zerg_config.union).await;
    let eth = chain_utils::evm::Evm::new(zerg_config.evm).await;

    let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
        channel: zerg_config.channel,
        receiver: "0x1111111111111111111111111111111111111111".to_string(),
        // TODO: use uuid in memo
        memo: "garbage".to_string(),
        timeout: None,
    });
    let transfer_msg = format!("{}", serde_json::to_string(&transfer_msg).unwrap());

    let msg = Any(MsgExecuteContract {
        sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2".to_string(),
        contract: zerg_config.contract,
        msg: transfer_msg.as_bytes().to_vec(),
        funds: vec![Coin {
            denom: "stake".into(),
            amount: "10000".into(),
        }],
    })
    .into_proto();

    let context = Context {
        output_file: "output.csv".into(),
    };

    tokio::join!(
        union.broadcast_tx_commit([msg]),
        context.listen_union(union.clone()),
        context.listen_eth(eth.clone())
    );
}
