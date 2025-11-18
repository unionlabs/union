use cosmwasm_std::StdError;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("the operation failed because the contract is paused")]
    EnforcedPause,

    #[error("the operation failed because the contract is not paused")]
    ExpectedPause,
}
