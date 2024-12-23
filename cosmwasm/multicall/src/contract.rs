use std::collections::BTreeMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, Coin, DepsMut, Env, MessageInfo, Reply, Response, SubMsg, Uint128, WasmMsg,
};
use unionlabs::never::Never;

use crate::{
    msg::{ExecuteMsg, InitMsg},
    ContractError,
};

const REPLY_ID: u64 = 0x1337;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<Response, Never> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Multicall { calls } => {
            let expected_funds = calls.iter().try_fold(
                BTreeMap::<String, Uint128>::new(),
                |mut s, c| -> Result<_, ContractError> {
                    for coin in &c.funds {
                        let entry = s.entry(coin.denom.clone()).or_default();
                        *entry = entry.checked_add(coin.amount)?;
                    }
                    Ok(s)
                },
            )?;
            for (denom, amount) in expected_funds.into_iter() {
                if !has_coins(&info.funds, &Coin::new(amount, denom)) {
                    return Err(ContractError::FundsMismatch);
                }
            }
            let submessages = calls.into_iter().map(|call| {
                let msg = WasmMsg::Execute {
                    contract_addr: call.target,
                    msg: call.calldata.to_vec().into(),
                    funds: call.funds,
                };
                if call.allow_failure {
                    SubMsg::reply_on_error(msg, REPLY_ID)
                } else {
                    SubMsg::new(msg)
                }
            });
            Ok(Response::new().add_submessages(submessages))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _reply: Reply) -> Result<Response, Never> {
    Ok(Response::default())
}
