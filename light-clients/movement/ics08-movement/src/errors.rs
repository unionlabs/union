use ics008_wasm_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::{core::client::height::Height, lightclients::movement},
};

use crate::client::MovementLightClient;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, movement::client_state::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(#[source] DecodeErrorOf<Proto, movement::consensus_state::ConsensusState>),
    #[error("error while calling custom query: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),
    #[error("header verification failure ({0})")]
    HeaderVerification(#[from] aptos_verifier::Error),
    #[error("invalid state_proof storage proof")]
    InvalidStateProof,
    #[error("empty ibc path")]
    EmptyIbcPath,
    #[error("consensus state not found ({0})")]
    ConsensusStateNotFound(Height),
}

impl From<Error> for IbcClientError<MovementLightClient> {
    fn from(value: Error) -> Self {
        Self::ClientSpecific(value)
    }
}
