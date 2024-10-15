// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use macros::model;

use crate::aptos::ledger_info::LedgerInfoWithSignatures;

/// A vector of `LedgerInfo` with contiguous increasing epoch numbers to prove a sequence of
/// epoch changes from the first `LedgerInfo`'s epoch.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::EpochChangeProof),
    into,
    from
))]
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::aptos::{
        epoch_change::EpochChangeProof, ledger_info::proto::TryFromLedgerInfoWithSignatures,
    };

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

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
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
}
