use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult,
};
use token_factory_api::{TokenFactoryMsg, TokenFactoryQuery};
use ucs03_zkgm_token_minter_api::{
    ExecuteMsg, LocalTokenMsg, MetadataResponse, QueryMsg, TokenToIdentifierResponse,
    WrappedTokenMsg,
};

use crate::{
    error::Error,
    state::{ADMIN, WRAPPED_TOKEN_TO_DENOM},
};

#[cw_serde]
pub enum TokenMinterInitMsg {
    Native,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    _: TokenMinterInitMsg,
) -> StdResult<Response> {
    ADMIN.save(deps.storage, &info.sender)?;
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
) -> Result<Response<TokenFactoryMsg>, Error> {
    if info.sender != ADMIN.load(deps.storage)? {
        return Err(Error::OnlyAdmin);
    }

    let resp = match msg {
        ExecuteMsg::Wrapped(msg) => {
            let msg = match msg {
                WrappedTokenMsg::CreateDenom { subdenom, metadata } => {
                    WRAPPED_TOKEN_TO_DENOM.save(
                        deps.storage,
                        subdenom.clone(),
                        &factory_denom(&subdenom, env.contract.address.as_str()),
                    )?;
                    TokenFactoryMsg::CreateDenom {
                        subdenom: subdenom.clone(),
                        metadata: Some(token_factory_api::Metadata {
                            description: None,
                            denom_units: vec![token_factory_api::DenomUnit {
                                denom: subdenom.clone(),
                                exponent: 0,
                                aliases: vec![],
                            }],
                            base: None,
                            display: Some(subdenom.clone()),
                            name: Some(metadata.name),
                            symbol: Some(metadata.symbol),
                            uri: None,
                            uri_hash: None,
                        }),
                    }
                }
                WrappedTokenMsg::MintTokens {
                    denom,
                    amount,
                    mint_to_address,
                } => {
                    let denom = WRAPPED_TOKEN_TO_DENOM.load(deps.storage, denom)?;
                    TokenFactoryMsg::MintTokens {
                        denom,
                        amount,
                        mint_to_address,
                    }
                }
                WrappedTokenMsg::BurnTokens {
                    denom,
                    amount,
                    burn_from_address,
                    ..
                } => {
                    let denom = WRAPPED_TOKEN_TO_DENOM.load(deps.storage, denom)?;
                    let contains_base_token = info
                        .funds
                        .iter()
                        .any(|coin| coin.denom == denom && coin.amount == amount);
                    if !contains_base_token {
                        return Err(Error::MissingFunds {
                            denom: denom.clone(),
                            amount,
                        });
                    }
                    TokenFactoryMsg::BurnTokens {
                        denom,
                        amount,
                        burn_from_address,
                    }
                }
            };
            Response::new().add_message(CosmosMsg::Custom(msg))
        }
        ExecuteMsg::Local(msg) => match msg {
            LocalTokenMsg::Escrow { denom, amount, .. } => {
                let contains_base_token = info
                    .funds
                    .iter()
                    .any(|coin| coin.denom == denom && coin.amount == amount);
                if !contains_base_token {
                    return Err(Error::MissingFunds { denom, amount });
                }
                Response::new()
            }
            LocalTokenMsg::Unescrow {
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

#[entry_point]
pub fn query(deps: Deps<TokenFactoryQuery>, _: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::TokenToIdentifier { token } => Ok(to_json_binary(&TokenToIdentifierResponse {
            token_identifier: token,
        })?),
        QueryMsg::Metadata { denom } => {
            let denom_metadata =
                deps.querier
                    .query::<token_factory_api::MetadataResponse>(
                        &QueryRequest::<TokenFactoryQuery>::Custom(TokenFactoryQuery::Metadata {
                            denom: denom.clone(),
                        }),
                    );
            let (name, symbol) = match denom_metadata {
                Ok(token_factory_api::MetadataResponse {
                    metadata: Some(metadata),
                }) => (
                    metadata.name.unwrap_or(denom.clone()),
                    metadata.symbol.unwrap_or(denom),
                ),
                _ => (denom.clone(), denom),
            };

            Ok(to_json_binary(&MetadataResponse { name, symbol })?)
        }
    }
}

fn factory_denom(token: &str, contract: &str) -> String {
    format!("factory/{}/{}", contract, token)
}
