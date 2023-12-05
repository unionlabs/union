use std::{fmt::Debug, num::ParseIntError, sync::Arc};

use dashmap::DashMap;
use ethers::prelude::k256::ecdsa;
use futures::{stream, Future, FutureExt, Stream, StreamExt};
use prost::Message;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tendermint_rpc::{query::Query, Client, Order, WebSocketClient, WebSocketClientUrl};
use unionlabs::{
    events::{IbcEvent, TryFromTendermintEventError, WriteAcknowledgement},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::cometbls,
    },
    id::{ClientId, ConnectionId},
    parse_wasm_client_type,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, IbcStateRead,
    },
    tendermint::abci::{event::Event, event_attribute::EventAttribute},
    traits::{Chain, ClientState, ClientStateOf, ConsensusStateOf},
    validated::ValidateT,
    CosmosAccountId, IntoEthAbi, MaybeRecoverableError, TryFromProto, TryFromProtoErrorOf,
    WasmClientType,
};

use crate::{
    private_key::PrivateKey,
    union::tm_types::{CosmosSdkError, SdkError},
    ChainEvent, EventSource, Pool,
};

#[derive(Debug, Clone)]
pub struct Union {
    pub chain_id: String,
    pub signers: Pool<CosmosAccountId>,
    pub fee_denom: String,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub prover_endpoint: String,
    pub grpc_url: String,

    pub code_id_cache: Arc<dashmap::DashMap<H256, WasmClientType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

impl Chain for Union {
    type SelfClientState = cometbls::client_state::ClientState;
    type SelfConsensusState = cometbls::consensus_state::ConsensusState;

    type Header = cometbls::header::Header;

    type Height = Height;

    type ClientId = UnionClientId;

    type ClientType = String;

    type Error = tendermint_rpc::Error;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id.clone()
    }

    fn query_latest_height(&self) -> impl Future<Output = Result<Height, Self::Error>> + '_ {
        async move {
            self.tm_client
                .latest_block()
                .await
                .map(|height| self.make_height(height.block.header.height.value()))
        }
    }

    fn query_latest_height_as_destination(
        &self,
    ) -> impl Future<Output = Result<Height, Self::Error>> + '_ {
        self.query_latest_height()
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = Result<i64, Self::Error>> + '_ {
        async move {
            let height = self.query_latest_height().await?;
            Ok(self
                .tm_client
                .block(u32::try_from(height.revision_height).unwrap())
                .await?
                .block
                .header
                .time
                .unix_timestamp())
        }
    }

    fn self_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        async move {
            let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
                self.grpc_url.clone(),
            )
            .await
            .unwrap()
            .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
            .await
            .unwrap()
            .into_inner()
            .params
            .unwrap();

            let commit = self
                .tm_client
                .commit(u32::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            let height = commit.signed_header.header.height;

            let unbonding_period: u64 = params
                .unbonding_time
                .clone()
                .unwrap()
                .seconds
                .try_into()
                .unwrap();

            cometbls::client_state::ClientState {
                chain_id: self.chain_id.clone(),
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                trusting_period: unbonding_period * 85 / 100,
                unbonding_period,
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                max_clock_drift: 60 * 20,
                frozen_height: Height {
                    revision_number: 0,
                    revision_height: 0,
                },
                latest_height: Height {
                    revision_number: self.chain_id.split('-').last().unwrap().parse().unwrap(),
                    revision_height: height.value(),
                },
            }
        }
    }

    fn self_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_ {
        async move {
            let commit = self
                .tm_client
                .commit(u32::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            cometbls::consensus_state::ConsensusState {
                timestamp: commit
                    .signed_header
                    .header
                    .time
                    .unix_timestamp()
                    .try_into()
                    .unwrap(),
                root: MerkleRoot {
                    hash: commit
                        .signed_header
                        .header
                        .app_hash
                        .as_bytes()
                        .to_vec()
                        .try_into()
                        .unwrap(),
                },
                next_validators_hash: commit
                    .signed_header
                    .header
                    .next_validators_hash
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .unwrap(),
            }
        }
    }

    fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: unionlabs::id::PortId,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            let block_height = self
                .tm_client
                .block_by_hash(block_hash.0.to_vec().try_into().unwrap())
                .await
                .unwrap()
                .block
                .unwrap()
                .header
                .height;

            tracing::info!(
                "Querying ack for {}/{}/{} at {}",
                destination_port_id,
                destination_channel_id,
                sequence,
                block_height
            );

            let wa = self
                .tm_client
                .tx_search(
                    Query::eq("tx.height", u64::from(block_height)),
                    false,
                    1,
                    255,
                    tendermint_rpc::Order::Ascending,
                )
                .await
                .unwrap()
                .txs
                .into_iter()
                .find_map(|tx| {
                    tx.tx_result.events.into_iter().find_map(|event| {
                        let maybe_ack = WriteAcknowledgement::try_from(
                            unionlabs::tendermint::abci::event::Event {
                                ty: event.kind,
                                attributes: event.attributes.into_iter().map(|attr| {
                                    unionlabs::tendermint::abci::event_attribute::EventAttribute {
                                        key: attr.key,
                                        value: attr.value,
                                        index: attr.index,
                                    }
                                }).collect()
                            },
                        );
                        match maybe_ack {
                            Ok(ok)
                                if ok.packet_sequence == sequence
                                    && ok.packet_dst_port == destination_port_id
                                    && ok.packet_dst_channel == destination_channel_id =>
                            {
                                Some(ok)
                            }
                            Ok(ok) => {
                                tracing::debug!("Found ack not matching our packet {:?}", ok);
                                None
                            }
                            Err(TryFromTendermintEventError::IncorrectType { .. }) => None,
                            Err(err) => {
                                panic!("{err:#?}")
                            }
                        }
                    })
                })
                .unwrap();

            wa.packet_ack_hex
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnionInitError {
    #[error("tendermint rpc error")]
    Tendermint(#[from] tendermint_rpc::Error),
    #[error(
        "unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`"
    )]
    // TODO: Once the `Id` trait in unionlabs is cleaned up to no longer use static id types, this error should just wrap `IdParseError`
    ChainIdParse {
        found: String,
        #[source]
        source: Option<ParseIntError>,
    },
}

