use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    marker::PhantomData,
};

use chain_utils::{
    evm::Evm,
    union::{BroadcastTxCommitError, Union},
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use futures::Future;
use lightclient::ethereum::{EthereumConfig, EthereumMainnet, EthereumMinimal};
use num_bigint::BigUint;
use prost::Message;
use protos::{
    cosmos::base::tendermint::v1beta1::AbciQueryRequest,
    union::galois::api::v1::union_prover_api_client,
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::Client;
use unionlabs::{
    bounded::BoundedI64,
    ethereum::config::{ChainSpec, Mainnet, Minimal},
    google::protobuf::{any::Any, timestamp::Timestamp},
    hash::{H160, H256, H512},
    ibc::{
        core::{
            client::{height::Height, msg_update_client::MsgUpdateClient},
            connection::{
                msg_connection_open_ack::MsgConnectionOpenAck,
                msg_connection_open_try::MsgConnectionOpenTry,
            },
        },
        lightclients::{cometbls, ethereum, wasm},
    },
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            block_id::BlockId, block_id_flag::BlockIdFlag,
            canonical_block_header::CanonicalPartSetHeader, canonical_block_id::CanonicalBlockId,
            canonical_vote::CanonicalVote, commit::Commit, commit_sig::CommitSig,
            part_set_header::PartSetHeader, signed_header::SignedHeader,
            signed_msg_type::SignedMsgType, simple_validator::SimpleValidator,
        },
    },
    traits::{Chain, HeightOf, LightClientBase},
    union::galois::{
        poll_request::PollRequest,
        poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
        prove_request::ProveRequest,
        prove_response,
        validator_set_commit::ValidatorSetCommit,
    },
    IntoProto, MsgIntoProto,
};

use crate::{
    aggregate,
    aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
    data,
    data::{
        AcknowledgementProof, AnyData, ChannelEndProof, ClientConsensusStateProof,
        ClientStateProof, CommitmentProof, ConnectionProof, Data, LightClientSpecificData,
    },
    defer_relative, fetch,
    fetch::{AnyFetch, Fetch, FetchStateProof, FetchUpdateHeaders, LightClientSpecificFetch},
    identified, msg,
    msg::{AnyMsg, Msg, MsgUpdateClientData},
    seq,
    use_aggregate::{do_aggregate, IsAggregateData, UseAggregate},
    wait,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, DoAggregate, Identified, LightClient, PathOf, RelayerMsg,
};

impl LightClient for EthereumMinimal {
    type BaseCounterparty = Self::Counterparty;

    type Data = EthereumDataMsg<Minimal>;
    type Fetch = EthereumFetchMsg<Self, Minimal>;
    type Aggregate = EthereumAggregateMsg<Self, Minimal>;

    type MsgError = BroadcastTxCommitError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg {
        seq([
            wait::<Self>(self.chain().chain_id(), WaitForBlock(msg.at.increment())),
            fetch::<Self>(
                self.chain().chain_id(),
                LightClientSpecificFetch(EthereumFetchMsg::AbciQuery(FetchAbciQuery {
                    path: msg.path,
                    height: msg.at,
                })),
            ),
        ])
    }

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), BroadcastTxCommitError>> + '_ {
        do_msg::<Self, Minimal>(self.chain().clone(), msg)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<Minimal, Self>(self.chain(), msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(self.chain(), update_info)
    }
}

impl LightClient for EthereumMainnet {
    type BaseCounterparty = Self::Counterparty;

    type Data = EthereumDataMsg<Mainnet>;
    type Fetch = EthereumFetchMsg<Self, Mainnet>;
    type Aggregate = EthereumAggregateMsg<Self, Mainnet>;

    type MsgError = BroadcastTxCommitError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg {
        seq([
            wait::<Self>(self.chain().chain_id(), WaitForBlock(msg.at.increment())),
            fetch::<Self>(
                self.chain().chain_id(),
                LightClientSpecificFetch(EthereumFetchMsg::AbciQuery(FetchAbciQuery {
                    path: msg.path,
                    height: msg.at,
                })),
            ),
        ])
    }

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        do_msg(self.chain().clone(), msg)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(self.chain(), msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(self.chain(), update_info)
    }
}

