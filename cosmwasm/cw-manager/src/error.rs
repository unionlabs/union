use cosmwasm_std::StdError;
use frissitheto::UpgradeError;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("migration error")]
    Migrate(#[from] UpgradeError),
}
