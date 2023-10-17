use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::Arc,
    time::Duration,
};

use chain_utils::{Chain, EventSource};
use contracts::{
    erc20,
    ucs01_relay::{self as ucs01relay, LocalToken},
};
use cosmwasm_std::Uint128;
use ethers::{prelude::SignerMiddleware, types::U256};
use futures::StreamExt;
use tokio::sync::Mutex;
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use ucs01_relay_api::types::Ucs01TransferPacket;
use unionlabs::{
    cosmos::base::coin::Coin, cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    ethereum_consts_traits::Minimal, ibc::google::protobuf::any::Any, IntoProto,
};

use crate::{
    config::Config,
    events::{Event, EventType},
};

#[derive(Clone)]
pub struct Context {
    pub output_file: String,
    pub zerg_config: Config,
    pub evm: chain_utils::evm::Evm<Minimal>,
    pub is_rush: bool,
    pub writer: Arc<Mutex<File>>,
}

impl Context {
    pub async fn new(zerg_config: Config, output: String, is_rush: bool) -> Context {
        let evm = chain_utils::evm::Evm::<Minimal>::new(zerg_config.evm.clone())
            .await
            .unwrap();

        let writer = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(output)
            .unwrap();

        Context {
            output_file: "output.csv".to_string(),
            zerg_config,
            evm,
            is_rush,
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    pub async fn tx_handler(&self) {
        let signer_middleware =
            SignerMiddleware::new(self.evm.provider.clone(), self.evm.wallet.clone());
        let receiver = format!("{:?}", signer_middleware.address());

        println!("Starting to rush Union txs...");

        let mut previous_height = 0;
        for _ in 0..self.zerg_config.rush_blocks {
            let mut height = previous_height;

            while height == previous_height {
                height = self
                    .zerg_config
                    .union
                    .get_union_for(0)
                    .await
                    .query_latest_height()
                    .await
                    .revision_height;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            previous_height = height;

            let mut txs = vec![];
            let mut msgs = vec![];
            let mut unions = vec![];
            for (i, _account) in self.zerg_config.union.signers.iter().enumerate() {
                let union = self.zerg_config.union.get_union_for(i).await;

                let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                    channel: self.zerg_config.channel.clone(),
                    receiver: receiver.clone(),
                    // TODO: use uuid in memo
                    memo: "garbage".to_string(),
                    timeout: None,
                });

                let transfer_msg = format!("{}", serde_json::to_string(&transfer_msg).unwrap());

                let msg = Any(MsgExecuteContract {
                    sender: union.signer.to_string(),
                    contract: self.zerg_config.union_contract.clone(),
                    msg: transfer_msg.as_bytes().to_vec(),
                    funds: vec![Coin {
                        denom: self.zerg_config.union.fee_denom.clone(),
                        amount: "1".into(),
                    }],
                })
                .into_proto();

                unions.push(union);
                msgs.push(msg);
            }

            unions.into_iter().zip(msgs).for_each(|(union, msg)| {
                txs.push(tokio::spawn(async move {
                    union.broadcast_tx_commit([msg]).await;
                }))
            });

            let _ = futures::future::try_join_all(txs.into_iter()).await;
        }
        println!("Done rushing Union txs!");
    }

    async fn send_from_eth(&self, e: unionlabs::events::RecvPacket) {
        let signer_middleware = Arc::new(SignerMiddleware::new(
            self.evm.provider.clone(),
            self.evm.wallet.clone(),
        ));

        let ucs01_relay = ucs01relay::UCS01Relay::new(
            self.zerg_config.evm_contract.clone(),
            signer_middleware.clone(),
        );

        let transfer =
            Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();
        let denom = format!(
            "{}/{}/{}",
            e.packet_src_port, e.packet_src_channel, self.zerg_config.union.fee_denom
        );
        let denom_address = ucs01_relay.denom_to_address(denom).call().await.unwrap();
        let erc_contract = erc20::ERC20::new(denom_address, signer_middleware.clone());

        erc_contract
            .approve(
                self.zerg_config.evm_contract.clone().into(),
                U256::max_value(),
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        ucs01_relay
            .send(
                e.packet_dst_port.clone(),
                e.packet_dst_channel.clone().to_string(),
                transfer.sender().to_string(),
                vec![LocalToken {
                    denom: denom_address,
                    amount: Uint128::try_from(transfer.tokens()[0].amount)
                        .unwrap()
                        .u128(),
                }],
                3,
                u64::MAX,
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
    }

    pub async fn listen_union(&self) {
        let union = self.zerg_config.union.get_union_for(0).await;
        let mut events = Box::pin(union.events(()));

        loop {
            println!("Listening for Union IBC events...");
            let event = events.next().await.unwrap().unwrap();

            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    println!("SendPacket from Union!");
                    self.append_record(Event::create_send_event(event.chain_id, e))
                        .await
                }
                unionlabs::events::IbcEvent::RecvPacket(e) => {
                    println!("RecvPacket on Union!");
                    self.append_record(Event::create_recv_event(event.chain_id, e))
                        .await
                }
                _ => {
                    println!("Untracked Union IBC event")
                }
            }
        }
    }

    pub async fn listen_eth(&self) {
        let mut events = Box::pin(self.evm.events(()));

        loop {
            println!("Listening for Evm IBC events...");
            let event = events.next().await.unwrap().unwrap();

            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    println!("SendPacket from Evm!");
                    self.append_record(Event::create_send_event(event.chain_id.to_string(), e))
                        .await
                }
                unionlabs::events::IbcEvent::RecvPacket(e) => {
                    println!("RecvPacket on Evm!");
                    if self.is_rush {
                        self.send_from_eth(e.clone()).await;
                    }
                    self.append_record(Event::create_recv_event(event.chain_id.to_string(), e))
                        .await
                }
                _ => {
                    println!("Untracked Evm IBC event")
                }
            }
        }
    }

    /// Appends a comma separated line to the `output_file` provided by the context.
    ///
    /// Line Format:
    /// `<uuid>, <address>, <timestamp>, <EVENT_TYPE>, <chain_id>`
    /// Where `EVENT_TYPE` is either `"SentFrom"` or `"ReceivedOn"`.
    pub async fn append_record(&self, event: Event) {
        let mut writer = self.writer.lock().await;
        match event.stamped_event {
            EventType::SendEvent(e) => {
                writeln!(
                    writer,
                    "{},{},{},SentFrom,{}",
                    event.uuid, event.sender, e.time, e.chain_id
                )
                .unwrap();
            }
            EventType::ReceiveEvent(e) => {
                writeln!(
                    writer,
                    "{},{},{},ReceivedOn,{}",
                    event.uuid, event.sender, e.time, e.chain_id
                )
                .unwrap();
            }
        }
    }
}
