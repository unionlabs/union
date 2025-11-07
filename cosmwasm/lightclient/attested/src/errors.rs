use std::num::NonZero;

use cosmwasm_std::StdError;
use ibc_union_light_client::{IbcClientError, access_managed, spec::Timestamp};
use unionlabs::primitives::{Bytes, H256};

use crate::{client::AttestedLightClient, types::AttestationValue};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    AccessManaged(#[from] access_managed::error::ContractError),

    #[error("no misbehaviour in an attested client")]
    NoMisbehaviourInAttestedClient,

    #[error(
        "key {key} was attested to at height {height} with value {value} but \
        attempted to verify against value {value}"
    )]
    InvalidAttestedValue {
        chain_id: String,
        height: u64,
        key: Bytes,
        value: AttestationValue,
        attested: AttestationValue,
    },

    #[error(
        "height {height} on chain {chain_id} was attested to with timestamp \
        {attested_timestamp}, but attempted to update with timestamp {timestamp}"
    )]
    InvalidTimestamp {
        chain_id: String,
        height: u64,
        attested_timestamp: Timestamp,
        timestamp: Timestamp,
    },

    #[error(
        "(height: {height}, timestamp: {timestamp}, key: {key}) has already been \
        attested to be {value} on chain {chain_id}"
    )]
    AlreadyAttested {
        chain_id: String,
        height: u64,
        timestamp: Timestamp,
        key: Bytes,
        value: AttestationValue,
    },

    #[error(
        "height {height} on chain {chain_id} was previously attested to timestamp \
        {previously_attested_timestamp}, but this attestation is for timestamp {timestamp}"
    )]
    InconsistentTimestamp {
        chain_id: String,
        height: u64,
        timestamp: Timestamp,
        previously_attested_timestamp: Timestamp,
    },

    #[error("no attestation found for height {height}, key {key} on chain {chain_id}")]
    AttestationNotFound {
        chain_id: String,
        height: u64,
        key: Bytes,
    },

    #[error("invalid attestation signature")]
    InvalidSignature,

    #[error("attestation already received for chain {chain_id}")]
    AttestationAlreadyReceived { chain_id: String },

    #[error("{attestor} is not a valid attestor for chain {chain_id}")]
    InvalidAttestor { chain_id: String, attestor: H256 },

    #[error("attestor {attestor} is already in the attestation set for chain {chain_id}")]
    AttestorAlreadyExists { chain_id: String, attestor: H256 },

    #[error("the quorum has not yet been set for {chain_id}")]
    QuorumNotSet { chain_id: String },

    #[error(
        "the attestation has not yet hit the quorum required for \
        chain {chain_id}: {current}/{quorum}"
    )]
    QuorumNotReached {
        chain_id: String,
        quorum: NonZero<u8>,
        current: u8,
    },
}

impl From<Error> for IbcClientError<AttestedLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
