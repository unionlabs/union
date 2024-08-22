// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::{
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
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
