use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_execute, Addr, BankMsg, Binary, Coin, DenomMetadataResponse,
    Deps, DepsMut, Env, Event, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use ucs03_zkgm_token_minter_api::{
    ExecuteMsg, LocalTokenMsg, MetadataResponse, QueryMsg, TokenToIdentifierResponse,
    WrappedTokenMsg,
};

use crate::{
    error::Error,
    state::{Config, ADDR_TO_DENOM, CONFIG, DENOM_TO_ADDR, DENOM_TO_BE_STORED},
};

pub const NATIVE_TOKEN_STORE_PREFIX: u32 = 0x1;

#[cw_serde]
pub enum TokenMinterInitMsg {
    Cw20 { cw20_base_code_id: u64 },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    TokenMinterInitMsg::Cw20 { cw20_base_code_id }: TokenMinterInitMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            admin: info.sender,
            cw20_base_code_id,
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
            WrappedTokenMsg::CreateDenom {
                metadata,
                subdenom: denom,
            } => {
                // the first denom is always the same as the generated denom
                DENOM_TO_BE_STORED.save(deps.storage, &denom)?;
                let name = metadata.name;
                // let symbol = metadata.symbol;
                let msg = WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: config.cw20_base_code_id,
                    label: denom.clone(),
                    msg: to_json_binary(&cw20_base::msg::InstantiateMsg {
                        // metadata is not guaranteed to always contain a name, however cw20_base::instantiate requires it to be set. if it is missing, we use the symbol instead.
                        name: if name.is_empty() || name.len() > 50 {
                            denom
                        } else {
                            name
                        },
                        symbol: "ZKGM".to_string(),
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
            WrappedTokenMsg::MintTokens {
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
            WrappedTokenMsg::BurnTokens {
                denom,
                amount,
                sender,
                ..
            } => {
                let addr = DENOM_TO_ADDR
                    .load(deps.storage, denom.clone())
                    .map_err(|_| Error::CantMint(denom))?;
                let msg = wasm_execute(
                    addr,
                    &cw20::Cw20ExecuteMsg::BurnFrom {
                        owner: sender.to_string(),
                        amount,
                    },
                    vec![],
                )?;
                Response::new().add_message(msg)
            }
        },
        ExecuteMsg::Local(msg) => match msg {
            LocalTokenMsg::Escrow {
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
            LocalTokenMsg::Unescrow {
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

        let addr = deps.api.addr_validate(&addr)?;

        DENOM_TO_ADDR.save(deps.storage, denom.clone(), &addr)?;
        ADDR_TO_DENOM.save(deps.storage, addr.clone(), &denom)?;

        Ok(Response::new().add_event(
            Event::new("cw20_instantiate")
                .add_attribute("quote_token", denom)
                .add_attribute("contract_address", addr),
        ))
    } else {
        Err(Error::UnexpectedReply(reply.id))
    }
}

#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::TokenToIdentifier { token } => {
            let token_identifier = match String::from_utf8(token.to_vec()) {
                Ok(str_token) => ADDR_TO_DENOM
                    .load(deps.storage, Addr::unchecked(str_token))
                    .map(String::into_bytes)
                    .map(Into::into)
                    .unwrap_or(token),
                // If the token is not a utf8 string, it means it's coming from
                // a remote chain. This means it can't be a wrapped token anyways.
                Err(_) => token,
            };

            Ok(to_json_binary(&TokenToIdentifierResponse {
                token_identifier,
            })?)
        }
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

/// Restricts the symbol to have a maximum of 12 characters. This restriction comes from
/// CW20. We know that the symbol won't be smaller than 3 characters and it will always be
/// UTF-8 thanks to the Cosmos SDK. So we only check the maximum character length.
fn restrict_symbol(symbol: &str) -> String {
    if symbol.len() > 12 {
        symbol[..12].to_string()
    } else {
        symbol.to_string()
    }
}
