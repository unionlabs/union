use cosmwasm_std::OverflowError;

pub mod contract;
pub mod msg;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("funds overflow")]
    FundsOverflow(#[from] OverflowError),
    #[error("funds mismatch")]
    FundsMismatch,
}
