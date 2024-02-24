use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

use chain_utils::{
    cosmos::Cosmos,
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain, CosmosSdkChainExt},
    wasm::Wraps,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use protos::ibc::core::connection::v1::MsgConnectionOpenInit;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, fetch, msg, wait,
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::Client;
use unionlabs::{
    bounded::BoundedI64,
    encoding::{Decode, Encode},
    google::protobuf::{
        any::{mk_any, Any},
        timestamp::Timestamp,
    },
    hash::{H160, H256},
    ibc::{
        core::{
            client::{
                height::{Height, IsHeight},
                msg_update_client::MsgUpdateClient,
            },
            commitment::merkle_proof::MerkleProof,
        },
        lightclients::tendermint,
    },
    proof::ClientStatePath,
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            block_id::BlockId, commit::Commit, commit_sig::CommitSig,
            part_set_header::PartSetHeader, signed_header::SignedHeader, validator::Validator,
        },
    },
    traits::{Chain, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
    IntoProto, Proto, TryFromProto, TryFromProtoErrorOf, TypeUrl,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
    chain_impls::cosmos_sdk::{
        fetch::{AbciQueryType, FetchAbciQuery},
        fetch_abci_query,
    },
    data::{AnyData, Data, IbcState, LightClientSpecificData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders, LightClientSpecificFetch},
    identified,
    msg::{
        AnyMsg, Msg, MsgConnectionOpenAckData, MsgConnectionOpenInitData, MsgConnectionOpenTryData,
        MsgUpdateClientData,
    },
    seq,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayerMsg, RelayerMsgTypes, Wasm, WasmConfig,
};

impl ChainExt for Cosmos {
    type Data<Tr: ChainExt> = CosmosDataMsg<Tr>;
    type Fetch<Tr: ChainExt> = CosmosFetch<Cosmos, Tr>;
    type Aggregate<Tr: ChainExt> = CosmosAggregateMsg<Cosmos, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = ();
}

impl ChainExt for Wasm<Cosmos> {
    type Data<Tr: ChainExt> = CosmosDataMsg<Tr>;
    type Fetch<Tr: ChainExt> = CosmosFetch<Wasm<Cosmos>, Tr>;
    type Aggregate<Tr: ChainExt> = CosmosAggregateMsg<Wasm<Cosmos>, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = WasmConfig;
}

