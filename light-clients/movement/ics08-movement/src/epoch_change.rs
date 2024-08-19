// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::state_proof::LedgerInfoWithSignatures;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
/// epoch changes from the first LedgerInfo's epoch.
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}
