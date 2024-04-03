use std::{collections::VecDeque, marker::PhantomData};

use chain_utils::{
    ethereum::{EthereumChain, IbcHandlerExt},
    scroll::Scroll,
};
use enumorph::Enumorph;
use ethers::providers::Middleware;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, effect, fetch, queue_msg, QueueMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Decode, Encode, EthAbi},
    hash::{H160, H256},
    ibc::{
        core::client::{height::IsHeight, msg_update_client::MsgUpdateClient},
        lightclients::{
            ethereum::{account_proof::AccountProof, storage_proof::StorageProof},
            scroll,
        },
    },
    proof::ClientStatePath,
    traits::{Chain, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf, IbcStateEncodingOf},
    uint::U256,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::ethereum::{
        do_msg, fetch_get_proof, fetch_ibc_state, EthereumConfig, FetchIbcState, GetProof,
        TxSubmitError,
    },
    data::{AnyData, Data},
    effect::{AnyEffect, Effect, MsgUpdateClientData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified,
    use_aggregate::IsAggregateData,
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayMessageTypes,
};

impl ChainExt for Scroll {
    type Data<Tr: ChainExt> = ScrollData<Tr>;
    type Fetch<Tr: ChainExt> = ScrollFetch<Tr>;
    type Aggregate<Tr: ChainExt> = ScrollAggregate<Tr>;

    type MsgError = TxSubmitError;

    type Config = EthereumConfig;
}

impl<Tr: ChainExt> DoMsg<Self, Tr> for Scroll
where
    ConsensusStateOf<Tr>: Encode<EthAbi>,
    ClientStateOf<Tr>: Encode<EthAbi>,
    HeaderOf<Tr>: Encode<EthAbi>,

    ClientStateOf<Scroll>: Encode<Tr::IbcStateEncoding>,
    Tr::StoredClientState<Scroll>: Encode<Tr::IbcStateEncoding>,
    Tr::StateProof: Encode<EthAbi>,
{
    async fn msg(&self, msg: Effect<Self, Tr>) -> Result<(), Self::MsgError> {
        do_msg(&self.ibc_handlers, msg).await
    }
}

impl<Tr: ChainExt> DoFetchProof<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
{
    fn proof(
        c: &Self,
        at: HeightOf<Self>,
        path: PathOf<Scroll, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        fetch(id::<Self, Tr, _>(
            c.chain_id(),
            Fetch::<Self, Tr>::specific(GetProof { path, height: at }),
        ))
    }
}

// REVIEW: This can probably be generic over Hc: EthereumChain, instead of being duplicated between ethereum and scroll
impl<Tr: ChainExt> DoFetchState<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
    Tr::SelfClientState: Decode<IbcStateEncodingOf<Scroll>>,

    Tr::SelfClientState: Encode<EthAbi>,
{
    fn state(
        hc: &Self,
        at: HeightOf<Self>,
        path: PathOf<Scroll, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        fetch(id::<Self, Tr, _>(
            hc.chain_id(),
            Fetch::<Self, Tr>::specific(FetchIbcState { path, height: at }),
        ))
    }

    async fn query_client_state(
        hc: &Self,
        client_id: Self::ClientId,
        height: Self::Height,
    ) -> Tr::SelfClientState {
        hc.ibc_handler()
            .ibc_state_read::<_, Scroll, Tr>(
                hc.execution_height_of_beacon_slot(height.revision_height())
                    .await,
                ClientStatePath { client_id },
            )
            .await
            .unwrap()
    }
}

impl<Tr: ChainExt> DoFetchUpdateHeaders<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Scroll, Tr>)>,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        // - scroll rollup contract root proof
        // - scroll latest batch index proof against rollup contract
        // - scroll finalized root at batch index against rollup contract
        // - ibc contract root against finalized root on L2

        aggregate(
            [
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchRollupContractRootProof {
                        height: update_info.update_to,
                        rollup_contract_address: c.rollup_contract_address,
                    }),
                )),
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchLatestBatchIndexProof {
                        height: update_info.update_to,
                        latest_batch_index_slot: c.rollup_last_finalized_batch_index_slot,
                        rollup_contract_address: c.rollup_contract_address,
                    }),
                )),
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchScrollFinalizedRootProof {
                        height: update_info.update_to,
                        finalized_root_slot: c.rollup_finalized_state_roots_slot,
                        rollup_contract_address: c.rollup_contract_address,
                    }),
                )),
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchIbcContractRootProof {
                        height: update_info.update_to,
                        ibc_contract_address: c.ibc_handler_address,
                    }),
                )),
            ],
            [],
            id(
                c.chain_id(),
                Aggregate::<Scroll, Tr>::specific(AggregateHeader { req: update_info }),
            ),
        )
    }
}

