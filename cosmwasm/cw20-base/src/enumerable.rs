use cosmwasm_std::{Deps, Order, StdResult};
use cw20::{
    AllAccountsResponse, AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceInfo,
    SpenderAllowanceInfo,
};

use crate::state::{ALLOWANCES, ALLOWANCES_SPENDER, BALANCES};
use cw_storage_plus::Bound;

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

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, DepsMut, Uint128};
    use cw20::{Cw20Coin, Expiration, TokenInfoResponse};

    use crate::contract::{execute, instantiate, query, query_token_info};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

    // this will set up the instantiation for other tests
    fn do_instantiate(mut deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
        let instantiate_msg = InstantiateMsg {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: addr.into(),
                amount,
            }],
            mint: None,
            marketing: None,
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        query_token_info(deps.as_ref()).unwrap()
    }

    #[test]
    fn query_all_owner_allowances_works() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let owner = deps.api.addr_make("owner").to_string();
        // these are in alphabetical order same than insert order
        let spender1 = deps.api.addr_make("earlier").to_string();
        let spender2 = deps.api.addr_make("later").to_string();

        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), &owner, Uint128::new(12340000));

        // no allowance to start
        let allowances = query_owner_allowances(deps.as_ref(), owner.clone(), None, None).unwrap();
        assert_eq!(allowances.allowances, vec![]);

        // set allowance with height expiration
        let allow1 = Uint128::new(7777);
        let expires = Expiration::AtHeight(123_456);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender1.clone(),
            amount: allow1,
            expires: Some(expires),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // set allowance with no expiration
        let allow2 = Uint128::new(54321);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender2.clone(),
            amount: allow2,
            expires: None,
        };
        execute(deps.as_mut(), env, info, msg).unwrap();

        // query list gets 2
        let allowances = query_owner_allowances(deps.as_ref(), owner.clone(), None, None).unwrap();
        assert_eq!(allowances.allowances.len(), 2);

        // first one is spender1 (order of CanonicalAddr uncorrelated with String)
        let allowances =
            query_owner_allowances(deps.as_ref(), owner.clone(), None, Some(1)).unwrap();
        assert_eq!(allowances.allowances.len(), 1);
        let allow = &allowances.allowances[0];
        assert_eq!(&allow.spender, &spender1);
        assert_eq!(&allow.expires, &expires);
        assert_eq!(&allow.allowance, &allow1);

        // next one is spender2
        let allowances = query_owner_allowances(
            deps.as_ref(),
            owner,
            Some(allow.spender.clone()),
            Some(10000),
        )
        .unwrap();
        assert_eq!(allowances.allowances.len(), 1);
        let allow = &allowances.allowances[0];
        assert_eq!(&allow.spender, &spender2);
        assert_eq!(&allow.expires, &Expiration::Never {});
        assert_eq!(&allow.allowance, &allow2);
    }

    #[test]
    fn query_all_spender_allowances_works() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let mut addresses = [
            deps.api.addr_make("owner1").to_string(),
            deps.api.addr_make("owner2").to_string(),
            deps.api.addr_make("spender").to_string(),
        ];
        addresses.sort();

        // these are in alphabetical order same than insert order
        let [owner1, owner2, spender] = addresses;

        let info = mock_info(owner1.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), &owner1, Uint128::new(12340000));

        // no allowance to start
        let allowances =
            query_spender_allowances(deps.as_ref(), spender.clone(), None, None).unwrap();
        assert_eq!(allowances.allowances, vec![]);

        // set allowance with height expiration
        let allow1 = Uint128::new(7777);
        let expires = Expiration::AtHeight(123_456);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow1,
            expires: Some(expires),
        };
        execute(deps.as_mut(), env, info, msg).unwrap();

        // set allowance with no expiration, from the other owner
        let info = mock_info(owner2.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), &owner2, Uint128::new(12340000));

        let allow2 = Uint128::new(54321);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow2,
            expires: None,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // query list gets both
        let msg = QueryMsg::AllSpenderAllowances {
            spender: spender.clone(),
            start_after: None,
            limit: None,
        };
        let allowances: AllSpenderAllowancesResponse =
            from_json(query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();
        assert_eq!(allowances.allowances.len(), 2);

        // one is owner1 (order of CanonicalAddr uncorrelated with String)
        let msg = QueryMsg::AllSpenderAllowances {
            spender: spender.clone(),
            start_after: None,
            limit: Some(1),
        };
        let allowances: AllSpenderAllowancesResponse =
            from_json(query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();
        assert_eq!(allowances.allowances.len(), 1);
        let allow = &allowances.allowances[0];
        assert_eq!(&allow.owner, &owner1);
        assert_eq!(&allow.expires, &expires);
        assert_eq!(&allow.allowance, &allow1);

        // other one is owner2
        let msg = QueryMsg::AllSpenderAllowances {
            spender,
            start_after: Some(owner1),
            limit: Some(10000),
        };
        let allowances: AllSpenderAllowancesResponse =
            from_json(query(deps.as_ref(), env, msg).unwrap()).unwrap();
        assert_eq!(allowances.allowances.len(), 1);
        let allow = &allowances.allowances[0];
        assert_eq!(&allow.owner, &owner2);
        assert_eq!(&allow.expires, &Expiration::Never {});
        assert_eq!(&allow.allowance, &allow2);
    }

    #[test]
    fn query_all_accounts_works() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // insert order and lexicographical order are different
        let acct1 = deps.api.addr_make("acct1").to_string();
        let acct2 = deps.api.addr_make("zebra").to_string();
        let acct3 = deps.api.addr_make("nice").to_string();
        let acct4 = deps.api.addr_make("aaardvark").to_string();

        let mut expected_order = [acct1.clone(), acct2.clone(), acct3.clone(), acct4.clone()];
        expected_order.sort();

        do_instantiate(deps.as_mut(), &acct1, Uint128::new(12340000));

        // put money everywhere (to create balanaces)
        let info = mock_info(acct1.as_ref(), &[]);
        let env = mock_env();
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Transfer {
                recipient: acct2,
                amount: Uint128::new(222222),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Transfer {
                recipient: acct3,
                amount: Uint128::new(333333),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::Transfer {
                recipient: acct4,
                amount: Uint128::new(444444),
            },
        )
        .unwrap();

        // make sure we get the proper results
        let accounts = query_all_accounts(deps.as_ref(), None, None).unwrap();
        assert_eq!(accounts.accounts, expected_order);

        // let's do pagination
        let accounts = query_all_accounts(deps.as_ref(), None, Some(2)).unwrap();
        assert_eq!(accounts.accounts, expected_order[0..2].to_vec());

        let accounts =
            query_all_accounts(deps.as_ref(), Some(accounts.accounts[1].clone()), Some(1)).unwrap();
        assert_eq!(accounts.accounts, expected_order[2..3].to_vec());

        let accounts =
            query_all_accounts(deps.as_ref(), Some(accounts.accounts[0].clone()), Some(777))
                .unwrap();
        assert_eq!(accounts.accounts, expected_order[3..].to_vec());
    }
}
