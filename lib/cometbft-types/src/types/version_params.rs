use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionParams {
    /// The ABCI application version.
    ///
    /// It was named app_version in CometBFT 0.34.
    #[serde(with = "::serde_utils::string")]
    pub app: u64,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
