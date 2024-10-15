use macros::model;

use crate::cosmos::ics23::hash_op::HashOp;

#[model(
    proto(raw(protos::cosmos::ics23::v1::InnerOp), into, from),
    ethabi(raw(ibc_solidity::ics23::InnerOp), into, from)
)]
pub struct InnerOp {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub prefix: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub suffix: Vec<u8>,
}

const EXPECTED_HASH_OP: HashOp = HashOp::Sha256;

#[cfg(feature = "proto")]
pub mod proto {
    use super::EXPECTED_HASH_OP;
    use crate::{cosmos::ics23::hash_op::HashOp, ensure, union::ics23::inner_op::InnerOp};

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromInnerOpError {
        #[error("unable to decode cosmos::ics23::InnerOp")]
        Cosmos(#[from] crate::cosmos::ics23::inner_op::proto::TryFromInnerOpError),
        #[error("hash must be {}, found {0}", EXPECTED_HASH_OP)]
        InvalidHash(HashOp),
    }

    impl TryFrom<protos::cosmos::ics23::v1::InnerOp> for InnerOp {
        type Error = TryFromInnerOpError;

        fn try_from(
            value: protos::cosmos::ics23::v1::InnerOp,
        ) -> Result<Self, TryFromInnerOpError> {
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
}

#[cfg(feature = "ethabi")]
impl From<ibc_solidity::ics23::InnerOp> for InnerOp {
    fn from(value: ibc_solidity::ics23::InnerOp) -> Self {
        Self {
            prefix: value.prefix.to_vec(),
            suffix: value.suffix.to_vec(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<InnerOp> for ibc_solidity::ics23::InnerOp {
    fn from(value: InnerOp) -> Self {
        Self {
            prefix: value.prefix.into(),
            suffix: value.suffix.into(),
        }
    }
}
