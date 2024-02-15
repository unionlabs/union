use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, ops::Div, sync::Arc};

use beacon_api::errors::{InternalServerError, NotFoundError};
use chain_utils::evm::{CometblsMiddleware, Evm};
use contracts::ibc_handler::{
    self, AcknowledgePacketCall, ChannelOpenAckCall, ChannelOpenConfirmCall, ChannelOpenInitCall,
    ChannelOpenTryCall, ConnectionOpenAckCall, ConnectionOpenConfirmCall, ConnectionOpenInitCall,
    ConnectionOpenTryCall, CreateClientCall, IBCHandler, RecvPacketCall, UpdateClientCall,
};
use ethers::{
    abi::AbiEncode,
    contract::{ContractError, EthCall},
    providers::{Middleware, ProviderError},
    utils::keccak256,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    data, fetch, msg, wait,
};
use serde::{Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    encoding::{Decode, Encode, EthAbi},
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
    IntoEthAbi, MaybeRecoverableError,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
    data::{AnyData, Data, IbcProof, IbcState, LightClientSpecificData},
    fetch::{AnyFetch, DoFetch, Fetch, FetchUpdateHeaders, LightClientSpecificFetch},
    identified,
    msg::{
        AnyMsg, Msg, MsgConnectionOpenAckData, MsgConnectionOpenInitData, MsgConnectionOpenTryData,
        MsgUpdateClientData,
    },
    seq,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForTimestamp},
    AnyLightClientIdentified, ChainExt, DoAggregate, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, DoMsg, Identified, PathOf, RelayerMsg, RelayerMsgTypes,
};

pub const EVM_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct EvmConfig {
    pub client_type: String,
    pub client_address: H160,
}

impl<C: ChainSpec> ChainExt for Evm<C> {
    type Data<Tr: ChainExt> = EvmDataMsg<C, Tr>;
    type Fetch<Tr: ChainExt> = EvmFetchMsg<C, Tr>;
    type Aggregate<Tr: ChainExt> = EvmAggregateMsg<C, Tr>;

    type MsgError = TxSubmitError;

    type Config = EvmConfig;
}

impl<C: ChainSpec, Tr: ChainExt> DoMsg<Self, Tr> for Evm<C>
where
    ConsensusStateOf<Tr>: IntoEthAbi,
    ClientStateOf<Tr>: IntoEthAbi,
    HeaderOf<Tr>: IntoEthAbi,

    ClientStateOf<Evm<C>>: Encode<Tr::IbcStateEncoding>,
    Tr::StoredClientState<Evm<C>>: Encode<Tr::IbcStateEncoding>,
    Tr::StateProof: Encode<EthAbi>,
{
    async fn msg(&self, msg: Msg<Self, Tr>) -> Result<(), Self::MsgError> {
        let f = |ibc_handler| async move {
            let msg: ethers::contract::FunctionCall<_, _, ()> = match msg.clone() {
                Msg::ConnectionOpenInit(MsgConnectionOpenInitData(data)) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenInitCall {
                        msg: contracts::ibc_handler::MsgConnectionOpenInit {
                            client_id: data.client_id.to_string(),
                            counterparty: data.counterparty.into(),
                            delay_period: data.delay_period,
                        },
                    },
                ),
                Msg::ConnectionOpenTry(MsgConnectionOpenTryData(data)) => mk_function_call(
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
                            consensus_height: data.consensus_height.into(),
                        },
                    },
                ),
                Msg::ConnectionOpenAck(MsgConnectionOpenAckData(data)) => mk_function_call(
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
                Msg::ConnectionOpenConfirm(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenConfirmCall {
                        msg: contracts::ibc_handler::MsgConnectionOpenConfirm {
                            connection_id: data.msg.connection_id.to_string(),
                            proof_ack: data.msg.proof_ack.encode().into(),
                            proof_height: data.msg.proof_height.into_height().into(),
                        },
                    },
                ),
                Msg::ChannelOpenInit(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenInitCall {
                        msg: contracts::ibc_handler::MsgChannelOpenInit {
                            port_id: data.msg.port_id.to_string(),
                            channel: data.msg.channel.into(),
                        },
                    },
                ),
                Msg::ChannelOpenTry(data) => mk_function_call(
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
                Msg::ChannelOpenAck(data) => mk_function_call(
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
                Msg::ChannelOpenConfirm(data) => mk_function_call(
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
                Msg::RecvPacket(data) => mk_function_call(
                    ibc_handler,
                    RecvPacketCall {
                        msg: contracts::ibc_handler::MsgPacketRecv {
                            packet: data.msg.packet.into(),
                            proof: data.msg.proof_commitment.encode().into(),
                            proof_height: data.msg.proof_height.into_height().into(),
                        },
                    },
                ),
                Msg::AckPacket(data) => mk_function_call(
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
                Msg::CreateClient(data) => {
                    let register_client_result = ibc_handler.register_client(
                        data.config.client_type.clone(),
                        data.config.client_address.clone().into(),
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
                                client_state_bytes: data
                                    .msg
                                    .client_state
                                    .into_eth_abi_bytes()
                                    .into(),
                                consensus_state_bytes: data
                                    .msg
                                    .consensus_state
                                    .into_eth_abi_bytes()
                                    .into(),
                            },
                        },
                    )
                }
                Msg::UpdateClient(MsgUpdateClientData(data)) => mk_function_call(
                    ibc_handler,
                    UpdateClientCall {
                        msg: ibc_handler::MsgUpdateClient {
                            client_id: data.client_id.to_string(),
                            client_message: data.client_message.clone().into_eth_abi_bytes().into(),
                        },
                    },
                ),
            };

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
                    Ok(())
                }
                _ => {
                    panic!("evm transaction non-recoverable failure");
                }
            }
        };

        self.ibc_handlers.with(f).await
    }
}

