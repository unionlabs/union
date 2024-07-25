use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::{
        core::{client::height::Height, commitment::merkle_proof::MerkleProof},
        lightclients::cometbls,
    },
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("math operation with overflow")]
    MathOverflow,

    #[error("unimplemented feature")]
    Unimplemented,

    #[error("unable to decode merkle proof")]
    MerkleProofDecode(#[source] DecodeErrorOf<Proto, MerkleProof>),

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, cometbls::client_state::ClientState>),

    #[error("Client state not found")]
    ClientStateNotFound,

    #[error("Invalid ZKP: {0:?}")]
    InvalidZKP(cometbls_groth16_verifier::Error),

    #[error("Consensus state not found for {0}")]
    ConsensusStateNotFound(Height),

    #[error("verify membership error: {0}")]
    VerifyMembership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,

    #[error("the chain id cannot be more than 31 bytes long to fit in the bn254 scalar field")]
    InvalidChainId,

    #[error("invalid zkp length")]
    InvalidZKPLength,

    #[error("invalid height")]
    InvalidHeight,

    #[error("invalid timestamp")]
    InvalidTimestamp,
}
