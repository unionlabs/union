use alloy::sol_types::SolValue;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_execute, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Env, Event, MessageInfo, QueryRequest, Response, StdResult, Uint128,
};
use ibc_union_spec::ChannelId;
use prost::Message;
use protos::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata;
use token_factory_api::{
    BurnTokensMsg, ChangeAdminMsg, MintTokensMsg, TokenFactoryMsg, TokenFactoryQuery,
};
use ucs03_zkgm_token_minter_api::{
    ExecuteMsg as ZkgmExecuteMsg, LocalTokenMsg, Metadata, MetadataResponse,
    PredictWrappedTokenResponse, QueryMsg, TokenMinterInitMsg, WrappedTokenMsg,
};
use unionlabs::{
    ethereum::keccak256,
    primitives::{encoding::Base58, H256, U256},
    prost::Name,
};

pub const DEFAULT_DECIMALS: u8 = 6;

use crate::{
    bank_types::{new_proto_metadata, DenomMetadataResponse},
    error::Error,
    msg::{ExecuteMsg, TokenFactoryAdminOperation},
    state::{OPERATOR, TOKEN_OWNERS, ZKGM_ADDR},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: TokenMinterInitMsg,
) -> Result<Response, Error> {
    let TokenMinterInitMsg::OsmosisTokenFactory { zkgm_admin } = msg else {
        return Err(Error::InvalidMinterConfig);
    };
    OPERATOR.save(deps.storage, &zkgm_admin)?;
    ZKGM_ADDR.save(deps.storage, &info.sender)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, Error> {
    let resp = match msg {
        ExecuteMsg::ZkgmExecuteMsg(msg) => {
            if info.sender != ZKGM_ADDR.load(deps.storage)? {
                return Err(Error::OnlyAdmin);
            }

            match msg {
                ZkgmExecuteMsg::Wrapped(WrappedTokenMsg::CreateDenom {
                    subdenom,
                    metadata,
                    ..
                }) => wrapped_create_denom(deps, env, subdenom, metadata)?,
                ZkgmExecuteMsg::Wrapped(WrappedTokenMsg::MintTokens {
                    denom,
                    amount,
                    mint_to_address,
                }) => delegate_token_operation(
                    TOKEN_OWNERS.load(deps.storage, denom.clone())?,
                    env.contract.address,
                    info.funds,
                    MintTokensMsg {
                        denom,
                        amount,
                        mint_to_address,
                    },
                )?,
                ZkgmExecuteMsg::Wrapped(WrappedTokenMsg::BurnTokens { denom, amount, .. }) => {
                    wrapped_burn_tokens(deps, env, info, denom, amount)?
                }
                ZkgmExecuteMsg::Local(LocalTokenMsg::Escrow { denom, amount, .. }) => {
                    let contains_base_token = info
                        .funds
                        .iter()
                        .any(|coin| coin.denom == denom && coin.amount == amount);
                    if !contains_base_token {
                        return Err(Error::MissingFunds { denom, amount });
                    }
                    Response::new()
                }
                ZkgmExecuteMsg::Local(LocalTokenMsg::Unescrow {
                    denom,
                    recipient,
                    amount,
                }) => Response::new().add_message(BankMsg::Send {
                    to_address: recipient,
                    amount: vec![Coin { denom, amount }],
                }),
            }
        }
        ExecuteMsg::ChangeTokenOwner { denom, new_owner } => {
            change_token_owner(deps, env, info, denom, new_owner)?
        }
    };

    Ok(resp)
}

fn change_token_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    new_owner: Addr,
) -> Result<Response<TokenFactoryMsg>, Error> {
    let token_owner = TOKEN_OWNERS.load(deps.storage, denom.clone())?;

    // The ownership of the self owned tokens can only
    if token_owner == env.contract.address {
        if info.sender != OPERATOR.load(deps.storage)? && info.sender != env.contract.address {
            return Err(Error::UnauthorizedWhenSelfOwned);
        }
    } else if token_owner != info.sender {
        return Err(Error::UnauthorizedThirdParty {
            owner: token_owner,
            sender: info.sender,
        });
    } else if token_owner == new_owner {
        return Ok(Response::new());
    }

    TOKEN_OWNERS.save(deps.storage, denom.clone(), &new_owner)?;

    Ok(delegate_token_operation(
        token_owner.clone(),
        env.contract.address,
        vec![],
        ChangeAdminMsg {
            denom: denom.clone(),
            new_admin_address: new_owner.clone(),
        },
    )?
    .add_event(
        Event::new("token_owner_update")
            .add_attribute("denom", denom)
            .add_attribute("from", token_owner)
            .add_attribute("to", new_owner),
    ))
}

