use serde::{Deserialize, Serialize};

use super::{block_info::BlockInfo, hash_value::HashValue, signature::AggregateSignature};

/// Wrapper around LedgerInfoWithScheme to support future upgrades, this is the data being persisted.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerInfoWithSignatures {
    V0(LedgerInfoWithV0),
}

/// The validator node returns this structure which includes signatures
/// from validators that confirm the state.  The client needs to only pass back
/// the LedgerInfo element since the validator node doesn't need to know the signatures
/// again when the client performs a query, those are only there for the client
/// to be able to verify the state
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfoWithV0 {
    pub ledger_info: LedgerInfo,
    /// Aggregated BLS signature of all the validators that signed the message. The bitmask in the
    /// aggregated signature can be used to find out the individual validators signing the message
    pub signatures: AggregateSignature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfo {
    pub commit_info: BlockInfo,

    /// Hash of consensus specific data that is opaque to all parts of the system other than
    /// consensus.
    pub consensus_data_hash: HashValue,
}
