use serde::{Deserialize, Serialize};

use crate::{hash::H256, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256,
}

impl Proto for CanonicalPartSetHeader {
    type Proto = protos::tendermint::types::CanonicalPartSetHeader;
}

impl TypeUrl for protos::tendermint::types::CanonicalPartSetHeader {
    const TYPE_URL: &'static str = "/tendermint.types.CanonicalPartSetHeader";
}

impl From<CanonicalPartSetHeader> for protos::tendermint::types::CanonicalPartSetHeader {
    fn from(value: CanonicalPartSetHeader) -> Self {
        Self {
            hash: value.hash.into(),
            total: value.total,
        }
    }
}
