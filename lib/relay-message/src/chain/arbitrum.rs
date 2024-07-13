use std::{collections::VecDeque, marker::PhantomData};

use chain_utils::{
    arbitrum::Arbitrum,
    ethereum::{EthereumConsensusChain, EthereumIbcChain, EthereumIbcChainExt, IbcHandlerExt},
};
use ethers::providers::Middleware;
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, effect, fetch, queue_msg, Op,
};
use unionlabs::{
    encoding::{Decode, Encode, EthAbi},
    hash::H160,
    ibc::{
        core::client::msg_update_client::MsgUpdateClient,
        lightclients::{
            arbitrum,
            ethereum::{account_proof::AccountProof, storage_proof::StorageProof},
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

impl ChainExt for Arbitrum {
    type Data<Tr: ChainExt> = ArbitrumData<Tr>;
    type Fetch<Tr: ChainExt> = ArbitrumFetch<Tr>;
    type Aggregate<Tr: ChainExt> = ArbitrumAggregate<Tr>;

    type MsgError = TxSubmitError;

    type Config = EthereumConfig;
}

impl<Tr> DoMsg<Self, Tr> for Arbitrum
where
    ClientStateOf<Arbitrum>: Encode<Tr::IbcStateEncoding>,
    Tr: ChainExt<
        SelfConsensusState: Encode<EthAbi>,
        SelfClientState: Encode<EthAbi>,
        Header: Encode<EthAbi>,
        StoredClientState<Arbitrum>: Encode<Tr::IbcStateEncoding>,
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
            false,
            None,
        )
        .await
    }
}

impl<Tr: ChainExt> DoFetchProof<Self, Tr> for Arbitrum
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Arbitrum, Tr>)>,
{
    fn proof(c: &Self, at: HeightOf<Self>, path: PathOf<Arbitrum, Tr>) -> Op<RelayMessage> {
        fetch(id::<Self, Tr, _>(
            c.chain_id(),
            Fetch::<Self, Tr>::specific(GetProof { path, height: at }),
        ))
    }
}

// REVIEW: This can probably be generic over Hc: EthereumChain, instead of being duplicated between ethereum and arbitrum
impl<Tr> DoFetchState<Self, Tr> for Arbitrum
where
    Tr: ChainExt<SelfClientState: Decode<IbcStateEncodingOf<Self>> + Encode<EthAbi>>,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Self, Tr>)>,
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

impl<Tr> DoFetchUpdateHeaders<Self, Tr> for Arbitrum
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Arbitrum, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Arbitrum, Tr>)>,
    Tr: ChainExt,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> Op<RelayMessage> {
        aggregate(
            [
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchL1ContractRootProof {
                        height: update_info.update_to,
                        l1_contract_address: c.l1_contract_address,
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
                    Fetch::specific(FetchLatestConfirmedProofs {
                        height: update_info.update_to,
                        l1_latest_confirmed_slot: c.l1_next_node_num_slot,
                        l1_nodes_slot: c.l1_nodes_slot,
                        l1_contract_address: c.l1_contract_address,
                    }),
                )),
                fetch(id(
                    c.chain_id(),
                    Fetch::specific(FetchL2Header {
                        height: update_info.update_to,
                    }),
                )),
            ],
            [],
            id(
                c.chain_id(),
                Aggregate::<Arbitrum, Tr>::specific(AggregateHeader { req: update_info }),
            ),
        )
    }
}

