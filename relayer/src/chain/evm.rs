use std::{
    ops::Div,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use beacon_api::client::BeaconApiClient;
use clap::Args;
use contracts::{
    devnet_ownable_ibc_handler::devnet_ownable_ibc_handler,
    glue::UnionIbcLightclientsCometblsV1HeaderData,
    ibc_handler::{
        self, GeneratedConnectionIdentifierFilter, GetChannelCall, GetChannelReturn,
        GetClientStateCall, GetClientStateReturn, GetConnectionCall, GetConnectionReturn,
        GetConsensusStateCall, GetConsensusStateReturn, GetHashedPacketCommitmentCall,
        GetHashedPacketCommitmentReturn, IBCHandler, IBCHandlerEvents, SendPacketFilter,
    },
    ics20_bank::ICS20Bank,
    ics20_transfer_bank::ICS20TransferBank,
    shared_types::{
        IbcCoreChannelV1ChannelData, IbcCoreChannelV1CounterpartyData,
        IbcCoreCommitmentV1MerklePrefixData, IbcCoreConnectionV1ConnectionEndData,
        IbcCoreConnectionV1CounterpartyData, IbcCoreConnectionV1VersionData,
    },
};
use ethers::{
    abi::{AbiEncode, Tokenizable},
    contract::EthCall,
    prelude::{decode_logs, k256::ecdsa, parse_log, LogMeta, SignerMiddleware},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer, Wallet},
    types::{Bytes, U256},
    utils::{keccak256, secret_key_to_address},
};
use futures::{Future, Stream, StreamExt};
use prost::Message;
use protos::{google, union::ibc::lightclients::ethereum::v1 as ethereum_v1};
use typenum::Unsigned;
use unionlabs::{
    ethereum::{beacon::LightClientFinalityUpdate, Address, H256},
    ethereum_consts_traits::ChainSpec,
    ibc::{
        core::{
            channel::{
                msg_channel_open_ack::MsgChannelOpenAck,
                msg_channel_open_confirm::MsgChannelOpenConfirm,
                msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
                msg_recv_packet::MsgRecvPacket, packet::Packet,
            },
            client::height::Height,
            connection::{
                msg_channel_open_ack::MsgConnectionOpenAck,
                msg_channel_open_confirm::MsgConnectionOpenConfirm,
                msg_channel_open_init::MsgConnectionOpenInit,
                msg_channel_open_try::MsgConnectionOpenTry,
            },
        },
        google::protobuf::any::Any,
        lightclients::{
            ethereum::{
                self,
                account_update::AccountUpdate,
                light_client_update::{LightClientUpdate, NextSyncCommitteeBranch},
                proof::Proof,
                sync_committee::SyncCommittee,
                trusted_sync_committee::TrustedSyncCommittee,
            },
            tendermint::fraction::Fraction,
            wasm,
        },
    },
    IntoProto, TryFromProto,
};

use crate::{
    chain::{
        cosmos::{Ethereum, Union},
        proof::{
            ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
            ConnectionPath, IbcPath,
        },
        Chain, ChainConnection, ClientStateOf, Connect, CreateClient, IbcStateRead, LightClient,
        StateProof,
    },
    config::EvmChainConfigFields,
};

pub const COMETBLS_CLIENT_TYPE: &str = "cometbls-new";

type CometblsMiddleware = SignerMiddleware<Provider<Ws>, Wallet<ecdsa::SigningKey>>;

/// The solidity light client, tracking the state of the 08-wasm light client on union.
// TODO(benluelo): Generic over middleware?
pub struct Cometbls<C: ChainSpec> {
    chain: Evm<C>,
}

