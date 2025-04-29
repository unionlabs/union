use alloy_sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    wasm_execute, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsg, SubMsgResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use frissitheto::UpgradeMsg;
use unionlabs::primitives::Bytes;

use crate::{
    msg::{ExecuteMsg, InitMsg},
    state::{CONFIG, EXECUTING_PARAMS},
    types::{FundedDispatchParameters, StoredFundedDispatchFund, StoredFundedDispatchParameters},
    ContractError,
};

pub const EXECUTE_REPLY_ID: u64 = 0x1337;

/// Instantiate `ucs06-funded-dispatch`.
pub fn init(deps: DepsMut, _: Env, msg: InitMsg) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::new())
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = init(deps, env, init_msg)?;
            Ok((res, None))
        },
        |_deps, _migrate_msg, _current_version| Ok((Response::new(), None)),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::OnZkgm { message, .. } => internal_execute(deps, message, false),
        ExecuteMsg::OnIntentZkgm { message, .. } => internal_execute(deps, message, true),
        ExecuteMsg::Execute { message, intent } => internal_execute(deps, message, intent),
    }
}

fn internal_execute(
    deps: DepsMut,
    message: Bytes,
    intent: bool,
) -> Result<Response, ContractError> {
    let params = FundedDispatchParameters::abi_decode_params(&message, true)?;
    if EXECUTING_PARAMS.exists(deps.storage) {
        return Err(ContractError::ForbiddenReentrancy);
    }
    if intent && !params.allow_market_maker() {
        return Err(ContractError::MarketMakerNotAllowed);
    }
    let contract_address =
        deps.api
            .addr_validate(std::str::from_utf8(&params.contract_address).map_err(|_| {
                ContractError::InvalidContractAddress {
                    address: params.contract_address.to_vec().into(),
                }
            })?)?;
    let mut messages = Vec::<CosmosMsg>::with_capacity(1 + params.funds.len());
    let mut funds = Vec::with_capacity(params.funds.len());
    let mut stored_funds = Vec::new();
    for fund in params.funds.iter() {
        let denom = std::str::from_utf8(&fund.token).map_err(|_| ContractError::InvalidDenom {
            denom: fund.token.to_vec().into(),
        })?;
        let amount = u128::try_from(fund.amount)
            .map_err(|_| ContractError::InvalidAmount {
                amount: fund.amount,
            })?
            .into();
        if deps.querier.query_wasm_contract_info(denom).is_ok() {
            messages.push(
                wasm_execute(
                    denom,
                    &Cw20ExecuteMsg::IncreaseAllowance {
                        spender: contract_address.clone().into(),
                        amount,
                        expires: None,
                    },
                    vec![],
                )?
                .into(),
            );
            stored_funds.push(StoredFundedDispatchFund::Cw20 {
                address: denom.into(),
                amount,
            });
        } else {
            funds.push(Coin::new(amount, denom));
            stored_funds.push(StoredFundedDispatchFund::Native {
                denom: denom.into(),
                amount,
            });
        }
    }
    let beneficiary = deps
        .api
        .addr_validate(std::str::from_utf8(&params.beneficiary).map_err(|_| {
            ContractError::InvalidBeneficiaryAddress {
                address: params.beneficiary.to_vec().into(),
            }
        })?)?;
    EXECUTING_PARAMS.save(
        deps.storage,
        &StoredFundedDispatchParameters {
            allow_failure: params.allow_failure(),
            beneficiary,
            funds: stored_funds,
            contract: contract_address.clone(),
        },
    )?;
    Ok(Response::new()
        .add_messages(messages)
        .add_submessage(SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: contract_address.into(),
                msg: params.contract_calldata.to_vec().into(),
                funds,
            },
            EXECUTE_REPLY_ID,
        )))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply {
        Reply {
            id: EXECUTE_REPLY_ID,
            result,
            ..
        } => {
            let params = EXECUTING_PARAMS.load(deps.storage)?;
            EXECUTING_PARAMS.remove(deps.storage);
            // Allowance reset + optional refund
            let mut messages = Vec::<CosmosMsg>::with_capacity(
                params.funds.len() * if result.is_err() { 2 } else { 1 },
            );
            if let SubMsgResult::Err(error) = result {
                // Refund disallowed, revert the entire pipeline
                if !params.allow_failure {
                    return Err(StdError::generic_err(error).into());
                }
                // If failure is allowed, refund to beneficiary
                for fund in params.funds.clone().into_iter() {
                    match fund {
                        StoredFundedDispatchFund::Native { denom, amount } => {
                            messages.push(
                                BankMsg::Send {
                                    to_address: params.beneficiary.clone().into(),
                                    amount: vec![Coin::new(amount, denom)],
                                }
                                .into(),
                            );
                        }
                        StoredFundedDispatchFund::Cw20 { address, amount } => {
                            messages.push(
                                wasm_execute(
                                    address,
                                    &Cw20ExecuteMsg::Transfer {
                                        recipient: params.beneficiary.clone().into(),
                                        amount,
                                    },
                                    vec![],
                                )?
                                .into(),
                            );
                        }
                    }
                }
            }
            // Reset allowance
            for fund in params.funds {
                if let StoredFundedDispatchFund::Cw20 { address, amount } = fund {
                    messages.push(
                        wasm_execute(
                            address,
                            // Safe to use the full amount, Cw20 removes the
                            // entry if decrease_amount > allowed_amount
                            &Cw20ExecuteMsg::DecreaseAllowance {
                                spender: params.contract.clone().into(),
                                amount,
                                expires: None,
                            },
                            vec![],
                        )?
                        .into(),
                    );
                }
            }
            Ok(Response::new().add_messages(messages))
        }
        _ => Err(StdError::generic_err("impossible").into()),
    }
}
