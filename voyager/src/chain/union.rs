use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    marker::PhantomData,
};

use chain_utils::{
    evm::Evm,
    union::{BroadcastTxCommitError, Union, Wasm},
};
use clap::Args;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use futures::Future;
use num_bigint::BigUint;
use prost::Message;
use protos::{
    cosmos::base::tendermint::v1beta1::AbciQueryRequest,
    union::galois::api::v1::{union_prover_api_client, ProveResponse as RawProveResponse},
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::Client;
use unionlabs::{
    bounded_int::BoundedI64,
    ethereum::{Address, H256, H512},
    ethereum_consts_traits::{ChainSpec, Mainnet, Minimal},
    ibc::{
        core::client::{height::Height, msg_update_client::MsgUpdateClient},
        google::protobuf::{any::Any, timestamp::Timestamp},
        lightclients::{cometbls, ethereum, wasm},
    },
    id::Id,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
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
    union::galois::{prove_request::ProveRequest, validator_set_commit::ValidatorSetCommit},
    IntoProto, MsgIntoProto, Proto, TryFromProto, TryFromProtoErrorOf,
};

use crate::{
    chain::{
        evm::{CometblsMainnet, CometblsMinimal},
        proof::StateProof,
        try_from_relayer_msg, Chain, ClientStateOf, ConsensusStateOf, HeightOf, IbcStateRead,
        LightClient, QueryHeight,
    },
    msg::{
        aggregate::{Aggregate, LightClientSpecificAggregate},
        data::{Data, LightClientSpecificData},
        fetch::{Fetch, FetchTrustedClientState, FetchUpdateHeaders, LightClientSpecificFetch},
        identified,
        msg::{Msg, MsgUpdateClientData},
        wait::{Wait, WaitForBlock},
        AggregateData, AggregateReceiver, AnyLcMsg, DoAggregate, Identified, LcMsg, RelayerMsg,
    },
    queue::aggregate_data::{do_aggregate, UseAggregate},
};

/// The 08-wasm light client tracking ethereum, running on the union chain.
pub struct EthereumMinimal {
    chain: <Self as LightClient>::HostChain,
}

/// The 08-wasm light client tracking ethereum, running on the union chain.
pub struct EthereumMainnet {
    chain: <Self as LightClient>::HostChain,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Args)]
pub struct EthereumConfig {
    #[arg(long)]
    pub code_id: H256,
}

impl LightClient for EthereumMinimal {
    type HostChain = Union;
    type Counterparty = CometblsMinimal;

    type ClientId = Id<Wasm>;
    type ClientType = Wasm;

    type Config = EthereumConfig;

    type Data = EthereumDataMsg<Minimal>;
    type Fetch = EthereumFetchMsg<Minimal>;
    type Aggregate = EthereumAggregateMsg<Self, Minimal>;

    type MsgError = BroadcastTxCommitError;

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        self::msg(self.chain.clone(), msg)
    }

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClient>::HostChain>> + '_
    {
        query_client_state::<Self>(&self.chain, client_id, height)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(&self.chain, msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(&self.chain, update_info)
    }
}

