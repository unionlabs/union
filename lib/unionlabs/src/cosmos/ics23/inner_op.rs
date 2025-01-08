use macros::model;
use unionlabs_primitives::Bytes;

use crate::{cosmos::ics23::hash_op::HashOp, errors::UnknownEnumVariant};

#[model(proto(raw(protos::cosmos::ics23::v1::InnerOp), into, from))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct InnerOp {
    pub hash: HashOp,
    pub prefix: Bytes,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub suffix: Bytes,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromInnerOpError {
    #[error("error decoding hash")]
    Hash(UnknownEnumVariant<i32>),
}

impl TryFrom<protos::cosmos::ics23::v1::InnerOp> for InnerOp {
    type Error = TryFromInnerOpError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerOp) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value.hash.try_into().map_err(TryFromInnerOpError::Hash)?,
            prefix: value.prefix.into(),
            suffix: value.suffix.into(),
        })
    }
}

impl From<InnerOp> for protos::cosmos::ics23::v1::InnerOp {
    fn from(value: InnerOp) -> Self {
        Self {
            hash: value.hash.into(),
            prefix: value.prefix.into(),
            suffix: value.suffix.into(),
        }
    }
}