impl Union {
    pub async fn new(config: Config) -> Result<Self, UnionInitError> {
        let (tm_client, driver) = WebSocketClient::builder(config.ws_url)
            .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
            .build()
            .await?;

        tokio::spawn(async move { driver.run().await });

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            signers: Pool::new(
                config
                    .signers
                    .into_iter()
                    .map(|signer| CosmosAccountId::new(signer.value(), "union".to_string())),
            ),
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoint: config.prover_endpoint,
            grpc_url: config.grpc_url,
            fee_denom: config.fee_denom,
            code_id_cache: Arc::new(DashMap::default()),
        })
    }

    pub async fn client_type_of_code_id(&self, code_id: H256) -> WasmClientType {
        if let Some(ty) = self.code_id_cache.get(&code_id) {
            tracing::debug!(
                code_id = %code_id.to_string_unprefixed(),
                ty = ?*ty,
                "cache hit for code_id"
            );

            return *ty;
        };

        tracing::info!(
            code_id = %code_id.to_string_unprefixed(),
            "cache miss for code_id"
        );

        let bz = protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .code(protos::ibc::lightclients::wasm::v1::QueryCodeRequest {
            code_id: code_id.to_string_unprefixed(),
        })
        .await
        .unwrap()
        .into_inner()
        .code;

        let ty = parse_wasm_client_type(bz).unwrap().unwrap();

        tracing::info!(
            code_id = %code_id.to_string_unprefixed(),
            ?ty,
            "parsed code_id"
        );

        self.code_id_cache.insert(code_id, ty);

        ty
    }

    pub async fn broadcast_tx_commit(
        &self,
        signer: CosmosAccountId,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
    ) -> Result<H256, BroadcastTxCommitError> {
        use protos::cosmos::tx;

        let account = self.account_info_of_signer(&signer).await;

        let sign_doc = tx::v1beta1::SignDoc {
            body_bytes: tx::v1beta1::TxBody {
                messages: messages.clone().into_iter().collect(),
                // TODO(benluelo): What do we want to use as our memo?
                memo: String::new(),
                timeout_height: 123_123_123,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }
            .encode_to_vec(),
            auth_info_bytes: tx::v1beta1::AuthInfo {
                signer_infos: [tx::v1beta1::SignerInfo {
                    public_key: Some(protos::google::protobuf::Any {
                        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                        value: signer.public_key().encode_to_vec(),
                    }),
                    mode_info: Some(tx::v1beta1::ModeInfo {
                        sum: Some(tx::v1beta1::mode_info::Sum::Single(
                            tx::v1beta1::mode_info::Single {
                                mode: tx::signing::v1beta1::SignMode::Direct.into(),
                            },
                        )),
                    }),
                    sequence: account.sequence,
                }]
                .to_vec(),
                fee: Some(tx::v1beta1::Fee {
                    amount: vec![protos::cosmos::base::v1beta1::Coin {
                        // TODO: This needs to be configurable
                        denom: self.fee_denom.clone(),
                        amount: "1".to_string(),
                    }],
                    gas_limit: 5_000_000_000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            }
            .encode_to_vec(),
            chain_id: self.chain_id.clone(),
            account_number: account.account_number,
        };

        let signature = signer
            .try_sign(&sign_doc.encode_to_vec())
            .expect("signing failed")
            .to_vec();

        let tx_raw = tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes,
            auth_info_bytes: sign_doc.auth_info_bytes,
            signatures: [signature].to_vec(),
        };

        let tx_raw_bytes = tx_raw.encode_to_vec();

        let tx_hash = hex::encode_upper(sha2::Sha256::new().chain_update(&tx_raw_bytes).finalize());

        if self
            .tm_client
            .tx(tx_hash.parse().unwrap(), false)
            .await
            .is_ok()
        {
            tracing::info!(%tx_hash, "tx already included");
            return Ok(tx_hash.parse().unwrap());
        }

        let response_result = self.tm_client.broadcast_tx_sync(tx_raw_bytes.clone()).await;

        let response = response_result.unwrap();

        assert_eq!(
            tx_hash,
            response.hash.to_string(),
            "tx hash calculated incorrectly"
        );

        tracing::debug!(%tx_hash);

        tracing::info!(check_tx_code = ?response.code, codespace = %response.codespace, check_tx_log = %response.log);

        let tx_hash_normalized = H256(hex::decode(&tx_hash).unwrap().try_into().unwrap());

        if response.code.is_err() {
            let value = tm_types::CosmosSdkError::from_code_and_codespace(
                &response.codespace,
                response.code.value(),
            );

            tracing::error!("cosmos tx failed: {}", value);

            return Ok(tx_hash_normalized);
        };

        let mut target_height = self.query_latest_height().await?.increment();
        let mut i = 0;
        loop {
            let reached_height = 'l: loop {
                let current_height = self.query_latest_height().await?;
                if current_height >= target_height {
                    break 'l current_height;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            };

            let tx_inclusion = self.tm_client.tx(tx_hash.parse().unwrap(), false).await;

            tracing::debug!(?tx_inclusion);

            match tx_inclusion {
                Ok(_) => break Ok(tx_hash_normalized),
                Err(err) if i > 5 => {
                    tracing::warn!("tx inclusion couldn't be retrieved after {} try", i);
                    break Err(BroadcastTxCommitError::Inclusion(err));
                }
                Err(_) => {
                    target_height = reached_height.increment();
                    i += 1;
                    continue;
                }
            }
        }
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }

    async fn account_info_of_signer(
        &self,
        signer: &CosmosAccountId,
    ) -> protos::cosmos::auth::v1beta1::BaseAccount {
        let account = protos::cosmos::auth::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .account(protos::cosmos::auth::v1beta1::QueryAccountRequest {
            address: signer.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();

        // TODO: Type in `unionlabs` for this
        assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

        protos::cosmos::auth::v1beta1::BaseAccount::decode(&*account.value).unwrap()
    }

    pub async fn code_id_of_client_id(&self, client_id: UnionClientId) -> H256 {
        let client_state = protos::ibc::core::client::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
            client_id: client_id.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .client_state
        .unwrap();

        // TODO: Type in `unionlabs` for this
        assert!(client_state.type_url == "/ibc.lightclients.wasm.v1.ClientState");

        protos::ibc::lightclients::wasm::v1::ClientState::decode(&*client_state.value)
            .unwrap()
            .code_id
            .try_into()
            .unwrap()
    }

    pub async fn client_id_of_connection(&self, connection_id: ConnectionId) -> UnionClientId {
        protos::ibc::core::connection::v1::query_client::QueryClient::connect(self.grpc_url.clone())
            .await
            .unwrap()
            .connection(protos::ibc::core::connection::v1::QueryConnectionRequest {
                connection_id: connection_id.to_string(),
            })
            .await
            .unwrap()
            .into_inner()
            .connection
            .unwrap()
            .client_id
            .validate()
            .unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastTxCommitError {
    #[error("tx was not included")]
    Inclusion(#[from] tendermint_rpc::Error),
    #[error("tx failed: {0:?}")]
    Tx(CosmosSdkError),
}

impl MaybeRecoverableError for BroadcastTxCommitError {
    fn is_recoverable(&self) -> bool {
        match self {
            // tx wasn't included, retry unconditionally
            Self::Inclusion(_) => true,
            Self::Tx(code) => matches!(
                code,
                CosmosSdkError::SdkError(SdkError::ErrTxInMempoolCache)
                    | CosmosSdkError::SdkError(SdkError::ErrMempoolIsFull)
                    | CosmosSdkError::SdkError(SdkError::ErrTxTimeoutHeight)
                    | CosmosSdkError::SdkError(SdkError::ErrWrongSequence)
            ),
        }
    }
}

pub type UnionClientId = ClientId;

#[derive(Debug)]
pub enum UnionEventSourceError {
    TryFromTendermintEvent(TryFromTendermintEventError),
    Subscription(tendermint_rpc::Error),
}

impl EventSource for Union {
    type Event = ChainEvent<Self>;
    type Error = UnionEventSourceError;
    // TODO: Make this the height to start from
    type Seed = ();

    fn events(self, _seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> {
        async move {
            let chain_revision = self.chain_revision;

            let latest_height = self.query_latest_height().await.unwrap();

            stream::unfold(
                (self, latest_height),
                move |(this, previous_height)| async move {
                    tracing::info!("fetching events");

                    let current_height = loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        let current_height = match this.query_latest_height().await {
                            Ok(current_height) => current_height,
                            Err(e) => {
                                tracing::error!(error = %e, "Error getting height from Union. Trying again in 1 second.");
                                continue;
                            }
                        };

                        tracing::debug!(%current_height, %previous_height);

                        if current_height > previous_height {
                            break current_height;
                        }
                    };

                    tracing::debug!(
                        previous_height = previous_height.revision_height,
                        current_height = current_height.revision_height
                    );

                    let mut events = vec![];

                    for h in
                        (previous_height.revision_height + 1)..=(current_height.revision_height)
                    {
                        let response = if let Ok(res) = this
                            .tm_client
                            .tx_search(Query::eq("tx.height", h), false, 1, 255, Order::Descending)
                            .await
                        {
                            res
                        } else {
                            return None;
                        };

                        let new_events = stream::iter(response.txs.into_iter().flat_map(|tx| {
                            tx.tx_result
                                .events
                                .into_iter()
                                .map(|event| Event {
                                    ty: event.kind,
                                    attributes: event
                                        .attributes
                                        .into_iter()
                                        .map(|attr| EventAttribute {
                                            key: attr.key,
                                            value: attr.value,
                                            index: attr.index,
                                        })
                                        .collect(),
                                })
                                .filter_map(IbcEvent::try_from_tendermint_event)
                                .map(move |res| {
                                    res.map(|x| (tx.height, x))
                                        .map_err(UnionEventSourceError::TryFromTendermintEvent)
                                })
                        }))
                        .then(|res| async {
                            match res {
                                Ok((height, event)) =>
                                    match this.tm_client.block(height).await {
                                    Ok(block) => Ok(ChainEvent {
                                            chain_id: this.chain_id(),
                                            block_hash: block
                                                .block_id
                                                .hash
                                                .as_bytes()
                                                .try_into()
                                                .unwrap(),
                                            height: Height {
                                                revision_number: chain_revision,
                                                revision_height: height.try_into().unwrap(),
                                            },
                                            event,
                                        }),
                                        Err(e) => Err(UnionEventSourceError::Subscription(e)), },
                                Err(err) => Err(err)
                            }
                        })
                        .collect::<Vec<_>>()
                        .await;

                        events.extend(new_events);
                    }

                    let iter = events;

                    Some((iter, (this, current_height)))
                },
            )
        }
        .flatten_stream()
        .map(futures::stream::iter)
        .flatten()
    }
}

#[allow(non_upper_case_globals)] // TODO: Report this upstream to num_enum
pub mod tm_types {
    macro_rules! cosmos_sdk_errors {
        (
            $(
                #[err(name = $Module:ident, codespace = $codespace:literal)]
                var (
                    $(
                    	$Err:ident = errorsmod.Register($ModuleName_:ident, $code:literal, $msg:literal)
                    )+
                )
            )+
        ) => {
            #[derive(
                Debug,
                Clone,
                PartialEq,
                Eq,
                Hash,
                thiserror::Error,
            )]
            pub enum CosmosSdkError {
                $(
                    #[error(transparent)]
                    $Module(#[from] $Module),
                )+
                #[error("unknown error: {0}: {1}")]
                Unknown(String, u32),
            }

            impl CosmosSdkError {
                pub fn from_code_and_codespace(codespace: &str, code: u32) -> Self {
                    match codespace {
                        $(
                            $codespace => $Module::try_from(code)
                                .map(Into::into)
                                .map_err(|x| x.number),
                        )+
                        _ => return Self::Unknown(codespace.to_string(), code),
                    }
                    .map_or_else(
                        |code| Self::Unknown(codespace.to_string(), code),
                        std::convert::identity,
                    )
                }
            }

            $(
                #[derive(
                    Debug,
                    Copy,
                    Clone,
                    PartialEq,
                    Eq,
                    Hash,
                    num_enum::IntoPrimitive,
                    num_enum::TryFromPrimitive,
                    thiserror::Error,
                )]
                #[repr(u32)]
                pub enum $Module {
                    $(
                        #[error($msg)]
                        $Err = $code,
                    )+
                }
            )+
        }
    }

    cosmos_sdk_errors! {
        #[err(name = HostError, codespace = "host")]
        var (
            ErrInvalidID     = errorsmod.Register(SubModuleName, 2, "invalid identifier")
            ErrInvalidPath   = errorsmod.Register(SubModuleName, 3, "invalid path")
            ErrInvalidPacket = errorsmod.Register(SubModuleName, 4, "invalid packet")
        )

        #[err(name = IbcError, codespace = "ibc")]
        var (
            // ErrInvalidSequence is used the sequence number (nonce) is incorrect
            // for the signature.
            ErrInvalidSequence = errorsmod.Register(codespace, 1, "invalid sequence")

            // ErrUnauthorized is used whenever a request without sufficient
            // authorization is handled.
            ErrUnauthorized = errorsmod.Register(codespace, 2, "unauthorized")

            // ErrInsufficientFunds is used when the account cannot pay requested amount.
            ErrInsufficientFunds = errorsmod.Register(codespace, 3, "insufficient funds")

            // ErrUnknownRequest is used when the request body.
            ErrUnknownRequest = errorsmod.Register(codespace, 4, "unknown request")

            // ErrInvalidAddress is used when an address is found to be invalid.
            ErrInvalidAddress = errorsmod.Register(codespace, 5, "invalid address")

            // ErrInvalidCoins is used when sdk.Coins are invalid.
            ErrInvalidCoins = errorsmod.Register(codespace, 6, "invalid coins")

            // ErrOutOfGas is used when there is not enough gas.
            ErrOutOfGas = errorsmod.Register(codespace, 7, "out of gas")

            // ErrInvalidRequest defines an ABCI typed error where the request contains
            // invalid data.
            ErrInvalidRequest = errorsmod.Register(codespace, 8, "invalid request")

            // ErrInvalidHeight defines an error for an invalid height
            ErrInvalidHeight = errorsmod.Register(codespace, 9, "invalid height")

            // ErrInvalidVersion defines a general error for an invalid version
            ErrInvalidVersion = errorsmod.Register(codespace, 10, "invalid version")

            // ErrInvalidChainID defines an error when the chain-id is invalid.
            ErrInvalidChainID = errorsmod.Register(codespace, 11, "invalid chain-id")

            // ErrInvalidType defines an error an invalid type.
            ErrInvalidType = errorsmod.Register(codespace, 12, "invalid type")

            // ErrPackAny defines an error when packing a protobuf message to Any fails.
            ErrPackAny = errorsmod.Register(codespace, 13, "failed packing protobuf message to Any")

            // ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
            ErrUnpackAny = errorsmod.Register(codespace, 14, "failed unpacking protobuf message from Any")

            // ErrLogic defines an internal logic error, e.g. an invariant or assertion
            // that is violated. It is a programmer error, not a user-facing error.
            ErrLogic = errorsmod.Register(codespace, 15, "internal logic error")

            // ErrNotFound defines an error when requested entity doesn't exist in the state.
            ErrNotFound = errorsmod.Register(codespace, 16, "not found")
        )

        #[err(name = PortError, codespace = "port")]
        var (
            // cspell:ignore binded
            ErrPortExists   = errorsmod.Register(SubModuleName, 2, "port is already binded")
            ErrPortNotFound = errorsmod.Register(SubModuleName, 3, "port not found")
            ErrInvalidPort  = errorsmod.Register(SubModuleName, 4, "invalid port")
            ErrInvalidRoute = errorsmod.Register(SubModuleName, 5, "route not found")
        )

        #[err(name = SdkError, codespace = "sdk")]
        var (
            // ErrTxDecode is returned if we cannot parse a transaction
            ErrTxDecode = errorsmod.Register(RootCodespace, 2, "tx parse error")

            // ErrInvalidSequence is used the sequence number (nonce) is incorrect
            // for the signature
            ErrInvalidSequence = errorsmod.Register(RootCodespace, 3, "invalid sequence")

            // ErrUnauthorized is used whenever a request without sufficient
            // authorization is handled.
            ErrUnauthorized = errorsmod.Register(RootCodespace, 4, "unauthorized")

            // ErrInsufficientFunds is used when the account cannot pay requested amount.
            ErrInsufficientFunds = errorsmod.Register(RootCodespace, 5, "insufficient funds")

            // ErrUnknownRequest to doc
            ErrUnknownRequest = errorsmod.Register(RootCodespace, 6, "unknown request")

            // ErrInvalidAddress to doc
            ErrInvalidAddress = errorsmod.Register(RootCodespace, 7, "invalid address")

            // ErrInvalidPubKey to doc
            ErrInvalidPubKey = errorsmod.Register(RootCodespace, 8, "invalid pubkey")

            // ErrUnknownAddress to doc
            ErrUnknownAddress = errorsmod.Register(RootCodespace, 9, "unknown address")

            // ErrInvalidCoins to doc
            ErrInvalidCoins = errorsmod.Register(RootCodespace, 10, "invalid coins")

            // ErrOutOfGas to doc
            ErrOutOfGas = errorsmod.Register(RootCodespace, 11, "out of gas")

            // ErrMemoTooLarge to doc
            ErrMemoTooLarge = errorsmod.Register(RootCodespace, 12, "memo too large")

            // ErrInsufficientFee to doc
            ErrInsufficientFee = errorsmod.Register(RootCodespace, 13, "insufficient fee")

            // ErrTooManySignatures to doc
            ErrTooManySignatures = errorsmod.Register(RootCodespace, 14, "maximum number of signatures exceeded")

            // ErrNoSignatures to doc
            ErrNoSignatures = errorsmod.Register(RootCodespace, 15, "no signatures supplied")

            // ErrJSONMarshal defines an ABCI typed JSON marshalling error
            ErrJSONMarshal = errorsmod.Register(RootCodespace, 16, "failed to marshal JSON bytes")

            // ErrJSONUnmarshal defines an ABCI typed JSON unmarshalling error
            ErrJSONUnmarshal = errorsmod.Register(RootCodespace, 17, "failed to unmarshal JSON bytes")

            // ErrInvalidRequest defines an ABCI typed error where the request contains
            // invalid data.
            ErrInvalidRequest = errorsmod.Register(RootCodespace, 18, "invalid request")

            // ErrTxInMempoolCache defines an ABCI typed error where a tx already exists
            // in the mempool.
            ErrTxInMempoolCache = errorsmod.Register(RootCodespace, 19, "tx already in mempool")

            // ErrMempoolIsFull defines an ABCI typed error where the mempool is full.
            ErrMempoolIsFull = errorsmod.Register(RootCodespace, 20, "mempool is full")

            // ErrTxTooLarge defines an ABCI typed error where tx is too large.
            ErrTxTooLarge = errorsmod.Register(RootCodespace, 21, "tx too large")

            // ErrKeyNotFound defines an error when the key doesn't exist
            ErrKeyNotFound = errorsmod.Register(RootCodespace, 22, "key not found")

            // ErrWrongPassword defines an error when the key password is invalid.
            ErrWrongPassword = errorsmod.Register(RootCodespace, 23, "invalid account password")

            // ErrorInvalidSigner defines an error when the tx intended signer does not match the given signer.
            ErrorInvalidSigner = errorsmod.Register(RootCodespace, 24, "tx intended signer does not match the given signer")

            // ErrorInvalidGasAdjustment defines an error for an invalid gas adjustment
            ErrorInvalidGasAdjustment = errorsmod.Register(RootCodespace, 25, "invalid gas adjustment")

            // ErrInvalidHeight defines an error for an invalid height
            ErrInvalidHeight = errorsmod.Register(RootCodespace, 26, "invalid height")

            // ErrInvalidVersion defines a general error for an invalid version
            ErrInvalidVersion = errorsmod.Register(RootCodespace, 27, "invalid version")

            // ErrInvalidChainID defines an error when the chain-id is invalid.
            ErrInvalidChainID = errorsmod.Register(RootCodespace, 28, "invalid chain-id")

            // ErrInvalidType defines an error an invalid type.
            ErrInvalidType = errorsmod.Register(RootCodespace, 29, "invalid type")

            // ErrTxTimeoutHeight defines an error for when a tx is rejected out due to an
            // explicitly set timeout height.
            ErrTxTimeoutHeight = errorsmod.Register(RootCodespace, 30, "tx timeout height")

            // ErrUnknownExtensionOptions defines an error for unknown extension options.
            ErrUnknownExtensionOptions = errorsmod.Register(RootCodespace, 31, "unknown extension options")

            // ErrWrongSequence defines an error where the account sequence defined in
            // the signer info doesn't match the account's actual sequence number.
            ErrWrongSequence = errorsmod.Register(RootCodespace, 32, "incorrect account sequence")

            // ErrPackAny defines an error when packing a protobuf message to Any fails.
            ErrPackAny = errorsmod.Register(RootCodespace, 33, "failed packing protobuf message to Any")

            // ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
            ErrUnpackAny = errorsmod.Register(RootCodespace, 34, "failed unpacking protobuf message from Any")

            // ErrLogic defines an internal logic error, e.g. an invariant or assertion
            // that is violated. It is a programmer error, not a user-facing error.
            ErrLogic = errorsmod.Register(RootCodespace, 35, "internal logic error")

            // ErrConflict defines a conflict error, e.g. when two goroutines try to access
            // the same resource and one of them fails.
            ErrConflict = errorsmod.Register(RootCodespace, 36, "conflict")

            // ErrNotSupported is returned when we call a branch of a code which is currently not
            // supported.
            ErrNotSupported = errorsmod.Register(RootCodespace, 37, "feature not supported")

            // ErrNotFound defines an error when requested entity doesn't exist in the state.
            ErrNotFound = errorsmod.Register(RootCodespace, 38, "not found")

            // ErrIO should be used to wrap internal errors caused by external operation.
            // Examples: not DB domain error, file writing etc...
            ErrIO = errorsmod.Register(RootCodespace, 39, "Internal IO error")

            // ErrAppConfig defines an error occurred if min-gas-prices field in BaseConfig is empty.
            ErrAppConfig = errorsmod.Register(RootCodespace, 40, "error in app.toml")

            // ErrInvalidGasLimit defines an error when an invalid GasWanted value is
            // supplied.
            ErrInvalidGasLimit = errorsmod.Register(RootCodespace, 41, "invalid gas limit")

            // ErrPanic should only be set when we recovering from a panic
            // TODO: Figure out what this is and where it comes from
            // ErrPanic = errorsmod.ErrPanic
        )

        #[err(name = ChannelError, codespace = "channel")]
        var (
            ErrChannelExists             = errorsmod.Register(SubModuleName, 2, "channel already exists")
            ErrChannelNotFound           = errorsmod.Register(SubModuleName, 3, "channel not found")
            ErrInvalidChannel            = errorsmod.Register(SubModuleName, 4, "invalid channel")
            ErrInvalidChannelState       = errorsmod.Register(SubModuleName, 5, "invalid channel state")
            ErrInvalidChannelOrdering    = errorsmod.Register(SubModuleName, 6, "invalid channel ordering")
            ErrInvalidCounterparty       = errorsmod.Register(SubModuleName, 7, "invalid counterparty channel")
            ErrInvalidChannelCapability  = errorsmod.Register(SubModuleName, 8, "invalid channel capability")
            ErrChannelCapabilityNotFound = errorsmod.Register(SubModuleName, 9, "channel capability not found")
            ErrSequenceSendNotFound      = errorsmod.Register(SubModuleName, 10, "sequence send not found")
            ErrSequenceReceiveNotFound   = errorsmod.Register(SubModuleName, 11, "sequence receive not found")
            ErrSequenceAckNotFound       = errorsmod.Register(SubModuleName, 12, "sequence acknowledgement not found")
            ErrInvalidPacket             = errorsmod.Register(SubModuleName, 13, "invalid packet")
            ErrPacketTimeout             = errorsmod.Register(SubModuleName, 14, "packet timeout")
            ErrTooManyConnectionHops     = errorsmod.Register(SubModuleName, 15, "too many connection hops")
            ErrInvalidAcknowledgement    = errorsmod.Register(SubModuleName, 16, "invalid acknowledgement")
            ErrAcknowledgementExists     = errorsmod.Register(SubModuleName, 17, "acknowledgement for packet already exists")
            ErrInvalidChannelIdentifier  = errorsmod.Register(SubModuleName, 18, "invalid channel identifier")

            // packets already relayed errors
            ErrPacketReceived           = errorsmod.Register(SubModuleName, 19, "packet already received")
            ErrPacketCommitmentNotFound = errorsmod.Register(SubModuleName, 20, "packet commitment not found") // may occur for already received acknowledgements or timeouts and in rare cases for packets never sent

            // ORDERED channel error
            ErrPacketSequenceOutOfOrder = errorsmod.Register(SubModuleName, 21, "packet sequence is out of order")

            // cspell:ignore Antehandler
            // Antehandler error
            ErrRedundantTx = errorsmod.Register(SubModuleName, 22, "packet messages are redundant")

            // Perform a no-op on the current Msg
            ErrNoOpMsg = errorsmod.Register(SubModuleName, 23, "message is redundant, no-op will be performed")

            ErrInvalidChannelVersion = errorsmod.Register(SubModuleName, 24, "invalid channel version")
            ErrPacketNotSent         = errorsmod.Register(SubModuleName, 25, "packet has not been sent")
            ErrInvalidTimeout        = errorsmod.Register(SubModuleName, 26, "invalid packet timeout")
        )
    }
}

pub trait AbciStateRead<Counterparty>: IbcPath<Union, Counterparty>
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output;
}

impl<Counterparty> AbciStateRead<Counterparty> for ClientStatePath<<Union as Chain>::ClientId>
where
    Counterparty: Chain,
    ClientStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ClientStateOf<Counterparty>>: Debug,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        Self::Output::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty>
    for ClientConsensusStatePath<<Union as Chain>::ClientId, <Counterparty as Chain>::Height>
where
    Counterparty: Chain,
    ConsensusStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ConsensusStateOf<Counterparty>>: Debug,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        Self::Output::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for ConnectionPath
where
    Counterparty: Chain,
    // <Counterparty as Chain>::ClientId: ClientId,
    // Self::Output: Proto + TryFrom<protos::ibc::core::connection::v1::ConnectionEnd>,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        Self::Output::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for ChannelEndPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        Self::Output::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for CommitmentPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        bytes.try_into().unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for AcknowledgementPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        bytes.try_into().unwrap()
    }
}

