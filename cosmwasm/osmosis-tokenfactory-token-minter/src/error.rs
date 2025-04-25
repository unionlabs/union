use cosmwasm_std::{StdError, Uint128};
use unionlabs::primitives::uint::FromDecStrErr;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("only the admin can execute")]
    OnlyAdmin,

    #[error("missing funds for denom {denom} with amount {amount}")]
    MissingFunds { denom: String, amount: Uint128 },

    #[error("invalid path: {0}")]
    InvalidPath(FromDecStrErr),

    #[error("invalid denom: {0}")]
    InvalidDenom(String),

    #[error("minter config is expected to be osmosis-tokenfactory")]
    InvalidMinterConfig,
}
