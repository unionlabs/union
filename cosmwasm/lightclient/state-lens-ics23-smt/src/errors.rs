use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bytes, H256, U256},
};

use crate::client::StateLensIcs23SmtLightClient;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("aptos verifier failure ({0})")]
    AptosVerifier(#[from] aptos_verifier::Error),

    #[error("unimplemented feature")]
    Unimplemented,

    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("verify l2 membership error")]
    VerifyL2Membership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error("error while querying l1 state: {0}")]
    L1Error(#[from] IbcClientError<CometblsLightClient>),

    #[error("could not decode l2 consensus state ({0:?})")]
    L2ConsensusStateDecode(Bytes),

    #[error("commitment key must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentKeyLength(Vec<u8>),

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("invalid commitment key, expected ({expected:#x}) but found ({found:#x})")]
    InvalidCommitmentKey { expected: U256, found: U256 },

    #[error("commitment value must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentValueLength(Vec<u8>),

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("could not decode the proof")]
    ProofDecode(bincode::error::DecodeError),

    #[error("membership proof without a value")]
    MembershipProofWithoutValue,

    #[error("proof value {proof_value} doesn't match the given value {given})", proof_value = serde_utils::to_hex(.0), given = serde_utils::to_hex(.1))]
    ProofValueMismatch(Vec<u8>, Vec<u8>),

    #[error("proof value hash doesn't match the calculated one")]
    ProofValueHashMismatch,

    #[error("proof key hash doesn't match the calculated one")]
    ProofKeyMismatch,
}

impl From<Error> for IbcClientError<StateLensIcs23SmtLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}
