use cosmwasm_std::StdError;
use frissitheto::UpgradeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("migration error: {0}")]
    Migrate(#[from] UpgradeError),

    #[error("only the owner can dispatch messages to be executed")]
    OnlyOwner,
}
