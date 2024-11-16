use cosmwasm_std::StdError;
use movement_light_client_types::{ClientState, ConsensusState};
use unionlabs::{
    aptos::storage_proof::TryFromStorageProofError,
    encoding::{DecodeErrorOf, Proto},
    ibc::core::client::height::Height,
    TryFromProtoBytesError,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(#[source] DecodeErrorOf<Proto, ConsensusState>),
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
    #[error(transparent)]
    StdError(#[from] StdError),
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}
