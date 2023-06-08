use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use cw20::Cw20Coin;
use std::convert::TryInto;

const CW20_DENOM_PREFIX: &str = "cw20:";

#[cw_serde]
pub enum Amount {
    Native(Coin),
    // FIXME? USe Cw20CoinVerified, and validate cw20 addresses
    Cw20(Cw20Coin),
}



impl Amount {
    pub fn from_parts(denom: String, amount: impl Into<Uint128>) -> Self {
        let amount = amount.into();
        match denom.strip_prefix(CW20_DENOM_PREFIX) {
            Some(address) => Amount::Cw20(Cw20Coin {
                address: address.to_string(),
                amount,
            }),
            None => Amount::Native(Coin { denom, amount }),
        }
    }

    pub fn cw20(amount: u128, addr: &str) -> Self {
        Amount::Cw20(Cw20Coin {
            address: addr.into(),
            amount: Uint128::new(amount),
        })
    }

    pub fn native(amount: u128, denom: &str) -> Self {
        Amount::Native(Coin {
            denom: denom.to_string(),
            amount: Uint128::new(amount),
        })
    }
}

impl Amount {
    pub fn denom(&self) -> String {
        match self {
            Amount::Native(c) => c.denom.clone(),
            Amount::Cw20(c) => format!("{}{}", CW20_DENOM_PREFIX, c.address.as_str()),
        }
    }

    pub fn amount(&self) -> Uint128 {
        match self {
            Amount::Native(c) => c.amount,
            Amount::Cw20(c) => c.amount,
        }
    }

    /// convert the amount into u64
    pub fn u64_amount(&self) -> Result<u64, ContractError> {
        Ok(self.amount().u128().try_into()?)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Amount::Native(c) => c.amount.is_zero(),
            Amount::Cw20(c) => c.amount.is_zero(),
        }
    }
}
