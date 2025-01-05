// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use macros::model;

use super::ledger_info::LedgerInfoWithSignatures;

/// A vector of `LedgerInfo` with contiguous increasing epoch numbers to prove a sequence of
/// epoch changes from the first `LedgerInfo`'s epoch.
#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}
