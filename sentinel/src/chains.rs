use std::{collections::HashMap, sync::Arc, time::Duration};

use bech32::FromBase32;
use chain_utils::{
    cosmos_sdk::CosmosSdkChainExt,
    ethereum::{EthereumExecutionRpcs, EthereumExecutionRpcsExt, IBCHandlerEvents},
};
use contracts::{
    erc20,
    ibc_packet::IBCPacketEvents,
    ucs01_relay::{LocalToken, UCS01Relay},
};
use ecdsa::SigningKey;
use ethers::{
    abi::RawLog,
    contract::EthLogDecode,
    core::k256::ecdsa,
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, Ws},
    signers::LocalWallet,
    types::{Address, BlockId, Filter},
    utils::secret_key_to_address,
};
use futures::StreamExt;
use hex::{decode as hex_decode, encode as hex_encode};
use prost::{Message, Name};
use protos::{google::protobuf::Any, ibc::applications::transfer::v1::MsgTransfer};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tendermint_rpc::{
    event::{Event, EventData},
    SubscriptionClient, WebSocketClient,
};
use ucs01_relay::{
    ibc,
    msg::{ExecuteMsg, TransferMsg},
};
use ucs01_relay_api::types::{Ics20Ack, JsonWasm, Ucs01Ack};
use unionlabs::{
    cosmos::base::coin::Coin,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    encoding::{self, Decode, DecodeAs},
    ethereum::config::{ChainSpec, Mainnet, Minimal},
    events::{AcknowledgePacket, RecvPacket, SendPacket, WriteAcknowledgement},
    google::protobuf::any,
    hash::H160,
    ibc::core::{channel::channel::Channel, client::height::Height},
    id::{ChannelId, ClientId},
    tendermint::abci::{event::Event as TendermintEvent, event_attribute::EventAttribute},
    uint::U256,
    validated::ValidateT,
};

use crate::{
    config::{CosmosConfig, EthereumConfig, TransferModule},
    context::SharedMap,
};
pub type IbcEvent = unionlabs::events::IbcEvent<ClientId, String, ClientId>;

pub trait IbcTransfer: Send + Sync {
    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
    );
}

pub trait IbcListen: Send + Sync {
    async fn listen(&self, shared_map: &SharedMap);

    // TODO(caglankaan): How can i know the protocol type here? On listen we don't know what is the destination chain
    // It can be anything, if i am listening on union since there is only one listener for union there could be 2 different
    // chains which are sending request to me 1- ethereum with ucs01 and 2- osmosis with ics20 so i am not sure how can i know
    // the protocol here. For know i'll try bruteforce but it's not a good solution.
    fn write_handler_packet_ack_hex_controller(
        &self,
        ack_hex: Vec<u8>, //protocol: Protocol
    ) -> bool {
        // match protocol {
        //     Protocol::Ics20 => {
        //         let val = Ics20Ack::try_from(cosmwasm_std::Binary::from(ack_hex)).unwrap();
        //         match val {
        //             Ics20Ack::Result(_) => {
        //                 return true;
        //             }
        //             Ics20Ack::Error(_) => {
        //                 return false;
        //             }
        //         }
        //     }
        //     Protocol::Ucs01 => {
        //         return (
        //             Ucs01Ack::try_from(cosmwasm_std::Binary::from(ack_hex)).unwrap() ==
        //             Ucs01Ack::Success
        //         );
        //     }
        //     _ => {
        //         tracing::error!("Unknown protocol {:?} -> {:?}", protocol, ack_hex);
        //         return false;
        //     }
        // }

        // Try to decode as Ics20Ack first;
        if let Ok(val) =
            Ics20Ack::decode_as::<JsonWasm>(cosmwasm_std::Binary::from(ack_hex.clone()).as_slice())
        {
            match val {
                Ics20Ack::Result(_) => {
                    tracing::info!("Ics20Ack::Result successfully decoded.");
                    return true;
                }
                Ics20Ack::Error(_) => {
                    tracing::warn!("Ics20Ack::Result failed decode.");
                }
            }
        }

        if let Ok(val) = Ucs01Ack::decode_as::<encoding::EthAbi>(
            cosmwasm_std::Binary::from(ack_hex.clone()).as_slice(),
        ) {
            tracing::info!(
                "Ucs01Ack:: successfully decoded: {}",
                val == Ucs01Ack::Success
            );
            return val == Ucs01Ack::Success;
        } else {
            tracing::error!("Failed to decode Ucs01Ack");
            return false;
        }

        // If both decoding attempts fail, return false
        tracing::error!("Failed to decode ack_hex as either Ics20Ack or Ucs01Ack.");
        false
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        shared_map: &SharedMap,
        block_number: u64,
    );