fn encode_dynamic_singleton_tuple(t: impl AbiEncode) -> Vec<u8> {
    U256::from(32)
        .encode()
        .into_iter()
        .chain(t.encode())
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
pub struct Evm<C: ChainSpec> {
    // NOTE: pub temporarily, should be private
    pub wallet: LocalWallet,
    ibc_handler: IBCHandler<CometblsMiddleware>,
    provider: Provider<Ws>,
    beacon_api_client: BeaconApiClient<C>,

    // NOTE: pub temporarily, should be private
    pub ics20_bank: ICS20Bank<CometblsMiddleware>,
    cometbls_client_address: Address,
    ics20_transfer_bank: ICS20TransferBank<CometblsMiddleware>,

    // NOTE: This is required here due to the wrapping of client/ consensus state in wasm
    wasm_code_id: H256,
}

impl<C: ChainSpec> ChainConnection<Union> for Evm<C> {
    type LightClient = Cometbls<C>;

    fn light_client(&self) -> Self::LightClient {
        Cometbls {
            chain: self.clone(),
        }
    }
}

impl<C: ChainSpec> Evm<C> {
    pub async fn new(config: EvmChainConfigFields) -> Self {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await.unwrap());

        let chain_id = provider.get_chainid().await.unwrap();

        let signing_key: ethers::prelude::k256::ecdsa::SigningKey = config.signer.value();
        let address = secret_key_to_address(&signing_key);

        let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id.as_u64());

        let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet.clone()));

        let ics20_bank = ICS20Bank::new(config.ics20_bank_address, signer_middleware.clone());

        ics20_bank
            .set_operator(config.ics20_transfer_bank_address.clone().into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        Self {
            ibc_handler: IBCHandler::new(config.ibc_handler_address, signer_middleware.clone()),
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await,
            wasm_code_id: config.wasm_code_id,
            wallet,
            ics20_bank,
            cometbls_client_address: config.cometbls_client_address,
            ics20_transfer_bank: ICS20TransferBank::new(
                config.ics20_transfer_bank_address,
                signer_middleware.clone(),
            ),
        }
    }

    async fn execution_height(&self, beacon_height: Height) -> Height {
        let height = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Slot(
                beacon_height.revision_height,
            ))
            .await
            .unwrap()
            .data
            .message
            .body
            .execution_payload
            .block_number;

        let execution_height = self.make_height(height);

        tracing::debug!("beacon height {beacon_height} is execution height {execution_height}");

        execution_height
    }

    fn make_height(&self, height: impl Into<u64>) -> Height {
        // NOTE: Revision is always 1 for EVM
        // REVIEW: Consider using the fork revision?
        Height::new(0, height.into())
    }

    async fn wait_for_beacon_block(&self, requested_height: Height) {
        const WAIT_TIME: u64 = 3;

        loop {
            let current_height = self.query_latest_height().await;

            tracing::debug!(
                "waiting for beacon block {requested_height}, current height is {current_height}",
            );

            if current_height.revision_height >= requested_height.revision_height {
                break;
            }

            tracing::debug!(
                "requested height {requested_height} not yet reached, trying again in {WAIT_TIME} seconds"
            );
            tokio::time::sleep(std::time::Duration::from_secs(WAIT_TIME)).await;
        }
    }

    pub async fn wait_for_execution_block(&self, block_number: ethers::types::U64) {
        loop {
            let latest_finalized_block_number: u64 = self
                .beacon_api_client
                .finality_update()
                .await
                .unwrap()
                .data
                .attested_header
                .execution
                .block_number;

            tracing::debug!(
                %latest_finalized_block_number,
                waiting_for = %block_number,
                "waiting for block"
            );

            if latest_finalized_block_number >= block_number.as_u64() {
                break;
            }

            tracing::debug!("requested height not yet reached");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    pub async fn transfer(
        &self,
        denom: String,
        amount: u64,
        receiver: String,
        source_port: String,
        source_channel: String,
    ) {
        self.ics20_transfer_bank
            .send_transfer(
                denom.clone(),
                amount,
                receiver,
                source_port,
                source_channel,
                1,
                u64::MAX,
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        let balance = self
            .ics20_bank
            .balance_of(self.wallet.address(), denom.clone())
            .await
            .unwrap();

        tracing::info!(balance_before = %balance, %denom);

        let balance: U256 = self
            .ics20_bank
            .balance_of(self.wallet.address(), denom.clone())
            .await
            .unwrap();

        tracing::info!(balance_after = %balance, %denom);
    }

    pub async fn setup_initial_channel(
        &self,
        module_address: Address,
        channel_id: String,
        port_id: String,
        counterparty_port_id: String,
    ) {
        let signer_middleware = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.wallet.clone(),
        ));

        let ibc_handler = devnet_ownable_ibc_handler::DevnetOwnableIBCHandler::new(
            self.ibc_handler.address(),
            signer_middleware,
        );

        ibc_handler
            .setup_initial_channel(
                "connection-0".into(),
                IbcCoreConnectionV1ConnectionEndData {
                    client_id: "cometbls-new-0".into(),
                    versions: vec![IbcCoreConnectionV1VersionData {
                        identifier: "1".into(),
                        features: vec!["ORDER_ORDERED".into(), "ORDER_UNORDERED".into()],
                    }],
                    state: 3,
                    counterparty: IbcCoreConnectionV1CounterpartyData {
                        client_id: "08-wasm-0".into(),
                        connection_id: "connection-0".into(),
                        prefix: IbcCoreCommitmentV1MerklePrefixData {
                            key_prefix: b"ibc".to_vec().into(),
                        },
                    },
                    delay_period: 6,
                },
                port_id,
                channel_id.clone(),
                IbcCoreChannelV1ChannelData {
                    state: 3,
                    ordering: 1,
                    counterparty: IbcCoreChannelV1CounterpartyData {
                        port_id: counterparty_port_id,
                        channel_id,
                    },
                    connection_hops: vec!["connection-0".into()],
                    version: "ics20-1".into(),
                },
                module_address.into(),
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
    }
}

impl<C: ChainSpec> Chain for Evm<C> {
    type SelfClientState =
        Any<wasm::client_state::ClientState<ethereum::client_state::ClientState>>;
    type SelfConsensusState =
        Any<wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>>;

    fn chain_id(&self) -> impl Future<Output = String> + '_ {
        // TODO: Cache this in `self`, it only needs to be fetched once
        async move { self.provider.get_chainid().await.unwrap().to_string() }
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = self
                .beacon_api_client
                .finality_update()
                .await
                .unwrap()
                .data
                .attested_header
                .beacon
                .slot;

            self.make_height(height)
        }
    }

    fn self_client_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        async move {
            let genesis = self.beacon_api_client.genesis().await.unwrap().data;

            let execution_height = self.execution_height(beacon_height).await;

            Any(wasm::client_state::ClientState {
                data: ethereum::client_state::ClientState {
                    genesis_validators_root: genesis.genesis_validators_root,
                    genesis_time: genesis.genesis_time,
                    fork_parameters: self
                        .beacon_api_client
                        .spec()
                        .await
                        .unwrap()
                        .data
                        .into_fork_parameters(),
                    // REVIEW: Is this a preset config param? Or a per-chain config?
                    seconds_per_slot: C::SECONDS_PER_SLOT::U64,
                    slots_per_epoch: C::SLOTS_PER_EPOCH::U64,
                    epochs_per_sync_committee_period: C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64,
                    trusting_period: 100_000_000,
                    latest_slot: beacon_height.revision_height,
                    min_sync_committee_participants: 0,
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    frozen_height: None,
                    counterparty_commitment_slot: 0,
                },
                code_id: self.wasm_code_id.clone(),
                latest_height: execution_height,
            })
        }
    }

    fn self_consensus_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_ {
        async move {
            let trusted_header = self
                .beacon_api_client
                .header(beacon_api::client::BlockId::Finalized)
                .await
                .unwrap()
                .data;

            let bootstrap = self
                .beacon_api_client
                .bootstrap(trusted_header.root)
                .await
                .unwrap()
                .data;

            let light_client_update = {
                let current_period = beacon_height.revision_height.div(C::PERIOD::U64);

                tracing::info!(%current_period);

                let light_client_updates = self
                    .beacon_api_client
                    .light_client_updates(current_period, 1)
                    .await
                    .unwrap();

                let [light_client_update] = &*light_client_updates.0 else {
                    panic!()
                };

                light_client_update.data.clone()
            };

            Any(wasm::consensus_state::ConsensusState {
                data: ethereum::consensus_state::ConsensusState {
                    slot: bootstrap.header.beacon.slot,
                    // REVIEW: Should this be default?
                    storage_root: H256::default(),
                    timestamp: bootstrap.header.execution.timestamp,
                    current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
                    next_sync_committee: light_client_update
                        .next_sync_committee
                        .map(|nsc| nsc.aggregate_pubkey),
                },
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        }
    }

    fn packet_stream(
        &self,
    ) -> impl Future<Output = impl Stream<Item = (Height, Packet)> + '_> + '_ {
        async move {
            self.provider
                .subscribe_logs(&self.ibc_handler.event::<SendPacketFilter>().filter)
                .await
                .unwrap()
                .then(move |log| async move {
                    let meta = LogMeta::from(&log);
                    let event: SendPacketFilter = parse_log(log).unwrap();

                    // TODO: Would be nice if this info was passed through in the SendPacket event
                    let (channel_data, is_found): (
                        contracts::ibc_handler::IbcCoreChannelV1ChannelData,
                        bool,
                    ) = self
                        .ibc_handler
                        .get_channel(event.source_port.clone(), event.source_channel.clone())
                        .await
                        .unwrap();

                    assert!(
                        is_found,
                        "channel not found for port_id {port}, channel_id {channel}",
                        port = event.source_port,
                        channel = event.source_channel
                    );

                    (
                        self.make_height(meta.block_number.0[0]),
                        Packet {
                            sequence: event.sequence,
                            source_port: event.source_port,
                            source_channel: event.source_channel,
                            destination_port: channel_data.counterparty.port_id,
                            destination_channel: channel_data.counterparty.channel_id,
                            data: event.data.to_vec(),
                            timeout_height: event.timeout_height.into(),
                            timeout_timestamp: event.timeout_timestamp,
                        },
                    )
                })
        }
    }
}

impl<C: ChainSpec> CreateClient<Cometbls<C>> for Evm<C> {
    // fn new(&self) -> impl Future<Output = Cometbls<C>> + '_ {
    //     async move {
    //         Cometbls {
    //             chain: self.clone(),
    //         }
    //     }
    // }

    // fn new_with_id(&self, client_id: String) -> impl Future<Output = Option<Cometbls<C>>> + '_ {
    //     async move {
    //         // NOTE: There's currently no way to check if a client exists other than by fetching the
    //         // client state
    //         let (_, is_found) = self
    //             .ibc_handler
    //             .get_client_state(client_id.clone())
    //             .await
    //             .unwrap();

    //         is_found.then(|| Cometbls {
    //             chain: self.clone(),
    //         })
    //     }
    // }

    fn create_client(
        &self,
        _config: <Cometbls<C> as LightClient>::Config,
        counterparty_chain: <Cometbls<C> as LightClient>::CounterpartyChain,
    ) -> impl Future<Output = (String, Cometbls<C>)> + '_ {
        async move {
            let register_client_result = self.ibc_handler.register_client(
                COMETBLS_CLIENT_TYPE.into(),
                self.cometbls_client_address.clone().into(),
            );

            // TODO(benluelo): Better way to check if client type has already been registered?
            match register_client_result.send().await {
                Ok(ok) => {
                    ok.await.unwrap().unwrap();
                }
                Err(why) => eprintln!("{}", why.decode_revert::<String>().unwrap()),
            }

            tracing::info!(ibc_handler_address = ?self.ibc_handler.address());

            let latest_height = counterparty_chain.query_latest_height().await;

            let client_state = counterparty_chain.self_client_state(latest_height).await;
            let consensus_state = counterparty_chain.self_consensus_state(latest_height).await;

            let tx_rcp = self
                .ibc_handler
                .create_client(ibc_handler::MsgCreateClient {
                    // TODO: Extract this constant out somehow?
                    client_type: COMETBLS_CLIENT_TYPE.to_string(),
                    client_state_bytes: client_state.into_proto_bytes().into(),
                    consensus_state_bytes: consensus_state.into_proto_bytes().into(),
                })
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let client_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedClientIdentifierFilter(client_id) => Some(client_id.0),
                _ => None,
            })
            .unwrap();

            tracing::info!(
                block_number = ?self.make_height(tx_rcp.block_number.unwrap().as_u64()),
                client_id
            );

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                client_id,
                Cometbls {
                    chain: self.clone(),
                },
            )
        }
    }
}

