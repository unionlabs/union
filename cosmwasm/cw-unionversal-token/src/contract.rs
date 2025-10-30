use std::slice;

use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError, StdResult, entry_point,
    to_json_binary,
};
use depolama::StorageExt;
use frissitheto::{UpgradeError, UpgradeMsg};
use ibc_union_spec::{ChannelId, path::commit_packets};
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use token_factory_api::TokenFactoryMsg;
use ucs03_solvable::Solvable;
use ucs03_zkgm::contract::{SOLVER_EVENT, SOLVER_EVENT_MARKET_MAKER_ATTR};
use unionlabs_primitives::{Bytes, U256, encoding::HexPrefixed};

use crate::{
    CwUCtx,
    error::Error,
    msg::{Cw20InstantiateMsg, ExecuteMsg, InitMsg, QueryMsg},
    state::{
        Admin, Cw20ImplType, Cw20Type, FungibleCounterparty, FungibleLane, IntentWhitelist,
        Minters, Zkgm,
    },
};

/// Major state versions of this contract, used in the [`frissitheto`] migrations.
pub mod version {
    use std::num::NonZeroU32;

    /// Initial state of the contract. Access management is handled internally in this contract for specific endpoints.
    pub const INIT: NonZeroU32 = NonZeroU32::new(1).unwrap();

    /// Same as [`INIT`], except that access management is handled externally via [`access_managed`]. All storage in this contract relating to internally handled access management has been removed, and additional storages for [`access_managed`] have been added.
    ///
    /// This is the current latest state version of this contract.
    pub const MANAGED: NonZeroU32 = NonZeroU32::new(2).unwrap();

    /// The latest state version of this contract. Any new deployments will be init'd with this version and the corresponding state.
    pub const LATEST: NonZeroU32 = MANAGED;
}

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!(
        "this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract."
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateMsg {
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response<TokenFactoryMsg>, Error> {
    msg.run(
        deps,
        |mut deps, init_msg| {
            access_managed::init(deps.branch(), init_msg.access_managed_init_msg)?;

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
        |deps, msg, version| match version {
            version::INIT => {
                access_managed::init(deps, msg.access_managed_init_msg)?;

                Ok((Response::default(), Some(version::MANAGED)))
            }
            version::MANAGED => Ok((Response::default(), None)),
            _ => Err(UpgradeError::UnknownStateVersion(version).into()),
        },
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
                let msg = from_value(raw_msg).map_err(|e| StdError::generic_err(e.to_string()))?;
                let res = cw20_base::contract::do_execute::<CwUCtx>(deps, env, info, msg)?;
                Ok(res.change_custom().expect("impossible"))
            }
            Cw20ImplType::Tokenfactory => {
                let msg = from_value(raw_msg).map_err(|e| StdError::generic_err(e.to_string()))?;
                cw20_wrapped_tokenfactory::contract::do_execute::<CwUCtx>(deps, env, info, msg)
                    .map_err(Into::into)
            }
        },
        ExecuteMsg::WhitelistIntents { hashes_whitelist } => {
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
        ExecuteMsg::Solvable(Solvable::DoSolve {
            packet,
            order,
            path,
            caller: _,
            relayer,
            relayer_msg: _,
            intent,
        }) => {
            ensure_zkgm(deps.as_ref(), &info)?;

            if intent {
                let packet_hash = commit_packets(slice::from_ref(&packet));

                let whitelisted = deps
                    .storage
                    .read::<IntentWhitelist>(&packet_hash)
                    .unwrap_or(false);

                if !whitelisted {
                    return Err(Error::IntentMustBeWhitelisted);
                }

                deps.storage.delete::<IntentWhitelist>(&packet_hash);
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
                        recipient: String,
                        amount: u128|
             -> Result<Response<TokenFactoryMsg>, Error> {
                if amount != 0 {
                    match cw20_type {
                        Cw20ImplType::Base => cw20_base::contract::unchecked_internal_mint(
                            deps,
                            recipient,
                            amount.into(),
                        )
                        .map(|x| x.change_custom().unwrap())
                        .map_err(Into::<Error>::into),
                        Cw20ImplType::Tokenfactory => {
                            cw20_wrapped_tokenfactory::contract::unchecked_internal_mint(
                                deps,
                                env,
                                recipient,
                                amount.into(),
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
                receiver.into(),
                order.quote_amount.try_into().expect("impossible"),
            )?;

            let fee = order
                .base_amount
                .checked_sub(order.quote_amount)
                .ok_or_else(|| Error::BaseAmountMustCoverQuoteAmount)?;
            let mint_fee_res = mint(
                deps,
                env,
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
        ExecuteMsg::AccessManaged(msg) => Ok(access_managed::execute(deps, env, info, msg)?
            .change_custom()
            .unwrap()),
        ExecuteMsg::Upgradable(msg) => Ok(upgradable::execute(deps, env, info, msg)?
            .change_custom()
            .unwrap()),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::AllMinters {} => deps
            .storage
            .read_item::<Minters>()
            .and_then(|minters| to_json_binary(&minters))
            .map_err(Into::into),
        QueryMsg::GetFungibleCounterparty {
            path,
            channel_id,
            base_token,
        } => deps
            .storage
            .maybe_read::<FungibleCounterparty>(&(path, channel_id, base_token))
            .and_then(|data| to_json_binary(&data))
            .map_err(Into::into),
        QueryMsg::GetAllFungibleCounterparties {} => deps
            .storage
            .iter::<FungibleCounterparty>(cosmwasm_std::Order::Ascending)
            .map(|res| {
                res.map(
                    |(
                        (path, channel_id, base_token),
                        FungibleLane {
                            counterparty_beneficiary,
                        },
                    )| FungibleLaneConfig {
                        path,
                        channel_id,
                        base_token,
                        counterparty_beneficiary,
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|data| to_json_binary(&data))
            .map_err(Into::into),
        QueryMsg::Minter {} => Err(Error::Unsupported),
        QueryMsg::Cw20(msg) => match deps.storage.read_item::<Cw20Type>()? {
            Cw20ImplType::Base => cw20_base::contract::query(
                deps,
                env,
                from_value(msg).map_err(|e| StdError::generic_err(e.to_string()))?,
            )
            .map_err(Into::into),
            Cw20ImplType::Tokenfactory => cw20_wrapped_tokenfactory::contract::query(
                deps,
                env,
                from_value(msg).map_err(|e| StdError::generic_err(e.to_string()))?,
            )
            .map_err(Into::into),
        },
        QueryMsg::AccessManaged(msg) => access_managed::query(deps, env, msg).map_err(Into::into),
    }
}

#[derive(serde::Serialize)]
pub struct FungibleLaneConfig {
    pub path: U256,
    pub channel_id: ChannelId,
    pub base_token: Bytes,
    pub counterparty_beneficiary: Bytes,
}
