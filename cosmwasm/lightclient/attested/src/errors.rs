use cosmwasm_std::StdError;
use ibc_union_light_client::{IbcClientError, spec::Timestamp};
use unionlabs::primitives::{Bytes, H256};

use crate::{client::AttestedLightClient, types::AttestationValue};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("no misbehaviour in an attested client")]
    NoMisbehaviourInAttestedClient,

    #[error("unauthorized call")]
    Unauthorized,

    #[error("key {key} has not been attested to at height {height}")]
    KeyNotAttested { height: u64, key: Bytes },

    #[error(
        "key {key} was attested to at height {height} with value {value} but \
        attempted to verify against value {value}"
    )]
    InvalidAttestedValue {
        height: u64,
        key: Bytes,
        value: AttestationValue,
        attested: AttestationValue,
    },

    #[error(
        "height {height} was attested to with timestamp {attested_timestamp}, \
        but attempted to update with timestamp {timestamp}"
    )]
    InvalidTimestamp {
        height: u64,
        attested_timestamp: Timestamp,
        timestamp: Timestamp,
    },

    #[error(
        "(height: {height}, timestamp: {timestamp}, key: {key}) has already been \
        attested to be {value}"
    )]
    AlreadyAttested {
        height: u64,
        timestamp: Timestamp,
        key: Bytes,
        value: AttestationValue,
    },

    #[error("invalid attestation signature")]
    InvalidSignature,

    #[error("attestation already received")]
    AttestationAlreadyReceived,

    #[error("{attestor} is not a valid attestor")]
    InvalidAttestor { attestor: H256 },

    #[error(
        "height {height} was previously attested to timestamp {previously_attested_timestamp}, \
        but this attestation is for timestamp {timestamp}"
    )]
    InconsistentTimestamp {
        height: u64,
        timestamp: Timestamp,
        previously_attested_timestamp: Timestamp,
    },

    #[error("no attestation found for height {height}, key {key}")]
    AttestationNotFound { height: u64, key: Bytes },
}

impl From<Error> for IbcClientError<AttestedLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
