use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, instantiate2_address, to_json_binary, wasm_execute, BankMsg, Binary,
    CodeInfoResponse, Coin, DenomMetadataResponse, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdResult, Storage, WasmMsg,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use frissitheto::UpgradeMsg;
use ibc_union_spec::ChannelId;
use ucs03_zkgm_token_minter_api::{
    ExecuteMsg, LocalTokenMsg, MetadataResponse, PredictWrappedTokenResponse, QueryMsg,
    TokenMinterInitMsg, WrappedTokenMsg,
};
use unionlabs::{ethereum::keccak256, primitives::H256};

use crate::{
    error::Error,
    state::{Config, CONFIG, CW20_ADMIN},
};

pub const DEFAULT_DECIMALS: u8 = 6;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: TokenMinterInitMsg,
) -> Result<Response, Error> {
    let TokenMinterInitMsg::Cw20 {
        cw20_base_code_id,
        dummy_code_id,
        zkgm_admin,
    } = msg
    else {
        return Err(Error::InvalidMinterConfig);
    };
    CONFIG.save(
        deps.storage,
        &Config {
            admin: info.sender,
            cw20_base_code_id,
            dummy_code_id,
        },
    )?;
    CW20_ADMIN.save(deps.storage, &zkgm_admin)?;
    Ok(Response::default())
}

#[cw_serde]
pub struct MigrateMsg {
    /// Update the admin for all *new* cw20 token contracts.
    #[serde(default)]
    pub new_admin: Option<cosmwasm_std::Addr>,

    /// New code id to store for all *new* cw20 token contracts.
    #[serde(default)]
    pub new_cw20_code_id: Option<u64>,
}

