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

pub struct Context {
    pub sent_queue: Arc<Mutex<BTreeMap<(String, String, u64), TimedEvent<SendPacket>>>>,
    pub output_file: String,
}

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    do_main(args).await
}

impl Context {
    async fn listen_union(&self, union: chain_utils::union::Union) {
        let mut events = Box::pin(union.events(()));
        loop {
            let event = events.next().await.unwrap().unwrap();
            println!("Event: {:?}", event);
            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    let transfer = Ucs01TransferPacket::try_from(cosmwasm_std::Binary(
                        e.packet_data_hex.clone(),
                    ))
                    .unwrap();
                    println!("{:?}", transfer);
                    self.sent_queue.lock().await.insert(
                        (
                            e.packet_src_port.clone(),
                            e.packet_src_channel.to_string(),
                            e.packet_sequence,
                        ),
                        TimedEvent::new(event.chain_id, e),
                    );
                }
                unionlabs::events::IbcEvent::RecvPacket(recv_event) => {
                    if let Some(send_event) = self.sent_queue.lock().await.remove(&(
                        recv_event.packet_src_port.clone(),
                        recv_event.packet_src_channel.to_string(),
                        recv_event.packet_sequence,
                    )) {
                        self.append_record(send_event, TimedEvent::new(event.chain_id, recv_event));
                    }
                }
                _ => (),
            }
        }
    }

    async fn listen_eth(&self, eth: chain_utils::evm::Evm<Minimal>) {
        let mut events = Box::pin(eth.events(()));
        loop {
            let event = events.next().await.unwrap().unwrap();
            println!("Event: {:?}", event);
            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    self.sent_queue.lock().await.insert(
                        (
                            e.packet_src_port.clone(),
                            e.packet_src_channel.to_string(),
                            e.packet_sequence,
                        ),
                        TimedEvent::new(event.chain_id.to_string(), e),
                    );
                }
                unionlabs::events::IbcEvent::RecvPacket(recv_event) => {
                    if let Some(send_event) = self.sent_queue.lock().await.remove(&(
                        recv_event.packet_src_port.clone(),
                        recv_event.packet_src_channel.to_string(),
                        recv_event.packet_sequence,
                    )) {
                        self.append_record(
                            send_event,
                            TimedEvent::new(event.chain_id.to_string(), recv_event),
                        );
                    }
                }
                _ => (),
            }
        }
    }

    fn append_record(
        &self,
        send_event: TimedEvent<SendPacket>,
        recv_event: TimedEvent<RecvPacket>,
    ) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(self.output_file.as_str())
            .unwrap();

        writeln!(
            file,
            "{},{},{},{}",
            send_event.chain_id, send_event.time, recv_event.chain_id, recv_event.time
        );
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
        sent_queue: Arc::new(Mutex::new(BTreeMap::new())),
        output_file: "output.csv".into(),
    };

    tokio::join!(
        union.broadcast_tx_commit([msg]),
        context.listen_union(union.clone()),
        context.listen_eth(eth.clone())
    );
}

/// Event types tracked by Zerg when exporting to CSV
enum EventType {
    /// Funds sent to Union
    SendToUnion,
    /// Funds received on Union
    ReceivedOnUnion,
    /// Funds sent to Ethereum
    SendToEthereum,
    /// Funds received on Ethereum
    ReceivedOnEthereum,
}
