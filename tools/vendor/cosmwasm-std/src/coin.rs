use core::{fmt, str::FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{errors::CoinFromStrError, math::Uint128};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, JsonSchema)]
pub struct Coin {
    pub denom: String,
    pub amount: Uint128,
}

impl Coin {
    pub fn new(amount: u128, denom: impl Into<String>) -> Self {
        Coin {
            amount: Uint128::new(amount),
            denom: denom.into(),
        }
    }
}

impl fmt::Debug for Coin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Coin {{ {} \"{}\" }}", self.amount, self.denom)
    }
}

impl FromStr for Coin {
    type Err = CoinFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s
            .find(|c: char| !c.is_ascii_digit())
            .ok_or(CoinFromStrError::MissingDenom)?;
        let (amount, denom) = s.split_at(pos);

        if amount.is_empty() {
            return Err(CoinFromStrError::MissingAmount);
        }

        Ok(Coin {
            amount: amount.parse::<u128>()?.into(),
            denom: denom.to_string(),
        })
    }
}

impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We use the formatting without a space between amount and denom,
        // which is common in the Cosmos SDK ecosystem:
        // https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/types/coin.go#L643-L645
        // For communication to end users, Coin needs to transformed anways (e.g. convert integer uatom to decimal ATOM).
        write!(f, "{}{}", self.amount, self.denom)
    }
}

/// A shortcut constructor for a set of one denomination of coins
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, CosmosMsg, Response, SubMsg};
/// # use cosmwasm_std::testing::{mock_env, mock_info};
/// # let env = mock_env();
/// # let info = mock_info("sender", &[]);
/// let tip = coins(123, "ucosm");
///
/// let mut response: Response = Default::default();
/// response.messages = vec![SubMsg::new(BankMsg::Send {
///   to_address: info.sender.into(),
///   amount: tip,
/// })];
/// ```
pub fn coins(amount: u128, denom: impl Into<String>) -> Vec<Coin> {
    vec![coin(amount, denom)]
}

/// A shorthand constructor for Coin
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::{coin, BankMsg, CosmosMsg, Response, SubMsg};
/// # use cosmwasm_std::testing::{mock_env, mock_info};
/// # let env = mock_env();
/// # let info = mock_info("sender", &[]);
/// let tip = vec![
///     coin(123, "ucosm"),
///     coin(24, "ustake"),
/// ];
///
/// let mut response: Response = Default::default();
/// response.messages = vec![SubMsg::new(BankMsg::Send {
///     to_address: info.sender.into(),
///     amount: tip,
/// })];
/// ```
pub fn coin(amount: u128, denom: impl Into<String>) -> Coin {
    Coin::new(amount, denom)
}

/// has_coins returns true if the list of coins has at least the required amount
pub fn has_coins(coins: &[Coin], required: &Coin) -> bool {
    coins
        .iter()
        .find(|c| c.denom == required.denom)
        .map(|m| m.amount >= required.amount)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_implements_display() {
        let a = Coin {
            amount: Uint128::new(123),
            denom: "ucosm".to_string(),
        };

        let embedded = format!("Amount: {a}");
        assert_eq!(embedded, "Amount: 123ucosm");
        assert_eq!(a.to_string(), "123ucosm");
    }

    #[test]
    fn coin_works() {
        let a = coin(123, "ucosm");
        assert_eq!(
            a,
            Coin {
                amount: Uint128::new(123),
                denom: "ucosm".to_string()
            }
        );

        let zero = coin(0, "ucosm");
        assert_eq!(
            zero,
            Coin {
                amount: Uint128::new(0),
                denom: "ucosm".to_string()
            }
        );

        let string_denom = coin(42, String::from("ucosm"));
        assert_eq!(
            string_denom,
            Coin {
                amount: Uint128::new(42),
                denom: "ucosm".to_string()
            }
        );
    }

    #[test]
    fn coins_works() {
        let a = coins(123, "ucosm");
        assert_eq!(
            a,
            vec![Coin {
                amount: Uint128::new(123),
                denom: "ucosm".to_string()
            }]
        );

        let zero = coins(0, "ucosm");
        assert_eq!(
            zero,
            vec![Coin {
                amount: Uint128::new(0),
                denom: "ucosm".to_string()
            }]
        );

        let string_denom = coins(42, String::from("ucosm"));
        assert_eq!(
            string_denom,
            vec![Coin {
                amount: Uint128::new(42),
                denom: "ucosm".to_string()
            }]
        );
    }

    #[test]
    fn has_coins_matches() {
        let wallet = vec![coin(12345, "ETH"), coin(555, "BTC")];

        // less than same type
        assert!(has_coins(&wallet, &coin(777, "ETH")));
    }

    #[test]
    fn parse_coin() {
        let expected = Coin::new(123, "ucosm");
        assert_eq!("123ucosm".parse::<Coin>().unwrap(), expected);
        // leading zeroes should be ignored
        assert_eq!("00123ucosm".parse::<Coin>().unwrap(), expected);
        // 0 amount parses correctly
        assert_eq!("0ucosm".parse::<Coin>().unwrap(), Coin::new(0, "ucosm"));
        // ibc denom should work
        let ibc_str = "11111ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2";
        let ibc_coin = Coin::new(
            11111,
            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2",
        );
        assert_eq!(ibc_str.parse::<Coin>().unwrap(), ibc_coin);

        // error cases
        assert_eq!(
            Coin::from_str("123").unwrap_err(),
            CoinFromStrError::MissingDenom
        );
        assert_eq!(
            Coin::from_str("ucosm").unwrap_err(), // no amount
            CoinFromStrError::MissingAmount
        );
        assert_eq!(
            Coin::from_str("-123ucosm").unwrap_err(), // negative amount
            CoinFromStrError::MissingAmount
        );
        assert_eq!(
            Coin::from_str("").unwrap_err(), // empty input
            CoinFromStrError::MissingDenom
        );
        assert_eq!(
            Coin::from_str(" 1ucosm").unwrap_err(), // unsupported whitespace
            CoinFromStrError::MissingAmount
        );
        assert_eq!(
            Coin::from_str("�1ucosm").unwrap_err(), // other broken data
            CoinFromStrError::MissingAmount
        );
        assert_eq!(
            Coin::from_str("340282366920938463463374607431768211456ucosm")
                .unwrap_err()
                .to_string(),
            "Invalid amount: number too large to fit in target type"
        );
    }

    #[test]
    fn debug_coin() {
        let coin = Coin::new(123, "ucosm");
        assert_eq!(format!("{coin:?}"), r#"Coin { 123 "ucosm" }"#);
    }
}
