use alloy::{primitives::U256, sol_types::SolValue};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, QueryRequest, Response, StdResult,
};
use ibc_union_spec::ChannelId;
use prost::Message;
use protos::{
    cosmos::bank::v1beta1::DenomUnit, osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata,
};
use token_factory_api::{TokenFactoryMsg, TokenFactoryQuery};
use ucs03_zkgm_token_minter_api::{
    ExecuteMsg, LocalTokenMsg, MetadataResponse, PredictWrappedTokenResponse, QueryMsg,
    WrappedTokenMsg,
};
use unionlabs::{
    ethereum::keccak256,
    primitives::{encoding::HexPrefixed, Bytes},
    prost::Name,
};

pub const DEFAULT_DECIMALS: u8 = 6;

use crate::{
    error::Error,
    state::{ADMIN, TOKEN_ADMIN, WRAPPED_TOKEN_TO_DENOM},
};

#[cw_serde]
pub enum TokenMinterInitMsg {
    Native { zkgm_admin: Addr },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    TokenMinterInitMsg::Native { zkgm_admin }: TokenMinterInitMsg,
) -> StdResult<Response> {
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
                    subdenom, metadata, ..
                } => {
                    WRAPPED_TOKEN_TO_DENOM.save(
                        deps.storage,
                        subdenom.clone(),
                        &factory_denom(&subdenom, env.contract.address.as_str()),
                    )?;

                    let factory = factory_denom(&subdenom, env.contract.address.as_str());

                    vec![
                        CosmosMsg::Custom(TokenFactoryMsg::CreateDenom {
                            subdenom: subdenom.clone(),
                            // metadata: Some(token_factory_api::Metadata {
                            //     description: None,
                            //     denom_units: vec![token_factory_api::DenomUnit {
                            //         denom: subdenom.clone(),
                            //         exponent: 0,
                            //         aliases: vec![],
                            //     }],
                            //     base: None,
                            //     display: Some(subdenom.clone()),
                            //     name: Some(metadata.name),
                            //     symbol: Some(metadata.symbol),
                            //     uri: None,
                            //     uri_hash: None,
                            // }),
                        }),
                        CosmosMsg::Stargate {
                            // type_url: MsgSetDenomMetadata::full_name(),
                            type_url: "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata"
                                .to_string(),
                            value: MsgSetDenomMetadata {
                                sender: env.contract.address.to_string(),
                                metadata: Some(protos::cosmos::bank::v1beta1::Metadata {
                                    description: "".to_string(),
                                    denom_units: vec![DenomUnit {
                                        denom: factory.clone(),
                                        exponent: 0,
                                        aliases: vec![],
                                    }],
                                    base: factory.clone(),
                                    display: factory.clone(),
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
                            denom: factory,
                            new_admin_address: TOKEN_ADMIN.load(deps.storage)?.to_string(),
                        }),
                    ]
                }
                WrappedTokenMsg::MintTokens {
                    denom,
                    amount,
                    mint_to_address,
                } => {
                    let denom = WRAPPED_TOKEN_TO_DENOM.load(deps.storage, denom)?;
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

fn calculate_salt(path: U256, channel_id: ChannelId, token: Vec<u8>) -> Vec<u8> {
    keccak256((path, channel_id.raw(), token.to_vec()).abi_encode_params())
        .into_bytes()
        .to_vec()
}

#[entry_point]
pub fn query(deps: Deps<TokenFactoryQuery>, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::PredictWrappedToken {
            path,
            channel_id,
            token,
        } => {
            let denom = calculate_salt(
                path.parse::<U256>().map_err(Error::U256Parse)?,
                channel_id,
                token.to_vec(),
            );

            let denom = format!(
                "factory/{}/{}",
                env.contract.address,
                Bytes::<HexPrefixed>::new(denom)
            );

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

fn factory_denom(token: &str, contract: &str) -> String {
    format!("factory/{}/{}", contract, token)
}
