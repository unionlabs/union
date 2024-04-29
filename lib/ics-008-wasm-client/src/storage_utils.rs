use core::fmt::Debug;

use cosmwasm_std::{Deps, DepsMut};
use prost::Message;
use protos::{
    google::protobuf::Any as ProtoAny,
    ibc::lightclients::wasm::v1::{
        ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
    },
};
use unionlabs::{
    encoding::{Decode, Encode, EncodeAs, Proto},
    google::protobuf::any::Any,
    ibc::{core::client::height::Height, lightclients::wasm},
};

use crate::{DecodeError, IbcClient, IbcClientError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("client state not found")]
    ClientStateNotFound,
    #[error("client state decode failed")]
    ClientStateDecode,
    #[error("consensus state decode failed")]
    ConsensusStateDecode,
}

// Client state that is stored by the host
pub const HOST_CLIENT_STATE_KEY: &str = "clientState";
pub const HOST_CONSENSUS_STATES_KEY: &str = "consensusStates";

pub const SUBJECT_CLIENT_STORE_PREFIX: &str = "subject/";
pub const SUBSTITUTE_CLIENT_STORE_PREFIX: &str = "substitute/";

pub fn consensus_db_key(height: &Height) -> String {
    format!(
        "{}/{}-{}",
        HOST_CONSENSUS_STATES_KEY, height.revision_number, height.revision_height
    )
}

/// Reads the client state from the host.
///
/// The host stores the client state with 'HOST_CLIENT_STATE_KEY' key in the following format:
/// - type_url: WASM_CLIENT_STATE_TYPE_URL
/// - value: (PROTO_ENCODED_WASM_CLIENT_STATE)
///     - code_id: Code ID of this contract's code
///     - latest_height: Latest height that the state is updated
///     - data: Contract defined raw bytes, which we use as protobuf encoded concrete client state.
pub fn read_client_state<T>(
    deps: Deps<T::CustomQuery>,
) -> Result<wasm::client_state::ClientState<T::ClientState>, IbcClientError<T>>
where
    T: IbcClient,
    Any<wasm::client_state::ClientState<T::ClientState>>: Decode<Proto>,
{
    read_prefixed_client_state::<T>(deps, "")
}

/// Reads the consensus state at a specific height from the host.
///
/// The host stores the consensus state with 'HOST_CONSENSUS_STATES_KEY/REVISION_NUMBER-REVISION_HEIGHT'
/// key in the following format:
/// - type_url: WASM_CONSENSUS_STATE_TYPE_URL
/// - value: (PROTO_ENCODED_WASM_CLIENT_STATE)
///     - timestamp: Time of this consensus state.
///     - data: Contract defined raw bytes, which we use as protobuf encoded concrete consensus state.
pub fn read_consensus_state<T>(
    deps: Deps<T::CustomQuery>,
    height: &Height,
) -> Result<Option<wasm::consensus_state::ConsensusState<T::ConsensusState>>, IbcClientError<T>>
where
    Any<wasm::consensus_state::ConsensusState<T::ConsensusState>>: Decode<Proto>,
    T: IbcClient,
{
    read_prefixed_consensus_state::<T>(deps, height, "")
}

pub fn save_client_state<T: IbcClient>(
    deps: DepsMut<T::CustomQuery>,
    wasm_client_state: wasm::client_state::ClientState<T::ClientState>,
) {
    let any_state = Any(wasm_client_state);
    deps.storage.set(
        HOST_CLIENT_STATE_KEY.as_bytes(),
        any_state.encode().as_slice(),
    );
}

pub fn save_proto_client_state<T: IbcClient>(
    deps: DepsMut<T::CustomQuery>,
    proto_wasm_state: ProtoClientState,
) {
    let any_state = ProtoAny {
        type_url: <ProtoClientState as ::prost::Name>::type_url(),
        value: proto_wasm_state.encode_to_vec(),
    };

    deps.storage.set(
        HOST_CLIENT_STATE_KEY.as_bytes(),
        any_state.encode_to_vec().as_slice(),
    );
}

/// Update the client state on the host store.
pub fn update_client_state<T: IbcClient>(
    deps: DepsMut<T::CustomQuery>,
    mut wasm_client_state: wasm::client_state::ClientState<T::ClientState>,
    latest_height: u64,
) {
    wasm_client_state.latest_height = Height {
        revision_number: wasm_client_state.latest_height.revision_number,
        revision_height: latest_height,
    };

    save_client_state::<T>(deps, wasm_client_state);
}

