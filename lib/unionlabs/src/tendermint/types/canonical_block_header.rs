use macros::proto;
use serde::{Deserialize, Serialize};

use crate::hash::H256;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::tendermint::types::CanonicalPartSetHeader, from)]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256,
}

impl From<CanonicalPartSetHeader> for protos::tendermint::types::CanonicalPartSetHeader {
    fn from(value: CanonicalPartSetHeader) -> Self {
        Self {
            hash: value.hash.into(),
            total: value.total,
        }
    }
}