impl<Tr: ChainExt> DoFetch<Scroll> for ScrollFetch<Tr>
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Scroll, Tr>)>,

    Tr::SelfClientState: Decode<IbcStateEncodingOf<Scroll>>,
    Tr::SelfConsensusState: Decode<IbcStateEncodingOf<Scroll>>,

    Tr::SelfClientState: Encode<EthAbi>,
{
    async fn do_fetch(scroll: &Scroll, msg: Self) -> QueueMsg<RelayMessageTypes> {
        let msg = match msg {
            Self::FetchGetProof(get_proof) => fetch_get_proof(scroll, get_proof).await,
            Self::FetchIbcState(ibc_state) => fetch_ibc_state(scroll, ibc_state).await,
            Self::FetchRollupContractRootProof(FetchRollupContractRootProof {
                height,
                rollup_contract_address,
            }) => {
                let account_proof = scroll
                    .l1
                    .provider()
                    .get_proof(
                        ethers::types::H160::from(rollup_contract_address),
                        vec![],
                        Some(ethers::types::BlockId::Number(
                            scroll
                                .l1
                                .execution_height_of_beacon_slot(height.revision_height)
                                .await
                                .into(),
                        )),
                    )
                    .await
                    .unwrap();

                Data::specific(RollupContractRootProof {
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
                })
            }
            Self::FetchLatestBatchIndexProof(FetchLatestBatchIndexProof {
                height,
                latest_batch_index_slot,
                rollup_contract_address,
            }) => {
                let latest_batch_index_proof = scroll
                    .l1
                    .provider
                    .get_proof(
                        ethers::types::H160::from(rollup_contract_address),
                        vec![H256(latest_batch_index_slot.to_big_endian()).into()],
                        Some(ethers::types::BlockId::Number(
                            scroll
                                .l1
                                .execution_height_of_beacon_slot(height.revision_height)
                                .await
                                .into(),
                        )),
                    )
                    .await
                    .unwrap();

                let proof = match <[_; 1]>::try_from(latest_batch_index_proof.storage_proof) {
                    Ok([proof]) => proof,
                    Err(invalid) => {
                        panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                    }
                };

                Data::specific(LatestBatchIndexProof {
                    height,
                    // REVIEW: is latest_batch_index a u64 or U256?
                    latest_batch_index: proof.value.try_into().unwrap(),
                    proof: StorageProof {
                        proofs: [unionlabs::ibc::lightclients::ethereum::proof::Proof {
                            key: U256::from_big_endian(proof.key.to_fixed_bytes()),
                            value: proof.value.into(),
                            proof: proof
                                .proof
                                .into_iter()
                                .map(|bytes| bytes.to_vec())
                                .collect(),
                        }]
                        .to_vec(),
                    },
                    __marker: PhantomData,
                })
            }
            Self::FetchScrollFinalizedRootProof(FetchScrollFinalizedRootProof {
                height,
                finalized_root_slot,
                rollup_contract_address,
            }) => {
                let batch_index = scroll
                    .batch_index_of_beacon_slot(height.revision_height)
                    .await;

                let finalized_root_proof = scroll
                    .l1
                    .provider
                    .get_proof(
                        ethers::types::H160::from(rollup_contract_address),
                        vec![H256(
                            scroll_verifier::verify::finalized_state_root_key(
                                finalized_root_slot,
                                batch_index.into(),
                            )
                            .to_big_endian(),
                        )
                        .into()],
                        Some(ethers::types::BlockId::Number(
                            scroll
                                .l1
                                .execution_height_of_beacon_slot(height.revision_height)
                                .await
                                .into(),
                        )),
                    )
                    .await
                    .unwrap();

                let proof = match <[_; 1]>::try_from(finalized_root_proof.storage_proof) {
                    Ok([proof]) => proof,
                    Err(invalid) => {
                        panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                    }
                };

                Data::specific(ScrollFinalizedRootProof {
                    height,
                    batch_index,
                    finalized_state_root: proof.value.into(),
                    proof: StorageProof {
                        proofs: [unionlabs::ibc::lightclients::ethereum::proof::Proof {
                            key: U256::from_big_endian(proof.key.to_fixed_bytes()),
                            value: proof.value.into(),
                            proof: proof
                                .proof
                                .into_iter()
                                .map(|bytes| bytes.to_vec())
                                .collect(),
                        }]
                        .to_vec(),
                    },
                    __marker: PhantomData,
                })
            }
            Self::FetchIbcContractRootProof(FetchIbcContractRootProof {
                height,
                ibc_contract_address,
            }) => {
                let batch_index = scroll
                    .batch_index_of_beacon_slot(height.revision_height)
                    .await;

                let batch = scroll.scroll_api_client.batch(batch_index).await;

                let proof = scroll
                    .scroll_rpc
                    .get_proof(
                        ibc_contract_address,
                        [],
                        scroll_rpc::BlockId::Number(batch.batch.end_block_number),
                    )
                    .await
                    .unwrap();

                Data::specific(IbcContractRootProof {
                    height,
                    proof: AccountProof {
                        storage_root: proof.storage_hash,
                        proof: proof
                            .account_proof
                            .into_iter()
                            .map(|x| x.to_vec())
                            .collect(),
                    },
                    __marker: PhantomData,
                })
            }
        };

        data(id::<Scroll, Tr, _>(scroll.chain_id, msg))
    }
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "")
)]
pub enum ScrollFetch<Tr: ChainExt> {
    #[display(fmt = "GetProof::{}", "_0.path")]
    FetchGetProof(GetProof<Scroll, Tr>),
    #[display(fmt = "IbcState::{}", "_0.path")]
    FetchIbcState(FetchIbcState<Scroll, Tr>),

