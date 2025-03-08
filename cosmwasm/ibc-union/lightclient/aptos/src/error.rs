use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;
use unionlabs::ibc::core::client::height::Height;

use crate::client::AptosLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
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

impl From<Error> for IbcClientError<AptosLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
