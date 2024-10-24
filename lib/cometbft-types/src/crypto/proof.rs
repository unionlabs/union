use serde::{Deserialize, Serialize};
use unionlabs::{bounded::BoundedI64, hash::H256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    pub total: BoundedI64<0, { i64::MAX }>,
    pub index: BoundedI64<0, { i64::MAX }>,
    pub leaf_hash: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    pub aunts: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{bounded::BoundedIntError, errors::InvalidLength};

    use crate::crypto::proof::Proof;

    impl From<Proof> for protos::tendermint::crypto::Proof {
        fn from(value: Proof) -> Self {
            Self {
                total: value.total.into(),
                index: value.index.into(),
                leaf_hash: value.leaf_hash.into(),
                aunts: value.aunts,
            }
        }
    }

    impl TryFrom<protos::tendermint::crypto::Proof> for Proof {
        type Error = Error;

        fn try_from(value: protos::tendermint::crypto::Proof) -> Result<Self, Self::Error> {
            Ok(Self {
                total: value.total.try_into().map_err(Error::Total)?,
                index: value.index.try_into().map_err(Error::Index)?,
                leaf_hash: value.leaf_hash.try_into()?,
                aunts: value.aunts,
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
