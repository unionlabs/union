use macros::model;

use crate::hash::H256;

#[model(proto(raw(protos::cometbft::types::v1::CanonicalPartSetHeader), from))]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256,
}

impl From<CanonicalPartSetHeader> for protos::cometbft::types::v1::CanonicalPartSetHeader {
    fn from(value: CanonicalPartSetHeader) -> Self {
        Self {
            total: value.total,
            hash: value.hash.into(),
        }
    }
}
