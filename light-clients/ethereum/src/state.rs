use crate::{errors::Error, ClientState};
use cosmwasm_std::{from_slice, Deps};
use cw_storage_plus::Item;
use ibc_proto::{
    google::protobuf::Any, ibc::lightclients::wasm::v1::ClientState as WasmClientState,
};
use prost::Message;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Client state that is stored by the host
pub const HOST_CLIENT_STATE_KEY: &[u8] = b"clientState";

pub const CLIENT_STATE: Item<ClientState> = Item::new("client_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {}

pub fn extract_client_state_from_wasm(deps: Deps) -> Result<ClientState, Error> {
    let any_state = deps
        .storage
        .get(HOST_CLIENT_STATE_KEY)
        .ok_or(Error::ClientStateNotFound)?;

    let any_state = Any::decode(any_state.as_slice()).map_err(|_| Error::DecodeError)?;

    let wasm_client_state =
        WasmClientState::decode(any_state.value.as_slice()).map_err(|_| Error::DecodeError)?;

    from_slice(wasm_client_state.data.as_slice()).map_err(|_| Error::DecodeError)
}
