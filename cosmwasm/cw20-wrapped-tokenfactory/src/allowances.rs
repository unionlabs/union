use cosmwasm_std::{
    attr, Addr, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, Uint128,
};
use cw20::{AllowanceResponse, Cw20ReceiveMsg, Expiration};
use token_factory_api::{BurnTokensMsg, ForceTransferMsg, TokenFactoryMsg};

use crate::{
    error::ContractError,
    self_tf_denom,
    state::{ALLOWANCES, ALLOWANCES_SPENDER},
};

// NOTE: copied exactly from cw20-base
pub fn execute_increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    expires: Option<Expiration>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
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

// NOTE: copied exactly from cw20-base
pub fn execute_decrease_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    expires: Option<Expiration>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
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

// NOTE: copied exactly from cw20-base
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
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    let res = Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::ForceTransfer(
            ForceTransferMsg {
                denom: self_tf_denom(&env),
                amount,
                from_address: owner_addr,
                to_address: rcpt_addr,
            },
        )))
        .add_attributes(vec![
            attr("action", "transfer_from"),
            attr("from", owner),
            attr("to", recipient),
            attr("by", info.sender),
            attr("amount", amount),
        ]);

    Ok(res)
}

pub fn execute_burn(
    _: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    Ok(Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::BurnTokens(
            BurnTokensMsg {
                denom: self_tf_denom(&env),
                amount,
                burn_from_address: info.sender.clone(),
            },
        )))
        .add_attributes(vec![
            attr("action", "burn"),
            attr("from", info.sender),
            attr("amount", amount),
        ]))
}

pub fn execute_burn_from(
    deps: DepsMut,

    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    let res = Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::BurnTokens(
            BurnTokensMsg {
                denom: self_tf_denom(&env),
                amount,
                burn_from_address: owner_addr,
            },
        )))
        .add_attributes(vec![
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
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    // create a send message
    let res = Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::ForceTransfer(
            ForceTransferMsg {
                denom: self_tf_denom(&env),
                amount,
                from_address: owner_addr,
                to_address: rcpt_addr,
            },
        )))
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.to_string(),
                amount,
                msg,
            }
            .into_cosmos_msg(&contract)?
            .change_custom::<TokenFactoryMsg>()
            .unwrap(),
        )
        .add_attributes(vec![
            attr("action", "send_from"),
            attr("from", &owner),
            attr("to", &contract),
            attr("by", &info.sender),
            attr("amount", amount),
        ]);
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
