use serde::{Deserialize, Serialize};
use unionlabs::errors::UnknownEnumVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum SignedMsgType {
    Prevote = 1,
    Precommit = 2,
    Proposal = 32,
}

impl TryFrom<i32> for SignedMsgType {
    type Error = UnknownEnumVariant<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
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
