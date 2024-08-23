// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use serde::{Deserialize, Serialize};

use super::validator_verifier::{TryFromValidatorVerifierError, ValidatorVerifier};
use crate::errors::{required, MissingField};

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

impl From<EpochState> for protos::union::ibc::lightclients::movement::v1::EpochState {
    fn from(value: EpochState) -> Self {
        Self {
            epoch: value.epoch,
            verifier: Some(value.verifier.into()),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum TryFromEpochStateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid verifier: {0}")]
    Verifier(#[from] TryFromValidatorVerifierError),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::EpochState> for EpochState {
    type Error = TryFromEpochStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::EpochState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: value.epoch,
            verifier: required!(value.verifier)?.try_into()?,
        })
    }
}
