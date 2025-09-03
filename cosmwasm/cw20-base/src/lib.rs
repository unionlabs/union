/*!
This is a basic implementation of a cw20 contract. It implements
the [CW20 spec](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md) and is designed to
be deployed as is, or imported into other contracts to easily build
cw20-compatible tokens with custom logic.

Implements:

- [x] CW20 Base
- [x] Mintable extension
- [x] Allowances extension

For more information on this contract, please check out the
[README](https://github.com/CosmWasm/cw-plus/blob/main/contracts/cw20-base/README.md).
*/

pub mod allowances;
pub mod contract;
pub mod enumerable;
pub mod error;
pub mod msg;
pub mod state;

use cosmwasm_std::{Addr, Deps, StdError, StdResult, Uint128};
use cw20_ctx::Cw20Ctx;

pub use crate::error::ContractError;
use crate::state::TOKEN_INFO;

pub struct Cw20BaseCtx {}

impl Cw20Ctx for Cw20BaseCtx {
    fn check_can_mint(deps: Deps, addr: &Addr, amount: Uint128) -> StdResult<()> {
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
        } else if config
            .get_cap()
            .is_some_and(|limit| config.total_supply + amount > limit)
        {
            Err(StdError::generic_err(
                ContractError::CannotExceedCap {}.to_string(),
            ))
        } else {
            Ok(())
        }
    }
}