async fn do_fetch<C, L>(union: &Union, msg: EthereumFetchMsg<L, C>) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Fetch = EthereumFetchMsg<L, C>, Data = EthereumDataMsg<C>>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    match msg {
        EthereumFetchMsg::FetchUntrustedCommit(FetchUntrustedCommit { height, __marker }) => {
            let commit = union
                .tm_client
                .commit(
                    TryInto::<tendermint::block::Height>::try_into(height.revision_height).unwrap(),
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
                            tendermint::block::CommitSig::BlockIdFlagAbsent => CommitSig {
                                block_id_flag: BlockIdFlag::Absent,
                                validator_address: H160([0; 20]),
                                timestamp: unionlabs::google::protobuf::timestamp::Timestamp {
                                    seconds: 0.try_into().unwrap(),
                                    nanos: 0.try_into().unwrap(),
                                },
                                signature: H512([0; 64]),
                            },
                            tendermint::block::CommitSig::BlockIdFlagCommit {
                                validator_address,
                                timestamp,
                                signature,
                            } => CommitSig {
                                block_id_flag: BlockIdFlag::Commit,
                                validator_address: Vec::from(validator_address).try_into().unwrap(),
                                timestamp: {
                                    let ts = tendermint_proto::google::protobuf::Timestamp::from(
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
                            } => CommitSig {
                                block_id_flag: BlockIdFlag::Nil,
                                validator_address: Vec::from(validator_address).try_into().unwrap(),
                                timestamp: {
                                    let ts = tendermint_proto::google::protobuf::Timestamp::from(
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

            let msg = EthereumDataMsg::UntrustedCommit(UntrustedCommit {
                height,
                // REVIEW: Ensure `commit.canonical`?
                signed_header,
                __marker: PhantomData,
            });

            [data::<L>(
                union.chain_id.clone(),
                LightClientSpecificData(msg),
            )]
            .into()
        }
        EthereumFetchMsg::FetchValidators(FetchValidators { height, __marker }) => {
            let validators = union
                .tm_client
                .validators(
                    TryInto::<tendermint::block::Height>::try_into(height.revision_height).unwrap(),
                    tendermint_rpc::Paging::All,
                )
                .await
                .unwrap()
                .validators;

            let msg = EthereumDataMsg::Validators(Validators {
                height,
                validators,
                __marker: PhantomData,
            });

            [data::<L>(
                union.chain_id.clone(),
                LightClientSpecificData(msg),
            )]
            .into()
        }
        EthereumFetchMsg::FetchProveRequest(FetchProveRequest { request, __marker }) => {
            let response = union_prover_api_client::UnionProverApiClient::connect(
                union.prover_endpoint.clone(),
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
            .map(|x| x.into_inner().try_into().unwrap())
            .unwrap();

            match response {
                PollResponse::Pending => [seq([
                    // REVIEW: How long should we wait between polls?
                    defer_relative(3),
                    fetch::<L>(
                        union.chain_id.clone(),
                        LightClientSpecificFetch(EthereumFetchMsg::FetchProveRequest(
                            FetchProveRequest { request, __marker },
                        )),
                    ),
                ])]
                .into(),
                PollResponse::Failed(ProveRequestFailed { message }) => {
                    tracing::error!(%message, "prove request failed");
                    panic!()
                }
                PollResponse::Done(ProveRequestDone { response }) => [data::<L>(
                    union.chain_id.clone(),
                    LightClientSpecificData(EthereumDataMsg::ProveResponse(ProveResponse {
                        response,
                        __marker: PhantomData,
                    })),
                )]
                .into(),
            }
        }
        EthereumFetchMsg::AbciQuery(FetchAbciQuery { path, height }) => {
            let mut client =
                protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                    union.grpc_url.clone(),
                )
                .await
                .unwrap();

            let query_result = client
                .abci_query(AbciQueryRequest {
                    data: path.to_string().into_bytes(),
                    path: "store/ibc/key".to_string(),
                    height: i64::try_from(height.revision_height).unwrap() - 1_i64,
                    prove: true,
                })
                .await
                .unwrap()
                .into_inner();

            let proof = protos::ibc::core::commitment::v1::MerkleProof {
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
            }
            .encode_to_vec();

            [match path {
                unionlabs::proof::Path::ClientStatePath(_) => {
                    data::<L>(union.chain_id(), ClientStateProof { proof, height })
                }
                unionlabs::proof::Path::ClientConsensusStatePath(_) => data::<L>(
                    union.chain_id(),
                    ClientConsensusStateProof { proof, height },
                ),
                unionlabs::proof::Path::ConnectionPath(_) => {
                    data::<L>(union.chain_id(), ConnectionProof { proof, height })
                }
                unionlabs::proof::Path::ChannelEndPath(_) => {
                    data::<L>(union.chain_id(), ChannelEndProof { proof, height })
                }
                unionlabs::proof::Path::CommitmentPath(_) => {
                    data::<L>(union.chain_id(), CommitmentProof { proof, height })
                }
                unionlabs::proof::Path::AcknowledgementPath(_) => {
                    data::<L>(union.chain_id(), AcknowledgementProof { proof, height })
                }
            }]
            .into()
        }
    }
}

fn generate_counterparty_updates<C, L>(
    union: &Union,
    update_info: FetchUpdateHeaders<L>,
) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Union,
        Aggregate = EthereumAggregateMsg<L, C>,
        Fetch = EthereumFetchMsg<L, C>,
    >,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    [seq([
        wait::<L>(
            union.chain_id(),
            // NOTE: There was previously an increment here, but we were unsure why - if there are issues with the updates, it may need to be added back. Please leave a comment explaining why if so!
            WaitForBlock(update_info.update_to),
        ),
        RelayerMsg::Aggregate {
            queue: [
                fetch::<L>(
                    union.chain_id.clone(),
                    LightClientSpecificFetch(EthereumFetchMsg::FetchUntrustedCommit(
                        FetchUntrustedCommit {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        },
                    )),
                ),
                fetch::<L>(
                    union.chain_id.clone(),
                    LightClientSpecificFetch(EthereumFetchMsg::FetchValidators(FetchValidators {
                        height: update_info.update_from,
                        __marker: PhantomData,
                    })),
                ),
                fetch::<L>(
                    union.chain_id.clone(),
                    LightClientSpecificFetch(EthereumFetchMsg::FetchValidators(FetchValidators {
                        height: update_info.update_to,
                        __marker: PhantomData,
                    })),
                ),
            ]
            .into(),
            data: [].into(),
            receiver: aggregate(
                union.chain_id.clone(),
                LightClientSpecificAggregate(EthereumAggregateMsg::AggregateProveRequest(
                    AggregateProveRequest { req: update_info },
                )),
            ),
        },
    ])]
    .into()
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, parse_display::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum EthereumDataMsg<C: ChainSpec> {
    // NOTE: Not used currently?
    // TrustedCommit {
    //     height: Height,
    // },
    #[display("UntrustedCommit")]
    UntrustedCommit(UntrustedCommit<C>),
    #[display("Validators")]
    Validators(Validators<C>),
    #[display("ProveResponse")]
    ProveResponse(ProveResponse<C>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, parse_display::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum EthereumFetchMsg<L: LightClient<HostChain = Union>, C: ChainSpec> {
    // FetchTrustedCommit { height: Height },
    #[display("FetchUntrustedCommit")]
    FetchUntrustedCommit(FetchUntrustedCommit<C>),
    #[display("FetchValidators")]
    FetchValidators(FetchValidators<C>),
    #[display("FetchProveRequest")]
    FetchProveRequest(FetchProveRequest<C>),
    #[display("FetchAbciQuery")]
    AbciQuery(FetchAbciQuery<L>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, parse_display::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum EthereumAggregateMsg<L, C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union>,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
{
    #[display("AggregateProveRequest")]
    AggregateProveRequest(AggregateProveRequest<L>),
    #[display("AggregateHeader")]
    AggregateHeader(AggregateHeader<L>),
}

impl<C, L> From<UntrustedCommit<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Data = EthereumDataMsg<C>>,
{
    fn from(value: UntrustedCommit<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(EthereumDataMsg::UntrustedCommit(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for UntrustedCommit<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Data = EthereumDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            EthereumDataMsg::UntrustedCommit(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<Validators<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Data = EthereumDataMsg<C>>,
{
    fn from(value: Validators<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(EthereumDataMsg::Validators(value)))
    }
}

impl<C, L> TryFrom<Data<L>> for Validators<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Data = EthereumDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            EthereumDataMsg::Validators(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<ProveResponse<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Data = EthereumDataMsg<C>>,
{
    fn from(value: ProveResponse<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(EthereumDataMsg::ProveResponse(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for ProveResponse<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Data = EthereumDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            EthereumDataMsg::ProveResponse(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<L, C> DoAggregate<L> for EthereumAggregateMsg<L, C>
where
    C: ChainSpec,
    // REVIEW: Use trait alias here?
    L: LightClient<
        HostChain = Union,
        Fetch = EthereumFetchMsg<L, C>,
        Aggregate = EthereumAggregateMsg<L, C>,
    >,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
    Identified<L, UntrustedCommit<C>>: IsAggregateData,
    Identified<L, Validators<C>>: IsAggregateData,
    Identified<L, ProveResponse<C>>: IsAggregateData,

    Identified<L, AggregateProveRequest<L>>: UseAggregate<L>,
    Identified<L, AggregateHeader<L>>: UseAggregate<L>,

    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    fn do_aggregate(
        Identified { chain_id, data }: Identified<L, Self>,
        aggregate_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> Vec<RelayerMsg> {
        [match data {
            EthereumAggregateMsg::AggregateProveRequest(data) => {
                do_aggregate(Identified { chain_id, data }, aggregate_data)
            }
            EthereumAggregateMsg::AggregateHeader(data) => {
                do_aggregate(Identified { chain_id, data }, aggregate_data)
            }
        }]
        .into()
    }
}

try_from_relayer_msg! {
    #[EthereumMainnet(
        lc_msg(
            msg = Data(LightClientSpecificData),
            ty = EthereumDataMsg,
            variants(
                UntrustedCommit(UntrustedCommit<Mainnet>),
                Validators(Validators<Mainnet>),
                ProveResponse(ProveResponse<Mainnet>),
            ),
        ),
        lc_msg(
            msg = Fetch(LightClientSpecificFetch),
            ty = EthereumFetchMsg,
            variants(
                FetchUntrustedCommit(FetchUntrustedCommit<Mainnet>),
                FetchValidators(FetchValidators<Mainnet>),
                FetchProveRequest(FetchProveRequest<Mainnet>),
            ),
        ),
    )]
}

try_from_relayer_msg! {
    #[EthereumMinimal(
        lc_msg(
            msg = Data(LightClientSpecificData),
            ty = EthereumDataMsg,
            variants(
                UntrustedCommit(UntrustedCommit<Minimal>),
                Validators(Validators<Minimal>),
                ProveResponse(ProveResponse<Minimal>),
            ),
        ),
        lc_msg(
            msg = Fetch(LightClientSpecificFetch),
            ty = EthereumFetchMsg,
            variants(
                FetchUntrustedCommit(FetchUntrustedCommit<Minimal>),
                FetchValidators(FetchValidators<Minimal>),
                FetchProveRequest(FetchProveRequest<Minimal>),
            ),
        ),
    )]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UntrustedCommit<C: ChainSpec> {
    pub height: Height,
    pub signed_header: SignedHeader,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Validators<C: ChainSpec> {
    pub height: Height,
    // TODO: Use non-`tendermint-rs` type here
    pub validators: Vec<tendermint::validator::Info>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveResponse<C: ChainSpec> {
    pub response: prove_response::ProveResponse,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchUntrustedCommit<C: ChainSpec> {
    pub height: Height,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// TODO: Add Height param
pub struct FetchValidators<C: ChainSpec> {
    pub height: Height,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchProveRequest<C: ChainSpec> {
    pub request: ProveRequest,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchAbciQuery<L: LightClient<HostChain = Union>> {
    path: PathOf<L>,
    height: HeightOf<L::HostChain>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateHeader<L>
where
    L: LightClient<HostChain = Union>,
{
    pub signed_header: SignedHeader,

    pub req: FetchUpdateHeaders<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateProveRequest<L>
where
    L: LightClient<HostChain = Union>,
{
    pub req: FetchUpdateHeaders<L>,
}

async fn do_msg<L, C: ChainSpec>(union: Union, msg: Msg<L>) -> Result<(), BroadcastTxCommitError>
where
    L: LightClient<HostChain = Union, Config = EthereumConfig, MsgError = BroadcastTxCommitError>,
    L::Counterparty: LightClientBase<HostChain = Evm<C>>,
{
    union
        .signers
        .with(|signer| async {
            let msg_any = match msg {
                Msg::ConnectionOpenInit(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::ConnectionOpenTry(data) => Any(MsgConnectionOpenTry {
                    client_id: data.msg.client_id,
                    client_state: Any(wasm::client_state::ClientState {
                        latest_height: data.msg.client_state.latest_height,
                        data: data.msg.client_state,
                        code_id: H256::default(),
                    }),
                    counterparty: data.msg.counterparty,
                    delay_period: data.msg.delay_period,
                    counterparty_versions: data.msg.counterparty_versions,
                    proof_height: data.msg.proof_height,
                    proof_init: data.msg.proof_init,
                    proof_client: data.msg.proof_client,
                    proof_consensus: data.msg.proof_consensus,
                    consensus_height: data.msg.consensus_height,
                })
                .into_proto_with_signer(&signer),
                Msg::ConnectionOpenAck(data) => Any(MsgConnectionOpenAck {
                    client_state: Any(wasm::client_state::ClientState {
                        latest_height: data.msg.client_state.latest_height,
                        data: data.msg.client_state,
                        code_id: H256::default(),
                    }),
                    proof_height: data.msg.proof_height,
                    proof_try: data.msg.proof_try,
                    proof_client: data.msg.proof_client,
                    proof_consensus: data.msg.proof_consensus,
                    consensus_height: data.msg.consensus_height,
                    connection_id: data.msg.connection_id,
                    counterparty_connection_id: data.msg.counterparty_connection_id,
                    version: data.msg.version,
                })
                .into_proto_with_signer(&signer),
                Msg::ConnectionOpenConfirm(data) => Any(data.0).into_proto_with_signer(&signer),
                Msg::ChannelOpenInit(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::ChannelOpenTry(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::ChannelOpenAck(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::ChannelOpenConfirm(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::RecvPacket(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::AckPacket(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::CreateClient(mut data) => {
                    // i hate this lol
                    data.msg.client_state.0.code_id = data.config.code_id;

                    Any(data.msg).into_proto_with_signer(&signer)
                }
                Msg::UpdateClient(data) => {
                    // check if update has already been done
                    // let existing_consensus_height = L::from_chain(union.clone())
                    //     .query_client_state(
                    //         data.msg.client_id.clone().into(),
                    //         union.query_latest_height().await,
                    //     )
                    //     .await
                    //     .0
                    //     .latest_height;

                    // let update_height = data.update_from;
                    // if dbg!(existing_consensus_height) >= dbg!(update_height.into_height()) {
                    //     tracing::warn!(
                    //         "consensus state has already been updated to or past {update_height}, found {existing_consensus_height}"
                    //     );

                    //     // don't do the update, already has been done
                    //     return;
                    // }

                    Any(data.msg).into_proto_with_signer(&signer)
                }
            };

            union.broadcast_tx_commit(signer, [msg_any]).await
        })
        .await
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

impl<L, C> UseAggregate<L> for Identified<L, AggregateProveRequest<L>>
where
    C: ChainSpec,
    // REVIEW: Use trait alias here?
    L: LightClient<
        HostChain = Union,
        Fetch = EthereumFetchMsg<L, C>,
        Aggregate = EthereumAggregateMsg<L, C>,
    >,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
    Identified<L, UntrustedCommit<C>>: IsAggregateData,
    Identified<L, Validators<C>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![Identified<L, UntrustedCommit<C>>, Identified<L, Validators<C>>, Identified<L, Validators<C>>];

    fn aggregate(
        Identified {
            chain_id,
            data: AggregateProveRequest { req },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: untrusted_commit_chain_id,
                data: UntrustedCommit {
                    height: untrusted_commit_height,
                    signed_header,
                    __marker: _
                }
            },
            Identified {
                chain_id: trusted_validators_chain_id,
                data: Validators {
                    height: trusted_validators_height,
                    validators: trusted_validators,
                    __marker: _
                }
            },
            Identified {
                chain_id: untrusted_validators_chain_id,
                data: Validators {
                    height: untrusted_validators_height,
                    validators: untrusted_validators,
                    __marker: _
                }
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(untrusted_commit_chain_id, untrusted_validators_chain_id);
        assert_eq!(chain_id, trusted_validators_chain_id);
        assert_eq!(chain_id, untrusted_validators_chain_id);

        assert_eq!(req.update_from, trusted_validators_height);
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
                match sig.block_id_flag {
                    BlockIdFlag::Absent => {
                        tracing::debug!("Validator did not sign: {:?}", sig);
                    }
                    BlockIdFlag::Commit => {
                        if let Some(validator_index) = validators_map
                            .get(&sig.validator_address.0.to_vec().try_into().unwrap())
                        {
                            bitmap.set_bit(*validator_index as u64, true);
                            signatures.push(sig.signature.clone().into());
                            tracing::debug!(
                                "Validator {:?} at index {} signed",
                                sig.validator_address,
                                validator_index
                            );
                        } else {
                            tracing::warn!("Validator set drifted? Could not find validator for signature {:?}", sig.validator_address);
                        }
                    }
                    BlockIdFlag::Nil { .. } => {
                        tracing::warn!("Validator commit is nil: {:?}", sig);
                    }
                    BlockIdFlag::Unknown => {
                        tracing::warn!("Validator commit is unknown, wtf: {:?}", sig);
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

        RelayerMsg::Aggregate {
            queue: [fetch(
                chain_id.clone(),
                LightClientSpecificFetch(EthereumFetchMsg::FetchProveRequest(FetchProveRequest {
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
                    __marker: PhantomData,
                })),
            )]
            .into(),
            data: [].into(),
            receiver: aggregate(
                chain_id,
                LightClientSpecificAggregate(EthereumAggregateMsg::AggregateHeader(
                    AggregateHeader { signed_header, req },
                )),
            ),
        }
    }
}

impl<L, C> UseAggregate<L> for Identified<L, AggregateHeader<L>>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Fetch = EthereumFetchMsg<L, C>>,
    Identified<L, ProveResponse<C>>: IsAggregateData,
    Identified<L, Validators<C>>: IsAggregateData,

    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![Identified<L, ProveResponse<C>>];

    fn aggregate(
        Identified {
            chain_id,
            data: AggregateHeader { signed_header, req },
        }: Self,
        hlist_pat![Identified {
            chain_id: untrusted_commit_chain_id,
            data: ProveResponse {
                response,
                __marker: _,
            }
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, untrusted_commit_chain_id);

        msg::<L::Counterparty>(
            req.counterparty_chain_id,
            MsgUpdateClientData {
                msg: MsgUpdateClient {
                    client_id: req.counterparty_client_id.clone(),
                    client_message: cometbls::header::Header {
                        signed_header,
                        trusted_height: req.update_from,
                        zero_knowledge_proof: response.proof.evm_proof,
                    },
                },
                update_from: req.update_from,
            },
        )
    }
}
