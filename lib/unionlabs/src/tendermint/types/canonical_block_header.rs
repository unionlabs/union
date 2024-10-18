use macros::model;

use crate::hash::H256;

#[model(proto(raw(protos::tendermint::types::CanonicalPartSetHeader), from))]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256,
}

#[cfg(feature = "proto")]
impl From<CanonicalPartSetHeader> for protos::tendermint::types::CanonicalPartSetHeader {
    fn from(value: CanonicalPartSetHeader) -> Self {
        Self {
            hash: value.hash.into(),
            total: value.total,
        }
    }
}
