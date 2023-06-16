use crate::{client_state::ClientState, consensus_state::ConsensusState, errors::Error};
use cosmwasm_std::{Deps, DepsMut};
use ibc::Height;
use prost::Message;
use protos::{
    google::protobuf::Any,
    ibc::lightclients::wasm::v1::{
        ClientState as WasmClientState, ConsensusState as WasmConsensusState,
    },
    union::ibc::lightclients::ethereum::v1::{
        ClientState as RawClientState, ConsensusState as RawConsensusState,
    },
};

pub const WASM_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientState";
pub const WASM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ConsensusState";

// Client state that is stored by the host
pub const HOST_CLIENT_STATE_KEY: &str = "clientState";
pub const HOST_CONSENSUS_STATES_KEY: &str = "consensusStates";

fn consensus_db_key(height: &Height) -> String {
    format!(
        "{}/{}-{}",
        HOST_CONSENSUS_STATES_KEY,
        height.revision_number(),
        height.revision_height()
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
pub fn read_client_state(deps: Deps) -> Result<(WasmClientState, ClientState), Error> {
    let any_state = deps
        .storage
        .get(HOST_CLIENT_STATE_KEY.as_bytes())
        .ok_or(Error::ClientStateNotFound)?;

    let any_state = Any::decode(any_state.as_slice())
        .map_err(|_| Error::decode("when decoding raw bytes to any in `read_client_state`"))?;

    let wasm_client_state = WasmClientState::decode(any_state.value.as_slice()).map_err(|_| {
        Error::decode("when decoding any value to wasm client state in `read_client_state`")
    })?;

    let raw_client_state =
        RawClientState::decode(wasm_client_state.data.as_slice()).map_err(|_| {
            Error::decode(
                "when decoding wasm client state to eth client state in `read_client_state`",
            )
        })?;

    ClientState::try_from(raw_client_state).map(|cs| (wasm_client_state, cs))
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
    deps: Deps,
    height: Height,
) -> Result<Option<(WasmConsensusState, ConsensusState)>, Error> {
    let any_state = if let Some(state) = deps.storage.get(consensus_db_key(&height).as_bytes()) {
        state
    } else {
        return Ok(None);
    };

    let any_state = Any::decode(any_state.as_slice())
        .map_err(|_| Error::decode("when decoding raw bytes to any in `read_consensus_state`"))?;

    let wasm_consensus_state =
        WasmConsensusState::decode(any_state.value.as_slice()).map_err(|_| {
            Error::decode(
                "when decoding any value to wasm consensus state in `read_consensus_state`",
            )
        })?;

    let raw_client_state = RawConsensusState::decode(wasm_consensus_state.data.as_slice())
        .map_err(|_| Error::decode("when decoding wasm consensus state to eth consensus state in `read_consensus_state`"))?;

    ConsensusState::try_from(raw_client_state).map(|cs| Some((wasm_consensus_state, cs)))
}

pub fn save_wasm_client_state(deps: DepsMut, wasm_client_state: &WasmClientState) {
    let any_state = Any {
        type_url: WASM_CLIENT_STATE_TYPE_URL.into(),
        value: wasm_client_state.encode_to_vec(),
    };
    deps.storage.set(
        HOST_CLIENT_STATE_KEY.as_bytes(),
        any_state.encode_to_vec().as_slice(),
    );
}

/// Update the client state on the host store.
pub fn update_client_state(
    deps: DepsMut,
    mut wasm_client_state: WasmClientState,
    client_state: ClientState,
) {
    let latest_revision_height = client_state.latest_slot;
    let raw_client_state = Into::<RawClientState>::into(client_state).encode_to_vec();

    wasm_client_state.data = raw_client_state;
    wasm_client_state.latest_height = Some(protos::ibc::core::client::v1::Height {
        revision_number: 0,
        revision_height: latest_revision_height,
    });

    save_wasm_client_state(deps, &wasm_client_state);
}

pub fn save_wasm_consensus_state(
    deps: DepsMut,
    wasm_consensus_state: &WasmConsensusState,
    height: &Height,
) {
    let any_state = Any {
        type_url: WASM_CONSENSUS_STATE_TYPE_URL.into(),
        value: wasm_consensus_state.encode_to_vec(),
    };
    deps.storage.set(
        consensus_db_key(height).as_bytes(),
        any_state.encode_to_vec().as_slice(),
    );
}

/// Save new consensus state at height `consensus_state.slot` to the host store.
pub fn save_consensus_state(
    deps: DepsMut,
    mut wasm_consensus_state: WasmConsensusState,
    consensus_state: ConsensusState,
) -> Result<(), Error> {
    let height = Height::new(0, consensus_state.slot).map_err(|_| Error::InvalidHeight)?;
    let timestamp = consensus_state.timestamp;
    let raw_consensus_state = Into::<RawConsensusState>::into(consensus_state).encode_to_vec();
    wasm_consensus_state.data = raw_consensus_state;
    wasm_consensus_state.timestamp = timestamp;
    save_wasm_consensus_state(deps, &wasm_consensus_state, &height);
    Ok(())
}