    // - scroll rollup contract root proof
    #[display(fmt = "FetchRollupContractRootProof")]
    FetchRollupContractRootProof(FetchRollupContractRootProof),
    // - scroll latest batch index proof against rollup contract
    #[display(fmt = "FetchLatestBatchIndexProof")]
    FetchLatestBatchIndexProof(FetchLatestBatchIndexProof),
    // - scroll finalized root at batch index against rollup contract
    #[display(fmt = "FetchScrollFinalizedRootProof")]
    FetchScrollFinalizedRootProof(FetchScrollFinalizedRootProof),
    // - ibc contract root against finalized root on L2
    #[display(fmt = "FetchIbcContractRootProof")]
    FetchIbcContractRootProof(FetchIbcContractRootProof),
}

#[queue_msg]
pub struct FetchRollupContractRootProof {
    // the height to update to
    pub height: HeightOf<Scroll>,
    pub rollup_contract_address: H160,
}

#[queue_msg]
pub struct FetchLatestBatchIndexProof {
    // the height to update to
    pub height: HeightOf<Scroll>,
    pub latest_batch_index_slot: U256,
    pub rollup_contract_address: H160,
}

#[queue_msg]
pub struct FetchScrollFinalizedRootProof {
    // the height to update to
    pub height: HeightOf<Scroll>,
    pub finalized_root_slot: U256,
    pub rollup_contract_address: H160,
}

#[queue_msg]
pub struct FetchIbcContractRootProof {
    // the height to update to
    pub height: HeightOf<Scroll>,
    pub ibc_contract_address: H160,
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "")
)]
pub enum ScrollData<Tr: ChainExt> {
    #[display(fmt = "RollupContractRootProof")]
    RollupContractRootProof(RollupContractRootProof<Tr>),
    #[display(fmt = "LatestBatchIndexProof")]
    LatestBatchIndexProof(LatestBatchIndexProof<Tr>),
    #[display(fmt = "ScrollFinalizedRootProof")]
    ScrollFinalizedRootProof(ScrollFinalizedRootProof<Tr>),
    #[display(fmt = "IbcContractRootProof")]
    IbcContractRootProof(IbcContractRootProof<Tr>),
}

