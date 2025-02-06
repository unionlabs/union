use cosmwasm_std::{
    attr, Addr, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage, Uint128,
};
use cw20::{AllowanceResponse, Cw20ReceiveMsg, Expiration};

use crate::error::ContractError;
use crate::state::{ALLOWANCES, ALLOWANCES_SPENDER, BALANCES, TOKEN_INFO};

pub fn execute_increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    if spender_addr == info.sender {
        return Err(ContractError::CannotSetOwnAccount {});
    }

    let update_fn = |allow: Option<AllowanceResponse>| -> Result<_, _> {
        let mut val = allow.unwrap_or_default();
        if let Some(exp) = expires {
            if exp.is_expired(&env.block) {
                return Err(ContractError::InvalidExpiration {});
            }
            val.expires = exp;
        }
        val.allowance += amount;
        Ok(val)
    };
    ALLOWANCES.update(deps.storage, (&info.sender, &spender_addr), update_fn)?;
    ALLOWANCES_SPENDER.update(deps.storage, (&spender_addr, &info.sender), update_fn)?;

    let res = Response::new().add_attributes(vec![
        attr("action", "increase_allowance"),
        attr("owner", info.sender),
        attr("spender", spender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_decrease_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    if spender_addr == info.sender {
        return Err(ContractError::CannotSetOwnAccount {});
    }

    let key = (&info.sender, &spender_addr);

    fn reverse<'a>(t: (&'a Addr, &'a Addr)) -> (&'a Addr, &'a Addr) {
        (t.1, t.0)
    }

    // load value and delete if it hits 0, or update otherwise
    let mut allowance = ALLOWANCES.load(deps.storage, key)?;
    if amount < allowance.allowance {
        // update the new amount
        allowance.allowance = allowance
            .allowance
            .checked_sub(amount)
            .map_err(StdError::overflow)?;
        if let Some(exp) = expires {
            if exp.is_expired(&env.block) {
                return Err(ContractError::InvalidExpiration {});
            }
            allowance.expires = exp;
        }
        ALLOWANCES.save(deps.storage, key, &allowance)?;
        ALLOWANCES_SPENDER.save(deps.storage, reverse(key), &allowance)?;
    } else {
        ALLOWANCES.remove(deps.storage, key);
        ALLOWANCES_SPENDER.remove(deps.storage, reverse(key));
    }

    let res = Response::new().add_attributes(vec![
        attr("action", "decrease_allowance"),
        attr("owner", info.sender),
        attr("spender", spender),
        attr("amount", amount),
    ]);
    Ok(res)
}

// this can be used to update a lower allowance - call bucket.update with proper keys
pub fn deduct_allowance(
    storage: &mut dyn Storage,
    owner: &Addr,
    spender: &Addr,
    block: &BlockInfo,
    amount: Uint128,
) -> Result<AllowanceResponse, ContractError> {
    let update_fn = |current: Option<AllowanceResponse>| -> _ {
        match current {
            Some(mut a) => {
                if a.expires.is_expired(block) {
                    Err(ContractError::Expired {})
                } else {
                    // deduct the allowance if enough
                    a.allowance = a
                        .allowance
                        .checked_sub(amount)
                        .map_err(StdError::overflow)?;
                    Ok(a)
                }
            }
            None => Err(ContractError::NoAllowance {}),
        }
    };
    ALLOWANCES.update(storage, (owner, spender), update_fn)?;
    ALLOWANCES_SPENDER.update(storage, (spender, owner), update_fn)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new().add_attributes(vec![
        attr("action", "transfer_from"),
        attr("from", owner),
        attr("to", recipient),
        attr("by", info.sender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_burn_from(
    deps: DepsMut,

    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    // lower balance
    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.total_supply = meta.total_supply.checked_sub(amount)?;
        Ok(meta)
    })?;

    let res = Response::new().add_attributes(vec![
        attr("action", "burn_from"),
        attr("from", owner),
        attr("by", info.sender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let attrs = vec![
        attr("action", "send_from"),
        attr("from", &owner),
        attr("to", &contract),
        attr("by", &info.sender),
        attr("amount", amount),
    ];

    // create a send message
    let msg = Cw20ReceiveMsg {
        sender: info.sender.into(),
        amount,
        msg,
    }
    .into_cosmos_msg(contract)?;

    let res = Response::new().add_message(msg).add_attributes(attrs);
    Ok(res)
}

pub fn query_allowance(deps: Deps, owner: String, spender: String) -> StdResult<AllowanceResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let spender_addr = deps.api.addr_validate(&spender)?;
    let allowance = ALLOWANCES
        .may_load(deps.storage, (&owner_addr, &spender_addr))?
        .unwrap_or_default();
    Ok(allowance)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, CosmosMsg, SubMsg, Timestamp, WasmMsg};
    use cw20::{Cw20Coin, TokenInfoResponse};

    use crate::contract::{execute, instantiate, query_balance, query_token_info};
    use crate::msg::{ExecuteMsg, InstantiateMsg};

    fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
        query_balance(deps, address.into()).unwrap().balance
    }

    // this will set up the instantiation for other tests
    fn do_instantiate<T: Into<String>>(
        mut deps: DepsMut,
        addr: T,
        amount: Uint128,
    ) -> TokenInfoResponse {
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
    fn increase_decrease_allowances() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let owner = deps.api.addr_make("addr0001").to_string();
        let spender = deps.api.addr_make("addr0002").to_string();
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), owner.clone(), Uint128::new(12340000));

        // no allowance to start
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(allowance, AllowanceResponse::default());

        // set allowance with height expiration
        let allow1 = Uint128::new(7777);
        let expires = Expiration::AtHeight(123_456);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow1,
            expires: Some(expires),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(
            allowance,
            AllowanceResponse {
                allowance: allow1,
                expires
            }
        );

        // decrease it a bit with no expire set - stays the same
        let lower = Uint128::new(4444);
        let allow2 = allow1.checked_sub(lower).unwrap();
        let msg = ExecuteMsg::DecreaseAllowance {
            spender: spender.clone(),
            amount: lower,
            expires: None,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(
            allowance,
            AllowanceResponse {
                allowance: allow2,
                expires
            }
        );

        // increase it some more and override the expires
        let raise = Uint128::new(87654);
        let allow3 = allow2 + raise;
        let new_expire = Expiration::AtTime(Timestamp::from_seconds(8888888888));
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: raise,
            expires: Some(new_expire),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(
            allowance,
            AllowanceResponse {
                allowance: allow3,
                expires: new_expire
            }
        );

        // decrease it below 0
        let msg = ExecuteMsg::DecreaseAllowance {
            spender: spender.clone(),
            amount: Uint128::new(99988647623876347),
            expires: None,
        };
        execute(deps.as_mut(), env, info, msg).unwrap();
        let allowance = query_allowance(deps.as_ref(), owner, spender).unwrap();
        assert_eq!(allowance, AllowanceResponse::default());
    }

    #[test]
    fn allowances_independent() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let owner = deps.api.addr_make("addr0001").to_string();
        let spender = deps.api.addr_make("addr0002").to_string();
        let spender2 = deps.api.addr_make("addr0003").to_string();
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), &owner, Uint128::new(12340000));

        // no allowance to start
        assert_eq!(
            query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap(),
            AllowanceResponse::default()
        );
        assert_eq!(
            query_allowance(deps.as_ref(), owner.clone(), spender2.clone()).unwrap(),
            AllowanceResponse::default()
        );
        assert_eq!(
            query_allowance(deps.as_ref(), spender.clone(), spender2.clone()).unwrap(),
            AllowanceResponse::default()
        );

        // set allowance with height expiration
        let allow1 = Uint128::new(7777);
        let expires = Expiration::AtHeight(123_456);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow1,
            expires: Some(expires),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // set other allowance with no expiration
        let allow2 = Uint128::new(87654);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender2.clone(),
            amount: allow2,
            expires: None,
        };
        execute(deps.as_mut(), env, info, msg).unwrap();

        // check they are proper
        let expect_one = AllowanceResponse {
            allowance: allow1,
            expires,
        };
        let expect_two = AllowanceResponse {
            allowance: allow2,
            expires: Expiration::Never {},
        };
        assert_eq!(
            query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap(),
            expect_one
        );
        assert_eq!(
            query_allowance(deps.as_ref(), owner.clone(), spender2.clone()).unwrap(),
            expect_two
        );
        assert_eq!(
            query_allowance(deps.as_ref(), spender.clone(), spender2.clone()).unwrap(),
            AllowanceResponse::default()
        );

        // also allow spender -> spender2 with no interference
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let allow3 = Uint128::new(1821);
        let expires3 = Expiration::AtTime(Timestamp::from_seconds(3767626296));
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender2.clone(),
            amount: allow3,
            expires: Some(expires3),
        };
        execute(deps.as_mut(), env, info, msg).unwrap();
        let expect_three = AllowanceResponse {
            allowance: allow3,
            expires: expires3,
        };
        assert_eq!(
            query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap(),
            expect_one
        );
        assert_eq!(
            query_allowance(deps.as_ref(), owner, spender2.clone()).unwrap(),
            expect_two
        );
        assert_eq!(
            query_allowance(deps.as_ref(), spender, spender2).unwrap(),
            expect_three
        );
    }

    #[test]
    fn no_self_allowance() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let owner = deps.api.addr_make("addr0001").to_string();
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), &owner, Uint128::new(12340000));

        // self-allowance
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: owner.clone(),
            amount: Uint128::new(7777),
            expires: None,
        };
        let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(err, ContractError::CannotSetOwnAccount {});

        // decrease self-allowance
        let msg = ExecuteMsg::DecreaseAllowance {
            spender: owner,
            amount: Uint128::new(7777),
            expires: None,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::CannotSetOwnAccount {});
    }

    #[test]
    fn transfer_from_respects_limits() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let owner = deps.api.addr_make("addr0001").to_string();
        let spender = deps.api.addr_make("addr0002").to_string();
        let rcpt = deps.api.addr_make("addr0003").to_string();

        let start = Uint128::new(999999);
        do_instantiate(deps.as_mut(), &owner, start);

        // provide an allowance
        let allow1 = Uint128::new(77777);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow1,
            expires: None,
        };
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        execute(deps.as_mut(), env, info, msg).unwrap();

        // valid transfer of part of the allowance
        let transfer = Uint128::new(44444);
        let msg = ExecuteMsg::TransferFrom {
            owner: owner.clone(),
            recipient: rcpt.clone(),
            amount: transfer,
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "transfer_from"));

        // make sure money arrived
        assert_eq!(
            get_balance(deps.as_ref(), owner.clone()),
            start.checked_sub(transfer).unwrap()
        );
        assert_eq!(get_balance(deps.as_ref(), rcpt.clone()), transfer);

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        let expect = AllowanceResponse {
            allowance: allow1.checked_sub(transfer).unwrap(),
            expires: Expiration::Never {},
        };
        assert_eq!(expect, allowance);

        // cannot send more than the allowance
        let msg = ExecuteMsg::TransferFrom {
            owner: owner.clone(),
            recipient: rcpt.clone(),
            amount: Uint128::new(33443),
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

        // let us increase limit, but set the expiration to expire in the next block
        let info = mock_info(owner.as_ref(), &[]);
        let mut env = mock_env();
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: Uint128::new(1000),
            expires: Some(Expiration::AtHeight(env.block.height + 1)),
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        env.block.height += 1;

        // we should now get the expiration error
        let msg = ExecuteMsg::TransferFrom {
            owner,
            recipient: rcpt,
            amount: Uint128::new(33443),
        };
        let info = mock_info(spender.as_ref(), &[]);
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::Expired {});
    }

    #[test]
    fn burn_from_respects_limits() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let owner = deps.api.addr_make("addr0001").to_string();
        let spender = deps.api.addr_make("addr0002").to_string();

        let start = Uint128::new(999999);
        do_instantiate(deps.as_mut(), &owner, start);

        // provide an allowance
        let allow1 = Uint128::new(77777);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow1,
            expires: None,
        };
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        execute(deps.as_mut(), env, info, msg).unwrap();

        // valid burn of part of the allowance
        let transfer = Uint128::new(44444);
        let msg = ExecuteMsg::BurnFrom {
            owner: owner.clone(),
            amount: transfer,
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "burn_from"));

        // make sure money burnt
        assert_eq!(
            get_balance(deps.as_ref(), owner.clone()),
            start.checked_sub(transfer).unwrap()
        );

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        let expect = AllowanceResponse {
            allowance: allow1.checked_sub(transfer).unwrap(),
            expires: Expiration::Never {},
        };
        assert_eq!(expect, allowance);

        // cannot burn more than the allowance
        let msg = ExecuteMsg::BurnFrom {
            owner: owner.clone(),
            amount: Uint128::new(33443),
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

        // let us increase limit, but set the expiration to expire in the next block
        let info = mock_info(owner.as_ref(), &[]);
        let mut env = mock_env();
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: Uint128::new(1000),
            expires: Some(Expiration::AtHeight(env.block.height + 1)),
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // increase block height, so the limit is expired now
        env.block.height += 1;

        // we should now get the expiration error
        let msg = ExecuteMsg::BurnFrom {
            owner,
            amount: Uint128::new(33443),
        };
        let info = mock_info(spender.as_ref(), &[]);
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::Expired {});
    }

    #[test]
    fn send_from_respects_limits() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let owner = deps.api.addr_make("addr0001").to_string();
        let spender = deps.api.addr_make("addr0002").to_string();
        let contract = deps.api.addr_make("addr0003").to_string();
        let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

        let start = Uint128::new(999999);
        do_instantiate(deps.as_mut(), &owner, start);

        // provide an allowance
        let allow1 = Uint128::new(77777);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow1,
            expires: None,
        };
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        execute(deps.as_mut(), env, info, msg).unwrap();

        // valid send of part of the allowance
        let transfer = Uint128::new(44444);
        let msg = ExecuteMsg::SendFrom {
            owner: owner.clone(),
            amount: transfer,
            contract: contract.clone(),
            msg: send_msg.clone(),
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "send_from"));
        assert_eq!(1, res.messages.len());

        // we record this as sent by the one who requested, not the one who was paying
        let binary_msg = Cw20ReceiveMsg {
            sender: spender.clone(),
            amount: transfer,
            msg: send_msg.clone(),
        }
        .into_json_binary()
        .unwrap();
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract.clone(),
                msg: binary_msg,
                funds: vec![],
            }))
        );

        // make sure money sent
        assert_eq!(
            get_balance(deps.as_ref(), owner.clone()),
            start.checked_sub(transfer).unwrap()
        );
        assert_eq!(get_balance(deps.as_ref(), contract.clone()), transfer);

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        let expect = AllowanceResponse {
            allowance: allow1.checked_sub(transfer).unwrap(),
            expires: Expiration::Never {},
        };
        assert_eq!(expect, allowance);

        // cannot send more than the allowance
        let msg = ExecuteMsg::SendFrom {
            owner: owner.clone(),
            amount: Uint128::new(33443),
            contract: contract.clone(),
            msg: send_msg.clone(),
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

        // let us increase limit, but set the expiration to the next block
        let info = mock_info(owner.as_ref(), &[]);
        let mut env = mock_env();
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: Uint128::new(1000),
            expires: Some(Expiration::AtHeight(env.block.height + 1)),
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // increase block height, so the limit is expired now
        env.block.height += 1;

        // we should now get the expiration error
        let msg = ExecuteMsg::SendFrom {
            owner,
            amount: Uint128::new(33443),
            contract,
            msg: send_msg,
        };
        let info = mock_info(spender.as_ref(), &[]);
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::Expired {});
    }

    #[test]
    fn no_past_expiration() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let owner = deps.api.addr_make("addr0001").to_string();
        let spender = deps.api.addr_make("addr0002").to_string();
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        do_instantiate(deps.as_mut(), owner.clone(), Uint128::new(12340000));

        // set allowance with height expiration at current block height
        let expires = Expiration::AtHeight(env.block.height);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: Uint128::new(7777),
            expires: Some(expires),
        };

        // ensure it is rejected
        assert_eq!(
            Err(ContractError::InvalidExpiration {}),
            execute(deps.as_mut(), env.clone(), info.clone(), msg)
        );

        // set allowance with time expiration in the past
        let expires = Expiration::AtTime(env.block.time.minus_seconds(1));
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: Uint128::new(7777),
            expires: Some(expires),
        };

        // ensure it is rejected
        assert_eq!(
            Err(ContractError::InvalidExpiration {}),
            execute(deps.as_mut(), env.clone(), info.clone(), msg)
        );

        // set allowance with height expiration at next block height
        let expires = Expiration::AtHeight(env.block.height + 1);
        let allow = Uint128::new(7777);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow,
            expires: Some(expires),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(
            allowance,
            AllowanceResponse {
                allowance: allow,
                expires
            }
        );

        // set allowance with time expiration in the future
        let expires = Expiration::AtTime(env.block.time.plus_seconds(10));
        let allow = Uint128::new(7777);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow,
            expires: Some(expires),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(
            allowance,
            AllowanceResponse {
                allowance: allow + allow, // we increased twice
                expires
            }
        );

        // decrease with height expiration at current block height
        let expires = Expiration::AtHeight(env.block.height);
        let allow = Uint128::new(7777);
        let msg = ExecuteMsg::IncreaseAllowance {
            spender: spender.clone(),
            amount: allow,
            expires: Some(expires),
        };

        // ensure it is rejected
        assert_eq!(
            Err(ContractError::InvalidExpiration {}),
            execute(deps.as_mut(), env.clone(), info.clone(), msg)
        );

        // decrease with height expiration at next block height
        let expires = Expiration::AtHeight(env.block.height + 1);
        let allow = Uint128::new(7777);
        let msg = ExecuteMsg::DecreaseAllowance {
            spender: spender.clone(),
            amount: allow,
            expires: Some(expires),
        };

        execute(deps.as_mut(), env, info, msg).unwrap();

        // ensure it looks good
        let allowance = query_allowance(deps.as_ref(), owner, spender).unwrap();
        assert_eq!(
            allowance,
            AllowanceResponse {
                allowance: allow,
                expires
            }
        );
    }
}
