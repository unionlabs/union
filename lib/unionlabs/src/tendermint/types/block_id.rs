use serde::{Deserialize, Serialize};

use crate::{
    errors::{InvalidLength, MissingField},
    hash::H256,
    tendermint::types::part_set_header::PartSetHeader,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct BlockId {
    pub hash: H256,
    pub part_set_header: PartSetHeader,
}

#[derive(Debug)]
pub enum TryFromBlockIdError {
    MissingField(MissingField),
    Hash(InvalidLength),
    PartSetHeader(TryFromProtoErrorOf<PartSetHeader>),
}

impl TryFrom<protos::tendermint::types::BlockId> for BlockId {
    type Error = TryFromBlockIdError;

    fn try_from(value: protos::tendermint::types::BlockId) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value.hash.try_into().map_err(TryFromBlockIdError::Hash)?,
            part_set_header: value
                .part_set_header
                .ok_or(TryFromBlockIdError::MissingField(MissingField(
                    "part_set_header",
                )))?
                .try_into()
                .map_err(TryFromBlockIdError::PartSetHeader)?,
        })
    }
}

impl From<BlockId> for protos::tendermint::types::BlockId {
    fn from(value: BlockId) -> Self {
        Self {
            hash: value.hash.into(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}

impl Proto for BlockId {
    type Proto = protos::tendermint::types::BlockId;
}

impl TypeUrl for protos::tendermint::types::BlockId {
    const TYPE_URL: &'static str = "/tendermint.types.BlockId";
}

#[test]
#[cfg(test)]
fn proto_roundtrip() {
    crate::test_utils::assert_proto_roundtrip(&BlockId {
        hash: [1; 32].into(),
        part_set_header: PartSetHeader {
            total: 1,
            hash: [2; 32].into(),
        },
    });
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for BlockId {
    type EthAbi = contracts::glue::TendermintTypesBlockIDData;
}

#[cfg(feature = "ethabi")]
impl From<BlockId> for contracts::glue::TendermintTypesBlockIDData {
    fn from(value: BlockId) -> Self {
        Self {
            hash: value.hash.into_bytes().into(),
            part_set_header: value.part_set_header.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiBlockIdError {
    Hash(crate::errors::InvalidLength),
    PartSetHeader(crate::TryFromEthAbiErrorOf<PartSetHeader>),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesBlockIDData> for BlockId {
    type Error = TryFromEthAbiBlockIdError;

    fn try_from(value: contracts::glue::TendermintTypesBlockIDData) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value
                .hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiBlockIdError::Hash)?,
            part_set_header: value
                .part_set_header
                .try_into()
                .map_err(TryFromEthAbiBlockIdError::PartSetHeader)?,
        })
    }
}
