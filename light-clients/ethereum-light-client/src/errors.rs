use ics008_wasm_client::IbcClientError;
use unionlabs::{bls::BlsPublicKey, hash::H256, ibc::core::client::height::Height};

use crate::client::EthereumLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,

    #[error("error while decoding proto ({reason})")]
    DecodeFromProto { reason: String },

    #[error("client state not found")]
    ClientStateNotFound,

    #[error("custom query error")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

    #[error(
        "given trusted sync committee doesn't match the given aggregate public \
        key ({given_aggregate}) or the stored one ({stored_aggregate})"
    )]
    TrustedSyncCommitteeMismatch {
        stored_aggregate: BlsPublicKey,
        given_aggregate: BlsPublicKey,
    },

    #[error("active sync committee is `next` but there is no next sync committee in the consensus state")]
    NoNextSyncCommittee,

    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("validate light client error")]
    ValidateLightClient(#[source] ethereum_verifier::Error),

    #[error("verify account storage root error")]
    VerifyAccountStorageRoot(#[source] ethereum_verifier::Error),

    #[error("verify storage absence error")]
    VerifyStorageAbsence(#[source] ethereum_verifier::Error),

    #[error("verify storage proof error")]
    VerifyStorageProof(#[source] ethereum_verifier::Error),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("invalid commitment key, expected ({expected}) but found ({found})")]
    InvalidCommitmentKey { expected: H256, found: H256 },

    #[error("client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("proof is empty")]
    EmptyProof,

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("expected value ({expected:?}) and stored value ({stored:?}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("the proof path {0} is not unknown")]
    UnknownIbcPath(String),

    #[error("not enough signatures")]
    NotEnoughSignatures,

    #[error("integer arithmetic overflow")]
    IntegerOverflow,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("misbehaviour can only exist if there exists two conflicting headers, the provided headers are not at the same height ({0} != {1})")]
    MisbehaviourCannotExist(u64, u64),
}

impl From<Error> for IbcClientError<EthereumLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
