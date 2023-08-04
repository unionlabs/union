use crate::{errors::InvalidLength, ethereum::H256, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq)]
pub struct PartSetHeader {
    pub total: u32,
    pub hash: H256,
}

#[derive(Debug)]
pub enum TryFromPartSetHeaderError {
    Hash(InvalidLength),
}

impl TryFrom<protos::tendermint::types::PartSetHeader> for PartSetHeader {
    type Error = TryFromPartSetHeaderError;

    fn try_from(value: protos::tendermint::types::PartSetHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            total: value.total,
            hash: value
                .hash
                .try_into()
                .map_err(TryFromPartSetHeaderError::Hash)?,
        })
    }
}

impl From<PartSetHeader> for protos::tendermint::types::PartSetHeader {
    fn from(value: PartSetHeader) -> Self {
        Self {
            total: value.total,
            hash: value.hash.into(),
        }
    }
}

impl Proto for PartSetHeader {
    type Proto = protos::tendermint::types::PartSetHeader;
}

impl TypeUrl for protos::tendermint::types::PartSetHeader {
    const TYPE_URL: &'static str = "/tendermint.types.PartSetHeader";
}
