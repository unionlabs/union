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
                    let fund_amount = info
                        .funds
                        .iter()
                        .find(|c| c.denom == denom)
                        .map(|c| c.amount)
                        .unwrap_or(0u128.into());
                    if fund_amount != amount {
                        return Err(Error::InvalidFunds {
                            needed: amount,
                            given: fund_amount,
                        });
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
    }

    if token_owner == new_owner {
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
    let fund_amount = info
        .funds
        .iter()
        .find(|c| c.denom == denom)
        .map(|c| c.amount)
        .unwrap_or(Uint128::zero());

    if fund_amount != amount {
        return Err(Error::InvalidFunds {
            needed: amount,
            given: fund_amount,
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
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
        Addr, Empty, OwnedDeps,
    };

    use super::*;

    const OPERATOR_ADDR: &str = "operator";
    const ZKGM_ADDR_: &str = "zkgm";

    fn setup(operator: &str, zkgm: &str) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            MessageInfo {
                sender: Addr::unchecked(zkgm),
                funds: vec![],
            },
            TokenMinterInitMsg::OsmosisTokenFactory {
                zkgm_admin: Addr::unchecked(operator),
            },
        )
        .unwrap();

        deps
    }

    #[test]
    fn wrapped_create_denom_ok() {
        let zkgm = Addr::unchecked(ZKGM_ADDR_);
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        let denom = format!("factory/{}/helloworld", mock_env().contract.address);

        let metadata = Metadata {
            name: "Union Token".into(),
            symbol: "UNO".into(),
            decimals: 6,
        };

        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(
                &zkgm,
                &[Coin {
                    denom: denom.clone(),
                    amount: 100u128.into(),
                }],
            ),
            ExecuteMsg::ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg::Wrapped(
                WrappedTokenMsg::CreateDenom {
                    subdenom: denom.clone(),
                    metadata: metadata.clone(),
                    path: vec![].into(),
                    channel_id: ChannelId!(1),
                    token: vec![].into(),
                },
            )),
        )
        .unwrap();

        assert_eq!(
            &res.messages
                .iter()
                .map(|m| m.msg.clone())
                .collect::<Vec<_>>(),
            &[
                CosmosMsg::Custom(TokenFactoryMsg::CreateDenom {
                    subdenom: "helloworld".into(),
                }),
                // We are using stargate for now instead of `Any` to be safe in case we would want to
                // deploy on < wasmvm 2 chain that uses Osmosis' Token Factory
                #[allow(deprecated)]
                CosmosMsg::Stargate {
                    type_url: MsgSetDenomMetadata::type_url(),
                    value: MsgSetDenomMetadata {
                        sender: mock_env().contract.address.into(),
                        metadata: Some(new_proto_metadata(denom.clone(), metadata).unwrap()),
                    }
                    .encode_to_vec()
                    .into(),
                },
            ]
        );

        assert_eq!(
            TOKEN_OWNERS.load(&deps.storage, denom).unwrap(),
            mock_env().contract.address
        );
    }

    #[test]
    fn wrapped_create_denom_invalid_denom() {
        let zkgm = Addr::unchecked(ZKGM_ADDR_);
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        let denom = "factory/omgomg/helloworld".to_string();

        let metadata = Metadata {
            name: "Union Token".into(),
            symbol: "UNO".into(),
            decimals: 6,
        };

        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                message_info(
                    &zkgm,
                    &[Coin {
                        denom: denom.clone(),
                        amount: 100u128.into(),
                    }],
                ),
                ExecuteMsg::ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg::Wrapped(
                    WrappedTokenMsg::CreateDenom {
                        subdenom: denom.clone(),
                        metadata: metadata.clone(),
                        path: vec![].into(),
                        channel_id: ChannelId!(1),
                        token: vec![].into(),
                    },
                )),
            ),
            Err(Error::InvalidDenom(denom))
        );
    }

    #[test]
    fn wrapped_mint_fails_if_no_token() {
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        let denom = "factory/omgomg/helloworld".to_string();

        let mint_tokens = |deps: DepsMut| {
            execute(
                deps,
                mock_env(),
                message_info(&Addr::unchecked(ZKGM_ADDR_), &[]),
                ExecuteMsg::ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg::Wrapped(
                    WrappedTokenMsg::MintTokens {
                        denom: denom.clone(),
                        amount: 100u128.into(),
                        mint_to_address: Addr::unchecked(OPERATOR_ADDR),
                    },
                )),
            )
        };

        assert!(matches!(
            mint_tokens(deps.as_mut()),
            Err(Error::StdError(_))
        ));
    }

    #[test]
    fn wrapped_mint_tokens_ok() {
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        let denom = "factory/omgomg/helloworld".to_string();

        TOKEN_OWNERS
            .save(
                &mut deps.storage,
                denom.clone(),
                &mock_env().contract.address,
            )
            .unwrap();

        let mint_tokens = |deps: DepsMut| {
            execute(
                deps,
                mock_env(),
                message_info(&Addr::unchecked(ZKGM_ADDR_), &[]),
                ExecuteMsg::ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg::Wrapped(
                    WrappedTokenMsg::MintTokens {
                        denom: denom.clone(),
                        amount: 100u128.into(),
                        mint_to_address: Addr::unchecked(OPERATOR_ADDR),
                    },
                )),
            )
        };

        let res = mint_tokens(deps.as_mut()).unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Custom(
                MintTokensMsg {
                    denom: denom.clone(),
                    amount: 100u128.into(),
                    mint_to_address: Addr::unchecked(OPERATOR_ADDR),
                }
                .into()
            )
        );

        TOKEN_OWNERS
            .save(
                &mut deps.storage,
                denom.clone(),
                &Addr::unchecked(ZKGM_ADDR_),
            )
            .unwrap();

        let res = mint_tokens(deps.as_mut()).unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: ZKGM_ADDR_.into(),
                msg: to_json_binary(&TokenFactoryMsg::MintTokens(MintTokensMsg {
                    denom,
                    amount: 100u128.into(),
                    mint_to_address: Addr::unchecked(OPERATOR_ADDR),
                }))
                .unwrap(),
                funds: vec![]
            })
        );
    }

    #[test]
    fn wrapped_token_burn_ok() {
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        let denom = "factory/omgomg/helloworld".to_string();

        TOKEN_OWNERS
            .save(
                &mut deps.storage,
                denom.clone(),
                &mock_env().contract.address,
            )
            .unwrap();

        let burn_tokens = |deps: DepsMut| {
            execute(
                deps,
                mock_env(),
                message_info(
                    &Addr::unchecked(ZKGM_ADDR_),
                    &[Coin {
                        denom: denom.clone(),
                        amount: 100u128.into(),
                    }],
                ),
                ExecuteMsg::ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg::Wrapped(
                    WrappedTokenMsg::BurnTokens {
                        denom: denom.clone(),
                        amount: 100u128.into(),
                        burn_from_address: Addr::unchecked(OPERATOR_ADDR),
                        sender: Addr::unchecked(ZKGM_ADDR_),
                    },
                )),
            )
        };

        let res = burn_tokens(deps.as_mut()).unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Custom(
                BurnTokensMsg {
                    denom: denom.clone(),
                    amount: 100u128.into(),
                    burn_from_address: mock_env().contract.address,
                }
                .into()
            )
        );

        TOKEN_OWNERS
            .save(
                &mut deps.storage,
                denom.clone(),
                &Addr::unchecked(ZKGM_ADDR_),
            )
            .unwrap();

        let res = burn_tokens(deps.as_mut()).unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: ZKGM_ADDR_.into(),
                msg: to_json_binary(&TokenFactoryAdminOperation::BurnTokens(BurnTokensMsg {
                    denom: denom.clone(),
                    amount: 100u128.into(),
                    burn_from_address: Addr::unchecked(ZKGM_ADDR_),
                }))
                .unwrap(),
                funds: vec![Coin {
                    denom,
                    amount: 100u128.into()
                }]
            })
        );
    }

    #[test]
    fn wrapped_token_burn_insufficient_funds() {
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        let denom = "factory/omgomg/helloworld".to_string();

        TOKEN_OWNERS
            .save(
                &mut deps.storage,
                denom.clone(),
                &mock_env().contract.address,
            )
            .unwrap();

        let burn_tokens = |deps: DepsMut, fund_amount: u128| {
            execute(
                deps,
                mock_env(),
                message_info(
                    &Addr::unchecked(ZKGM_ADDR_),
                    if fund_amount == 0 {
                        vec![]
                    } else {
                        vec![Coin {
                            denom: denom.clone(),
                            amount: fund_amount.into(),
                        }]
                    }
                    .as_slice(),
                ),
                ExecuteMsg::ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg::Wrapped(
                    WrappedTokenMsg::BurnTokens {
                        denom: denom.clone(),
                        amount: 100u128.into(),
                        burn_from_address: Addr::unchecked(OPERATOR_ADDR),
                        sender: Addr::unchecked(ZKGM_ADDR_),
                    },
                )),
            )
        };

        assert!(matches!(
            burn_tokens(deps.as_mut(), 50),
            Err(Error::InvalidFunds { .. })
        ));

        assert!(matches!(
            burn_tokens(deps.as_mut(), 0),
            Err(Error::InvalidFunds { .. })
        ));

        assert!(matches!(
            burn_tokens(deps.as_mut(), 101),
            Err(Error::InvalidFunds { .. })
        ));
    }

    #[test]
    fn change_token_owner() {
        let self_addr = mock_env().contract.address;
        let operator = Addr::unchecked(OPERATOR_ADDR);
        let new_owner = Addr::unchecked(ZKGM_ADDR_);
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        TOKEN_OWNERS
            .save(&mut deps.storage, "omg".into(), &self_addr)
            .unwrap();

        // 1. Operator is allowed to change the ownership if the owner of the denom is this contract.
        // If the new owner is the same as the current, this call expects to do nothing and just early return.
        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&operator, &[]),
            ExecuteMsg::ChangeTokenOwner {
                denom: "omg".into(),
                new_owner: self_addr.clone(),
            },
        )
        .unwrap();

        assert!(res.events.is_empty());
        assert!(res.messages.is_empty());

        // 2. The contract itself is allowed to change the ownership when it owns the denom, we expect the proper
        // event and message to be set, and the storage update to be done.
        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&self_addr, &[]),
            ExecuteMsg::ChangeTokenOwner {
                denom: "omg".into(),
                new_owner: new_owner.clone(),
            },
        )
        .unwrap();

        let owner_update_event = res
            .events
            .iter()
            .find(|e| e.ty == "token_owner_update")
            .cloned()
            .unwrap();

        assert_eq!(
            owner_update_event,
            Event::new("token_owner_update")
                .add_attribute("denom", "omg")
                .add_attribute("from", self_addr.as_str())
                .add_attribute("to", ZKGM_ADDR_)
        );

        assert_eq!(res.messages.len(), 1);

        // The owner is self, so we just return a custom msg, no need to a contract call
        assert_eq!(
            CosmosMsg::Custom(
                ChangeAdminMsg {
                    denom: "omg".into(),
                    new_admin_address: new_owner.clone()
                }
                .into()
            ),
            res.messages[0].msg
        );

        assert_eq!(
            TOKEN_OWNERS.load(&deps.storage, "omg".into()).unwrap(),
            new_owner
        );

        // 3. Third party owners are allowed to change the ownership of their tokens too.
        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&new_owner, &[]),
            ExecuteMsg::ChangeTokenOwner {
                denom: "omg".into(),
                new_owner: self_addr.clone(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);

        // This time the owner is not the self, so we do a contract call
        assert_eq!(
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: new_owner.into_string(),
                msg: to_json_binary(&TokenFactoryAdminOperation::ChangeAdmin(ChangeAdminMsg {
                    denom: "omg".into(),
                    new_admin_address: self_addr
                }))
                .unwrap(),
                funds: vec![]
            }),
            res.messages[0].msg
        );
    }

    #[test]
    fn change_token_owner_unauthorized() {
        let self_addr = mock_env().contract.address;
        let third_party_owner = Addr::unchecked(ZKGM_ADDR_);
        let mut deps = setup(OPERATOR_ADDR, ZKGM_ADDR_);

        TOKEN_OWNERS
            .save(&mut deps.storage, "omg".into(), &self_addr)
            .unwrap();

        let change_admin =
            |deps: DepsMut, caller: &str| -> Result<Response<TokenFactoryMsg>, Error> {
                execute(
                    deps,
                    mock_env(),
                    message_info(&Addr::unchecked(caller), &[]),
                    ExecuteMsg::ChangeTokenOwner {
                        denom: "omg".into(),
                        new_owner: self_addr.clone(),
                    },
                )
            };

        assert_eq!(
            change_admin(deps.as_mut(), "unauthorized"),
            Err(Error::UnauthorizedWhenSelfOwned)
        );

        TOKEN_OWNERS
            .save(&mut deps.storage, "omg".into(), &third_party_owner)
            .unwrap();

        assert!(matches!(
            change_admin(deps.as_mut(), "unauthorized"),
            Err(Error::UnauthorizedThirdParty { .. })
        ));

        // The operator also cannot upgrade a third-party owned token
        assert!(matches!(
            change_admin(deps.as_mut(), OPERATOR_ADDR),
            Err(Error::UnauthorizedThirdParty { .. })
        ));
    }

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
