use cosmwasm_std::{Deps, DepsMut};
use ibc_types::{
    ibc::{
        core::client::height::Height,
        google::protobuf::any::Any,
        lightclients::{ethereum, wasm},
    },
    IntoProto, TryFromProto,
};

use crate::{custom_query::CustomQuery, errors::Error};

// Client state that is stored by the host
pub const HOST_CLIENT_STATE_KEY: &str = "clientState";
pub const HOST_CONSENSUS_STATES_KEY: &str = "consensusStates";

fn consensus_db_key(height: &Height) -> String {
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
///     - data: Contract defined raw bytes, which we use as protobuf encoded ethereum client state.
pub fn read_client_state(
    deps: Deps<CustomQuery>,
) -> Result<wasm::client_state::ClientState<ethereum::client_state::ClientState>, Error> {
    let any_state = deps
        .storage
        .get(HOST_CLIENT_STATE_KEY.as_bytes())
        .ok_or(Error::ClientStateNotFound)?;

    Any::try_from_proto_bytes(any_state.as_slice())
        .map(|any| any.0)
        .map_err(|err| {
            Error::decode(format!(
                "when decoding raw bytes to any in `read_client_state`: {err:#?}"
            ))
        })
}

/// Reads the consensus state at a specific height from the host.
///
/// The host stores the consensus state with 'HOST_CONSENSUS_STATES_KEY/REVISION_NUMBER-REVISION_HEIGHT'
/// key in the following format:
/// - type_url: WASM_CONSENSUS_STATE_TYPE_URL
/// - value: (PROTO_ENCODED_WASM_CLIENT_STATE)
///     - timestamp: Time of this consensus state.
///     - data: Contract defined raw bytes, which we use as protobuf encoded ethereum consensus state.
pub fn read_consensus_state(
    deps: Deps<CustomQuery>,
    height: &Height,
) -> Result<
    Option<wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>>,
    Error,
> {
    deps.storage
        .get(consensus_db_key(height).as_bytes())
        .map(|bytes| Any::try_from_proto_bytes(&bytes).map(|any| any.0))
        .transpose()
        .map_err(|err| Error::decode(format!("error reading consensus state: {err:#?}")))
}

pub fn save_wasm_client_state(
    deps: DepsMut<CustomQuery>,
    wasm_client_state: wasm::client_state::ClientState<ethereum::client_state::ClientState>,
) {
    let any_state = Any(wasm_client_state);
    deps.storage.set(
        HOST_CLIENT_STATE_KEY.as_bytes(),
        any_state.into_proto_bytes().as_slice(),
    );
}

/// Update the client state on the host store.
pub fn update_client_state(
    deps: DepsMut<CustomQuery>,
    mut wasm_client_state: wasm::client_state::ClientState<ethereum::client_state::ClientState>,
    // new_client_state: ethereum::client_state::ClientState,
    latest_execution_height: u64,
) {
    // wasm_client_state.data = new_client_state;
    wasm_client_state.latest_height = Height {
        revision_number: 0,
        revision_height: latest_execution_height,
    };

    save_wasm_client_state(deps, wasm_client_state);
}

pub fn save_wasm_consensus_state(
    deps: DepsMut<CustomQuery>,
    wasm_consensus_state: wasm::consensus_state::ConsensusState<
        ethereum::consensus_state::ConsensusState,
    >,
    height: &Height,
) {
    deps.storage.set(
        consensus_db_key(height).as_bytes(),
        &Any(wasm_consensus_state).into_proto_bytes(),
    );
}

/// Save new consensus state at height `consensus_state.slot` to the host store.
pub fn save_consensus_state(
    deps: DepsMut<CustomQuery>,
    wasm_consensus_state: wasm::consensus_state::ConsensusState<
        ethereum::consensus_state::ConsensusState,
    >,
    // new_consensus_state: ethereum::consensus_state::ConsensusState,
    execution_height: u64,
) -> Result<(), Error> {
    let height = Height {
        revision_number: 0,
        revision_height: execution_height,
    };
    // REVIEW: Is the timestamp set properly somewhere else?
    // wasm_consensus_state.timestamp = new_consensus_state.timestamp;
    // wasm_consensus_state.data = new_consensus_state;
    save_wasm_consensus_state(deps, wasm_consensus_state, &height);
    Ok(())
}
