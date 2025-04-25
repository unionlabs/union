use alloy::{primitives::U256, sol_types::SolValue};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult,
};
use ibc_union_spec::ChannelId;
use prost::Message;
use protos::{
    cosmos::bank::v1beta1::DenomUnit, osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata,
};
use token_factory_api::{TokenFactoryMsg, TokenFactoryQuery};
use ucs03_zkgm_token_minter_api::{
    ExecuteMsg, LocalTokenMsg, MetadataResponse, PredictWrappedTokenResponse, QueryMsg,
    TokenMinterInitMsg, WrappedTokenMsg,
};
use unionlabs::{ethereum::keccak256, primitives::H256, prost::Name};

pub const DEFAULT_DECIMALS: u8 = 6;

use crate::{
    error::Error,
    state::{ADMIN, TOKEN_ADMIN},
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: TokenMinterInitMsg,
) -> Result<Response, Error> {
    let TokenMinterInitMsg::OsmosisTokenFactory { zkgm_admin } = msg else {
        return Err(Error::InvalidMinterConfig);
    };
    ADMIN.save(deps.storage, &info.sender)?;
    TOKEN_ADMIN.save(deps.storage, &zkgm_admin)?;
    Ok(Response::default())
}

#[cw_serde]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

fn deconstruct_factory_denom<'a>(env: &Env, denom: &'a str) -> Result<&'a str, Error> {
    let denom_parts = denom
        .split_once('/')
        .and_then(|(a, b)| b.split_once('/').map(|(b, c)| (a, b, c)));

    match denom_parts {
        Some(("factory", addr, subdenom)) if addr == env.contract.address.as_str() => Ok(subdenom),
        _ => Err(Error::InvalidDenom(denom.to_string())),
    }
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
            let msgs = match msg {
                WrappedTokenMsg::CreateDenom {
                    subdenom: denom,
                    metadata,
                    ..
                } => {
                    let subdenom = deconstruct_factory_denom(&env, &denom)?;

                    vec![
                        CosmosMsg::Custom(TokenFactoryMsg::CreateDenom {
                            subdenom: subdenom.to_owned(),
                        }),
                        #[allow(deprecated)]
                        CosmosMsg::Stargate {
                            type_url: MsgSetDenomMetadata::type_url(),
                            value: MsgSetDenomMetadata {
                                sender: env.contract.address.to_string(),
                                metadata: Some(protos::cosmos::bank::v1beta1::Metadata {
                                    description: "".to_string(),
                                    denom_units: vec![
                                        DenomUnit {
                                            denom: denom.clone(),
                                            exponent: 0,
                                            aliases: vec![metadata.symbol.clone()],
                                        },
                                        DenomUnit {
                                            denom: metadata.symbol.clone(),
                                            exponent: metadata.decimals.into(),
                                            aliases: vec![],
                                        },
                                    ],
                                    base: denom.clone(),
                                    display: metadata.symbol.clone(),
                                    name: metadata.name,
                                    symbol: metadata.symbol,
                                    uri: "".to_string(),
                                    uri_hash: "".to_string(),
                                }),
                            }
                            .encode_to_vec()
                            .into(),
                        },
                        CosmosMsg::Custom(TokenFactoryMsg::ChangeAdmin {
                            denom,
                            new_admin_address: TOKEN_ADMIN.load(deps.storage)?.to_string(),
                        }),
                    ]
                }
                WrappedTokenMsg::MintTokens {
                    denom,
                    amount,
                    mint_to_address,
                } => {
                    vec![CosmosMsg::Custom(TokenFactoryMsg::MintTokens {
                        denom,
                        amount,
                        mint_to_address,
                    })]
                }
                WrappedTokenMsg::BurnTokens {
                    denom,
                    amount,
                    burn_from_address,
                    ..
                } => {
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
                    vec![CosmosMsg::Custom(TokenFactoryMsg::BurnTokens {
                        denom,
                        amount,
                        burn_from_address,
                    })]
                }
            };
            Response::new().add_messages(msgs)
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

fn calculate_salt(path: U256, channel_id: ChannelId, token: Vec<u8>) -> H256 {
    keccak256((path, channel_id.raw(), token.to_vec()).abi_encode_params())
}

#[entry_point]
pub fn query(deps: Deps<TokenFactoryQuery>, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::PredictWrappedToken {
            path,
            channel_id,
            token,
        } => {
            let subdenom = calculate_salt(
                path.parse::<U256>().map_err(Error::InvalidPath)?,
                channel_id,
                token.to_vec(),
            );

            let denom = format!("factory/{}/{}", env.contract.address, subdenom);

            Ok(to_json_binary(&PredictWrappedTokenResponse {
                wrapped_token: denom,
            })?)
        }
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

            Ok(to_json_binary(&MetadataResponse {
                name,
                symbol,
                decimals: DEFAULT_DECIMALS,
            })?)
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_env, Addr};

    use super::*;

    #[test]
    fn deconstruct_factory_denom_ok() {
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("addr");

        assert_eq!(
            deconstruct_factory_denom(&env, "factory/addr/denom"),
            Ok("denom")
        );
    }

    #[test]
    fn deconstruct_factory_denom_invalid_address() {
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("addr");

        assert_eq!(
            deconstruct_factory_denom(&env, "factory/wrongaddr/denom"),
            Err(Error::InvalidDenom("factory/wrongaddr/denom".to_owned()))
        );
    }

    #[test]
    fn deconstruct_factory_denom_missing_prefix() {
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("addr");

        assert_eq!(
            deconstruct_factory_denom(&env, "oogabooga/addr/denom"),
            Err(Error::InvalidDenom("oogabooga/addr/denom".to_owned()))
        );
    }

    #[test]
    fn deconstruct_factory_denom_invalid_segments() {
        let mut env = mock_env();
        env.contract.address = Addr::unchecked("addr");

        assert_eq!(
            deconstruct_factory_denom(&env, "factory/addr"),
            Err(Error::InvalidDenom("factory/addr".to_owned()))
        );
    }
}