#[entry_point]
pub fn migrate(deps: DepsMut, _: Env, msg: MigrateMsg) -> StdResult<Response> {
    if let Some(new_admin) = msg.new_admin {
        CW20_ADMIN.save(deps.storage, &new_admin)?;
    }

    if let Some(new_cw20_code_id) = msg.new_cw20_code_id {
        CONFIG.update::<_, cosmwasm_std::StdError>(deps.storage, |mut c| {
            c.cw20_base_code_id = new_cw20_code_id;
            Ok(c)
        })?;
    }

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
                subdenom,
                path,
                channel_id,
                token,
            } => {
                let token_name = if metadata.name.is_empty() {
                    restrict_name(subdenom.clone())
                } else {
                    restrict_name(metadata.name)
                };
                let token_symbol = if metadata.symbol.is_empty() {
                    restrict_symbol(subdenom.clone())
                } else {
                    restrict_symbol(metadata.symbol)
                };
                let cw20_admin = CW20_ADMIN.load(deps.storage)?;
                Response::new()
                    .add_message(
                        // Instantiating the dummy contract first to be able to get the deterministic address
                        WasmMsg::Instantiate2 {
                            admin: Some(env.contract.address.to_string()),
                            code_id: config.dummy_code_id,
                            label: subdenom.clone(),
                            msg: to_json_binary(&cosmwasm_std::Empty {})?,
                            funds: vec![],
                            salt: Binary::new(calculate_salt(
                                U256::from_be_bytes::<{ U256::BYTES }>(
                                    path.as_slice().try_into().expect("correctly encoded; qed"),
                                ),
                                channel_id,
                                token.to_vec(),
                            )),
                        },
                    )
                    .add_message(
                        // Then migrating to the actual `cw20_base` contract. Note that this contract has a custom
                        // migrate entrypoint where it expects `InstantiateMsg` and calls the its `instantiate` function
                        // in the `migrate` function
                        WasmMsg::Migrate {
                            contract_addr: subdenom.clone(),
                            new_code_id: config.cw20_base_code_id,
                            msg: to_json_binary(&UpgradeMsg::<_, Empty>::Init(
                                cw20_base::msg::InstantiateMsg {
                                    // metadata is not guaranteed to always contain a name, however cw20_base::instantiate requires it to be set
                                    name: token_name,
                                    symbol: token_symbol,
                                    decimals: metadata.decimals,
                                    initial_balances: vec![],
                                    mint: Some(cw20::MinterResponse {
                                        minter: env.contract.address.to_string(),
                                        cap: None,
                                    }),
                                    marketing: None,
                                },
                            ))?,
                        },
                    )
                    .add_message(WasmMsg::UpdateAdmin {
                        // We temporarily set ourselves as admin previously to be able to migrate the contract.
                        // Updating the admin to the correct admin finally.
                        contract_addr: subdenom,
                        admin: cw20_admin.to_string(),
                    })
            }
            WrappedTokenMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            } => Response::new().add_message(wasm_execute(
                denom,
                &cw20::Cw20ExecuteMsg::Mint {
                    recipient: mint_to_address.into_string(),
                    amount,
                },
                vec![],
            )?),
            WrappedTokenMsg::BurnTokens {
                denom,
                amount,
                sender,
                ..
            } => Response::new().add_message(wasm_execute(
                denom,
                &cw20::Cw20ExecuteMsg::BurnFrom {
                    owner: sender.to_string(),
                    amount,
                },
                vec![],
            )?),
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
                    save_native_token(deps.storage, &denom);
                    Response::new()
                } else {
                    Response::new().add_message(wasm_execute(
                        denom,
                        &cw20::Cw20ExecuteMsg::TransferFrom {
                            owner: from,
                            recipient,
                            amount,
                        },
                        vec![],
                    )?)
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
                    Response::new().add_message(wasm_execute(
                        denom,
                        &cw20::Cw20ExecuteMsg::Transfer { recipient, amount },
                        vec![],
                    )?)
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

pub fn save_native_token(storage: &mut dyn Storage, token: &str) {
    storage.set(
        &0x3_u8
            .to_le_bytes()
            .into_iter()
            .chain(token.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        &[1],
    );
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::PredictWrappedToken {
            path,
            channel_id,
            token,
        } => {
            let Config { dummy_code_id, .. } = CONFIG.load(deps.storage)?;
            let code_hash = get_code_hash(deps, dummy_code_id)?;
            let token_addr = instantiate2_address(
                &code_hash.into_bytes(),
                &deps.api.addr_canonicalize(env.contract.address.as_str())?,
                &calculate_salt(
                    path.parse::<U256>().map_err(Error::U256Parse)?,
                    channel_id,
                    token.to_vec(),
                ),
            )?;

            Ok(to_json_binary(&PredictWrappedTokenResponse {
                wrapped_token: deps.api.addr_humanize(&token_addr)?.to_string(),
            })?)
        }
        QueryMsg::Metadata { denom } => match query_token_info(deps, &denom) {
            Ok(TokenInfoResponse {
                name,
                symbol,
                decimals,
                ..
            }) => Ok(to_json_binary(&MetadataResponse {
                name,
                symbol,
                decimals,
            })?),
            Err(_) => {
                let denom_metadata = deps.querier.query(&QueryRequest::Bank(
                    cosmwasm_std::BankQuery::DenomMetadata {
                        denom: denom.clone(),
                    },
                ));

                let (name, symbol, decimals) = match denom_metadata {
                    Ok(DenomMetadataResponse { metadata, .. }) => {
                        let decimals = match metadata.denom_units.first() {
                            Some(unit) => unit.exponent.try_into().unwrap_or(DEFAULT_DECIMALS),
                            None => DEFAULT_DECIMALS,
                        };
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
        },
    }
}

fn get_code_hash(deps: Deps, code_id: u64) -> StdResult<H256> {
    Ok(H256::new(
        *deps
            .querier
            .query::<CodeInfoResponse>(&QueryRequest::Wasm(cosmwasm_std::WasmQuery::CodeInfo {
                code_id,
            }))?
            .checksum
            .as_ref(),
    ))
}

fn query_token_info(deps: Deps, addr: &str) -> StdResult<TokenInfoResponse> {
    deps.querier
        .query(&QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
            contract_addr: addr.to_string(),
            msg: to_json_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))
}

fn calculate_salt(path: U256, channel_id: ChannelId, token: Vec<u8>) -> Vec<u8> {
    keccak256((path, channel_id.raw(), token.to_vec()).abi_encode_params())
        .into_bytes()
        .to_vec()
}

fn restrict_name(name: String) -> String {
    if name.len() > 50 {
        let name = &name[(name.len() - 50)..];
        let split = name.split('/').collect::<Vec<&str>>();
        split[split.len() - 1].to_string()
    } else {
        name
    }
}

/// Restricts the token symbol by the following rules:
/// 1. symbol.len() > 12:
///    Since the symbol can be `factory/ADDR/real_denom`, we try to get the `real_denom` part.
///    Then do sanity check to the characters. And postfix to match the length 3.
/// 2. symbol.len() <= 12:
///    We only do sanity checks and postfix to match the length 3.
fn restrict_symbol(symbol: String) -> String {
    if symbol.len() > 12 {
        // truncate the symbol to get the last 12 chars
        let symbol = &symbol[(symbol.len() - 12)..];
        // split it by `/` incase this is a factory token and only get the last part
        let split = symbol.split('/').collect::<Vec<&str>>();
        // filter the unwanted chars
        let symbol = split[split.len() - 1]
            .chars()
            .filter(|c| *c == '-' || c.is_ascii_alphabetic())
            .collect::<String>();
        // filtering might make the token length < 3, so postfix the denom with '-'
        format!("{symbol:-<3}")
    } else {
        let symbol = symbol
            .chars()
            .filter(|c| *c == '-' || c.is_ascii_alphabetic())
            .collect::<String>();
        // filtering might make the token length < 3, so postfix the denom with '-'
        format!("{symbol:-<3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restrict_name() {
        assert_eq!(&restrict_symbol("muno".into()), "muno");
        assert_eq!(
            &restrict_name(
                "factory/asdelfnaslednunion12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua/clown".into()
            ),
            "clown"
        );
        assert_eq!(
            &restrict_name(
                "alsednfelasndfelasndfleansdfelnasdlefnasledfnleasdnfleasndflenasdfelnasledfelasdnalsednfelasndfelasndfleansdfelnasdflen"
                    .into()
            ),
            "asledfelasdnalsednfelasndfelasndfleansdfelnasdflen"
        );
    }

    #[test]
    fn test_restrict_symbol() {
        assert_eq!(&restrict_symbol("muno".into()), "muno");
        assert_eq!(
            &restrict_symbol("factory/union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua/clown".into()),
            "clown"
        );
        assert_eq!(
            &restrict_symbol(
                "alsednfelasndfelasndfleansdfelnasdlefnasledfnleasdnfleasndflenasdfelnasledfelasdn"
                    .into()
            ),
            "asledfelasdn"
        );
        assert_eq!(
            &restrict_symbol("factory/union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua/a12c".into()),
            "ac-"
        );
        assert_eq!(restrict_symbol("u.".into()), "u--");
        assert_eq!(restrict_symbol("uasd..__".into()), "uasd");
    }
}