impl<C: ChainSpec, Tr: ChainExt> DoFetchProof<Self, Tr> for Evm<C>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
{
    fn proof(c: &Self, at: HeightOf<Self>, path: PathOf<Evm<C>, Tr>) -> RelayerMsg {
        fetch(Identified::<Self, Tr, _>::new(
            c.chain_id(),
            LightClientSpecificFetch::<Self, Tr>(EvmFetchMsg::FetchGetProof(GetProof {
                path,
                height: at,
            }))
            .into(),
        ))
    }
}

impl<C: ChainSpec, Tr: ChainExt> DoFetchState<Self, Tr> for Evm<C>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
    Tr::SelfClientState: Decode<<Evm<C> as Chain>::IbcStateEncoding>,

    Tr::SelfClientState: Encode<EthAbi>,
    Tr::SelfClientState: unionlabs::EthAbi,
    Tr::SelfClientState: TryFrom<<Tr::SelfClientState as unionlabs::EthAbi>::EthAbi>,
    <Tr::SelfClientState as unionlabs::EthAbi>::EthAbi: From<Tr::SelfClientState>,
{
    fn state(hc: &Self, at: HeightOf<Self>, path: PathOf<Evm<C>, Tr>) -> RelayerMsg {
        fetch(Identified::<Self, Tr, _>::new(
            hc.chain_id(),
            LightClientSpecificFetch::<Self, Tr>(EvmFetchMsg::FetchIbcState(FetchIbcState {
                path,
                height: at,
            }))
            .into(),
        ))
    }

    async fn query_client_state(
        hc: &Self,
        client_id: Self::ClientId,
        height: Self::Height,
    ) -> Tr::SelfClientState {
        hc.ibc_state_read::<_, Tr>(height, ClientStatePath { client_id })
            .await
            .unwrap()
    }
}

