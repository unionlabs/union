use cosmwasm_std::{Addr, StdError};
use frissitheto::UpgradeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("migration error")]
    Migrate(#[from] UpgradeError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No allowance for this account ({spender} attempting to spend for {owner})")]
    NoAllowance { owner: Addr, spender: Addr },

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Invalid expiration value")]
    InvalidExpiration {},

    #[error("Duplicate initial balance addresses")]
    DuplicateInitialBalanceAddresses {},
}
