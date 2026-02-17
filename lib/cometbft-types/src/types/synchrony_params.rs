use serde::{Serialize, Deserialize};

/// SynchronyParams determine the validity of block timestamps.
///
/// These parameters are part of the Proposer-Based Timestamps (PBTS) algorithm.
/// For more information on the relationship of the synchrony parameters to
/// block timestamps validity, refer to the PBTS specification:
/// <https://github.com/tendermint/spec/blob/master/spec/consensus/proposer-based-timestamp/README.md>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynchronyParams {
    /// Bound for how skewed a proposer's clock may be from any validator on the
    /// network while still producing valid proposals.
    #[serde(with = "::serde_utils::string")]
    pub precision: i64,
    /// Bound for how long a proposal message may take to reach all validators on
    /// a network and still be considered valid.
    #[serde(with = "::serde_utils::string")]
    pub message_delay: i64,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
