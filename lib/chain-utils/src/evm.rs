use std::{fmt::Display, ops::Div, str::FromStr, sync::Arc};

use beacon_api::client::BeaconApiClient;
use contracts::{
    ibc_handler::{
        GetChannelCall, GetChannelReturn, GetClientStateCall, GetClientStateReturn,
        GetConnectionCall, GetConnectionReturn, GetConsensusStateCall, GetConsensusStateReturn,
        GetHashedPacketAcknowledgementCommitmentCall,
        GetHashedPacketAcknowledgementCommitmentReturn, GetHashedPacketCommitmentCall,
        GetHashedPacketCommitmentReturn, IBCHandler, IBCHandlerEvents, WriteAcknowledgementFilter,
    },
    shared_types::{IbcCoreChannelV1ChannelData, IbcCoreConnectionV1ConnectionEndData},
};
use ethers::{
    abi::Tokenizable,
    contract::{ContractError, EthCall, EthLogDecode},
    core::k256::ecdsa,
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
    signers::{LocalWallet, Wallet},
    utils::secret_key_to_address,
};
use futures::{stream, Future, FutureExt, Stream, StreamExt};
use hubble::hasura::{insert_demo_tx, Datastore, HasuraConfig, HasuraDataStore, InsertDemoTx};
use serde::{Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    ethereum::{Address, H256, U256},
    ethereum_consts_traits::ChainSpec,
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        CreateClient, IbcEvent, RecvPacket, SendPacket,
    },
    ibc::{
        core::{
            channel::channel::Channel, client::height::Height,
            connection::connection_end::ConnectionEnd,
        },
        google::protobuf::any::Any,
        lightclients::{cometbls, ethereum, tendermint::fraction::Fraction, wasm},
    },
    id::{ChannelId, Id, IdParseError},
    id_type, EmptyString, TryFromEthAbiErrorOf, TryFromProto,
};

use crate::{
    chain_client_id, private_key::PrivateKey, Chain, ChainEvent, ClientState, EventSource, Pool,
};

pub type CometblsMiddleware =
    SignerMiddleware<NonceManagerMiddleware<Provider<Ws>>, Wallet<ecdsa::SigningKey>>;

// TODO(benluelo): Generic over middleware?
#[derive(Debug, Clone)]
pub struct Evm<C: ChainSpec> {
    pub chain_id: U256,

    pub readonly_ibc_handler: IBCHandler<Provider<Ws>>,

    // pub wallet: LocalWallet,
    pub ibc_handlers: Pool<IBCHandler<CometblsMiddleware>>,
    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient<C>,

    pub hasura_client: Option<HasuraDataStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: Address,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,

    pub hasura_config: Option<HasuraConfig>,
}

impl<C: ChainSpec> Chain for Evm<C> {
    // TODO: Unwrap these out of wasm, and re-wrap them in union
    type SelfClientState =
        Any<wasm::client_state::ClientState<ethereum::client_state::ClientState>>;
    type SelfConsensusState =
        Any<wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>>;

    type Header = wasm::header::Header<ethereum::header::Header<C>>;

    type Height = Height;

    type ClientId = EvmClientId;

    type ClientType = EvmClientType;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id
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

