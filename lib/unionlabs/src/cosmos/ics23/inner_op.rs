use serde::{Deserialize, Serialize};

use crate::{cosmos::ics23::hash_op::HashOp, errors::UnknownEnumVariant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerOp {
    pub hash: HashOp,
    #[serde(with = "::serde_utils::hex_string")]
    pub prefix: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub suffix: Vec<u8>,
}

#[derive(Debug)]
pub enum TryFromInnerOpError {
    Hash(UnknownEnumVariant<i32>),
}

impl crate::Proto for InnerOp {
    type Proto = protos::cosmos::ics23::v1::InnerOp;
}

impl TryFrom<protos::cosmos::ics23::v1::InnerOp> for InnerOp {
    type Error = TryFromInnerOpError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerOp) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value.hash.try_into().map_err(TryFromInnerOpError::Hash)?,
            prefix: value.prefix,
            suffix: value.suffix,
        })
    }
}

impl From<InnerOp> for protos::cosmos::ics23::v1::InnerOp {
    fn from(value: InnerOp) -> Self {
        Self {
            hash: value.hash.into(),
            prefix: value.prefix,
            suffix: value.suffix,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<InnerOp> for contracts::glue::CosmosIcs23V1InnerOpData {
    fn from(value: InnerOp) -> Self {
        Self {
            hash: value.hash.into(),
            prefix: value.prefix.into(),
            suffix: value.suffix.into(),
        }
    }
}
