// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::ledger_info::{LedgerInfoWithSignatures, TryFromLedgerInfoWithSignatures};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
/// epoch changes from the first LedgerInfo's epoch.
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}

impl From<EpochChangeProof> for protos::union::ibc::lightclients::movement::v1::EpochChangeProof {
    fn from(value: EpochChangeProof) -> Self {
        Self {
            ledger_info_with_sigs: value
                .ledger_info_with_sigs
                .into_iter()
                .map(Into::into)
                .collect(),
            more: value.more,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TryFromEpochChangeProof {
    #[error("invalid ledger info with sigs")]
    LedgerInfoWithSigs(#[from] TryFromLedgerInfoWithSignatures),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::EpochChangeProof>
    for EpochChangeProof
{
    type Error = TryFromEpochChangeProof;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::EpochChangeProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            ledger_info_with_sigs: value
                .ledger_info_with_sigs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            more: value.more,
        })
    }
}
