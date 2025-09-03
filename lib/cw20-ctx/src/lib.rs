use cosmwasm_std::{Addr, Deps, StdResult, Uint128};

pub trait Cw20Ctx {
    /// Ensure that the provided address is allowed to mint the specified amount of tokens.
    fn check_can_mint(deps: Deps, addr: &Addr, amount: Uint128) -> StdResult<()>;
}
