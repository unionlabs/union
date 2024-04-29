use ics008_wasm_client::IbcClientError;
use scroll_codec::{
    batch_header::BatchHeaderDecodeError,
    chunk::{ChunkV0DecodeError, ChunkV1DecodeError},
};
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    google::protobuf::any::Any,
    hash::H256,
    ibc::{core::client::height::Height, lightclients::wasm},
    ics24::PathParseError,
};

use crate::client::ScrollLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("unable to decode storage proof")]
    StorageProofDecode(
        #[source]
        DecodeErrorOf<Proto, unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof>,
    ),
    #[error("unable to decode counterparty's stored cometbls client state")]
    CometblsClientStateDecode(
        #[source]
        DecodeErrorOf<
            Proto,
            Any<unionlabs::ibc::lightclients::cometbls::client_state::ClientState>,
        >,
    ),
    #[error("unable to decode counterparty's stored cometbls consensus state")]
    CometblsConsensusStateDecode(
        #[source]
        DecodeErrorOf<
            Proto,
            Any<
                wasm::consensus_state::ConsensusState<
                    unionlabs::ibc::lightclients::cometbls::consensus_state::ConsensusState,
                >,
            >,
        >,
    ),
    #[error("unable to decode client state")]
    ClientStateDecode(
        #[source]
        DecodeErrorOf<Proto, unionlabs::ibc::lightclients::scroll::client_state::ClientState>,
    ),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(
        #[source]
        DecodeErrorOf<
            Proto,
            unionlabs::ibc::lightclients::scroll::consensus_state::ConsensusState,
        >,
    ),

    // REVIEW: Move this variant to IbcClientError?
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("invalid commitment key, expected ({expected}) but found ({found})")]
    InvalidCommitmentKey { expected: H256, found: H256 },

    #[error("proof is empty")]
    EmptyProof,

    #[error("batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("unable to parse ics24 path")]
    PathParse(#[from] PathParseError),

    #[error("failed to verify scroll header")]
    Verify(#[from] scroll_verifier::Error),

    #[error("the operation has not been implemented yet")]
    Unimplemented,

    #[error("error while calling custom query: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

    // TODO: Condense all of these together?
    #[error("error decoding commit batch calldata")]
    CommitBatchDecode(#[from] ethers_core::abi::AbiError),
    #[error("empty batch")]
    EmptyBatch,
    #[error("error decoding v0 chunk")]
    ChunkV0Decode(#[from] ChunkV0DecodeError),
    #[error("error decoding v1 chunk")]
    ChunkV1Decode(#[from] ChunkV1DecodeError),
    #[error("error decoding batch header")]
    BatchHeaderDecode(#[from] BatchHeaderDecodeError),
}

impl From<Error> for IbcClientError<ScrollLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
