use std::{collections::VecDeque, marker::PhantomData};

use chain_utils::{
    berachain::Berachain,
    ethereum::{EthereumChainExt, EthereumConsensusChain, IbcHandlerExt},
};
use enumorph::Enumorph;
use ethers::providers::Middleware;
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, effect, fetch, queue_msg, seq, wait, QueueMsg,
};
use unionlabs::{
    berachain::{BerachainChainSpec, LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX},
    cosmos::ics23::commitment_proof::CommitmentProof,
    encoding::{Decode, DecodeAs, Encode, EthAbi, Proto, Ssz},
    hash::H160,
    ibc::{
        core::{
            client::{height::IsHeight, msg_update_client::MsgUpdateClient},
            commitment::merkle_proof::MerkleProof,
        },
        lightclients::{
            berachain,
            ethereum::{
                account_proof::AccountProof, execution_payload_header::ExecutionPayloadHeader,
            },
        },
    },
    ics24::ClientStatePath,
    traits::{Chain, HeightOf, IbcStateEncodingOf},
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::{
        cosmos::mk_valset,
        cosmos_sdk::{
            data::{TrustedCommit, TrustedValidators, UntrustedCommit, UntrustedValidators},
            fetch::{
                FetchTrustedCommit, FetchTrustedValidators, FetchUntrustedCommit,
                FetchUntrustedValidators,
            },
        },
        ethereum::{
            self, fetch_get_proof, fetch_ibc_state, EthereumConfig, FetchIbcState, GetProof,
            TxSubmitError,
        },
    },
    data::{AnyData, Data},
    effect::{AnyEffect, Effect, MsgUpdateClientData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForHeight},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayMessageTypes,
};

impl ChainExt for Berachain {
    type Data<Tr: ChainExt> = BerachainData<Tr>;
    type Fetch<Tr: ChainExt> = BerachainFetch<Tr>;
    type Aggregate<Tr: ChainExt> = BerachainAggregate<Tr>;

    type MsgError = TxSubmitError;

    type Config = EthereumConfig;
}

#[queue_msg]
#[derive(Enumorph)]
pub enum BerachainData<Tr: ChainExt> {
    // Eth (nothing yet)
    IbcAccountProof(IbcAccountProof<Tr>),

    // Cosmos
    LatestExecutionPayloadHeaderAbciProof(LatestExecutionPayloadHeaderAbciProof),
    TrustedCommit(TrustedCommit<Berachain, Tr>),
    UntrustedCommit(UntrustedCommit<Berachain, Tr>),
    TrustedValidators(TrustedValidators<Berachain, Tr>),
    UntrustedValidators(UntrustedValidators<Berachain, Tr>),
}

#[queue_msg]
pub struct IbcAccountProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Berachain>,
    pub proof: AccountProof,
}

#[queue_msg]
pub struct LatestExecutionPayloadHeaderAbciProof {
    header: ExecutionPayloadHeader<BerachainChainSpec>,
    proof: MerkleProof,
}

#[queue_msg]
#[derive(Enumorph)]
pub enum BerachainFetch<Tr: ChainExt> {
    // Eth
    FetchIbcState(FetchIbcState<Berachain, Tr>),
    FetchGetProof(GetProof<Berachain, Tr>),
    FetchIbcAccountProof(FetchIbcAccountProof<Tr>),

    // Cosmos
    FetchLatestExecutionPayloadHeaderAbciProof(FetchLatestExecutionPayloadHeaderAbciProof<Tr>),
    FetchTrustedCommit(FetchTrustedCommit<Berachain, Tr>),
    FetchUntrustedCommit(FetchUntrustedCommit<Berachain, Tr>),
    FetchTrustedValidators(FetchTrustedValidators<Berachain, Tr>),
    FetchUntrustedValidators(FetchUntrustedValidators<Berachain, Tr>),
}

#[queue_msg]
pub struct FetchIbcAccountProof<#[cover] Tr> {
    pub ibc_contract_address: H160,
    pub height: HeightOf<Berachain>,
}

#[queue_msg]
pub struct FetchLatestExecutionPayloadHeaderAbciProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Berachain>,
}

