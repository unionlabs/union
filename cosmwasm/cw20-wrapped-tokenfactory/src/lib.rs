pub mod allowances;
pub mod contract;
pub mod enumerable;
pub mod error;
pub mod msg;
pub mod state;

use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Uint128};
use cw20_ctx::Cw20Ctx;

pub use crate::error::ContractError;
use crate::state::TOKEN_INFO;

pub fn self_tf_denom(env: &Env) -> String {
    format!("factory/{}/", env.contract.address)
}

pub enum Cw20WrappedTokenfactoryCtx {}

impl Cw20Ctx for Cw20WrappedTokenfactoryCtx {
    fn check_can_mint(deps: Deps, addr: &Addr, amount: Uint128) -> StdResult<()> {
        // no cap on the tf wrapped tokens
        let _ = amount;

        let config = TOKEN_INFO.load(deps.storage)?;

        if config
            .mint
            .as_ref()
            .ok_or_else(|| StdError::generic_err(ContractError::Unauthorized {}.to_string()))?
            .minter
            != addr
        {
            Err(StdError::generic_err(
                ContractError::Unauthorized {}.to_string(),
            ))
        } else {
            Ok(())
        }
    }
}
