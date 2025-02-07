use cosmwasm_std::{Deps, Order, StdResult};
use cw20::{
    AllAccountsResponse, AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceInfo,
    SpenderAllowanceInfo,
};
use cw_storage_plus::Bound;

use crate::state::{ALLOWANCES, ALLOWANCES_SPENDER, BALANCES};

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_owner_allowances(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAllowancesResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

    let allowances = ALLOWANCES
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, allow)| AllowanceInfo {
                spender: addr.into(),
                allowance: allow.allowance,
                expires: allow.expires,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(AllAllowancesResponse { allowances })
}

pub fn query_spender_allowances(
    deps: Deps,
    spender: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllSpenderAllowancesResponse> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

    let allowances = ALLOWANCES_SPENDER
        .prefix(&spender_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, allow)| SpenderAllowanceInfo {
                owner: addr.into(),
                allowance: allow.allowance,
                expires: allow.expires,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(AllSpenderAllowancesResponse { allowances })
}

pub fn query_all_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAccountsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let accounts = BALANCES
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(Into::into))
        .collect::<StdResult<_>>()?;

    Ok(AllAccountsResponse { accounts })
}
