use cosmwasm_std::{Addr, StdError, Uint128};
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

    #[error("the token ownership can only be changed by this contract or the minter operator since this token is owner by this contract")]
    UnauthorizedWhenSelfOwned,

    #[error("the token is owned by {owner} and, {sender} cannot change the ownership")]
    UnauthorizedThirdParty { owner: Addr, sender: Addr },

    #[error("empty name or symbol in metadata")]
    EmptyNameOrSymbol,
}
