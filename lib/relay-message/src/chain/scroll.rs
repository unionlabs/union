use std::{
    collections::{BTreeMap, VecDeque},
    marker::PhantomData,
};

use chain_utils::{
    ethereum::{EthereumConsensusChain, EthereumIbcChain, EthereumIbcChainExt, IbcHandlerExt},
    scroll::Scroll,
};
use ethers::{abi::AbiDecode, providers::Middleware};
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, effect, fetch, queue_msg, Op,
};
use unionlabs::{
    encoding::{Decode, Encode, EthAbi},
    hash::{H160, H256},
    ibc::{
        core::client::msg_update_client::MsgUpdateClient,
        lightclients::{
            ethereum::{account_proof::AccountProof, storage_proof::StorageProof},
            scroll,
        },
    },
    ics24::ClientStatePath,
    never::Never,
    traits::{Chain, ClientStateOf, HeightOf, IbcStateEncodingOf},
    uint::U256,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain::ethereum::{
        do_msg, fetch_get_proof, fetch_ibc_state, EthereumConfig, FetchIbcState, GetProof,
        TxSubmitError,
    },
    data::{AnyData, Data},
    effect::{AnyEffect, Effect, MsgUpdateClientData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified,
    use_aggregate::IsAggregateData,
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayMessage,
};

impl ChainExt for Scroll {
    type Data<Tr: ChainExt> = ScrollData<Tr>;
    type Fetch<Tr: ChainExt> = ScrollFetch<Tr>;
    type Aggregate<Tr: ChainExt> = ScrollAggregate<Tr>;

    type MsgError = TxSubmitError;

    type Config = EthereumConfig;
}

impl<Tr> DoMsg<Self, Tr> for Scroll
where
    ClientStateOf<Scroll>: Encode<Tr::IbcStateEncoding>,
    Tr: ChainExt<
        SelfConsensusState: Encode<EthAbi>,
        SelfClientState: Encode<EthAbi>,
        Header: Encode<EthAbi>,
        StoredClientState<Scroll>: Encode<Tr::IbcStateEncoding>,
        StateProof: Encode<EthAbi>,
    >,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Self, Tr>)>,
{
    async fn msg(&self, msg: Effect<Self, Tr>) -> Result<Op<RelayMessage>, Self::MsgError> {
        do_msg(
            self.chain_id(),
            self.multicall_address,
            &self.keyring,
            msg,
            true,
        )
        .await
    }
}

impl<Tr: ChainExt> DoFetchProof<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
{
    fn proof(c: &Self, at: HeightOf<Self>, path: PathOf<Scroll, Tr>) -> Op<RelayMessage> {
        fetch(id::<Self, Tr, _>(
            c.chain_id(),
            Fetch::<Self, Tr>::specific(GetProof { path, height: at }),
        ))
    }
}

// REVIEW: This can probably be generic over Hc: EthereumChain, instead of being duplicated between ethereum and scroll
impl<Tr> DoFetchState<Self, Tr> for Scroll
where
    Tr: ChainExt<SelfClientState: Decode<IbcStateEncodingOf<Scroll>> + Encode<EthAbi>>,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
{
    type QueryUnfinalizedTrustedClientStateError = Never;

    fn state(hc: &Self, at: HeightOf<Self>, path: PathOf<Self, Tr>) -> Op<RelayMessage> {
        fetch(id::<Self, Tr, _>(
            hc.chain_id(),
            Fetch::<Self, Tr>::specific(FetchIbcState { path, height: at }),
        ))
    }

    async fn query_unfinalized_trusted_client_state(
        hc: &Self,
        client_id: Self::ClientId,
    ) -> Result<Self::StoredClientState<Tr>, Self::QueryUnfinalizedTrustedClientStateError> {
        let latest_execution_height = hc.provider.get_block_number().await.unwrap().as_u64();

        Ok(hc
            .ibc_handler()
            .ibc_state_read::<_, Self, Tr>(latest_execution_height, ClientStatePath { client_id })
            .await
            .unwrap())
    }
}

