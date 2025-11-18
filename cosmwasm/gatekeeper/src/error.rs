use cosmwasm_std::StdError;
use frissitheto::UpgradeError;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    AccessManager(#[from] access_manager::error::ContractError),

    #[error(transparent)]
    Upgradable(#[from] upgradable::error::ContractError),
}