impl<Tr> DoFetch<Arbitrum> for ArbitrumFetch<Tr>
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Arbitrum, Tr>)>,
    Tr: ChainExt<
        SelfClientState: Decode<IbcStateEncodingOf<Arbitrum>>,
        SelfConsensusState: Decode<IbcStateEncodingOf<Arbitrum>> + Encode<EthAbi>,
    >,
{
    type Error = Never;

    async fn do_fetch(arbitrum: &Arbitrum, msg: Self) -> Result<Op<RelayMessage>, Self::Error> {
        let msg = match msg {
            Self::FetchGetProof(get_proof) => fetch_get_proof(arbitrum, get_proof).await,
            Self::FetchIbcState(ibc_state) => fetch_ibc_state(arbitrum, ibc_state).await,
            Self::FetchL1ContractRootProof(FetchL1ContractRootProof {
                height,
                l1_contract_address,
            }) => {
                let account_proof = arbitrum
                    .l1
                    .provider()
                    .get_proof(
                        ethers::types::H160::from(l1_contract_address),
                        vec![],
                        Some(ethers::types::BlockId::Number(
                            arbitrum
                                .l1
                                .execution_height_of_beacon_slot(height.revision_height)
                                .await
                                .into(),
                        )),
                    )
                    .await
                    .unwrap();

                Data::specific(L1ContractRootProof {
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
            Self::FetchIbcContractRootProof(FetchIbcContractRootProof {
                height,
                ibc_contract_address,
            }) => {
                let arbitrum_height = arbitrum
                    .execution_height_of_beacon_slot(height.revision_height)
                    .await;

                let proof = arbitrum
                    .provider
                    .get_proof(
                        ethers::types::H160(ibc_contract_address.0),
                        vec![],
                        Some(ethers::types::BlockNumber::Number(arbitrum_height.into()).into()),
                    )
                    .await
                    .unwrap();

                Data::specific(IbcContractRootProof {
                    height,
                    proof: AccountProof {
                        storage_root: proof.storage_hash.0.into(),
                        proof: proof
                            .account_proof
                            .into_iter()
                            .map(|x| x.to_vec())
                            .collect(),
                    },
                    __marker: PhantomData,
                })
            }
            Self::FetchLatestConfirmedProofs(FetchLatestConfirmedProofs {
                height,
                l1_latest_confirmed_slot,
                l1_nodes_slot,
                l1_contract_address,
            }) => {
                let l1_height = arbitrum
                    .l1
                    .execution_height_of_beacon_slot(height.revision_height)
                    .await;

                let latest_confirmed = arbitrum
                    .next_node_num_at_beacon_slot(height.revision_height)
                    .await;

                let [latest_confirmed_slot_proof, nodes_slot_proof] = arbitrum
                    .l1
                    .provider()
                    .get_proof(
                        ethers::types::H160(l1_contract_address.0),
                        vec![
                            l1_latest_confirmed_slot.to_be_bytes().into(),
                            arbitrum_verifier::nodes_confirm_data_mapping_key(
                                l1_nodes_slot,
                                latest_confirmed,
                                arbitrum.l1_nodes_confirm_data_offset,
                            )
                            .to_be_bytes()
                            .into(),
                        ],
                        Some(ethers::types::BlockNumber::Number(l1_height.into()).into()),
                    )
                    .await
                    .unwrap()
                    .storage_proof
                    .try_into()
                    .unwrap();

                Data::specific(LatestConfirmedProofs {
                    height,
                    latest_confirmed,
                    // TODO: Extract this logic into a fn, we do it all over the place
                    latest_confirmed_slot_proof: StorageProof {
                        key: U256::from_be_bytes(latest_confirmed_slot_proof.key.0),
                        value: latest_confirmed_slot_proof.value.into(),
                        proof: latest_confirmed_slot_proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
                    },
                    nodes_slot_proof: StorageProof {
                        key: U256::from_be_bytes(nodes_slot_proof.key.0),
                        value: nodes_slot_proof.value.into(),
                        proof: nodes_slot_proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
                    },
                    __marker: PhantomData,
                })
            }
            Self::FetchL2Header(FetchL2Header { height }) => {
                let arbitrum_height = arbitrum
                    .execution_height_of_beacon_slot(height.revision_height)
                    .await;

                let block = arbitrum
                    .provider
                    .get_block(ethers::types::BlockNumber::Number(arbitrum_height.into()))
                    .await
                    .unwrap()
                    .unwrap();

                let l2_header = arbitrum::l2_header::L2Header {
                    parent_hash: block.parent_hash.0.into(),
                    sha3_uncles: block.uncles_hash.0.into(),
                    miner: block.author.unwrap().0.into(),
                    state_root: block.state_root.0.into(),
                    transactions_root: block.transactions_root.0.into(),
                    receipts_root: block.receipts_root.0.into(),
                    logs_bloom: Box::new(block.logs_bloom.unwrap().0.into()),
                    difficulty: block.difficulty.into(),
                    number: block.number.unwrap().as_u64().into(),
                    gas_limit: block.gas_limit.as_u64(),
                    gas_used: block.gas_used.as_u64(),
                    timestamp: block.timestamp.as_u64(),
                    extra_data: block.extra_data.try_into().unwrap(),
                    mix_hash: block.mix_hash.unwrap().0.into(),
                    nonce: block.nonce.unwrap().0.into(),
                    base_fee_per_gas: block.base_fee_per_gas.unwrap().into(),
                };

                Data::specific(L2Header {
                    height,
                    l2_header,
                    __marker: PhantomData,
                })
            }
        };

        Ok(data(id::<Arbitrum, Tr, _>(arbitrum.chain_id(), msg)))
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ArbitrumFetch<Tr: ChainExt> {
    FetchGetProof(GetProof<Arbitrum, Tr>),
    FetchIbcState(FetchIbcState<Arbitrum, Tr>),

    // - arbitrum rollup contract root proof
    FetchL1ContractRootProof(FetchL1ContractRootProof),
    // - ibc contract root against finalized root on L2
    FetchIbcContractRootProof(FetchIbcContractRootProof),
    /// Fetch the latest confirmed node and the relevant proofs.
    FetchLatestConfirmedProofs(FetchLatestConfirmedProofs),
    /// Fetch the Arbitrum header.
    FetchL2Header(FetchL2Header),
}

#[queue_msg]
pub struct FetchL1ContractRootProof {
    // the height to update to
    pub height: HeightOf<Arbitrum>,
    pub l1_contract_address: H160,
}

#[queue_msg]
pub struct FetchIbcContractRootProof {
    // the height to update to
    pub height: HeightOf<Arbitrum>,
    pub ibc_contract_address: H160,
}

#[queue_msg]
pub struct FetchLatestConfirmedProofs {
    pub height: HeightOf<Arbitrum>,
    pub l1_latest_confirmed_slot: U256,
    pub l1_nodes_slot: U256,
    pub l1_contract_address: H160,
}

#[queue_msg]
pub struct FetchL2Header {
    // the height to update to
    pub height: HeightOf<Arbitrum>,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ArbitrumData<Tr: ChainExt> {
    L1ContractRootProof(L1ContractRootProof<Tr>),
    IbcContractRootProof(IbcContractRootProof<Tr>),
    LatestConfirmedProofs(LatestConfirmedProofs<Tr>),
    L2Header(L2Header<Tr>),
}

try_from_relayer_msg! {
    chain = Arbitrum,
    generics = (Tr: ChainExt),
    msgs = ArbitrumData(
        L1ContractRootProof(L1ContractRootProof<Tr>),
        IbcContractRootProof(IbcContractRootProof<Tr>),
        LatestConfirmedProofs(LatestConfirmedProofs<Tr>),
        L2Header(L2Header<Tr>),
    ),
}

#[queue_msg]
pub struct L1ContractRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Arbitrum>,
    pub proof: AccountProof,
}

#[queue_msg]
pub struct IbcContractRootProof<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Arbitrum>,
    pub proof: AccountProof,
}

#[queue_msg]
pub struct LatestConfirmedProofs<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Arbitrum>,
    pub latest_confirmed: u64,
    pub latest_confirmed_slot_proof: StorageProof,
    pub nodes_slot_proof: StorageProof,
}

#[queue_msg]
pub struct L2Header<#[cover] Tr: ChainExt> {
    pub height: HeightOf<Arbitrum>,
    pub l2_header: arbitrum::l2_header::L2Header,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ArbitrumAggregate<Tr: ChainExt> {
    AggregateHeader(AggregateHeader<Tr>),
}

#[queue_msg]
pub struct AggregateHeader<Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Arbitrum, Tr>,
}

impl<Tr: ChainExt> DoAggregate for Identified<Arbitrum, Tr, ArbitrumAggregate<Tr>>
where
    Identified<Arbitrum, Tr, L1ContractRootProof<Tr>>: IsAggregateData,
    Identified<Arbitrum, Tr, IbcContractRootProof<Tr>>: IsAggregateData,
    Identified<Arbitrum, Tr, LatestConfirmedProofs<Tr>>: IsAggregateData,
    Identified<Arbitrum, Tr, L2Header<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Arbitrum>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Arbitrum, Tr>)>,
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
            ArbitrumAggregate::AggregateHeader(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

impl<Tr> UseAggregate<RelayMessage> for Identified<Arbitrum, Tr, AggregateHeader<Tr>>
where
    Tr: ChainExt,
    Identified<Arbitrum, Tr, L1ContractRootProof<Tr>>: IsAggregateData,
    Identified<Arbitrum, Tr, IbcContractRootProof<Tr>>: IsAggregateData,
    Identified<Arbitrum, Tr, LatestConfirmedProofs<Tr>>: IsAggregateData,
    Identified<Arbitrum, Tr, L2Header<Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Arbitrum>)>,
{
    type AggregatedData = HList![
        Identified<Arbitrum, Tr, L1ContractRootProof<Tr>>,
        Identified<Arbitrum, Tr, IbcContractRootProof<Tr>>,
        Identified<Arbitrum, Tr, LatestConfirmedProofs<Tr>>,
        Identified<Arbitrum, Tr, L2Header<Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: _chain_id,
            t: AggregateHeader { req },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: _l1_contract_root_proof_chain_id,
                t: L1ContractRootProof {
                    height: _l1_contract_root_proof_height,
                    proof: l1_contract_root_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: _ibc_contract_root_proof_chain_id,
                t: IbcContractRootProof {
                    height: _ibc_contract_root_proof_height,
                    proof: ibc_contract_root_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: _latest_confirmed_proofs_chain_id,
                t: LatestConfirmedProofs {
                    height: _latest_confirmed_proofs_height,
                    latest_confirmed: _latest_confirmed,
                    latest_confirmed_slot_proof,
                    nodes_slot_proof,
                    __marker: _
                },
                __marker: _,
            },
            Identified {
                chain_id: _l2_header_proof_chain_id,
                t: L2Header {
                    height: _l2_header_proof_height,
                    l2_header,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> Op<RelayMessage> {
        // assert_eq!(rollup_contract_root_proof_chain_id, chain_id);
        // assert_eq!(latest_batch_index_proof_chain_id, chain_id);
        // assert_eq!(arbitrum_finalized_root_proof_chain_id, chain_id);
        // assert_eq!(ibc_contract_root_proof_chain_id, chain_id);
        // assert_eq!(batch_hash_proof_chain_id, chain_id);
        // assert_eq!(commit_batch_transaction_input_chain_id, chain_id);

        // assert_eq!(
        //     rollup_contract_root_proof_height,
        //     latest_batch_index_proof_height
        // );
        // assert_eq!(
        //     rollup_contract_root_proof_height,
        //     arbitrum_finalized_root_proof_height
        // );
        // assert_eq!(
        //     rollup_contract_root_proof_height,
        //     ibc_contract_root_proof_height
        // );
        // assert_eq!(rollup_contract_root_proof_height, batch_hash_proof_height);
        // assert_eq!(
        //     rollup_contract_root_proof_height,
        //     commit_batch_transaction_input_height
        // );

        // assert_eq!(
        //     arbitrum_finalized_root_proof_batch_index,
        //     batch_hash_proof_batch_index
        // );
        // assert_eq!(
        //     arbitrum_finalized_root_proof_batch_index,
        //     commit_batch_transaction_input_batch_index
        // );

        effect(id::<Tr, Arbitrum, _>(
            req.counterparty_chain_id,
            MsgUpdateClientData(MsgUpdateClient {
                client_id: req.counterparty_client_id,
                client_message: arbitrum::header::Header {
                    l1_height: req.update_to,
                    l1_account_proof: l1_contract_root_proof,
                    l2_ibc_account_proof: ibc_contract_root_proof,
                    l1_next_node_num_slot_proof: latest_confirmed_slot_proof,
                    l1_nodes_slot_proof: nodes_slot_proof,
                    l2_header,
                },
            }),
        ))
    }
}
