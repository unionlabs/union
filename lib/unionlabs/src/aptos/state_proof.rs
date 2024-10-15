// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0
use macros::model;
use sha2::Digest;

use crate::{
    aptos::{
        block_info::BlockInfo,
        epoch_change::EpochChangeProof,
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        signature::AggregateSignature,
    },
    hash::hash_v2::Hash,
};

/// A convenience type for the collection of sub-proofs that constitute a
/// response to a `get_state_proof` request.
///
/// From a `StateProof` response, a client should be able to ratchet their
/// `TrustedState` to the last epoch change LI in the [`EpochChangeProof`]
/// or the latest [`LedgerInfoWithSignatures`] if the epoch changes get them into
/// the most recent epoch.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::StateProof),
    into,
    from
))]
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
                        id: Hash::default(),
                        executed_state_id: Hash::default(),
                        version: 0,
                        timestamp_usecs: 0,
                        next_epoch_state: None,
                    },
                    consensus_data_hash: Hash::default(),
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
    // TODO: Remove
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

    #[cfg(feature = "bcs")]
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // panics are impossible
    pub fn hash(&self) -> [u8; 32] {
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

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        aptos::{
            epoch_change::proto::TryFromEpochChangeProof,
            ledger_info::proto::TryFromLedgerInfoWithSignatures, state_proof::StateProof,
        },
        errors::{required, MissingField},
    };

    impl From<StateProof> for protos::union::ibc::lightclients::movement::v1::StateProof {
        fn from(value: StateProof) -> Self {
            Self {
                latest_li_w_sigs: Some(value.latest_li_w_sigs.into()),
                epoch_changes: Some(value.epoch_changes.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromStateProofError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid latest ledger info with sigs")]
        LatestLiWSigs(#[from] TryFromLedgerInfoWithSignatures),
        #[error("invalid epoch changes")]
        EpochChanges(#[from] TryFromEpochChangeProof),
    }

    impl TryFrom<protos::union::ibc::lightclients::movement::v1::StateProof> for StateProof {
        type Error = TryFromStateProofError;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::StateProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                latest_li_w_sigs: required!(value.latest_li_w_sigs)?.try_into()?,
                epoch_changes: required!(value.epoch_changes)?.try_into()?,
            })
        }
    }
}
