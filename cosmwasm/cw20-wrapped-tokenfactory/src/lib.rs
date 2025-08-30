pub mod allowances;
pub mod contract;
pub mod enumerable;
pub mod error;
pub mod msg;
pub mod state;

use cosmwasm_std::Env;

pub use crate::error::ContractError;

pub fn self_tf_denom(env: &Env) -> String {
    format!("factory/{}/", env.contract.address)
}
