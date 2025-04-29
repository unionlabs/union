pub mod contract;
pub mod msg;
pub mod state;
pub mod types;
use alloy_primitives::U256;
use cosmwasm_std::StdError;
use frissitheto::UpgradeError;
use thiserror::Error;
use unionlabs::primitives::Bytes;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("migration error")]
    Migrate(#[from] UpgradeError),
    #[error(transparent)]
    Alloy(#[from] alloy_sol_types::Error),
    #[error("a market maker is not allowed to fill this call")]
    MarketMakerNotAllowed,
    #[error("invalid contract address: {address}")]
    InvalidContractAddress { address: Bytes },
    #[error("invalid beneficiary address: {address}")]
    InvalidBeneficiaryAddress { address: Bytes },
    #[error("invalid denom: {denom}")]
    InvalidDenom { denom: Bytes },
    #[error("cosmos does not support amount > u128: {amount}")]
    InvalidAmount { amount: U256 },
    #[error("reentrancy is forbidden")]
    ForbiddenReentrancy,
}
