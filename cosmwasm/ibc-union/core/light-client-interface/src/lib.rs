#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use core::fmt::Debug;
use std::error::Error;

use access_managed::{EnsureCanCallResult, state::Authority};
use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Env, MessageInfo, Querier, Response, StdError, to_json_binary,
};
use depolama::{QuerierExt, StorageExt, Store};
use frissitheto::UpgradeError;
use ibc_union::state::{ClientConsensusStates, ClientImpls, ClientStates, ClientStore, QueryStore};
use ibc_union_msg::lightclient::{
    ExecuteMsg, MisbehaviourQuery, MisbehaviourResponse, QueryMsg, StorageWrites, UpdateStateQuery,
    UpdateStateResponse, VerificationQueryMsg, VerifyCreationQuery, VerifyCreationResponse,
    VerifyCreationResponseEvent, VerifyMembershipQuery, VerifyNonMembershipQuery,
};
use ibc_union_spec::{ClientId, Status, Timestamp};
use unionlabs::{
    ErrorReporter,
    encoding::{Decode, DecodeAs, DecodeErrorOf, Encode, EncodeAs, Encoding, EthAbi},
    primitives::Bytes,
};

use crate::{msg::InitMsg, state::IbcHost};

pub mod state;

pub use access_managed;
pub use ibc_union_msg::lightclient as msg;
pub use ibc_union_spec as spec;
pub use pausable;
pub use upgradable;

#[macro_export]
macro_rules! entrypoints {
    ($LightClient:ty) => {
        $crate::default_query!($LightClient);
        $crate::default_execute!($LightClient);
        $crate::default_migrate!($LightClient);
        $crate::default_reply!();
    };
    ($LightClient:ty; library) => {
        $crate::default_query!($LightClient, library);
        $crate::default_execute!($LightClient, library);
        $crate::default_migrate!($LightClient, library);
        $crate::default_reply!(library);
    };
}

#[macro_export]
macro_rules! default_query {
    ($LightClient:ty) => {
        $crate::default_query!(
            $LightClient;
            #[::cosmwasm_std::entry_point]
        );
    };
    ($LightClient:ty; library) => {
        $crate::default_query!(
            $LightClient;
            #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        );
    };
    ($LightClient:ty; #[$meta:meta]) => {
        #[$meta]
        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: $crate::msg::QueryMsg,
        ) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Binary> {
            $crate::query::<$LightClient>(deps, env, msg).map_err(Into::into)
        }
    };
}

#[macro_export]
macro_rules! default_execute {
    ($LightClient:ty) => {
        $crate::default_execute!(
            $LightClient;
            #[::cosmwasm_std::entry_point]
        );
    };
    ($LightClient:ty; library) => {
        $crate::default_execute!(
            $LightClient;
            #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        );
    };
    ($LightClient:ty; #[$meta:meta]) => {
        #[$meta]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut<<$LightClient as $crate::IbcClient>::CustomQuery>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: $crate::msg::ExecuteMsg,
        ) -> ::core::result::Result<::cosmwasm_std::Response<<$LightClient as $crate::IbcClient>::CustomQuery>, $crate::IbcClientError<$LightClient>> {
            $crate::execute::<$LightClient>(deps, env, info, msg).map_err(Into::into)
        }
    };
}

/// Major state versions of this contract, used in the [`frissitheto`] migrations.
pub mod version {
    use std::num::NonZeroU32;

    /// Initial state of the contract. Access management is handled internally in this contract for specific endpoints.
    pub const INIT: NonZeroU32 = NonZeroU32::new(1).unwrap();

    /// Same as [`INIT`], except that access management is handled externally via [`access_managed`]. All storage in this contract relating to internally handled access management has been removed, and additional storages for [`access_managed`] have been added.
    ///
    /// This is the current latest state version of this contract.
    pub const MANAGED: NonZeroU32 = NonZeroU32::new(2).unwrap();

    /// The latest state version of this contract. Any new deployments will be init'd with this version and the corresponding state.
    pub const LATEST: NonZeroU32 = MANAGED;
}