impl<C: ChainSpec> LightClient for Cometbls<C> {
    // TODO(benluelo): Better type for this
    type UpdateClientMessage = UnionIbcLightclientsCometblsV1HeaderData;

    type IbcStateRead = EthStateRead;

    type HostChain = Evm<C>;

    type CounterpartyChain = Union;

    type Config = CometblsConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn update_client(
        &self,
        client_id: String,
        msg: Self::UpdateClientMessage,
    ) -> impl Future<Output = ()> + '_ {
        async move {
            self.chain
                .ibc_handler
                .update_client(ibc_handler::MsgUpdateClient {
                    client_id,
                    client_message: encode_dynamic_singleton_tuple(msg).into(),
                })
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn query_client_state(
        &self,
        client_id: String,
    ) -> impl Future<Output = ClientStateOf<Self::CounterpartyChain>> + '_ {
        async move {
            let (client_state_bytes, is_found) = self
                .chain
                .ibc_handler
                .get_client_state(client_id.clone())
                .await
                .unwrap();

            assert!(is_found);

            google::protobuf::Any::decode(&*client_state_bytes)
                .unwrap()
                .try_into()
                .unwrap()
        }
    }

    fn process_height_for_counterparty(&self, height: Height) -> impl Future<Output = Height> + '_ {
        self.chain.execution_height(height)
    }
}