#[queue_msg]
#[derive(Enumorph)]
pub enum BerachainAggregate<Tr: ChainExt> {
    AggregateHeader(AggregateHeader<Berachain, Tr>),
}

#[queue_msg]
pub struct AggregateHeader<Hc: ChainExt, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Hc, Tr>,
}

impl<Tr> UseAggregate<RelayMessageTypes> for identified!(AggregateHeader<Berachain, Tr>)
where
    Tr: ChainExt,

    identified!(TrustedCommit<Berachain, Tr>): IsAggregateData,
    identified!(UntrustedCommit<Berachain, Tr>): IsAggregateData,
    identified!(TrustedValidators<Berachain, Tr>): IsAggregateData,
    identified!(UntrustedValidators<Berachain, Tr>): IsAggregateData,
    Identified<Berachain, Tr, LatestExecutionPayloadHeaderAbciProof>: IsAggregateData,
    Identified<Berachain, Tr, IbcAccountProof<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Berachain>)>,
{
    type AggregatedData = HList![
        Identified<Berachain, Tr, IbcAccountProof<Tr>>,
        Identified<Berachain, Tr, LatestExecutionPayloadHeaderAbciProof>,
        identified!(TrustedCommit<Berachain, Tr>),
        identified!(UntrustedCommit<Berachain, Tr>),
        identified!(TrustedValidators<Berachain, Tr>),
        identified!(UntrustedValidators<Berachain, Tr>),
    ];

    fn aggregate(
        Identified {
            chain_id,
            t: AggregateHeader { req },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: _,
                t: IbcAccountProof {
                    height: _,
                    proof: ibc_account_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: _,
                t: LatestExecutionPayloadHeaderAbciProof {
                    header: execution_header,
                    proof: execution_header_proof
                },
                __marker: _,
            },
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

        effect(id::<Tr, Berachain, _>(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id.clone(),
                client_message: berachain::header::Header {
                    cometbft_header: unionlabs::ibc::lightclients::tendermint::header::Header {
                        signed_header: untrusted_signed_header,
                        trusted_height: req.update_from,
                        validator_set: untrusted_valset,
                        trusted_validators: trusted_valset,
                    },
                    execution_header,
                    execution_header_proof,
                    account_proof: ibc_account_proof,
                },
            }),
        ))
    }
}

