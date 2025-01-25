use cosmwasm_std::{StdError, Uint128};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("only the admin can execute")]
    OnlyAdmin,

    #[error("missing funds for denom {denom} with amount {amount}")]
    MissingFunds { denom: String, amount: Uint128 },
}
