use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_execute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20::Cw20ExecuteMsg;
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use ibc_union_spec::path::commit_packets;
use unionlabs::primitives::U256;

use crate::{
    error::Error,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Admin, FungibleCounterparty, FungibleLane, IntentWhitelist, Zkgm},
};

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[cw_serde]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InstantiateMsg, MigrateMsg>,
) -> Result<Response, Error> {
    msg.run(
        deps,
        |deps, msg| {
            deps.storage.write_item::<Admin>(&msg.admin);
            Ok((Response::new(), None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

fn ensure_zkgm(deps: Deps, info: &MessageInfo) -> Result<(), Error> {
    let admin = deps.storage.read_item::<Zkgm>()?;
    if info.sender != admin {
        return Err(Error::OnlyZkgm);
    }
    Ok(())
}

fn ensure_admin(deps: Deps, info: &MessageInfo) -> Result<(), Error> {
    let admin = deps.storage.read_item::<Admin>()?;
    if info.sender != admin {
        return Err(Error::OnlyAdmin);
    }
    Ok(())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::WhitelistIntents { hashes_whitelist } => {
            ensure_admin(deps.as_ref(), &info)?;
            for (packet_hash, allowed) in hashes_whitelist {
                deps.storage
                    .write::<IntentWhitelist>(&packet_hash, &allowed);
            }
            Ok(Response::new())
        }
        ExecuteMsg::SetFungibleCounterparty {
            path,
            channel_id,
            base_token,
            counterparty_beneficiary,
            escrowed_denom,
        } => {
            ensure_admin(deps.as_ref(), &info)?;
            let is_cw20 = deps
                .querier
                .query_wasm_contract_info(escrowed_denom.clone())
                .is_ok();
            deps.storage.write::<FungibleCounterparty>(
                &(
                    U256::from_be_bytes(path.to_be_bytes()),
                    channel_id,
                    base_token,
                ),
                &FungibleLane {
                    counterparty_beneficiary,
                    escrowed_denom,
                    is_cw20,
                },
            );
            Ok(Response::new())
        }
        ExecuteMsg::DoSolve {
            packet,
            order,
            path,
            caller: _,
            relayer,
            relayer_msg: _,
            intent,
        } => {
            ensure_zkgm(deps.as_ref(), &info)?;
            if intent {
                let whitelisted = deps
                    .storage
                    .read::<IntentWhitelist>(&commit_packets(&[packet.clone()]))
                    .unwrap_or(false);
                if !whitelisted {
                    return Err(Error::IntentMustBeWhitelisted);
                }
            }

            let fungible_lane = deps
                .storage
                .maybe_read::<FungibleCounterparty>(&(
                    U256::from_be_bytes(path.to_be_bytes()),
                    packet.destination_channel_id,
                    order.base_token,
                ))?
                .ok_or_else(|| Error::LaneIsNotFungible {
                    channel_id: packet.destination_channel_id,
                })?;

            let mut messages = Vec::<CosmosMsg>::with_capacity(2);
            let mut push_transfer = move |to, amount: Uint128| -> StdResult<()> {
                if !amount.is_zero() {
                    if fungible_lane.is_cw20 {
                        messages.push(
                            wasm_execute(
                                fungible_lane.escrowed_denom.clone(),
                                &Cw20ExecuteMsg::Transfer {
                                    recipient: to,
                                    amount,
                                },
                                vec![],
                            )?
                            .into(),
                        );
                    } else {
                        messages.push(
                            BankMsg::Send {
                                to_address: to,
                                amount: vec![Coin::new(
                                    amount,
                                    fungible_lane.escrowed_denom.clone(),
                                )],
                            }
                            .into(),
                        );
                    }
                }
                Ok(())
            };

            let fee = order
                .base_amount
                .checked_sub(order.quote_amount)
                .map_err(|_| Error::BaseAmountMustCoverQuoteAmount)?;
            push_transfer(relayer.into(), fee.try_into().expect("impossible"))?;

            let receiver = deps
                .api
                .addr_validate(
                    str::from_utf8(order.receiver.as_ref()).map_err(|_| Error::InvalidReceiver)?,
                )
                .map_err(|_| Error::InvalidReceiver)?;
            push_transfer(
                receiver.into(),
                order.quote_amount.try_into().expect("impossible"),
            )?;

            Ok(Response::new().set_data(fungible_lane.counterparty_beneficiary.to_vec()))
        }
    }
}

#[entry_point]
pub fn query(_: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsSolver => to_json_binary(&()),
        QueryMsg::AllowMarketMakers => to_json_binary(&true),
    }
}
