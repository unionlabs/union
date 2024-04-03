use std::{collections::VecDeque, marker::PhantomData};

use chain_utils::{
    cosmos::Cosmos,
    cosmos_sdk::{BroadcastTxCommitError, CosmosSdkChain, CosmosSdkChainExt},
    wasm::Wraps,
};
use frunk::{hlist_pat, HList};
use protos::ibc::core::connection::v1::MsgConnectionOpenInit;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    effect, fetch, queue_msg, wait, QueueMsg,
};
use unionlabs::{
    encoding::{Decode, Encode, Proto},
    google::protobuf::any::{mk_any, Any},
    hash::H160,
    ibc::{
        core::{
            client::{height::IsHeight, msg_update_client::MsgUpdateClient},
            commitment::merkle_proof::MerkleProof,
        },
        lightclients::tendermint,
    },
    proof::ClientStatePath,
    tendermint::types::validator::Validator,
    traits::{Chain, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
    TypeUrl,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::cosmos_sdk::{
        data::{TrustedCommit, TrustedValidators, UntrustedCommit, UntrustedValidators},
        fetch::{
            fetch_trusted_commit, fetch_trusted_validators, fetch_untrusted_commit,
            fetch_untrusted_validators, AbciQueryType, FetchAbciQuery, FetchTrustedCommit,
            FetchTrustedValidators, FetchUntrustedCommit, FetchUntrustedValidators,
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
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayMessageTypes, Wasm, WasmConfig,
};

impl ChainExt for Cosmos {
    type Data<Tr: ChainExt> = CosmosDataMsg<Self, Tr>;
    type Fetch<Tr: ChainExt> = CosmosFetch<Cosmos, Tr>;
    type Aggregate<Tr: ChainExt> = CosmosAggregateMsg<Cosmos, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = ();
}

impl ChainExt for Wasm<Cosmos> {
    type Data<Tr: ChainExt> = CosmosDataMsg<Self, Tr>;
    type Fetch<Tr: ChainExt> = CosmosFetch<Wasm<Cosmos>, Tr>;
    type Aggregate<Tr: ChainExt> = CosmosAggregateMsg<Wasm<Cosmos>, Tr>;

    type MsgError = BroadcastTxCommitError;

    type Config = WasmConfig;
}

impl<Tr: ChainExt, Hc: ChainExt + Wraps<Self>> DoMsg<Hc, Tr> for Cosmos
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

impl<
        Tr: ChainExt,
        Hc: Wraps<Self> + ChainExt<StateProof = MerkleProof, Fetch<Tr> = CosmosFetch<Hc, Tr>>,
    > DoFetchState<Hc, Tr> for Cosmos
where
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
    fn state(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> QueueMsg<RelayMessageTypes> {
        seq([
            wait(id(
                hc.chain_id(),
                WaitForBlock {
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

impl<Tr: ChainExt, Hc: Wraps<Self> + ChainExt<Fetch<Tr> = CosmosFetch<Hc, Tr>>> DoFetchProof<Hc, Tr>
    for Cosmos
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
{
    fn proof(hc: &Hc, at: HeightOf<Hc>, path: PathOf<Hc, Tr>) -> QueueMsg<RelayMessageTypes> {
        seq([
            wait(id(
                hc.chain_id(),
                WaitForBlock {
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

const _: () = {
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
};

const _: () = {
    try_from_relayer_msg! {
        chain = Wasm<Cosmos>,
        generics = (Tr: ChainExt),
        msgs = CosmosDataMsg(
            TrustedCommit(TrustedCommit<Wasm<Cosmos>, Tr>),
            UntrustedCommit(UntrustedCommit<Wasm<Cosmos>, Tr>),
            TrustedValidators(TrustedValidators<Wasm<Cosmos>, Tr>),
            UntrustedValidators(UntrustedValidators<Wasm<Cosmos>, Tr>),
        ),
    }
};

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