pub fn save_consensus_state<T: IbcClient>(
    deps: DepsMut<T::CustomQuery>,
    wasm_consensus_state: wasm::consensus_state::ConsensusState<T::ConsensusState>,
    height: &Height,
) {
    deps.storage.set(
        consensus_db_key(height).as_bytes(),
        &Any(wasm_consensus_state).encode(),
    );
}

pub fn save_proto_consensus_state<T: IbcClient>(
    deps: DepsMut<T::CustomQuery>,
    proto_wasm_state: ProtoConsensusState,
    height: &Height,
) {
    let any_state = ProtoAny {
        type_url: <ProtoConsensusState as ::prost::Name>::type_url(),
        value: proto_wasm_state.encode_to_vec(),
    };

    deps.storage.set(
        consensus_db_key(height).as_bytes(),
        any_state.encode_to_vec().as_slice(),
    );
}

/// Reads the client state from the subject's (this client) store
pub fn read_subject_client_state<T>(
    deps: Deps<T::CustomQuery>,
) -> Result<wasm::client_state::ClientState<T::ClientState>, IbcClientError<T>>
where
    Any<wasm::client_state::ClientState<T::ClientState>>: Decode<Proto>,
    T: IbcClient,
{
    read_prefixed_client_state::<T>(deps, SUBJECT_CLIENT_STORE_PREFIX)
}

/// Reads the client state from the substitute's (other client) store
pub fn read_substitute_client_state<T>(
    deps: Deps<T::CustomQuery>,
) -> Result<wasm::client_state::ClientState<T::ClientState>, IbcClientError<T>>
where
    Any<wasm::client_state::ClientState<T::ClientState>>: Decode<Proto>,
    T: IbcClient,
{
    read_prefixed_client_state::<T>(deps, SUBSTITUTE_CLIENT_STORE_PREFIX)
}

pub fn save_subject_client_state<T>(
    deps: DepsMut<T::CustomQuery>,
    wasm_client_state: wasm::client_state::ClientState<T::ClientState>,
) where
    T: IbcClient,
{
    let any_state = Any(wasm_client_state);
    deps.storage.set(
        format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
        any_state.encode().as_slice(),
    );
}

pub fn read_subject_consensus_state<T>(
    deps: Deps<T::CustomQuery>,
    height: &Height,
) -> Result<Option<wasm::consensus_state::ConsensusState<T::ConsensusState>>, IbcClientError<T>>
where
    T: IbcClient,
{
    read_prefixed_consensus_state(deps, height, SUBJECT_CLIENT_STORE_PREFIX)
}

pub fn read_substitute_consensus_state<T>(
    deps: Deps<T::CustomQuery>,
    height: &Height,
) -> Result<Option<wasm::consensus_state::ConsensusState<T::ConsensusState>>, IbcClientError<T>>
where
    T: IbcClient,
{
    read_prefixed_consensus_state(deps, height, SUBSTITUTE_CLIENT_STORE_PREFIX)
}

pub fn save_subject_consensus_state<T>(
    deps: DepsMut<T::CustomQuery>,
    wasm_consensus_state: wasm::consensus_state::ConsensusState<T::ConsensusState>,
    height: &Height,
) where
    T: IbcClient,
{
    deps.storage.set(
        format!("{SUBJECT_CLIENT_STORE_PREFIX}{}", consensus_db_key(height)).as_bytes(),
        &Any(wasm_consensus_state).encode_as::<Proto>(),
    );
}

fn read_prefixed_client_state<T>(
    deps: Deps<T::CustomQuery>,
    prefix: &str,
) -> Result<wasm::client_state::ClientState<T::ClientState>, IbcClientError<T>>
where
    T: IbcClient,
{
    let any_state = deps
        .storage
        .get(format!("{prefix}{HOST_CLIENT_STATE_KEY}").as_bytes())
        .ok_or(IbcClientError::<T>::ClientStateNotFound)?;

    Any::decode(any_state.as_slice())
        .map(|any| any.0)
        .map_err(DecodeError::AnyWasmClientState)
        .map_err(IbcClientError::<T>::Decode)
}

fn read_prefixed_consensus_state<T>(
    deps: Deps<T::CustomQuery>,
    height: &Height,
    prefix: &str,
) -> Result<Option<wasm::consensus_state::ConsensusState<T::ConsensusState>>, IbcClientError<T>>
where
    T: IbcClient,
{
    deps.storage
        .get(format!("{prefix}{}", consensus_db_key(height)).as_bytes())
        .map(|bytes| Any::decode(&bytes).map(|any| any.0))
        .transpose()
        .map_err(DecodeError::AnyWasmConsensusState)
        .map_err(IbcClientError::<T>::Decode)
}
