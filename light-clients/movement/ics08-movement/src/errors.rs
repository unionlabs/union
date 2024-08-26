use ics008_wasm_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::lightclients::movement,
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
}

impl From<Error> for IbcClientError<MovementLightClient> {
    fn from(value: Error) -> Self {
        Self::ClientSpecific(value)
    }
}