impl<Counterparty, P> IbcStateRead<Counterparty, P> for Union
where
    Counterparty: Chain,
    ClientStateOf<Counterparty>: TryFromProto,
    ConsensusStateOf<Counterparty>: TryFromProto,
    P: IbcPath<Union, Counterparty> + AbciStateRead<Counterparty> + 'static,
{
    fn proof(&self, path: P, at: Height) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            tracing::info!(%path, %at, "fetching state proof");

            let mut client =
                protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                    self.grpc_url.clone(),
                )
                .await
                .unwrap();

            let query_result = client
                .abci_query(
                    protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
                        data: path.to_string().into_bytes(),
                        path: "store/ibc/key".to_string(),
                        height: i64::try_from(at.revision_height).unwrap(),
                        prove: true,
                    },
                )
                .await
                .unwrap()
                .into_inner();

            unionlabs::cosmos::ics23::proof::MerkleProof::try_from(
                protos::ibc::core::commitment::v1::MerkleProof {
                    proofs: query_result
                        .proof_ops
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| {
                            protos::cosmos::ics23::v1::CommitmentProof::decode(op.data.as_slice())
                                .unwrap()
                        })
                        .collect::<Vec<_>>(),
                },
            )
            .unwrap()
            .into_eth_abi_bytes()
        }
    }

    fn state(
        &self,
        path: P,
        at: Self::Height,
    ) -> impl Future<Output = <P as IbcPath<Union, Counterparty>>::Output> + '_ {
        async move {
            let mut client =
                protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                    self.grpc_url.clone(),
                )
                .await
                .unwrap();

            let query_result = client
                .abci_query(
                    protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
                        data: path.to_string().into_bytes(),
                        path: "store/ibc/key".to_string(),
                        height: i64::try_from(at.revision_height).unwrap(),
                        prove: false,
                    },
                )
                .await
                .unwrap()
                .into_inner();

            P::from_abci_bytes(query_result.value)
        }
    }
}