impl<C: ChainSpec, Tr: ChainExt> DoFetchUpdateHeaders<Self, Tr> for Evm<C>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Evm<C>, Tr>)>,
{
    fn fetch_update_headers(c: &Self, update_info: FetchUpdateHeaders<Self, Tr>) -> RelayerMsg {
        aggregate(
            [fetch(Identified::<Evm<C>, Tr, _>::new(
                c.chain_id,
                LightClientSpecificFetch(EvmFetchMsg::FetchFinalityUpdate(PhantomData)).into(),
            ))],
            [],
            Identified::new(
                c.chain_id,
                LightClientSpecificAggregate(EvmAggregateMsg::MakeCreateUpdates(
                    MakeCreateUpdatesData { req: update_info },
                )),
            ),
        )
    }
}

impl<C: ChainSpec, Tr: ChainExt> DoFetch<Evm<C>> for EvmFetchMsg<C, Tr>
where
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Evm<C>, Tr>)>,

    Tr::SelfClientState: Decode<<Evm<C> as Chain>::IbcStateEncoding>,
    Tr::SelfConsensusState: Decode<<Evm<C> as Chain>::IbcStateEncoding>,

    Tr::SelfClientState: unionlabs::EthAbi,
    <Tr::SelfClientState as unionlabs::EthAbi>::EthAbi: From<Tr::SelfClientState>,
{
    async fn do_fetch(c: &Evm<C>, msg: Self) -> RelayerMsg {
        let msg: EvmFetchMsg<C, Tr> = msg;
        let msg = match msg {
            EvmFetchMsg::FetchFinalityUpdate(PhantomData {}) => {
                EvmDataMsg::FinalityUpdate(FinalityUpdate {
                    finality_update: c.beacon_api_client.finality_update().await.unwrap().data,
                    __marker: PhantomData,
                })
            }
            EvmFetchMsg::FetchLightClientUpdates(FetchLightClientUpdates {
                trusted_period,
                target_period,
                __marker: PhantomData,
            }) => EvmDataMsg::LightClientUpdates(LightClientUpdates {
                light_client_updates: c
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
            EvmFetchMsg::FetchLightClientUpdate(FetchLightClientUpdate {
                period,
                __marker: PhantomData,
            }) => EvmDataMsg::LightClientUpdate(LightClientUpdate {
                update: c
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
            }),
            EvmFetchMsg::FetchBootstrap(FetchBootstrap { slot, __marker: _ }) => {
                // NOTE(benluelo): While this is technically two actions, I consider it to be one
                // action - if the beacon chain doesn't have the header, it won't have the bootstrap
                // either. It would be nice if the beacon chain exposed "fetch bootstrap by slot"
                // functionality; I'm surprised it doesn't.

                let mut amount_of_slots_back: u64 = 0;

                let floored_slot = slot
                    / (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64)
                    * (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64);

                tracing::info!("fetching bootstrap at {}", floored_slot);

                let bootstrap = loop {
                    let header_response = c
                        .beacon_api_client
                        .header(beacon_api::client::BlockId::Slot(
                            floored_slot - amount_of_slots_back,
                        ))
                        .await;

                    let header = match header_response {
                        Ok(header) => header,
                        Err(beacon_api::errors::Error::NotFound(NotFoundError {
                            status_code: _,
                            error: _,
                            message,
                        })) if message.starts_with("No block found for id") => {
                            amount_of_slots_back += 1;
                            continue;
                        }

                        Err(err) => panic!("{err}"),
                    };

                    let bootstrap_response = c
                        .beacon_api_client
                        .bootstrap(header.data.root.clone())
                        .await;

                    match bootstrap_response {
                        Ok(ok) => break ok.data,
                        Err(err) => match err {
                            beacon_api::errors::Error::Internal(InternalServerError {
                                status_code: _,
                                error: _,
                                message,
                            }) if message.starts_with("syncCommitteeWitness not available") => {
                                amount_of_slots_back += 1;
                            }
                            _ => panic!("{err}"),
                        },
                    };
                };

                // bootstrap contains the current sync committee for the given height
                EvmDataMsg::Bootstrap(BootstrapData {
                    slot,
                    bootstrap,
                    __marker: PhantomData,
                })
            }
            EvmFetchMsg::FetchAccountUpdate(FetchAccountUpdate { slot, __marker: _ }) => {
                let execution_height = c
                    .execution_height(Height {
                        revision_number: EVM_REVISION_NUMBER,
                        revision_height: slot,
                    })
                    .await;

                let account_update = c
                    .provider
                    .get_proof(
                        c.readonly_ibc_handler.address(),
                        vec![],
                        // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                        Some(execution_height.into()),
                    )
                    .await
                    .unwrap();

                EvmDataMsg::AccountUpdate(AccountUpdateData {
                    slot,
                    update: AccountUpdate {
                        account_proof: AccountProof {
                            contract_address: c.readonly_ibc_handler.address().0.into(),
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
            EvmFetchMsg::FetchBeaconGenesis(_) => EvmDataMsg::BeaconGenesis(BeaconGenesisData {
                genesis: c.beacon_api_client.genesis().await.unwrap().data,
                __marker: PhantomData,
            }),
            EvmFetchMsg::FetchGetProof(get_proof) => {
                let execution_height = c.execution_height(get_proof.height).await;

                let path = get_proof.path.to_string();

                let location = keccak256(
                    keccak256(path.as_bytes())
                        .into_iter()
                        .chain(ethers::types::U256::from(0).encode())
                        .collect::<Vec<_>>(),
                );

                let proof = c
                    .provider
                    .get_proof(
                        c.readonly_ibc_handler.address(),
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

                return match get_proof.path {
                    Path::ClientStatePath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcProof::<Evm<C>, Tr, _> {
                            proof,
                            height: get_proof.height,
                            path,
                            __marker: PhantomData,
                        },
                    )),
                    Path::ClientConsensusStatePath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcProof::<Evm<C>, Tr, _> {
                            proof,
                            height: get_proof.height,
                            path,
                            __marker: PhantomData,
                        },
                    )),
                    Path::ConnectionPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcProof::<Evm<C>, Tr, _> {
                            proof,
                            height: get_proof.height,
                            path,
                            __marker: PhantomData,
                        },
                    )),
                    Path::ChannelEndPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcProof::<Evm<C>, Tr, _> {
                            proof,
                            height: get_proof.height,
                            path,
                            __marker: PhantomData,
                        },
                    )),
                    Path::CommitmentPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcProof::<Evm<C>, Tr, _> {
                            proof,
                            height: get_proof.height,
                            path,
                            __marker: PhantomData,
                        },
                    )),
                    Path::AcknowledgementPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcProof::<Evm<C>, Tr, _> {
                            proof,
                            height: get_proof.height,
                            path,
                            __marker: PhantomData,
                        },
                    )),
                };
            }
            EvmFetchMsg::FetchIbcState(get_storage_at) => {
                return match get_storage_at.path {
                    Path::ClientStatePath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcState {
                            state: c
                                .ibc_state_read::<_, Tr>(get_storage_at.height, path.clone())
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::ClientConsensusStatePath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcState {
                            state: c
                                .ibc_state_read::<_, Tr>(get_storage_at.height, path.clone())
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::ConnectionPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcState {
                            state: c
                                .ibc_state_read::<_, Tr>(get_storage_at.height, path.clone())
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::ChannelEndPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcState {
                            state: c
                                .ibc_state_read::<_, Tr>(get_storage_at.height, path.clone())
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::CommitmentPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcState {
                            state: c
                                .ibc_state_read::<_, Tr>(get_storage_at.height, path.clone())
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                    Path::AcknowledgementPath(path) => data(Identified::<Evm<C>, Tr, _>::new(
                        c.chain_id,
                        IbcState {
                            state: c
                                .ibc_state_read::<_, Tr>(get_storage_at.height, path.clone())
                                .await
                                .unwrap(),
                            height: get_storage_at.height,
                            path,
                        },
                    )),
                };
            }
        };

        data(Identified::<Evm<C>, Tr, _>::new(
            c.chain_id,
            LightClientSpecificData(msg),
        ))
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
pub struct CreateUpdateData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Evm<C>, Tr>,
    pub currently_trusted_slot: u64,
    pub light_client_update: light_client_update::LightClientUpdate<C>,
    pub is_next: bool,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]

pub struct MakeCreateUpdatesData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Evm<C>, Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
pub struct MakeCreateUpdatesFromLightClientUpdatesData<C: ChainSpec, Tr: ChainExt> {
    pub req: FetchUpdateHeaders<Evm<C>, Tr>,
    pub trusted_height: Height,
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
pub struct FetchLightClientUpdate<C: ChainSpec> {
    pub period: u64,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
pub struct FetchLightClientUpdates<C: ChainSpec> {
    pub trusted_period: u64,
    pub target_period: u64,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
pub struct FetchBootstrap<C: ChainSpec> {
    pub slot: u64,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
pub struct FetchAccountUpdate<C: ChainSpec> {
    pub slot: u64,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
pub struct FetchBeaconGenesis<C: ChainSpec> {
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
pub struct BootstrapData<C: ChainSpec, Tr: ChainExt> {
    pub slot: u64,
    pub bootstrap: LightClientBootstrap<C>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
pub struct AccountUpdateData<C: ChainSpec, Tr: ChainExt> {
    pub slot: u64,
    // pub ibc_handler_address: H160,
    pub update: AccountUpdate,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (C, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]
pub struct BeaconGenesisData<C: ChainSpec, Tr: ChainExt> {
    genesis: GenesisData,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (C, Tr)>,
}

try_from_relayer_msg! {
    chain = Evm<C>,
    generics = (C: ChainSpec, Tr: ChainExt),
    msgs = EvmDataMsg(
        FinalityUpdate(FinalityUpdate<C, Tr>),
        LightClientUpdates(LightClientUpdates<C, Tr>),
        LightClientUpdate(LightClientUpdate<C, Tr>),
        Bootstrap(BootstrapData<C, Tr>),
        AccountUpdate(AccountUpdateData<C, Tr>),
        BeaconGenesis(BeaconGenesisData<C, Tr>),
    ),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
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
pub enum EvmFetchMsg<C: ChainSpec, Tr: ChainExt> {
    #[display(fmt = "FinalityUpdate")]
    FetchFinalityUpdate(PhantomData<C>),
    #[display(fmt = "LightClientUpdates")]
    FetchLightClientUpdates(FetchLightClientUpdates<C>),
    #[display(fmt = "LightClientUpdate")]
    FetchLightClientUpdate(FetchLightClientUpdate<C>),
    #[display(fmt = "Bootstrap")]
    FetchBootstrap(FetchBootstrap<C>),
    #[display(fmt = "AccountUpdate")]
    FetchAccountUpdate(FetchAccountUpdate<C>),
    #[display(fmt = "BeaconGenesis")]
    FetchBeaconGenesis(FetchBeaconGenesis<C>),
    #[display(fmt = "GetProof::{}", "_0.path")]
    FetchGetProof(GetProof<C, Tr>),
    #[display(fmt = "IbcState::{}", "_0.path")]
    FetchIbcState(FetchIbcState<C, Tr>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
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
pub enum EvmDataMsg<C: ChainSpec, Tr: ChainExt> {
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
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
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
pub enum EvmAggregateMsg<C: ChainSpec, Tr: ChainExt> {
    #[display(fmt = "CreateUpdate")]
    CreateUpdate(CreateUpdateData<C, Tr>),
    #[display(fmt = "MakeCreateUpdates")]
    MakeCreateUpdates(MakeCreateUpdatesData<C, Tr>),
    #[display(fmt = "MakeCreateUpdatesFromLightClientUpdates")]
    MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<C, Tr>),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]

pub struct FinalityUpdate<C: ChainSpec, Tr: ChainExt> {
    pub finality_update: LightClientFinalityUpdate<C>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]

pub struct LightClientUpdates<C: ChainSpec, Tr: ChainExt> {
    pub light_client_updates: Vec<light_client_update::LightClientUpdate<C>>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]

pub struct LightClientUpdate<C: ChainSpec, Tr: ChainExt> {
    pub update: light_client_update::LightClientUpdate<C>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Tr>,
}

impl<C, Tr> DoAggregate for Identified<Evm<C>, Tr, EvmAggregateMsg<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

    Identified<Evm<C>, Tr, AccountUpdateData<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, BootstrapData<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, BeaconGenesisData<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, FinalityUpdate<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, LightClientUpdates<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, LightClientUpdate<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Evm<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Evm<C>>)>,

    AnyLightClientIdentified<AnyData>: From<identified!(Data<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Evm<C>, Tr>)>,

    Tr::SelfClientState: unionlabs::EthAbi,
    <Tr::SelfClientState as unionlabs::EthAbi>::EthAbi: From<Tr::SelfClientState>,
{
    fn do_aggregate(
        Identified {
            chain_id,
            t: data,
            __marker: _,
        }: Self,
        aggregated_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> RelayerMsg {
        match data {
            EvmAggregateMsg::CreateUpdate(msg) => {
                do_aggregate(Identified::new(chain_id, msg), aggregated_data)
            }
            EvmAggregateMsg::MakeCreateUpdates(msg) => {
                do_aggregate(Identified::new(chain_id, msg), aggregated_data)
            }
            EvmAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(msg) => {
                do_aggregate(Identified::new(chain_id, msg), aggregated_data)
            }
        }
    }
}

fn make_create_update<C, Tr>(
    req: FetchUpdateHeaders<Evm<C>, Tr>,
    chain_id: <<Evm<C> as Chain>::SelfClientState as ClientState>::ChainId,
    currently_trusted_slot: u64,
    light_client_update: light_client_update::LightClientUpdate<C>,
    is_next: bool,
) -> RelayerMsg
where
    C: ChainSpec,
    Tr: ChainExt,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Evm<C>, Tr>)>,
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
            fetch(Identified::<Evm<C>, Tr, _>::new(
                chain_id,
                LightClientSpecificFetch(EvmFetchMsg::FetchLightClientUpdate(
                    FetchLightClientUpdate {
                        period: previous_period,
                        __marker: PhantomData,
                    },
                )),
            )),
            fetch(Identified::<Evm<C>, Tr, _>::new(
                chain_id,
                LightClientSpecificFetch(EvmFetchMsg::FetchAccountUpdate(FetchAccountUpdate {
                    slot: light_client_update.attested_header.beacon.slot,
                    __marker: PhantomData,
                })),
            )),
            fetch(Identified::<Evm<C>, Tr, _>::new(
                chain_id,
                LightClientSpecificFetch(EvmFetchMsg::FetchBeaconGenesis(FetchBeaconGenesis {
                    __marker: PhantomData,
                })),
            )),
        ],
        [],
        Identified::new(
            chain_id,
            LightClientSpecificAggregate(EvmAggregateMsg::CreateUpdate(CreateUpdateData {
                req,
                currently_trusted_slot,
                light_client_update,
                is_next,
            })),
        ),
    )
}

fn sync_committee_period<H: Into<u64>, C: ChainSpec>(height: H) -> u64 {
    height.into().div(C::PERIOD::U64)
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Contract(#[from] ContractError<CometblsMiddleware>),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("no tx receipt from tx")]
    NoTxReceipt,
}

impl MaybeRecoverableError for TxSubmitError {
    fn is_recoverable(&self) -> bool {
        // TODO: Figure out if any failures are unrecoverable
        true
    }
}

fn mk_function_call<Call: EthCall>(
    ibc_handler: IBCHandler<CometblsMiddleware>,
    data: Call,
) -> ethers::contract::FunctionCall<Arc<CometblsMiddleware>, CometblsMiddleware, ()> {
    ibc_handler
        .method_hash(<Call as EthCall>::selector(), data)
        .expect("method selector is generated; qed;")
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]

pub struct GetProof<C: ChainSpec, Tr: ChainExt> {
    path: Path<ClientIdOf<Evm<C>>, HeightOf<Tr>>,
    height: HeightOf<Evm<C>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec, Tr: ChainExt")
)]

pub struct FetchIbcState<C: ChainSpec, Tr: ChainExt> {
    path: Path<ClientIdOf<Evm<C>>, HeightOf<Tr>>,
    height: HeightOf<Evm<C>>,
}

impl<C, Tr> UseAggregate<RelayerMsgTypes> for Identified<Evm<C>, Tr, CreateUpdateData<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

    Identified<Evm<C>, Tr, AccountUpdateData<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, LightClientUpdate<C, Tr>>: IsAggregateData,
    Identified<Evm<C>, Tr, BeaconGenesisData<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Evm<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Evm<C>>)>,
{
    type AggregatedData = HList![
        Identified<Evm<C>, Tr, LightClientUpdate<C, Tr>>,
        Identified<Evm<C>, Tr, AccountUpdateData<C, Tr>>,
        Identified<Evm<C>, Tr, BeaconGenesisData<C, Tr>>
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
    ) -> RelayerMsg {
        assert_eq!(light_client_update_chain_id, account_update_chain_id);
        assert_eq!(chain_id, account_update_chain_id);
        assert_eq!(chain_id, beacon_api_chain_id);

        let header = ethereum::header::Header {
            consensus_update: light_client_update,
            trusted_sync_committee: TrustedSyncCommittee {
                trusted_height: Height {
                    revision_number: EVM_REVISION_NUMBER,
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
            wait(Identified::new(
                req.counterparty_chain_id.clone(),
                WaitForTimestamp {
                    timestamp: (genesis.genesis_time
                        + (header.consensus_update.signature_slot * C::SECONDS_PER_SLOT::U64))
                        .try_into()
                        .unwrap(),
                    __marker: PhantomData,
                },
            )),
            msg(Identified::<Tr, Evm<C>, _>::new(
                req.counterparty_chain_id,
                MsgUpdateClientData(MsgUpdateClient {
                    client_id: req.counterparty_client_id,
                    client_message: header,
                }),
            )),
        ])
    }
}

impl<C, Tr> UseAggregate<RelayerMsgTypes> for Identified<Evm<C>, Tr, MakeCreateUpdatesData<C, Tr>>
where
    C: ChainSpec,

    Tr: ChainExt,

    Identified<Evm<C>, Tr, FinalityUpdate<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Evm<C>, Tr>)>,
{
    type AggregatedData = HList![Identified<Evm<C>, Tr, FinalityUpdate<C, Tr>>];

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
    ) -> RelayerMsg {
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
            [fetch(Identified::<Evm<C>, Tr, _>::new(
                chain_id,
                LightClientSpecificFetch(EvmFetchMsg::FetchLightClientUpdates(
                    FetchLightClientUpdates {
                        trusted_period,
                        target_period,
                        __marker: PhantomData,
                    },
                )),
            ))],
            [],
            Identified::new(
                chain_id,
                LightClientSpecificAggregate(
                    EvmAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(
                        MakeCreateUpdatesFromLightClientUpdatesData {
                            req: req.clone(),
                            trusted_height: req.update_from,
                            finality_update,
                        },
                    ),
                ),
            ),
        )
    }
}

impl<C, Tr> UseAggregate<RelayerMsgTypes>
    for Identified<Evm<C>, Tr, MakeCreateUpdatesFromLightClientUpdatesData<C, Tr>>
where
    C: ChainSpec,
    Tr: ChainExt,

    Identified<Evm<C>, Tr, LightClientUpdates<C, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Evm<C>>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Evm<C>>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Evm<C>, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Evm<C>, Tr>)>,

    Identified<Evm<C>, Tr, LightClientUpdates<C, Tr>>: TryFrom<AnyLightClientIdentified<AnyData>>,

    Tr::SelfClientState: Encode<EthAbi>,
    Tr::SelfClientState: unionlabs::EthAbi,
    <Tr::SelfClientState as unionlabs::EthAbi>::EthAbi: From<Tr::SelfClientState>,
{
    type AggregatedData = HList![Identified<Evm<C>, Tr, LightClientUpdates<C, Tr>>];

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
    ) -> RelayerMsg {
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

        tracing::error!(last_update_block_number, req.update_to.revision_height);

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
