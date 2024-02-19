use core::fmt::Debug;

use cosmwasm_std::{CustomQuery, Deps, DepsMut};
use prost::Message;
use protos::{
    google::protobuf::Any as ProtoAny,
    ibc::lightclients::wasm::v1::{
        ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
    },
};
use unionlabs::{
    google::protobuf::any::Any,
    ibc::{core::client::height::Height, lightclients::wasm},
    IntoProto, Proto, TryFromProto, TryFromProtoErrorOf,
};

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
pub fn read_client_state<C: CustomQuery, CS>(
    deps: Deps<C>,
) -> Result<wasm::client_state::ClientState<CS>, Error>
where
    wasm::client_state::ClientState<CS>: Proto + TryFromProto,
    Any<wasm::client_state::ClientState<CS>>: TryFromProto,
    TryFromProtoErrorOf<Any<wasm::client_state::ClientState<CS>>>: Debug,
{
    let any_state = deps
        .storage
        .get(HOST_CLIENT_STATE_KEY.as_bytes())
        .ok_or(Error::ClientStateNotFound)?;

    Any::try_from_proto_bytes(any_state.as_slice())
        .map(|any| any.0)
        .map_err(|_| Error::ClientStateDecode)
}

/// Reads the consensus state at a specific height from the host.
///
/// The host stores the consensus state with 'HOST_CONSENSUS_STATES_KEY/REVISION_NUMBER-REVISION_HEIGHT'
/// key in the following format:
/// - type_url: WASM_CONSENSUS_STATE_TYPE_URL
/// - value: (PROTO_ENCODED_WASM_CLIENT_STATE)
///     - timestamp: Time of this consensus state.
///     - data: Contract defined raw bytes, which we use as protobuf encoded concrete consensus state.
pub fn read_consensus_state<C: CustomQuery, CS>(
    deps: Deps<C>,
    height: &Height,
) -> Result<Option<wasm::consensus_state::ConsensusState<CS>>, Error>
where
    wasm::consensus_state::ConsensusState<CS>: Proto + TryFromProto,
    Any<wasm::consensus_state::ConsensusState<CS>>: TryFromProto,
    TryFromProtoErrorOf<Any<wasm::consensus_state::ConsensusState<CS>>>: Debug,
{
    deps.storage
        .get(consensus_db_key(height).as_bytes())
        .map(|bytes| Any::try_from_proto_bytes(&bytes).map(|any| any.0))
        .transpose()
        .map_err(|_| Error::ConsensusStateDecode)
}

pub fn save_client_state<C: CustomQuery, CS: IntoProto>(
    deps: DepsMut<C>,
    wasm_client_state: wasm::client_state::ClientState<CS>,
) {
    let any_state = Any(wasm_client_state);
    deps.storage.set(
        HOST_CLIENT_STATE_KEY.as_bytes(),
        any_state.into_proto_bytes().as_slice(),
    );
}

pub fn save_proto_client_state<C: CustomQuery>(
    deps: DepsMut<C>,
    proto_wasm_state: ProtoClientState,
) {
    let any_state = ProtoAny {
        type_url: <ProtoClientState as unionlabs::TypeUrl>::TYPE_URL.to_string(),
        value: proto_wasm_state.encode_to_vec(),
    };

    deps.storage.set(
        HOST_CLIENT_STATE_KEY.as_bytes(),
        any_state.encode_to_vec().as_slice(),
    );
}

/// Update the client state on the host store.
pub fn update_client_state<C: CustomQuery, CS: IntoProto>(
    deps: DepsMut<C>,
    mut wasm_client_state: wasm::client_state::ClientState<CS>,
    latest_height: u64,
) {
    wasm_client_state.latest_height = Height {
        revision_number: wasm_client_state.latest_height.revision_number,
        revision_height: latest_height,
    };

    save_client_state(deps, wasm_client_state);
}

pub fn save_consensus_state<C: CustomQuery, CS: IntoProto>(
    deps: DepsMut<C>,
    wasm_consensus_state: wasm::consensus_state::ConsensusState<CS>,
    height: &Height,
) {
    deps.storage.set(
        consensus_db_key(height).as_bytes(),
        &Any(wasm_consensus_state).into_proto_bytes(),
    );
}

pub fn save_proto_consensus_state<C: CustomQuery>(
    deps: DepsMut<C>,
    proto_wasm_state: ProtoConsensusState,
    height: &Height,
) {
    let any_state = ProtoAny {
        type_url: <ProtoConsensusState as unionlabs::TypeUrl>::TYPE_URL.to_string(),
        value: proto_wasm_state.encode_to_vec(),
    };

    deps.storage.set(
        consensus_db_key(height).as_bytes(),
        any_state.encode_to_vec().as_slice(),
    );
}
