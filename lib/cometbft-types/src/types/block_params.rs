use serde::{Deserialize, Serialize};
use unionlabs::bounded::BoundedI64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockParams {
    /// Maximum size of a block, in bytes.
    ///
    /// Must be greater or equal to -1 and cannot be greater than the hard-coded
    /// maximum block size, which is 100MB.
    ///
    /// If set to -1, the limit is the hard-coded maximum block size.
    ///
    /// SEE: <https://github.com/cometbft/cometbft/blob/a6184c7d834de3d90552aded5b3c63d4b1b4be62/types/params.go#L17>
    #[serde(with = "::serde_utils::string")]
    pub max_bytes: BoundedI64<-1, 104857600>,
    /// Maximum gas wanted by transactions included in a block.
    ///
    /// Must be greater or equal to -1. If set to -1, no limit is enforced.
    #[serde(with = "::serde_utils::string")]
    pub max_gas: BoundedI64<-1>,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
