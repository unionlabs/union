use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_execute, Addr, BankMsg, Coin, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, WasmMsg,
};
use token_factory_api::TokenFactoryMsg;
use ucs03_zkgm_token_minter_api::{ExecuteMsg, LocalTokenMsg};

use crate::{
    error::Error,
    state::{Config, CONFIG, DENOM_TO_ADDR, DENOM_TO_BE_STORED},
};

pub const NATIVE_TOKEN_STORE_PREFIX: u32 = 0x1;

#[cw_serde]
pub struct InitMsg {
    config: Config,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    CONFIG.save(deps.storage, &msg.config)?;
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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(Error::OnlyAdmin);
    }

    let response = match msg {
        ExecuteMsg::Wrapped(msg) => match msg {
            TokenFactoryMsg::CreateDenom { .. } => Response::new(),
            TokenFactoryMsg::ChangeAdmin { .. } => {
                panic!("admin is always this contract")
            }
            TokenFactoryMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            } => {
                let addr = DENOM_TO_ADDR
                    .load(deps.storage, denom.clone())
                    .map_err(|_| Error::CantMint(denom))?;
                let msg = wasm_execute(
                    addr,
                    &cw20::Cw20ExecuteMsg::Mint {
                        recipient: mint_to_address,
                        amount,
                    },
                    vec![],
                )?;
                Response::new().add_message(msg)
            }
            TokenFactoryMsg::BurnTokens {
                denom,
                amount,
                burn_from_address,
            } => {
                let addr = DENOM_TO_ADDR
                    .load(deps.storage, denom.clone())
                    .map_err(|_| Error::CantMint(denom))?;
                let msg = wasm_execute(
                    addr,
                    &cw20::Cw20ExecuteMsg::BurnFrom {
                        owner: burn_from_address,
                        amount,
                    },
                    vec![],
                )?;
                Response::new().add_message(msg)
            }
            TokenFactoryMsg::SetDenomMetadata { denom, metadata } => {
                DENOM_TO_BE_STORED.save(deps.storage, &denom)?;
                let msg = WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: config.cw20_code_id,
                    label: denom,
                    msg: to_json_binary(&cw20_base::msg::InstantiateMsg {
                        name: metadata.name.expect("metadata name exists"),
                        symbol: metadata.symbol.expect("metadata name exists"),
                        decimals: 0,
                        initial_balances: vec![],
                        mint: Some(cw20::MinterResponse {
                            minter: env.contract.address.to_string(),
                            cap: None,
                        }),
                        marketing: None,
                    })?,
                    funds: vec![],
                };
                Response::new().add_submessage(SubMsg {
                    id: 1,
                    msg: msg.into(),
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                })
            }
        },
        ExecuteMsg::Local(msg) => match msg {
            LocalTokenMsg::TakeFunds {
                from,
                recipient,
                denom,
                amount,
            } => {
                let contains_base_token = info
                    .funds
                    .iter()
                    .any(|coin| coin.denom == denom && coin.amount == amount);
                if contains_base_token {
                    // this means we are actually sending a native token, no need to
                    // take the funds as they are already given.
                    save_native_token(deps, &denom);
                    Response::new()
                } else {
                    let addr = DENOM_TO_ADDR
                        .load(deps.storage, denom.clone())
                        .map_err(|_| Error::CantMint(denom))?;
                    let msg = wasm_execute(
                        addr,
                        &cw20::Cw20ExecuteMsg::TransferFrom {
                            owner: from,
                            recipient,
                            amount,
                        },
                        vec![],
                    )?;
                    Response::new().add_message(msg)
                }
            }
            LocalTokenMsg::Transfer {
                denom,
                recipient,
                amount,
            } => {
                if is_native_token(deps.as_ref(), &denom) {
                    Response::new().add_message(BankMsg::Send {
                        to_address: recipient,
                        amount: vec![Coin { denom, amount }],
                    })
                } else {
                    let addr = DENOM_TO_ADDR
                        .load(deps.storage, denom.clone())
                        .map_err(|_| Error::CantMint(denom))?;
                    let msg = wasm_execute(
                        addr,
                        &cw20::Cw20ExecuteMsg::Transfer { recipient, amount },
                        vec![],
                    )?;
                    Response::new().add_message(msg)
                }
            }
        },
    };

    Ok(response)
}

fn is_native_token(deps: Deps, token: &str) -> bool {
    match deps.storage.get(
        &0x3_u8
            .to_le_bytes()
            .into_iter()
            .chain(token.as_bytes().to_vec())
            .collect::<Vec<_>>(),
    ) {
        None => false,
        Some(val) => val.len() == 1 && val[0] == 1,
    }
}

fn save_native_token(deps: DepsMut, token: &str) {
    deps.storage.set(
        &0x3_u8
            .to_le_bytes()
            .into_iter()
            .chain(token.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        &[1],
    );
}

#[entry_point]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, Error> {
    if reply.id == 1 {
        let denom = DENOM_TO_BE_STORED
            .load(deps.storage)
            .map_err(|_| Error::DenomToStoreDoesNotExist)?;
        let addr = reply
            .result
            .into_result()
            .map_err(Error::SubMsgError)?
            .events
            .into_iter()
            .find(|e| &e.ty == "wasm")
            .ok_or(Error::ContractCreationEventNotFound)?
            .attributes
            .into_iter()
            .find(|a| &a.key == "_contract_address")
            .ok_or(Error::ContractCreationEventNotFound)?
            .value;

        DENOM_TO_ADDR.save(deps.storage, denom, &Addr::unchecked(addr))?;

        Ok(Response::new())
    } else {
        Err(Error::UnexpectedReply(reply.id))
    }
}
