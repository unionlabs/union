use macros::model;

use crate::{
    errors::{required, MissingField},
    union::ics23::inner_op::{InnerOp, TryFromInnerOpError},
};

#[model(proto(raw(protos::union::ics23::v1::ExistenceProof), into, from))]
pub struct ExistenceProof {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub value: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub leaf_prefix: Vec<u8>,
    pub path: Vec<InnerOp>,
}

#[derive(Debug)]
pub enum TryFromExistenceProofError {
    MissingField(MissingField),
    Leaf(TryFromLeafOpError),
    Path(TryFromInnerOpError),
}

impl TryFrom<protos::union::ics23::v1::ExistenceProof> for ExistenceProof {
    type Error = TryFromExistenceProofError;

    fn try_from(value: protos::union::ics23::v1::ExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            value: value.value,
            leaf_prefix: value.leaf_prefix,
            path: value
                .path
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromExistenceProofError::Path)?,
        })
    }
}

impl From<ExistenceProof> for protos::union::ics23::v1::ExistenceProof {
    fn from(value: ExistenceProof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            leaf: Some(value.leaf.into()),
            path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

// #[cfg(feature = "ethabi")]
// impl From<ExistenceProof> for contracts::glue::CosmosIcs23V1ExistenceProofData {
//     fn from(value: ExistenceProof) -> Self {
//         Self {
//             key: value.key.into(),
//             value: value.value.into(),
//             leaf: value.leaf.into(),
//             path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
//         }
//     }
// }
