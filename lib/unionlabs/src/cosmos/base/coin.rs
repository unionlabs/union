use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coin {
    // REVIEW: Is this bounded?
    pub denom: String,
    // NOTE: According to the proto docs: "Exists in range from -(2^256 - 1) to 2^256 - 1"
    // If we ever have a use for amounts outside the range, you probably have other issues
    #[serde(with = "::serde_utils::string")]
    pub amount: u128,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::num::ParseIntError;

    use super::Coin;
    use crate::impl_proto_via_try_from_into;

    impl_proto_via_try_from_into!(Coin => protos::cosmos::base::v1beta1::Coin);

    impl From<Coin> for protos::cosmos::base::v1beta1::Coin {
        fn from(value: Coin) -> Self {
            Self {
                denom: value.denom,
                amount: value.amount.to_string(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid amount")]
        Amount(#[from] ParseIntError),
    }

    impl TryFrom<protos::cosmos::base::v1beta1::Coin> for Coin {
        type Error = Error;

        fn try_from(value: protos::cosmos::base::v1beta1::Coin) -> Result<Self, Self::Error> {
            Ok(Self {
                denom: value.denom,
                amount: value.amount.parse()?,
            })
        }
    }
}
