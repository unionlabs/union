use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    bytes::Bytes,
    hash::{hash_v2::Base64, H256},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    #[serde(with = "::serde_utils::string")]
    pub total: BoundedI64<0, { i64::MAX }>,
    #[serde(with = "::serde_utils::string")]
    pub index: BoundedI64<0, { i64::MAX }>,
    pub leaf_hash: H256<Base64>,
    pub aunts: Vec<Bytes<Base64>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{bounded::BoundedIntError, errors::InvalidLength};

    use crate::crypto::proof::Proof;

    impl From<Proof> for protos::cometbft::crypto::v1::Proof {
        fn from(value: Proof) -> Self {
            Self {
                total: value.total.into(),
                index: value.index.into(),
                leaf_hash: value.leaf_hash.into(),
                aunts: value.aunts.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<protos::cometbft::crypto::v1::Proof> for Proof {
        type Error = Error;

        fn try_from(value: protos::cometbft::crypto::v1::Proof) -> Result<Self, Self::Error> {
            Ok(Self {
                total: value.total.try_into().map_err(Error::Total)?,
                index: value.index.try_into().map_err(Error::Index)?,
                leaf_hash: value.leaf_hash.try_into()?,
                aunts: value.aunts.into_iter().map(Into::into).collect(),
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("invalid total")]
        Total(#[source] BoundedIntError<i64>),
        #[error("invalid index")]
        Index(#[source] BoundedIntError<i64>),
        #[error("invalid leaf hash")]
        LeafHash(#[from] InvalidLength),
    }
}
