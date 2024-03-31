use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, ops::Div, sync::Arc};

use chain_utils::{
    ethereum::{
        Ethereum, EthereumChain, EthereumSignerMiddleware, IbcHandlerErrors, IbcHandlerExt,
        ETHEREUM_REVISION_NUMBER,
    },
    Pool,
};
use contracts::ibc_handler::{
    self, AcknowledgePacketCall, ChannelOpenAckCall, ChannelOpenConfirmCall, ChannelOpenInitCall,
    ChannelOpenTryCall, ConnectionOpenAckCall, ConnectionOpenConfirmCall, ConnectionOpenInitCall,
    ConnectionOpenTryCall, CreateClientCall, IBCHandler, RecvPacketCall, UpdateClientCall,
};
use ethers::{
    abi::AbiEncode,
    contract::{ContractError, EthCall},
    providers::{Middleware, ProviderError},
    types::Bytes,
    utils::keccak256,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use macros::apply;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, effect, fetch, msg_struct, wait, QueueMsg,
};
use serde::{Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    encoding::{Decode, Encode, EncodeAs, EthAbi},
    ethereum::{
        beacon::{GenesisData, LightClientBootstrap, LightClientFinalityUpdate},
        config::ChainSpec,
    },
    hash::H160,
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
    proof::{ClientStatePath, Path},
    traits::{Chain, ClientIdOf, ClientState, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
    uint::U256,
    MaybeRecoverableError,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    data::{AnyData, Data, IbcProof, IbcState},
    effect::{
        AnyEffect, Effect, MsgConnectionOpenAckData, MsgConnectionOpenInitData,
        MsgConnectionOpenTryData, MsgUpdateClientData,
    },
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders},
    id, identified, seq,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForTimestamp},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayerMsgTypes,
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

impl<C: ChainSpec, Tr: ChainExt> DoMsg<Self, Tr> for Ethereum<C>
where
    ConsensusStateOf<Tr>: Encode<EthAbi>,
    ClientStateOf<Tr>: Encode<EthAbi>,
    HeaderOf<Tr>: Encode<EthAbi>,

    ClientStateOf<Ethereum<C>>: Encode<Tr::IbcStateEncoding>,
    Tr::StoredClientState<Ethereum<C>>: Encode<Tr::IbcStateEncoding>,
    Tr::StateProof: Encode<EthAbi>,
{
    async fn msg(&self, msg: Effect<Self, Tr>) -> Result<(), Self::MsgError> {
        do_msg(&self.ibc_handlers, msg).await
    }
}

