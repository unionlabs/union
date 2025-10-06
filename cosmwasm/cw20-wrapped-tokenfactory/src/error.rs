use cosmwasm_std::StdError;
use frissitheto::UpgradeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("Invalid expiration value")]
    InvalidExpiration {},

    #[error("No allowance for this account")]
    NoAllowance {},
}
