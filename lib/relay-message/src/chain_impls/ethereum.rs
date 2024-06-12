use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, ops::Div, sync::Arc};

use chain_utils::{
    ethereum::{
        Ethereum, EthereumChain, EthereumChainExt as _, EthereumConsensusChain,
        EthereumSignerMiddleware, IbcHandlerErrors, IbcHandlerExt, ETHEREUM_REVISION_NUMBER,
    },
    Pool,
};
use contracts::ibc_handler::{
    self, AcknowledgePacketCall, ChannelOpenAckCall, ChannelOpenConfirmCall, ChannelOpenInitCall,
    ChannelOpenTryCall, ConnectionOpenAckCall, ConnectionOpenConfirmCall, ConnectionOpenInitCall,
    ConnectionOpenTryCall, CreateClientCall, IBCHandler, RecvPacketCall, TimeoutPacketCall,
    UpdateClientCall,
};
use ethers::{
    abi::{AbiDecode, AbiEncode},
    contract::{ContractError, EthCall},
    providers::{Middleware, ProviderError},
    types::{Bytes, TransactionReceipt},
    utils::keccak256,
};
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, effect, fetch, queue_msg, void, wait, QueueMsg,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use typenum::Unsigned;
use unionlabs::{
    encoding::{Decode, Encode, EncodeAs, EthAbi},
    ethereum::{
        beacon::{GenesisData, LightClientBootstrap, LightClientFinalityUpdate},
        config::ChainSpec,
        IBC_HANDLER_COMMITMENTS_SLOT,
    },
    hash::{H160, H256},
    ibc::{
        core::client::{
            height::{Height, IsHeight},
            msg_update_client::MsgUpdateClient,
        },
        lightclients::ethereum::{
            self,
            account_proof::AccountProof,
            account_update::AccountUpdate,
            light_client_update,
            trusted_sync_committee::{ActiveSyncCommittee, TrustedSyncCommittee},
        },
    },
    ics24::{ClientStatePath, NextSequenceAckPath, NextSequenceSendPath, Path},
    traits::{Chain, ClientIdOf, ClientState, ClientStateOf, HeightOf, IbcStateEncodingOf},
    uint::U256,
    MaybeRecoverableError,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    data::{AnyData, Data, IbcProof, IbcState},
    effect::{
        AnyEffect, BatchMsg, Effect, MsgConnectionOpenAckData, MsgConnectionOpenInitData,
        MsgConnectionOpenTryData, MsgUpdateClientData,
    },
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified, seq,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForTimestamp},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayMessageTypes,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct EthereumConfig {
    pub client_type: String,
    pub client_address: H160,
}

impl<C: ChainSpec> ChainExt for Ethereum<C> {
    type Data<Tr: ChainExt> = EthereumDataMsg<C, Tr>;
    type Fetch<Tr: ChainExt> = EthereumFetchMsg<C, Tr>;
    type Aggregate<Tr: ChainExt> = EthereumAggregateMsg<C, Tr>;

    type MsgError = TxSubmitError;

    type Config = EthereumConfig;
}

impl<C, Tr> DoMsg<Self, Tr> for Ethereum<C>
where
    C: ChainSpec,
    Tr: ChainExt<
        SelfConsensusState: Encode<EthAbi>,
        SelfClientState: Encode<EthAbi>,
        Header: Encode<EthAbi>,
        StoredClientState<Ethereum<C>>: Encode<Tr::IbcStateEncoding>,
        StateProof: Encode<EthAbi>,
    >,

    ClientStateOf<Ethereum<C>>: Encode<Tr::IbcStateEncoding>,
{
    async fn msg(&self, msg: Effect<Self, Tr>) -> Result<(), Self::MsgError> {
        do_msg(&self.ibc_handlers, msg, false).await
    }
}

