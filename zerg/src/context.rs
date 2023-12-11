use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chain_utils::{union::broadcast_tx_commit, EventSource};
use contracts::{
    erc20,
    ibc_handler::SendPacketFilter,
    ucs01_relay::{self as ucs01relay, LocalToken},
};
use cosmwasm_std::Uint128;
use ecdsa::SigningKey;
use ethers::{
    abi::Address,
    contract::EthLogDecode,
    core::k256::ecdsa,
    middleware::SignerMiddleware,
    providers::Middleware,
    signers::{LocalWallet, Signer, Wallet},
    types::U256,
    utils::secret_key_to_address,
};
use futures::StreamExt;
use tendermint_rpc::Client;
use tokio::sync::Mutex;
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use ucs01_relay_api::types::Ucs01TransferPacket;
use unionlabs::{
    cosmos::base::coin::Coin, cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    ethereum::config::Minimal, events::IbcEvent, google::protobuf::any::Any, traits::Chain,
    IntoProto,
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
    pub denom_address: Address,
    pub union_txs: Arc<Mutex<HashMap<u64, uuid::Uuid>>>,
    pub evm_txs: Arc<Mutex<HashMap<u64, uuid::Uuid>>>,
}

impl Context {
    pub async fn new(zerg_config: Config, output: String, is_rush: bool) -> Context {
        let writer = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(output.clone())
            .unwrap();
        tracing::debug!("Created writer.");
        let union = chain_utils::union::Union::new(zerg_config.clone().union)
            .await
            .unwrap();
        tracing::debug!("Created Union instance.");
        let evm = chain_utils::evm::Evm::new(zerg_config.clone().evm)
            .await
            .unwrap();
        tracing::debug!("Created Evm instance.");

        let mut evm_accounts = HashMap::new();

        let chain_id = evm.chain_id().0.as_u64();
        let ucs01_relay = ucs01relay::UCS01Relay::new(
            zerg_config.evm_contract.clone(),
            evm.provider.clone().into(),
        );
        tracing::debug!("Created usc01 relay.");
        let denom = format!(
            "wasm.{}/{}/{}",
            zerg_config.union_contract, zerg_config.channel, zerg_config.union.fee_denom
        );
        let denom_address = ucs01_relay.denom_to_address(denom).call().await.unwrap();
        tracing::debug!("Fetched denom address.");

        for signer in zerg_config.clone().evm.signers.into_iter() {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address = secret_key_to_address(&signing_key);
            let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id);
            evm_accounts.insert(format!("{:?}", address), wallet.clone());

            let signer_middleware =
                Arc::new(SignerMiddleware::new(evm.provider.clone(), wallet.clone()));

            let erc_contract = erc20::ERC20::new(denom_address, signer_middleware.clone());

            let ecr_contact_address = zerg_config.evm_contract.clone();

            tokio::spawn(async move {
                if let Ok(res) = erc_contract
                    .approve(ecr_contact_address.into(), U256::max_value())
                    .send()
                    .await
                {
                    res.await.unwrap().unwrap();
                    tracing::debug!("Approved balance.");
                };
            });
        }

