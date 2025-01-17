#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use core::fmt::Debug;

use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    Response, StdError,
};
use cw_storage_plus::{Item, Map};
use ibc_union_msg::lightclient::{
    MisbehaviourResponse, QueryMsg, Status, VerifyClientMessageUpdate, VerifyCreationResponse,
    VerifyCreationResponseEvent,
};
use msg::InstantiateMsg;
use state::IBC_HOST;
use unionlabs::{
    encoding::{Decode, DecodeAs, DecodeErrorOf, Encode, EncodeAs, Encoding, EthAbi},
    primitives::{encoding::Base64, Bytes},
    ErrorReporter,
};

pub mod msg;
pub mod state;

// These are only used for `key` calculation. We don't want this crate to depend on `ibc-union`.
pub const CLIENT_STATES: Map<u32, Binary> = Map::new("client_states");
pub const CLIENT_CONSENSUS_STATES: Map<(u32, u64), Binary> = Map::new("client_consensus_states");
const CLIENT_IMPLS: Map<u32, Addr> = Map::new("client_impls");
const QUERY_STORE: Item<Binary> = Item::new("query_store");

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
}

#[derive(macros::Debug, thiserror::Error)]
#[debug(bound())]
pub enum IbcClientError<T: IbcClient> {
    #[error("decode error ({0:?})")]
    Decode(#[from] DecodeError<T>),
    #[error("std error ({0:?})")]
    Std(#[from] StdError),
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
    pub client_id: u32,
    pub ibc_host: Addr,
    pub deps: Deps<'a, T::CustomQuery>,
    pub env: Env,
}

impl<'a, T: IbcClient> IbcClientCtx<'a, T> {
    pub fn new(client_id: u32, ibc_host: Addr, deps: Deps<'a, T::CustomQuery>, env: Env) -> Self {
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

    pub fn read_client_state<Client: IbcClient>(
        &self,
        client_id: u32,
    ) -> Result<Client::ClientState, IbcClientError<Client>> {
        read_client_state(self.deps.querier.into_empty(), &self.ibc_host, client_id)
    }

    pub fn read_consensus_state<Client: IbcClient>(
        &self,
        client_id: u32,
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
        client_id: u32,
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
    client_id: u32,
) -> Result<Addr, IbcClientError<T>> {
    let addr = from_json::<Addr>(
        querier
            .query_wasm_raw(ibc_host.to_string(), CLIENT_IMPLS.key(client_id).to_vec())?
            .ok_or_else(|| {
                IbcClientError::Std(StdError::generic_err(format!(
                    "unable to read client state of client {client_id}"
                )))
            })?,
    )?;

    Ok(addr)
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
    fn status(client_state: &Self::ClientState) -> Status;

    /// Verify the initial state of the client
    fn verify_creation(
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
    ) -> Result<Option<Vec<VerifyCreationResponseEvent>>, IbcClientError<Self>>;

    /// Verify `header` against the trusted state (`client_state` and `consensus_state`)
    /// and return `(updated height, updated client state, updated consensus state)`
    fn verify_header(
        ctx: IbcClientCtx<Self>,
        header: Self::Header,
        caller: Addr,
    ) -> Result<(u64, Self::ClientState, Self::ConsensusState), IbcClientError<Self>>;

    fn misbehaviour(
        ctx: IbcClientCtx<Self>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, IbcClientError<Self>>;
}

pub fn instantiate<T: IbcClient>(
    deps: DepsMut<T::CustomQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, IbcClientError<T>> {
    IBC_HOST.save(deps.storage, &msg.ibc_host)?;
    Ok(Response::default())
}

pub fn query<T: IbcClient>(
    deps: Deps<T::CustomQuery>,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, IbcClientError<T>> {
    match msg {
        QueryMsg::GetTimestamp { client_id, height } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let consensus_state =
                read_consensus_state::<T>(deps.querier.into_empty(), &ibc_host, client_id, height)?;
            to_json_binary(&T::get_timestamp(&consensus_state)).map_err(Into::into)
        }
        QueryMsg::GetLatestHeight { client_id } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state =
                read_client_state::<T>(deps.querier.into_empty(), &ibc_host, client_id)?;
            to_json_binary(&T::get_latest_height(&client_state)).map_err(Into::into)
        }
        QueryMsg::GetStatus { client_id } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state =
                read_client_state::<T>(deps.querier.into_empty(), &ibc_host, client_id)?;
            let status = T::status(&client_state);
            to_json_binary(&status).map_err(Into::into)
        }
        QueryMsg::VerifyCreation {
            // NOTE(aeryz): we don't need `client_id` since we already got the client and
            // consensus states
            client_id: _,
            client_state,
            consensus_state,
        } => {
            let client_state = T::ClientState::decode_as::<T::Encoding>(&client_state)
                .map_err(|e| IbcClientError::Decode(DecodeError::ClientState(e)))?;

            let consensus_state = T::ConsensusState::decode(&consensus_state)
                .map_err(|e| IbcClientError::Decode(DecodeError::ConsensusState(e)))?;

            let events = T::verify_creation(&client_state, &consensus_state)?;

            let response = VerifyCreationResponse {
                latest_height: T::get_latest_height(&client_state),
                counterparty_chain_id: T::get_counterparty_chain_id(&client_state),
                events,
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
            let ibc_host = IBC_HOST.load(deps.storage)?;
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
            let ibc_host = IBC_HOST.load(deps.storage)?;
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
        QueryMsg::VerifyClientMessage { client_id, caller } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let message = QUERY_STORE.query(&deps.querier, ibc_host.clone())?;
            let header =
                T::Header::decode_as::<T::Encoding>(&message).map_err(DecodeError::Header)?;

            let (height, client_state, consensus_state) = T::verify_header(
                IbcClientCtx::new(client_id, ibc_host, deps, env),
                header,
                Addr::unchecked(caller),
            )?;

            to_json_binary(
                &(VerifyClientMessageUpdate {
                    height,
                    consensus_state: consensus_state.encode().into(),
                    client_state: client_state.encode_as::<T::Encoding>().into(),
                }),
            )
            .map_err(Into::into)
        }
        QueryMsg::Misbehaviour { client_id, message } => {
            let misbehaviour = T::Misbehaviour::decode_as::<T::Encoding>(&message)
                .map_err(DecodeError::Misbehaviour)?;

            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state = T::misbehaviour(
                IbcClientCtx::new(client_id, ibc_host, deps, env),
                misbehaviour,
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
    client_id: u32,
) -> Result<T::ClientState, IbcClientError<T>> {
    let client_state = from_json::<Bytes<Base64>>(
        querier
            .query_wasm_raw(ibc_host.to_string(), CLIENT_STATES.key(client_id).to_vec())?
            .ok_or_else(|| {
                IbcClientError::Std(StdError::generic_err(format!(
                    "unable to read client state of client {client_id}"
                )))
            })?,
    )?;

    T::ClientState::decode_as::<T::Encoding>(&client_state)
        .map_err(|e| IbcClientError::Decode(DecodeError::ClientState(e)))
}

pub fn read_consensus_state<T: IbcClient>(
    querier: QuerierWrapper,
    ibc_host: &Addr,
    client_id: u32,
    height: u64,
) -> Result<T::ConsensusState, IbcClientError<T>> {
    let consensus_state = from_json::<Bytes<Base64>>(
        querier
            .query_wasm_raw(
                ibc_host.to_string(),
                CLIENT_CONSENSUS_STATES.key((client_id, height)).to_vec()
            )?
            .ok_or_else(|| {
                IbcClientError::Std(
                    StdError::generic_err(
                        format!(
                            "unable to read consensus state of client {client_id} at trusted height {height}"
                        )
                    )
                )
            })?
    )?;

    T::ConsensusState::decode(&consensus_state)
        .map_err(|e| IbcClientError::Decode(DecodeError::ConsensusState(e)))
}
