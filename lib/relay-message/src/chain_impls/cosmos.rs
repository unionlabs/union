use std::{collections::VecDeque, marker::PhantomData};

use chain_utils::{
    cosmos::Cosmos,
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain},
    wasm::Wraps,
};
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    effect, fetch, queue_msg, wait, QueueMsg,
};
use unionlabs::{
    encoding::{Decode, Encode, Proto},
    google::protobuf::any::IntoAny,
    hash::H160,
    ibc::{
        core::{
            client::{height::IsHeight, msg_update_client::MsgUpdateClient},
            commitment::merkle_proof::MerkleProof,
        },
        lightclients::tendermint,
    },
    ics24::ClientStatePath,
    tendermint::types::validator::Validator,
    traits::{Chain, ClientStateOf, ConsensusStateOf, HeaderOf},
    TypeUrl,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::cosmos_sdk::{
        data::{TrustedCommit, TrustedValidators, UntrustedCommit, UntrustedValidators},
        do_msg,
        fetch::{
            fetch_trusted_commit, fetch_trusted_validators, fetch_untrusted_commit,
            fetch_untrusted_validators, FetchAbciQuery, FetchTrustedCommit, FetchTrustedValidators,
            FetchUntrustedCommit, FetchUntrustedValidators,
        },
        fetch_abci_query, CosmosSdkChainSealed,
    },
    data::{AnyData, Data, IbcState},
    effect::{AnyEffect, Effect, MsgUpdateClientData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified, seq,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchUpdateHeaders, DoMsg, Identified,
    RelayMessageTypes,
};

impl ChainExt for Cosmos {
    type Data<Tr: ChainExt> = CosmosDataMsg<Self, Tr>;
    type Fetch<Tr: ChainExt> = CosmosFetch<Cosmos, Tr>;
    type Aggregate<Tr: ChainExt> = CosmosAggregateMsg<Cosmos, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = ();
}

impl CosmosSdkChainSealed for Cosmos {}

impl<Tr: ChainExt> DoMsg<Cosmos, Tr> for Cosmos
where
    ConsensusStateOf<Tr>: Encode<Proto> + TypeUrl,
    ClientStateOf<Tr>: Encode<Proto> + TypeUrl,
    HeaderOf<Tr>: Encode<Proto> + TypeUrl,

    Tr::StoredClientState<Cosmos>: IntoAny,
    Tr::StateProof: Encode<Proto>,
{
    async fn msg(&self, msg: Effect<Cosmos, Tr>) -> Result<(), BroadcastTxCommitError> {
        do_msg(
            self,
            msg,
            |(), client_state, consensus_state| {
                (
                    client_state.into_any().into(),
                    consensus_state.into_any().into(),
                )
            },
            |client_message| client_message.into_any().into(),
        )
        .await
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
    fn fetch_update_headers(
        hc: &Hc,
        update_info: FetchUpdateHeaders<Hc, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
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
                        Fetch::specific(FetchTrustedCommit {
                            height: update_info.update_from.increment(),
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        Fetch::specific(FetchUntrustedCommit {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        Fetch::specific(FetchTrustedValidators {
                            height: update_info.update_from.increment(),
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
                ],
                [],
                id(
                    hc.chain_id(),
                    Aggregate::specific(AggregateHeader { req: update_info }),
                ),
            ),
        ])
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum CosmosDataMsg<Hc: ChainExt, Tr: ChainExt> {
    TrustedCommit(TrustedCommit<Hc, Tr>),
    UntrustedCommit(UntrustedCommit<Hc, Tr>),
    TrustedValidators(TrustedValidators<Hc, Tr>),
    UntrustedValidators(UntrustedValidators<Hc, Tr>),
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum CosmosFetch<Hc: ChainExt, Tr: ChainExt> {
    FetchTrustedCommit(FetchTrustedCommit<Hc, Tr>),
    FetchUntrustedCommit(FetchUntrustedCommit<Hc, Tr>),
    FetchTrustedValidators(FetchTrustedValidators<Hc, Tr>),
    FetchUntrustedValidators(FetchUntrustedValidators<Hc, Tr>),
    AbciQuery(FetchAbciQuery<Hc, Tr>),
}

impl<Hc, Tr> DoFetch<Hc> for CosmosFetch<Hc, Tr>
where
    Hc: CosmosSdkChain
        + ChainExt<
            StateProof = MerkleProof,
            Data<Tr> = CosmosDataMsg<Hc, Tr>,
            Fetch<Tr> = CosmosFetch<Hc, Tr>,
        >,
    Tr: ChainExt,

    Hc::StoredClientState<Tr>: Decode<Proto>,
    Hc::StoredConsensusState<Tr>: Decode<Proto>,

    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,

    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
{
    async fn do_fetch(hc: &Hc, msg: Self) -> QueueMsg<RelayMessageTypes> {
        match msg {
            Self::FetchTrustedCommit(FetchTrustedCommit {
                height,
                __marker: _,
            }) => fetch_trusted_commit(hc, height).await,
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
            Self::AbciQuery(FetchAbciQuery { path, height, ty }) => {
                fetch_abci_query::<Hc, Tr>(hc, path, height, ty).await
            }
        }
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum CosmosAggregateMsg<Hc: ChainExt, Tr: ChainExt> {
    AggregateHeader(AggregateHeader<Hc, Tr>),
}

impl<Hc, Tr> DoAggregate for Identified<Hc, Tr, CosmosAggregateMsg<Hc, Tr>>
where
    Tr: ChainExt,
    Hc: ChainExt,

    identified!(TrustedCommit<Hc, Tr>): IsAggregateData,
    identified!(UntrustedCommit<Hc, Tr>): IsAggregateData,
    identified!(TrustedValidators<Hc, Tr>): IsAggregateData,
    identified!(UntrustedValidators<Hc, Tr>): IsAggregateData,

    Identified<Hc, Tr, AggregateHeader<Hc, Tr>>: UseAggregate<RelayMessageTypes>,

    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t: data,
            __marker: _,
        }: Self,
        aggregate_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes> {
        match data {
            CosmosAggregateMsg::AggregateHeader(data) => {
                do_aggregate(id(chain_id, data), aggregate_data)
            }
        }
    }
}

try_from_relayer_msg! {
    chain = Cosmos,
    generics = (Tr: ChainExt),
    msgs = CosmosDataMsg(
        TrustedCommit(TrustedCommit<Cosmos, Tr>),
        UntrustedCommit(UntrustedCommit<Cosmos, Tr>),
        TrustedValidators(TrustedValidators<Cosmos, Tr>),
        UntrustedValidators(UntrustedValidators<Cosmos, Tr>),
    ),
}

#[queue_msg]
pub struct AggregateHeader<Hc: ChainExt, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

impl<Hc, Tr> UseAggregate<RelayMessageTypes> for Identified<Hc, Tr, AggregateHeader<Hc, Tr>>
where
    Hc: ChainExt<Header = <Cosmos as Chain>::Header>,
    Tr: ChainExt,

    identified!(TrustedCommit<Hc, Tr>): IsAggregateData,
    identified!(UntrustedCommit<Hc, Tr>): IsAggregateData,
    identified!(TrustedValidators<Hc, Tr>): IsAggregateData,
    identified!(UntrustedValidators<Hc, Tr>): IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        identified!(TrustedCommit<Hc, Tr>),
        identified!(UntrustedCommit<Hc, Tr>),
        identified!(TrustedValidators<Hc, Tr>),
        identified!(UntrustedValidators<Hc, Tr>),
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
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(chain_id, untrusted_commit_chain_id);

        let trusted_valset = mk_valset(
            trusted_validators,
            trusted_signed_header.header.proposer_address,
        );

        let untrusted_valset = mk_valset(
            untrusted_validators,
            untrusted_signed_header.header.proposer_address,
        );

        effect(id::<Tr, Hc, _>(
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