impl<Tr> DoFetch<Berachain> for BerachainFetch<Tr>
where
    Tr: ChainExt<
        SelfClientState: Decode<IbcStateEncodingOf<Berachain>> + Encode<EthAbi>,
        SelfConsensusState: Decode<IbcStateEncodingOf<Berachain>>,
    >,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Berachain, Tr>)>,
{
    async fn do_fetch(c: &Berachain, fetch: Self) -> QueueMsg<RelayMessageTypes> {
        match fetch {
            Self::FetchIbcState(fetch) => data(id(c.chain_id(), fetch_ibc_state(c, fetch).await)),
            Self::FetchGetProof(fetch) => data(id(c.chain_id(), fetch_get_proof(c, fetch).await)),

            Self::FetchIbcAccountProof(FetchIbcAccountProof {
                ibc_contract_address,
                height,
                __marker: _,
            }) => {
                let account_proof = c
                    .provider
                    .get_proof(
                        ethers::types::H160::from(ibc_contract_address),
                        vec![],
                        Some(ethers::types::BlockId::Number(
                            c.execution_height_of_beacon_slot(height.revision_height)
                                .await
                                .into(),
                        )),
                    )
                    .await
                    .unwrap();

                data(id(
                    c.chain_id(),
                    Data::specific(IbcAccountProof {
                        height,
                        proof: AccountProof {
                            storage_root: account_proof.storage_hash.into(),
                            proof: account_proof
                                .account_proof
                                .into_iter()
                                .map(|x| x.to_vec())
                                .collect(),
                        },
                        __marker: PhantomData,
                    }),
                ))
            }

            Self::FetchLatestExecutionPayloadHeaderAbciProof(
                FetchLatestExecutionPayloadHeaderAbciProof { height, __marker },
            ) => {
                let query_result = c
                    .beacon_store_abci_query(
                        [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX],
                        height.revision_height,
                        true,
                    )
                    .await;

                let execution_header =
                    ExecutionPayloadHeader::<BerachainChainSpec>::decode_as::<Ssz>(
                        &query_result.response.value,
                    )
                    .unwrap();

                let proof = MerkleProof {
                    proofs: query_result
                        .response
                        .proof_ops
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| CommitmentProof::decode_as::<Proto>(op.data.as_slice()).unwrap())
                        .collect(),
                };

                data(id::<Berachain, Tr, _>(
                    c.chain_id(),
                    LatestExecutionPayloadHeaderAbciProof {
                        header: execution_header,
                        proof,
                    },
                ))
            }
            // TODO: Refactor cosmos & union to use the new cometbft_rpc::Client instead of tendermint_rpc, and then deduplicate these fetchers (they have been inlined from the fns in chain_impls::cosmos_sdk)
            Self::FetchTrustedCommit(FetchTrustedCommit {
                height,
                __marker: _,
            }) => {
                let commit = c
                    .tm_client
                    .commit(Some(height.revision_height().try_into().unwrap()))
                    .await
                    .unwrap();

                data(id::<Berachain, Tr, _>(
                    c.chain_id(),
                    Data::specific(TrustedCommit {
                        height,
                        // REVIEW: Ensure `commit.canonical`?
                        signed_header: commit.signed_header,
                        __marker: PhantomData,
                    }),
                ))
            }
            Self::FetchUntrustedCommit(FetchUntrustedCommit {
                height,
                __marker: _,
            }) => {
                let commit = c
                    .tm_client
                    .commit(Some(height.revision_height().try_into().unwrap()))
                    .await
                    .unwrap();

                data(id::<Berachain, Tr, _>(
                    c.chain_id(),
                    Data::specific(UntrustedCommit {
                        height,
                        // REVIEW: Ensure `commit.canonical`?
                        signed_header: commit.signed_header,
                        __marker: PhantomData,
                    }),
                ))
            }
            Self::FetchTrustedValidators(FetchTrustedValidators {
                height,
                __marker: _,
            }) => {
                let validators = c
                    .tm_client
                    .all_validators(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                data(id::<Berachain, Tr, _>(
                    c.chain_id(),
                    Data::specific(TrustedValidators {
                        height,
                        validators,
                        __marker: PhantomData,
                    }),
                ))
            }
            Self::FetchUntrustedValidators(FetchUntrustedValidators {
                height,
                __marker: _,
            }) => {
                let validators = c
                    .tm_client
                    .all_validators(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                data(id::<Berachain, Tr, _>(
                    c.chain_id(),
                    Data::specific(UntrustedValidators {
                        height,
                        validators,
                        __marker: PhantomData,
                    }),
                ))
            }
        }
    }
}

impl<Tr: ChainExt> DoMsg<Berachain, Tr> for Berachain
where
    Tr: ChainExt<
        SelfConsensusState: Encode<EthAbi>,
        SelfClientState: Encode<EthAbi>,
        Header: Encode<EthAbi>,
        StoredClientState<Berachain>: Encode<Tr::IbcStateEncoding>,
        StateProof: Encode<EthAbi>,
    >,
    Self::SelfClientState: Encode<Tr::IbcStateEncoding>,
{
    async fn msg(&self, msg: Effect<Berachain, Tr>) -> Result<(), Self::MsgError> {
        ethereum::do_msg(&self.ibc_handlers, msg, false).await
    }
}

impl<Tr: ChainExt> DoFetchState<Self, Tr> for Berachain
where
    Tr: ChainExt<SelfClientState: Decode<IbcStateEncodingOf<Berachain>> + Encode<EthAbi>>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Berachain, Tr>)>,
{
    fn state(hc: &Self, at: HeightOf<Self>, path: PathOf<Self, Tr>) -> QueueMsg<RelayMessageTypes> {
        fetch(id::<Self, Tr, _>(
            hc.chain_id(),
            Fetch::specific(FetchIbcState { path, height: at }),
        ))
    }

    async fn query_unfinalized_trusted_client_state(
        hc: &Self,
        client_id: Self::ClientId,
    ) -> Tr::SelfClientState {
        hc.ibc_handler()
            .ibc_state_read::<_, Berachain, Tr>(
                hc.provider.get_block_number().await.unwrap().as_u64(),
                ClientStatePath { client_id },
            )
            .await
            .unwrap()
    }
}

