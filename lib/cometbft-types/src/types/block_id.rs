use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

use crate::types::part_set_header::PartSetHeader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlockId {
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256>,
    pub part_set_header: PartSetHeader,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{ExpectedLength, InvalidLength, MissingField},
        hash::H256,
        impl_proto_via_try_from_into, required,
    };

    use crate::types::{block_id::BlockId, part_set_header};

    impl_proto_via_try_from_into!(BlockId => protos::tendermint::types::BlockId);

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid hash")]
        Hash(#[source] InvalidLength),
        #[error("invalid part set header")]
        PartSetHeader(#[from] part_set_header::proto::Error),
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

    pub(crate) fn maybe_empty_h256(value: &[u8]) -> Result<Option<H256>, InvalidLength> {
        Ok(if value.is_empty() {
            None
        } else {
            Some(
                value
                    .try_into()
                    .map_err(|err: InvalidLength| InvalidLength {
                        expected: ExpectedLength::Either(0, 32),
                        found: err.found,
                    })?,
            )
        })
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
