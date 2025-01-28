use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_execute, Addr, BankMsg, Binary, Coin, DenomMetadataResponse,
    Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdResult, SubMsg,
    WasmMsg,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use token_factory_api::TokenFactoryMsg;
use ucs03_zkgm_token_minter_api::{ExecuteMsg, LocalTokenMsg, MetadataResponse, QueryMsg};

use crate::{
    error::Error,
    state::{Config, CONFIG, DENOM_TO_ADDR, DENOM_TO_BE_STORED},
};

pub const NATIVE_TOKEN_STORE_PREFIX: u32 = 0x1;

#[cw_serde]
pub enum TokenMinterInitMsg {
    Cw20 { cw20_code_id: u64 },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    TokenMinterInitMsg::Cw20 { cw20_code_id }: TokenMinterInitMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            admin: info.sender,
            cw20_code_id,
        },
    )?;
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
            TokenFactoryMsg::CreateDenom { metadata, .. } => {
                let metadata = metadata.expect("metadata exists");
                // the first denom is always the same as the generated denom
                let denom = metadata.denom_units[0].denom.clone();
                DENOM_TO_BE_STORED.save(deps.storage, &denom)?;
                let name = metadata.name.expect("metadata name exists");
                let symbol = metadata.symbol.expect("metadata symbol exists");
                let msg = WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: config.cw20_code_id,
                    label: denom,
                    msg: to_json_binary(&cw20_base::msg::InstantiateMsg {
                        name: if !name.is_empty() {
                            name
                        } else {
                            symbol.clone()
                        },
                        symbol,
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
            TokenFactoryMsg::BurnTokens { denom, amount, .. } => {
                let addr = DENOM_TO_ADDR
                    .load(deps.storage, denom.clone())
                    .map_err(|_| Error::CantMint(denom))?;
                let msg = wasm_execute(addr, &cw20::Cw20ExecuteMsg::Burn { amount }, vec![])?;
                Response::new().add_message(msg)
            }
            _ => return Err(Error::UnexpectedExecuteMsg(msg)),
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
                    let msg = wasm_execute(
                        denom,
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
                    let msg = wasm_execute(
                        denom,
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
            .find(|e| &e.ty == "instantiate")
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

#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::Metadata { denom } => match DENOM_TO_ADDR.load(deps.storage, denom.clone()) {
            Ok(addr) => {
                let TokenInfoResponse { name, symbol, .. } = query_token_info(deps, addr.as_str())?;

                Ok(to_json_binary(&MetadataResponse { name, symbol })?)
            }
            Err(_) => match query_token_info(deps, &denom) {
                Ok(TokenInfoResponse { name, symbol, .. }) => {
                    Ok(to_json_binary(&MetadataResponse { name, symbol })?)
                }
                Err(_) => {
                    let denom_metadata = deps.querier.query(&QueryRequest::Bank(
                        cosmwasm_std::BankQuery::DenomMetadata {
                            denom: denom.clone(),
                        },
                    ));

                    let (name, symbol) = match denom_metadata {
                        Ok(DenomMetadataResponse { metadata, .. }) => {
                            (metadata.name, metadata.symbol)
                        }
                        _ => (denom.clone(), denom.clone()),
                    };

                    Ok(to_json_binary(&MetadataResponse { name, symbol })?)
                }
            },
        },
    }
}

fn query_token_info(deps: Deps, addr: &str) -> StdResult<TokenInfoResponse> {
    deps.querier
        .query(&QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
            contract_addr: addr.to_string(),
            msg: to_json_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))
}
