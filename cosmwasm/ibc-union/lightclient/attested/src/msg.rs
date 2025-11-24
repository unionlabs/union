use std::num::NonZero;

use ibc_union_light_client::access_managed::Restricted;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, H256, H512};

use crate::types::Attestation;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Attest to a key/value state.
    ///
    /// `attestor` must be a valid attestor and must have signed the `attestation` payload.
    Attest {
        attestation: Attestation,
        attestor: H256,
        signature: H512,
    },
    /// Confirm an attestation.
    ///
    /// Attestations are typically confirmed in [`ExecuteMsg::Attest`] upon receiving the attestation that pushes the total attestations over the quorum. However, if the quorum is decreased while there are still pending attestations, then this can called to confirm those attestations if they have already hit the new quorum without requiring another attestation to be submitted (which may not be possible if there are not enough attestors in the set).
    ConfirmAttestation { attestation: Attestation },
    #[serde(untagged)]
    Restricted(Restricted<RestrictedExecuteMsg>),
    #[serde(untagged)]
    LightClient(ibc_union_light_client::msg::ExecuteMsg),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum RestrictedExecuteMsg {
    /// Set a new quorum for the attestations to be considered valid.
    ///
    /// If the new quorum is larger than the currently configured quorum, any existing attestations that have already hit the quorum will still be considered valid, but any current pending attestations will need to reach the new quorum in order to be confirmed.
    SetQuorum {
        chain_id: String,
        new_quorum: NonZero<u8>,
    },
    /// Add a new attestor to the attestation set.
    AddAttestor {
        chain_id: String,
        new_attestor: H256,
    },
    /// Remove an existing attestor from the attestation set.
    RemoveAttestor {
        chain_id: String,
        old_attestor: H256,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the currently configured quorum for `chain_id`.
    Quorum { chain_id: String },
    /// Returns the current attestation set for `chain_id`.
    Attestors { chain_id: String },
    /// Returns the value attested to under `key` at `height` on `chain_id`.
    AttestedValue {
        chain_id: String,
        #[serde(with = "serde_utils::string")]
        height: u64,
        key: Bytes,
    },
    /// Returns the timestamp attested to at `height` on `chain_id`.
    TimestampAtHeight {
        chain_id: String,
        #[serde(with = "serde_utils::string")]
        height: u64,
    },
    /// Returns the latest height and timestamp attested to for `chain_id`.
    LatestHeight { chain_id: String },
    #[serde(untagged)]
    LightClient(ibc_union_light_client::msg::QueryMsg),
}
