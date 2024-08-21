// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    block_info::BlockInfo, epoch_change::EpochChangeProof, hash_value::HashValue, types::Signature,
};

/// A convenience type for the collection of sub-proofs that consistitute a
/// response to a `get_state_proof` request.
///
/// From a `StateProof` response, a client should be able to ratchet their
/// [`TrustedState`] to the last epoch change LI in the [`EpochChangeProof`]
/// or the latest [`LedgerInfoWithSignatures`] if the epoch changes get them into
/// the most recent epoch.
///
/// [`TrustedState`]: crate::trusted_state::TrustedState
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateProof {
    pub latest_li_w_sigs: LedgerInfoWithSignatures,
    pub epoch_changes: EpochChangeProof,
}

impl StateProof {
    pub fn new(
        latest_li_w_sigs: LedgerInfoWithSignatures,
        epoch_changes: EpochChangeProof,
    ) -> Self {
        Self {
            latest_li_w_sigs,
            epoch_changes,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        bcs::serialize_into(&mut hasher, self).expect("unexpected serialization error");
        hasher.finalize().into()
    }

    pub fn latest_ledger_info(&self) -> &LedgerInfo {
        let LedgerInfoWithSignatures::V0(ledger_info) = &self.latest_li_w_sigs;
        &ledger_info.ledger_info
    }
}

/// Wrapper around LedgerInfoWithScheme to support future upgrades, this is the data being persisted.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerInfoWithSignatures {
    V0(LedgerInfoWithV0),
}

/// The validator node returns this structure which includes signatures
/// from validators that confirm the state.  The client needs to only pass back
/// the LedgerInfo element since the validator node doesn't need to know the signatures
/// again when the client performs a query, those are only there for the client
/// to be able to verify the state
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfoWithV0 {
    pub ledger_info: LedgerInfo,
    /// Aggregated BLS signature of all the validators that signed the message. The bitmask in the
    /// aggregated signature can be used to find out the individual validators signing the message
    pub signatures: AggregateSignature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AggregateSignature {
    validator_bitmask: BitVec,
    sig: Option<Signature>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfo {
    pub commit_info: BlockInfo,

    /// Hash of consensus specific data that is opaque to all parts of the system other than
    /// consensus.
    pub consensus_data_hash: HashValue,
}

// TODO(aeryz): custom deserialize
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BitVec {
    #[serde(with = "serde_bytes")]
    inner: Vec<u8>,
}
