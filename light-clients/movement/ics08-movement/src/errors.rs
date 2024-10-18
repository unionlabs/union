use ics008_wasm_client::IbcClientError;
use unionlabs::{
    aptos::storage_proof::proto::TryFromStorageProofError,
    encoding::{DecodeErrorOf, Proto},
    google::protobuf::any::Any,
    ibc::{
        core::client::height::Height,
        lightclients::{cometbls, movement, wasm},
    },
    TryFromProtoBytesError,
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
    #[error("membership proof with no value")]
    MembershipProofWithoutValue,
    #[error("proof value {proof_value} doesn't match the given value {given})", proof_value = serde_utils::to_hex(.0), given = serde_utils::to_hex(.1))]
    ProofValueMismatch(Vec<u8>, Vec<u8>),
    #[error("proof value hash doesn't match the calculated one")]
    ProofValueHashMismatch,
    #[error("proof key hash doesn't match the calculated one")]
    ProofKeyMismatch,
    #[error("storage proof decode: {0}")]
    StorageProofDecode(#[from] TryFromProtoBytesError<TryFromStorageProofError>),
    #[error("invalid ibc path {0}")]
    InvalidIbcPath(String),
    #[error("unable to decode counterparty's stored cometbls client state")]
    CometblsClientStateDecode(
        #[source] DecodeErrorOf<Proto, Any<cometbls::client_state::ClientState>>,
    ),
    #[error("unable to decode counterparty's stored cometbls consensus state")]
    CometblsConsensusStateDecode(
        #[source]
        DecodeErrorOf<
            Proto,
            Any<wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>>,
        >,
    ),
}

impl From<Error> for IbcClientError<MovementLightClient> {
    fn from(value: Error) -> Self {
        Self::ClientSpecific(value)
    }
}
