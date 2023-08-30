use prost::Message;
use protos::{
    google::protobuf::Any,
    ibc::lightclients::wasm::v1::{
        ClientState as RawWasmClientState, ConsensusState as RawWasmConsensusState,
    },
};

mod ibc_client;
pub mod msg;

pub use ibc_client::*;

pub enum Error {
    Decode(String),
    NotSpecCompilant(String),
}

impl Error {
    pub fn decode<S: Into<String>>(msg: S) -> Error {
        Error::Decode(msg.into())
    }
}

pub fn decode_client_state_to_concrete_state<T: Message + Default>(
    state: &[u8],
) -> Result<T, Error> {
    let any_state = Any::decode(state)
        .map_err(|_| Error::decode("when decoding raw bytes to any in `verify_membership`"))?;

    let wasm_client_state =
        RawWasmClientState::decode(any_state.value.as_slice()).map_err(|_| {
            Error::decode("when decoding any value to wasm client state in `verify_membership`")
        })?;

    let any_state = Any::decode(wasm_client_state.data.as_slice()).map_err(|_| {
        Error::decode("when decoding wasm client state to tm client state in `verify_membership`")
    })?;

    T::decode(any_state.value.as_slice()).map_err(|_| {
        Error::decode("when decoding any state to tm client state in `verify_membership`")
    })
}

pub fn decode_consensus_state_to_concrete_state<T: Message + Default>(
    state: &[u8],
) -> Result<T, Error> {
    let any_state = Any::decode(state)
        .map_err(|_| Error::decode("when decoding raw bytes to any in `verify_membership`"))?;

    let wasm_consensus_state =
        RawWasmConsensusState::decode(any_state.value.as_slice()).map_err(|_| {
            Error::decode("when decoding any value to wasm client state in `verify_membership`")
        })?;

    let any_state = Any::decode(wasm_consensus_state.data.as_slice()).map_err(|_| {
        Error::decode("when decoding wasm client state to tm client state in `verify_membership`")
    })?;

    T::decode(any_state.value.as_slice()).map_err(|_| {
        Error::decode("when decoding any state to tm client state in `verify_membership`")
    })
}
