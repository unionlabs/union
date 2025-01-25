use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult,
};
use token_factory_api::TokenFactoryMsg;
use ucs03_zkgm_token_minter_api::{ExecuteMsg, LocalTokenMsg};

#[cw_serde]
pub struct InitMsg {}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<Response> {
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
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<TokenFactoryMsg>> {
    let resp = match msg {
        ExecuteMsg::Wrapped(msg) => {
            if let TokenFactoryMsg::BurnTokens { denom, amount, .. } = &msg {
                let contains_base_token = info
                    .funds
                    .iter()
                    .any(|coin| &coin.denom == denom && &coin.amount == amount);
                if !contains_base_token {
                    panic!("missing funds");
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
                    panic!("missing funds");
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
