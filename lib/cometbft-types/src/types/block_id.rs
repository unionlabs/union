use serde::{Deserialize, Serialize};
use unionlabs::primitives::{H256, encoding::HexUnprefixed};

use crate::types::part_set_header::PartSetHeader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct BlockId {
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    #[serde(with = "crate::serde::maybe_empty_h256")]
    pub hash: Option<H256<HexUnprefixed>>,
    #[serde(rename = "parts")]
    pub part_set_header: PartSetHeader,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        impl_proto_via_try_from_into, required,
    };

    use crate::{
        types::{block_id::BlockId, part_set_header},
        utils::maybe_empty_h256,
    };

    impl_proto_via_try_from_into!(BlockId => protos::cometbft::types::v1::BlockId);

    impl TryFrom<protos::cometbft::types::v1::BlockId> for BlockId {
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::BlockId) -> Result<Self, Self::Error> {
            Ok(Self {
                hash: maybe_empty_h256(&value.hash).map_err(Error::Hash)?,
                part_set_header: required!(value.part_set_header)?.try_into()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid hash")]
        Hash(#[source] InvalidLength),
        #[error("invalid part set header")]
        PartSetHeader(#[from] part_set_header::proto::Error),
    }

    impl From<BlockId> for protos::cometbft::types::v1::BlockId {
        fn from(value: BlockId) -> Self {
            Self {
                hash: value.hash.map(Into::into).unwrap_or_default(),
                part_set_header: Some(value.part_set_header.into()),
            }
        }
    }

    impl TryFrom<protos::tendermint::types::BlockId> for BlockId {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::BlockId) -> Result<Self, Self::Error> {
            Ok(Self {
                hash: maybe_empty_h256(&value.hash).map_err(Error::Hash)?,
                part_set_header: required!(value.part_set_header)?.try_into()?,
            })
        }
    }

    impl From<BlockId> for protos::tendermint::types::BlockId {
        fn from(value: BlockId) -> Self {
            Self {
                hash: value.hash.map(Into::into).unwrap_or_default(),
                part_set_header: Some(value.part_set_header.into()),
            }
        }
    }
}

#[test]
#[cfg(test)]
fn proto_roundtrip() {
    use unionlabs::test_utils::assert_proto_roundtrip;

    assert_proto_roundtrip(&BlockId {
        hash: Some([1; 32].into()),
        part_set_header: PartSetHeader {
            total: 1,
            hash: Some([2; 32].into()),
        },
    });

    assert_proto_roundtrip(&BlockId {
        hash: None,
        part_set_header: PartSetHeader {
            total: 1,
            hash: None,
        },
    });
}