pub async fn do_msg<Hc, Tr>(
    ibc_handlers: &Pool<IBCHandler<EthereumSignerMiddleware>>,
    msg: Effect<Hc, Tr>,
    legacy: bool,
) -> Result<(), TxSubmitError>
where
    Hc: ChainExt<Config = EthereumConfig, SelfClientState: Encode<Tr::IbcStateEncoding>>
        + EthereumChain,
    Tr: ChainExt<
        SelfConsensusState: Encode<EthAbi>,
        SelfClientState: Encode<EthAbi>,
        Header: Encode<EthAbi>,
        StoredClientState<Hc>: Encode<Tr::IbcStateEncoding>,
        StateProof: Encode<EthAbi>,
    >,
{
    let f = |ibc_handler| async move {
        let msg: ethers::contract::FunctionCall<_, _, ()> = match msg.clone() {
            Effect::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => mk_function_call(
                ibc_handler,
                ConnectionOpenInitCall(contracts::ibc_handler::MsgConnectionOpenInit {
                    client_id: data.client_id.to_string(),
                    version: data.version.into(),
                    counterparty: data.counterparty.into(),
                    delay_period: data.delay_period,
                }),
            ),
            Effect::ConnectionOpenTry(MsgConnectionOpenTryData(data)) => mk_function_call(
                ibc_handler,
                ConnectionOpenTryCall(contracts::ibc_handler::MsgConnectionOpenTry {
                    counterparty: data.counterparty.into(),
                    delay_period: data.delay_period,
                    client_id: data.client_id.to_string(),
                    // needs to be encoded how the counterparty is encoding it
                    client_state_bytes: Encode::<Tr::IbcStateEncoding>::encode(data.client_state)
                        .into(),
                    counterparty_versions: data
                        .counterparty_versions
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                    proof_init: data.proof_init.encode().into(),
                    proof_client: data.proof_client.encode().into(),
                    proof_consensus: data.proof_consensus.encode().into(),
                    proof_height: data.proof_height.into_height().into(),
                    consensus_height: data.consensus_height.into_height().into(),
                }),
            ),
            Effect::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => mk_function_call(
                ibc_handler,
                ConnectionOpenAckCall(contracts::ibc_handler::MsgConnectionOpenAck {
                    connection_id: data.connection_id.to_string(),
                    counterparty_connection_id: data.counterparty_connection_id.to_string(),
                    version: data.version.into(),
                    // needs to be encoded how the counterparty is encoding it
                    client_state_bytes: Encode::<Tr::IbcStateEncoding>::encode(data.client_state)
                        .into(),
                    proof_height: data.proof_height.into(),
                    proof_try: data.proof_try.encode().into(),
                    proof_client: data.proof_client.encode().into(),
                    proof_consensus: data.proof_consensus.encode().into(),
                    consensus_height: data.consensus_height.into(),
                }),
            ),
            Effect::ConnectionOpenConfirm(data) => mk_function_call(
                ibc_handler,
                ConnectionOpenConfirmCall(contracts::ibc_handler::MsgConnectionOpenConfirm {
                    connection_id: data.msg.connection_id.to_string(),
                    proof_ack: data.msg.proof_ack.encode().into(),
                    proof_height: data.msg.proof_height.into_height().into(),
                }),
            ),
            Effect::ChannelOpenInit(data) => mk_function_call(
                ibc_handler,
                ChannelOpenInitCall(contracts::ibc_handler::MsgChannelOpenInit {
                    port_id: data.msg.port_id.to_string(),
                    channel: data.msg.channel.into(),
                }),
            ),
            Effect::ChannelOpenTry(data) => mk_function_call(
                ibc_handler,
                ChannelOpenTryCall(contracts::ibc_handler::MsgChannelOpenTry {
                    port_id: data.msg.port_id.to_string(),
                    channel: data.msg.channel.into(),
                    counterparty_version: data.msg.counterparty_version,
                    proof_init: data.msg.proof_init.encode().into(),
                    proof_height: data.msg.proof_height.into(),
                }),
            ),
            Effect::ChannelOpenAck(data) => mk_function_call(
                ibc_handler,
                ChannelOpenAckCall(contracts::ibc_handler::MsgChannelOpenAck {
                    port_id: data.msg.port_id.to_string(),
                    channel_id: data.msg.channel_id.to_string(),
                    counterparty_version: data.msg.counterparty_version,
                    counterparty_channel_id: data.msg.counterparty_channel_id.to_string(),
                    proof_try: data.msg.proof_try.encode().into(),
                    proof_height: data.msg.proof_height.into_height().into(),
                }),
            ),
            Effect::ChannelOpenConfirm(data) => mk_function_call(
                ibc_handler,
                ChannelOpenConfirmCall(contracts::ibc_handler::MsgChannelOpenConfirm {
                    port_id: data.msg.port_id.to_string(),
                    channel_id: data.msg.channel_id.to_string(),
                    proof_ack: data.msg.proof_ack.encode().into(),
                    proof_height: data.msg.proof_height.into_height().into(),
                }),
            ),
            Effect::RecvPacket(data) => mk_function_call(
                ibc_handler,
                RecvPacketCall(contracts::ibc_handler::MsgPacketRecv {
                    packet: data.msg.packet.into(),
                    proof: data.msg.proof_commitment.encode().into(),
                    proof_height: data.msg.proof_height.into_height().into(),
                }),
            ),
            Effect::AckPacket(data) => mk_function_call(
                ibc_handler,
                AcknowledgePacketCall(contracts::ibc_handler::MsgPacketAcknowledgement {
                    packet: data.msg.packet.into(),
                    acknowledgement: data.msg.acknowledgement.into(),
                    proof: data.msg.proof_acked.encode().into(),
                    proof_height: data.msg.proof_height.into_height().into(),
                }),
            ),
            Effect::TimeoutPacket(data) => mk_function_call(
                ibc_handler,
                TimeoutPacketCall(contracts::ibc_handler::MsgPacketTimeout {
                    packet: data.msg.packet.into(),
                    proof: data.msg.proof_unreceived.encode().into(),
                    proof_height: data.msg.proof_height.into_height().into(),
                    next_sequence_recv: data.msg.next_sequence_recv.get(),
                }),
            ),
            Effect::CreateClient(data) => {
                let register_client_result = ibc_handler.register_client(
                    data.config.client_type.clone(),
                    data.config.client_address.into(),
                );

                // TODO(benluelo): Better way to check if client type has already been registered?
                match register_client_result.send().await {
                    Ok(ok) => {
                        ok.await.unwrap().unwrap();
                    }

                    Err(why) => info!(
                        "error registering client type, it is likely already registered: {:?}",
                        why
                    ),
                }

                mk_function_call(
                    ibc_handler,
                    CreateClientCall(contracts::shared_types::MsgCreateClient {
                        client_type: data.config.client_type,
                        client_state_bytes: data.msg.client_state.encode_as::<EthAbi>().into(),
                        consensus_state_bytes: data
                            .msg
                            .consensus_state
                            .encode_as::<EthAbi>()
                            .into(),
                    }),
                )
            }
            Effect::UpdateClient(MsgUpdateClientData(data)) => mk_function_call(
                ibc_handler,
                UpdateClientCall(ibc_handler::MsgUpdateClient {
                    client_id: data.client_id.to_string(),
                    client_message: data.client_message.clone().encode_as::<EthAbi>().into(),
                }),
            ),
            Effect::Batch(BatchMsg(msgs)) => {
                for msg in msgs {
                    Box::pin(do_msg(ibc_handlers, msg, legacy)).await.unwrap();
                }

                return Ok(());
            }
        };

        let msg = if legacy { msg.legacy() } else { msg };

        debug!(msg = %msg.function.name, "submitting evm tx");

        match msg.estimate_gas().await {
            Ok(estimated_gas) => {
                debug!(%estimated_gas, "gas estimation");

                // TODO: config
                let msg = msg.gas(estimated_gas + (estimated_gas / 10));
                let result = msg.send().await;
                match result {
                    Ok(ok) => {
                        info!(
                            tx_hash = %H256::from(ok.tx_hash()),
                            msg = %msg.function.name,
                            "evm tx"
                        );
                        let tx_rcp: TransactionReceipt =
                            ok.await?.ok_or(TxSubmitError::NoTxReceipt)?;
                        info!(
                            tx_hash = %H256::from(tx_rcp.transaction_hash),
                            "evm transaction submitted"
                        );
                        Ok(())
                    }
                    Err(ContractError::Revert(revert)) => {
                        error!(?revert, "evm transaction failed");
                        let err =
                            <IbcHandlerErrors as ethers::abi::AbiDecode>::decode(revert.clone())
                                .map_err(|_| TxSubmitError::InvalidRevert(revert.clone()))?;
                        error!(?revert, ?err, "evm transaction failed");
                        Ok(())
                    }
                    _ => {
                        panic!("evm transaction non-recoverable failure");
                    }
                }
            }
            Err(ContractError::Revert(revert)) => {
                error!(?revert, "evm transaction failed");
                let err = <IbcHandlerErrors as ethers::abi::AbiDecode>::decode(revert.clone())
                    .map_err(|_| TxSubmitError::InvalidRevert(revert.clone()))?;
                error!(?revert, ?err, "evm estimation failed");
                Ok(())
            }
            _ => {
                panic!("evm estimation non-recoverable failure");
            }
        }
    };

    ibc_handlers.with(f).await
}