    fn query_latest_height_as_destination(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = self
                .beacon_api_client
                .block(beacon_api::client::BlockId::Head)
                .await
                .unwrap()
                .data
                .message
                .slot;

            self.make_height(height)
        }
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = i64> + '_ {
        async move {
            self.beacon_api_client
                .finality_update()
                .await
                .unwrap()
                .data
                .attested_header
                .execution
                .timestamp
                .try_into()
                .unwrap()
        }
    }

    fn self_client_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        async move {
            let genesis = self.beacon_api_client.genesis().await.unwrap().data;

            Any(wasm::client_state::ClientState {
                data: ethereum::client_state::ClientState {
                    chain_id: self.chain_id,
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
                code_id: H256::default(),
                latest_height: beacon_height,
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
                .header(beacon_api::client::BlockId::Slot(
                    beacon_height.revision_height,
                ))
                .await
                .unwrap()
                .data;

            let bootstrap = self
                .beacon_api_client
                .bootstrap(trusted_header.root)
                .await
                .unwrap()
                .data;

            assert!(bootstrap.header.beacon.slot == beacon_height.revision_height);

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

            let timestamp = bootstrap.header.execution.timestamp;
            Any(wasm::consensus_state::ConsensusState {
                data: ethereum::consensus_state::ConsensusState {
                    slot: bootstrap.header.beacon.slot,
                    // REVIEW: Should this be default?
                    storage_root: H256::default(),
                    timestamp,
                    current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
                    next_sync_committee: light_client_update
                        .next_sync_committee
                        .map(|nsc| nsc.aggregate_pubkey),
                },
                timestamp,
            })
        }
    }

    fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: ChannelId,
        destination_port_id: String,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            let filter = self
                .readonly_ibc_handler
                .write_acknowledgement_filter()
                .filter
                .at_block_hash(block_hash);

            let log = self.provider.get_logs(&filter).await.unwrap();

            log.into_iter()
                .map(|log| <WriteAcknowledgementFilter as EthLogDecode>::decode_log(&log.into()))
                .find_map(|e| match e {
                    Ok(WriteAcknowledgementFilter {
                        destination_port_id: ack_dst_port_id,
                        destination_channel,
                        sequence: ack_sequence,
                        acknowledgement,
                    }) if ack_dst_port_id == destination_port_id
                        && destination_channel == destination_channel_id.to_string()
                        && sequence == ack_sequence =>
                    {
                        Some(acknowledgement)
                    }
                    _ => None,
                })
                .unwrap_or_default()
                .to_vec()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EvmInitError {
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
}

impl<C: ChainSpec> Evm<C> {
    pub async fn new(config: Config) -> Result<Self, EvmInitError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;

        let ibc_handlers = config.signers.into_iter().map(|signer| {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address = secret_key_to_address(&signing_key);

            let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id.as_u64());

            let signer_middleware = Arc::new(SignerMiddleware::new(
                NonceManagerMiddleware::new(provider.clone(), address),
                wallet.clone(),
            ));

            IBCHandler::new(
                config.ibc_handler_address.clone(),
                signer_middleware.clone(),
            )
        });

        Ok(Self {
            chain_id: U256(chain_id),
            ibc_handlers: Pool::new(ibc_handlers),
            readonly_ibc_handler: IBCHandler::new(
                config.ibc_handler_address,
                provider.clone().into(),
            ),
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await,
            // wallet,
            hasura_client: config.hasura_config.map(|hasura_config| {
                HasuraDataStore::new(
                    reqwest::Client::new(),
                    hasura_config.url,
                    hasura_config.secret,
                )
            }),
        })
    }

    // TODO: Change to take a beacon slot instead of a height
    pub async fn execution_height(&self, beacon_height: Height) -> u64 {
        let response = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Slot(
                beacon_height.revision_height,
            ))
            .await;

        let height = response
            .unwrap()
            .data
            .message
            .body
            .execution_payload
            .block_number;

        tracing::debug!("beacon height {beacon_height} is execution height {height}");

        height
    }

    pub fn make_height(&self, height: impl Into<u64>) -> Height {
        // NOTE: Revision is always 0 for EVM
        // REVIEW: Consider using the fork revision?
        {
            Height {
                revision_number: 0,
                revision_height: height.into(),
            }
        }
    }

    pub async fn read_ibc_state<Call>(
        &self,
        call: Call,
        at_execution_height: u64,
    ) -> Result<
        Option<<<Call as EthCallExt>::Return as TupleToOption>::Inner>,
        ContractError<Provider<Ws>>,
    >
    where
        Call: EthCallExt + 'static,
        Call::Return: TupleToOption,
    {
        self.readonly_ibc_handler
            .method_hash::<Call, Call::Return>(Call::selector(), call)
            .expect("valid contract selector")
            .block(at_execution_height)
            .call()
            .await
            .map(Call::Return::tuple_to_option)
    }
}

chain_client_id! {
    #[ty = EvmClientType]
    pub enum EvmClientId {
        #[id(ty = "cometbls")]
        Cometbls(Id<_>),
    }
}

#[derive(Debug)]
pub enum EvmEventSourceError {
    Contract(ContractError<Provider<Ws>>),
    ChannelNotFound {
        port_id: String,
        channel_id: String,
    },
    ChannelConversion(TryFromEthAbiErrorOf<Channel>),
    ConnectionNotFound {
        connection_id: String,
    },
    ConnectionConversion(TryFromEthAbiErrorOf<ConnectionEnd<EvmClientId, String>>),
    // this is a mess, should be cleaned up
    ConnectionOpenInitConnectionConversion(
        TryFromEthAbiErrorOf<ConnectionEnd<EvmClientId, String, EmptyString>>,
    ),
    Parse(IdParseError),
    ClientTypeParse(<EvmClientType as FromStr>::Err),
    ClientIdParse(<EvmClientId as FromStr>::Err),
    EthAbi(ethers::core::abi::Error),
}