    fn handle_ibc_event_boxed<'a>(
        &'a self,
        ibc_event: IbcEvent,
        shared_map: &'a SharedMap,
        block_number: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let (packet_sequence, key) = match &ibc_event {
                IbcEvent::SendPacket(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                IbcEvent::RecvPacket(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                IbcEvent::WriteAcknowledgement(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                IbcEvent::AcknowledgePacket(e) => (
                    e.packet_sequence,
                    format!("{}->{}", e.packet_src_channel, e.packet_dst_channel),
                ),
                // Handle other events if necessary,
                _ => {
                    return;
                }
            };
            tracing::info!("packet_sequence: {:?}, key: {:?}", packet_sequence, key);

            let sequence = packet_sequence.get() as i32;
            {
                let mut map = shared_map.lock().await;

                let entry = map.entry(key.clone()).or_insert_with(HashMap::new);

                let sequence_entry = entry.entry(sequence).or_insert_with(|| {
                    let mut event_map = HashMap::new();
                    event_map.insert(0, false);
                    event_map.insert(1, false);
                    event_map.insert(2, false);
                    event_map.insert(3, false);
                    event_map
                });
                match ibc_event {
                    IbcEvent::SendPacket(_) => {
                        sequence_entry.insert(0, true);
                        tracing::info!("SendPacket event recorded for sequence {}", sequence);
                    }
                    IbcEvent::RecvPacket(_) => {
                        if !sequence_entry.get(&0).unwrap_or(&false) {
                            tracing::warn!(
                                "RecvPacket event received without SendPacket for sequence {}",
                                sequence
                            );
                        } else {
                            sequence_entry.insert(1, true);
                            tracing::info!("RecvPacket event recorded for sequence {}", sequence);
                        }
                    }
                    IbcEvent::WriteAcknowledgement(ref e) => {
                        if self.write_handler_packet_ack_hex_controller(e.packet_ack_hex.clone()) {
                            sequence_entry.insert(2, true);
                            tracing::info!(
                                "WriteAcknowledgement event recorded for sequence {}",
                                sequence
                            );
                        } else {
                            tracing::error!(
                                "WriteAcknowledgement indicates failure. Sequence: {}, packet_hack_hex: {:?}",
                                sequence,
                                e.packet_ack_hex.clone()
                            );
                            // Here remove it from the map
                            entry.remove(&sequence);
                        }
                    }
                    IbcEvent::AcknowledgePacket(_) => {
                        if !sequence_entry.get(&0).unwrap_or(&false)
                            || !sequence_entry.get(&1).unwrap_or(&false)
                            || !sequence_entry.get(&2).unwrap_or(&false)
                        {
                            tracing::warn!(
                                "AcknowledgePacket event received out of order for sequence {}",
                                sequence
                            );
                        } else {
                            sequence_entry.insert(3, true);
                            tracing::info!(
                                "AcknowledgePacket event recorded for sequence {}",
                                sequence
                            );

                            if sequence_entry.values().all(|&v| v) {
                                tracing::info!(
                                    "All events completed for sequence {}: {:?}",
                                    sequence,
                                    sequence_entry
                                );
                                entry.remove(&sequence);
                            }
                        }
                    }
                    _ => {
                        return;
                    }
                }
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Protocol {
    Ics20 {
        receivers: Vec<String>, // Changed to Vec<String>
        module: TransferModule,
    },
    Ucs01 {
        receivers: Vec<String>, //Vec<Vec<u8>>,
        contract: String,
    },
}

#[derive(Debug, Clone)]
pub enum Chain {
    Ethereum(Ethereum),
    Osmosis(Cosmos),
    Union(Cosmos),
}

impl IbcListen for Chain {
    async fn listen(&self, shared_map: &SharedMap) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum.listen(shared_map).await;
            }
            Chain::Union(union) => {
                union.listen(shared_map).await;
            }
            Chain::Osmosis(osmosis) => {
                osmosis.listen(shared_map).await;
            }
        }
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        shared_map: &SharedMap,
        block_number: u64,
    ) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum
                    .handle_ibc_event(ibc_event, shared_map, block_number)
                    .await;
            }
            Chain::Union(union) => {
                union
                    .handle_ibc_event(ibc_event, shared_map, block_number)
                    .await;
            }
            Chain::Osmosis(osmosis) => {
                osmosis
                    .handle_ibc_event(ibc_event, shared_map, block_number)
                    .await;
            }
        }
    }

    fn write_handler_packet_ack_hex_controller(
        &self,
        ack_hex: Vec<u8>, // protocol: Protocol // TODO: Add it after find a way
    ) -> bool {
        IbcListen::write_handler_packet_ack_hex_controller(self, ack_hex /* , protocol*/)
    }
}

