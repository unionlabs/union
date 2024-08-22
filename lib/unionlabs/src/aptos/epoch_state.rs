// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use serde::{Deserialize, Serialize};

use super::validator_verifier::ValidatorVerifier;

/// EpochState represents a trusted validator set to validate messages from the specific epoch,
/// it could be updated with EpochChangeProof.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpochState {
    pub epoch: u64,
    pub verifier: ValidatorVerifier,
}

// this is required by structured log
impl fmt::Debug for EpochState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
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
