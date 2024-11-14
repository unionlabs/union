use core::fmt::Debug;

use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdError};
use frame_support_procedural::{CloneNoBound, PartialEqNoBound};
use state::IBC_HOST;
use union_ibc::state::{CLIENT_CONSENSUS_STATES, CLIENT_STATES};
use union_ibc_msg::lightclient::{QueryMsg, Status, VerifyClientMessageUpdate};
use unionlabs::encoding::{Decode, DecodeAs, DecodeErrorOf, Encode, EncodeAs, Encoding};

pub mod state;

// TODO: Add #[source] to all variants
#[derive(macros::Debug, CloneNoBound, PartialEqNoBound, thiserror::Error)]
#[debug(bound())]
pub enum DecodeError<T: IbcClient> {
    #[error("unable to decode header")]
    Header(DecodeErrorOf<T::Encoding, T::Header>),
    #[error("unable to decode misbehaviour")]
    Misbehaviour(DecodeErrorOf<T::Encoding, T::Misbehaviour>),
    #[error("unable to decode client state")]
    ClientState(DecodeErrorOf<T::Encoding, T::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusState(DecodeErrorOf<T::Encoding, T::ConsensusState>),
    #[error("unable to decode storage proof")]
    StorageProof(DecodeErrorOf<T::Encoding, T::StorageProof>),
}

#[derive(macros::Debug, PartialEqNoBound, thiserror::Error)]
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
}

impl<T: IbcClient> From<IbcClientError<T>> for StdError {
    fn from(value: IbcClientError<T>) -> Self {
        Self::generic_err(value.to_string())
    }
}

pub trait IbcClient: Sized {
    type Error: core::error::Error + PartialEq + Into<IbcClientError<Self>>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: Decode<Self::Encoding, Error: Debug + PartialEq + Clone> + Debug + 'static;
    type Misbehaviour: Decode<Self::Encoding, Error: Debug + PartialEq + Clone> + Debug + 'static;
    type ClientState: Decode<Self::Encoding, Error: PartialEq + Clone>
        + Encode<Self::Encoding>
        + Debug
        + 'static;
    type ConsensusState: Decode<Self::Encoding, Error: PartialEq + Clone>
        + Encode<Self::Encoding>
        + Debug
        + 'static;
    type StorageProof: Decode<Self::Encoding, Error: Debug + PartialEq + Clone> + Debug + 'static;
    type Encoding: Encoding;