impl<C: ChainSpec, Tr: ChainExt> DoFetchProof<Self, Tr> for Ethereum<C>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
{
    fn proof(
        c: &Self,
        at: HeightOf<Self>,
        path: PathOf<Ethereum<C>, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        fetch(id::<Self, Tr, _>(
            c.chain_id(),
            Fetch::specific(GetProof { path, height: at }),
        ))
    }
}

impl<C, Tr> DoFetchState<Self, Tr> for Ethereum<C>
where
    C: ChainSpec,
    Tr: ChainExt<SelfClientState: Decode<IbcStateEncodingOf<Ethereum<C>>> + Encode<EthAbi>>,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
{
    fn state(
        hc: &Self,
        at: HeightOf<Self>,
        path: PathOf<Ethereum<C>, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
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
            .ibc_state_read::<_, Ethereum<C>, Tr>(
                hc.provider.get_block_number().await.unwrap().as_u64(),
                ClientStatePath { client_id },
            )
            .await
            .unwrap()
    }
}

impl<C, Tr> DoFetchUpdateHeaders<Self, Tr> for Ethereum<C>
where
    C: ChainSpec,
    Tr: ChainExt,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> QueueMsg<RelayMessageTypes> {
        aggregate(
            [fetch(id::<Ethereum<C>, Tr, _>(
                c.chain_id,
                Fetch::specific(FetchFinalityUpdate {}),
            ))],
            [],
            id(
                c.chain_id,
                Aggregate::specific(MakeCreateUpdatesData { req: update_info }),
            ),
        )
    }
}

