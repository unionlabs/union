use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use ibc_union_spec::path::commit_packets;
use serde_json::from_value;
use token_factory_api::TokenFactoryMsg;
use ucs03_zkgm::contract::{SOLVER_EVENT, SOLVER_EVENT_MARKET_MAKER_ATTR};
use unionlabs::{
    primitives::{encoding::HexPrefixed, Bytes},
    ErrorReporter,
};

use crate::{
    error::Error,
    msg::{Cw20InstantiateMsg, ExecuteMsg, InitMsg, QueryMsg},
    state::{
        Admin, Cw20ImplType, Cw20Type, FungibleCounterparty, FungibleLane, IntentWhitelist,
        Minters, Zkgm,
    },
    CwUCtx,
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
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response<TokenFactoryMsg>, Error> {
    msg.run(
        deps,
        |deps, init_msg| {
            let sender = env.contract.address.clone();

            deps.storage.write_item::<Admin>(&init_msg.admin);
            deps.storage.write_item::<Zkgm>(&init_msg.zkgm);
            deps.storage.write_item::<Minters>(&init_msg.extra_minters);

            let res = {
                match init_msg.cw20_init {
                    Cw20InstantiateMsg::Cw20(cw20_init) => {
                        deps.storage.write_item::<Cw20Type>(&Cw20ImplType::Base);
                        let res = cw20_base::contract::instantiate(
                            deps,
                            env,
                            MessageInfo {
                                sender,
                                funds: vec![],
                            },
                            cw20_init,
                        )?;
                        Ok(res.change_custom().expect("impossible"))
                    }
                    Cw20InstantiateMsg::Tokenfactory(cw20_init) => {
                        deps.storage
                            .write_item::<Cw20Type>(&Cw20ImplType::Tokenfactory);
                        cw20_wrapped_tokenfactory::contract::init(deps, env, cw20_init)
                    }
                }
            }?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

fn ensure_zkgm(deps: Deps, info: &MessageInfo) -> Result<(), Error> {
    let zkgm = deps.storage.read_item::<Zkgm>()?;
    if info.sender != zkgm {
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
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, Error> {
    match msg {
        ExecuteMsg::Cw20(raw_msg) => match deps.storage.read_item::<Cw20Type>()? {
            Cw20ImplType::Base => {
                let msg = from_value(raw_msg)
                    .map_err(|e| StdError::generic_err(ErrorReporter(e).to_string()))?;
                let res = cw20_base::contract::do_execute::<CwUCtx>(deps, env, info, msg)?;
                Ok(res.change_custom().expect("impossible"))
            }
            Cw20ImplType::Tokenfactory => {
                let msg = from_value(raw_msg)
                    .map_err(|e| StdError::generic_err(ErrorReporter(e).to_string()))?;
                cw20_wrapped_tokenfactory::contract::do_execute::<CwUCtx>(deps, env, info, msg)
                    .map_err(Into::into)
            }
        },
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
        } => {
            ensure_admin(deps.as_ref(), &info)?;
            deps.storage.write::<FungibleCounterparty>(
                &(path, channel_id, base_token),
                &FungibleLane {
                    counterparty_beneficiary,
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
                    path,
                    packet.destination_channel_id,
                    order.base_token,
                ))?
                .ok_or_else(|| Error::LaneIsNotFungible {
                    channel_id: packet.destination_channel_id,
                })?;

            let quote_token = String::from_utf8(Vec::from(order.quote_token))
                .map_err(|_| Error::InvalidQuoteToken)?;

            if quote_token != env.contract.address.as_str() {
                return Err(Error::InvalidFill { quote_token });
            }

            let cw20_type = deps.storage.read_item::<Cw20Type>()?;

            let mint = |deps: DepsMut,
                        env: Env,
                        info: MessageInfo,
                        recipient: String,
                        amount: Uint128|
             -> Result<Response<TokenFactoryMsg>, Error> {
                if !amount.is_zero() {
                    match cw20_type {
                        Cw20ImplType::Base => cw20_base::contract::execute_mint::<CwUCtx>(
                            deps, env, info, recipient, amount,
                        )
                        .map(|x| x.change_custom().unwrap())
                        .map_err(Into::<Error>::into),
                        Cw20ImplType::Tokenfactory => {
                            cw20_wrapped_tokenfactory::contract::execute_mint::<CwUCtx>(
                                deps, env, info, recipient, amount,
                            )
                            .map_err(Into::<Error>::into)
                        }
                    }
                } else {
                    Ok(Response::new())
                }
            };

            let receiver = deps
                .api
                .addr_validate(
                    str::from_utf8(order.receiver.as_ref()).map_err(|_| Error::InvalidReceiver)?,
                )
                .map_err(|_| Error::InvalidReceiver)?;
            let mint_quote_res = mint(
                deps.branch(),
                env.clone(),
                info.clone(),
                receiver.into(),
                order.quote_amount.try_into().expect("impossible"),
            )?;

            let fee = order
                .base_amount
                .checked_sub(order.quote_amount)
                .map_err(|_| Error::BaseAmountMustCoverQuoteAmount)?;
            let mint_fee_res = mint(
                deps,
                env,
                info.clone(),
                relayer.into(),
                fee.try_into().expect("impossible"),
            )?;

            Ok(Response::new()
                .add_events(mint_quote_res.events)
                .add_attributes(mint_quote_res.attributes)
                .add_submessages(mint_quote_res.messages)
                .add_events(mint_fee_res.events)
                .add_attributes(mint_fee_res.attributes)
                .add_submessages(mint_fee_res.messages)
                .add_event(
                    Event::new(SOLVER_EVENT).add_attribute(
                        SOLVER_EVENT_MARKET_MAKER_ATTR,
                        Bytes::<HexPrefixed>::from(fungible_lane.counterparty_beneficiary.to_vec())
                            .to_string(),
                    ),
                ))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::Minter {} => Err(Error::Unsupported),
        QueryMsg::Cw20(msg) => match deps.storage.read_item::<Cw20Type>()? {
            Cw20ImplType::Base => cw20_base::contract::query(
                deps,
                env,
                from_value(msg).map_err(|e| StdError::generic_err(ErrorReporter(e).to_string()))?,
            )
            .map_err(Into::into),
            Cw20ImplType::Tokenfactory => cw20_wrapped_tokenfactory::contract::query(
                deps,
                env,
                from_value(msg).map_err(|e| StdError::generic_err(ErrorReporter(e).to_string()))?,
            )
            .map_err(Into::into),
        },
    }
}
