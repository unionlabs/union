use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
};

use chain_utils::{
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain, CosmosSdkChainExt},
    union::Union,
    wasm::Wraps,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use macros::apply;
use num_bigint::BigUint;
use protos::{
    ibc::core::connection::v1::MsgConnectionOpenInit,
    union::galois::api::v3::union_prover_api_client,
};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, defer_relative, effect, fetch, msg_struct, wait, QueueMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    cometbls::types::canonical_vote::CanonicalVote,
    encoding::{Decode, Encode, Proto},
    google::protobuf::any::{mk_any, Any},
    ibc::{
        core::client::{height::IsHeight, msg_update_client::MsgUpdateClient},
        lightclients::cometbls,
    },
    proof::ClientStatePath,
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            canonical_block_header::CanonicalPartSetHeader, canonical_block_id::CanonicalBlockId,
            commit_sig::CommitSig, signed_header::SignedHeader, signed_msg_type::SignedMsgType,
            simple_validator::SimpleValidator,
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
    TypeUrl,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::cosmos_sdk::{
        data::{TrustedValidators, UntrustedCommit, UntrustedValidators},
        fetch::{
            fetch_trusted_validators, fetch_untrusted_commit, fetch_untrusted_validators,
            AbciQueryType, FetchAbciQuery, FetchTrustedValidators, FetchUntrustedCommit,
            FetchUntrustedValidators,
        },
        fetch_abci_query,
    },
    data::{AnyData, Data, IbcState},
    effect::{
        AnyEffect, Effect, MsgConnectionOpenAckData, MsgConnectionOpenInitData,
        MsgConnectionOpenTryData, MsgUpdateClientData,
    },
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified, seq,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayerMsgTypes, Wasm, WasmConfig,
};

impl ChainExt for Union {
    type Data<Tr: ChainExt> = UnionDataMsg<Union, Tr>;
    type Fetch<Tr: ChainExt> = UnionFetch<Union, Tr>;
    type Aggregate<Tr: ChainExt> = UnionAggregateMsg<Union, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = ();
}

impl ChainExt for Wasm<Union> {
    type Data<Tr: ChainExt> = UnionDataMsg<Wasm<Union>, Tr>;
    type Fetch<Tr: ChainExt> = UnionFetch<Wasm<Union>, Tr>;
    type Aggregate<Tr: ChainExt> = UnionAggregateMsg<Wasm<Union>, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = WasmConfig;
}