    fn verify_membership(
        client_id: u32,
        consensus_state: Self::ConsensusState,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>>;

    fn verify_non_membership(
        client_id: u32,
        consensus_state: Self::ConsensusState,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>>;

    /// Get the timestamp
    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64;

    /// Get the height
    fn get_latest_height(client_state: &Self::ClientState) -> u64;

    /// Get the status of the client
    fn status(client_state: &Self::ClientState) -> Status;

    /// Verify the initial state of the client
    fn verify_creation(
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
    ) -> Result<(), IbcClientError<Self>>;

    /// Verify `header` against the trusted state (`client_state` and `consensus_state`)
    /// and return `(updated height, updated client state, updated consensus state)`
    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        client_id: u32,
        client_state: Self::ClientState,
        consensus_state: Self::ConsensusState,
        header: Self::Header,
    ) -> Result<(u64, Self::ClientState, Self::ConsensusState), IbcClientError<Self>>;

    fn misbehaviour(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        client_id: u32,
        ibc_host: Addr,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>>;
}

pub fn query<T: IbcClient>(
    deps: Deps<T::CustomQuery>,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, IbcClientError<T>> {
    match msg {
        QueryMsg::GetTimestamp { client_id, height } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let consensus_state = read_consensus_state::<T>(deps, &ibc_host, client_id, height)?;
            to_json_binary(&T::get_timestamp(&consensus_state)).map_err(Into::into)
        }
        QueryMsg::GetLatestHeight { client_id } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state = read_client_state::<T>(deps, &ibc_host, client_id)?;
            to_json_binary(&T::get_latest_height(&client_state)).map_err(Into::into)
        }
        QueryMsg::GetStatus { client_id } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state = read_client_state::<T>(deps, &ibc_host, client_id)?;
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
            let client_state = T::ClientState::decode_as::<T::Encoding>(&client_state).unwrap();
            // if we are able to parse it then it's fine
            let consensus_state =
                T::ConsensusState::decode_as::<T::Encoding>(&consensus_state).unwrap();
            T::verify_creation(&client_state, &consensus_state)?;
            to_json_binary(&T::get_latest_height(&client_state)).map_err(Into::into)
        }
        QueryMsg::VerifyMembership {
            client_id,
            height,
            proof,
            path,
            value,
        } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let consensus_state = read_consensus_state::<T>(deps, &ibc_host, client_id, height)?;
            let storage_proof = T::StorageProof::decode_as::<T::Encoding>(&proof)
                .map_err(DecodeError::StorageProof)?;

            T::verify_membership(
                client_id,
                consensus_state,
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
            let consensus_state = read_consensus_state::<T>(deps, &ibc_host, client_id, height)?;
            let storage_proof = T::StorageProof::decode_as::<T::Encoding>(&proof)
                .map_err(DecodeError::StorageProof)?;

            T::verify_non_membership(client_id, consensus_state, path.to_vec(), storage_proof)?;

            to_json_binary(&()).map_err(Into::into)
        }
        QueryMsg::VerifyClientMessage { client_id, message } => {
            let header =
                T::Header::decode_as::<T::Encoding>(&message).map_err(DecodeError::Header)?;
            let ibc_host = IBC_HOST.load(deps.storage)?;

            let client_state = read_client_state::<T>(deps, &ibc_host, client_id)?;
            let consensus_state = read_consensus_state::<T>(
                deps,
                &ibc_host,
                client_id,
                T::get_latest_height(&client_state),
            )?;

            let (height, client_state, consensus_state) =
                T::verify_header(deps, env, client_id, client_state, consensus_state, header)?;

            to_json_binary(&VerifyClientMessageUpdate {
                height,
                consensus_state: consensus_state.encode_as::<T::Encoding>().into(),
                client_state: client_state.encode_as::<T::Encoding>().into(),
            })
            .map_err(Into::into)
        }
        QueryMsg::Misbehaviour { client_id, message } => {
            let misbehaviour = T::Misbehaviour::decode_as::<T::Encoding>(&message)
                .map_err(DecodeError::Misbehaviour)?;

            let ibc_host = IBC_HOST.load(deps.storage)?;
            T::misbehaviour(deps, env, client_id, ibc_host, misbehaviour)?;

            to_json_binary(&()).map_err(Into::into)
        }
    }
}

pub fn read_client_state<T: IbcClient>(
    deps: Deps<T::CustomQuery>,
    ibc_host: &Addr,
    client_id: u32,
) -> Result<T::ClientState, IbcClientError<T>> {
    let client_state = deps
        .querier
        .query_wasm_raw(ibc_host.to_string(), CLIENT_STATES.key(client_id).to_vec())?
        .unwrap();
    Ok(T::ClientState::decode_as::<T::Encoding>(&client_state).unwrap())
}

pub fn read_consensus_state<T: IbcClient>(
    deps: Deps<T::CustomQuery>,
    ibc_host: &Addr,
    client_id: u32,
    height: u64,
) -> Result<T::ConsensusState, IbcClientError<T>> {
    let consensus_state = deps
        .querier
        .query_wasm_raw(
            ibc_host.to_string(),
            CLIENT_CONSENSUS_STATES.key((client_id, height)).to_vec(),
        )?
        .unwrap();
    Ok(T::ConsensusState::decode_as::<T::Encoding>(&consensus_state).unwrap())
}