impl<Tr: ChainExt> DoFetchProof<Self, Tr> for Berachain
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Berachain, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Berachain, Tr>)>,
{
    fn proof(hc: &Self, at: HeightOf<Self>, path: PathOf<Self, Tr>) -> QueueMsg<RelayMessageTypes> {
        fetch(id::<Berachain, Tr, _>(
            hc.chain_id(),
            Fetch::specific(GetProof::<Berachain, Tr> { path, height: at }),
        ))
    }
}

impl<Tr: ChainExt> DoFetchUpdateHeaders<Self, Tr> for Berachain
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Berachain, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Berachain, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Berachain, Tr>)>,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        seq([
            wait(id(
                c.chain_id(),
                WaitForHeight::<Berachain, Tr> {
                    height: update_info.update_to,
                    __marker: PhantomData,
                },
            )),
            aggregate(
                [
                    fetch(id::<Self, Tr, _>(
                        c.chain_id(),
                        Fetch::specific(FetchIbcAccountProof {
                            ibc_contract_address: c.ibc_handler_address,
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Self, Tr, _>(
                        c.chain_id(),
                        Fetch::specific(FetchLatestExecutionPayloadHeaderAbciProof {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Self, Tr, _>(
                        c.chain_id(),
                        Fetch::specific(FetchTrustedCommit {
                            height: update_info.update_from.increment(),
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Self, Tr, _>(
                        c.chain_id(),
                        Fetch::specific(FetchUntrustedCommit {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Self, Tr, _>(
                        c.chain_id(),
                        Fetch::specific(FetchTrustedValidators {
                            height: update_info.update_from.increment(),
                            __marker: PhantomData,
                        }),
                    )),
                    fetch(id::<Self, Tr, _>(
                        c.chain_id(),
                        Fetch::specific(FetchUntrustedValidators {
                            height: update_info.update_to,
                            __marker: PhantomData,
                        }),
                    )),
                ],
                [],
                id::<Berachain, Tr, _>(
                    c.chain_id(),
                    Aggregate::specific(AggregateHeader { req: update_info }),
                ),
            ),
        ])
    }
}

impl<Tr> DoAggregate for Identified<Berachain, Tr, BerachainAggregate<Tr>>
where
    Tr: ChainExt,

    identified!(TrustedCommit<Berachain, Tr>): IsAggregateData,
    identified!(UntrustedCommit<Berachain, Tr>): IsAggregateData,
    identified!(TrustedValidators<Berachain, Tr>): IsAggregateData,
    identified!(UntrustedValidators<Berachain, Tr>): IsAggregateData,
    Identified<Berachain, Tr, LatestExecutionPayloadHeaderAbciProof>: IsAggregateData,
    Identified<Berachain, Tr, IbcAccountProof<Tr>>: IsAggregateData,

    Identified<Berachain, Tr, AggregateHeader<Berachain, Tr>>: UseAggregate<RelayMessageTypes>,

    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Berachain, Tr>)>,
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
            BerachainAggregate::AggregateHeader(data) => {
                do_aggregate(id(chain_id, data), aggregate_data)
            }
        }
    }
}

try_from_relayer_msg! {
    chain = Berachain,
    generics = (Tr: ChainExt),
    msgs = BerachainData(
        IbcAccountProof(IbcAccountProof<Tr>),
        LatestExecutionPayloadHeaderAbciProof(LatestExecutionPayloadHeaderAbciProof),
        TrustedCommit(TrustedCommit<Berachain, Tr>),
        UntrustedCommit(UntrustedCommit<Berachain, Tr>),
        TrustedValidators(TrustedValidators<Berachain, Tr>),
        UntrustedValidators(UntrustedValidators<Berachain, Tr>),
    ),
}
