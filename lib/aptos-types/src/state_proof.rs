// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use unionlabs_primitives::FixedBytes;

use crate::{
    block_info::BlockInfo,
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    signature::AggregateSignature,
};

/// A convenience type for the collection of sub-proofs that constitute a
/// response to a `get_state_proof` request.
///
/// From a `StateProof` response, a client should be able to ratchet their
/// `TrustedState` to the last epoch change LI in the [`EpochChangeProof`]
/// or the latest [`LedgerInfoWithSignatures`] if the epoch changes get them into
/// the most recent epoch.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct StateProof {
    pub latest_li_w_sigs: LedgerInfoWithSignatures,
    pub epoch_changes: EpochChangeProof,
}

// TODO(aeryz): only for testing purposes, will remove this once we have state proofs
impl Default for StateProof {
    fn default() -> Self {
        Self {
            latest_li_w_sigs: LedgerInfoWithSignatures::V0(super::ledger_info::LedgerInfoWithV0 {
                ledger_info: LedgerInfo {
                    commit_info: BlockInfo {
                        epoch: 0,
                        round: 0,
                        id: FixedBytes::default(),
                        executed_state_id: FixedBytes::default(),
                        version: 0,
                        timestamp_usecs: 0,
                        next_epoch_state: None,
                    },
                    consensus_data_hash: FixedBytes::default(),
                },
                signatures: AggregateSignature {
                    validator_bitmask: super::signature::ValidatorBitmask { inner: vec![] },
                    sig: None,
                },
            }),
            epoch_changes: EpochChangeProof {
                ledger_info_with_sigs: vec![],
                more: false,
            },
        }
    }
}

impl StateProof {
    #[must_use]
    pub fn new(
        latest_li_w_sigs: LedgerInfoWithSignatures,
        epoch_changes: EpochChangeProof,
    ) -> Self {
        Self {
            latest_li_w_sigs,
            epoch_changes,
        }
    }

    #[cfg(feature = "std")]
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // panics are impossible
    pub fn hash(&self) -> [u8; 32] {
        use sha2::Digest;

        let mut hasher = sha2::Sha256::new();
        bcs::serialize_into(&mut hasher, self).expect("unexpected serialization error");
        hasher.finalize().into()
    }

    #[must_use]
    pub fn latest_ledger_info(&self) -> &LedgerInfo {
        let LedgerInfoWithSignatures::V0(ledger_info) = &self.latest_li_w_sigs;
        &ledger_info.ledger_info
    }
}
