use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    marker::PhantomData,
};

use chain_utils::{
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain, CosmosSdkChainExt},
    union::Union,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use num_bigint::BigUint;
use protos::{
    ibc::core::connection::v1::MsgConnectionOpenInit,
    union::galois::api::v2::union_prover_api_client,
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
    hash::H256,
    ibc::{
        core::{
            client::{
                height::{Height, IsHeight},
                msg_update_client::MsgUpdateClient,
            },
            commitment::merkle_proof::MerkleProof,
        },
        lightclients::cometbls,
    },
    proof::ClientStatePath,
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            block_id::BlockId, canonical_block_header::CanonicalPartSetHeader,
            canonical_block_id::CanonicalBlockId, canonical_vote::CanonicalVote, commit::Commit,
            commit_sig::CommitSig, part_set_header::PartSetHeader, signed_header::SignedHeader,
            signed_msg_type::SignedMsgType, simple_validator::SimpleValidator,
        },
    },
    traits::{Chain, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
    union::galois::{
        poll_request::PollRequest,
        poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
        prove_request::ProveRequest,
        prove_response,
        validator_set_commit::ValidatorSetCommit,
    },
    IntoProto, Proto, TryFromProto, TryFromProtoErrorOf, TypeUrl,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
    chain_impls::cosmos_sdk::{
        fetch::{AbciQueryType, FetchAbciQuery},
        fetch_abci_query,
    },
    ctors::{aggregate, data, defer_relative, fetch, msg, wait},
    data::{AnyData, Data, IbcState, LightClientSpecificData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders, LightClientSpecificFetch},
    identified,
    msg::{
        AnyMsg, Msg, MsgConnectionOpenAckData, MsgConnectionOpenInitData, MsgConnectionOpenTryData,
        MsgUpdateClientData,
    },
    seq,
    use_aggregate::{do_aggregate, IsAggregateData, UseAggregate},
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayerMsg, Wasm, WasmConfig, Wraps,
};

impl ChainExt for Union {
    type Data<Tr: ChainExt> = UnionDataMsg<Tr>;
    type Fetch<Tr: ChainExt> = UnionFetch<Union, Tr>;
    type Aggregate<Tr: ChainExt> = UnionAggregateMsg<Union, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = ();
}

impl ChainExt for Wasm<Union> {
    type Data<Tr: ChainExt> = UnionDataMsg<Tr>;
    type Fetch<Tr: ChainExt> = UnionFetch<Wasm<Union>, Tr>;
    type Aggregate<Tr: ChainExt> = UnionAggregateMsg<Wasm<Union>, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = WasmConfig;
}

impl<Tr: ChainExt, Hc: Wraps<Self>> DoMsg<Hc, Tr> for Union
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
                let msg_any = match msg {
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

                self.broadcast_tx_commit(signer, [msg_any])
                    .await
                    .map(|_| ())
            })
            .await
    }
}