const _: () = {
    try_from_relayer_msg! {
        chain = Scroll,
        generics = (Tr: ChainExt),
        msgs = ScrollData(
            RollupContractRootProof(RollupContractRootProof<Tr>),
            LatestBatchIndexProof(LatestBatchIndexProof<Tr>),
            ScrollFinalizedRootProof(ScrollFinalizedRootProof<Tr>),
            IbcContractRootProof(IbcContractRootProof<Tr>),
        ),
    }
};

#[queue_msg]
pub struct RollupContractRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub proof: AccountProof,
}

#[queue_msg]
pub struct LatestBatchIndexProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub latest_batch_index: u64,
    pub proof: StorageProof,
}

#[queue_msg]
pub struct ScrollFinalizedRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub batch_index: u64,
    pub finalized_state_root: U256,
    pub proof: StorageProof,
}

#[queue_msg]
pub struct IbcContractRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub proof: AccountProof,
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "")
)]
pub enum ScrollAggregate<Tr: ChainExt> {
    #[display(fmt = "AggregateHeader")]
    AggregateHeader(AggregateHeader<Tr>),
}

#[queue_msg]
pub struct AggregateHeader<Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Scroll, Tr>,
}

impl<Tr: ChainExt> DoAggregate for Identified<Scroll, Tr, ScrollAggregate<Tr>>
where
    Identified<Scroll, Tr, RollupContractRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, LatestBatchIndexProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, ScrollFinalizedRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, IbcContractRootProof<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Scroll>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Scroll, Tr>)>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t,
            __marker,
        }: Self,
        data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes> {
        match t {
            ScrollAggregate::AggregateHeader(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

impl<Tr> UseAggregate<RelayMessageTypes> for Identified<Scroll, Tr, AggregateHeader<Tr>>
where
    Tr: ChainExt,
    Identified<Scroll, Tr, RollupContractRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, LatestBatchIndexProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, ScrollFinalizedRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, IbcContractRootProof<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Scroll>)>,
{
    type AggregatedData = HList![
        Identified<Scroll, Tr, RollupContractRootProof<Tr>>,
        Identified<Scroll, Tr, LatestBatchIndexProof<Tr>>,
        Identified<Scroll, Tr, ScrollFinalizedRootProof<Tr>>,
        Identified<Scroll, Tr, IbcContractRootProof<Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            t: AggregateHeader { req },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: rollup_contract_root_proof_chain_id,
                t: RollupContractRootProof {
                    height: _rollup_contract_root_proof_height,
                    proof: rollup_contract_root_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: latest_batch_index_proof_chain_id,
                t: LatestBatchIndexProof {
                    height: _latest_batch_index_proof_height,
                    latest_batch_index,
                    proof: latest_batch_index_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: scroll_finalized_root_proof_chain_id,
                t: ScrollFinalizedRootProof {
                    height: _scroll_finalized_root_proof_height,
                    batch_index: _batch_index,
                    finalized_state_root,
                    proof: finalized_root_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: ibc_contract_root_proof_chain_id,
                t: IbcContractRootProof {
                    height: _ibc_contract_root_proof_height,
                    proof: ibc_contract_account_proof,
                    __marker: _
                },
                __marker: _,
            }
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(rollup_contract_root_proof_chain_id, chain_id);
        assert_eq!(latest_batch_index_proof_chain_id, chain_id);
        assert_eq!(scroll_finalized_root_proof_chain_id, chain_id);
        assert_eq!(ibc_contract_root_proof_chain_id, chain_id);

        effect(id::<Tr, Scroll, _>(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id,
                client_message: scroll::header::Header {
                    l1_height: req.update_to,
                    l1_account_proof: rollup_contract_root_proof,
                    l2_state_root: H256(finalized_state_root.to_big_endian()),
                    l2_state_proof: finalized_root_proof,
                    last_batch_index: latest_batch_index,
                    last_batch_index_proof: latest_batch_index_proof,
                    l2_ibc_account_proof: ibc_contract_account_proof,
                },
            }),
        ))
    }
}