pub async fn do_msg<Hc: ChainExt<Config = EthereumConfig> + EthereumChain, Tr: ChainExt>(
    ibc_handlers: &Pool<IBCHandler<EthereumSignerMiddleware>>,
    msg: Effect<Hc, Tr>,
) -> Result<(), TxSubmitError>
where
    ConsensusStateOf<Tr>: Encode<EthAbi>,
    ClientStateOf<Tr>: Encode<EthAbi>,
    HeaderOf<Tr>: Encode<EthAbi>,

    ClientStateOf<Hc>: Encode<Tr::IbcStateEncoding>,
    Tr::StoredClientState<Hc>: Encode<Tr::IbcStateEncoding>,
    Tr::StateProof: Encode<EthAbi>,
{
    let f = |ibc_handler| async move {
        let msg: ethers::contract::FunctionCall<_, _, ()> = match msg.clone() {
            Effect::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => mk_function_call(
                ibc_handler,
                ConnectionOpenInitCall {
                    msg: contracts::ibc_handler::MsgConnectionOpenInit {
                        client_id: data.client_id.to_string(),
                        counterparty: data.counterparty.into(),
                        delay_period: data.delay_period,
                    },
                },
            ),
            Effect::ConnectionOpenTry(MsgConnectionOpenTryData(data)) => mk_function_call(
                ibc_handler,
                ConnectionOpenTryCall {
                    msg: contracts::ibc_handler::MsgConnectionOpenTry {
                        counterparty: data.counterparty.into(),
                        delay_period: data.delay_period,
                        client_id: data.client_id.to_string(),
                        // needs to be encoded how the counterparty is encoding it
                        client_state_bytes: Encode::<Tr::IbcStateEncoding>::encode(
                            data.client_state,
                        )
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
                    },
                },
            ),
            Effect::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => mk_function_call(
                ibc_handler,
                ConnectionOpenAckCall {
                    msg: contracts::ibc_handler::MsgConnectionOpenAck {
                        connection_id: data.connection_id.to_string(),
                        counterparty_connection_id: data.counterparty_connection_id.to_string(),
                        version: data.version.into(),
                        // needs to be encoded how the counterparty is encoding it
                        client_state_bytes: Encode::<Tr::IbcStateEncoding>::encode(
                            data.client_state,
                        )
                        .into(),
                        proof_height: data.proof_height.into(),
                        proof_try: data.proof_try.encode().into(),
                        proof_client: data.proof_client.encode().into(),
                        proof_consensus: data.proof_consensus.encode().into(),
                        consensus_height: data.consensus_height.into(),
                    },
                },
            ),
            Effect::ConnectionOpenConfirm(data) => mk_function_call(
                ibc_handler,
                ConnectionOpenConfirmCall {
                    msg: contracts::ibc_handler::MsgConnectionOpenConfirm {
                        connection_id: data.msg.connection_id.to_string(),
                        proof_ack: data.msg.proof_ack.encode().into(),
                        proof_height: data.msg.proof_height.into_height().into(),
                    },
                },
            ),
            Effect::ChannelOpenInit(data) => mk_function_call(
                ibc_handler,
                ChannelOpenInitCall {
                    msg: contracts::ibc_handler::MsgChannelOpenInit {
                        port_id: data.msg.port_id.to_string(),
                        channel: data.msg.channel.into(),
                    },
                },
            ),
            Effect::ChannelOpenTry(data) => mk_function_call(
                ibc_handler,
                ChannelOpenTryCall {
                    msg: contracts::ibc_handler::MsgChannelOpenTry {
                        port_id: data.msg.port_id.to_string(),
                        channel: data.msg.channel.into(),
                        counterparty_version: data.msg.counterparty_version,
                        proof_init: data.msg.proof_init.encode().into(),
                        proof_height: data.msg.proof_height.into(),
                    },
                },
            ),
            Effect::ChannelOpenAck(data) => mk_function_call(
                ibc_handler,
                ChannelOpenAckCall {
                    msg: contracts::ibc_handler::MsgChannelOpenAck {
                        port_id: data.msg.port_id.to_string(),
                        channel_id: data.msg.channel_id.to_string(),
                        counterparty_version: data.msg.counterparty_version,
                        counterparty_channel_id: data.msg.counterparty_channel_id.to_string(),
                        proof_try: data.msg.proof_try.encode().into(),
                        proof_height: data.msg.proof_height.into_height().into(),
                    },
                },
            ),
            Effect::ChannelOpenConfirm(data) => mk_function_call(
                ibc_handler,
                ChannelOpenConfirmCall {
                    msg: contracts::ibc_handler::MsgChannelOpenConfirm {
                        port_id: data.msg.port_id.to_string(),
                        channel_id: data.msg.channel_id.to_string(),
                        proof_ack: data.msg.proof_ack.encode().into(),
                        proof_height: data.msg.proof_height.into_height().into(),
                    },
                },
            ),
            Effect::RecvPacket(data) => mk_function_call(
                ibc_handler,
                RecvPacketCall {
                    msg: contracts::ibc_handler::MsgPacketRecv {
                        packet: data.msg.packet.into(),
                        proof: data.msg.proof_commitment.encode().into(),
                        proof_height: data.msg.proof_height.into_height().into(),
                    },
                },
            ),
            Effect::AckPacket(data) => mk_function_call(
                ibc_handler,
                AcknowledgePacketCall {
                    msg: contracts::ibc_handler::MsgPacketAcknowledgement {
                        packet: data.msg.packet.into(),
                        acknowledgement: data.msg.acknowledgement.into(),
                        proof: data.msg.proof_acked.encode().into(),
                        proof_height: data.msg.proof_height.into_height().into(),
                    },
                },
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

                    Err(why) => tracing::info!(
                        "error registering client type, it is likely already registered: {:?}",
                        why
                    ),
                }

                mk_function_call(
                    ibc_handler,
                    CreateClientCall {
                        msg: contracts::shared_types::MsgCreateClient {
                            client_type: data.config.client_type,
                            client_state_bytes: data.msg.client_state.encode_as::<EthAbi>().into(),
                            consensus_state_bytes: data
                                .msg
                                .consensus_state
                                .encode_as::<EthAbi>()
                                .into(),
                        },
                    },
                )
            }
            Effect::UpdateClient(MsgUpdateClientData(data)) => mk_function_call(
                ibc_handler,
                UpdateClientCall {
                    msg: ibc_handler::MsgUpdateClient {
                        client_id: data.client_id.to_string(),
                        client_message: data.client_message.clone().encode_as::<EthAbi>().into(),
                    },
                },
            ),
        };

        tracing::debug!(?msg, "submitting evm tx");

        match msg.estimate_gas().await {
            Ok(estimated_gas) => {
                tracing::debug!(%estimated_gas, "gas estimation");

                // TODO: config
                let msg = msg.gas(estimated_gas + (estimated_gas / 10));
                let result = msg.send().await;
                match result {
                    Ok(ok) => {
                        tracing::info!("evm tx {:?} => {:?}", ok.tx_hash(), msg);
                        let tx_rcp = ok.await?.ok_or(TxSubmitError::NoTxReceipt)?;
                        tracing::info!(?tx_rcp, "evm transaction submitted");
                        Ok(())
                    }
                    Err(ContractError::Revert(revert)) => {
                        tracing::error!(?revert, "evm transaction failed");
                        let err =
                            <IbcHandlerErrors as ethers::abi::AbiDecode>::decode(revert.clone())
                                .map_err(|_| TxSubmitError::InvalidRevert(revert.clone()))?;
                        tracing::error!(?revert, ?err, "evm transaction failed");
                        Ok(())
                    }
                    _ => {
                        panic!("evm transaction non-recoverable failure");
                    }
                }
            }
            Err(ContractError::Revert(revert)) => {
                tracing::error!(?revert, "evm transaction failed");
                let err = <IbcHandlerErrors as ethers::abi::AbiDecode>::decode(revert.clone())
                    .map_err(|_| TxSubmitError::InvalidRevert(revert.clone()))?;
                tracing::error!(?revert, ?err, "evm estimation failed");
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
    ) -> QueueMsg<RelayerMsgTypes> {
        fetch(id::<Self, Tr, _>(
            c.chain_id(),
            Fetch::specific(GetProof { path, height: at }),
        ))
    }
}

impl<C: ChainSpec, Tr: ChainExt> DoFetchState<Self, Tr> for Ethereum<C>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    Tr::SelfClientState: Decode<<Ethereum<C> as Chain>::IbcStateEncoding>,

    Tr::SelfClientState: Encode<EthAbi>,
{
    fn state(
        hc: &Self,
        at: HeightOf<Self>,
        path: PathOf<Ethereum<C>, Tr>,
    ) -> QueueMsg<RelayerMsgTypes> {
        fetch(id::<Self, Tr, _>(
            hc.chain_id(),
            Fetch::specific(FetchIbcState { path, height: at }),
        ))
    }

    async fn query_client_state(
        hc: &Self,
        client_id: Self::ClientId,
        height: Self::Height,
    ) -> Tr::SelfClientState {
        hc.ibc_handler()
            .ibc_state_read::<_, Ethereum<C>, Tr>(
                hc.execution_height_of_beacon_slot(height.revision_height)
                    .await,
                ClientStatePath { client_id },
            )
            .await
            .unwrap()
    }
}

impl<C: ChainSpec, Tr: ChainExt> DoFetchUpdateHeaders<Self, Tr> for Ethereum<C>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,
{
    fn fetch_update_headers(
        c: &Self,
        update_info: FetchUpdateHeaders<Self, Tr>,
    ) -> QueueMsg<RelayerMsgTypes> {
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

impl<C: ChainSpec, Tr: ChainExt> DoFetch<Ethereum<C>> for EthereumFetchMsg<C, Tr>
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Ethereum<C>, Tr>)>,

    Tr::SelfClientState: Decode<<Ethereum<C> as Chain>::IbcStateEncoding>,
    Tr::SelfConsensusState: Decode<<Ethereum<C> as Chain>::IbcStateEncoding>,

    Tr::SelfClientState: Encode<EthAbi>,
{
    async fn do_fetch(ethereum: &Ethereum<C>, msg: Self) -> QueueMsg<RelayerMsgTypes> {
        let msg: EthereumFetchMsg<C, Tr> = msg;
        let msg = match msg {
            EthereumFetchMsg::FetchFinalityUpdate(FetchFinalityUpdate {}) => {
                Data::specific(FinalityUpdate {
                    finality_update: ethereum
                        .beacon_api_client
                        .finality_update()
                        .await
                        .unwrap()
                        .data,
                    __marker: PhantomData,
                })
            }
            EthereumFetchMsg::FetchLightClientUpdates(FetchLightClientUpdates {
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
            EthereumFetchMsg::FetchLightClientUpdate(FetchLightClientUpdate { period }) => {
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
            EthereumFetchMsg::FetchBootstrap(FetchBootstrap { slot }) => {
                Data::specific(BootstrapData {
                    slot,
                    bootstrap: ethereum
                        .beacon_api_client
                        .bootstrap_for_slot(slot)
                        .await
                        .unwrap()
                        .data,
                    __marker: PhantomData,
                })
            }
            EthereumFetchMsg::FetchAccountUpdate(FetchAccountUpdate { slot }) => {
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
            EthereumFetchMsg::FetchBeaconGenesis(_) => Data::specific(BeaconGenesisData {
                genesis: ethereum.beacon_api_client.genesis().await.unwrap().data,
                __marker: PhantomData,
            }),
            EthereumFetchMsg::FetchGetProof(get_proof) => {
                fetch_get_proof(ethereum, get_proof).await
            }
            EthereumFetchMsg::FetchIbcState(ibc_state) => {
                fetch_ibc_state(ethereum, ibc_state).await
            }
        };

        data(id::<Ethereum<C>, Tr, _>(ethereum.chain_id, msg))
    }
}

pub async fn fetch_get_proof<Hc, Tr>(c: &Hc, get_proof: GetProof<Hc, Tr>) -> Data<Hc, Tr>
where
    Hc: ChainExt + EthereumChain,
    Tr: ChainExt,
{
    let path = get_proof.path.to_string();

    let location = keccak256(
        keccak256(path.as_bytes())
            .into_iter()
            .chain(AbiEncode::encode(U256::from(0)))
            .collect::<Vec<_>>(),
    );

    let execution_height = c
        .execution_height_of_beacon_slot(get_proof.height.revision_height())
        .await;

    let proof = c
        .provider()
        .get_proof(
            ethers::types::H160::from(c.ibc_handler_address()),
            vec![location.into()],
            Some(execution_height.into()),
        )
        .await
        .unwrap();

    tracing::info!(?proof);

    let proof = match <[_; 1]>::try_from(proof.storage_proof) {
        Ok([proof]) => proof,
        Err(invalid) => {
            panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
        }
    };

    let proof = unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof {
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
    };

    match get_proof.path {
        Path::ClientStatePath(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::ClientConsensusStatePath(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::ConnectionPath(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::ChannelEndPath(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::CommitmentPath(path) => Data::from(IbcProof::<_, Hc, Tr> {
            proof,
            height: get_proof.height,
            path,
            __marker: PhantomData,
        }),
        Path::AcknowledgementPath(path) => Data::from(IbcProof::<_, Hc, Tr> {
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
    Hc: ChainExt + EthereumChain,
    Tr: ChainExt,
    Hc::StoredClientState<Tr>: Decode<Hc::IbcStateEncoding>,
    Hc::StoredConsensusState<Tr>: Decode<Hc::IbcStateEncoding>,
{
    let execution_height = c
        .execution_height_of_beacon_slot(height.revision_height())
        .await;

    match path {
        Path::ClientStatePath(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::ClientConsensusStatePath(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::ConnectionPath(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::ChannelEndPath(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::CommitmentPath(path) => Data::from(IbcState {
            state: c
                .ibc_handler()
                .ibc_state_read::<_, Hc, Tr>(execution_height, path.clone())
                .await
                .unwrap(),
            height,
            path,
        }),
        Path::AcknowledgementPath(path) => Data::from(IbcState {
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

#[apply(msg_struct)]
pub struct CreateUpdateData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Ethereum<C>, Tr>,
    pub currently_trusted_slot: u64,
    pub light_client_update: light_client_update::LightClientUpdate<C>,
    pub is_next: bool,
}

#[apply(msg_struct)]
pub struct MakeCreateUpdatesData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Ethereum<C>, Tr>,
}

#[apply(msg_struct)]
pub struct MakeCreateUpdatesFromLightClientUpdatesData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Ethereum<C>, Tr>,
    pub trusted_height: Height,
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[apply(msg_struct)]
pub struct FetchLightClientUpdate {
    pub period: u64,
}

#[apply(msg_struct)]
pub struct FetchFinalityUpdate {}

#[apply(msg_struct)]
pub struct FetchLightClientUpdates {
    pub trusted_period: u64,
    pub target_period: u64,
}

#[apply(msg_struct)]
pub struct FetchBootstrap {
    pub slot: u64,
}

#[apply(msg_struct)]
pub struct FetchAccountUpdate {
    pub slot: u64,
}

#[apply(msg_struct)]
pub struct FetchBeaconGenesis {}

#[apply(msg_struct)]
#[cover(Tr)]
pub struct BootstrapData<C: ChainSpec, Tr: ChainExt> {
    pub slot: u64,
    pub bootstrap: LightClientBootstrap<C>,
}

#[apply(msg_struct)]
#[cover(C, Tr)]
pub struct AccountUpdateData<C: ChainSpec, Tr: ChainExt> {
    pub slot: u64,
    pub update: AccountUpdate,
}

#[apply(msg_struct)]
#[cover(C, Tr)]
pub struct BeaconGenesisData<C: ChainSpec, Tr: ChainExt> {
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
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
pub enum EthereumFetchMsg<C: ChainSpec, Tr: ChainExt> {
    #[display(fmt = "FinalityUpdate")]
    FetchFinalityUpdate(FetchFinalityUpdate),
    #[display(fmt = "LightClientUpdates")]
    FetchLightClientUpdates(FetchLightClientUpdates),
    #[display(fmt = "LightClientUpdate")]
    FetchLightClientUpdate(FetchLightClientUpdate),
    #[display(fmt = "Bootstrap")]
    FetchBootstrap(FetchBootstrap),
    #[display(fmt = "AccountUpdate")]
    FetchAccountUpdate(FetchAccountUpdate),
    #[display(fmt = "BeaconGenesis")]
    FetchBeaconGenesis(FetchBeaconGenesis),
    #[display(fmt = "GetProof::{}", "_0.path")]
    FetchGetProof(GetProof<Ethereum<C>, Tr>),
    #[display(fmt = "IbcState::{}", "_0.path")]
    FetchIbcState(FetchIbcState<Ethereum<C>, Tr>),
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
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
#[allow(clippy::large_enum_variant)]
pub enum EthereumDataMsg<C: ChainSpec, Tr: ChainExt> {
    #[display(fmt = "FinalityUpdate")]
    FinalityUpdate(FinalityUpdate<C, Tr>),
    #[display(fmt = "LightClientUpdates")]
    LightClientUpdates(LightClientUpdates<C, Tr>),
    #[display(fmt = "LightClientUpdate")]
    LightClientUpdate(LightClientUpdate<C, Tr>),
    #[display(fmt = "Bootstrap")]
    Bootstrap(BootstrapData<C, Tr>),
    #[display(fmt = "AccountUpdate")]
    AccountUpdate(AccountUpdateData<C, Tr>),
    #[display(fmt = "BeaconGenesis")]
    BeaconGenesis(BeaconGenesisData<C, Tr>),
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
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
#[allow(clippy::large_enum_variant)]
pub enum EthereumAggregateMsg<C: ChainSpec, Tr: ChainExt> {
    #[display(fmt = "CreateUpdate")]
    CreateUpdate(CreateUpdateData<C, Tr>),
    #[display(fmt = "MakeCreateUpdates")]
    MakeCreateUpdates(MakeCreateUpdatesData<C, Tr>),
    #[display(fmt = "MakeCreateUpdatesFromLightClientUpdates")]
    MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<C, Tr>),
}

#[apply(msg_struct)]
#[cover(Tr)]
pub struct FinalityUpdate<C: ChainSpec, Tr: ChainExt> {
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[apply(msg_struct)]
#[cover(Tr)]
pub struct LightClientUpdates<C: ChainSpec, Tr: ChainExt> {
    pub light_client_updates: Vec<light_client_update::LightClientUpdate<C>>,
}

#[apply(msg_struct)]
#[cover(Tr)]
pub struct LightClientUpdate<C: ChainSpec, Tr: ChainExt> {
    pub update: light_client_update::LightClientUpdate<C>,
}

impl<C, Tr> DoAggregate for Identified<Ethereum<C>, Tr, EthereumAggregateMsg<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

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

    Tr::SelfClientState: Encode<EthAbi>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t: data,
            __marker: _,
        }: Self,
        aggregated_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayerMsgTypes> {
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
) -> QueueMsg<RelayerMsgTypes>
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

#[apply(msg_struct)]
pub struct GetProof<Hc: EthereumChainExt, Tr: ChainExt> {
    pub path: Path<ClientIdOf<Hc>, HeightOf<Tr>>,
    pub height: HeightOf<Hc>,
}

#[apply(msg_struct)]
pub struct FetchIbcState<Hc: EthereumChainExt, Tr: ChainExt> {
    pub path: Path<ClientIdOf<Hc>, HeightOf<Tr>>,
    pub height: HeightOf<Hc>,
}

impl<C, Tr> UseAggregate<RelayerMsgTypes> for Identified<Ethereum<C>, Tr, CreateUpdateData<C, Tr>>
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
    ) -> QueueMsg<RelayerMsgTypes> {
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
            wait(id(
                req.counterparty_chain_id.clone(),
                WaitForTimestamp {
                    timestamp: (genesis.genesis_time
                        + (header.consensus_update.signature_slot * C::SECONDS_PER_SLOT::U64))
                        .try_into()
                        .unwrap(),
                    __marker: PhantomData,
                },
            )),
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

impl<C, Tr> UseAggregate<RelayerMsgTypes>
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
    ) -> QueueMsg<RelayerMsgTypes> {
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

impl<C, Tr> UseAggregate<RelayerMsgTypes>
    for Identified<Ethereum<C>, Tr, MakeCreateUpdatesFromLightClientUpdatesData<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

    Identified<Ethereum<C>, Tr, LightClientUpdates<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Ethereum<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Ethereum<C>>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Ethereum<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Ethereum<C>, Tr>)>,

    Identified<Ethereum<C>, Tr, LightClientUpdates<C, Tr>>:
        TryFrom<AnyLightClientIdentified<AnyData>>,

    Tr::SelfClientState: Encode<EthAbi>,
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
    ) -> QueueMsg<RelayerMsgTypes> {
        assert_eq!(chain_id, light_client_updates_chain_id);

        let target_period = sync_committee_period::<_, C>(finality_update.signature_slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        let (updates, last_update_block_number) = light_client_updates.into_iter().fold(
            (VecDeque::new(), trusted_height.revision_height),
            |(mut vec, mut trusted_slot), update| {
                let old_trusted_slot = trusted_slot;

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

        tracing::debug!(last_update_block_number, req.update_to.revision_height);

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