impl<Tr: ChainExt, Hc: ChainExt + Wraps<Self>> DoMsg<Hc, Tr> for Cosmos
where
    ConsensusStateOf<Tr>: IntoProto,
    <ConsensusStateOf<Tr> as Proto>::Proto: TypeUrl,

    ClientStateOf<Tr>: IntoProto,
    <ClientStateOf<Tr> as Proto>::Proto: TypeUrl,

    HeaderOf<Tr>: IntoProto,
    <HeaderOf<Tr> as Proto>::Proto: TypeUrl,

    ConsensusStateOf<Hc>: IntoProto,
    <ConsensusStateOf<Hc> as Proto>::Proto: TypeUrl,

    ClientStateOf<Hc>: IntoProto,
    <ClientStateOf<Hc> as Proto>::Proto: TypeUrl,
    // HeaderOf<Hc>: IntoProto,
    // <HeaderOf<Hc> as Proto>::Proto: TypeUrl,
    Tr::StoredClientState<Hc>: IntoProto<Proto = protos::google::protobuf::Any>,
    Tr::StateProof: Encode<unionlabs::encoding::Proto>,
{
    async fn msg(&self, msg: Msg<Hc, Tr>) -> Result<(), BroadcastTxCommitError> {
        self.signers
            .with(|signer| async {
                let msg_any = match msg.clone() {
                    Msg::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => {
                        mk_any(&MsgConnectionOpenInit {
                            client_id: data.client_id.to_string(),
                            counterparty: Some(data.counterparty.into()),
                            version: Some(data.version.into()),
                            signer: signer.to_string(),
                            delay_period: data.delay_period,
                        })
                    }
                    Msg::ConnectionOpenTry(MsgConnectionOpenTryData(data)) =>
                    {
                        #[allow(deprecated)]
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenTry {
                            client_id: data.client_id.to_string(),
                            previous_connection_id: String::new(),
                            client_state: Some(data.client_state.into_proto()),
                            counterparty: Some(data.counterparty.into()),
                            delay_period: data.delay_period,
                            counterparty_versions: data
                                .counterparty_versions
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            proof_height: Some(data.proof_height.into_height().into()),
                            proof_init: data.proof_init.encode(),
                            proof_client: data.proof_client.encode(),
                            proof_consensus: data.proof_consensus.encode(),
                            consensus_height: Some(data.consensus_height.into_height().into()),
                            signer: signer.to_string(),
                            host_consensus_state_proof: vec![],
                        })
                    }
                    Msg::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenAck {
                            client_state: Some(data.client_state.into()),
                            proof_height: Some(data.proof_height.into_height().into()),
                            proof_client: data.proof_client.encode(),
                            proof_consensus: data.proof_consensus.encode(),
                            consensus_height: Some(data.consensus_height.into_height().into()),
                            signer: signer.to_string(),
                            host_consensus_state_proof: vec![],
                            connection_id: data.connection_id.to_string(),
                            counterparty_connection_id: data.counterparty_connection_id.to_string(),
                            version: Some(data.version.into()),
                            proof_try: data.proof_try.encode(),
                        })
                    }
                    Msg::ConnectionOpenConfirm(data) => mk_any(
                        &protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
                            connection_id: data.msg.connection_id.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        },
                    ),
                    Msg::ChannelOpenInit(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenInit {
                            port_id: data.msg.port_id.to_string(),
                            channel: Some(data.msg.channel.into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::ChannelOpenTry(data) =>
                    {
                        #[allow(deprecated)]
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenTry {
                            port_id: data.msg.port_id.to_string(),
                            channel: Some(data.msg.channel.into()),
                            counterparty_version: data.msg.counterparty_version,
                            proof_init: data.msg.proof_init.encode(),
                            proof_height: Some(data.msg.proof_height.into()),
                            previous_channel_id: String::new(),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::ChannelOpenAck(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenAck {
                            port_id: data.msg.port_id.to_string(),
                            channel_id: data.msg.channel_id.to_string(),
                            counterparty_version: data.msg.counterparty_version,
                            counterparty_channel_id: data.msg.counterparty_channel_id.to_string(),
                            proof_try: data.msg.proof_try.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::ChannelOpenConfirm(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
                            port_id: data.msg.port_id.to_string(),
                            channel_id: data.msg.channel_id.to_string(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                        })
                    }
                    Msg::RecvPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgRecvPacket {
                            packet: Some(data.msg.packet.into()),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_commitment: data.msg.proof_commitment.encode(),
                        })
                    }
                    Msg::AckPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgAcknowledgement {
                            packet: Some(data.msg.packet.into()),
                            acknowledgement: data.msg.acknowledgement,
                            proof_acked: data.msg.proof_acked.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::CreateClient(data) => {
                        mk_any(&protos::ibc::core::client::v1::MsgCreateClient {
                            client_state: Some(Any(data.msg.client_state).into()),
                            consensus_state: Some(Any(data.msg.consensus_state).into()),
                            signer: signer.to_string(),
                        })
                    }
                    Msg::UpdateClient(MsgUpdateClientData(data)) => {
                        mk_any(&protos::ibc::core::client::v1::MsgUpdateClient {
                            signer: signer.to_string(),
                            client_id: data.client_id.to_string(),
                            client_message: Some(Any(data.client_message).into()),
                        })
                    }
                };

                let tx_hash = self.broadcast_tx_commit(signer, [msg_any]).await?;

                tracing::info!("cosmos tx {:?} => {:?}", tx_hash, msg);

                Ok(())
            })
            .await
    }
}

impl<
        Tr: ChainExt,
        Hc: Wraps<Self> + ChainExt<StateProof = MerkleProof, Fetch<Tr> = CosmosFetch<Hc, Tr>>,
    > DoFetchState<Hc, Tr> for Cosmos
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    // required by fetch_abci_query, can be removed once that's been been removed
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    Tr::SelfClientState: Decode<unionlabs::encoding::Proto>,
    Tr::SelfConsensusState: Decode<unionlabs::encoding::Proto>,

    Hc::StoredClientState<Tr>: TryFromProto,
    Hc::StoredConsensusState<Tr>: TryFromProto,
    TryFromProtoErrorOf<Hc::StoredClientState<Tr>>: Debug,
    TryFromProtoErrorOf<Hc::StoredConsensusState<Tr>>: Debug,
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
{
    fn state(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> RelayerMsg {
        seq([
            wait(Identified::new(
                hc.chain_id(),
                WaitForBlock {
                    height: at,
                    __marker: PhantomData,
                },
            )),
            fetch(Identified::<Hc, Tr, _>::new(
                hc.chain_id(),
                LightClientSpecificFetch(CosmosFetch::AbciQuery(FetchAbciQuery {
                    path,
                    height: at,
                    ty: AbciQueryType::State,
                })),
            )),
        ])
    }

    async fn query_client_state(
        hc: &Hc,
        client_id: Hc::ClientId,
        height: Hc::Height,
    ) -> Hc::StoredClientState<Tr> {
        let RelayerMsg::Data(relayer_msg) = fetch_abci_query::<Hc, Tr>(
            hc,
            ClientStatePath { client_id }.into(),
            height,
            AbciQueryType::State,
        )
        .await
        else {
            panic!()
        };

        Identified::<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>::try_from(relayer_msg)
            .unwrap()
            .t
            .state
    }
}

impl<Tr: ChainExt, Hc: Wraps<Self> + ChainExt<Fetch<Tr> = CosmosFetch<Hc, Tr>>> DoFetchProof<Hc, Tr>
    for Cosmos
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
{
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> RelayerMsg {
        seq([
            wait(Identified::new(
                hc.chain_id(),
                WaitForBlock {
                    height: at,
                    __marker: PhantomData,
                },
            )),
            fetch(Identified::<Hc, Tr, _>::new(
                hc.chain_id(),
                LightClientSpecificFetch(CosmosFetch::AbciQuery(FetchAbciQuery::<Hc, Tr> {
                    path,
                    height: at,
                    ty: AbciQueryType::Proof,
                }))
                .into(),
            )),
        ])
    }
}

impl<Tr, Hc> DoFetchUpdateHeaders<Hc, Tr> for Cosmos
where
    Tr: ChainExt,
    Hc: Wraps<Self>
        + ChainExt<Fetch<Tr> = CosmosFetch<Hc, Tr>, Aggregate<Tr> = CosmosAggregateMsg<Hc, Tr>>,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    fn fetch_update_headers(hc: &Hc, update_info: FetchUpdateHeaders<Hc, Tr>) -> RelayerMsg {
        seq([
            wait(Identified::new(
                hc.chain_id(),
                WaitForBlock {
                    height: update_info.update_to,
                    __marker: PhantomData,
                },
            )),
            aggregate(
                [
                    fetch(Identified::<Hc, Tr, _>::new(
                        hc.chain_id(),
                        LightClientSpecificFetch(CosmosFetch::FetchTrustedCommit(
                            FetchTrustedCommit {
                                height: Into::<Height>::into(update_info.update_from).increment(),
                            },
                        ))
                        .into(),
                    )),
                    fetch(Identified::<Hc, Tr, _>::new(
                        hc.chain_id(),
                        LightClientSpecificFetch(CosmosFetch::FetchUntrustedCommit(
                            FetchUntrustedCommit {
                                height: update_info.update_to.into(),
                            },
                        ))
                        .into(),
                    )),
                    fetch(Identified::<Hc, Tr, _>::new(
                        hc.chain_id(),
                        LightClientSpecificFetch(CosmosFetch::FetchTrustedValidators(
                            FetchTrustedValidators {
                                height: Into::<Height>::into(update_info.update_from).increment(),
                            },
                        ))
                        .into(),
                    )),
                    fetch(Identified::<Hc, Tr, _>::new(
                        hc.chain_id(),
                        LightClientSpecificFetch(CosmosFetch::FetchUntrustedValidators(
                            FetchUntrustedValidators {
                                height: update_info.update_to.into(),
                            },
                        ))
                        .into(),
                    )),
                ],
                [],
                Identified::new(
                    hc.chain_id(),
                    LightClientSpecificAggregate(CosmosAggregateMsg::AggregateHeader(
                        AggregateHeader { req: update_info },
                    )),
                ),
            ),
        ])
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Tr: ChainExt")
)]
pub enum CosmosDataMsg<Tr: ChainExt> {
    #[display(fmt = "FetchUntrustedCommit")]
    TrustedCommit(TrustedCommit<Tr>),
    #[display(fmt = "UntrustedCommit")]
    UntrustedCommit(UntrustedCommit<Tr>),
    #[display(fmt = "TrustedValidators")]
    TrustedValidators(TrustedValidators<Tr>),
    #[display(fmt = "UntrustedValidators")]
    UntrustedValidators(UntrustedValidators<Tr>),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Tr: ChainExt")
)]
pub struct UntrustedCommit<Tr: ChainExt> {
    pub height: Height,
    pub signed_header: SignedHeader,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Tr: ChainExt")
)]
pub struct TrustedCommit<Tr: ChainExt> {
    pub height: Height,
    pub signed_header: SignedHeader,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Tr: ChainExt")
)]
pub struct TrustedValidators<Tr: ChainExt> {
    pub height: Height,
    pub validators: Vec<Validator>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Tr: ChainExt")
)]
pub struct UntrustedValidators<Tr: ChainExt> {
    pub height: Height,
    pub validators: Vec<Validator>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub enum CosmosFetch<Hc: ChainExt, Tr: ChainExt> {
    #[display(fmt = "FetchUntrustedCommit")]
    FetchTrustedCommit(FetchTrustedCommit),
    #[display(fmt = "FetchUntrustedCommit")]
    FetchUntrustedCommit(FetchUntrustedCommit),
    #[display(fmt = "FetchTrustedValidators")]
    FetchTrustedValidators(FetchTrustedValidators),
    #[display(fmt = "FetchUntrustedValidators")]
    FetchUntrustedValidators(FetchUntrustedValidators),
    #[display(fmt = "FetchAbciQuery")]
    AbciQuery(FetchAbciQuery<Hc, Tr>),
}

impl<Hc, Tr> DoFetch<Hc> for CosmosFetch<Hc, Tr>
where
    Hc: Wraps<Cosmos>
        + CosmosSdkChain
        + ChainExt<
            StateProof = MerkleProof,
            Data<Tr> = CosmosDataMsg<Tr>,
            Fetch<Tr> = CosmosFetch<Hc, Tr>,
        >,
    Tr: ChainExt,