        Context {
            output_file: output,
            zerg_config,
            is_rush,
            writer: Arc::new(Mutex::new(writer)),
            union,
            evm,
            evm_accounts,
            denom_address,
            union_txs: Arc::new(Mutex::new(HashMap::new())),
            evm_txs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn tx_handler(&self) {
        tracing::info!("Rush: Starting to rush Union txs...");

        let mut previous_height = 0;
        for _ in 0..self.zerg_config.rush_blocks {
            let mut height = previous_height;

            while height == previous_height {
                match self.union.query_latest_height().await {
                    Ok(maybe_height) => {
                        height = if maybe_height.revision_height == 0 {
                            height
                        } else {
                            maybe_height.revision_height
                        };
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Rush: Error getting height from Union.");
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            previous_height = height;

            for pk in self.zerg_config.clone().evm.signers.iter() {
                let signing_key: ecdsa::SigningKey = pk.clone().value();
                let address = secret_key_to_address(&signing_key);
                let receiver = format!("{:?}", address);
                let uuid = uuid::Uuid::new_v4();
                let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                    channel: self.zerg_config.channel.clone(),
                    receiver,
                    memo: uuid.to_string(),
                    timeout: None,
                });
                let transfer_msg = serde_json::to_string(&transfer_msg).unwrap().to_string();
                self.union
                    .signers.clone()
                    .with(|signer| async move {
                        tracing::info!("Union: Sending Tx for {}.", signer.to_string());
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

                        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                        match broadcast_tx_commit(&self.union, signer.clone(), [msg]).await {
                            Ok(tx_hash) => {
                                let tx_res = self
                                    .union
                                    .tm_client
                                    .tx(tx_hash.clone().into_bytes().try_into().expect("Bytes are Hash"), false)
                                    .await
                                    .unwrap();
                                let events: Result<Vec<_>, _> = tx_res
                                    .tx_result
                                    .events
                                    .into_iter()
                                    .map(|event| unionlabs::tendermint::abci::event::Event {
                                        ty: event.kind,
                                        attributes: event
                                            .attributes
                                            .into_iter()
                                            .map(|attr| {
                                                unionlabs::tendermint::abci::event_attribute::EventAttribute {
                                                    key: attr.key,
                                                    value: attr.value,
                                                    index: attr.index,
                                                }
                                            })
                                            .collect(),
                                    })
                                    .filter_map(IbcEvent::<String, String, String>::try_from_tendermint_event)
                                    .collect();
                                let event = events.unwrap().into_iter().find_map(|e| {
                                    match e {
                                        IbcEvent::SendPacket(e) => Some(e),
                                        _ => None
                                    }
                                }).expect("Tx totally exists, QED");
                                let mut union_txs = self.union_txs.lock().await;
                                tracing::info!("Union: Transaction sent with packet sequence: {}", event.packet_sequence);
                                union_txs.insert(event.packet_sequence, uuid);
                                self.append_record(Event::create_send_event(self.union.chain_id.clone(), uuid, signer.to_string(), Some(timestamp), None)).await;
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Union: Failed to submit tx!");
                            }
                        };
                    })
                    .await
            }
        }
        let finished_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        tracing::info!("Rush: Done rushing Union txs!");
        loop {
            tracing::info!("Rush: Union transaction rush finished at {}.", finished_at);
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    async fn send_from_eth(self, e: unionlabs::events::RecvPacket) {
        let transfer =
            Ucs01TransferPacket::try_from(cosmwasm_std::Binary(e.packet_data_hex.clone())).unwrap();

        let wallet =
            if let Some(wallet) = self.evm_accounts.get(&format!("{:?}", transfer.receiver())) {
                wallet
            } else {
                tracing::debug!("Evm: Recv Packet not from zerg.");
                return;
            };

        let signer_middleware = Arc::new(SignerMiddleware::new(
            self.evm.provider.clone(),
            wallet.clone(),
        ));

        let ucs01_relay = ucs01relay::UCS01Relay::new(
            self.zerg_config.evm_contract.clone(),
            signer_middleware.clone(),
        );

        let mut previous_height = 0;
        loop {
            let mut height = previous_height;

            while height == previous_height {
                height = self.evm.provider.get_block_number().await.unwrap().as_u64();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            previous_height = height;

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let uuid = uuid::Uuid::new_v4();

            if let Ok(pending) = ucs01_relay
                .send(
                    e.packet_dst_port.clone().to_string(),
                    e.packet_dst_channel.clone().to_string(),
                    transfer.sender().to_vec().into(),
                    vec![LocalToken {
                        denom: self.denom_address,
                        amount: Uint128::try_from(transfer.tokens()[0].amount)
                            .unwrap()
                            .u128(),
                    }],
                    3,
                    u64::MAX,
                )
                .send()
                .await
            {
                if let Ok(sent) = pending.await {
                    let tx = sent.unwrap();
                    let send: SendPacketFilter = tx
                        .logs
                        .into_iter()
                        .find_map(|log| SendPacketFilter::decode_log(&log.into()).ok())
                        .unwrap();

                    let mut evm_txs = self.evm_txs.lock().await;
                    evm_txs.insert(send.sequence, uuid);

                    self.append_record(Event::create_send_event(
                        self.evm.chain_id().to_string(),
                        uuid,
                        wallet.address().to_string(),
                        Some(timestamp),
                        None,
                    ))
                    .await;
                    tracing::info!(
                        "Eth: Transaction {}/{} was submitted!",
                        e.packet_sequence,
                        send.sequence
                    );
                    break;
                }
            } else {
                tracing::error!(
                    "Eth: Transaction {} failed, trying again next block...",
                    e.packet_sequence
                );
            }
        }
    }

    pub async fn listen_union(&self) {
        let mut events = Box::pin(self.union.clone().events(()));

        loop {
            tracing::info!("Union: Listening for IBC events...");

            match events.next().await {
                Some(Ok(event)) => match event.event {
                    IbcEvent::SendPacket(_) => {
                        tracing::info!("Union: SendPacket observed!");
                    }
                    IbcEvent::RecvPacket(e) => {
                        tracing::info!("Union: RecvPacket observed!");
                        let evm_txs = self.evm_txs.lock().await;
                        let uuid = match evm_txs.get(&e.packet_sequence) {
                            Some(uuid) => uuid.to_owned(),
                            None => {
                                tracing::warn!(
                                    "Union: no matching uuid for packet sequence: {}",
                                    e.packet_sequence
                                );
                                uuid::Uuid::new_v4()
                            }
                        };
                        self.append_record(Event::create_recv_event(
                            event.chain_id,
                            uuid,
                            e,
                            None,
                            None,
                        ))
                        .await;
                    }
                    _ => {
                        tracing::debug!("Union: Untracked event observed: {:?}", event.event);
                    }
                },
                Some(Err(e)) => {
                    tracing::error!("Union: Skipping events due to error: {:?}", e);
                }
                None => {
                    tracing::debug!("Union: No events...");
                }
            }
        }
    }

    pub async fn listen_eth(&self) {
        let mut events = Box::pin(self.evm.clone().events(()));

        loop {
            tracing::info!("Evm: Listening for IBC events...");

            match events.next().await {
                Some(Ok(event)) => {
                    let block = self
                        .evm
                        .provider
                        .get_block(ethers::types::H256(event.block_hash.into()))
                        .await
                        .unwrap()
                        .unwrap();
                    let timestamp = block.timestamp.as_u64();

                    match event.event {
                        IbcEvent::SendPacket(_e) => {
                            tracing::info!("Evm: SendPacket observed!");
                        }
                        IbcEvent::RecvPacket(e) => {
                            tracing::info!("Evm: RecvPacket observed!");
                            let union_txs = self.union_txs.lock().await;
                            let uuid = match union_txs.get(&e.packet_sequence) {
                                Some(uuid) => uuid.to_owned(),
                                None => {
                                    tracing::warn!(
                                        "Evm: no matching uuid for packet sequence: {}.",
                                        e.packet_sequence
                                    );
                                    uuid::Uuid::new_v4()
                                }
                            };
                            self.append_record(Event::create_recv_event(
                                event.chain_id.to_string(),
                                uuid,
                                e.clone(),
                                Some(timestamp),
                                None,
                            ))
                            .await;
                            if self.is_rush {
                                tokio::spawn(self.clone().send_from_eth(e));
                            }
                        }
                        _ => {
                            tracing::debug!("Evm: Untracked event observed: {:?}", event.event)
                        }
                    }
                }
                Some(Err(e)) => {
                    tracing::error!(error = %e, "Evm: Skipping events due to error.");
                }
                None => {
                    tracing::debug!("Evm: No events...");
                }
            }
        }
    }

    /// Appends a comma separated line to the `output_file` provided by the context.
    ///
    /// Line Format:
    /// `<uuid>,<address>,<execution_timestamp>,<finalization_timestamp>,<event_type>,<chain_id>`
    /// Where `event_type` is either `"SentFrom"` or `"ReceivedOn"`.
    pub async fn append_record(&self, event: Event) {
        let mut writer = self.writer.lock().await;
        match event.stamped_event {
            EventType::SendEvent(e) => {
                writeln!(
                    writer,
                    "{},{},{},{},SentFrom,{}",
                    event.uuid,
                    event.sender,
                    e.execution_timestamp,
                    e.finalization_timestamp,
                    e.chain_id
                )
                .unwrap();
            }
            EventType::ReceiveEvent(e) => {
                writeln!(
                    writer,
                    "{},{},{},{},ReceivedOn,{}",
                    event.uuid,
                    event.sender,
                    e.execution_timestamp,
                    e.finalization_timestamp,
                    e.chain_id
                )
                .unwrap();
            }
        }
    }
}
