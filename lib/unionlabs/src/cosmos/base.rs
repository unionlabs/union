pub mod query;
use serde::{Deserialize, Serialize};

use crate::{Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    // NOTE: Exists in range from -(2^256 - 1) to 2^256 - 1
    pub amount: String,
}

impl Proto for Coin {
    type Proto = protos::cosmos::base::v1beta1::Coin;
}

impl TypeUrl for protos::cosmos::base::v1beta1::Coin {
    const TYPE_URL: &'static str = "/cosmos.base.v1beta1.Coin";
}

impl From<protos::cosmos::base::v1beta1::Coin> for Coin {
    fn from(value: protos::cosmos::base::v1beta1::Coin) -> Self {
        Self {
            denom: value.denom,
            amount: value.amount,
        }
    }
}

impl From<Coin> for protos::cosmos::base::v1beta1::Coin {
    fn from(value: Coin) -> Self {
        Self {
            denom: value.denom,
            amount: value.amount,
        }
    }
}
