use alloy::primitives::ruint::ParseError;
use cosmwasm_std::{StdError, Uint128};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("only the admin can execute")]
    OnlyAdmin,

    #[error("missing funds for denom {denom} with amount {amount}")]
    MissingFunds { denom: String, amount: Uint128 },

    #[error("{0:?}")]
    U256Parse(ParseError),

    #[error("invalid denom {0}")]
    InvalidDenom(String),

    #[error("minter config is expected to be cw20")]
    InvalidMinterConfig,
}
