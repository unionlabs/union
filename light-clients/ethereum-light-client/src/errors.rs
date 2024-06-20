use ics008_wasm_client::IbcClientError;
use unionlabs::{
    bls::BlsPublicKey,
    encoding::{DecodeErrorOf, Proto},
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::client::height::Height,
        lightclients::{
            cometbls,
            ethereum::{self, storage_proof::StorageProof},
            wasm,
        },
    },
    uint::U256,
};

use crate::client::EthereumLightClient;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,

    #[error("unable to decode storage proof")]
    StorageProofDecode(#[source] DecodeErrorOf<Proto, StorageProof>),

    #[error("client state not found")]
    ClientStateNotFound,

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, ethereum::client_state::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(#[source] DecodeErrorOf<Proto, ethereum::consensus_state::ConsensusState>),

    #[error(transparent)]
    CanonicalizeStoredValue(#[from] CanonicalizeStoredValueError),

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
    ValidateLightClient(#[source] ethereum_verifier::error::Error),

    #[error("verify account storage root error")]
    VerifyAccountStorageRoot(#[source] ethereum_verifier::error::Error),

    #[error("verify storage absence error")]
    VerifyStorageAbsence(#[source] ethereum_verifier::error::Error),

    #[error("verify storage proof error")]
    VerifyStorageProof(#[source] ethereum_verifier::error::Error),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error(transparent)]
    InvalidCommitmentKey(#[from] InvalidCommitmentKey),

    #[error("client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("proof is empty")]
    EmptyProof,

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error(transparent)]
    // TODO: use in other eth l2s
    StoredValueMismatch(#[from] StoredValueMismatch),

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

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum CanonicalizeStoredValueError {
    #[error("the proof path {0} is unknown")]
    UnknownIbcPath(String),
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

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
#[error("invalid commitment key, expected ({expected:#x}) but found ({found:#x})")]
pub struct InvalidCommitmentKey {
    pub expected: U256,
    pub found: U256,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("expected value ({expected}) and stored value ({stored}) don't match")]
pub struct StoredValueMismatch {
    pub expected: H256,
    pub stored: H256,
}

impl From<Error> for IbcClientError<EthereumLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
