use std::fs::read_to_string;

use unionlabs::{cosmwasm::wasm::MsgExecuteContract, ibc::google::protobuf::any::Any, IntoProto};

pub mod config;

#[tokio::main]
async fn main() {
    println!("Hello, zerg!");
}

async fn do_main() {
    let zerg_config = read_to_string("zerg_conf.json").unwrap();
    let zerg_config: config::Config = serde_json::from_str(&zerg_config).unwrap();

    let union = chain_utils::union::Union::new(zerg_config.union).await;

    let transfer_msg = TransferMsg {
        channel: zerg_config.channel,
        receiver: &"garbage",
        // TODO: use uuid in memo
        memo: &"garbage",
    };
    let transfer_msg = serde_json::to_string(&transfer_msg).unwrap();
    let transfer_msg = format!("{ {} }", transfer_msg);

    let msg = MsgExecuteContract {
        sender: &"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
        contract: zerg_config.contract,
        msg: transfer_msg.as_bytes().to_vec(),
        funds: vec![],
    }
    .into_proto_bytes();

    union.broadcast_tx_commit([Any(msg)])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransferMsg {
    channel: String,
    receiver: String,
    memo: String,
}

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
