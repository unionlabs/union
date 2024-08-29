// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

use macros::model;

use super::validator_verifier::{TryFromValidatorVerifierError, ValidatorVerifier};
use crate::errors::{required, MissingField};

/// `EpochState` represents a trusted validator set to validate messages from the specific epoch,
/// it could be updated with `EpochChangeProof`.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::EpochState),
    into,
    from
))]
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

impl From<EpochState> for protos::union::ibc::lightclients::movement::v1::EpochState {
    fn from(value: EpochState) -> Self {
        Self {
            epoch: value.epoch,
            verifier: Some(value.verifier.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
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
