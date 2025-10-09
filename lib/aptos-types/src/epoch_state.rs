// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

use crate::validator_verifier::ValidatorVerifier;

/// `EpochState` represents a trusted validator set to validate messages from the specific epoch,
/// it could be updated with `EpochChangeProof`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct EpochState {
    pub epoch: u64,
    pub verifier: ValidatorVerifier,
}

impl fmt::Display for EpochState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EpochState [epoch: {}, validator: {:?}]",
            self.epoch, self.verifier
        )
    }
}
