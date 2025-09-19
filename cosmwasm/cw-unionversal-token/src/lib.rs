use cosmwasm_std::{Addr, Deps, StdResult, Uint128};
use cw20_base::Cw20BaseCtx;
use cw20_ctx::Cw20Ctx;
use cw20_wrapped_tokenfactory::Cw20WrappedTokenfactoryCtx;
use depolama::StorageExt;

use crate::state::{Cw20ImplType, Cw20Type, Minters};

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub enum CwUCtx {}

impl Cw20Ctx for CwUCtx {
    fn check_can_mint(deps: Deps, addr: &Addr, amount: Uint128) -> StdResult<()> {
        // first check any extra configured minters, then fall back to the underlying implementation
        if deps
            .storage
            .maybe_read_item::<Minters>()?
            .unwrap_or_default()
            .iter()
            .any(|s| s == addr.as_str())
        {
            Ok(())
        } else {
            match deps.storage.read_item::<Cw20Type>()? {
                Cw20ImplType::Base => Cw20BaseCtx::check_can_mint(deps, addr, amount)?,
                Cw20ImplType::Tokenfactory => {
                    Cw20WrappedTokenfactoryCtx::check_can_mint(deps, addr, amount)?
                }
            };

            Ok(())
        }
    }
}