fn wrapped_burn_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    amount: Uint128,
) -> Result<Response<TokenFactoryMsg>, Error> {
    let token_owner = TOKEN_OWNERS.load(deps.storage, denom.clone())?;

    // Although the `BurnTokens` include `burn_from_address`, this functionality is not
    // supported by `TokenFactory` yet and you can only set `burn_from_address` to the token owner.
    // So we are ensuring here that the funds are attached to the call so that we can burn from the
    // token owner.
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
    delegate_token_operation(
        token_owner.clone(),
        env.contract.address,
        info.funds,
        BurnTokensMsg {
            denom,
            amount,
            burn_from_address: token_owner,
        },
    )
}

fn wrapped_create_denom(
    deps: DepsMut,
    env: Env,
    denom: String,
    metadata: Metadata,
) -> Result<Response<TokenFactoryMsg>, Error> {
    TOKEN_OWNERS.save(deps.storage, denom.clone(), &env.contract.address)?;

    let subdenom = deconstruct_factory_denom(&env, &denom)?;

    Ok(Response::new().add_messages(vec![
        CosmosMsg::Custom(TokenFactoryMsg::CreateDenom {
            subdenom: subdenom.to_owned(),
        }),
        // We are using stargate for now instead of `Any` to be safe in case we would want to
        // deploy on < wasmvm 2 chain that uses Osmosis' Token Factory
        #[allow(deprecated)]
        CosmosMsg::Stargate {
            type_url: MsgSetDenomMetadata::type_url(),
            value: MsgSetDenomMetadata {
                sender: env.contract.address.to_string(),
                metadata: Some(new_proto_metadata(denom.clone(), metadata)?),
            }
            .encode_to_vec()
            .into(),
        },
    ]))
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
            let denom_metadata = deps.querier.query(&QueryRequest::Bank(
                cosmwasm_std::BankQuery::DenomMetadata {
                    denom: denom.clone(),
                },
            ));

            let (name, symbol, decimals) = match denom_metadata {
                Ok(DenomMetadataResponse { metadata, .. }) => {
                    let decimals = metadata
                        .denom_units
                        .iter()
                        .find_map(|unit| {
                            if unit.exponent == 0 {
                                None
                            } else {
                                Some(unit.exponent as u8)
                            }
                        })
                        .unwrap_or(DEFAULT_DECIMALS);
                    (metadata.name, metadata.symbol, decimals)
                }
                _ => (denom.clone(), denom.clone(), DEFAULT_DECIMALS),
            };

            Ok(to_json_binary(&MetadataResponse {
                name,
                symbol,
                decimals,
            })?)
        }
    }
}

fn delegate_token_operation<T: Into<TokenFactoryAdminOperation>>(
    token_owner: Addr,
    self_addr: Addr,
    funds: Vec<Coin>,
    operation: T,
) -> Result<Response<TokenFactoryMsg>, Error> {
    if token_owner == self_addr {
        Ok(Response::new().add_message(operation.into().into_cosmos_msg()))
    } else {
        Ok(Response::new().add_message(wasm_execute(token_owner, &operation.into(), funds)?))
    }
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

/// NOTE: Salt is base58 to ensure that the length of the subdenom is 44, as required by tokenfactory.
///
/// <https://github.com/osmosis-labs/osmosis/blob/e14ace31b7ba46be3d519966fb8563127534b245/x/tokenfactory/types/denoms.go#L15>
fn calculate_salt(path: U256, channel_id: ChannelId, token: Vec<u8>) -> H256<Base58> {
    keccak256(
        (
            Into::<alloy::primitives::U256>::into(path),
            channel_id.raw(),
            token.to_vec(),
        )
            .abi_encode_params(),
    )
    .into_encoding()
}

#[cfg(test)]
mod tests {
    use alloy::hex;
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

    #[test]
    fn salt_length_is_44() {
        let salt = calculate_salt(U256::default(), ChannelId!(1), b"muno".into());
        assert_eq!(dbg!(salt.to_string()).len(), 44);

        // 11111111111111111111111111111111
        let min_salt = <H256<Base58>>::from(hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(dbg!(min_salt.to_string()).len(), 32);

        // JEKNVnkbo3jma5nREBBJCDoXFVeKkD56V3xKrvRmWxFG
        let max_salt = <H256<Base58>>::from(hex!(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        ));
        assert_eq!(dbg!(max_salt.to_string()).len(), 44);
    }
}
