use serde::{Deserialize, Serialize};

/// EvidenceParams determine the validity of evidences of Byzantine behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceParams {
    /// Maximum age of evidence, in blocks.
    ///
    /// The recommended formula for calculating it is max_age_duration / {average
    /// block time}.
    // REVIEW: Bounded?
    #[serde(with = "::serde_utils::string")]
    pub max_age_num_blocks: i64,
    /// Maximum age of evidence, in time.
    ///
    /// The recommended value of is should correspond to the application's
    /// "unbonding period" or other similar mechanism for handling
    /// Nothing-At-Stake attacks.
    /// See: <https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed.>
    #[serde(with = "::serde_utils::string")]
    pub max_age_duration: i64,
    /// Maximum size in bytes of evidence allowed to be included in a block.
    ///
    /// It should fall comfortably under the maximum size of a block.
    // REVIEW: Bounded?
    #[serde(with = "::serde_utils::string")]
    pub max_bytes: i64,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