async fn do_fetch<C, L>(union: &Union, msg: EthereumFetchMsg<C>) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Fetch = EthereumFetchMsg<C>, Data = EthereumDataMsg<C>>,
    AnyLcMsg: From<LcMsg<L>>,
    AggregateData: From<identified!(Data<L>)>,
    AggregateReceiver: From<identified!(Aggregate<L>)>,
{
    let msg = match msg {
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
                                validator_address: Address([0; 20]),
                                timestamp: unionlabs::ibc::google::protobuf::timestamp::Timestamp {
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

            EthereumDataMsg::UntrustedCommit(UntrustedCommit {
                height,
                // REVIEW: Ensure `commit.canonical`?
                signed_header,
                __marker: PhantomData,
            })
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

            EthereumDataMsg::Validators(Validators {
                height,
                validators,
                __marker: PhantomData,
            })
        }
        EthereumFetchMsg::FetchProveRequest(FetchProveRequest { request, __marker }) => {
            let response = union_prover_api_client::UnionProverApiClient::connect(
                tonic::transport::Endpoint::from_shared(union.prover_endpoint.clone()).unwrap(),
            )
            .await
            .unwrap()
            .prove(request.into_proto())
            .await
            .unwrap()
            .into_inner();

            EthereumDataMsg::ProveResponse(ProveResponse {
                response,
                __marker: PhantomData,
            })
        }
    };

    [RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L>::Data(
        Identified {
            chain_id: union.chain_id.clone(),
            data: Data::LightClientSpecific(LightClientSpecificData(msg)),
        },
    )))]
    .into()
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
        Fetch = EthereumFetchMsg<C>,
    >,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
    AnyLcMsg: From<LcMsg<L>>,
    AggregateData: From<identified!(Data<L>)>,
    AggregateReceiver: From<identified!(Aggregate<L>)>,
{
    [RelayerMsg::Sequence(
        [
            RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L>::Wait(Identified {
                chain_id: union.chain_id(),
                data: Wait::Block(WaitForBlock(update_info.update_to.increment())),
            }))),
            RelayerMsg::Aggregate {
                queue: [
                    RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L>::Fetch(Identified {
                        chain_id: union.chain_id.clone(),
                        data: Fetch::LightClientSpecific(LightClientSpecificFetch(
                            EthereumFetchMsg::FetchUntrustedCommit(FetchUntrustedCommit {
                                height: update_info.update_to,
                                __marker: PhantomData,
                            }),
                        )),
                    }))),
                    RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L>::Fetch(Identified {
                        chain_id: union.chain_id.clone(),
                        data: Fetch::LightClientSpecific(LightClientSpecificFetch(
                            EthereumFetchMsg::FetchValidators(FetchValidators {
                                height: update_info.update_from,
                                __marker: PhantomData,
                            }),
                        )),
                    }))),
                    RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L>::Fetch(Identified {
                        chain_id: union.chain_id.clone(),
                        data: Fetch::LightClientSpecific(LightClientSpecificFetch(
                            EthereumFetchMsg::FetchValidators(FetchValidators {
                                height: update_info.update_to,
                                __marker: PhantomData,
                            }),
                        )),
                    }))),
                ]
                .into(),
                data: [].into(),
                receiver: AggregateReceiver::from(Identified {
                    chain_id: union.chain_id.clone(),
                    data: Aggregate::LightClientSpecific(LightClientSpecificAggregate(
                        EthereumAggregateMsg::AggregateProveRequest(AggregateProveRequest {
                            req: update_info,
                        }),
                    )),
                }),
            },
        ]
        .into(),
    )]
    .into()
}

impl LightClient for EthereumMainnet {
    type HostChain = Union;
    type Counterparty = CometblsMainnet;

    type ClientId = Id<Wasm>;
    type ClientType = Wasm;

    type Config = EthereumConfig;

    type Data = EthereumDataMsg<Mainnet>;
    type Fetch = EthereumFetchMsg<Mainnet>;
    type Aggregate = EthereumAggregateMsg<Self, Mainnet>;

