use macros::model;

use crate::{cosmos::ics23::hash_op::HashOp, ensure, primitives::Bytes};

#[model(proto(raw(protos::cosmos::ics23::v1::InnerOp), into, from))]
pub struct InnerOp {
    pub prefix: Bytes,
    pub suffix: Bytes,
}

const EXPECTED_HASH_OP: HashOp = HashOp::Sha256;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromInnerOpError {
    #[error("unable to decode cosmos::ics23::InnerOp")]
    Cosmos(#[from] crate::cosmos::ics23::inner_op::TryFromInnerOpError),
    #[error("hash must be {EXPECTED_HASH_OP}, found {0}")]
    InvalidHash(HashOp),
}

impl TryFrom<protos::cosmos::ics23::v1::InnerOp> for InnerOp {
    type Error = TryFromInnerOpError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerOp) -> Result<Self, TryFromInnerOpError> {
        let value = crate::cosmos::ics23::inner_op::InnerOp::try_from(value)?;

        ensure(
            value.hash == EXPECTED_HASH_OP,
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
            hash: EXPECTED_HASH_OP,
            prefix: value.prefix,
            suffix: value.suffix,
        }
        .into()
    }
}
