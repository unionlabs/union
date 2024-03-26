use macros::model;

use crate::{cosmos::ics23::hash_op::HashOp, ensure};

#[model(proto(raw(protos::cosmos::ics23::v1::InnerOp), into, from))]
pub struct InnerOp {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub prefix: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub suffix: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromInnerOpError {
    #[error("unable to decode cosmos::ics23::InnerOp")]
    Cosmos(#[from] crate::cosmos::ics23::inner_op::TryFromInnerOpError),
    #[error("hashop must be Sha256, found {0}")]
    InvalidHash(HashOp),
}

impl TryFrom<protos::cosmos::ics23::v1::InnerOp> for InnerOp {
    type Error = TryFromInnerOpError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerOp) -> Result<Self, TryFromInnerOpError> {
        let value = crate::cosmos::ics23::inner_op::InnerOp::try_from(value)?;

        ensure(
            value.hash == HashOp::Sha256,
            TryFromInnerOpError::InvalidHash(value.hash),
        )?;

        Ok(Self {
            prefix: value.prefix,
            suffix: value.suffix,
        })
    }
}

impl From<InnerOp> for protos::cosmos::ics23::v1::InnerOp {
    fn from(value: InnerOp) -> Self {
        crate::cosmos::ics23::inner_op::InnerOp {
            hash: HashOp::Sha256,
            prefix: value.prefix,
            suffix: value.suffix,
        }
        .into()
    }
}

// #[cfg(feature = "ethabi")]
// impl From<InnerOp> for contracts::glue::unionIcs23V1InnerOpData {
//     fn from(value: InnerOp) -> Self {
//         Self {
//             hash: value.hash.into(),
//             prefix: value.prefix.into(),
//             suffix: value.suffix.into(),
//         }
//     }
// }
