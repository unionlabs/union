use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankQuery, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Uint128,
};
use cw20::{BalanceResponse, Cw20ReceiveMsg, TokenInfoResponse};
use frissitheto::UpgradeMsg;
use token_factory_api::{
    BurnTokensMsg, DenomUnit, ForceTransferMsg, Metadata, MintTokensMsg, TokenFactoryMsg,
};

use crate::{
    allowances::{
        execute_burn_from, execute_decrease_allowance, execute_increase_allowance,
        execute_transfer_from, query_allowance,
    },
    enumerable::{query_owner_allowances, query_spender_allowances},
    error::ContractError,
    msg::{ExecuteMsg, InitMsg, MintInfo, QueryMsg},
    self_tf_denom,
    state::{MinterData, TokenInfo, TOKEN_INFO},
};

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match msg {
        ExecuteMsg::MigrateBalances { count } => {
            const BALANCES: cw_storage_plus::Map<&cosmwasm_std::Addr, Uint128> =
                cw_storage_plus::Map::new("balance");

            let denom = self_tf_denom(&env);

            Ok(Response::new().add_messages(
                BALANCES
                    .keys(deps.storage, None, None, Order::Descending)
                    .take(count)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|addr| {
                        let addr = addr.unwrap();
                        let amount = BALANCES.load(deps.storage, &addr).unwrap();

                        BALANCES.remove(deps.storage, &addr);

                        CosmosMsg::Custom(TokenFactoryMsg::MintTokens(MintTokensMsg {
                            denom: denom.clone(),
                            amount,
                            mint_to_address: addr,
                        }))
                    }),
            ))
        }
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::ForceTransfer(
            ForceTransferMsg {
                denom: self_tf_denom(&env),
                amount,
                from_address: info.sender.clone(),
                to_address: rcpt_addr,
            },
        )))
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount))
}

pub fn execute_burn(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    Ok(Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::BurnTokens(
            BurnTokensMsg {
                denom: self_tf_denom(&env),
                amount,
                burn_from_address: info.sender.clone(),
            },
        )))
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if config
        .mint
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        .minter
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::MintTokens(
            MintTokensMsg {
                denom: self_tf_denom(&env),
                amount,
                mint_to_address: rcpt_addr,
            },
        )))
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount))
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount)
        .add_message(CosmosMsg::Custom(TokenFactoryMsg::ForceTransfer(
            ForceTransferMsg {
                denom: self_tf_denom(&env),
                amount,
                from_address: info.sender.clone(),
                to_address: rcpt_addr,
            },
        )))
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract)?
            .change_custom()
            .unwrap(),
        );
    Ok(res)
}

pub fn execute_update_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_minter: Option<String>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let mut config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    let mint = config.mint.as_ref().ok_or(ContractError::Unauthorized {})?;
    if mint.minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let minter_data = new_minter
        .map(|new_minter| deps.api.addr_validate(&new_minter))
        .transpose()?
        .map(|minter| MinterData { minter });

    config.mint = minter_data;

    TOKEN_INFO.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "update_minter")
        .add_attribute(
            "new_minter",
            config
                .mint
                .map(|m| m.minter.into_string())
                .unwrap_or_else(|| "None".to_string()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, env, address)?),
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps, env)?),
        QueryMsg::Allowance { owner, spender } => {
            to_json_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_json_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        } => to_json_binary(&query_spender_allowances(
            deps,
            spender,
            start_after,
            limit,
        )?),
    }
}

pub fn query_balance(deps: Deps, env: Env, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;

    let balance = deps.querier.query_balance(address, self_tf_denom(&env))?;

    Ok(BalanceResponse {
        balance: balance.amount,
    })
}

pub fn query_token_info(deps: Deps, env: Env) -> StdResult<TokenInfoResponse> {
    // see cosmwasm/osmosis-tokenfactory-token-minter/src/bank_types.rs for an explanation of the bearishness
    #[cw_serde]
    pub struct DenomMetadata {
        pub description: String,
        pub denom_units: Vec<DenomUnit>,
        pub base: String,
        pub display: String,
        pub name: String,
        pub symbol: String,
        pub uri: String,
        pub uri_hash: String,
    }

    #[cw_serde]
    pub struct DenomUnit {
        pub denom: String,
        pub exponent: u32,
        pub aliases: Option<Vec<String>>,
    }

    #[cw_serde]
    pub struct DenomMetadataResponse {
        pub metadata: DenomMetadata,
    }

    let denom = self_tf_denom(&env);

    let denom_metadata = deps.querier.query(
        &BankQuery::DenomMetadata {
            denom: denom.clone(),
        }
        .into(),
    );

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
                .unwrap_or(0);
            (metadata.name, metadata.symbol, decimals)
        }
        _ => (denom.clone(), denom.clone(), 0),
    };

    let total_supply = deps.querier.query_supply(denom)?;

    Ok(TokenInfoResponse {
        name,
        symbol,
        decimals,
        total_supply: total_supply.amount,
    })
}

pub fn init(
    deps: DepsMut,
    env: Env,
    msg: InitMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    // store token info
    let data = TokenInfo {
        mint: msg.mint.map(|m| MinterData { minter: m.minter }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    let denom = self_tf_denom(&env);

    Ok(
        Response::new().add_message(CosmosMsg::Custom(TokenFactoryMsg::CreateDenom {
            subdenom: String::new(),
            metadata: Some(Metadata {
                description: Some(format!(
                    "native representation of cw20 token at {}",
                    env.contract.address
                )),
                denom_units: [DenomUnit {
                    denom: denom.clone(),
                    exponent: 0,
                    aliases: vec![msg.symbol.clone()],
                }]
                .into_iter()
                .chain((msg.decimals != 0).then(|| DenomUnit {
                    denom: msg.symbol.clone(),
                    exponent: msg.decimals.into(),
                    aliases: vec![],
                }))
                .collect(),
                base: Some(denom.clone()),
                display: Some(msg.symbol.clone()),
                name: Some(msg.name),
                symbol: Some(msg.symbol),
                uri: None,
                uri_hash: None,
            }),
        })),
    )
}

#[cw_serde]
pub enum MigrateMsg {
    Noop {},
    MigrateFromCw20Base {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    msg.run(
        deps,
        {
            let env = env.clone();
            |deps, init_msg| {
                let res = init(deps, env, init_msg)?;

                Ok((res, None))
            }
        },
        |deps, msg, _| {
            #[cw_serde]
            pub struct TokenInfo {
                pub name: String,
                pub symbol: String,
                pub decimals: u8,
                pub total_supply: Uint128,
                pub mint: Option<MinterData>,
            }

            #[cw_serde]
            pub struct MinterData {
                pub minter: cosmwasm_std::Addr,
                pub cap: Option<Uint128>,
            }

            pub const ORIGINAL_TOKEN_INFO: cw_storage_plus::Item<TokenInfo> =
                cw_storage_plus::Item::new("token_info");

            match msg {
                MigrateMsg::Noop {} => Ok((Response::new(), None)),
                MigrateMsg::MigrateFromCw20Base {} => {
                    let info = ORIGINAL_TOKEN_INFO.load(deps.storage)?;

                    init(
                        deps,
                        env,
                        InitMsg {
                            name: info.name,
                            symbol: info.symbol,
                            decimals: info.decimals,
                            mint: info.mint.map(|m| MintInfo { minter: m.minter }),
                        },
                    )
                    .map(|res| (res, None))
                }
            }
        },
    )
}
