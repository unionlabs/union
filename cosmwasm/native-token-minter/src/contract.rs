use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult,
};
use token_factory_api::TokenFactoryMsg;
use ucs03_zkgm_token_minter_api::{ExecuteMsg, LocalTokenMsg};

use crate::{error::Error, state::ADMIN};

#[cw_serde]
pub struct InitMsg {
    admin: Addr,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    ADMIN.save(deps.storage, &msg.admin)?;
    Ok(Response::default())
}

#[cw_serde]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, Error> {
    let admin = ADMIN.load(deps.storage)?;

    if admin != info.sender {
        return Err(Error::OnlyAdmin);
    }

    let resp = match msg {
        ExecuteMsg::Wrapped(msg) => {
            if let TokenFactoryMsg::BurnTokens { denom, amount, .. } = &msg {
                let contains_base_token = info
                    .funds
                    .iter()
                    .any(|coin| &coin.denom == denom && &coin.amount == amount);
                if !contains_base_token {
                    return Err(Error::MissingFunds {
                        denom: denom.clone(),
                        amount: amount.clone(),
                    });
                }
            }
            Response::new().add_message(CosmosMsg::Custom(msg))
        }
        ExecuteMsg::Local(msg) => match msg {
            LocalTokenMsg::TakeFunds { denom, amount, .. } => {
                let contains_base_token = info
                    .funds
                    .iter()
                    .any(|coin| coin.denom == denom && coin.amount == amount);
                if !contains_base_token {
                    return Err(Error::MissingFunds { denom, amount });
                }
                Response::new()
            }
            LocalTokenMsg::Transfer {
                denom,
                recipient,
                amount,
            } => Response::new().add_message(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin { denom, amount }],
            }),
        },
    };
    Ok(resp)
}
