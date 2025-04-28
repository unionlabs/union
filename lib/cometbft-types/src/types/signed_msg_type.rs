use serde::{Deserialize, Serialize};
use unionlabs::errors::UnknownEnumVariant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub enum SignedMsgType {
    Unknown = 0,
    Prevote = 1,
    Precommit = 2,
    Proposal = 32,
}

impl TryFrom<i32> for SignedMsgType {
    type Error = UnknownEnumVariant<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Prevote),
            2 => Ok(Self::Precommit),
            32 => Ok(Self::Proposal),
            _ => Err(UnknownEnumVariant(value)),
        }
    }
}

impl From<SignedMsgType> for i32 {
    fn from(value: SignedMsgType) -> Self {
        value as i32
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::signed_msg_type::SignedMsgType;

    impl From<SignedMsgType> for protos::cometbft::types::v1::SignedMsgType {
        fn from(value: SignedMsgType) -> Self {
            match value {
                SignedMsgType::Unknown => Self::Unknown,
                SignedMsgType::Prevote => Self::Prevote,
                SignedMsgType::Precommit => Self::Precommit,
                SignedMsgType::Proposal => Self::Proposal,
            }
        }
    }

    impl From<protos::cometbft::types::v1::SignedMsgType> for SignedMsgType {
        fn from(value: protos::cometbft::types::v1::SignedMsgType) -> Self {
            match value {
                protos::cometbft::types::v1::SignedMsgType::Unknown => Self::Unknown,
                protos::cometbft::types::v1::SignedMsgType::Prevote => Self::Prevote,
                protos::cometbft::types::v1::SignedMsgType::Precommit => Self::Precommit,
                protos::cometbft::types::v1::SignedMsgType::Proposal => Self::Proposal,
            }
        }
    }
}