impl<Tr> DoFetchUpdateHeaders<Self, Tr> for Scroll
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Scroll, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Scroll, Tr>)>,
    Tr: ChainExt,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> Op<RelayMessage> {
        // - scroll rollup contract root proof
        // - scroll latest batch index proof against rollup contract
        // - scroll finalized root at batch index against rollup contract
        // - ibc contract root against finalized root on L2
        // - commitBatch calldata and popped messages to verify timestamp

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
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchBatchHashProof {
                        height: update_info.update_to,
                        committed_batches_slot: c.rollup_committed_batches_slot,
                        rollup_contract_address: c.rollup_contract_address,
                    }),
                )),
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchCommitBatchTransactionInput {
                        height: update_info.update_to,
                        rollup_contract_address: c.rollup_contract_address,
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

impl<Tr> DoFetch<Scroll> for ScrollFetch<Tr>
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Scroll, Tr>)>,
    Tr: ChainExt<
        SelfClientState: Decode<IbcStateEncodingOf<Scroll>>,
        SelfConsensusState: Decode<IbcStateEncodingOf<Scroll>> + Encode<EthAbi>,
    >,
{
    type Error = Never;

    async fn do_fetch(scroll: &Scroll, msg: Self) -> Result<Op<RelayMessage>, Self::Error> {
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
                        vec![H256(latest_batch_index_slot.to_be_bytes()).into()],
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
                        key: U256::from_be_bytes(proof.key.to_fixed_bytes()),
                        value: proof.value.into(),
                        proof: proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
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

                let batch_hash_proof = scroll
                    .l1
                    .provider
                    .get_proof(
                        ethers::types::H160::from(rollup_contract_address),
                        vec![H256(
                            scroll_verifier::batch_index_mapping_key(
                                finalized_root_slot,
                                batch_index.into(),
                            )
                            .to_be_bytes(),
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

                let proof = match <[_; 1]>::try_from(batch_hash_proof.storage_proof) {
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
                        key: U256::from_be_bytes(proof.key.to_fixed_bytes()),
                        value: proof.value.into(),
                        proof: proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
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
            Self::FetchBatchHashProof(FetchBatchHashProof {
                height,
                committed_batches_slot,
                rollup_contract_address,
            }) => {
                let batch_index = scroll
                    .batch_index_of_beacon_slot(height.revision_height)
                    .await;

                let batch_hash_proof = scroll
                    .l1
                    .provider
                    .get_proof(
                        ethers::types::H160::from(rollup_contract_address),
                        vec![H256(
                            scroll_verifier::batch_index_mapping_key(
                                committed_batches_slot,
                                batch_index.into(),
                            )
                            .to_be_bytes(),
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

                let proof = match <[_; 1]>::try_from(batch_hash_proof.storage_proof) {
                    Ok([proof]) => proof,
                    Err(invalid) => {
                        panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                    }
                };

                Data::specific(BatchHashProof {
                    height,
                    batch_index,
                    proof: StorageProof {
                        key: U256::from_be_bytes(proof.key.to_fixed_bytes()),
                        value: proof.value.into(),
                        proof: proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
                    },
                    __marker: PhantomData,
                })
            }
            ScrollFetch::FetchCommitBatchTransactionInput(FetchCommitBatchTransactionInput {
                height,
                // TODO: This needs to be passed to `scroll_codec::fetch_l1_message_hashes`
                rollup_contract_address: _,
            }) => {
                let batch_index = scroll
                    .batch_index_of_beacon_slot(height.revision_height)
                    .await;

                let batch = scroll.scroll_api_client.batch(batch_index).await;

                let tx = scroll
                    .l1
                    .provider
                    .get_transaction(batch.batch.commit_tx_hash)
                    .await
                    .unwrap()
                    .unwrap();
                let calldata = tx.input.to_vec();

                let blob_versioned_hash = tx
                    .blob_versioned_hashes
                    .unwrap_or_default()
                    .first()
                    .map(|x| H256(x.0));

                let message_hashes = scroll_codec::fetch_l1_message_hashes(
                    scroll.l1.provider.as_ref(),
                    scroll
                        .l1
                        .execution_height_of_beacon_slot(height.revision_height)
                        .await,
                    <scroll_codec::CommitBatchCall as AbiDecode>::decode(&calldata).unwrap(),
                )
                .await
                .unwrap();

                Data::specific(CommitBatchTransactionInput {
                    height,
                    batch_index,
                    message_hashes,
                    calldata,
                    blob_versioned_hash,
                    __marker: PhantomData,
                })
            }
        };

        Ok(data(id::<Scroll, Tr, _>(scroll.chain_id, msg)))
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ScrollFetch<Tr: ChainExt> {
    FetchGetProof(GetProof<Scroll, Tr>),
    FetchIbcState(FetchIbcState<Scroll, Tr>),

    // - scroll rollup contract root proof
    FetchRollupContractRootProof(FetchRollupContractRootProof),
    // - scroll latest batch index proof against rollup contract
    FetchLatestBatchIndexProof(FetchLatestBatchIndexProof),
    // - scroll finalized root at batch index against rollup contract
    FetchScrollFinalizedRootProof(FetchScrollFinalizedRootProof),
    // - ibc contract root against finalized root on L2
    FetchIbcContractRootProof(FetchIbcContractRootProof),
    FetchBatchHashProof(FetchBatchHashProof),
    FetchCommitBatchTransactionInput(FetchCommitBatchTransactionInput),
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
pub struct FetchBatchHashProof {
    pub height: HeightOf<Scroll>,
    pub committed_batches_slot: U256,
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
pub struct FetchCommitBatchTransactionInput {
    // the height to update to
    pub height: HeightOf<Scroll>,
    pub rollup_contract_address: H160,
}

#[queue_msg]
pub struct FetchIbcContractRootProof {
    // the height to update to
    pub height: HeightOf<Scroll>,
    pub ibc_contract_address: H160,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ScrollData<Tr: ChainExt> {
    RollupContractRootProof(RollupContractRootProof<Tr>),
    LatestBatchIndexProof(LatestBatchIndexProof<Tr>),
    ScrollFinalizedRootProof(ScrollFinalizedRootProof<Tr>),
    IbcContractRootProof(IbcContractRootProof<Tr>),
    BatchHashProof(BatchHashProof<Tr>),
    CommitBatchTransactionInput(CommitBatchTransactionInput<Tr>),
}

try_from_relayer_msg! {
    chain = Scroll,
    generics = (Tr: ChainExt),
    msgs = ScrollData(
        RollupContractRootProof(RollupContractRootProof<Tr>),
        LatestBatchIndexProof(LatestBatchIndexProof<Tr>),
        ScrollFinalizedRootProof(ScrollFinalizedRootProof<Tr>),
        IbcContractRootProof(IbcContractRootProof<Tr>),
        BatchHashProof(BatchHashProof<Tr>),
        CommitBatchTransactionInput(CommitBatchTransactionInput<Tr>),
    ),
}

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
pub struct BatchHashProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub batch_index: u64,
    pub proof: StorageProof,
}

#[queue_msg]
pub struct ScrollFinalizedRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub batch_index: u64,
    // TODO: Remove this field as it is present in proof[0].value
    pub finalized_state_root: U256,
    pub proof: StorageProof,
}

#[queue_msg]
pub struct IbcContractRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub proof: AccountProof,
}

#[queue_msg]
pub struct CommitBatchTransactionInput<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Scroll>,
    pub batch_index: u64,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub calldata: Vec<u8>,
    #[serde(with = "::serde_utils::map_numeric_keys_as_string")]
    pub message_hashes: BTreeMap<u64, H256>,
    pub blob_versioned_hash: Option<H256>,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ScrollAggregate<Tr: ChainExt> {
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
    Identified<Scroll, Tr, BatchHashProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, CommitBatchTransactionInput<Tr>>: IsAggregateData,

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
    ) -> Op<RelayMessage> {
        match t {
            ScrollAggregate::AggregateHeader(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

impl<Tr> UseAggregate<RelayMessage> for Identified<Scroll, Tr, AggregateHeader<Tr>>
where
    Tr: ChainExt,
    Identified<Scroll, Tr, RollupContractRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, LatestBatchIndexProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, ScrollFinalizedRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, IbcContractRootProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, BatchHashProof<Tr>>: IsAggregateData,
    Identified<Scroll, Tr, CommitBatchTransactionInput<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Scroll>)>,
{
    type AggregatedData = HList![
        Identified<Scroll, Tr, RollupContractRootProof<Tr>>,
        Identified<Scroll, Tr, LatestBatchIndexProof<Tr>>,
        Identified<Scroll, Tr, ScrollFinalizedRootProof<Tr>>,
        Identified<Scroll, Tr, IbcContractRootProof<Tr>>,
        Identified<Scroll, Tr, BatchHashProof<Tr>>,
        Identified<Scroll, Tr, CommitBatchTransactionInput<Tr>>,
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
                    height: rollup_contract_root_proof_height,
                    proof: rollup_contract_root_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: latest_batch_index_proof_chain_id,
                t: LatestBatchIndexProof {
                    height: latest_batch_index_proof_height,
                    latest_batch_index,
                    proof: latest_batch_index_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: scroll_finalized_root_proof_chain_id,
                t: ScrollFinalizedRootProof {
                    height: scroll_finalized_root_proof_height,
                    batch_index: scroll_finalized_root_proof_batch_index,
                    finalized_state_root,
                    proof: scroll_finalized_root_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: ibc_contract_root_proof_chain_id,
                t: IbcContractRootProof {
                    height: ibc_contract_root_proof_height,
                    proof: ibc_contract_account_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: batch_hash_proof_chain_id,
                t: BatchHashProof {
                    height: batch_hash_proof_height,
                    batch_index: batch_hash_proof_batch_index,
                    proof: batch_hash_proof,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: commit_batch_transaction_input_chain_id,
                t: CommitBatchTransactionInput {
                    height: commit_batch_transaction_input_height,
                    batch_index: commit_batch_transaction_input_batch_index,
                    calldata,
                    message_hashes,
                    blob_versioned_hash,
                    __marker,
                },
                __marker: _,
            }
        ]: Self::AggregatedData,
    ) -> Op<RelayMessage> {
        assert_eq!(rollup_contract_root_proof_chain_id, chain_id);
        assert_eq!(latest_batch_index_proof_chain_id, chain_id);
        assert_eq!(scroll_finalized_root_proof_chain_id, chain_id);
        assert_eq!(ibc_contract_root_proof_chain_id, chain_id);
        assert_eq!(batch_hash_proof_chain_id, chain_id);
        assert_eq!(commit_batch_transaction_input_chain_id, chain_id);

        assert_eq!(
            rollup_contract_root_proof_height,
            latest_batch_index_proof_height
        );
        assert_eq!(
            rollup_contract_root_proof_height,
            scroll_finalized_root_proof_height
        );
        assert_eq!(
            rollup_contract_root_proof_height,
            ibc_contract_root_proof_height
        );
        assert_eq!(rollup_contract_root_proof_height, batch_hash_proof_height);
        assert_eq!(
            rollup_contract_root_proof_height,
            commit_batch_transaction_input_height
        );

        assert_eq!(
            scroll_finalized_root_proof_batch_index,
            batch_hash_proof_batch_index
        );
        assert_eq!(
            scroll_finalized_root_proof_batch_index,
            commit_batch_transaction_input_batch_index
        );

        effect(id::<Tr, Scroll, _>(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id,
                client_message: scroll::header::Header {
                    l1_height: req.update_to,
                    l1_account_proof: rollup_contract_root_proof,
                    l2_state_root: H256(finalized_state_root.to_be_bytes()),
                    l2_state_proof: scroll_finalized_root_proof,
                    last_batch_index: latest_batch_index,
                    last_batch_index_proof: latest_batch_index_proof,
                    l2_ibc_account_proof: ibc_contract_account_proof,
                    batch_hash_proof,
                    commit_batch_calldata: calldata,
                    l1_message_hashes: message_hashes,
                    blob_versioned_hash,
                },
            }),
        ))
    }
}
