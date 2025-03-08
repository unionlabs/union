use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    aptos::{block_info::BlockInfo, signature::AggregateSignature},
    primitives::{encoding::HexUnprefixed, H256},
};

/// Wrapper to support future upgrades, this is the data being persisted.
#[model(no_serde)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum LedgerInfoWithSignatures {
    V0(LedgerInfoWithV0),
}

impl LedgerInfoWithSignatures {
    #[must_use]
    pub fn ledger_info(&self) -> &LedgerInfo {
        let Self::V0(ledger_info) = self;
        &ledger_info.ledger_info
    }

    #[must_use]
    pub fn signatures(&self) -> &AggregateSignature {
        let Self::V0(ledger_info) = self;
        &ledger_info.signatures
    }
}

/// The validator node returns this structure which includes signatures
/// from validators that confirm the state.  The client needs to only pass back
/// the `LedgerInfo` element since the validator node doesn't need to know the signatures
/// again when the client performs a query, those are only there for the client
/// to be able to verify the state
#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LedgerInfoWithV0 {
    pub ledger_info: LedgerInfo,
    /// Aggregated BLS signature of all the validators that signed the message. The bitmask in the
    /// aggregated signature can be used to find out the individual validators signing the message
    pub signatures: AggregateSignature,
}

#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LedgerInfo {
    pub commit_info: BlockInfo,

    /// Hash of consensus specific data that is opaque to all parts of the system other than
    /// consensus.
    pub consensus_data_hash: H256<HexUnprefixed>,
}