#[macro_export]
macro_rules! default_migrate {
    ($LightClient:ty) => {
        $crate::default_migrate!(
            $LightClient;
            #[::cosmwasm_std::entry_point]
        );
    };
    ($LightClient:ty, library) => {
        $crate::default_migrate!(
            $LightClient;
            #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        );
    };
    ($LightClient:ty; #[$meta:meta]) => {
        #[$meta]
        pub fn migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::frissitheto::UpgradeMsg<$crate::msg::InitMsg, $crate::msg::MigrateMsg>,
        ) -> Result<::cosmwasm_std::Response, $crate::IbcClientError<$LightClient>> {
            msg.run(
                deps,
                |deps, init_msg| {
                    let res = $crate::init(deps, init_msg)?;

                    Ok((res, Some($crate::version::LATEST)))
                },
                |_, _, version| match version {
                    $crate::version::INIT => Err(::cosmwasm_std::StdError::generic_err("unsupported version: INIT").into()),
                    $crate::version::MANAGED => Ok((::cosmwasm_std::Response::default(), None)),
                    _ => Err(::frissitheto::UpgradeError::UnknownStateVersion(version).into()),
                },
            )
        }
    };
}

#[macro_export]
macro_rules! default_reply {
    () => {
        $crate::default_reply!(
            #[::cosmwasm_std::entry_point]
        );
    };
    (library) => {
        $crate::default_reply!(
            #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        );
    };
    (#[$meta:meta]) => {
        #[$meta]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            _: ::cosmwasm_std::Env,
            reply: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, $crate::access_managed::error::ContractError> {
            if let Some(reply) =
                $crate::access_managed::handle_consume_scheduled_op_reply(deps, reply)?
            {
                Err(
                    ::cosmwasm_std::StdError::generic_err(format!("unknown reply: {reply:?}"))
                        .into(),
                )
            } else {
                Ok(::cosmwasm_std::Response::new())
            }
        }
    };
}

#[derive(macros::Debug, thiserror::Error)]
#[debug(bound())]
pub enum IbcClientError<T: IbcClient> {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error(transparent)]
    Migrate(#[from] UpgradeError),
    #[error("decode error ({0:?})")]
    Decode(#[from] DecodeError<T>),
    #[error("unexpected call from the host module ({0})")]
    UnexpectedCallDataFromHostModule(String),
    #[error("client state not found")]
    ClientStateNotFound,
    #[error("`ClientMessage` cannot be decoded ({0})")]
    InvalidClientMessage(Bytes),
    #[error(transparent)]
    ClientSpecific(T::Error),
    #[error(transparent)]
    AccessManaged(#[from] access_managed::error::ContractError),
    #[error(transparent)]
    Pausable(#[from] pausable::error::ContractError),
    #[error(transparent)]
    Upgradable(#[from] upgradable::error::ContractError),
}

#[derive(macros::Debug, thiserror::Error)]
#[debug(bound())]
pub enum DecodeError<T: IbcClient> {
    #[error("unable to decode header")]
    Header(#[source] DecodeErrorOf<T::Encoding, T::Header>),
    #[error("unable to decode misbehaviour")]
    Misbehaviour(#[source] DecodeErrorOf<T::Encoding, T::Misbehaviour>),
    #[error("unable to decode client state")]
    ClientState(#[source] DecodeErrorOf<T::Encoding, T::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusState {
        counterparty_height: u64,
        #[source]
        error: DecodeErrorOf<EthAbi, T::ConsensusState>,
    },
    #[error("unable to decode storage proof")]
    StorageProof(#[source] DecodeErrorOf<T::Encoding, T::StorageProof>),
    #[error("unable to decode raw storage ({0})")]
    RawStorage(Bytes),
}

impl<T: IbcClient> From<IbcClientError<T>> for StdError {
    fn from(value: IbcClientError<T>) -> Self {
        Self::generic_err(ErrorReporter(value).to_string())
    }
}

pub struct IbcClientCtx<'a, T: IbcClient> {
    pub client_id: ClientId,
    pub ibc_host: Addr,
    pub deps: Deps<'a, T::CustomQuery>,
    pub env: Env,
}

impl<'a, T: IbcClient> IbcClientCtx<'a, T> {
    pub fn new(
        client_id: ClientId,
        ibc_host: Addr,
        deps: Deps<'a, T::CustomQuery>,
        env: Env,
    ) -> Self {
        Self {
            client_id,
            ibc_host,
            deps,
            env,
        }
    }

    pub fn read_self_client_state(&self) -> Result<T::ClientState, IbcClientError<T>> {
        read_client_state(&*self.deps.querier, &self.ibc_host, self.client_id)
    }

    pub fn read_self_consensus_state(
        &self,
        height: u64,
    ) -> Result<T::ConsensusState, IbcClientError<T>> {
        read_consensus_state(&*self.deps.querier, &self.ibc_host, self.client_id, height)
    }

    pub fn read_self_storage<S: Store>(&self, key: S::Key) -> Result<S::Value, IbcClientError<T>> {
        self.deps
            .querier
            .read::<ClientStore<S>>(&self.ibc_host, &(self.client_id, key))
            .map_err(Into::into)
    }

    pub fn read_client_state<Client: IbcClient>(
        &self,
        client_id: ClientId,
    ) -> Result<Client::ClientState, IbcClientError<Client>> {
        read_client_state(&*self.deps.querier, &self.ibc_host, client_id)
    }

    pub fn read_consensus_state<Client: IbcClient>(
        &self,
        client_id: ClientId,
        height: u64,
    ) -> Result<Client::ConsensusState, IbcClientError<Client>> {
        read_consensus_state(&*self.deps.querier, &self.ibc_host, client_id, height)
    }

    pub fn verify_membership<Client: IbcClient>(
        &self,
        client_id: ClientId,
        height: u64,
        path: Bytes,
        storage_proof: Client::StorageProof,
        value: Bytes,
    ) -> Result<(), IbcClientError<Client>> {
        let client_impl = client_impl(&*self.deps.querier, &self.ibc_host, client_id)?;

        self.deps.querier.query_wasm_smart::<()>(
            &client_impl,
            &VerificationQueryMsg::VerifyMembership(VerifyMembershipQuery {
                client_id,
                height,
                proof: storage_proof.encode_as::<Client::Encoding>().into(),
                path,
                value,
            }),
        )?;

        Ok(())
    }

    pub fn status<Client: IbcClient>(
        &self,
        client_id: ClientId,
    ) -> Result<Status, IbcClientError<Client>> {
        let client_impl = client_impl(&*self.deps.querier, &self.ibc_host, client_id)?;

        let status = self
            .deps
            .querier
            .query_wasm_smart::<Status>(&client_impl, &(QueryMsg::GetStatus { client_id }))?;

        Ok(status)
    }
}

fn client_impl<T: IbcClient>(
    querier: &dyn Querier,
    ibc_host: &Addr,
    client_id: ClientId,
) -> Result<Addr, IbcClientError<T>> {
    let addr = querier
        .read::<ClientImpls>(ibc_host, &client_id)
        .map_err(|err| {
            IbcClientError::Std(StdError::generic_err(format!(
                "unable to read client state of client {client_id}: {err}"
            )))
        })?;

    Ok(addr)
}

#[derive(Debug)]
pub struct StateUpdate<T: IbcClient> {
    pub height: u64,
    pub client_state: Option<T::ClientState>,
    pub consensus_state: T::ConsensusState,
    pub storage_writes: StorageWrites,
}

impl<T: IbcClient> StateUpdate<T> {
    pub fn new(height: u64, consensus_state: T::ConsensusState) -> Self {
        StateUpdate {
            height,
            consensus_state,
            client_state: None,
            storage_writes: Default::default(),
        }
    }

    pub fn overwrite_client_state(mut self, client_state: T::ClientState) -> Self {
        self.client_state = Some(client_state);
        self
    }

    pub fn add_storage_write<S: Store>(mut self, key: S::Key, value: S::Value) -> Self {
        self.storage_writes
            .insert(depolama::raw_key::<S>(&key), S::encode_value(&value));
        self
    }
}

/// Client creation output type
pub struct ClientCreationResult<T: IbcClient> {
    /// The client state that is going to be stored by IBC. If set to `None`, IBC will store the
    /// client state given by the creator as is
    pub client_state: Option<T::ClientState>,
    /// Custom events that will be emitted by IBC.
    pub events: Vec<VerifyCreationResponseEvent>,
    /// Arbitrary storage saves to the client's corresponding storage. These are accessible to the
    /// client at any time.
    pub storage_writes: StorageWrites,
}

impl<T: IbcClient> ClientCreationResult<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event(mut self, event: VerifyCreationResponseEvent) -> Self {
        self.events.push(event);
        self
    }

    pub fn add_storage_write<S: Store>(mut self, key: S::Key, value: S::Value) -> Self {
        self.storage_writes
            .insert(depolama::raw_key::<S>(&key), S::encode_value(&value));
        self
    }

    pub fn overwrite_client_state(mut self, client_state: T::ClientState) -> Self {
        self.client_state = Some(client_state);
        self
    }
}

impl<T: IbcClient> Default for ClientCreationResult<T> {
    fn default() -> Self {
        Self {
            client_state: None,
            events: Vec::new(),
            storage_writes: Default::default(),
        }
    }
}

pub trait IbcClient: Sized + 'static {
    type Error: core::error::Error + Into<IbcClientError<Self>>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: Decode<Self::Encoding, Error: Error + 'static> + Debug + 'static;
    type Misbehaviour: Decode<Self::Encoding, Error: Error + 'static> + Debug + 'static;
    type ClientState: Decode<Self::Encoding, Error: Error + 'static>
        + Encode<Self::Encoding>
        + Debug
        + 'static;
    /// Note that this type only requires `Encode/Decode<Ethabi>`, cause `ConsensusState` must have
    /// a common encoding scheme for state lenses. When doing state lenses, client X will read the
    /// consensus state of client Y by assuming it's state is ethabi-encoded.
    type ConsensusState: Decode<EthAbi, Error: Error + 'static> + Encode<EthAbi> + Debug + 'static;
    type StorageProof: Encode<Self::Encoding>
        + Decode<Self::Encoding, Error: Error + 'static>
        + Debug
        + 'static;
    type Encoding: Encoding;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>>;

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>>;

    /// Get the timestamp
    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp;

    /// Get the height
    fn get_latest_height(client_state: &Self::ClientState) -> u64;

    /// Get the tracked (counterparty) chain id.
    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String;

    /// Get the status of the client
    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status;

    /// Verify the initial state of the client
    fn verify_creation(
        caller: Addr,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>>;

    /// Verify `header` against the trusted state (`client_state` and `consensus_state`)
    /// and return `(updated height, updated client state, updated consensus state)`
    fn verify_header(
        ctx: IbcClientCtx<Self>,
        caller: Addr,
        header: Self::Header,
        relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>>;

    fn misbehaviour(
        ctx: IbcClientCtx<Self>,
        caller: Addr,
        misbehaviour: Self::Misbehaviour,
        relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>>;
}

pub fn init<T: IbcClient>(
    mut deps: DepsMut<T::CustomQuery>,
    msg: InitMsg,
) -> Result<Response, IbcClientError<T>> {
    access_managed::init(deps.branch().into_empty(), msg.access_managed_init_msg)?;

    // cosmwasm doesn't understand newtypes. the addr type in the message is not validated, so make sure we check it ourselves
    let ibc_host = deps.api.addr_validate(msg.ibc_host.as_ref())?;

    deps.storage.write_item::<IbcHost>(&ibc_host);

    Ok(Response::default())
}

pub fn execute<T: IbcClient>(
    mut deps: DepsMut<T::CustomQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<T::CustomQuery>, IbcClientError<T>> {
    match msg {
        ExecuteMsg::AccessManaged(msg) => {
            Ok(access_managed::execute(deps.into_empty(), env, info, msg)?
                .change_custom()
                .expect("custom is not used here; qed;"))
        }
        ExecuteMsg::Upgradable(msg) => {
            let msg =
                match msg.ensure_can_call::<Authority>(deps.branch().into_empty(), &env, &info)? {
                    EnsureCanCallResult::Msg(msg) => msg,
                    EnsureCanCallResult::Scheduled(sub_msgs) => {
                        return Ok(Response::new()
                            .add_submessages(sub_msgs)
                            .change_custom()
                            .expect("custom is not used here; qed;"));
                    }
                };

            Ok(upgradable::execute(&env, msg)?
                .change_custom()
                .expect("custom is not used here; qed;"))
        }
        ExecuteMsg::Pausable(msg) => {
            let msg =
                match msg.ensure_can_call::<Authority>(deps.branch().into_empty(), &env, &info)? {
                    EnsureCanCallResult::Msg(msg) => msg,
                    EnsureCanCallResult::Scheduled(sub_msgs) => {
                        return Ok(Response::new()
                            .add_submessages(sub_msgs)
                            .change_custom()
                            .expect("custom is not used here; qed;"));
                    }
                };

            Ok(pausable::execute(deps.into_empty(), &info, &msg)?
                .change_custom()
                .expect("custom is not used here; qed;"))
        }
    }
}

pub fn query<T: IbcClient>(
    deps: Deps<T::CustomQuery>,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, IbcClientError<T>> {
    match msg {
        QueryMsg::GetTimestamp { client_id, height } => {
            let ibc_host = deps.storage.read_item::<IbcHost>()?;

            let consensus_state =
                read_consensus_state::<T>(&*deps.querier, &ibc_host, client_id, height)?;

            to_json_binary(&T::get_timestamp(&consensus_state)).map_err(Into::into)
        }
        QueryMsg::GetLatestHeight { client_id } => {
            let ibc_host = deps.storage.read_item::<IbcHost>()?;

            let client_state = read_client_state::<T>(&*deps.querier, &ibc_host, client_id)?;

            to_json_binary(&T::get_latest_height(&client_state)).map_err(Into::into)
        }
        QueryMsg::GetStatus { client_id } => {
            let ibc_host = deps.storage.read_item::<IbcHost>()?;

            let client_state = read_client_state::<T>(&*deps.querier, &ibc_host, client_id)?;

            let status = T::status(
                IbcClientCtx::new(client_id, ibc_host, deps, env),
                &client_state,
            );

            to_json_binary(&status).map_err(Into::into)
        }
        QueryMsg::Verification(msg) => match msg.ensure_not_paused(deps.into_empty())? {
            VerificationQueryMsg::VerifyCreation(VerifyCreationQuery {
                caller,
                client_id,
                relayer,
            }) => {
                let ibc_host = deps.storage.read_item::<IbcHost>()?;

                let client_state = read_client_state::<T>(&*deps.querier, &ibc_host, client_id)?;
                let consensus_state = read_consensus_state::<T>(
                    &*deps.querier,
                    &ibc_host,
                    client_id,
                    T::get_latest_height(&client_state),
                )?;

                let client_creation = T::verify_creation(
                    Addr::unchecked(caller),
                    &client_state,
                    &consensus_state,
                    Addr::unchecked(relayer),
                )?;

                let response = VerifyCreationResponse {
                    counterparty_chain_id: T::get_counterparty_chain_id(&client_state),
                    client_state_bytes: client_creation
                        .client_state
                        .map(|cs| cs.encode_as::<T::Encoding>().into()),
                    storage_writes: client_creation.storage_writes,
                    events: client_creation.events,
                };

                to_json_binary(&response).map_err(Into::into)
            }
            VerificationQueryMsg::VerifyMembership(VerifyMembershipQuery {
                client_id,
                height,
                proof,
                path,
                value,
            }) => {
                let ibc_host = deps.storage.read_item::<IbcHost>()?;

                let storage_proof = T::StorageProof::decode_as::<T::Encoding>(&proof)
                    .map_err(DecodeError::StorageProof)?;

                T::verify_membership(
                    IbcClientCtx::new(client_id, ibc_host, deps, env),
                    height,
                    path.to_vec(),
                    storage_proof,
                    value.to_vec(),
                )?;

                to_json_binary(&()).map_err(Into::into)
            }
            VerificationQueryMsg::VerifyNonMembership(VerifyNonMembershipQuery {
                client_id,
                height,
                proof,
                path,
            }) => {
                let ibc_host = deps.storage.read_item::<IbcHost>()?;

                let storage_proof = T::StorageProof::decode_as::<T::Encoding>(&proof)
                    .map_err(DecodeError::StorageProof)?;

                T::verify_non_membership(
                    IbcClientCtx::new(client_id, ibc_host, deps, env),
                    height,
                    path.to_vec(),
                    storage_proof,
                )?;

                to_json_binary(&()).map_err(Into::into)
            }
            VerificationQueryMsg::UpdateState(UpdateStateQuery {
                caller,
                client_id,
                relayer,
            }) => {
                let ibc_host = deps.storage.read_item::<IbcHost>()?;

                let message = deps.querier.read_item::<QueryStore>(&ibc_host)?;

                let header =
                    T::Header::decode_as::<T::Encoding>(&message).map_err(DecodeError::Header)?;

                let StateUpdate {
                    height,
                    client_state,
                    consensus_state,
                    storage_writes,
                } = T::verify_header(
                    IbcClientCtx::new(client_id, ibc_host, deps, env),
                    Addr::unchecked(caller),
                    header,
                    Addr::unchecked(relayer),
                )?;

                to_json_binary(&UpdateStateResponse {
                    height,
                    consensus_state_bytes: consensus_state.encode().into(),
                    client_state_bytes: client_state.map(|cs| cs.encode_as::<T::Encoding>().into()),
                    storage_writes,
                })
                .map_err(Into::into)
            }
            VerificationQueryMsg::Misbehaviour(MisbehaviourQuery {
                caller,
                client_id,
                message,
                relayer,
            }) => {
                let ibc_host = deps.storage.read_item::<IbcHost>()?;

                let misbehaviour = T::Misbehaviour::decode_as::<T::Encoding>(&message)
                    .map_err(DecodeError::Misbehaviour)?;

                let client_state = T::misbehaviour(
                    IbcClientCtx::new(client_id, ibc_host, deps, env),
                    Addr::unchecked(caller),
                    misbehaviour,
                    Addr::unchecked(relayer),
                )?;

                to_json_binary(&MisbehaviourResponse {
                    client_state: client_state.encode_as::<T::Encoding>().into(),
                })
                .map_err(Into::into)
            }
        },
        QueryMsg::AccessManaged(msg) => {
            access_managed::query(deps.into_empty(), env, msg).map_err(Into::into)
        }
        QueryMsg::Pausable(msg) => pausable::query(deps.into_empty(), &msg).map_err(Into::into),
    }
}

pub fn read_client_state<T: IbcClient>(
    querier: &dyn Querier,
    ibc_host: &Addr,
    client_id: ClientId,
) -> Result<T::ClientState, IbcClientError<T>> {
    let client_state = querier
        .read::<ClientStates>(ibc_host, &client_id)
        .map_err(|err| {
            IbcClientError::Std(StdError::generic_err(format!(
                "unable to read client state of client {client_id}: {err}"
            )))
        })?;

    T::ClientState::decode_as::<T::Encoding>(&client_state)
        .map_err(|e| IbcClientError::Decode(DecodeError::ClientState(e)))
}

pub fn read_consensus_state<T: IbcClient>(
    querier: &dyn Querier,
    ibc_host: &Addr,
    client_id: ClientId,
    height: u64,
) -> Result<T::ConsensusState, IbcClientError<T>> {
    let consensus_state = querier
        .read::<ClientConsensusStates>(ibc_host, &(client_id, height))
        .map_err(|err| {
            IbcClientError::Std(StdError::generic_err(format!(
                "unable to read consensus state of client {client_id} at trusted height {height}: {err}"
            )))
        })?;

    T::ConsensusState::decode(&consensus_state).map_err(|e| {
        IbcClientError::Decode(DecodeError::ConsensusState {
            counterparty_height: height,
            error: e,
        })
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        ContractResult, Empty, SystemResult, WasmQuery, from_json,
        testing::{mock_dependencies, mock_env},
    };
    use unionlabs::encoding;

    use super::*;

    #[test]
    fn verify_membership_call() {
        enum Lc {}

        #[derive(Debug, thiserror::Error)]
        enum Error {}

        impl From<Error> for IbcClientError<Lc> {
            fn from(value: Error) -> Self {
                match value {}
            }
        }

        #[derive(Debug)]
        struct ConsensusState {}

        impl Decode<EthAbi> for ConsensusState {
            type Error = Error;

            fn decode(_: &[u8]) -> Result<Self, Self::Error> {
                unreachable!()
            }
        }

        impl Encode<EthAbi> for ConsensusState {
            fn encode(self) -> Vec<u8> {
                unreachable!()
            }
        }

        impl IbcClient for Lc {
            type Error = Error;

            type CustomQuery = Empty;

            type Header = ();

            type Misbehaviour = ();

            type ClientState = ();

            type ConsensusState = ConsensusState;

            type StorageProof = ();

            type Encoding = encoding::Json;

            fn verify_membership(
                _ctx: IbcClientCtx<Self>,
                _height: u64,
                _key: Vec<u8>,
                _storage_proof: Self::StorageProof,
                _value: Vec<u8>,
            ) -> Result<(), IbcClientError<Self>> {
                unreachable!()
            }

            fn verify_non_membership(
                _ctx: IbcClientCtx<Self>,
                _height: u64,
                _key: Vec<u8>,
                _storage_proof: Self::StorageProof,
            ) -> Result<(), IbcClientError<Self>> {
                unreachable!()
            }

            fn get_timestamp(_consensus_state: &Self::ConsensusState) -> Timestamp {
                unreachable!()
            }

            fn get_latest_height(_client_state: &Self::ClientState) -> u64 {
                unreachable!()
            }

            fn get_counterparty_chain_id(_client_state: &Self::ClientState) -> String {
                unreachable!()
            }

            fn status(_ctx: IbcClientCtx<Self>, _client_state: &Self::ClientState) -> Status {
                unreachable!()
            }

            fn verify_creation(
                _caller: Addr,
                _client_state: &Self::ClientState,
                _consensus_state: &Self::ConsensusState,
                _relayer: Addr,
            ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
                unreachable!()
            }

            fn verify_header(
                _ctx: IbcClientCtx<Self>,
                _caller: Addr,
                _header: Self::Header,
                _relayer: Addr,
            ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
                unreachable!()
            }

            fn misbehaviour(
                _ctx: IbcClientCtx<Self>,
                _caller: Addr,
                _misbehaviour: Self::Misbehaviour,
                _relayer: Addr,
            ) -> Result<Self::ClientState, IbcClientError<Self>> {
                unreachable!()
            }
        }

        let mut deps = mock_dependencies();
        let env = mock_env();

        deps.querier.update_wasm(|wq| match wq {
            WasmQuery::Smart { contract_addr, msg } => match contract_addr.as_str() {
                "client-2-impl" => SystemResult::Ok(ContractResult::Ok(
                    match from_json::<QueryMsg>(msg).unwrap() {
                        QueryMsg::Verification(when_not_paused) => {
                            match when_not_paused
                                .ensure_not_paused(mock_dependencies().as_ref())
                                .unwrap()
                            {
                                VerificationQueryMsg::VerifyMembership(_) => {
                                    Binary::new(b"null".to_vec())
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    },
                )),
                _ => {
                    unreachable!()
                }
            },
            WasmQuery::Raw { contract_addr, .. } => match contract_addr.as_str() {
                "ibc-host" => {
                    SystemResult::Ok(ContractResult::Ok(Binary::new(b"client-2-impl".to_vec())))
                }
                _ => {
                    unreachable!()
                }
            },
            _ => unreachable!(),
        });

        let ctx = IbcClientCtx::<Lc>::new(
            ClientId!(1),
            Addr::unchecked("ibc-host"),
            deps.as_ref(),
            env,
        );

        ctx.verify_membership::<Lc>(ClientId!(2), 1, Bytes::default(), (), Bytes::default())
            .unwrap();
    }
}
