use ethereum_light_client::errors::{CanonicalizeStoredValueError, InvalidCommitmentKey};
use ics008_wasm_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::client::height::Height,
        lightclients::{cometbls, linea, wasm},
    },
    ics24::PathParseError,
    linea::proof::InclusionProof,
};

use crate::client::LineaLightClient;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("unable to decode storage proof")]
    InclusionProofDecode(#[source] DecodeErrorOf<Proto, InclusionProof>),

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
    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, linea::client_state::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(#[source] DecodeErrorOf<Proto, linea::consensus_state::ConsensusState>),

    #[error(transparent)]
    CanonicalizeStoredValue(#[from] CanonicalizeStoredValueError),

    // REVIEW: Move this variant to IbcClientError?
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error(transparent)]
    InvalidCommitmentKey(#[from] InvalidCommitmentKey),

    #[error("proof is empty")]
    EmptyProof,

    #[error("batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("unable to parse ics24 path")]
    PathParse(#[from] PathParseError),

    #[error("failed to verify linea header: {0}")]
    Verify(#[from] linea_verifier::Error),

    #[error("the operation has not been implemented yet")]
    Unimplemented,

    #[error("error while calling custom query: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

    #[error("L2 account proof must be an inclusion proof")]
    InvalidL2AccountProof,

    #[error("failed to verify linea membership proof {0}")]
    InvalidMembershipProof(#[from] linea_zktrie::verify::Error),
}

impl From<Error> for IbcClientError<LineaLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
