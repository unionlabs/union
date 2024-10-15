use macros::model;

use crate::{bounded::BoundedI32, cosmos::ics23::leaf_op::LeafOp};

#[model(proto(raw(protos::cosmos::ics23::v1::CompressedExistenceProof), into, from))]
pub struct CompressedExistenceProof {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub value: Vec<u8>,
    pub leaf: LeafOp,
    // these are indexes into an array, hence non-negative
    pub path: Vec<BoundedI32<0, { i32::MAX }>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        bounded::BoundedIntError,
        cosmos::ics23::{
            compressed_existence_proof::CompressedExistenceProof,
            leaf_op::proto::TryFromLeafOpError,
        },
        errors::{required, MissingField},
    };

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromCompressedExistenceProofError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid leaf")]
        Leaf(#[from] TryFromLeafOpError),
        #[error("invalid path")]
        Path(#[source] BoundedIntError<i32>),
    }

    impl TryFrom<protos::cosmos::ics23::v1::CompressedExistenceProof> for CompressedExistenceProof {
        type Error = TryFromCompressedExistenceProofError;

        fn try_from(
            value: protos::cosmos::ics23::v1::CompressedExistenceProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                key: value.key,
                value: value.value,
                leaf: required!(value.leaf)?
                    .try_into()
                    .map_err(TryFromCompressedExistenceProofError::Leaf)?,
                path: value
                    .path
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(TryFromCompressedExistenceProofError::Path)?,
            })
        }
    }

    impl From<CompressedExistenceProof> for protos::cosmos::ics23::v1::CompressedExistenceProof {
        fn from(value: CompressedExistenceProof) -> Self {
            Self {
                key: value.key,
                value: value.value,
                leaf: Some(value.leaf.into()),
                path: value.path.into_iter().map(Into::into).collect(),
            }
        }
    }
}