impl<C: ChainSpec> Connect<Ethereum<C>> for Cometbls<C> {
    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_ {
        async move {
            let tx_rcp = self
                .chain
                .ibc_handler
                .connection_open_init(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            // TODO(benluelo): Better way to get logs
            let connection_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedConnectionIdentifierFilter(
                    GeneratedConnectionIdentifierFilter(connection_id),
                ) => {
                    tracing::info!(connection_id, "created connection");

                    Some(connection_id)
                }
                _ => None,
            })
            .unwrap();

            tracing::info!("in connection open init, waiting for execution block to be finalized");
            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                connection_id,
                self.chain
                    .make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<ClientStateOf<<Ethereum<C> as LightClient>::CounterpartyChain>>,
    ) -> impl Future<Output = (String, Height)> + '_ {
        async move {
            let tx_rcp = self
                .chain
                .ibc_handler
                .connection_open_try(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let connection_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedConnectionIdentifierFilter(connection_id) => {
                    Some(connection_id.0)
                }
                _ => None,
            })
            .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                connection_id,
                self.chain
                    .make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<ClientStateOf<<Ethereum<C> as LightClient>::CounterpartyChain>>,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            tracing::debug!(
                "Client state: {}",
                ethers::utils::hex::encode(msg.client_state.clone().into_proto().encode_to_vec())
            );