impl<C: ChainSpec> EventSource for Evm<C> {
    type Event = ChainEvent<Self>;
    type Error = EvmEventSourceError;

    // TODO: Make this the height to start from
    type Seed = ();

    fn events(self, _seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> {
        async move {
            let genesis_time = self
                .beacon_api_client
                .genesis()
                .await
                .unwrap()
                .data
                .genesis_time;

            let latest_height = self.query_latest_height().await;

            stream::unfold(
                (self, latest_height),
                move |(this, previous_beacon_height)| async move {
                    tracing::info!("fetching events");

                    let current_beacon_height = loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        let current_beacon_height = this.query_latest_height().await;

                        tracing::debug!(%current_beacon_height, %previous_beacon_height);

                        if current_beacon_height > previous_beacon_height {
                            break current_beacon_height;
                        }
                    };
                    tracing::debug!(
                        previous_beacon_height = previous_beacon_height.revision_height,
                        current_beacon_height = current_beacon_height.revision_height
                    );
                    let previous_execution_height =
                        this.execution_height(previous_beacon_height).await;
                    let current_execution_height =
                        this.execution_height(current_beacon_height).await;

                    let packets = futures::stream::iter(
                        this.provider
                            .get_logs(
                                &this
                                    .readonly_ibc_handler
                                    .events()
                                    .filter
                                    .from_block(previous_execution_height)
                                    .to_block(current_execution_height - 1),
                            )
                            .await
                            .unwrap(),
                    )
                    .then(|log| async {
                        // dbg!(&log);

                        let block_hash = log.block_hash.expect("log should have block_hash");

                        // let event_height = this.make_height(
                        //     log.block_number.expect("log should have block_number").0[0],
                        // );
                        let event_height =
                            log.block_number.expect("log should have block_number").0[0];

                        let event = IBCHandlerEvents::decode_log(&log.into());
                        let event = match event {
                            Ok(x) => x,
                            Err(e) => {
                                tracing::warn!("Failed to decode ibc handler event, we may need to regenerate them: {:?}", e);
                                return Ok::<_, EvmEventSourceError>(None::<ChainEvent<Evm<C>>>)
                            }
                        };

                        let read_channel = |port_id: String, channel_id: String| async {
                            this.read_ibc_state(
                                GetChannelCall {
                                    port_id: port_id.clone(),
                                    channel_id: channel_id.clone(),
                                },
                                event_height,
                            )
                            .await
                            .map_err(EvmEventSourceError::Contract)?
                            .ok_or(EvmEventSourceError::ChannelNotFound {
                                port_id,
                                channel_id,
                            })?
                            .try_into()
                            .map_err(EvmEventSourceError::ChannelConversion)
                        };

                        let read_connection = |connection_id: String| async {
                            this.read_ibc_state(
                                GetConnectionCall {
                                    connection_id: connection_id.clone(),
                                },
                                event_height,
                            )
                            .await
                            .map_err(EvmEventSourceError::Contract)?
                            .ok_or(EvmEventSourceError::ConnectionNotFound { connection_id })?
                            .try_into()
                            .map_err(EvmEventSourceError::ConnectionConversion)
                        };

                        let event = match event {
                            IBCHandlerEvents::AcknowledgePacketFilter(packet_ack) => {
                                // TODO: Would be nice if this info was passed through in the SendPacket event
                                let channel_data: Channel = read_channel(
                                    packet_ack.packet.source_port.clone(),
                                    packet_ack.packet.source_channel.clone(),
                                )
                                .await?;

                                Some(IbcEvent::AcknowledgePacket(AcknowledgePacket {
                                    packet_timeout_height: packet_ack.packet.timeout_height.into(),
                                    packet_timeout_timestamp: packet_ack.packet.timeout_timestamp,
                                    packet_sequence: packet_ack.packet.sequence,
                                    packet_src_port: packet_ack.packet.source_port,
                                    packet_src_channel: packet_ack
                                        .packet
                                        .source_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    packet_dst_port: packet_ack.packet.destination_port,
                                    packet_dst_channel: packet_ack
                                        .packet
                                        .destination_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    packet_channel_ordering: channel_data.ordering,
                                    connection_id: channel_data.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::ChannelCloseConfirmFilter(_) => todo!(),
                            IBCHandlerEvents::ChannelCloseInitFilter(_) => todo!(),
                            IBCHandlerEvents::ChannelOpenAckFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenAck(ChannelOpenAck {
                                    port_id: event.port_id,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    counterparty_port_id: channel.counterparty.port_id,
                                    counterparty_channel_id: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::ChannelOpenConfirmFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
                                    port_id: event.port_id,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    counterparty_port_id: channel.counterparty.port_id,
                                    counterparty_channel_id: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::ChannelOpenInitFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenInit(ChannelOpenInit {
                                    port_id: event.port_id,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    // TODO: Ensure that event.counterparty_channel_id is `EmptyString`
                                    counterparty_channel_id: EmptyString,
                                    counterparty_port_id: event.counterparty_port_id,
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    version: channel.version,
                                }))
                            }
                            IBCHandlerEvents::ChannelOpenTryFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenTry(ChannelOpenTry {
                                    port_id: event.port_id,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    counterparty_port_id: event.counterparty_port_id,
                                    counterparty_channel_id: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    version: event.version,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenAckFilter(event) => {
                                let connection: ConnectionEnd<<Self as Chain>::ClientId, String> =
                                    read_connection(event.connection_id.clone()).await?;

                                Some(IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenConfirmFilter(event) => {
                                let connection: ConnectionEnd<<Self as Chain>::ClientId, String> =
                                    read_connection(event.connection_id.clone()).await?;

                                Some(IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenInitFilter(event) => {
                                let connection: ConnectionEnd<
                                    <Self as Chain>::ClientId,
                                    String,
                                    EmptyString,
                                > = this
                                    .read_ibc_state(
                                        GetConnectionCall {
                                            connection_id: event.connection_id.clone(),
                                        },
                                        event_height,
                                    )
                                    .await
                                    .map_err(EvmEventSourceError::Contract)?
                                    .ok_or(EvmEventSourceError::ConnectionNotFound {
                                        connection_id: event.connection_id.clone(),
                                    })?
                                    .try_into()
                                    .map_err(
                                        EvmEventSourceError::ConnectionOpenInitConnectionConversion,
                                    )?;

                                Some(IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenTryFilter(event) => {
                                let connection: ConnectionEnd<<Self as Chain>::ClientId, String> =
                                    read_connection(event.connection_id.clone()).await?;

                                Some(IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::GeneratedClientIdentifierFilter(event) => {
                                let client_type = this
                                    .readonly_ibc_handler
                                    .client_types(event.0.clone())
                                    .await
                                    .map_err(EvmEventSourceError::Contract)?;

                                let (client_state, success) = this
                                    .readonly_ibc_handler
                                    .get_client_state(event.0.clone())
                                    .await
                                    .unwrap();

                                assert!(success);

                                let client_state = <Any<
                                    wasm::client_state::ClientState<
                                        cometbls::client_state::ClientState,
                                    >,
                                >>::try_from_proto_bytes(
                                    &client_state
                                )
                                .unwrap();

                                Some(IbcEvent::CreateClient(CreateClient {
                                    client_id: event
                                        .0
                                        .parse()
                                        .map_err(EvmEventSourceError::ClientIdParse)?,
                                    client_type: client_type
                                        .parse()
                                        .map_err(EvmEventSourceError::ClientTypeParse)?,
                                    consensus_height: client_state.0.latest_height,
                                }))
                            }
                            IBCHandlerEvents::RecvPacketFilter(event) => {
                                let channel = read_channel(
                                    event.packet.destination_port.clone(),
                                    event.packet.destination_channel.clone(),
                                )
                                .await?;

                                Some(IbcEvent::RecvPacket(RecvPacket {
                                    packet_data_hex: event.packet.data.to_vec(),
                                    packet_timeout_height: event.packet.timeout_height.into(),
                                    packet_timeout_timestamp: event.packet.timeout_timestamp,
                                    packet_sequence: event.packet.sequence,
                                    packet_src_port: event.packet.source_port,
                                    packet_src_channel: event
                                        .packet
                                        .source_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    packet_dst_port: event.packet.destination_port,
                                    packet_dst_channel: event
                                        .packet
                                        .destination_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    packet_channel_ordering: channel.ordering,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::SendPacketFilter(event) => {
                                let channel = read_channel(
                                    event.source_port.clone(),
                                    event.source_channel.clone(),
                                )
                                .await?;

                                Some(IbcEvent::SendPacket(SendPacket {
                                    packet_data_hex: event.data.to_vec(),
                                    packet_timeout_height: event.timeout_height.into(),
                                    packet_timeout_timestamp: event.timeout_timestamp,
                                    packet_sequence: event.sequence,
                                    packet_src_port: event.source_port,
                                    packet_src_channel: event
                                        .source_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    // REVIEW: Should we query the packet instead? Or is that the same info? Is it even possible to
                                    // query packets from the evm?
                                    packet_dst_port: channel.counterparty.port_id,
                                    packet_dst_channel: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::Parse)?,
                                    packet_channel_ordering: channel.ordering,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::WriteAcknowledgementFilter(_) => None,
                            IBCHandlerEvents::TimeoutPacketFilter(_) => None,
                        };

                        Ok(event.map(|event| {
                            ChainEvent::<Evm<C>> {
                                // TODO: Cache
                                chain_id: this.chain_id(),
                                block_hash: block_hash.into(),
                                height: current_beacon_height,
                                event,
                            }
                        }))
                    })
                    .filter_map(|x| async { x.transpose() })
                    .then(|event: Result<ChainEvent<Evm<C>>, EvmEventSourceError>| async {
                        if let Ok(ref event) = event {
                            let current_slot = event.height.revision_height;

                            let next_epoch_ts = next_epoch_timestamp::<C>(current_slot, genesis_time);

                                if let Some(hc) = &this.hasura_client {
                                    hc
                                        .do_post::<InsertDemoTx>(insert_demo_tx::Variables {
                                            data: serde_json::json! {{
                                                "latest_execution_block_hash": event.block_hash,
                                                "timestamp": next_epoch_ts,
                                            }},
                                        })
                                        .await
                                        .unwrap();
                                }
                        }

                        // pass it back through
                        event
                    });

                    let iter = futures::stream::iter(packets.collect::<Vec<_>>().await);

                    Some((iter, (this, current_beacon_height)))
                },
            )
            .flatten()
        }
        .flatten_stream()
    }
}

/// Many contract calls return some form of [`(bool, T)`] as a way to emulate nullable/[`Option`].
/// This trait allows for easy conversion from the aforementioned tuple to an [`Option`].
pub trait TupleToOption {
    type Inner;

    fn tuple_to_option(self) -> Option<Self::Inner>;
}

impl TupleToOption for GetClientStateReturn {
    type Inner = Vec<u8>;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0.to_vec())
    }
}

impl TupleToOption for GetConsensusStateReturn {
    type Inner = Vec<u8>;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.p1.then_some(self.consensus_state_bytes.to_vec())
    }
}

impl TupleToOption for GetConnectionReturn {
    type Inner = IbcCoreConnectionV1ConnectionEndData;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetChannelReturn {
    type Inner = IbcCoreChannelV1ChannelData;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetHashedPacketCommitmentReturn {
    type Inner = [u8; 32];

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetHashedPacketAcknowledgementCommitmentReturn {
    type Inner = [u8; 32];

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
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
    GetClientStateCall                           -> GetClientStateReturn;
    GetConsensusStateCall                        -> GetConsensusStateReturn;
    GetConnectionCall                            -> GetConnectionReturn;
    GetChannelCall                               -> GetChannelReturn;
    GetHashedPacketCommitmentCall                -> GetHashedPacketCommitmentReturn;
    GetHashedPacketAcknowledgementCommitmentCall -> GetHashedPacketAcknowledgementCommitmentReturn;
}

pub fn next_epoch_timestamp<C: ChainSpec>(slot: u64, genesis_timestamp: u64) -> u64 {
    let next_epoch_slot = slot + (C::SLOTS_PER_EPOCH::U64 - (slot % C::SLOTS_PER_EPOCH::U64));
    genesis_timestamp + (next_epoch_slot * C::SECONDS_PER_SLOT::U64)
}

// #[test]
// fn next_epoch_ts() {
//     dbg!(next_epoch_timestamp::<Mainnet>(6, 0));
//     dbg!(next_epoch_timestamp::<Mainnet>(7, 0));
//     dbg!(next_epoch_timestamp::<Mainnet>(8, 0));
//     dbg!(next_epoch_timestamp::<Mainnet>(9, 0));

//     dbg!(next_epoch_timestamp::<Minimal>(6, 0));
//     // dbg!(next_epoch::<Minimal>(48, 0));
//     // dbg!(next_epoch::<Minimal>(49, 0));
//     // dbg!(next_epoch::<Minimal>(47, 0));
// }
