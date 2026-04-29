use serde::{Deserialize, Serialize};
use unionlabs::bounded::BoundedI64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureParams {
    /// Height during which vote extensions will be enabled.
    ///
    /// A value of 0 means vote extensions are disabled. A value > 0 denotes
    /// the height at which vote extensions will be (or have been) enabled.
    ///
    /// During the specified height, and for all subsequent heights, precommit
    /// messages that do not contain valid extension data will be considered
    /// invalid. Prior to this height, or when this height is set to 0, vote
    /// extensions will not be used or accepted by validators on the network.
    ///
    /// Once enabled, vote extensions will be created by the application in
    /// ExtendVote, validated by the application in VerifyVoteExtension, and
    /// used by the application in PrepareProposal, when proposing the next block.
    ///
    /// Cannot be set to heights lower or equal to the current blockchain height.
    #[serde(
        with = "::serde_utils::string_opt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vote_extensions_enable_height: Option<BoundedI64<0>>,

    /// Height at which Proposer-Based Timestamps (PBTS) will be enabled.
    ///
    /// A value of 0 means PBTS is disabled. A value > 0 denotes the height at
    /// which PBTS will be (or has been) enabled.
    ///
    /// From the specified height, and for all subsequent heights, the PBTS
    /// algorithm will be used to produce and validate block timestamps. Prior to
    /// this height, or when this height is set to 0, the legacy BFT Time
    /// algorithm is used to produce and validate timestamps.
    ///
    /// Cannot be set to heights lower or equal to the current blockchain height.
    #[serde(
        with = "::serde_utils::string_opt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pbts_enable_height: Option<BoundedI64<0>>,

    #[serde(
        with = "::serde_utils::string_opt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sbt_enable_height: Option<BoundedI64<0>>,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
