use crate::{
    errors::{InvalidLength, MissingField},
    ethereum::H256,
    tendermint::types::part_set_header::PartSetHeader,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq)]
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
    crate::assert_proto_roundtrip(&BlockId {
        hash: [1; 32].into(),
        part_set_header: PartSetHeader {
            total: 1,
            hash: [2; 32].into(),
        },
    });
}