            let msg: contracts::ibc_handler::MsgConnectionOpenAck = msg.into();

            tracing::debug!(
                "Client state bytes {}",
                ethers::utils::hex::encode(&msg.client_state_bytes)
            );

            let tx_rcp = self
                .chain
                .ibc_handler
                .connection_open_ack(msg)
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.chain
                .make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn connection_open_confirm(
        &self,
        msg: MsgConnectionOpenConfirm,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            let tx_rcp = self
                .chain
                .ibc_handler
                .connection_open_confirm(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.chain
                .make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn channel_open_init(
        &self,
        msg: MsgChannelOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_ {
        async move {
            // TODO: Make sure this is done in both init and try
            let bind_port_result = self.chain.ibc_handler.bind_port(
                "transfer".to_string(),
                self.chain.ics20_transfer_bank.address(),
            );

            match bind_port_result.send().await {
                Ok(ok) => {
                    ok.await.unwrap().unwrap();
                }
                Err(why) => tracing::info!(why = ?why.decode_revert::<String>()),
            }

            let tx_rcp = self
                .chain
                .ibc_handler
                .channel_open_init(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let channel_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedChannelIdentifierFilter(channel_id) => {
                    Some(channel_id.0)
                }
                _ => None,
            })
            .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                channel_id,
                self.chain
                    .make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn channel_open_try(
        &self,
        msg: MsgChannelOpenTry,
    ) -> impl Future<Output = (String, Height)> + '_ {
        async move {
            let tx_rcp = self
                .chain
                .ibc_handler
                .channel_open_try(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let channel_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedChannelIdentifierFilter(channel_id) => {
                    Some(channel_id.0)
                }
                _ => None,
            })
            .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                channel_id,
                self.chain
                    .make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn channel_open_ack(&self, msg: MsgChannelOpenAck) -> impl Future<Output = Height> + '_ {
        async move {
            let tx_rcp = self
                .chain
                .ibc_handler
                .channel_open_ack(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.chain
                .make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn channel_open_confirm(
        &self,
        msg: MsgChannelOpenConfirm,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            let tx_rcp = self
                .chain
                .ibc_handler
                .channel_open_confirm(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.chain
                .wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.chain
                .make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn recv_packet(&self, packet: MsgRecvPacket) -> impl Future<Output = ()> + '_ {
        async move {
            self.chain
                .ibc_handler
                .recv_packet(packet.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a Ethereum<C>,
        counterparty_client_id: String,
        mut trusted_slot: Height,
        target_slot: Height,
    ) -> impl Future<Output = Height> + 'a {
        async move {
            // We need to wait until the target slot is attested, because the update
            // won't be available otherwise.
            self.chain.wait_for_beacon_block(target_slot).await;

            let finality_update = self
                .chain
                .beacon_api_client
                .finality_update()
                .await
                .unwrap();

            let target_period =
                self.sync_committee_period(finality_update.data.attested_header.beacon.slot);

            let trusted_period = self.sync_committee_period(trusted_slot.revision_height);

            assert!(
                trusted_period <= target_period,
                "trusted period {trusted_period} is behind target period {target_period}, something is wrong!",
            );

            // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
            // updates until we reach the `target_period - 1`.
            if trusted_period < target_period {
                tracing::debug!(
                    "updating sync committee from period {trusted_period} to {target_period}",
                );
                trusted_slot = self
                    .apply_sync_committee_updates(
                        counterparty,
                        &counterparty_client_id,
                        trusted_slot,
                        target_period,
                    )
                    .await;
            }

            if trusted_slot >= target_slot {
                return trusted_slot;
            }

            let execution_height = self.chain.execution_height(trusted_slot).await;

            let updated_height = self
                .chain
                .make_height(finality_update.data.attested_header.beacon.slot);
            let block_root = self
                .chain
                .beacon_api_client
                .header(beacon_api::client::BlockId::Slot(
                    trusted_slot.revision_height,
                ))
                .await
                .unwrap()
                .data
                .root;
            let bootstrap = self
                .chain
                .beacon_api_client
                .bootstrap(block_root)
                .await
                .unwrap();

            let header = self
                .make_finality_update(
                    finality_update.data,
                    TrustedSyncCommittee {
                        trusted_height: execution_height,
                        sync_committee: bootstrap.data.current_sync_committee,
                        is_next: false,
                    },
                    None,
                    None,
                )
                .await;

            let header_json = serde_json::to_string(&header).unwrap();

            tracing::info!(%header_json, "submitting finality update");

            counterparty
                .update_client(counterparty_client_id, header)
                .await;

            updated_height
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct CometblsConfig {
    // #[arg(long)]
    // pub cometbls_client_address: Address,
    // #[arg(long)]
    // pub ics20_transfer_address: Address,
    // #[arg(long)]
    // pub ics20_bank_address: Address,
}

impl<C: ChainSpec> Cometbls<C> {
    async fn apply_sync_committee_updates(
        &self,
        counterparty: &Ethereum<C>,
        counterparty_client_id: &str,
        mut trusted_slot: Height,
        target_period: u64,
    ) -> Height {
        let trusted_period = self.sync_committee_period(trusted_slot.revision_height);

        let light_client_updates = loop {
            let updates = self
                .chain
                .beacon_api_client
                .light_client_updates(trusted_period + 1, target_period - trusted_period)
                .await
                .unwrap();

            if updates
                .0
                .iter()
                .any(|update| update.data.finalized_header.beacon.slot == 0)
            {
                tracing::debug!("lightclient update not available yet; retrying in 3 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                continue;
            }

            break updates;
        };

        let mut trusted_block = self
            .chain
            .beacon_api_client
            .header(beacon_api::client::BlockId::Slot(
                trusted_slot.revision_height,
            ))
            .await
            .unwrap()
            .data;

        for light_client_update in light_client_updates.0 {
            tracing::debug!(
                light_client_update = %serde_json::to_string(&light_client_update).unwrap(),
                "applying light client update",
            );

            // bootstrap contains the current sync committee for the given height
            let bootstrap = self
                .chain
                .beacon_api_client
                .bootstrap(trusted_block.root.clone())
                .await
                .unwrap()
                .data;

            let header = self
                .make_update(
                    light_client_update.data.clone(),
                    TrustedSyncCommittee {
                        trusted_height: self
                            .chain
                            .execution_height(self.chain.make_height(bootstrap.header.beacon.slot))
                            .await,
                        sync_committee: bootstrap.current_sync_committee.clone(),
                        is_next: true,
                    },
                )
                .await;

            tracing::debug!(
                message = "Checking if updated height > update from revision height",
                update_to_finalized_slot = header.data.consensus_update.finalized_header.beacon.slot,
                update_to_attested_slot = header.data.consensus_update.attested_header.beacon.slot,
                %trusted_slot
            );

            // If we update, we also need to advance `update_from`
            if header.data.consensus_update.attested_header.beacon.slot
                > trusted_slot.revision_height
            {
                trusted_block = self
                    .chain
                    .beacon_api_client
                    .header(beacon_api::client::BlockId::Slot(
                        light_client_update.data.attested_header.beacon.slot,
                    ))
                    .await
                    .unwrap()
                    .data;

                tracing::debug!(
                    trusted_block = %serde_json::to_string(&trusted_block).unwrap(),
                    "updating trusted_block"
                );

                let old_trusted_slot = trusted_slot;

                trusted_slot = self
                    .chain
                    .make_height(header.data.consensus_update.attested_header.beacon.slot);

                tracing::debug!("updating trusted_slot from {old_trusted_slot} to {trusted_slot}");
            }

            tracing::debug!(header = %serde_json::to_string(&header).unwrap());

            counterparty
                .update_client(counterparty_client_id.into(), header)
                .await;
        }

        trusted_slot
    }

    #[allow(clippy::unused_self)] // a convenient way to get C
    fn sync_committee_period<H: Into<u64>>(&self, height: H) -> u64 {
        height.into().div(C::PERIOD::U64)
    }

    async fn make_update(
        &self,
        light_client_update: LightClientUpdate<C>,
        trusted_sync_committee: TrustedSyncCommittee<C>,
    ) -> wasm::header::Header<ethereum::header::Header<C>> {
        let execution_block_number = light_client_update.attested_header.execution.block_number;
        let updated_height = self.chain.make_height(execution_block_number);

        let account_update = self
            .chain
            .provider
            .get_proof(
                self.chain.ibc_handler.address(),
                vec![],
                // Proofs are from the execution layer, so we use execution height, not beacon slot.
                Some(execution_block_number.into()),
            )
            .await
            .unwrap();

        wasm::header::Header {
            height: updated_height,
            data: ethereum::header::Header {
                consensus_update: light_client_update,
                trusted_sync_committee,
                account_update: AccountUpdate {
                    proofs: [Proof {
                        key: self.chain.ibc_handler.address().as_bytes().to_vec(),
                        value: account_update.storage_hash.as_bytes().to_vec(),
                        proof: account_update
                            .account_proof
                            .into_iter()
                            .map(|x| x.to_vec())
                            .collect(),
                    }]
                    .to_vec(),
                },
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        }
    }

    async fn make_finality_update(
        &self,
        finality_update: LightClientFinalityUpdate<C>,
        trusted_sync_committee: TrustedSyncCommittee<C>,
        next_sync_committee: Option<SyncCommittee<C>>,
        next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    ) -> wasm::header::Header<ethereum::header::Header<C>> {
        self.make_update(
            LightClientUpdate {
                attested_header: finality_update.attested_header,
                next_sync_committee,
                next_sync_committee_branch,
                finalized_header: finality_update.finalized_header,
                finality_branch: finality_update.finality_branch,
                sync_aggregate: finality_update.sync_aggregate,
                signature_slot: finality_update.signature_slot,
            },
            trusted_sync_committee,
        )
        .await
    }
}

// pub trait IntoEthAbi: Into<Self::EthAbi> {
//     type EthAbi;

//     fn into_eth_abi(self) -> Self::EthAbi {
//         self.into()
//     }
// }

trait TupleToOption<P>
where
    P: IbcPath + IntoEthCall,
    <P as IntoEthCall>::EthCall: EthCallExt<Return = Self>,
{
    fn tuple_to_option<C: ChainSpec>(
        ret: <P::EthCall as EthCallExt>::Return,
    ) -> Option<P::Output<Cometbls<C>>>;
}

macro_rules! impl_eth_state_read {
    ($($Path:ident { $($field:ident),+ } -> $Call:ident $(-> $parse:expr)?;)+) => {
        $(
            impl From<$Path> for $Call {
                fn from($Path {
                    $($field),+
                }: $Path) -> Self {
                    Self {
                        $($field),+
                    }
                }
            }

            impl IntoEthCall for $Path {
                type EthCall = $Call;
            }

            impl TupleToOption<$Path> for <<$Path as IntoEthCall>::EthCall as EthCallExt>::Return {
                fn tuple_to_option<C: ChainSpec>(ret: <<$Path as IntoEthCall>::EthCall as EthCallExt>::Return) -> Option<<$Path as IbcPath>::Output<Cometbls<C>>> {
                    #[allow(clippy::redundant_closure_call)]
                    ret.1.then_some($(($parse))?(ret.0))
                }
            }
        )+
    }
}

// struct EthStateRead<C: ChainSpec, P: IbcPath<Cometbls<C>>>(PhantomData<(P, C)>);
pub struct EthStateRead;

impl<C: ChainSpec, P: 'static + IbcPath> IbcStateRead<Cometbls<C>, P> for EthStateRead
where
    P: IntoEthCall,
    <<P as IntoEthCall>::EthCall as EthCallExt>::Return: TupleToOption<P>,
{
    fn state_proof(
        light_client: &Cometbls<C>,
        path: P,
        at: Height,
    ) -> impl Future<Output = StateProof<P::Output<Cometbls<C>>>> + '_ {
        async move {
            let ret = light_client
                .chain
                .ibc_handler
                .method_hash::<P::EthCall, <P::EthCall as EthCallExt>::Return>(
                    P::EthCall::selector(),
                    path.clone().into(),
                )
                .expect("valid contract selector")
                .block(at.revision_height)
                .call()
                .await
                .map(<P::EthCall as EthCallExt>::Return::tuple_to_option)
                .unwrap()
                .unwrap();

            // let block_number = self.provider.get_block_number().await.unwrap();
            // tracing::info!(?block_number);

            let path = path.to_string();

            tracing::info!(path, ?at);

            let location = keccak256(
                keccak256(path.as_bytes())
                    .into_iter()
                    .chain(U256::from(0).encode())
                    .collect::<Vec<_>>(),
            );

            let proof = light_client
                .chain
                .provider
                .get_proof(
                    light_client.chain.ibc_handler.address(),
                    vec![location.into()],
                    Some(at.revision_height.into()),
                )
                .await
                .unwrap();

            tracing::info!(?proof);

            let proof = match <[_; 1]>::try_from(proof.storage_proof) {
                Ok([proof]) => proof,
                Err(invalid) => {
                    panic!("received invalid response from eth_getProof, expected length of 1 but got {invalid:#?}");
                }
            };

            StateProof {
                state: ret,
                proof: ethereum_v1::StorageProof {
                    proofs: [ethereum_v1::Proof {
                        key: proof.key.to_fixed_bytes().to_vec(),
                        // REVIEW(benluelo): Make sure this encoding works
                        value: proof.value.encode(),
                        proof: proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
                    }]
                    .to_vec(),
                }
                .encode_to_vec(),
                proof_height: at,
            }
        }
    }
}

impl_eth_state_read! {
    ClientStatePath { client_id } -> GetClientStateCall -> |x: Bytes| TryFromProto::try_from_proto_bytes(&x).unwrap();
    ConnectionPath { connection_id } -> GetConnectionCall -> |x| <ConnectionPath as IbcPath>::Output::<Cometbls<C>>::try_from(x).unwrap();
    ChannelEndPath { port_id, channel_id } -> GetChannelCall -> |x| <ChannelEndPath as IbcPath>::Output::<Cometbls<C>>::try_from(x).unwrap();
    CommitmentPath { port_id, channel_id, sequence } -> GetHashedPacketCommitmentCall;
}

// NOTE: Implemented this one manually since it's a bit different than the others
impl From<ClientConsensusStatePath> for GetConsensusStateCall {
    fn from(value: ClientConsensusStatePath) -> Self {
        Self {
            client_id: value.client_id,
            height: value.height.into(),
        }
    }
}

impl IntoEthCall for ClientConsensusStatePath {
    type EthCall = GetConsensusStateCall;
}

impl TupleToOption<ClientConsensusStatePath>
    for <<ClientConsensusStatePath as IntoEthCall>::EthCall as EthCallExt>::Return
{
    fn tuple_to_option<C: ChainSpec>(
        ret: <<ClientConsensusStatePath as IntoEthCall>::EthCall as EthCallExt>::Return,
    ) -> Option<<ClientConsensusStatePath as super::proof::IbcPath>::Output<Cometbls<C>>> {
        ret.p1
            .then(|| TryFromProto::try_from_proto_bytes(&ret.consensus_state_bytes).unwrap())
    }
}

/// Wrapper trait for a contract call's signature, to map the input type to the return type.
/// `ethers` generates both of these types, but doesn't correlate them.
pub trait EthCallExt: EthCall {
    type Return: Tokenizable;
}

macro_rules! impl_eth_call_ext {
    ($($Call:ident -> $Return:ident;)+) => {
        $(
            impl EthCallExt for $Call {
                type Return = $Return;
            }
        )+
    }
}

impl_eth_call_ext! {
    GetClientStateCall -> GetClientStateReturn;
    GetConsensusStateCall -> GetConsensusStateReturn;
    GetConnectionCall -> GetConnectionReturn;
    GetChannelCall -> GetChannelReturn;
    GetHashedPacketCommitmentCall -> GetHashedPacketCommitmentReturn;
}

pub trait IntoEthCall: Into<Self::EthCall> {
    type EthCall: EthCallExt;
}