// TODO: Deduplicate this implementation between union and cosmos, its literally just a copy-paste right now
impl<Tr: ChainExt, Hc: ChainExt + Wraps<Self>> DoMsg<Hc, Tr> for Union
where
    ConsensusStateOf<Tr>: Encode<Proto> + TypeUrl,
    ClientStateOf<Tr>: Encode<Proto> + TypeUrl,
    HeaderOf<Tr>: Encode<Proto> + TypeUrl,

    ConsensusStateOf<Hc>: Encode<Proto> + TypeUrl,

    ClientStateOf<Hc>: Encode<Proto> + TypeUrl,
    // HeaderOf<Hc>: IntoProto,
    // <HeaderOf<Hc> as Proto>::Proto: TypeUrl,
    Tr::StoredClientState<Hc>: Into<protos::google::protobuf::Any>,
    Tr::StateProof: Encode<Proto>,
{
    async fn msg(&self, msg: Effect<Hc, Tr>) -> Result<(), BroadcastTxCommitError> {
        self.signers
            .with(|signer| async {
                let msg_any = match msg.clone() {
                    Effect::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => {
                        mk_any(&MsgConnectionOpenInit {
                            client_id: data.client_id.to_string(),
                            counterparty: Some(data.counterparty.into()),
                            version: Some(data.version.into()),
                            signer: signer.to_string(),
                            delay_period: data.delay_period,
                        })
                    }
                    Effect::ConnectionOpenTry(MsgConnectionOpenTryData(data)) =>
                    {
                        #[allow(deprecated)]
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenTry {
                            client_id: data.client_id.to_string(),
                            previous_connection_id: String::new(),
                            client_state: Some(data.client_state.into()),
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
                    Effect::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => {
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
                    Effect::ConnectionOpenConfirm(data) => mk_any(
                        &protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
                            connection_id: data.msg.connection_id.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        },
                    ),
                    Effect::ChannelOpenInit(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenInit {
                            port_id: data.msg.port_id.to_string(),
                            channel: Some(data.msg.channel.into()),
                            signer: signer.to_string(),
                        })
                    }
                    Effect::ChannelOpenTry(data) =>
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
                    Effect::ChannelOpenAck(data) => {
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
                    Effect::ChannelOpenConfirm(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
                            port_id: data.msg.port_id.to_string(),
                            channel_id: data.msg.channel_id.to_string(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_ack: data.msg.proof_ack.encode(),
                        })
                    }
                    Effect::RecvPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgRecvPacket {
                            packet: Some(data.msg.packet.into()),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                            proof_commitment: data.msg.proof_commitment.encode(),
                        })
                    }
                    Effect::AckPacket(data) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgAcknowledgement {
                            packet: Some(data.msg.packet.into()),
                            acknowledgement: data.msg.acknowledgement,
                            proof_acked: data.msg.proof_acked.encode(),
                            proof_height: Some(data.msg.proof_height.into_height().into()),
                            signer: signer.to_string(),
                        })
                    }
                    Effect::CreateClient(data) => {
                        mk_any(&protos::ibc::core::client::v1::MsgCreateClient {
                            client_state: Some(Any(data.msg.client_state).into()),
                            consensus_state: Some(Any(data.msg.consensus_state).into()),
                            signer: signer.to_string(),
                        })
                    }
                    Effect::UpdateClient(MsgUpdateClientData(data)) => {
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

impl<Tr, Hc> DoFetchState<Hc, Tr> for Union
where
    Tr: ChainExt,
    Hc: ChainExt<
            StateProof = unionlabs::union::ics23::merkle_proof::MerkleProof,
            Fetch<Tr> = UnionFetch<Hc, Tr>,
        > + Wraps<Self>,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    // required by fetch_abci_query, can be removed once that's been been removed
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    Tr::SelfClientState: Decode<Proto>,
    Tr::SelfConsensusState: Decode<Proto>,

    Hc::StoredClientState<Tr>: Decode<Proto>,
    Hc::StoredConsensusState<Tr>: Decode<Proto>,

    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
{
    fn state(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> QueueMsg<RelayerMsgTypes> {
        seq([
            wait(id(
                hc.chain_id(),
                WaitForBlock {
                    // height: at.increment(),
                    height: at,
                    __marker: PhantomData,
                },
            )),
            fetch(id::<Hc, Tr, _>(
                hc.chain_id(),
                Fetch::specific(FetchAbciQuery {
                    path,
                    height: at,
                    ty: AbciQueryType::State,
                }),
            )),
        ])
    }

    async fn query_client_state(
        hc: &Hc,
        client_id: Hc::ClientId,
        height: Hc::Height,
    ) -> Hc::StoredClientState<Tr> {
        let QueueMsg::Data(relayer_msg) = fetch_abci_query::<Hc, Tr>(
            hc,
            ClientStatePath { client_id }.into(),
            height,
            AbciQueryType::State,
        )
        .await
        else {
            panic!()
        };

        Identified::<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>::try_from(relayer_msg)
            .unwrap()
            .t
            .state
    }
}

impl<Tr: ChainExt, Hc: ChainExt<Fetch<Tr> = UnionFetch<Hc, Tr>> + Wraps<Self>> DoFetchProof<Hc, Tr>
    for Union
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
{
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> QueueMsg<RelayerMsgTypes> {
        seq([
            wait(id(
                hc.chain_id(),
                WaitForBlock {
                    // height: at.increment(),
                    height: at,
                    __marker: PhantomData,
                },
            )),
            fetch(id::<Hc, Tr, _>(
                hc.chain_id(),
                Fetch::specific(FetchAbciQuery::<Hc, Tr> {
                    path,
                    height: at,
                    ty: AbciQueryType::Proof,
                }),
            )),
        ])
    }
}

impl<Tr, Hc> DoFetchUpdateHeaders<Hc, Tr> for Union
where
    Tr: ChainExt,
    Hc: ChainExt<Fetch<Tr> = UnionFetch<Hc, Tr>, Aggregate<Tr> = UnionAggregateMsg<Hc, Tr>>
        + Wraps<Self>,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    fn fetch_update_headers(
        hc: &Hc,
        update_info: FetchUpdateHeaders<Hc, Tr>,
    ) -> QueueMsg<RelayerMsgTypes> {
        seq([
            wait(id(
                hc.chain_id(),
                WaitForBlock {
                    height: update_info.update_to,
                    __marker: PhantomData,
                },
            )),
            aggregate(
                [
                    fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        Fetch::specific(FetchUntrustedCommit {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        Fetch::specific(FetchUntrustedValidators {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        Fetch::specific(FetchTrustedValidators {
                            height: update_info.update_from,
                            __marker: PhantomData,
                        }),
                    )),
                ],
                [],
                id(
                    hc.chain_id(),
                    Aggregate::specific(AggregateProveRequest { req: update_info }),
                ),
            ),
        ])
    }
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    enumorph::Enumorph,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Tr: ChainExt")
)]
pub enum UnionDataMsg<Hc: ChainExt, Tr: ChainExt> {
    #[display(fmt = "UntrustedCommit")]
    UntrustedCommit(UntrustedCommit<Hc, Tr>),
    #[display(fmt = "TrustedValidators")]
    TrustedValidators(TrustedValidators<Hc, Tr>),
    #[display(fmt = "UntrustedValidators")]
    UntrustedValidators(UntrustedValidators<Hc, Tr>),
    #[display(fmt = "ProveResponse")]
    ProveResponse(ProveResponse<Tr>),
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    enumorph::Enumorph,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub enum UnionFetch<Hc: ChainExt, Tr: ChainExt> {
    // FetchTrustedCommit { height: Height },
    #[display(fmt = "FetchUntrustedCommit")]
    FetchUntrustedCommit(FetchUntrustedCommit<Hc, Tr>),
    #[display(fmt = "FetchTrustedValidators")]
    FetchTrustedValidators(FetchTrustedValidators<Hc, Tr>),
    #[display(fmt = "FetchUntrustedValidators")]
    FetchUntrustedValidators(FetchUntrustedValidators<Hc, Tr>),
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
            StateProof = unionlabs::union::ics23::merkle_proof::MerkleProof,
            Data<Tr> = UnionDataMsg<Hc, Tr>,
            Fetch<Tr> = UnionFetch<Hc, Tr>,
        >,
    Tr: ChainExt,

    Hc::StoredClientState<Tr>: Decode<Proto>,
    Hc::StoredConsensusState<Tr>: Decode<Proto>,

    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,

    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
{
    async fn do_fetch(hc: &Hc, msg: Self) -> QueueMsg<RelayerMsgTypes> {
        match msg {
            Self::FetchUntrustedCommit(FetchUntrustedCommit {
                height,
                __marker: _,
            }) => fetch_untrusted_commit(hc, height).await,
            Self::FetchTrustedValidators(FetchTrustedValidators {
                height,
                __marker: _,
            }) => fetch_trusted_validators(hc, height).await,
            Self::FetchUntrustedValidators(FetchUntrustedValidators {
                height,
                __marker: _,
            }) => fetch_untrusted_validators(hc, height).await,
            Self::FetchProveRequest(FetchProveRequest { request }) => {
                let response = union_prover_api_client::UnionProverApiClient::connect(
                    hc.inner().prover_endpoint.clone(),
                )
                .await
                .unwrap()
                .poll(protos::union::galois::api::v3::PollRequest::from(
                    PollRequest {
                        request: request.clone(),
                    },
                ))
                .await
                .map(|x| x.into_inner().try_into().unwrap());

                let retry = || {
                    seq([
                        // REVIEW: How long should we wait between polls?
                        defer_relative(3),
                        fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            Fetch::specific(FetchProveRequest { request }),
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
                    Ok(PollResponse::Done(ProveRequestDone { response })) => data(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        Data::specific(ProveResponse {
                            prove_response: response,
                            __marker: PhantomData,
                        }),
                    )),
                }
            }
            Self::AbciQuery(FetchAbciQuery { path, height, ty }) => {
                fetch_abci_query::<Hc, Tr>(hc, path, height, ty).await
            }
        }
    }
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    enumorph::Enumorph,
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

    identified!(UntrustedCommit<Hc, Tr>): IsAggregateData,
    identified!(TrustedValidators<Hc, Tr>): IsAggregateData,
    identified!(UntrustedValidators<Hc, Tr>): IsAggregateData,

    Identified<Hc, Tr, ProveResponse<Tr>>: IsAggregateData,

    identified!(AggregateProveRequest<Hc, Tr>): UseAggregate<RelayerMsgTypes>,
    identified!(AggregateHeader<Hc, Tr>): UseAggregate<RelayerMsgTypes>,

    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t: data,
            __marker: _,
        }: Self,
        aggregate_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayerMsgTypes> {
        match data {
            UnionAggregateMsg::AggregateProveRequest(data) => {
                do_aggregate(id(chain_id, data), aggregate_data)
            }
            UnionAggregateMsg::AggregateHeader(data) => {
                do_aggregate(id(chain_id, data), aggregate_data)
            }
        }
    }
}

const _: () = {
    try_from_relayer_msg! {
        chain = Union,
        generics = (Tr: ChainExt),
        msgs = UnionDataMsg(
            UntrustedCommit(UntrustedCommit<Union, Tr>),
            TrustedValidators(TrustedValidators<Union, Tr>),
            UntrustedValidators(UntrustedValidators<Union, Tr>),
            ProveResponse(ProveResponse<Tr>),
        ),
    }
};

const _: () = {
    try_from_relayer_msg! {
        chain = Wasm<Union>,
        generics = (Tr: ChainExt),
        msgs = UnionDataMsg(
            UntrustedCommit(UntrustedCommit<Wasm<Union>, Tr>),
            TrustedValidators(TrustedValidators<Wasm<Union>, Tr>),
            UntrustedValidators(UntrustedValidators<Wasm<Union>, Tr>),
            ProveResponse(ProveResponse<Tr>),
        ),
    }
};

#[apply(msg_struct)]
#[cover(Tr)]
pub struct ProveResponse<Tr: ChainExt> {
    pub prove_response: prove_response::ProveResponse,
}

#[apply(msg_struct)]
pub struct FetchProveRequest {
    pub request: ProveRequest,
}

#[apply(msg_struct)]
pub struct AggregateHeader<Hc: ChainExt, Tr: ChainExt> {
    pub signed_header: SignedHeader,
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

#[apply(msg_struct)]
pub struct AggregateProveRequest<Hc: ChainExt, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

impl<Hc, Tr> UseAggregate<RelayerMsgTypes> for Identified<Hc, Tr, AggregateProveRequest<Hc, Tr>>
where
    Hc: ChainExt<Fetch<Tr> = UnionFetch<Hc, Tr>, Aggregate<Tr> = UnionAggregateMsg<Hc, Tr>>,
    Tr: ChainExt,

    identified!(UntrustedCommit<Hc, Tr>): IsAggregateData,
    identified!(TrustedValidators<Hc, Tr>): IsAggregateData,
    identified!(UntrustedValidators<Hc, Tr>): IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![
        identified!(UntrustedCommit<Hc, Tr>),
        identified!(TrustedValidators<Hc, Tr>),
        identified!(UntrustedValidators<Hc, Tr>),
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
                t: TrustedValidators {
                    height: trusted_validators_height,
                    validators: trusted_validators,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: untrusted_validators_chain_id,
                t: UntrustedValidators {
                    height: untrusted_validators_height,
                    validators: untrusted_validators,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayerMsgTypes> {
        assert_eq!(untrusted_commit_chain_id, untrusted_validators_chain_id);
        assert_eq!(chain_id, trusted_validators_chain_id);
        assert_eq!(chain_id, untrusted_validators_chain_id);

        assert_eq!(req.update_from, trusted_validators_height);
        assert_eq!(untrusted_commit_height, untrusted_validators_height);

        let make_validators_commit = |mut validators: Vec<
            unionlabs::tendermint::types::validator::Validator,
        >| {
            // Validators must be sorted to match the root, by token then address
            validators.sort_by(|a, b| {
                // TODO: Double check how these comparisons are supposed to work
                #[allow(clippy::collapsible_else_if)]
                if a.voting_power == b.voting_power {
                    if a.address < b.address {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                } else {
                    if a.voting_power > b.voting_power {
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
                            signatures.push((*signature).into());
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
                    let PublicKey::Bn254(ref key) = v.pub_key else {
                        panic!("must be bn254")
                    };
                    SimpleValidator {
                        pub_key: PublicKey::Bn254(key.to_vec()),
                        voting_power: v.voting_power.into(),
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
            [fetch(id::<Hc, Tr, _>(
                chain_id.clone(),
                Fetch::specific(FetchProveRequest {
                    request: ProveRequest {
                        vote: CanonicalVote {
                            // REVIEW: Should this be hardcoded to precommit?
                            ty: SignedMsgType::Precommit,
                            height: signed_header.commit.height,
                            round: BoundedI64::new(signed_header.commit.round.inner().into())
                                .expect("0..=i32::MAX can be converted to 0..=i64::MAX safely"),
                            block_id: CanonicalBlockId {
                                hash: signed_header.commit.block_id.hash,
                                part_set_header: CanonicalPartSetHeader {
                                    total: signed_header.commit.block_id.part_set_header.total,
                                    hash: signed_header.commit.block_id.part_set_header.hash,
                                },
                            },
                            chain_id: signed_header.header.chain_id.clone(),
                        },
                        untrusted_header: signed_header.header.clone(),
                        trusted_commit: trusted_validators_commit,
                        untrusted_commit: untrusted_validators_commit,
                    },
                }),
            ))],
            [],
            id(
                chain_id,
                Aggregate::specific(AggregateHeader { signed_header, req }),
            ),
        )
    }
}

impl<Hc, Tr> UseAggregate<RelayerMsgTypes> for Identified<Hc, Tr, AggregateHeader<Hc, Tr>>
where
    Hc: ChainExt<Header = <Union as Chain>::Header>,
    Tr: ChainExt,

    Identified<Hc, Tr, ProveResponse<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, ProveResponse<Tr>>];

    fn aggregate(
        Identified {
            chain_id,
            t:
                AggregateHeader {
                    mut signed_header,
                    req,
                },
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
    ) -> QueueMsg<RelayerMsgTypes> {
        assert_eq!(chain_id, untrusted_commit_chain_id);

        // TODO: maybe introduce a new commit for union signed header as we don't need the signatures but the ZKP only
        // Keeping this signatures significantly increase the size of the structure and the associated gas cost in EVM (calldata).
        signed_header.commit.signatures.clear();

        effect(id::<Tr, Hc, _>(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id.clone(),
                client_message: cometbls::header::Header {
                    signed_header: signed_header.into(),
                    trusted_height: req.update_from.into(),
                    zero_knowledge_proof: response.proof.evm_proof,
                },
            }),
        ))
    }
}
