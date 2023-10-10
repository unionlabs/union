use std::fs::read_to_string;

use chain_utils::EventSource;
use clap::Parser;
use cli::AppArgs;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use unionlabs::{
    cosmos::base::coin::Coin, cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    ethereum_consts_traits::Minimal, ibc::google::protobuf::any::Any, IntoProto,
};

pub mod cli;
pub mod config;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    do_main(args).await
}

async fn listen_union(union: chain_utils::union::Union) {
    let mut events = Box::pin(union.events(()));
    loop {
        let event = events.next().await;
        println!("Event: {:?}", event);
    }
}

async fn listen_eth(eth: chain_utils::evm::Evm<Minimal>) {
    let mut events = Box::pin(eth.events(()));
    loop {
        let event = events.next().await;
        println!("Event: {:?}", event);
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

    tokio::join!(
        union.broadcast_tx_commit([msg]),
        listen_union(union.clone()),
        listen_eth(eth.clone())
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