impl IbcTransfer for Chain {
    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
        match self {
            Chain::Ethereum(ethereum) => {
                ethereum
                    .send_ibc_transfer(protocol, channel, destination_channel, denom, amount)
                    .await;
            }
            Chain::Osmosis(osmosis) => {
                osmosis
                    .send_ibc_transfer(protocol, channel, destination_channel, denom, amount)
                    .await;
            }
            Chain::Union(union) => {
                union
                    .send_ibc_transfer(protocol, channel, destination_channel, denom, amount)
                    .await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ethereum {
    pub rpc: EthereumRpc,
    pub relays:
        Vec<UCS01Relay<SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>>>,
    pub signer_middlewares:
        Vec<Arc<SignerMiddleware<NonceManagerMiddleware<Arc<Provider<Ws>>>, LocalWallet>>>,
    pub ucs01_contract: String,
    pub msg_senders: Vec<ethers::types::H160>,
    pub relay_addr: ethers::types::H160,
}

#[derive(Debug, Clone)]
pub struct EthereumRpc {
    pub provider: Arc<Provider<Ws>>,
    pub ibc_handler_address: H160,
}

impl EthereumExecutionRpcs for EthereumRpc {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl IbcListen for Ethereum {
    async fn listen(&self, shared_map: &SharedMap) {
        let mut latest_checked_block = 0;
        loop {
            let provider = self.rpc.provider.clone();

            let mut latest_block = provider.get_block_number().await.unwrap().as_u64();
            if latest_checked_block == latest_block {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            latest_checked_block = latest_block;
            tracing::info!("latest_block {:?}.", latest_block);
            // Update the filter to fetch logs from the latest block processed + 1
            let filter = Filter::new()
                .address(ethers::types::H160::from(self.rpc.ibc_handler_address))
                .from_block(latest_block)
                .to_block(latest_block); //TODO(caglankaan): How can we make here subscribe like instead of latest_block

            let logs = provider.get_logs(&filter).await.unwrap();

            let logs_clone = logs.clone(); // Clone logs for processing
            futures::stream::iter(logs_clone)
                .filter_map(|log| async move {
                    let raw_log = RawLog {
                        topics: log.topics.clone(),
                        data: log.data.clone().to_vec(),
                    };

                    Some(raw_log)
                })
                .for_each_concurrent(None, |raw_log| async move {
                    let decoded_log = IBCHandlerEvents::decode_log(&raw_log);
                    let ibc_event =
                        ibchandler_events_to_ibc_event(raw_log, &self.rpc, latest_block).await;

                    if let Some(ibc_event) = ibc_event {
                        self.handle_ibc_event(ibc_event, &shared_map, latest_block)
                            .await;
                    }
                    // let packet_event = IBCPacketEvents::try_from(raw_log).unwrap();
                    // tracing::info!("Packet event: {:?}", packet_event);
                })
                .await;

            // Update the latest block to the most recent block fetched
            if let Some(last_log) = logs.last() {
                latest_block = last_log.block_number.unwrap().as_u64() + 1;
            } else {
                // If no logs were found, just increment the block number to avoid getting stuck
                latest_block += 1;
            }

            // TODO: 6 seconds for new block fetching?
            tokio::time::sleep(Duration::from_secs(6)).await;
        }
    }

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        shared_map: &SharedMap,
        block_number: u64,
    ) {
        IbcListen::handle_ibc_event_boxed(self, ibc_event, shared_map, block_number).await;
    }
}
impl IbcListen for Cosmos {
    async fn listen(&self, shared_map: &SharedMap) {
        tracing::info!("Listening to Cosmos chain events");
        let mut subs = self
            .chain
            .tm_client
            .subscribe(tendermint_rpc::query::EventType::Tx.into())
            .await
            .unwrap();
        loop {
            tokio::select! {
                Some(event_result) = subs.next() => {
                    match event_result {
                        Ok(event) => {
                            let mut block_number: u64 = 0;
                            // tracing::info!("Received event_result: {:?}", event.clone());
                            if let Some(ref events) = event.events {
                                if let Some(heights) = events.get("tx.height") {
                                    if let Some(height) = heights.first() {
                                        block_number = height.parse().expect("Failed to parse block number");
                                        tracing::info!("Block number: {}", block_number);
                                    }
                                }
                            }

                            match event.data {
                                EventData::Tx { tx_result, .. } => {
                                    for event in tx_result.result.events {
                                        // tracing::info!("Received event: {:?}", event.clone());
                                        let Some(my_event) = IbcEvent::try_from_tendermint_event(TendermintEvent {
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
                                        let ibc_event = my_event.unwrap();
                                        self.handle_ibc_event(ibc_event, &shared_map, 13).await;
                                    }
                                }
                                _ => {
                                    tracing::error!("Unhandled event type: {:?}", event);
                                }
                            }
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

    async fn handle_ibc_event(
        &self,
        ibc_event: IbcEvent,
        shared_map: &SharedMap,
        block_number: u64,
    ) {
        IbcListen::handle_ibc_event_boxed(self, ibc_event, shared_map, block_number).await;
    }
}

impl IbcTransfer for Ethereum {
    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
        let mut rng = StdRng::from_entropy();
        let index = rng.gen_range(0..self.relays.len()); // Select a random index

        let relay = &self.relays[index];
        let signer_middleware = &self.signer_middlewares[index];
        let msg_sender = self.msg_senders[index];

        let denom: String = format!(
            "{}/{}/{}",
            self.ucs01_contract.to_lowercase(),
            destination_channel,
            denom
        );

        let denom_address = relay
            .get_denom_address(destination_channel.clone().to_string(), denom.clone())
            .call()
            .await
            .unwrap();
        if denom_address == ethers::types::H160::zero() {
            tracing::warn!("Denom address not found");
            return;
        }
        let erc_contract = erc20::ERC20::new(denom_address, signer_middleware.clone());
        let balance = erc_contract.balance_of(msg_sender).await.unwrap();
        tracing::info!("balance: {}, amount: {}", balance, amount);
        if balance < amount.into() {
            tracing::warn!("Insufficient balance");
            return;
        }

        let allowance = erc_contract
            .allowance(msg_sender, self.relay_addr)
            .await
            .unwrap();
        tracing::info!("allowance: {}", allowance);
        if allowance < amount.into() {
            erc_contract
                .approve(self.relay_addr, (U256::MAX / U256::from(2)).into())
                .send()
                .await;
        } else {
            tracing::info!("Already approved");
        }

        match protocol {
            Protocol::Ucs01 {
                receivers,
                contract,
            } => {
                let mut rng = StdRng::from_entropy();
                let index = rng.gen_range(0..receivers.len()); // Select a random index

                let receiver = &receivers[index];
                tracing::info!(
                    "Sending IBC transfer via Ethereum: {:?}, {:?}, {}, {}",
                    receiver,
                    contract,
                    denom,
                    amount
                );

                let (_hrp, data, _variant) =
                    bech32::decode(&receiver).expect("Invalid Bech32 address");

                let bytes: Vec<u8> = Vec::<u8>::from_base32(&data).expect("Invalid base32 data");

                let _tx_rcp: Option<ethers::types::TransactionReceipt> = match relay
                    .send(
                        destination_channel.clone().to_string(),
                        bytes.into(),
                        [LocalToken {
                            denom: denom_address,
                            amount: amount as u128,
                        }]
                        .into(),
                        "".into(),
                        (Height {
                            revision_number: 0,
                            revision_height: 0,
                        })
                        .into(),
                        u64::MAX,
                    )
                    .send()
                    .await
                {
                    Ok(response) => match response.await {
                        Ok(receipt) => Some(receipt.expect("Failed to get transaction receipt")),
                        Err(e) => {
                            tracing::error!("Failed to get transaction receipt: {:?}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to send transaction eth->union: {:?}", e);
                        return;
                    }
                };
                tracing::info!(
                    "Transaction sent successfully from eth. Tx: {:?}",
                    _tx_rcp.unwrap().transaction_hash
                );
            }
            Protocol::Ics20 { receivers, module } => {
                unimplemented!("Ics20 protocol not implemented"); // TODO: Do we even have this case?
            }
        }
    }
}

impl Ethereum {
    pub async fn new(config: EthereumConfig) -> Self {
        let ethereum_rpc = EthereumRpc {
            provider: Arc::new(Provider::new(
                Ws::connect(config.eth_rpc_api.clone()).await.unwrap(),
            )),
            ibc_handler_address: config.ibc_handler_address,
        };

        let mut relays = Vec::new();
        let mut signers_middleware = Vec::new();
        let mut msg_senders = Vec::new();

        let (relay_addr, ucs01_contract) = match config.transfer_module {
            TransferModule::Contract { ref address } => {
                let relay_addr: Address = address.parse().expect("Invalid contract address");
                (relay_addr, address.clone())
            }
            TransferModule::Native => {
                panic!("Native transfer module is not supported in this context")
            }
        };
        for signer in config.signers.clone() {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address_of_privkey: ethers::types::H160 = secret_key_to_address(&signing_key);
            tracing::info!("address: {:?}", address_of_privkey);

            let provider: Arc<Provider<Ws>> = ethereum_rpc.provider.clone();

            let chain_id = provider
                .get_chainid()
                .await
                .expect("Failed to get chain ID")
                .as_u64();
            let wallet = LocalWallet::new_with_signer(signing_key, address_of_privkey, chain_id);

            let signer_middleware = Arc::new(SignerMiddleware::new(
                NonceManagerMiddleware::new(provider.clone(), address_of_privkey),
                wallet.clone(),
            ));

            let relay = UCS01Relay::new(relay_addr, signer_middleware.clone());

            relays.push(relay);
            signers_middleware.push(signer_middleware);
            msg_senders.push(address_of_privkey);
        }

        Ethereum {
            rpc: ethereum_rpc,
            relays,
            signer_middlewares: signers_middleware,
            ucs01_contract,
            msg_senders,
            relay_addr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cosmos {
    pub chain: chain_utils::cosmos::Cosmos,
}

#[derive(Debug, Clone)]
pub struct Union {
    pub chain: chain_utils::union::Union,
}

// TODO(caglankaan): This will be %100 identical for osmosis too.
impl IbcTransfer for Cosmos {
    async fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        destination_channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
        self.chain
            .signers
            .with(|signer| async move {
                let transfer_msg = match protocol {
                    Protocol::Ics20 { receivers, module } => {
                        let mut rng = StdRng::from_entropy();
                        let index = rng.gen_range(0..receivers.len()); // Select a random index

                        let receiver = &receivers[index];
                        tracing::info!(
                            "Sending ibc transfer from: {:?}. Receiver: {:?}. amount: {:?}",
                            signer.to_string(),
                            receiver,
                            amount
                        );
                        let msg = MsgTransfer {
                            source_port: "transfer".into(),
                            source_channel: destination_channel.to_string(),
                            token: Some(
                                (Coin {
                                    denom: denom.to_string(),
                                    amount: amount as u128,
                                })
                                .into(),
                            ),
                            sender: signer.to_string(),
                            receiver: receiver.to_string(),
                            timeout_height: None,
                            timeout_timestamp: u64::MAX / 2,
                            memo: String::new(),
                        };

                        Any {
                            type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                            value: msg.encode_to_vec().into(),
                        }
                    }
                    Protocol::Ucs01 {
                        receivers,
                        contract,
                    } => {
                        let mut rng = StdRng::from_entropy();
                        let index = rng.gen_range(0..receivers.len()); // Select a random index
                        let receiver = &receivers[index];
                        tracing::info!(
                            "Sending ibc transfer from: {:?}. Receiver: {:?}. amount: {:?}",
                            signer.to_string(),
                            receiver,
                            amount
                        );

                        let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                            channel: destination_channel.to_string(),
                            receiver: receiver[2..].to_string(),
                            memo: Default::default(),
                            timeout: None,
                        });

                        let transfer_msg_bytes = serde_json::to_vec(&transfer_msg).unwrap();

                        any::Any(MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: contract.clone(),
                            msg: transfer_msg_bytes,
                            funds: vec![Coin {
                                denom: denom.to_string(),
                                amount: amount as u128,
                            }],
                        })
                        .into()
                    }
                };

                match self.chain.broadcast_tx_commit(signer, [transfer_msg]).await {
                    Ok(tx_hash) => {
                        tracing::info!("Transaction sent successfully. Hash: {:?}", tx_hash);
                    }
                    Err(e) => {
                        tracing::error!("Failed to submit tx!{:?}", e.to_string());
                    }
                }
            })
            .await;
    }
}

impl Cosmos {
    pub async fn new(config: CosmosConfig) -> Self {
        let cosmos = chain_utils::cosmos::Cosmos::new(config.chain_config)
            .await
            .unwrap();

        Cosmos { chain: cosmos }
    }
}

// TODO: Are there any other similar function to this? It's not good.
async fn ibchandler_events_to_ibc_event(
    log: RawLog,
    eth_rpcs: &EthereumRpc,
    block_number: u64,
) -> Option<IbcEvent> {
    match IBCHandlerEvents::decode_log(&log) {
        Ok(event) => {
            // tracing::info!("Decoded Ethereum log event: {:?}", event);
            // Handle the decoded event similarly to Tendermint events
            let ibc_event: Option<IbcEvent> = match event {
                IBCHandlerEvents::PacketEvent(packet_event) => match packet_event {
                    IBCPacketEvents::SendPacketFilter(event) => {
                        let channel: Channel = eth_rpcs
                            .ibc_handler()
                            .get_channel(
                                event.source_port.parse().unwrap(),
                                event.source_channel.parse().unwrap(),
                            )
                            .block(block_number)
                            .await
                            .unwrap()
                            .try_into()
                            .unwrap();
                        tracing::info!("channel: {:?}", channel);
                        Some(IbcEvent::SendPacket(SendPacket {
                            packet_sequence: event.sequence.try_into().unwrap(),
                            packet_src_port: event.source_port.parse().unwrap(),
                            packet_src_channel: event.source_channel.parse().unwrap(),
                            packet_dst_port: "RANDOM_VALUE".to_string().parse().unwrap(),
                            packet_dst_channel: channel
                                .counterparty
                                .channel_id
                                .to_string()
                                .parse()
                                .unwrap(),
                            packet_timeout_height: event.timeout_height.into(),
                            packet_timeout_timestamp: event.timeout_timestamp,
                            packet_data_hex: hex_encode(event.data).into(),
                            packet_channel_ordering:
                                unionlabs::ibc::core::channel::order::Order::NoneUnspecified,
                            connection_id: "connection-0".to_string().validate().unwrap(),
                        }))
                    }
                    IBCPacketEvents::RecvPacketFilter(event) => {
                        Some(IbcEvent::RecvPacket(RecvPacket {
                            packet_sequence: event.packet.sequence.try_into().unwrap(),
                            packet_src_port: event.packet.source_port.parse().unwrap(),
                            packet_src_channel: event.packet.source_channel.parse().unwrap(),
                            packet_dst_port: event.packet.destination_port.parse().unwrap(),
                            packet_dst_channel: event.packet.destination_channel.parse().unwrap(),
                            packet_timeout_height: event.packet.timeout_height.into(),
                            packet_timeout_timestamp: event.packet.timeout_timestamp,
                            packet_data_hex: hex_encode(event.packet.data).into(),
                            packet_channel_ordering:
                                unionlabs::ibc::core::channel::order::Order::NoneUnspecified,
                            connection_id: "connection-0".to_string().validate().unwrap(),
                        }))
                    }
                    IBCPacketEvents::AcknowledgePacketFilter(event) => {
                        Some(IbcEvent::AcknowledgePacket(AcknowledgePacket {
                            packet_sequence: event.packet.sequence.try_into().unwrap(),
                            packet_src_port: "RANDOM_VALUE".to_string().parse().unwrap(),
                            packet_src_channel: event.packet.source_channel.parse().unwrap(),
                            packet_dst_port: event.packet.destination_port.parse().unwrap(),
                            packet_dst_channel: event.packet.destination_channel.parse().unwrap(),
                            packet_timeout_height: event.packet.timeout_height.into(),
                            packet_timeout_timestamp: event.packet.timeout_timestamp,
                            packet_channel_ordering:
                                unionlabs::ibc::core::channel::order::Order::NoneUnspecified,
                            connection_id: "connection-0".to_string().validate().unwrap(),
                        }))
                    }
                    IBCPacketEvents::WriteAcknowledgementFilter(event) => {
                        Some(IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
                            packet_sequence: event.packet.sequence.try_into().unwrap(),
                            packet_src_port: "RANDOM_VALUE".to_string().parse().unwrap(),
                            packet_src_channel: event.packet.source_channel.parse().unwrap(),
                            packet_dst_port: event.packet.destination_port.parse().unwrap(),
                            packet_dst_channel: event.packet.destination_channel.parse().unwrap(),
                            packet_timeout_height: Height {
                                revision_number: 0,
                                revision_height: 0,
                            },
                            packet_ack_hex: event.acknowledgement.to_vec(),
                            packet_data_hex: hex_encode("RANDOM_VALUE").into(),
                            packet_timeout_timestamp: 0,
                            connection_id: "connection-0".to_string().validate().unwrap(),
                        }))
                    }
                    _ => {
                        tracing::warn!("Unhandled packet event type.");
                        None
                    }
                },
                _ => {
                    // tracing::warn!("Unhandled event type.");
                    None
                }
            };
            return ibc_event;
        }
        Err(e) => {
            // tracing::warn!("Could not decode Ethereum log event: {}", e);
            // tracing::warn!("Could not decode Ethereum log: {:?}", log);
        }
    }
    return None;
}
