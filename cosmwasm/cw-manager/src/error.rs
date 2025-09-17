use cosmwasm_std::StdError;
use frissitheto::UpgradeError;

use crate::types::RoleId;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("migration error")]
    Migrate(#[from] UpgradeError),

    #[error("role {0} is locked")]
    AccessManagerLockedRole(RoleId),
}
