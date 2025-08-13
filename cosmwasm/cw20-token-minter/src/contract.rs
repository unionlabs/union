use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, instantiate2_address, to_json_binary, wasm_execute, BankMsg, Binary,
    CodeInfoResponse, Coin, DenomMetadataResponse, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdResult, Storage, WasmMsg,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use frissitheto::UpgradeMsg;
use ibc_union_spec::ChannelId;
use ucs03_zkgm_token_minter_api::{
    encode_metadata, new_wrapped_token_event, ExecuteMsg, LocalTokenMsg, MetadataResponse,
    PredictWrappedTokenResponse, QueryMsg, TokenMinterInitMsg, WrappedTokenKind, WrappedTokenMsg,
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
        cw20_impl_code_id,
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
            cw20_impl_code_id,
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
            c.cw20_impl_code_id = new_cw20_code_id;
            Ok(c)
        })?;
    }

    Ok(Response::new())
}

#[cw_serde]
pub struct Cw20TokenMinterImplementation {
    pub admin: String,
    pub code_id: u64,
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
            WrappedTokenMsg::CreateDenomV2 {
                subdenom,
                path,
                channel_id,
                token,
                implementation,
                initializer,
            } => {
                let cw20_token_minter_implementation =
                    from_json::<Cw20TokenMinterImplementation>(&implementation)?;
                let admin = deps
                    .api
                    .addr_validate(&cw20_token_minter_implementation.admin)?;

                let cw20_admin = CW20_ADMIN.load(deps.storage)?;
                let is_cw20_base_code_id =
                    cw20_token_minter_implementation.code_id == config.cw20_impl_code_id;
                let is_cw20_admin = admin == cw20_admin;
                let is_local_minter =
                    from_json::<UpgradeMsg<cw20_base::msg::InstantiateMsg, Empty>>(&initializer)
                        .map(|msg| match msg {
                            UpgradeMsg::Init(init) => init
                                .mint
                                .map(|minter| {
                                    minter.minter == env.contract.address.clone().into_string()
                                })
                                .unwrap_or(false),
                            UpgradeMsg::Migrate(_) => false,
                        })
                        .unwrap_or(false);

                let is_secure_wrapped_token =
                    is_cw20_base_code_id && is_cw20_admin && is_local_minter;

                let encoded_metadata = encode_metadata(&implementation, &initializer);
                let metadata_image = compute_metadata_image(&encoded_metadata);

                let path = U256::from_be_bytes::<{ U256::BYTES }>(
                    path.as_slice().try_into().expect("correctly encoded; qed"),
                );

                let instantiate_msg = WasmMsg::Instantiate2 {
                    admin: Some(env.contract.address.to_string()),
                    code_id: config.dummy_code_id,
                    label: subdenom.clone(),
                    msg: to_json_binary(&cosmwasm_std::Empty {})?,
                    funds: vec![],
                    salt: Binary::new(calculate_salt_v2(
                        path,
                        channel_id,
                        token.to_vec(),
                        metadata_image,
                    )),
                };

                let migrate_msg = WasmMsg::Migrate {
                    contract_addr: subdenom.clone(),
                    new_code_id: cw20_token_minter_implementation.code_id,
                    msg: initializer,
                };

                let update_admin_msg = WasmMsg::UpdateAdmin {
                    contract_addr: subdenom.clone(),
                    admin: admin.into(),
                };

                let kind = if is_secure_wrapped_token {
                    WrappedTokenKind::Protocol
                } else {
                    WrappedTokenKind::ThirdParty
                };

                Response::new()
                    .add_message(instantiate_msg)
                    .add_message(migrate_msg)
                    .add_message(update_admin_msg)
                    .add_event(new_wrapped_token_event(
                        path,
                        channel_id,
                        token.to_vec(),
                        &subdenom,
                        encoded_metadata,
                        kind,
                    ))
            }
            WrappedTokenMsg::CreateDenom { .. } => {
                return Err(Error::TokenOrderV1DeploymentIsDeprecated);
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
        QueryMsg::PredictWrappedTokenV2 {
            path,
            channel_id,
            token,
            metadata_image,
        } => {
            let Config { dummy_code_id, .. } = CONFIG.load(deps.storage)?;
            let code_hash = get_code_hash(deps, dummy_code_id)?;
            let token_addr = instantiate2_address(
                &code_hash.into_bytes(),
                &deps.api.addr_canonicalize(env.contract.address.as_str())?,
                &calculate_salt_v2(
                    path.parse::<U256>().map_err(Error::U256Parse)?,
                    channel_id,
                    token.to_vec(),
                    metadata_image,
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

fn calculate_salt_v2(
    path: U256,
    channel_id: ChannelId,
    token: Vec<u8>,
    metadata_image: H256,
) -> Vec<u8> {
    keccak256(
        (
            path,
            channel_id.raw(),
            token.to_vec(),
            U256::from_be_bytes(*metadata_image.get()),
        )
            .abi_encode_params(),
    )
    .into_bytes()
    .to_vec()
}

fn compute_metadata_image(metadata: &[u8]) -> H256 {
    keccak256(metadata)
}
