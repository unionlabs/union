use ics008_wasm_client::IbcClientError;
use near_primitives_core::hash::CryptoHash;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::lightclients::near::{client_state::ClientState, consensus_state::ConsensusState},
};

use crate::client::NearLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(u64),
    #[error("epoch block producer not found for epoch {0}")]
    EpochBlockProducerNotFound(CryptoHash),
    #[error(transparent)]
    Verifier(#[from] near_verifier::error::Error),
    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(#[source] DecodeErrorOf<Proto, ConsensusState>),
    #[error("the proof path {0} is unknown")]
    UnknownIbcPath(String),
    #[error("empty path")]
    EmptyPath,
    #[error("unable to decode cometbls state {0:?}")]
    ForeignStateDecode(Vec<u8>),
}

impl From<Error> for IbcClientError<NearLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
