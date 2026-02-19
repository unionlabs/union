use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciParams {
    /// vote_extensions_enable_height has been deprecated.
    /// Instead, use FeatureParams.vote_extensions_enable_height.
    #[serde(with = "::serde_utils::string")]
    pub vote_extensions_enable_height: i64,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