    Hc::StoredClientState<Tr>: TryFromProto,
    Hc::StoredConsensusState<Tr>: TryFromProto,
    TryFromProtoErrorOf<Hc::StoredClientState<Tr>>: Debug,
    TryFromProtoErrorOf<Hc::StoredConsensusState<Tr>>: Debug,

    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,

    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
{
    async fn do_fetch(hc: &Hc, msg: Self) -> RelayerMsg {
        match msg {
            CosmosFetch::FetchTrustedCommit(FetchTrustedCommit { height }) => {
                let commit = hc
                    .tm_client()
                    .commit(
                        TryInto::<::tendermint::block::Height>::try_into(height.revision_height)
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                let signed_header = tendermint_commit_to_signed_header(commit);

                let msg = CosmosDataMsg::TrustedCommit(TrustedCommit {
                    height,
                    // REVIEW: Ensure `commit.canonical`?
                    signed_header,
                    __marker: PhantomData,
                });

                data(Identified::<Hc, Tr, _>::new(
                    hc.chain_id(),
                    LightClientSpecificData(msg),
                ))
            }
            CosmosFetch::FetchUntrustedCommit(FetchUntrustedCommit { height }) => {
                let commit = hc
                    .tm_client()
                    .commit(
                        TryInto::<::tendermint::block::Height>::try_into(height.revision_height)
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                let signed_header = tendermint_commit_to_signed_header(commit);

                let msg = CosmosDataMsg::UntrustedCommit(UntrustedCommit {
                    height,
                    // REVIEW: Ensure `commit.canonical`?
                    signed_header,
                    __marker: PhantomData,
                });

                data(Identified::<Hc, Tr, _>::new(
                    hc.chain_id(),
                    LightClientSpecificData(msg),
                ))
            }
            CosmosFetch::FetchTrustedValidators(FetchTrustedValidators { height }) => {
                let validators = hc
                    .tm_client()
                    .validators(
                        TryInto::<::tendermint::block::Height>::try_into(height.revision_height)
                            .unwrap(),
                        tendermint_rpc::Paging::All,
                    )
                    .await
                    .unwrap()
                    .validators
                    .into_iter()
                    .map(tendermint_validator_info_to_validator)
                    .collect();

                let msg = CosmosDataMsg::TrustedValidators(TrustedValidators {
                    height,
                    validators,
                    __marker: PhantomData,
                });

                data(Identified::<Hc, Tr, _>::new(
                    hc.chain_id(),
                    LightClientSpecificData(msg),
                ))
            }
            CosmosFetch::FetchUntrustedValidators(FetchUntrustedValidators { height }) => {
                let validators = hc
                    .tm_client()
                    .validators(
                        TryInto::<::tendermint::block::Height>::try_into(height.revision_height)
                            .unwrap(),
                        tendermint_rpc::Paging::All,
                    )
                    .await
                    .unwrap()
                    .validators
                    .into_iter()
                    .map(tendermint_validator_info_to_validator)
                    .collect();

                let msg = CosmosDataMsg::UntrustedValidators(UntrustedValidators {
                    height,
                    validators,
                    __marker: PhantomData,
                });

                data(Identified::<Hc, Tr, _>::new(
                    hc.chain_id(),
                    LightClientSpecificData(msg),
                ))
            }
            CosmosFetch::AbciQuery(FetchAbciQuery { path, height, ty }) => {
                fetch_abci_query::<Hc, Tr>(hc, path, height, ty).await
            }
        }
    }
}

fn tendermint_commit_to_signed_header(
    commit: tendermint_rpc::endpoint::commit::Response,
) -> SignedHeader {
    let header_timestamp =
        tendermint_proto::google::protobuf::Timestamp::from(commit.signed_header.header.time);

    SignedHeader {
        header: unionlabs::tendermint::types::header::Header {
            version: unionlabs::tendermint::version::consensus::Consensus {
                block: commit.signed_header.header.version.block,
                app: commit.signed_header.header.version.app,
            },
            chain_id: commit.signed_header.header.chain_id.into(),
            height: tendermint_height_to_bounded_i64(commit.signed_header.header.height),
            time: Timestamp {
                seconds: header_timestamp.seconds.try_into().unwrap(),
                nanos: header_timestamp.nanos.try_into().unwrap(),
            },
            last_block_id: BlockId {
                hash: tendermint_hash_to_h256(
                    commit.signed_header.header.last_block_id.unwrap().hash,
                ),
                part_set_header: PartSetHeader {
                    total: commit
                        .signed_header
                        .header
                        .last_block_id
                        .unwrap()
                        .part_set_header
                        .total,
                    hash: tendermint_hash_to_h256(
                        commit
                            .signed_header
                            .header
                            .last_block_id
                            .unwrap()
                            .part_set_header
                            .hash,
                    ),
                },
            },
            last_commit_hash: tendermint_hash_to_h256(
                commit.signed_header.header.last_commit_hash.unwrap(),
            ),
            data_hash: tendermint_hash_to_h256(commit.signed_header.header.data_hash.unwrap()),
            validators_hash: tendermint_hash_to_h256(commit.signed_header.header.validators_hash),
            next_validators_hash: tendermint_hash_to_h256(
                commit.signed_header.header.next_validators_hash,
            ),
            consensus_hash: tendermint_hash_to_h256(commit.signed_header.header.consensus_hash),
            app_hash: commit
                .signed_header
                .header
                .app_hash
                .as_bytes()
                .try_into()
                .unwrap(),
            last_results_hash: tendermint_hash_to_h256(
                commit.signed_header.header.last_results_hash.unwrap(),
            ),
            evidence_hash: tendermint_hash_to_h256(
                commit.signed_header.header.evidence_hash.unwrap(),
            ),
            proposer_address: commit
                .signed_header
                .header
                .proposer_address
                .as_bytes()
                .try_into()
                .expect("value is a [u8; 20] internally, this should not fail; qed;"),
        },
        commit: Commit {
            height: tendermint_height_to_bounded_i64(commit.signed_header.commit.height),
            round: i32::from(commit.signed_header.commit.round)
                .try_into()
                .unwrap(),
            block_id: BlockId {
                hash: tendermint_hash_to_h256(commit.signed_header.commit.block_id.hash),
                part_set_header: PartSetHeader {
                    total: commit.signed_header.commit.block_id.part_set_header.total,
                    hash: tendermint_hash_to_h256(
                        commit.signed_header.commit.block_id.part_set_header.hash,
                    ),
                },
            },
            signatures: commit
                .signed_header
                .commit
                .signatures
                .into_iter()
                .map(|sig| match sig {
                    ::tendermint::block::CommitSig::BlockIdFlagAbsent => CommitSig::Absent,
                    ::tendermint::block::CommitSig::BlockIdFlagCommit {
                        validator_address,
                        timestamp,
                        signature,
                    } => CommitSig::Commit {
                        validator_address: Vec::from(validator_address).try_into().unwrap(),
                        timestamp: {
                            let ts = tendermint_proto::google::protobuf::Timestamp::from(timestamp);

                            Timestamp {
                                seconds: ts.seconds.try_into().unwrap(),
                                nanos: ts.nanos.try_into().unwrap(),
                            }
                        },
                        signature: signature.unwrap().into_bytes().try_into().unwrap(),
                    },
                    ::tendermint::block::CommitSig::BlockIdFlagNil {
                        validator_address,
                        timestamp,
                        signature,
                    } => CommitSig::Nil {
                        validator_address: Vec::from(validator_address).try_into().unwrap(),
                        timestamp: {
                            let ts = tendermint_proto::google::protobuf::Timestamp::from(timestamp);

                            Timestamp {
                                seconds: ts.seconds.try_into().unwrap(),
                                nanos: ts.nanos.try_into().unwrap(),
                            }
                        },
                        signature: signature.unwrap().into_bytes().try_into().unwrap(),
                    },
                })
                .collect(),
        },
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub enum CosmosAggregateMsg<Hc: ChainExt, Tr: ChainExt> {
    #[display(fmt = "AggregateHeader")]
    AggregateHeader(AggregateHeader<Hc, Tr>),
}

impl<Hc, Tr> DoAggregate for Identified<Hc, Tr, CosmosAggregateMsg<Hc, Tr>>
where
    Tr: ChainExt,
    Hc: ChainExt,

    Identified<Hc, Tr, TrustedCommit<Tr>>: IsAggregateData,
    Identified<Hc, Tr, UntrustedCommit<Tr>>: IsAggregateData,
    Identified<Hc, Tr, TrustedValidators<Tr>>: IsAggregateData,
    Identified<Hc, Tr, UntrustedValidators<Tr>>: IsAggregateData,

    Identified<Hc, Tr, AggregateHeader<Hc, Tr>>: UseAggregate<RelayerMsgTypes>,

    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t: data,
            __marker: _,
        }: Self,
        aggregate_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> RelayerMsg {
        match data {
            CosmosAggregateMsg::AggregateHeader(data) => {
                do_aggregate(Identified::new(chain_id, data), aggregate_data)
            }
        }
    }
}

const _: () = {
    try_from_relayer_msg! {
        chain = Cosmos,
        generics = (Tr: ChainExt),
        msgs = CosmosDataMsg(
            TrustedCommit(TrustedCommit<Tr>),
            UntrustedCommit(UntrustedCommit<Tr>),
            TrustedValidators(TrustedValidators<Tr>),
            UntrustedValidators(UntrustedValidators<Tr>),
        ),
    }
};

const _: () = {
    try_from_relayer_msg! {
        chain = Wasm<Cosmos>,
        generics = (Tr: ChainExt),
        msgs = CosmosDataMsg(
            TrustedCommit(TrustedCommit<Tr>),
            UntrustedCommit(UntrustedCommit<Tr>),
            TrustedValidators(TrustedValidators<Tr>),
            UntrustedValidators(UntrustedValidators<Tr>),
        ),
    }
};

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct FetchTrustedCommit {
    pub height: Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct FetchUntrustedCommit {
    pub height: Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct FetchTrustedValidators {
    pub height: Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct FetchUntrustedValidators {
    pub height: Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct AggregateHeader<Hc: ChainExt, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

fn tendermint_hash_to_h256(hash: ::tendermint::Hash) -> H256 {
    match hash {
        ::tendermint::Hash::Sha256(hash) => hash.into(),
        ::tendermint::Hash::None => panic!("empty hash???"),
    }
}

fn tendermint_height_to_bounded_i64(
    height: ::tendermint::block::Height,
) -> BoundedI64<0, { i64::MAX }> {
    i64::from(height).try_into().unwrap()
}

impl<Hc, Tr> UseAggregate<RelayerMsgTypes> for Identified<Hc, Tr, AggregateHeader<Hc, Tr>>
where
    Hc: ChainExt<Header = <Cosmos as Chain>::Header>,
    Tr: ChainExt,

    Identified<Hc, Tr, TrustedCommit<Tr>>: IsAggregateData,
    Identified<Hc, Tr, UntrustedCommit<Tr>>: IsAggregateData,
    Identified<Hc, Tr, TrustedValidators<Tr>>: IsAggregateData,
    Identified<Hc, Tr, UntrustedValidators<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, TrustedCommit<Tr>>,
        Identified<Hc, Tr, UntrustedCommit<Tr>>,
        Identified<Hc, Tr, TrustedValidators<Tr>>,
        Identified<Hc, Tr, UntrustedValidators<Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            t: AggregateHeader { req },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: _trusted_commit_chain_id,
                t: TrustedCommit {
                    height: _trusted_commit_height,
                    signed_header: trusted_signed_header,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: untrusted_commit_chain_id,
                t: UntrustedCommit {
                    height: _untrusted_commit_height,
                    signed_header: untrusted_signed_header,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: _trusted_validators_chain_id,
                t: TrustedValidators {
                    height: _trusted_validators_height,
                    validators: trusted_validators,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: _untrusted_validators_chain_id,
                t: UntrustedValidators {
                    height: _untrusted_validators_height,
                    validators: untrusted_validators,
                    __marker: _
                },
                __marker: _,
            }
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, untrusted_commit_chain_id);

        let trusted_valset = mk_valset(
            trusted_validators,
            trusted_signed_header.header.proposer_address,
        );

        let untrusted_valset = mk_valset(
            untrusted_validators,
            untrusted_signed_header.header.proposer_address.clone(),
        );

        msg(Identified::<Tr, Hc, _>::new(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id.clone(),
                client_message: tendermint::header::Header {
                    signed_header: untrusted_signed_header,
                    trusted_height: req.update_from.into(),
                    validator_set: untrusted_valset,
                    trusted_validators: trusted_valset,
                },
            }),
        ))
    }
}

fn mk_valset(
    validators: Vec<Validator>,
    proposer_address: H160,
) -> unionlabs::tendermint::types::validator_set::ValidatorSet {
    let proposer = validators
        .iter()
        .find(|val| val.address == proposer_address)
        .unwrap()
        .clone();

    let total_voting_power = validators
        .iter()
        .map(|v| v.voting_power.inner())
        .sum::<i64>();

    unionlabs::tendermint::types::validator_set::ValidatorSet {
        validators,
        proposer,
        total_voting_power,
    }
}

fn tendermint_validator_info_to_validator(val: ::tendermint::validator::Info) -> Validator {
    Validator {
        address: val
            .address
            .as_bytes()
            .try_into()
            .expect("value is 20 bytes internally; should not fail; qed"),
        pub_key: match val.pub_key {
            ::tendermint::PublicKey::Ed25519(key) => PublicKey::Ed25519(key.as_bytes().into()),
            ::tendermint::PublicKey::Bn254(key) => PublicKey::Bn254(key.to_vec()),
            _ => todo!(),
        },
        voting_power: BoundedI64::new(val.power.value().try_into().unwrap()).unwrap(),
        proposer_priority: val.proposer_priority.value(),
    }
}