impl<Tr: ChainExt, Hc: Wraps<Self, StateProof = MerkleProof, Fetch<Tr> = UnionFetch<Hc, Tr>>>
    DoFetchState<Hc, Tr> for Union
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
                    // height: at.increment(),
                    height: at,
                    __marker: PhantomData,
                },
            )),
            fetch(Identified::<Hc, Tr, _>::new(
                hc.chain_id(),
                LightClientSpecificFetch(UnionFetch::AbciQuery(FetchAbciQuery {
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

impl<Tr: ChainExt, Hc: Wraps<Self, Fetch<Tr> = UnionFetch<Hc, Tr>>> DoFetchProof<Hc, Tr> for Union
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
{
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> RelayerMsg {
        seq([
            wait(Identified::new(
                hc.chain_id(),
                WaitForBlock {
                    // height: at.increment(),
                    height: at,
                    __marker: PhantomData,
                },
            )),
            fetch(Identified::<Hc, Tr, _>::new(
                hc.chain_id(),
                LightClientSpecificFetch(UnionFetch::AbciQuery(FetchAbciQuery::<Hc, Tr> {
                    path,
                    height: at,
                    ty: AbciQueryType::Proof,
                }))
                .into(),
            )),
        ])
    }
}

impl<Tr, Hc> DoFetchUpdateHeaders<Hc, Tr> for Union
where
    Tr: ChainExt,
    Hc: Wraps<Self, Fetch<Tr> = UnionFetch<Hc, Tr>, Aggregate<Tr> = UnionAggregateMsg<Hc, Tr>>,

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
                        LightClientSpecificFetch(UnionFetch::FetchUntrustedCommit(
                            FetchUntrustedCommit {
                                height: update_info.update_to.into(),
                            },
                        ))
                        .into(),
                    )),
                    fetch(Identified::<Hc, Tr, _>::new(
                        hc.chain_id(),
                        LightClientSpecificFetch(UnionFetch::FetchValidators(FetchValidators {
                            height: update_info.update_from.into(),
                        }))
                        .into(),
                    )),
                    fetch(Identified::<Hc, Tr, _>::new(
                        hc.chain_id(),
                        LightClientSpecificFetch(UnionFetch::FetchValidators(FetchValidators {
                            height: update_info.update_to.into(),
                        }))
                        .into(),
                    )),
                ],
                [],
                Identified::new(
                    hc.chain_id(),
                    LightClientSpecificAggregate(UnionAggregateMsg::AggregateProveRequest(
                        AggregateProveRequest { req: update_info },
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
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[allow(clippy::large_enum_variant)]
pub enum UnionDataMsg<Tr: ChainExt> {
    // NOTE: Not used currently?
    // TrustedCommit {
    //     height: Height,
    // },
    #[display(fmt = "UntrustedCommit")]
    UntrustedCommit(UntrustedCommit<Tr>),
    #[display(fmt = "Validators")]
    Validators(Validators<Tr>),
    #[display(fmt = "ProveResponse")]
    ProveResponse(ProveResponse<Tr>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[allow(clippy::large_enum_variant)]
pub enum UnionFetch<Hc: ChainExt, Tr: ChainExt> {
    // FetchTrustedCommit { height: Height },
    #[display(fmt = "FetchUntrustedCommit")]
    FetchUntrustedCommit(FetchUntrustedCommit),
    #[display(fmt = "FetchValidators")]
    FetchValidators(FetchValidators),
    #[display(fmt = "FetchProveRequest")]
    FetchProveRequest(FetchProveRequest),
    #[display(fmt = "FetchAbciQuery")]
    AbciQuery(FetchAbciQuery<Hc, Tr>),
}

impl<Hc, Tr> DoFetch<Hc> for UnionFetch<Hc, Tr>
where
    Hc: Wraps<Union>
        + CosmosSdkChain
        + ChainExt<
            StateProof = MerkleProof,
            Data<Tr> = UnionDataMsg<Tr>,
            Fetch<Tr> = UnionFetch<Hc, Tr>,
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
            UnionFetch::FetchUntrustedCommit(FetchUntrustedCommit { height }) => {
                let commit = hc
                    .tm_client()
                    .commit(
                        TryInto::<tendermint::block::Height>::try_into(height.revision_height)
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                let header_timestamp = tendermint_proto::google::protobuf::Timestamp::from(
                    commit.signed_header.header.time,
                );

                let signed_header = SignedHeader {
                    header: unionlabs::tendermint::types::header::Header {
                        version: unionlabs::tendermint::version::consensus::Consensus {
                            block: commit.signed_header.header.version.block,
                            app: commit.signed_header.header.version.app,
                        },
                        chain_id: commit.signed_header.header.chain_id.into(),
                        height: tendermint_height_to_bounded_i64(
                            commit.signed_header.header.height,
                        ),
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
                        data_hash: tendermint_hash_to_h256(
                            commit.signed_header.header.data_hash.unwrap(),
                        ),
                        validators_hash: tendermint_hash_to_h256(
                            commit.signed_header.header.validators_hash,
                        ),
                        next_validators_hash: tendermint_hash_to_h256(
                            commit.signed_header.header.next_validators_hash,
                        ),
                        consensus_hash: tendermint_hash_to_h256(
                            commit.signed_header.header.consensus_hash,
                        ),
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
                        height: tendermint_height_to_bounded_i64(
                            commit.signed_header.commit.height,
                        ),
                        round: i32::from(commit.signed_header.commit.round)
                            .try_into()
                            .unwrap(),
                        block_id: BlockId {
                            hash: tendermint_hash_to_h256(
                                commit.signed_header.commit.block_id.hash,
                            ),
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
                                tendermint::block::CommitSig::BlockIdFlagAbsent => {
                                    CommitSig::Absent
                                }
                                tendermint::block::CommitSig::BlockIdFlagCommit {
                                    validator_address,
                                    timestamp,
                                    signature,
                                } => CommitSig::Commit {
                                    validator_address: Vec::from(validator_address)
                                        .try_into()
                                        .unwrap(),
                                    timestamp: {
                                        let ts =
                                            tendermint_proto::google::protobuf::Timestamp::from(
                                                timestamp,
                                            );

                                        Timestamp {
                                            seconds: ts.seconds.try_into().unwrap(),
                                            nanos: ts.nanos.try_into().unwrap(),
                                        }
                                    },
                                    signature: signature.unwrap().into_bytes().try_into().unwrap(),
                                },
                                tendermint::block::CommitSig::BlockIdFlagNil {
                                    validator_address,
                                    timestamp,
                                    signature,
                                } => CommitSig::Nil {
                                    validator_address: Vec::from(validator_address)
                                        .try_into()
                                        .unwrap(),
                                    timestamp: {
                                        let ts =
                                            tendermint_proto::google::protobuf::Timestamp::from(
                                                timestamp,
                                            );

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
                };

                let msg = UnionDataMsg::UntrustedCommit(UntrustedCommit {
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
            UnionFetch::FetchValidators(FetchValidators { height }) => {
                let validators = hc
                    .tm_client()
                    .validators(
                        TryInto::<tendermint::block::Height>::try_into(height.revision_height)
                            .unwrap(),
                        tendermint_rpc::Paging::All,
                    )
                    .await
                    .unwrap()
                    .validators;

                let msg = UnionDataMsg::Validators(Validators {
                    height,
                    validators,
                    __marker: PhantomData,
                });

                data(Identified::<Hc, Tr, _>::new(
                    hc.chain_id(),
                    LightClientSpecificData(msg),
                ))
            }
            UnionFetch::FetchProveRequest(FetchProveRequest { request }) => {
                let response = union_prover_api_client::UnionProverApiClient::connect(
                    hc.inner().prover_endpoint.clone(),
                )
                .await
                .unwrap()
                .poll(
                    PollRequest {
                        request: request.clone(),
                    }
                    .into_proto(),
                )
                .await
                .map(|x| x.into_inner().try_into().unwrap());

                let retry = || {
                    seq([
                        // REVIEW: How long should we wait between polls?
                        defer_relative(3),
                        fetch(Identified::<Hc, Tr, _>::new(
                            hc.chain_id(),
                            LightClientSpecificFetch(UnionFetch::FetchProveRequest(
                                FetchProveRequest { request },
                            )),
                        )),
                    ])
                };

                match response {
                    Ok(PollResponse::Pending) => retry(),
                    Err(status) if status.message() == "busy_building" => retry(),
                    Err(err) => panic!("prove request failed: {:?}", err),
                    Ok(PollResponse::Failed(ProveRequestFailed { message })) => {
                        tracing::error!(%message, "prove request failed");
                        panic!()
                    }
                    Ok(PollResponse::Done(ProveRequestDone { response })) => {
                        data(Identified::<Hc, Tr, _>::new(
                            hc.chain_id(),
                            LightClientSpecificData(UnionDataMsg::ProveResponse(ProveResponse {
                                prove_response: response,
                                __marker: PhantomData,
                            })),
                        ))
                    }
                }
            }
            UnionFetch::AbciQuery(FetchAbciQuery { path, height, ty }) => {
                fetch_abci_query::<Hc, Tr>(hc, path, height, ty).await
            }
        }
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
pub enum UnionAggregateMsg<Hc: ChainExt, Tr: ChainExt> {
    #[display(fmt = "AggregateProveRequest")]
    AggregateProveRequest(AggregateProveRequest<Hc, Tr>),
    #[display(fmt = "AggregateHeader")]
    AggregateHeader(AggregateHeader<Hc, Tr>),
}

impl<Hc, Tr> DoAggregate for Identified<Hc, Tr, UnionAggregateMsg<Hc, Tr>>
where
    Tr: ChainExt,
    Hc: ChainExt,

    Identified<Hc, Tr, UntrustedCommit<Tr>>: IsAggregateData,
    Identified<Hc, Tr, Validators<Tr>>: IsAggregateData,
    Identified<Hc, Tr, ProveResponse<Tr>>: IsAggregateData,

    Identified<Hc, Tr, AggregateProveRequest<Hc, Tr>>: UseAggregate,
    Identified<Hc, Tr, AggregateHeader<Hc, Tr>>: UseAggregate,

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
            UnionAggregateMsg::AggregateProveRequest(data) => {
                do_aggregate(Identified::new(chain_id, data), aggregate_data)
            }
            UnionAggregateMsg::AggregateHeader(data) => {
                do_aggregate(Identified::new(chain_id, data), aggregate_data)
            }
        }
    }
}

const _: () = {
    try_from_relayer_msg! {
        chain = Union,
        generics = (Tr: ChainExt),
        msgs = UnionDataMsg(
            UntrustedCommit(UntrustedCommit<Tr>),
            Validators(Validators<Tr>),
            ProveResponse(ProveResponse<Tr>),
        ),
    }
};

const _: () = {
    try_from_relayer_msg! {
        chain = Wasm<Union>,
        generics = (Tr: ChainExt),
        msgs = UnionDataMsg(
            UntrustedCommit(UntrustedCommit<Tr>),
            Validators(Validators<Tr>),
            ProveResponse(ProveResponse<Tr>),
        ),
    }
};

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct UntrustedCommit<Tr: ChainExt> {
    pub height: Height,
    pub signed_header: SignedHeader,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct Validators<Tr: ChainExt> {
    pub height: Height,
    // TODO: Use non-`tendermint-rs` type here
    pub validators: Vec<tendermint::validator::Info>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct ProveResponse<Tr: ChainExt> {
    pub prove_response: prove_response::ProveResponse,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct FetchUntrustedCommit {
    pub height: Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
// TODO: Add Height param
pub struct FetchValidators {
    pub height: Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct FetchProveRequest {
    pub request: ProveRequest,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct AggregateHeader<Hc: ChainExt, Tr: ChainExt> {
    pub signed_header: SignedHeader,
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct AggregateProveRequest<Hc: ChainExt, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

fn tendermint_hash_to_h256(hash: tendermint::Hash) -> H256 {
    match hash {
        tendermint::Hash::Sha256(hash) => hash.into(),
        tendermint::Hash::None => panic!("empty hash???"),
    }
}

fn tendermint_height_to_bounded_i64(
    height: tendermint::block::Height,
) -> BoundedI64<0, { i64::MAX }> {
    i64::from(height).try_into().unwrap()
}

impl<Hc, Tr> UseAggregate for Identified<Hc, Tr, AggregateProveRequest<Hc, Tr>>
where
    Hc: ChainExt<Fetch<Tr> = UnionFetch<Hc, Tr>, Aggregate<Tr> = UnionAggregateMsg<Hc, Tr>>,
    Tr: ChainExt,

    Identified<Hc, Tr, UntrustedCommit<Tr>>: IsAggregateData,
    Identified<Hc, Tr, Validators<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, UntrustedCommit<Tr>>,
        Identified<Hc, Tr, Validators<Tr>>,
        Identified<Hc, Tr, Validators<Tr>>
    ];

    fn aggregate(
        Identified {
            chain_id,
            t: AggregateProveRequest { req },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: untrusted_commit_chain_id,
                t: UntrustedCommit {
                    height: untrusted_commit_height,
                    signed_header,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: trusted_validators_chain_id,
                t: Validators {
                    height: trusted_validators_height,
                    validators: trusted_validators,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: untrusted_validators_chain_id,
                t: Validators {
                    height: untrusted_validators_height,
                    validators: untrusted_validators,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(untrusted_commit_chain_id, untrusted_validators_chain_id);
        assert_eq!(chain_id, trusted_validators_chain_id);
        assert_eq!(chain_id, untrusted_validators_chain_id);

        assert_eq!(req.update_from, trusted_validators_height.into());
        assert_eq!(untrusted_commit_height, untrusted_validators_height);

        let make_validators_commit = |mut validators: Vec<tendermint::validator::Info>| {
            // Validators must be sorted to match the root, by token then address
            validators.sort_by(|a, b| {
                let a_power = a.power;
                let b_power = b.power;
                #[allow(clippy::collapsible_else_if)]
                if a_power == b_power {
                    let a_address = a.address;
                    let b_address = b.address;
                    if a_address < b_address {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                } else {
                    if a_power > b_power {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
            });

            // The bitmap is a public input of the circuit, it must fit in Fr (scalar field) bn254
            let mut bitmap = BigUint::default();
            // REVIEW: This will over-allocate for the trusted validators; should be benchmarked
            let mut signatures = Vec::<Vec<u8>>::with_capacity(validators.len());

            let validators_map = validators
                .iter()
                .enumerate()
                .map(|(i, v)| (v.address, i))
                .collect::<HashMap<_, _>>();

            // For each validator signature, we search for the actual validator
            // in the set and set it's signed bit to 1. We then push the
            // signature only if the validator signed. It's possible that we
            // don't find a validator for a given signature as the validator set
            // may have drifted (trusted validator set).
            for sig in signed_header.commit.signatures.iter() {
                match sig {
                    CommitSig::Absent => {
                        tracing::debug!("Validator did not sign: {:?}", sig);
                    }
                    CommitSig::Commit {
                        validator_address,
                        timestamp: _,
                        signature,
                    } => {
                        if let Some(validator_index) =
                            validators_map.get(&validator_address.0.to_vec().try_into().unwrap())
                        {
                            bitmap.set_bit(*validator_index as u64, true);
                            signatures.push(signature.clone().into());
                            tracing::debug!(
                                "Validator {:?} at index {} signed",
                                validator_address,
                                validator_index
                            );
                        } else {
                            tracing::warn!("Validator set drifted? Could not find validator for signature {:?}", validator_address);
                        }
                    }
                    CommitSig::Nil { .. } => {
                        tracing::warn!("Validator commit is nil: {:?}", sig);
                    }
                }
            }

            let simple_validators = validators
                .iter()
                .map(|v| {
                    let tendermint::PublicKey::Bn254(key) = v.pub_key else {
                        panic!("must be bn254")
                    };
                    SimpleValidator {
                        pub_key: PublicKey::Bn254(key.to_vec()),
                        voting_power: v.power.into(),
                    }
                })
                .collect::<Vec<_>>();

            ValidatorSetCommit {
                validators: simple_validators,
                signatures,
                bitmap: bitmap.to_bytes_be(),
            }
        };

        let trusted_validators_commit = make_validators_commit(trusted_validators);
        let untrusted_validators_commit = make_validators_commit(untrusted_validators);

        aggregate(
            [{
                fetch(Identified::<Hc, Tr, _>::new(
                    chain_id.clone(),
                    LightClientSpecificFetch(UnionFetch::FetchProveRequest(FetchProveRequest {
                        request: ProveRequest {
                            vote: CanonicalVote {
                                // REVIEW: Should this be hardcoded to precommit?
                                ty: SignedMsgType::Precommit,
                                height: signed_header.commit.height,
                                round: BoundedI64::new(signed_header.commit.round.inner().into())
                                    .expect("0..=i32::MAX can be converted to 0..=i64::MAX safely"),
                                block_id: CanonicalBlockId {
                                    hash: signed_header.commit.block_id.hash.clone(),
                                    part_set_header: CanonicalPartSetHeader {
                                        total: signed_header.commit.block_id.part_set_header.total,
                                        hash: signed_header
                                            .commit
                                            .block_id
                                            .part_set_header
                                            .hash
                                            .clone(),
                                    },
                                },
                                chain_id: signed_header.header.chain_id.clone(),
                            },
                            trusted_commit: trusted_validators_commit,
                            untrusted_commit: untrusted_validators_commit,
                        },
                    }))
                    .into(),
                ))
            }],
            [],
            Identified::new(
                chain_id,
                LightClientSpecificAggregate(UnionAggregateMsg::AggregateHeader(AggregateHeader {
                    signed_header,
                    req,
                })),
            ),
        )
    }
}

impl<Hc, Tr> UseAggregate for Identified<Hc, Tr, AggregateHeader<Hc, Tr>>
where
    Hc: ChainExt<Header = <Union as Chain>::Header>,
    Tr: ChainExt,

    Identified<Hc, Tr, ProveResponse<Tr>>: IsAggregateData,
    Identified<Hc, Tr, Validators<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, ProveResponse<Tr>>];

    fn aggregate(
        Identified {
            chain_id,
            t: AggregateHeader { signed_header, req },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: untrusted_commit_chain_id,
            t: ProveResponse {
                prove_response: response,
                __marker: _
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, untrusted_commit_chain_id);

        msg(Identified::<Tr, Hc, _>::new(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id.clone(),
                client_message: cometbls::header::Header {
                    signed_header,
                    trusted_height: req.update_from.into(),
                    zero_knowledge_proof: response.proof.evm_proof,
                },
            }),
        ))
    }
}
