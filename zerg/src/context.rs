use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    sync::Arc,
    time::Duration,
};

use chain_utils::EventSource;
use contracts::{
    erc20,
    ucs01_relay::{self as ucs01relay, LocalToken},
};
use cosmwasm_std::Uint128;
use ecdsa::SigningKey;
use ethers::{
    core::k256::ecdsa,
    prelude::SignerMiddleware,
    providers::Middleware,
    signers::{LocalWallet, Wallet},
    types::U256,
    utils::secret_key_to_address,
};
use futures::StreamExt;
use tokio::sync::Mutex;
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use ucs01_relay_api::types::Ucs01TransferPacket;
use unionlabs::{
    cosmos::base::coin::Coin, cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    ethereum_consts_traits::Minimal, ibc::google::protobuf::any::Any, traits::Chain, IntoProto,
};

use crate::{
    config::Config,
    events::{Event, EventType},
};

#[derive(Clone)]
pub struct Context {
    pub output_file: String,
    pub zerg_config: Config,
    pub is_rush: bool,
    pub writer: Arc<Mutex<File>>,
    pub union: chain_utils::union::Union,
    pub evm: chain_utils::evm::Evm<Minimal>,
    pub evm_accounts: HashMap<String, Wallet<SigningKey>>,
}

impl Context {
    pub async fn new(zerg_config: Config, output: String, is_rush: bool) -> Context {
        let writer = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(output)
            .unwrap();
        let union = chain_utils::union::Union::new(zerg_config.clone().union)
            .await
            .unwrap();
        let evm = chain_utils::evm::Evm::new(zerg_config.clone().evm)
            .await
            .unwrap();

        let mut evm_accounts = HashMap::new();

        let chain_id = evm.provider.get_chainid().await.unwrap().as_u64();

        zerg_config
            .clone()
            .evm
            .signers
            .into_iter()
            .for_each(|signer| {
                let signing_key: ecdsa::SigningKey = signer.value();
                let address = secret_key_to_address(&signing_key);
                let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id);
                evm_accounts.insert(format!("{:?}", address), wallet);
            });

        Context {
            output_file: "output.csv".to_string(),
            zerg_config,
            is_rush,
            writer: Arc::new(Mutex::new(writer)),
            union,
            evm,
            evm_accounts,
        }
    }

    pub async fn tx_handler(&self) {
        println!("Rush: Starting to rush Union txs...");

        let mut previous_height = 0;
        for _ in 0..self.zerg_config.rush_blocks {
            let mut height = previous_height;

            while height == previous_height {
                height = self.union.query_latest_height().await.revision_height;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            previous_height = height;

            for pk in self.zerg_config.evm.signers.iter() {
                let signing_key: ecdsa::SigningKey = pk.clone().value();
                let address = secret_key_to_address(&signing_key);
                let receiver = format!("{:?}", address);
                let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                    channel: self.zerg_config.channel.clone(),
                    receiver,
                    // TODO: use uuid in memo
                    memo: "garbage".to_string(),
                    timeout: None,
                });
                let transfer_msg = format!("{}", serde_json::to_string(&transfer_msg).unwrap());
                self.union
                    .signers
                    .with(|signer| async {
                        let msg = Any(MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: self.zerg_config.union_contract.clone(),
                            msg: transfer_msg.as_bytes().to_vec(),
                            funds: vec![Coin {
                                denom: self.zerg_config.union.fee_denom.clone(),
                                amount: "1".into(),
                            }],
                        })
                        .into_proto();

                        self.union.broadcast_tx_commit(signer, [msg]).await.unwrap()
                    })
                    .await
            }
        }
        println!("Rush: Done rushing Union txs!");
    }

    async fn send_from_eth(self, e: unionlabs::events::RecvPacket) {
        let transfer =
            Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

        let wallet = self.evm_accounts.get(transfer.receiver()).unwrap();

        let signer_middleware = Arc::new(SignerMiddleware::new(
            self.evm.provider.clone(),
            wallet.clone(),
        ));

        let ucs01_relay = ucs01relay::UCS01Relay::new(
            self.zerg_config.evm_contract.clone(),
            signer_middleware.clone(),
        );

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
        let mut events = Box::pin(self.union.clone().events(()));

        loop {
            println!("Union: Listening for IBC events...");
            let event = events.next().await.unwrap().unwrap();

            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    println!("Union: SendPacket observed!");
                    self.append_record(Event::create_send_event(event.chain_id, e))
                        .await
                }
                unionlabs::events::IbcEvent::RecvPacket(e) => {
                    println!("Union: RecvPacket observed!");
                    self.append_record(Event::create_recv_event(event.chain_id, e))
                        .await
                }
                _ => {
                    println!("Union: Untracked event observed.")
                }
            }
        }
    }

    pub async fn listen_eth(&self) {
        let mut events = Box::pin(self.evm.clone().events(()));

        loop {
            println!("Evm: Listening for IBC events...");
            let event = events.next().await.unwrap().unwrap();

            match event.event {
                unionlabs::events::IbcEvent::SendPacket(e) => {
                    println!("Evm: SendPacket observed!");
                    self.append_record(Event::create_send_event(event.chain_id.to_string(), e))
                        .await;
                }
                unionlabs::events::IbcEvent::RecvPacket(e) => {
                    println!("Evm: RecvPacket observed!");
                    self.append_record(Event::create_recv_event(
                        event.chain_id.to_string(),
                        e.clone(),
                    ))
                    .await;
                    if self.is_rush {
                        tokio::spawn(self.clone().send_from_eth(e.clone()));
                    }
                }
                _ => {
                    println!("Evm: Untracked event observed.")
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