    type MsgError = BroadcastTxCommitError;

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        self::msg(self.chain.clone(), msg)
    }

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClient>::HostChain>> + '_
    {
        query_client_state::<Self>(&self.chain, client_id, height)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(&self.chain, msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(&self.chain, update_info)
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum EthereumDataMsg<C: ChainSpec> {
    // NOTE: Not used currently?
    // TrustedCommit {
    //     height: Height,
    // },
    #[display(fmt = "UntrustedCommit")]
    UntrustedCommit(UntrustedCommit<C>),
    #[display(fmt = "Validators")]
    Validators(Validators<C>),
    #[display(fmt = "ProveResponse")]
    ProveResponse(ProveResponse<C>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum EthereumFetchMsg<C: ChainSpec> {
    // FetchTrustedCommit { height: Height },
    #[display(fmt = "FetchUntrustedCommit")]
    FetchUntrustedCommit(FetchUntrustedCommit<C>),
    #[display(fmt = "FetchValidators")]
    FetchValidators(FetchValidators<C>),
    #[display(fmt = "FetchProveRequest")]
    FetchProveRequest(FetchProveRequest<C>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum EthereumAggregateMsg<L, C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union>,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
{
    #[display(fmt = "AggregateProveRequest")]
    AggregateProveRequest(AggregateProveRequest<L>),
    #[display(fmt = "AggregateHeader")]
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
        Fetch = EthereumFetchMsg<C>,
        Aggregate = EthereumAggregateMsg<L, C>,
    >,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
    Identified<L, UntrustedCommit<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, Validators<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, ProveResponse<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,

    Identified<L, AggregateProveRequest<L>>: UseAggregate<L>,
    Identified<L, AggregateHeader<L>>: UseAggregate<L>,

    AnyLcMsg: From<LcMsg<L>>,
    AggregateReceiver: From<identified!(Aggregate<L>)>,
{
    fn do_aggregate(
        Identified { chain_id, data }: Identified<L, Self>,
        aggregate_data: VecDeque<AggregateData>,
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
        lc_msg(
            msg = Aggregate(LightClientSpecificAggregate),
            ty = EthereumAggregateMsg,
            variants(
                AggregateHeader(AggregateHeader<EthereumMainnet>),
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
        lc_msg(
            msg = Aggregate(LightClientSpecificAggregate),
            ty = EthereumAggregateMsg,
            variants(
                AggregateHeader(AggregateHeader<EthereumMinimal>),
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
    pub validators: Vec<tendermint::validator::Info>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveResponse<C: ChainSpec> {
    pub response: RawProveResponse,
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
pub struct AggregateHeader<L>
where
    L: LightClient<HostChain = Union>,
    // L::Counterparty: LightClient<HostChain = Evm<C>>,
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
    // pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    // pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    // pub update_from: HeightOf<ChainOf<L>>,
    // pub update_to: HeightOf<ChainOf<L>>,
}

async fn msg<L, C: ChainSpec>(union: Union, msg: Msg<L>) -> Result<(), L::MsgError>
where
    L: LightClient<HostChain = Union, Config = EthereumConfig, MsgError = BroadcastTxCommitError>,
    <L::Counterparty as LightClient>::HostChain: Chain<
        SelfClientState = Any<wasm::client_state::ClientState<ethereum::client_state::ClientState>>,
        SelfConsensusState = Any<
            wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>,
        >,
        Header = wasm::header::Header<ethereum::header::Header<C>>,
    >,
{
    union
        .signers
        .with(|signer| async {
            let msg_any = match msg {
                Msg::ConnectionOpenInit(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::ConnectionOpenTry(data) => Any(data.msg).into_proto_with_signer(&signer),
                Msg::ConnectionOpenAck(data) => Any(data.msg).into_proto_with_signer(&signer),
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

async fn query_client_state<L>(
    union: &Union,
    client_id: chain_utils::union::UnionClientId,
    height: Height,
) -> ClientStateOf<<L::Counterparty as LightClient>::HostChain>
where
    L: LightClient<HostChain = Union>,
    ClientStateOf<<L::Counterparty as LightClient>::HostChain>: Proto<Proto = protos::google::protobuf::Any>
        + TryFrom<protos::google::protobuf::Any>
        + TryFromProto<Proto = protos::google::protobuf::Any>,
    // NOTE: This bound can be removed once we don't unwrap anymore
    TryFromProtoErrorOf<ClientStateOf<<L::Counterparty as LightClient>::HostChain>>: Debug,
{
    let mut client =
        protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
            union.grpc_url.clone(),
        )
        .await
        .unwrap();

    <ClientStateOf<<L::Counterparty as LightClient>::HostChain>>::try_from_proto_bytes(
        &client
            .abci_query(AbciQueryRequest {
                data: ClientStatePath { client_id }.to_string().into_bytes(),
                path: "store/ibc/key".to_string(),
                height: height.revision_height.try_into().unwrap(),
                prove: false,
            })
            .await
            .unwrap()
            .into_inner()
            .value,
    )
    .unwrap()
}

// IbcStateRead stuff

trait AbciStateRead<Counterparty>: IbcPath<Union, Counterparty>
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
    fn state_proof(&self, path: P, at: Height) -> impl Future<Output = StateProof<P::Output>> + '_ {
        async move {
            tracing::info!(%path, %at, "fetching state proof");

            let mut client =
                protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                    self.grpc_url.clone(),
                )
                .await
                .unwrap();

            let query_result = client
                .abci_query(AbciQueryRequest {
                    data: path.to_string().into_bytes(),
                    path: "store/ibc/key".to_string(),
                    height: i64::try_from(at.revision_height).unwrap() - 1_i64,
                    prove: true,
                })
                .await
                .unwrap()
                .into_inner();

            // dbg!(&query_result);

            let state = P::from_abci_bytes(query_result.value);
            tracing::info!(?state, "fetched state proof");

            StateProof {
                state,
                proof: protos::ibc::core::commitment::v1::MerkleProof {
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
                .encode_to_vec(),
                // NOTE: query_result.height == AbciQueryRequest.height, hence the increment
                // we could use at.revision_height here as well, maybe add an assert?
                proof_height: self
                    .make_height(query_result.height.try_into().unwrap())
                    .increment(),
            }
        }
    }
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
        Fetch = EthereumFetchMsg<C>,
        Aggregate = EthereumAggregateMsg<L, C>,
    >,
    L::Counterparty: LightClient<HostChain = Evm<C>>,
    Identified<L, UntrustedCommit<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, Validators<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    AnyLcMsg: From<LcMsg<L>>,
    AggregateReceiver: From<identified!(Aggregate<L>)>,
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
            queue: [RelayerMsg::Lc(AnyLcMsg::from(LcMsg::Fetch(Identified {
                chain_id: chain_id.clone(),
                data: Fetch::LightClientSpecific(LightClientSpecificFetch(
                    EthereumFetchMsg::FetchProveRequest(FetchProveRequest {
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
                    }),
                )),
            })))]
            .into(),
            data: [].into(),
            receiver: AggregateReceiver::from(Identified {
                chain_id,
                data: Aggregate::LightClientSpecific(LightClientSpecificAggregate(
                    EthereumAggregateMsg::AggregateHeader(AggregateHeader { signed_header, req }),
                )),
            }),
        }
    }
}

impl<L, C> UseAggregate<L> for Identified<L, AggregateHeader<L>>
where
    C: ChainSpec,
    L: LightClient<HostChain = Union, Fetch = EthereumFetchMsg<C>>,
    // L::Counterparty: LightClient<HostChain = Evm<C>>,
    Identified<L, ProveResponse<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
    Identified<L, Validators<C>>:
        TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,

    AnyLcMsg: From<LcMsg<L>>,
    AnyLcMsg: From<LcMsg<L::Counterparty>>,
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

        RelayerMsg::Sequence(
            [
                RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L::Counterparty>::Msg(Identified {
                    chain_id: req.counterparty_chain_id,
                    data: Msg::UpdateClient(MsgUpdateClientData {
                        msg: MsgUpdateClient {
                            client_id: req.counterparty_client_id.clone(),
                            client_message: cometbls::header::Header {
                                signed_header,
                                trusted_height: req.update_from,
                                zero_knowledge_proof: response.proof.unwrap().evm_proof,
                            },
                        },
                        update_from: req.update_from,
                    }),
                }))),
                // RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L::Counterparty>::Fetch(
                //     Identified {
                //         chain_id: req.counterparty_chain_id,
                //         data: Fetch::TrustedClientState(FetchTrustedClientState {
                //             // NOTE: We can pass update_to directly here since cosmos -> evm always updates to the exact height requested.
                //             at: QueryHeight::Specific(req.update_to),
                //             // at: QueryHeight::Latest,
                //             client_id: req.counterparty_client_id,
                //         }),
                //     },
                // ))),
                RelayerMsg::Lc(AnyLcMsg::from(LcMsg::<L>::Fetch(Identified {
                    chain_id,
                    data: Fetch::TrustedClientState(FetchTrustedClientState {
                        // NOTE: We can pass update_to directly here since cosmos -> evm always updates to the exact height requested.
                        at: QueryHeight::Specific(req.update_to),
                        // at: QueryHeight::Latest,
                        client_id: req.client_id,
                    }),
                }))),
            ]
            .into(),
        )
    }
}
