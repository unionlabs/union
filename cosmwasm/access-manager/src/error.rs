use access_manager_types::manager::error::AccessManagerError;
use cosmwasm_std::StdError;
use frissitheto::{InitStateVersionError, UpgradeError};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    InitStateVersion(#[from] InitStateVersionError),

    #[error(transparent)]
    AccessManager(#[from] AccessManagerError),
}