impl<C, Tr> DoFetch<Ethereum<C>> for EthereumFetchMsg<C, Tr>
where
    C: ChainSpec,
    Tr: ChainExt<
        SelfClientState: Decode<<Ethereum<C> as Chain>::IbcStateEncoding> + Encode<EthAbi>,
        SelfConsensusState: Decode<<Ethereum<C> as Chain>::IbcStateEncoding>,
    >,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Ethereum<C>, Tr>)>,
{
    async fn do_fetch(ethereum: &Ethereum<C>, msg: Self) -> QueueMsg<RelayMessageTypes> {
        let msg: EthereumFetchMsg<C, Tr> = msg;
        let msg = match msg {
            Self::FetchFinalityUpdate(FetchFinalityUpdate {}) => Data::specific(FinalityUpdate {
                finality_update: ethereum
                    .beacon_api_client
                    .finality_update()
                    .await
                    .unwrap()
                    .data,
                __marker: PhantomData,
            }),
            Self::FetchLightClientUpdates(FetchLightClientUpdates {
                trusted_period,
                target_period,
            }) => Data::specific(LightClientUpdates {
                light_client_updates: ethereum
                    .beacon_api_client
                    .light_client_updates(trusted_period + 1, target_period - trusted_period)
                    .await
                    .unwrap()
                    .0
                    .into_iter()
                    .map(|x| x.data)
                    .collect(),
                __marker: PhantomData,
            }),
            Self::FetchLightClientUpdate(FetchLightClientUpdate { period }) => {
                Data::specific(LightClientUpdate {
                    update: ethereum
                        .beacon_api_client
                        .light_client_updates(period, 1)
                        .await
                        .unwrap()
                        .0
                        .into_iter()
                        .map(|x| x.data)
                        .collect::<Vec<light_client_update::LightClientUpdate<_>>>()
                        .pop()
                        .unwrap(),
                    __marker: PhantomData,
                })
            }
            Self::FetchBootstrap(FetchBootstrap { slot }) => Data::specific(BootstrapData {
                slot,
                bootstrap: ethereum
                    .beacon_api_client
                    .bootstrap_for_slot(slot)
                    .await
                    .unwrap()
                    .data,
                __marker: PhantomData,
            }),
            Self::FetchAccountUpdate(FetchAccountUpdate { slot }) => {
                let execution_height = ethereum
                    .beacon_api_client
                    .execution_height(beacon_api::client::BlockId::Slot(slot))
                    .await
                    .unwrap();

                let account_update = ethereum
                    .provider
                    .get_proof(
                        ethers::types::H160::from(ethereum.ibc_handler_address),
                        vec![],
                        // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                        Some(execution_height.into()),
                    )
                    .await
                    .unwrap();

                Data::specific(AccountUpdateData {
                    slot,
                    update: AccountUpdate {
                        account_proof: AccountProof {
                            storage_root: account_update.storage_hash.into(),
                            proof: account_update
                                .account_proof
                                .into_iter()
                                .map(|x| x.to_vec())
                                .collect(),
                        },
                    },
                    __marker: PhantomData,
                })
            }
            Self::FetchBeaconGenesis(_) => Data::specific(BeaconGenesisData {
                genesis: ethereum.beacon_api_client.genesis().await.unwrap().data,
                __marker: PhantomData,
            }),
            Self::FetchGetProof(get_proof) => fetch_get_proof(ethereum, get_proof).await,
            Self::FetchIbcState(ibc_state) => fetch_ibc_state(ethereum, ibc_state).await,
        };

        data(id::<Ethereum<C>, Tr, _>(ethereum.chain_id, msg))
    }
}

