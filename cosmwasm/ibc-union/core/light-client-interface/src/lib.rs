#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use core::fmt::Debug;

use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, QuerierWrapper, Response, StdError,
};
use depolama::{QuerierExt, StorageExt, Store};
use ibc_union::state::{ClientConsensusStates, ClientImpls, ClientStates, ClientStore, QueryStore};
use ibc_union_msg::lightclient::{
    MisbehaviourResponse, QueryMsg, Status, StorageWrites, UpdateStateResponse,
    VerifyCreationResponse, VerifyCreationResponseEvent,
};
use ibc_union_spec::ClientId;
use unionlabs::{
    encoding::{Decode, DecodeAs, DecodeErrorOf, Encode, EncodeAs, Encoding, EthAbi},
    primitives::Bytes,
    ErrorReporter,
};
use unionlabs_cosmwasm_upgradable::UpgradeError;

use crate::{msg::InitMsg, state::IbcHost};

pub mod msg;
pub mod state;

// TODO: Add #[source] to all variants
#[derive(macros::Debug, thiserror::Error)]
#[debug(bound())]
pub enum DecodeError<T: IbcClient> {
    #[error("unable to decode header")]
    Header(DecodeErrorOf<T::Encoding, T::Header>),
    #[error("unable to decode misbehaviour")]
    Misbehaviour(DecodeErrorOf<T::Encoding, T::Misbehaviour>),
    #[error("unable to decode client state")]
    ClientState(DecodeErrorOf<T::Encoding, T::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusState(DecodeErrorOf<EthAbi, T::ConsensusState>),
    #[error("unable to decode storage proof")]
    StorageProof(DecodeErrorOf<T::Encoding, T::StorageProof>),
    #[error("unable to decode raw storage ({0})")]
    RawStorage(Bytes),
}

#[derive(macros::Debug, thiserror::Error)]
#[debug(bound())]
pub enum IbcClientError<T: IbcClient> {
    #[error("std error ({0:?})")]
    Std(#[from] StdError),
    #[error("migration error")]
    Migrate(#[from] UpgradeError),
    #[error("decode error ({0:?})")]
    Decode(#[from] DecodeError<T>),
    #[error("unexpected call from the host module ({0})")]
    UnexpectedCallDataFromHostModule(String),
    #[error("client state not found")]
    ClientStateNotFound,
    #[error(transparent)]
    ClientSpecific(T::Error),
    #[error("`ClientMessage` cannot be decoded ({data})", data = serde_utils::to_hex(.0))]
    InvalidClientMessage(Vec<u8>),
    #[error("caller `{0}` is not a whitelisted relayer")]
    UnauthorizedCaller(String),
}

impl<T: IbcClient + 'static> From<IbcClientError<T>> for StdError {
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
        read_client_state(
            self.deps.querier.into_empty(),
            &self.ibc_host,
            self.client_id,
        )
    }

    pub fn read_self_consensus_state(
        &self,
        height: u64,
    ) -> Result<T::ConsensusState, IbcClientError<T>> {
        read_consensus_state(
            self.deps.querier.into_empty(),
            &self.ibc_host,
            self.client_id,
            height,
        )
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
        read_client_state(self.deps.querier.into_empty(), &self.ibc_host, client_id)
    }

    pub fn read_consensus_state<Client: IbcClient>(
        &self,
        client_id: ClientId,
        height: u64,
    ) -> Result<Client::ConsensusState, IbcClientError<Client>> {
        read_consensus_state(
            self.deps.querier.into_empty(),
            &self.ibc_host,
            client_id,
            height,
        )
    }

    pub fn verify_membership<Client: IbcClient>(
        &self,
        client_id: ClientId,
        height: u64,
        path: Bytes,
        storage_proof: Client::StorageProof,
        value: Bytes,
    ) -> Result<(), IbcClientError<Client>> {
        let client_impl = client_impl(self.deps.querier.into_empty(), &self.ibc_host, client_id)?;
        self.deps.querier.query_wasm_smart::<()>(
            &client_impl,
            &(QueryMsg::VerifyMembership {
                client_id,
                height,
                proof: storage_proof.encode_as::<Client::Encoding>().into(),
                path,
                value,
            }),
        )?;

        Ok(())
    }
}

fn client_impl<T: IbcClient>(
    querier: QuerierWrapper,
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

pub trait IbcClient: Sized {
    type Error: core::error::Error + Into<IbcClientError<Self>>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: Decode<Self::Encoding, Error: Debug> + Debug + 'static;
    type Misbehaviour: Decode<Self::Encoding, Error: Debug> + Debug + 'static;
    type ClientState: Decode<Self::Encoding, Error: Debug>
        + Encode<Self::Encoding>
        + Debug
        + 'static;
    /// Note that this type only requires `Encode/Decode<Ethabi>`, cause `ConsensusState` must have
    /// a common encoding scheme for state lenses. When doing state lenses, client X will read the
    /// consensus state of client Y by assuming it's state is ethabi-encoded.
    type ConsensusState: Decode<EthAbi, Error: Debug> + Encode<EthAbi> + Debug + 'static;
    type StorageProof: Encode<Self::Encoding>
        + Decode<Self::Encoding, Error: Debug>
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
    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64;

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
    deps: DepsMut<T::CustomQuery>,
    msg: InitMsg,
) -> Result<Response, IbcClientError<T>> {
    // cosmwasm doesn't understand newtypes. the addr type in the message is not validated, so make sure we check it ourselves
    let ibc_host = deps.api.addr_validate(msg.ibc_host.as_ref())?;

    deps.storage.write_item::<IbcHost>(&ibc_host);

    Ok(Response::default())
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
                read_consensus_state::<T>(deps.querier.into_empty(), &ibc_host, client_id, height)?;
            to_json_binary(&T::get_timestamp(&consensus_state)).map_err(Into::into)
        }
        QueryMsg::GetLatestHeight { client_id } => {
            let ibc_host = deps.storage.read_item::<IbcHost>()?;
            let client_state =
                read_client_state::<T>(deps.querier.into_empty(), &ibc_host, client_id)?;
            to_json_binary(&T::get_latest_height(&client_state)).map_err(Into::into)
        }
        QueryMsg::GetStatus { client_id } => {
            let ibc_host = deps.storage.read_item::<IbcHost>()?;
            let client_state =
                read_client_state::<T>(deps.querier.into_empty(), &ibc_host, client_id)?;
            let status = T::status(
                IbcClientCtx::new(client_id, ibc_host, deps, env),
                &client_state,
            );
            to_json_binary(&status).map_err(Into::into)
        }
        QueryMsg::VerifyCreation {
            // NOTE(aeryz): we don't need `client_id` since we already got the client and
            // consensus states
            caller,
            client_id: _,
            client_state,
            consensus_state,
            relayer,
        } => {
            let client_state = T::ClientState::decode_as::<T::Encoding>(&client_state)
                .map_err(|e| IbcClientError::Decode(DecodeError::ClientState(e)))?;

            let consensus_state = T::ConsensusState::decode(&consensus_state)
                .map_err(|e| IbcClientError::Decode(DecodeError::ConsensusState(e)))?;

            let client_creation = T::verify_creation(
                Addr::unchecked(caller),
                &client_state,
                &consensus_state,
                Addr::unchecked(relayer),
            )?;

            let response = VerifyCreationResponse {
                latest_height: T::get_latest_height(&client_state),
                counterparty_chain_id: T::get_counterparty_chain_id(&client_state),
                client_state_bytes: client_creation
                    .client_state
                    .map(|cs| cs.encode_as::<T::Encoding>().into()),
                storage_writes: client_creation.storage_writes,
                events: client_creation.events,
            };

            to_json_binary(&response).map_err(Into::into)
        }
        QueryMsg::VerifyMembership {
            client_id,
            height,
            proof,
            path,
            value,
        } => {
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
        QueryMsg::VerifyNonMembership {
            client_id,
            height,
            proof,
            path,
        } => {
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
        QueryMsg::UpdateState {
            caller,
            client_id,
            relayer,
        } => {
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

            to_json_binary(
                &(UpdateStateResponse {
                    height,
                    consensus_state_bytes: consensus_state.encode().into(),
                    client_state_bytes: client_state.map(|cs| cs.encode_as::<T::Encoding>().into()),
                    storage_writes,
                }),
            )
            .map_err(Into::into)
        }
        QueryMsg::Misbehaviour {
            caller,
            client_id,
            message,
            relayer,
        } => {
            let misbehaviour = T::Misbehaviour::decode_as::<T::Encoding>(&message)
                .map_err(DecodeError::Misbehaviour)?;

            let ibc_host = deps.storage.read_item::<IbcHost>()?;
            let client_state = T::misbehaviour(
                IbcClientCtx::new(client_id, ibc_host, deps, env),
                Addr::unchecked(caller),
                misbehaviour,
                Addr::unchecked(relayer),
            )?;

            to_json_binary(
                &(MisbehaviourResponse {
                    client_state: client_state.encode_as::<T::Encoding>().into(),
                }),
            )
            .map_err(Into::into)
        }
    }
}

pub fn read_client_state<T: IbcClient>(
    querier: QuerierWrapper,
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
    querier: QuerierWrapper,
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

    T::ConsensusState::decode(&consensus_state)
        .map_err(|e| IbcClientError::Decode(DecodeError::ConsensusState(e)))
}