pub async fn fetch_get_proof<Hc, Tr>(c: &Hc, get_proof: GetProof<Hc, Tr>) -> Data<Hc, Tr>
where
    Hc: ChainExt + EthereumConsensusChain,
    Tr: ChainExt,
{
    let path = get_proof.path.to_string();

    // TODO: Use unionlabs::slot here
    // or ibc_commitment_key?
    let location = keccak256(
        keccak256(path.as_bytes())
            .into_iter()
            .chain(AbiEncode::encode(IBC_HANDLER_COMMITMENTS_SLOT))
            .collect::<Vec<_>>(),
    );

    let execution_height = c
        .execution_height_of_beacon_slot(get_proof.height.revision_height())
        .await;

    let proof = c
        .get_proof(
            c.ibc_handler_address(),
            U256::from_be_bytes(location),
            execution_height,
        )
        .await;

    match get_proof.path {
        Path::ClientState(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::ClientConsensusState(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::Connection(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::ChannelEnd(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::Commitment(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::Acknowledgement(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::Receipt(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::NextSequenceSend(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::NextSequenceRecv(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::NextSequenceAck(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::NextConnectionSequence(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::NextClientSequence(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
    }
}

pub async fn fetch_ibc_state<Hc, Tr>(
    c: &Hc,
    FetchIbcState { path, height }: FetchIbcState<Hc, Tr>,
) -> Data<Hc, Tr>
where
    Hc: ChainExt<
            StoredClientState<Tr>: Decode<Hc::IbcStateEncoding>,
            StoredConsensusState<Tr>: Decode<Hc::IbcStateEncoding>,
        > + EthereumChain
        + EthereumConsensusChain,
    Tr: ChainExt,
{
    let execution_height = c
        .execution_height_of_beacon_slot(height.revision_height())
        .await;

    match path {
        Path::ClientState(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::ClientConsensusState(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::Connection(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::ChannelEnd(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::Commitment(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::Acknowledgement(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::Receipt(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::NextSequenceSend(path) => {
            let path_str = path.to_string();

            let location = keccak256(
                keccak256(path_str.as_bytes())
                    .into_iter()
                    .chain(AbiEncode::encode(IBC_HANDLER_COMMITMENTS_SLOT))
                    .collect::<Vec<_>>(),
            );

            let state = c
                .provider()
                .get_storage_at(
                    ethers::types::H160::from(c.ibc_handler_address()),
                    location.into(),
                    Some(execution_height.into()),
                )
                .await
                .unwrap();

            Data::from(IbcState::<NextSequenceSendPath, _, _> {
                state: AbiDecode::decode(state.0).unwrap(),
                height,
                path,
            })
        }
        Path::NextSequenceRecv(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        // REVIEW: Can we use ibc_handler().ibc_state_read() here?
        Path::NextSequenceAck(path) => {
            let path_str = path.to_string();

            let location = keccak256(
                keccak256(path_str.as_bytes())
                    .into_iter()
                    .chain(AbiEncode::encode(IBC_HANDLER_COMMITMENTS_SLOT))
                    .collect::<Vec<_>>(),
            );

            let state = c
                .provider()
                .get_storage_at(
                    ethers::types::H160::from(c.ibc_handler_address()),
                    location.into(),
                    Some(execution_height.into()),
                )
                .await
                .unwrap();

            Data::from(IbcState::<NextSequenceAckPath, _, _> {
                state: AbiDecode::decode(state.0).unwrap(),
                height,
                path,
            })
        }
        Path::NextConnectionSequence(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::NextClientSequence(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
    }
}

#[queue_msg]
pub struct CreateUpdateData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Ethereum<C>, Tr>,
    pub currently_trusted_slot: u64,
    pub light_client_update: light_client_update::LightClientUpdate<C>,
    pub is_next: bool,
}

#[queue_msg]
pub struct MakeCreateUpdatesData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Ethereum<C>, Tr>,
}

#[queue_msg]
pub struct MakeCreateUpdatesFromLightClientUpdatesData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Ethereum<C>, Tr>,
    pub trusted_height: Height,
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[queue_msg]
pub struct FetchLightClientUpdate {
    pub period: u64,
}

#[queue_msg]
pub struct FetchFinalityUpdate {}

#[queue_msg]
pub struct FetchLightClientUpdates {
    pub trusted_period: u64,
    pub target_period: u64,
}

#[queue_msg]
pub struct FetchBootstrap {
    pub slot: u64,
}

#[queue_msg]
pub struct FetchAccountUpdate {
    pub slot: u64,
}

#[queue_msg]
pub struct FetchBeaconGenesis {}

#[queue_msg]
pub struct BootstrapData<C: ChainSpec, #[cover] Tr: ChainExt> {
    pub slot: u64,
    pub bootstrap: LightClientBootstrap<C>,
}

#[queue_msg]
pub struct AccountUpdateData<#[cover] C: ChainSpec, #[cover] Tr: ChainExt> {
    pub slot: u64,
    pub update: AccountUpdate,
}

#[queue_msg]
pub struct BeaconGenesisData<#[cover] C: ChainSpec, #[cover] Tr: ChainExt> {
    pub genesis: GenesisData,
}

try_from_relayer_msg! {
    chain = Ethereum<C>,
    generics = (C: ChainSpec, Tr: ChainExt),
    msgs = EthereumDataMsg(
        FinalityUpdate(FinalityUpdate<C, Tr>),
        LightClientUpdates(LightClientUpdates<C, Tr>),
        LightClientUpdate(LightClientUpdate<C, Tr>),
        Bootstrap(BootstrapData<C, Tr>),
        AccountUpdate(AccountUpdateData<C, Tr>),
        BeaconGenesis(BeaconGenesisData<C, Tr>),
    ),
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum EthereumFetchMsg<C: ChainSpec, Tr: ChainExt> {
    FetchFinalityUpdate(FetchFinalityUpdate),
    FetchLightClientUpdates(FetchLightClientUpdates),
    FetchLightClientUpdate(FetchLightClientUpdate),
    FetchBootstrap(FetchBootstrap),
    FetchAccountUpdate(FetchAccountUpdate),
    FetchBeaconGenesis(FetchBeaconGenesis),
    FetchGetProof(GetProof<Ethereum<C>, Tr>),
    FetchIbcState(FetchIbcState<Ethereum<C>, Tr>),
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum EthereumDataMsg<C: ChainSpec, Tr: ChainExt> {
    FinalityUpdate(FinalityUpdate<C, Tr>),
    LightClientUpdates(LightClientUpdates<C, Tr>),
    LightClientUpdate(LightClientUpdate<C, Tr>),
    Bootstrap(BootstrapData<C, Tr>),
    AccountUpdate(AccountUpdateData<C, Tr>),
    BeaconGenesis(BeaconGenesisData<C, Tr>),
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum EthereumAggregateMsg<C: ChainSpec, Tr: ChainExt> {
    CreateUpdate(CreateUpdateData<C, Tr>),
    MakeCreateUpdates(MakeCreateUpdatesData<C, Tr>),
    MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<C, Tr>),
}

#[queue_msg]
pub struct FinalityUpdate<C: ChainSpec, #[cover] Tr: ChainExt> {
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[queue_msg]
pub struct LightClientUpdates<C: ChainSpec, #[cover] Tr: ChainExt> {
    pub light_client_updates: Vec<light_client_update::LightClientUpdate<C>>,
}

#[queue_msg]
pub struct LightClientUpdate<C: ChainSpec, #[cover] Tr: ChainExt> {
    pub update: light_client_update::LightClientUpdate<C>,
}

impl<C, Tr> DoAggregate for Identified<Ethereum<C>, Tr, EthereumAggregateMsg<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt<SelfClientState: Encode<EthAbi>>,

    Identified<Ethereum<C>, Tr, AccountUpdateData<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, BootstrapData<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, BeaconGenesisData<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, FinalityUpdate<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, LightClientUpdates<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, LightClientUpdate<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Ethereum<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Ethereum<C>>)>,

    AnyLightClientIdentified<AnyData>: From<identified!(Data<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t: data,
            __marker: _,
        }: Self,
        aggregated_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes> {
        match data {
            EthereumAggregateMsg::CreateUpdate(msg) => {
                do_aggregate(id(chain_id, msg), aggregated_data)
            }
            EthereumAggregateMsg::MakeCreateUpdates(msg) => {
                do_aggregate(id(chain_id, msg), aggregated_data)
            }
            EthereumAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(msg) => {
                do_aggregate(id(chain_id, msg), aggregated_data)
            }
        }
    }
}

fn make_create_update<C, Tr>(
    req: FetchUpdateHeaders<Ethereum<C>, Tr>,
    chain_id: <<Ethereum<C> as Chain>::SelfClientState as ClientState>::ChainId,
    currently_trusted_slot: u64,
    light_client_update: light_client_update::LightClientUpdate<C>,
    is_next: bool,
) -> QueueMsg<RelayMessageTypes>
where
    C: ChainSpec,
    Tr: ChainExt,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,
{
    // When we fetch the update at this height, the `next_sync_committee` will
    // be the current sync committee of the period that we want to update to.
    let previous_period = u64::max(
        1,
        light_client_update.attested_header.beacon.slot
            / (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64),
    ) - 1;

    aggregate(
        [
            fetch(id::<Ethereum<C>, Tr, _>(
                chain_id,
                Fetch::specific(FetchLightClientUpdate {
                    period: previous_period,
                }),
            )),
            fetch(id::<Ethereum<C>, Tr, _>(
                chain_id,
                Fetch::specific(FetchAccountUpdate {
                    slot: light_client_update.attested_header.beacon.slot,
                }),
            )),
            fetch(id::<Ethereum<C>, Tr, _>(
                chain_id,
                Fetch::specific(FetchBeaconGenesis {}),
            )),
        ],
        [],
        id(
            chain_id,
            Aggregate::specific(CreateUpdateData {
                req,
                currently_trusted_slot,
                light_client_update,
                is_next,
            }),
        ),
    )
}

fn sync_committee_period<H: Into<u64>, C: ChainSpec>(height: H) -> u64 {
    height.into().div(C::PERIOD::U64)
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Contract(#[from] ContractError<EthereumSignerMiddleware>),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("no tx receipt from tx")]
    NoTxReceipt,
    #[error("invalid revert code: {0}")]
    InvalidRevert(Bytes),
}

impl MaybeRecoverableError for TxSubmitError {
    fn is_recoverable(&self) -> bool {
        // TODO: Figure out if any failures are unrecoverable
        true
    }
}

pub fn mk_function_call<Call: EthCall>(
    ibc_handler: IBCHandler<EthereumSignerMiddleware>,
    data: Call,
) -> ethers::contract::FunctionCall<Arc<EthereumSignerMiddleware>, EthereumSignerMiddleware, ()> {
    ibc_handler
        .method_hash(<Call as EthCall>::selector(), data)
        .expect("method selector is generated; qed;")
}

pub trait EthereumChainExt = ChainExt + EthereumChain;

#[queue_msg]
pub struct GetProof<Hc: EthereumChainExt, Tr: ChainExt> {
    pub path: Path<ClientIdOf<Hc>, HeightOf<Tr>>,
    pub height: HeightOf<Hc>,
}

#[queue_msg]
pub struct FetchIbcState<Hc: EthereumChainExt, Tr: ChainExt> {
    pub path: Path<ClientIdOf<Hc>, HeightOf<Tr>>,
    pub height: HeightOf<Hc>,
}

impl<C, Tr> UseAggregate<RelayMessageTypes> for Identified<Ethereum<C>, Tr, CreateUpdateData<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

    Identified<Ethereum<C>, Tr, AccountUpdateData<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, LightClientUpdate<C, Tr>>: IsAggregateData,
    Identified<Ethereum<C>, Tr, BeaconGenesisData<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Ethereum<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Ethereum<C>>)>,
{
    type AggregatedData = HList![
        Identified<Ethereum<C>, Tr, LightClientUpdate<C, Tr>>,
        Identified<Ethereum<C>, Tr, AccountUpdateData<C, Tr>>,
        Identified<Ethereum<C>, Tr, BeaconGenesisData<C, Tr>>
    ];

    fn aggregate(
        Identified {
            chain_id,
            t:
                CreateUpdateData {
                    req,
                    currently_trusted_slot,
                    light_client_update,
                    is_next,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: light_client_update_chain_id,
                t: LightClientUpdate {
                    update: light_client_update::LightClientUpdate {
                        next_sync_committee,
                        ..
                    },
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: account_update_chain_id,
                t: AccountUpdateData {
                    slot: _account_update_data_beacon_slot,
                    // ibc_handler_address,
                    update: account_update,
                    __marker,
                },
                __marker: _,
            },
            Identified {
                chain_id: beacon_api_chain_id,
                t: BeaconGenesisData {
                    genesis,
                    __marker: _,
                },
                __marker: _,
            }
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(light_client_update_chain_id, account_update_chain_id);
        assert_eq!(chain_id, account_update_chain_id);
        assert_eq!(chain_id, beacon_api_chain_id);

        let header = ethereum::header::Header {
            consensus_update: light_client_update,
            trusted_sync_committee: TrustedSyncCommittee {
                trusted_height: Height {
                    revision_number: ETHEREUM_REVISION_NUMBER,
                    revision_height: currently_trusted_slot,
                },
                sync_committee: if is_next {
                    ActiveSyncCommittee::Next(next_sync_committee.unwrap())
                } else {
                    ActiveSyncCommittee::Current(next_sync_committee.unwrap())
                },
            },
            account_update,
        };

        seq([
            void(wait(id(
                req.counterparty_chain_id.clone(),
                WaitForTimestamp {
                    timestamp: (genesis.genesis_time
                        + (header.consensus_update.signature_slot * C::SECONDS_PER_SLOT::U64))
                        .try_into()
                        .unwrap(),
                    __marker: PhantomData,
                },
            ))),
            effect(id::<Tr, Ethereum<C>, _>(
                req.counterparty_chain_id,
                MsgUpdateClientData(MsgUpdateClient {
                    client_id: req.counterparty_client_id,
                    client_message: header,
                }),
            )),
        ])
    }
}

impl<C, Tr> UseAggregate<RelayMessageTypes>
    for Identified<Ethereum<C>, Tr, MakeCreateUpdatesData<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

    Identified<Ethereum<C>, Tr, FinalityUpdate<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,
{
    type AggregatedData = HList![Identified<Ethereum<C>, Tr, FinalityUpdate<C, Tr>>];

    fn aggregate(
        Identified {
            chain_id,
            t: MakeCreateUpdatesData { req },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: bootstrap_chain_id,
            t: FinalityUpdate {
                finality_update,
                __marker: _
            },
            __marker: _,
        },]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(chain_id, bootstrap_chain_id);

        let target_period =
            sync_committee_period::<_, C>(finality_update.attested_header.beacon.slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        assert!(
        trusted_period <= target_period,
        "trusted period {trusted_period} is behind target period {target_period}, something is wrong!",
    );

        // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
        // updates until we reach the `target_period - 1`.
        aggregate(
            [fetch(id::<Ethereum<C>, Tr, _>(
                chain_id,
                Fetch::specific(FetchLightClientUpdates {
                    trusted_period,
                    target_period,
                }),
            ))],
            [],
            id(
                chain_id,
                Aggregate::specific(MakeCreateUpdatesFromLightClientUpdatesData {
                    req: req.clone(),
                    trusted_height: req.update_from,
                    finality_update,
                }),
            ),
        )
    }
}

impl<C, Tr> UseAggregate<RelayMessageTypes>
    for Identified<Ethereum<C>, Tr, MakeCreateUpdatesFromLightClientUpdatesData<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt<SelfClientState: Encode<EthAbi>>,

    Identified<Ethereum<C>, Tr, LightClientUpdates<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Ethereum<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Ethereum<C>>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,
    // Identified<Ethereum<C>, Tr, LightClientUpdates<C, Tr>>:
    //     TryFrom<AnyLightClientIdentified<AnyData>>,
{
    type AggregatedData = HList![Identified<Ethereum<C>, Tr, LightClientUpdates<C, Tr>>];

    fn aggregate(
        Identified {
            chain_id,
            t:
                MakeCreateUpdatesFromLightClientUpdatesData {
                    req,
                    trusted_height,
                    finality_update,
                },
            __marker,
        }: Self,
        hlist_pat![Identified {
            chain_id: light_client_updates_chain_id,
            t: LightClientUpdates {
                light_client_updates,
                __marker: _,
            },
            __marker: _,
        },]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(chain_id, light_client_updates_chain_id);

        let target_period = sync_committee_period::<_, C>(finality_update.signature_slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        let (updates, last_update_block_number) = light_client_updates.into_iter().fold(
            (VecDeque::new(), trusted_height.revision_height),
            |(mut vec, mut trusted_slot), update| {
                let old_trusted_slot = trusted_slot;

                // REVIEW: Assert that this is greater (i.e. increasing)?
                trusted_slot = update.attested_header.beacon.slot;

                vec.push_back(make_create_update::<C, Tr>(
                    req.clone(),
                    chain_id,
                    old_trusted_slot,
                    update,
                    true,
                ));

                (vec, trusted_slot)
            },
        );

        let lc_updates = if trusted_period < target_period {
            updates
        } else {
            [].into()
        };

        let does_not_have_finality_update =
            last_update_block_number >= req.update_to.revision_height;

        debug!(last_update_block_number, req.update_to.revision_height);

        let finality_update_msg = if does_not_have_finality_update {
            // do nothing
            None
        } else {
            // do finality update
            Some(make_create_update(
                req.clone(),
                chain_id,
                last_update_block_number,
                light_client_update::LightClientUpdate {
                    attested_header: finality_update.attested_header,
                    next_sync_committee: None,
                    next_sync_committee_branch: None,
                    finalized_header: finality_update.finalized_header,
                    finality_branch: finality_update.finality_branch,
                    sync_aggregate: finality_update.sync_aggregate,
                    signature_slot: finality_update.signature_slot,
                },
                false,
            ))
        };

        seq(lc_updates.into_iter().chain(finality_update_msg))
    }
}
