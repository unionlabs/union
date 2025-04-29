use core::str;

use alloy_primitives::U256;
use alloy_sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, to_json_string, wasm_execute, Addr, Binary,
    CodeInfoResponse, Coin, Coins, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest,
    Reply, Response, StdError, StdResult, Storage, SubMsg, SubMsgResult, Uint256, WasmMsg,
};
use frissitheto::UpgradeMsg;
use ibc_union_msg::{
    module::IbcUnionMsg,
    msg::{MsgSendPacket, MsgWriteAcknowledgement},
};
use ibc_union_spec::{path::BatchPacketsPath, ChannelId, Packet, Timestamp};
use ucs03_zkgm_token_minter_api::{LocalTokenMsg, Metadata, MetadataResponse, WrappedTokenMsg};
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bytes, H256},
};

use crate::{
    com::{
        Ack, Batch, BatchAck, Forward, FungibleAssetOrder, FungibleAssetOrderAck, Instruction,
        Multiplex, ZkgmPacket, ACK_ERR_ONLY_MAKER, FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL,
        FORWARD_SALT_MAGIC, INSTR_VERSION_0, INSTR_VERSION_1, OP_BATCH, OP_FORWARD,
        OP_FUNGIBLE_ASSET_ORDER, OP_MULTIPLEX, TAG_ACK_FAILURE, TAG_ACK_SUCCESS,
    },
    msg::{ExecuteMsg, InitMsg, PredictWrappedTokenResponse, QueryMsg, ZkgmMsg},
    state::{
        BATCH_EXECUTION_ACKS, CHANNEL_BALANCE, CONFIG, EXECUTING_PACKET, EXECUTING_PACKET_IS_BATCH,
        EXECUTION_ACK, HASH_TO_FOREIGN_TOKEN, IN_FLIGHT_PACKET, MARKET_MAKER, TOKEN_BUCKET,
        TOKEN_MINTER, TOKEN_ORIGIN,
    },
    token_bucket::TokenBucket,
    ContractError,
};

pub const PROTOCOL_VERSION: &str = "ucs03-zkgm-0";

pub const EXECUTE_REPLY_ID: u64 = 0x1337;
pub const TOKEN_INIT_REPLY_ID: u64 = 0xbeef;
pub const FORWARD_REPLY_ID: u64 = 0xbabe;
pub const MULTIPLEX_REPLY_ID: u64 = 0xface;
pub const MM_FILL_REPLY_ID: u64 = 0xdead;

pub const ZKGM_TOKEN_MINTER_LABEL: &str = "zkgm-token-minter";

/// Instantiate `ucs03-zkgm`.
///
/// This will instantiate the minter contract with the provided [`TokenMinterInitMsg`][crate::msg::TokenMinterInitMsg]. The admin of the minter contract is set to `ucs03-zkgm`. All migrations for the minter will be threaded through the `ucs03-zkgm` migrate entrypoint.
pub fn init(deps: DepsMut, env: Env, msg: InitMsg) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;

    let salt = minter_salt();

    let minter_address = instantiate2_address(
        get_code_hash(deps.as_ref(), msg.config.token_minter_code_id)?.as_ref(),
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        salt.as_bytes(),
    )
    .expect("valid instantiate2 address");

    TOKEN_MINTER.save(deps.storage, &deps.api.addr_humanize(&minter_address)?)?;

    let msg = WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.config.token_minter_code_id,
        msg: to_json_binary(&msg.minter_init_params.into_msg(msg.config.admin))?,
        funds: vec![],
        label: ZKGM_TOKEN_MINTER_LABEL.to_string(),
        salt: salt.into_bytes().into(),
    };

    Ok(Response::new().add_submessage(SubMsg::reply_never(msg)))
}

/// The salt used to instantiate the token minter (using [instantiate2_address]).
pub fn minter_salt() -> String {
    format!("{PROTOCOL_VERSION}/{ZKGM_TOKEN_MINTER_LABEL}")
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::IbcUnionMsg(ibc_msg) => {
            if info.sender != CONFIG.load(deps.storage)?.ibc_host {
                return Err(ContractError::OnlyIBCHost);
            }

            match ibc_msg {
                IbcUnionMsg::OnChannelOpenInit { version, .. } => {
                    enforce_version(&version, None)?;
                    Ok(Response::default())
                }
                IbcUnionMsg::OnChannelOpenTry {
                    version,
                    counterparty_version,
                    ..
                } => {
                    enforce_version(&version, Some(&counterparty_version))?;
                    Ok(Response::default())
                }
                IbcUnionMsg::OnChannelOpenAck { .. } | IbcUnionMsg::OnChannelOpenConfirm { .. } => {
                    Ok(Response::default())
                }
                IbcUnionMsg::OnRecvPacket {
                    caller,
                    packet,
                    relayer,
                    relayer_msg,
                } => dispatch_execute_packet(
                    deps,
                    env,
                    info,
                    caller,
                    packet,
                    relayer,
                    relayer_msg,
                    false,
                ),
                IbcUnionMsg::OnIntentRecvPacket {
                    caller,
                    packet,
                    market_maker,
                    market_maker_msg,
                } => dispatch_execute_packet(
                    deps,
                    env,
                    info,
                    caller,
                    packet,
                    market_maker,
                    market_maker_msg,
                    true,
                ),
                IbcUnionMsg::OnAcknowledgementPacket {
                    caller,
                    packet,
                    acknowledgement,
                    relayer,
                } => {
                    let caller = deps.api.addr_validate(&caller)?;
                    let relayer = deps.api.addr_validate(&relayer)?;
                    acknowledge_packet(deps, env, info, caller, packet, relayer, acknowledgement)
                }
                IbcUnionMsg::OnTimeoutPacket {
                    caller,
                    packet,
                    relayer,
                } => {
                    let caller = deps.api.addr_validate(&caller)?;
                    let relayer = deps.api.addr_validate(&relayer)?;
                    timeout_packet(deps, env, info, caller, packet, relayer)
                }
                IbcUnionMsg::OnChannelCloseInit { .. }
                | IbcUnionMsg::OnChannelCloseConfirm { .. } => {
                    Err(StdError::generic_err("the show must go on").into())
                }
            }
        }
        ExecuteMsg::InternalBatch { messages } => {
            if info.sender != env.contract.address {
                Err(ContractError::OnlySelf)
            } else {
                Ok(Response::new().add_messages(messages))
            }
        }
        ExecuteMsg::InternalExecutePacket {
            caller,
            packet,
            relayer,
            relayer_msg,
            intent,
        } => {
            if info.sender != env.contract.address {
                Err(ContractError::OnlySelf)
            } else {
                execute_packet(
                    deps,
                    env,
                    info,
                    caller,
                    packet,
                    relayer,
                    relayer_msg,
                    intent,
                )
            }
        }
        ExecuteMsg::InternalWriteAck { ack } => {
            if info.sender != env.contract.address {
                Err(ContractError::OnlySelf)
            } else {
                if EXECUTING_PACKET_IS_BATCH.may_load(deps.storage)?.is_some() {
                    let acks = match BATCH_EXECUTION_ACKS.load(deps.storage) {
                        Ok(mut batch_acks) => {
                            batch_acks.push(ack);
                            batch_acks
                        }
                        Err(_) => {
                            vec![ack]
                        }
                    };
                    BATCH_EXECUTION_ACKS.save(deps.storage, &acks)?;
                } else {
                    EXECUTION_ACK.save(deps.storage, &ack)?;
                }
                Ok(Response::new())
            }
        }
        ExecuteMsg::Send {
            channel_id,
            timeout_height,
            timeout_timestamp,
            salt,
            instruction,
        } => send(
            deps,
            info,
            channel_id,
            timeout_height.u64(),
            timeout_timestamp,
            salt,
            Instruction::abi_decode_params(&instruction, true)?,
        ),
        ExecuteMsg::SetRateLimitOperators {
            rate_limit_operators,
        } => {
            if info.sender != CONFIG.load(deps.storage)?.rate_limit_admin {
                return Err(ContractError::OnlyRateLimitAdmin);
            }
            let mut config = CONFIG.load(deps.storage)?;
            config.rate_limit_operators = rate_limit_operators;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_event(Event::new("rate_limit_operators_update")))
        }
        ExecuteMsg::SetBucketConfig {
            denom,
            capacity,
            refill_rate,
            reset,
        } => {
            if !CONFIG
                .load(deps.storage)?
                .rate_limit_operators
                .contains(&info.sender)
            {
                return Err(ContractError::OnlyRateLimitOperator);
            }
            let token_bucket = TOKEN_BUCKET.update(
                deps.storage,
                denom.clone(),
                |entry| -> Result<_, ContractError> {
                    match entry {
                        Some(mut token_bucket) => {
                            token_bucket.update(capacity, refill_rate, reset)?;
                            Ok(token_bucket)
                        }
                        None => Ok(TokenBucket::new(
                            capacity,
                            refill_rate,
                            env.block.time.seconds(),
                        )?),
                    }
                },
            )?;
            Ok(Response::new().add_event(
                Event::new("token_bucket_update")
                    .add_attribute("denom", denom)
                    .add_attribute("capacity", token_bucket.capacity)
                    .add_attribute("refill_rate", token_bucket.refill_rate),
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_execute_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: String,
    packet: Packet,
    relayer: String,
    relayer_msg: Bytes,
    intent: bool,
) -> Result<Response, ContractError> {
    let caller = deps.api.addr_validate(&caller)?;
    let relayer = deps.api.addr_validate(&relayer)?;
    if EXECUTING_PACKET.exists(deps.storage) {
        Err(ContractError::AlreadyExecuting)
    } else {
        EXECUTING_PACKET.save(deps.storage, &packet)?;
        Ok(Response::default().add_submessage(SubMsg::reply_always(
            wasm_execute(
                env.contract.address,
                &ExecuteMsg::InternalExecutePacket {
                    caller,
                    packet,
                    relayer,
                    relayer_msg,
                    intent,
                },
                info.funds,
            )?,
            EXECUTE_REPLY_ID,
        )))
    }
}

/// Enforces that the IBC protocol version matches the expected version.
/// Checks both local and counterparty versions if provided.
fn enforce_version(version: &str, counterparty_version: Option<&str>) -> Result<(), ContractError> {
    if version != PROTOCOL_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: version.to_string(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != PROTOCOL_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    Ok(())
}

/// Handles IBC packet timeouts by either processing forwarded packet timeouts or
/// executing timeout logic for normal packets.
fn timeout_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
) -> Result<Response, ContractError> {
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    // Check if this is an in-flight packet (forwarded packet)
    if is_forwarded_packet(zkgm_packet.salt.0.into()) {
        // This is a forwarded packet timeout
        // Find the parent packet that initiated the forward
        let commitment_key = BatchPacketsPath::from_packets(&[packet.clone()]).key();

        if let Some(parent_packet) =
            IN_FLIGHT_PACKET.may_load(deps.storage, commitment_key.into_bytes().into())?
        {
            // Erase the parent packet
            IN_FLIGHT_PACKET.remove(deps.storage, commitment_key.into_bytes().into());

            let config = CONFIG.load(deps.storage)?;
            return Ok(Response::new().add_message(wasm_execute(
                &config.ibc_host,
                &ibc_union_msg::msg::ExecuteMsg::WriteAcknowledgement(MsgWriteAcknowledgement {
                    packet: parent_packet,
                    acknowledgement: Ack {
                        tag: TAG_ACK_FAILURE,
                        inner_ack: Default::default(),
                    }
                    .abi_encode_params()
                    .into(),
                }),
                vec![],
            )?));
        }
    }
    timeout_internal(
        deps,
        env,
        info,
        caller,
        packet,
        relayer,
        zkgm_packet.salt.into(),
        zkgm_packet.path,
        zkgm_packet.instruction,
    )
}

#[allow(clippy::too_many_arguments)]
/// Handles the internal timeout logic for a packet.
/// Processes timeouts based on instruction type and executes appropriate refund/cleanup actions.
fn timeout_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    salt: H256,
    path: U256,
    instruction: Instruction,
) -> Result<Response, ContractError> {
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            if instruction.version != INSTR_VERSION_1 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            refund(deps, path, packet.source_channel_id, order)
        }
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let mut response = Response::new();
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            for (i, instruction) in batch.instructions.into_iter().enumerate() {
                let sub_response = timeout_internal(
                    deps.branch(),
                    env.clone(),
                    info.clone(),
                    caller.clone(),
                    packet.clone(),
                    relayer.clone(),
                    derive_batch_salt(i.try_into().unwrap(), salt),
                    path,
                    instruction,
                )?;
                response = response
                    .add_events(sub_response.events)
                    .add_attributes(sub_response.attributes)
                    .add_submessages(sub_response.messages)
            }
            Ok(response)
        }
        OP_MULTIPLEX => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            timeout_multiplex(deps, caller, packet, relayer, path, multiplex)
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn timeout_multiplex(
    deps: DepsMut,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    path: U256,
    multiplex: Multiplex,
) -> Result<Response, ContractError> {
    if multiplex.eureka {
        // For eureka mode, forward the timeout to the sender contract
        let sender_addr = deps
            .api
            .addr_validate(
                str::from_utf8(&multiplex.sender).map_err(|_| ContractError::InvalidSender)?,
            )
            .map_err(|_| ContractError::UnableToValidateSender)?;

        // Create a virtual packet with the multiplex data
        let multiplex_packet = Packet {
            source_channel_id: packet.source_channel_id,
            destination_channel_id: packet.destination_channel_id,
            data: encode_multiplex_calldata(
                path,
                multiplex.sender.into(),
                multiplex.contract_calldata.into(),
            )
            .into(),
            timeout_height: packet.timeout_height,
            timeout_timestamp: packet.timeout_timestamp,
        };

        // Forward the timeout to the sender contract
        Ok(Response::new().add_message(wasm_execute(
            sender_addr,
            &IbcUnionMsg::OnTimeoutPacket {
                caller: caller.into(),
                packet: multiplex_packet,
                relayer: relayer.into(),
            },
            vec![],
        )?))
    } else {
        // For standard mode, no action needed
        Ok(Response::new())
    }
}

/// Handles IBC packet acknowledgements by either forwarding them for forwarded packets
/// or processing them for normal packets.
fn acknowledge_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    ack: Bytes,
) -> Result<Response, ContractError> {
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    // Check if this is an in-flight packet (forwarded packet)
    if is_forwarded_packet(zkgm_packet.salt.0.into()) {
        // This is a forwarded packet acknowledgement
        // Find the parent packet that initiated the forward
        let commitment_key = BatchPacketsPath::from_packets(&[packet.clone()]).key();

        if let Some(parent_packet) =
            IN_FLIGHT_PACKET.may_load(deps.storage, commitment_key.into_bytes().into())?
        {
            // Forward the acknowledgement to the parent packet
            IN_FLIGHT_PACKET.remove(deps.storage, commitment_key.into_bytes().into());

            let config = CONFIG.load(deps.storage)?;
            return Ok(Response::new().add_message(wasm_execute(
                &config.ibc_host,
                &ibc_union_msg::msg::ExecuteMsg::WriteAcknowledgement(MsgWriteAcknowledgement {
                    packet: parent_packet,
                    acknowledgement: ack,
                }),
                vec![],
            )?));
        }
    }
    let ack = Ack::abi_decode_params(&ack, true)?;
    acknowledge_internal(
        deps,
        env,
        info,
        caller,
        packet,
        relayer,
        zkgm_packet.salt.into(),
        zkgm_packet.path,
        zkgm_packet.instruction,
        ack.tag == TAG_ACK_SUCCESS,
        Vec::from(ack.inner_ack).into(),
    )
}

#[allow(clippy::too_many_arguments)]
/// Handles the internal acknowledgement logic for a packet.
/// Processes acknowledgements based on instruction type and success status.
/// For successful acknowledgements, executes the appropriate success handlers.
/// For failed acknowledgements, executes refund/cleanup actions.
fn acknowledge_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    salt: H256,
    path: U256,
    instruction: Instruction,
    successful: bool,
    ack: Bytes,
) -> Result<Response, ContractError> {
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            if instruction.version != INSTR_VERSION_1 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            let order_ack = if successful {
                Some(FungibleAssetOrderAck::abi_decode_params(&ack, true)?)
            } else {
                None
            };
            acknowledge_fungible_asset_order(
                deps, env, info, packet, relayer, salt, path, order, order_ack,
            )
        }
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let mut response = Response::new();
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            let batch_ack = if successful {
                Some(BatchAck::abi_decode_params(&ack, true)?)
            } else {
                None
            };
            for (i, instruction) in batch.instructions.into_iter().enumerate() {
                let sub_response = acknowledge_internal(
                    deps.branch(),
                    env.clone(),
                    info.clone(),
                    caller.clone(),
                    packet.clone(),
                    relayer.clone(),
                    derive_batch_salt(i.try_into().unwrap(), salt),
                    path,
                    instruction,
                    successful,
                    batch_ack
                        .as_ref()
                        .map(|batch_ack| Vec::from(batch_ack.acknowledgements[i].clone()).into())
                        .unwrap_or_default(),
                )?;
                response = response
                    .add_attributes(sub_response.attributes)
                    .add_events(sub_response.events)
                    .add_submessages(sub_response.messages);
            }
            Ok(response)
        }
        OP_MULTIPLEX => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            acknowledge_multiplex(
                deps, caller, packet, relayer, path, multiplex, successful, ack,
            )
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn refund(
    deps: DepsMut,
    path: U256,
    source_channel: ChannelId,
    order: FungibleAssetOrder,
) -> Result<Response, ContractError> {
    // 1. Native minter + sent native tokens: correct
    // 2. Cw20 minter + sent native tokens:
    let sender = deps
        .api
        .addr_validate(str::from_utf8(&order.sender).map_err(|_| ContractError::InvalidSender)?)
        .map_err(|_| ContractError::UnableToValidateSender)?;
    let minter = TOKEN_MINTER.load(deps.storage)?;
    let base_token = order.base_token;
    let base_amount =
        u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;
    let base_denom =
        String::from_utf8(base_token.to_vec()).map_err(|_| ContractError::InvalidBaseToken)?;
    let mut messages = Vec::<CosmosMsg>::new();

    if base_amount > 0 {
        if !order.base_token_path.is_zero() {
            // If the token is from a different chain (wrapped token), mint it back
            messages.push(make_wasm_msg(
                WrappedTokenMsg::MintTokens {
                    denom: base_denom,
                    amount: base_amount.into(),
                    mint_to_address: sender,
                },
                minter,
                vec![],
            )?);
        } else {
            // If the token is native to this chain, unescrow it and decrease the channel balance
            decrease_channel_balance(
                deps,
                source_channel,
                path,
                base_denom.clone(),
                base_amount.into(),
            )?;

            messages.push(make_wasm_msg(
                LocalTokenMsg::Unescrow {
                    denom: base_denom,
                    recipient: sender.into_string(),
                    amount: base_amount.into(),
                },
                minter,
                vec![],
            )?);
        }
    }
    Ok(Response::new().add_messages(messages))
}

#[allow(clippy::too_many_arguments)]
fn acknowledge_fungible_asset_order(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    packet: Packet,
    _relayer: Addr,
    _salt: H256,
    path: U256,
    order: FungibleAssetOrder,
    order_ack: Option<FungibleAssetOrderAck>,
) -> Result<Response, ContractError> {
    match order_ack {
        Some(successful_ack) => {
            let mut messages = Vec::<CosmosMsg>::new();
            match successful_ack.fill_type {
                FILL_TYPE_PROTOCOL => {
                    // Protocol filled, fee was paid on destination to the relayer.
                }
                FILL_TYPE_MARKETMAKER => {
                    // A market maker filled, we pay (unescrow|mint) with the base asset.
                    let base_amount = u128::try_from(order.base_amount)
                        .map_err(|_| ContractError::AmountOverflow)?;
                    let market_maker = deps
                        .api
                        .addr_validate(
                            str::from_utf8(successful_ack.market_maker.as_ref())
                                .map_err(|_| ContractError::InvalidReceiver)?,
                        )
                        .map_err(|_| ContractError::UnableToValidateMarketMaker)?;
                    let minter = TOKEN_MINTER.load(deps.storage)?;
                    let base_denom = order.base_token;
                    let base_denom = String::from_utf8(base_denom.to_vec())
                        .map_err(|_| ContractError::InvalidBaseToken)?;

                    if !order.base_token_path.is_zero() {
                        // If the token is from a different chain (wrapped token), mint it
                        messages.push(make_wasm_msg(
                            WrappedTokenMsg::MintTokens {
                                denom: base_denom,
                                amount: base_amount.into(),
                                mint_to_address: market_maker,
                            },
                            minter,
                            vec![],
                        )?);
                    } else {
                        // If the token is native to this chain, decrease the channel balance and unescrow
                        decrease_channel_balance(
                            deps,
                            packet.source_channel_id,
                            path,
                            base_denom.clone(),
                            base_amount.into(),
                        )?;

                        messages.push(make_wasm_msg(
                            LocalTokenMsg::Unescrow {
                                denom: base_denom,
                                recipient: market_maker.into_string(),
                                amount: base_amount.into(),
                            },
                            minter,
                            vec![],
                        )?);
                    }
                }
                _ => return Err(StdError::generic_err("unknown fill_type, impossible?").into()),
            }
            Ok(Response::new().add_messages(messages))
        }
        // Transfer failed, refund
        None => refund(deps, path, packet.source_channel_id, order),
    }
}

#[allow(clippy::too_many_arguments)]
fn acknowledge_multiplex(
    deps: DepsMut,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    path: U256,
    multiplex: Multiplex,
    successful: bool,
    ack: Bytes,
) -> Result<Response, ContractError> {
    if successful && multiplex.eureka {
        // For eureka mode, forward the acknowledgement to the sender contract
        let sender_addr = deps
            .api
            .addr_validate(
                str::from_utf8(&multiplex.sender).map_err(|_| ContractError::InvalidSender)?,
            )
            .map_err(|_| ContractError::UnableToValidateSender)?;

        // Create a virtual packet with the multiplex data
        let multiplex_packet = Packet {
            source_channel_id: packet.source_channel_id,
            destination_channel_id: packet.destination_channel_id,
            data: encode_multiplex_calldata(
                path,
                multiplex.sender.into(),
                multiplex.contract_calldata.into(),
            )
            .into(),
            timeout_height: packet.timeout_height,
            timeout_timestamp: packet.timeout_timestamp,
        };

        // Forward the acknowledgement to the sender contract
        Ok(Response::new().add_message(wasm_execute(
            sender_addr,
            &IbcUnionMsg::OnAcknowledgementPacket {
                caller: caller.into(),
                packet: multiplex_packet,
                acknowledgement: ack,
                relayer: relayer.into(),
            },
            vec![],
        )?))
    } else {
        // For standard mode or failed eureka, no action needed
        Ok(Response::new())
    }
}

/// Executes an IBC packet by decoding and processing its contents.
/// This is the main entry point for packet execution that routes to specific execute functions
/// based on the instruction type.
#[allow(clippy::too_many_arguments)]
fn execute_packet(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    intent: bool,
) -> Result<Response, ContractError> {
    let mut funds = Coins::try_from(info.funds.clone()).expect("impossible");
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    execute_internal(
        deps.branch(),
        env,
        info,
        &mut funds,
        caller,
        packet,
        relayer,
        relayer_msg,
        zkgm_packet.salt.into(),
        zkgm_packet.path,
        zkgm_packet.instruction,
        intent,
    )
}

#[allow(clippy::too_many_arguments)]
/// Executes the internal logic for a packet based on its instruction type.
/// This is the core execution function that handles different instruction types:
/// - Fungible asset orders (transfers)
/// - Batch operations
/// - Multiplex operations
/// - Forward operations
fn execute_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    funds: &mut Coins,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    salt: H256,
    path: U256,
    instruction: Instruction,
    intent: bool,
) -> Result<Response, ContractError> {
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            if instruction.version != INSTR_VERSION_1 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            execute_fungible_asset_order(
                deps,
                env,
                info,
                funds,
                caller,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                order,
                intent,
            )
        }
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            execute_batch(
                deps,
                env,
                info,
                funds,
                caller,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                batch,
                intent,
            )
        }
        OP_MULTIPLEX => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            execute_multiplex(
                deps,
                env,
                caller,
                packet,
                relayer,
                relayer_msg,
                path,
                multiplex,
                intent,
            )
        }
        OP_FORWARD => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let forward = Forward::abi_decode_params(&instruction.operand, true)?;
            execute_forward(
                deps,
                env,
                info,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                forward,
                intent,
            )
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn predict_wrapped_token(
    deps: Deps,
    minter: &Addr,
    path: U256,
    channel_id: ChannelId,
    token: Bytes,
) -> StdResult<(String, Bytes)> {
    let wrapped_token = deps
        .querier
        .query::<ucs03_zkgm_token_minter_api::PredictWrappedTokenResponse>(&QueryRequest::Wasm(
            cosmwasm_std::WasmQuery::Smart {
                contract_addr: minter.to_string(),
                msg: to_json_binary(
                    &ucs03_zkgm_token_minter_api::QueryMsg::PredictWrappedToken {
                        path: path.to_string(),
                        channel_id,
                        token: Binary::new(token.to_vec()),
                    },
                )?,
            },
        ))?
        .wrapped_token;

    // Return both the string and bytes representation
    Ok((wrapped_token.clone(), wrapped_token.as_bytes().into()))
}

#[allow(clippy::too_many_arguments)]
fn execute_forward(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    packet: Packet,
    _relayer: Addr,
    _relayer_msg: Bytes,
    salt: H256,
    path: U256,
    forward: Forward,
    intent: bool,
) -> Result<Response, ContractError> {
    // We cannot allow market makers to fill packets containing forward
    // instruction. This would allow them to submit of a proof and fill via the
    // protocol on destination for a fake forward.

    // Instead, they must first fill on destination the orders, awaits finality
    // to settle the forward, then cascade acknowledge.
    if intent {
        return Err(ContractError::InvalidMarketMakerOperation);
    }

    let (tail_path, Some(previous_destination_channel_id)) =
        dequeue_channel_from_path(forward.path)
    else {
        return Err(ContractError::InvalidForwardDestinationChannelId {
            actual: None,
            expected: packet.destination_channel_id,
        });
    };

    let (continuation_path, Some(next_source_channel_id)) = dequeue_channel_from_path(tail_path)
    else {
        return Err(ContractError::MissingForwardSourceChannelId);
    };

    if packet.destination_channel_id != previous_destination_channel_id {
        return Err(ContractError::InvalidForwardDestinationChannelId {
            actual: Some(previous_destination_channel_id),
            expected: packet.destination_channel_id,
        });
    }

    let next_instruction = if continuation_path == U256::ZERO {
        // If we are done hopping, the sub-instruction is dispatched for execution
        forward.instruction
    } else {
        // If we are not done, the continuation path is used and the forward is re-executed
        Instruction {
            version: INSTR_VERSION_0,
            opcode: OP_FORWARD,
            operand: Forward {
                path: continuation_path,
                timeout_height: forward.timeout_height,
                timeout_timestamp: forward.timeout_timestamp,
                instruction: forward.instruction,
            }
            .abi_encode_params()
            .into(),
        }
    };

    let config = CONFIG.load(deps.storage)?;
    let next_path = update_channel_path(
        update_channel_path(path, previous_destination_channel_id)?,
        next_source_channel_id,
    )?;

    let next_packet = MsgSendPacket {
        source_channel_id: next_source_channel_id,
        timeout_height: forward.timeout_height,
        timeout_timestamp: Timestamp::from_nanos(forward.timeout_timestamp),
        data: ZkgmPacket {
            salt: derive_forward_salt(salt).into(),
            path: next_path,
            instruction: next_instruction,
        }
        .abi_encode_params()
        .into(),
    };

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            wasm_execute(
                &config.ibc_host,
                &ibc_union_msg::msg::ExecuteMsg::PacketSend(next_packet),
                vec![],
            )?,
            FORWARD_REPLY_ID,
        ))
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: Default::default(),
            },
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn execute_multiplex(
    deps: DepsMut,
    env: Env,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    path: U256,
    multiplex: Multiplex,
    intent: bool,
) -> Result<Response, ContractError> {
    let contract_address = deps
        .api
        .addr_validate(
            str::from_utf8(&multiplex.contract_address)
                .map_err(|_| ContractError::InvalidContractAddress)?,
        )
        .map_err(|_| ContractError::UnableToValidateMultiplexTarget)?;

    if multiplex.eureka {
        // Create a virtual packet with the multiplex data, consistent with ack and timeout handling
        let multiplex_packet = Packet {
            source_channel_id: packet.source_channel_id,
            destination_channel_id: packet.destination_channel_id,
            data: encode_multiplex_calldata(
                path,
                multiplex.sender.into(),
                multiplex.contract_calldata.into(),
            )
            .into(),
            timeout_height: packet.timeout_height,
            timeout_timestamp: packet.timeout_timestamp,
        };

        // Call the target contract with a reply to capture the acknowledgement
        let msg = if intent {
            IbcUnionMsg::OnIntentRecvPacket {
                caller: caller.into(),
                packet: multiplex_packet,
                market_maker: relayer.into(),
                market_maker_msg: relayer_msg,
            }
        } else {
            IbcUnionMsg::OnRecvPacket {
                caller: caller.into(),
                packet: multiplex_packet,
                relayer: relayer.into(),
                relayer_msg,
            }
        };
        Ok(Response::new().add_submessage(SubMsg::reply_on_success(
            wasm_execute(contract_address, &msg, vec![])?,
            MULTIPLEX_REPLY_ID,
        )))
    } else {
        // Standard mode - fire and forget
        let msg = if intent {
            ZkgmMsg::OnIntentZkgm {
                caller,
                path: Uint256::from_be_bytes(path.to_be_bytes()),
                source_channel_id: packet.source_channel_id,
                destination_channel_id: packet.destination_channel_id,
                sender: multiplex.sender.to_vec().into(),
                message: multiplex.contract_calldata.to_vec().into(),
                market_maker: relayer,
                market_maker_msg: relayer_msg,
            }
        } else {
            ZkgmMsg::OnZkgm {
                caller,
                path: Uint256::from_be_bytes(path.to_be_bytes()),
                source_channel_id: packet.source_channel_id,
                destination_channel_id: packet.destination_channel_id,
                sender: multiplex.sender.to_vec().into(),
                message: multiplex.contract_calldata.to_vec().into(),
                relayer,
                relayer_msg,
            }
        };
        Ok(Response::new()
            .add_message(wasm_execute(contract_address, &msg, vec![])?)
            .add_message(wasm_execute(
                env.contract.address,
                &ExecuteMsg::InternalWriteAck {
                    ack: TAG_ACK_SUCCESS.abi_encode().into(),
                },
                vec![],
            )?))
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_batch(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    funds: &mut Coins,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    salt: H256,
    path: U256,
    batch: Batch,
    intent: bool,
) -> Result<Response, ContractError> {
    EXECUTING_PACKET_IS_BATCH.save(deps.storage, &batch.instructions.len())?;
    let mut response = Response::new();
    for (i, instruction) in batch.instructions.into_iter().enumerate() {
        let sub_response = execute_internal(
            deps.branch(),
            env.clone(),
            info.clone(),
            funds,
            caller.clone(),
            packet.clone(),
            relayer.clone(),
            relayer_msg.clone(),
            derive_batch_salt(i.try_into().unwrap(), salt),
            path,
            instruction,
            intent,
        )?;
        response = response
            .add_attributes(sub_response.attributes)
            .add_events(sub_response.events)
            .add_submessages(sub_response.messages);
    }
    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn market_maker_fill(
    deps: DepsMut,
    env: Env,
    funds: &mut Coins,
    caller: Addr,
    relayer_msg: Bytes,
    quote_amount: u128,
    quote_token_str: String,
    minter: Addr,
    receiver: Addr,
) -> Result<Response, ContractError> {
    /* Gas Station

     Determine the native denom that we transfer to the minter contract.
     The MM may fill multiple order within the packet, we need to
     provide each token individually, subtracting from the total funds.
    */
    let mut funds_to_escrow = vec![];
    if !funds.amount_of(&quote_token_str).is_zero() {
        let native_denom = Coin {
            denom: quote_token_str.clone(),
            amount: quote_amount.into(),
        };
        funds.sub(native_denom.clone())?;
        funds_to_escrow.push(native_denom);
    }
    MARKET_MAKER.save(deps.storage, &relayer_msg)?;
    let mut messages = Vec::with_capacity(2);
    if quote_amount > 0 {
        // Make sure the market maker provide the funds
        messages.push(make_wasm_msg(
            LocalTokenMsg::Escrow {
                from: caller.to_string(),
                denom: quote_token_str.clone(),
                recipient: minter.to_string(),
                amount: quote_amount.into(),
            },
            &minter,
            funds_to_escrow,
        )?);
        // Release the funds to the user
        messages.push(make_wasm_msg(
            LocalTokenMsg::Unescrow {
                denom: quote_token_str,
                recipient: receiver.to_string(),
                amount: quote_amount.into(),
            },
            minter,
            vec![],
        )?);
    }
    Ok(Response::new().add_submessage(SubMsg::reply_always(
        wasm_execute(
            &env.contract.address,
            &ExecuteMsg::InternalBatch { messages },
            vec![],
        )?,
        MM_FILL_REPLY_ID,
    )))
}

#[allow(clippy::too_many_arguments)]
fn execute_fungible_asset_order(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    funds: &mut Coins,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    _salt: H256,
    path: U256,
    order: FungibleAssetOrder,
    intent: bool,
) -> Result<Response, ContractError> {
    // Get the token minter contract
    let minter = TOKEN_MINTER.load(deps.storage)?;

    // Convert quote token to string for comparison
    let quote_token_str = String::from_utf8(Vec::from(order.quote_token.clone()))
        .map_err(|_| ContractError::InvalidQuoteToken)?;

    // Validate the receiver address
    let receiver = deps
        .api
        .addr_validate(
            str::from_utf8(order.receiver.as_ref()).map_err(|_| ContractError::InvalidReceiver)?,
        )
        .map_err(|_| ContractError::UnableToValidateReceiver)?;

    // Calculate amounts and fee
    let base_amount =
        u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;
    let quote_amount =
        u128::try_from(order.quote_amount).map_err(|_| ContractError::AmountOverflow)?;

    if intent {
        return market_maker_fill(
            deps,
            env.clone(),
            funds,
            caller,
            relayer_msg,
            quote_amount,
            quote_token_str,
            minter,
            receiver,
        );
    }

    let base_covers_quote = base_amount >= quote_amount;
    let fee_amount = base_amount.saturating_sub(quote_amount);

    // Predict the wrapped token denom based on path, destination channel, and base token
    let (wrapped_denom, _) = predict_wrapped_token(
        deps.as_ref(),
        &minter,
        path,
        packet.destination_channel_id,
        Vec::from(order.base_token.clone()).into(),
    )?;

    let mut messages = Vec::new();

    // Protocol Fill - If the quote token matches the wrapped version of the base token and base amount >= quote amount
    if quote_token_str == wrapped_denom && base_covers_quote {
        // Ensure rate limit is respected
        rate_limit(
            deps.storage,
            wrapped_denom.clone(),
            quote_amount,
            env.block.time.seconds(),
        )?;

        // For new assets: Deploy wrapped token contract and mint quote amount to receiver
        if !HASH_TO_FOREIGN_TOKEN.has(deps.storage, wrapped_denom.clone()) {
            // Create the wrapped token if it doesn't exist
            HASH_TO_FOREIGN_TOKEN.save(
                deps.storage,
                wrapped_denom.clone(),
                &Vec::from(order.base_token.clone()).into(),
            )?;

            // Create the token with metadata
            messages.push(SubMsg::reply_never(make_wasm_msg(
                WrappedTokenMsg::CreateDenom {
                    subdenom: wrapped_denom.clone(),
                    metadata: Metadata {
                        name: order.base_token_name,
                        symbol: order.base_token_symbol,
                        decimals: order.base_token_decimals,
                    },
                    path: path.to_be_bytes_vec().into(),
                    channel_id: packet.destination_channel_id,
                    token: Vec::from(order.base_token.clone()).into(),
                },
                &minter,
                vec![],
            )?));

            // Save the token origin for future unwrapping
            TOKEN_ORIGIN.save(
                deps.storage,
                wrapped_denom.clone(),
                &Uint256::from_be_bytes(
                    update_channel_path(path, packet.destination_channel_id)?.to_be_bytes(),
                ),
            )?;
        }

        // Mint the quote amount to the receiver
        if quote_amount > 0 {
            messages.push(SubMsg::reply_never(make_wasm_msg(
                WrappedTokenMsg::MintTokens {
                    denom: wrapped_denom.clone(),
                    amount: quote_amount.into(),
                    mint_to_address: receiver,
                },
                &minter,
                vec![],
            )?));
        }

        // Mint any fee to the relayer
        if fee_amount > 0 {
            messages.push(SubMsg::reply_never(make_wasm_msg(
                WrappedTokenMsg::MintTokens {
                    denom: wrapped_denom,
                    amount: fee_amount.into(),
                    mint_to_address: relayer,
                },
                &minter,
                vec![],
            )?));
        }
    }
    // Unwrapping - For returning assets: Unwrap base token and transfer quote amount to receiver
    else if order.base_token_path != U256::ZERO && base_covers_quote {
        // Ensure rate limit is respected
        rate_limit(
            deps.storage,
            quote_token_str.clone(),
            quote_amount,
            env.block.time.seconds(),
        )?;

        // Decrease the outstanding balance for this channel and path
        decrease_channel_balance(
            deps,
            packet.destination_channel_id,
            reverse_channel_path(path)?,
            quote_token_str.clone(),
            base_amount.into(),
        )?;

        // Transfer the quote amount to the receiver
        if quote_amount > 0 {
            messages.push(SubMsg::reply_never(make_wasm_msg(
                LocalTokenMsg::Unescrow {
                    denom: quote_token_str.clone(),
                    recipient: receiver.into_string(),
                    amount: quote_amount.into(),
                },
                &minter,
                vec![],
            )?));
        }

        // Transfer any fee to the relayer
        if fee_amount > 0 {
            messages.push(SubMsg::reply_never(make_wasm_msg(
                LocalTokenMsg::Unescrow {
                    denom: quote_token_str,
                    recipient: relayer.into_string(),
                    amount: fee_amount.into(),
                },
                &minter,
                vec![],
            )?));
        }
    }
    // Market Maker Fill - Any party can fill the order by providing the quote token
    else {
        return market_maker_fill(
            deps,
            env.clone(),
            funds,
            caller,
            relayer_msg,
            quote_amount,
            quote_token_str,
            minter,
            receiver,
        );
    }

    // Return success acknowledgement with protocol fill type
    Ok(Response::new()
        .add_submessages(messages)
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: FungibleAssetOrderAck {
                    fill_type: FILL_TYPE_PROTOCOL,
                    market_maker: Default::default(),
                }
                .abi_encode_params()
                .into(),
            },
            vec![],
        )?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        // This reply is triggered after we execute a packet.
        // We need to handle this reply to process the acknowledgement that was generated during
        // packet execution. This is a critical part of the IBC protocol - after a packet is
        // executed, we must write an acknowledgement back to the source chain. This reply
        // allows us to collect the acknowledgement data (which may come from batch instructions
        // or a single instruction) and format it properly before sending it back through the
        // IBC host contract.
        EXECUTE_REPLY_ID => {
            let ibc_host = CONFIG.load(deps.storage)?.ibc_host;
            let packet = EXECUTING_PACKET.load(deps.storage)?;
            EXECUTING_PACKET.remove(deps.storage);
            match reply.result {
                SubMsgResult::Ok(_) => {
                    // If the execution succedeed one of the acks is guaranteed to exist.
                    let execution_ack = (|| -> Result<Bytes, ContractError> {
                        match EXECUTING_PACKET_IS_BATCH.may_load(deps.storage)? {
                            Some(expected_acks) => {
                                EXECUTING_PACKET_IS_BATCH.remove(deps.storage);
                                let acks = BATCH_EXECUTION_ACKS.load(deps.storage)?;
                                BATCH_EXECUTION_ACKS.remove(deps.storage);
                                // Ensure all acknowledgements has been written
                                // This is guaranteed because allowed
                                // instructions are always yielding an ack in
                                // this case (multiplex/fungibleAssetOrder). We keep this assertion for future upgrades.
                                if acks.len() != expected_acks {
                                    return Err(ContractError::BatchMustBeSync);
                                }
                                for ack in &acks {
                                    if ack.is_empty() {
                                        // This be guaranteed as well.
                                        return Err(ContractError::BatchMustBeSync);
                                    } else if ack == ACK_ERR_ONLY_MAKER {
                                        // Ensure we don't ask for a revert (onlyMaker).
                                        return Err(ContractError::OnlyMaker);
                                    }
                                }
                                Ok(BatchAck {
                                    acknowledgements: acks
                                        .into_iter()
                                        .map(Into::into)
                                        .collect::<Vec<_>>(),
                                }
                                .abi_encode_params()
                                .into())
                            }
                            None => {
                                let ack = EXECUTION_ACK.load(deps.storage)?;
                                EXECUTION_ACK.remove(deps.storage);
                                if ack == ACK_ERR_ONLY_MAKER {
                                    return Err(ContractError::OnlyMaker);
                                }
                                Ok(ack)
                            }
                        }
                    })()?;
                    if !execution_ack.is_empty() {
                        let zkgm_ack = Ack {
                            tag: TAG_ACK_SUCCESS,
                            inner_ack: Vec::from(execution_ack).into(),
                        }
                        .abi_encode_params();
                        Ok(Response::new().add_message(wasm_execute(
                            &ibc_host,
                            &ibc_union_msg::msg::ExecuteMsg::WriteAcknowledgement(
                                MsgWriteAcknowledgement {
                                    packet,
                                    acknowledgement: zkgm_ack.into(),
                                },
                            ),
                            vec![],
                        )?))
                    } else {
                        // Async acknowledgement, we don't write anything
                        Ok(Response::new())
                    }
                }
                // Something went horribly wrong.
                SubMsgResult::Err(e) => {
                    let zkgm_ack = Ack {
                        tag: TAG_ACK_FAILURE,
                        inner_ack: Default::default(),
                    }
                    .abi_encode_params();
                    Ok(Response::new()
                        .add_attribute("fatal_error", to_json_string(&e)?)
                        .add_message(wasm_execute(
                            &ibc_host,
                            &ibc_union_msg::msg::ExecuteMsg::WriteAcknowledgement(
                                MsgWriteAcknowledgement {
                                    packet,
                                    acknowledgement: zkgm_ack.into(),
                                },
                            ),
                            vec![],
                        )?))
                }
            }
        }
        // This reply is triggered after we forward a packet to another chain.
        // We need to handle this reply to store the sent packet in our in-flight packet mapping.
        // This is crucial for the forwarding mechanism - when the forwarded packet is acknowledged
        // or times out, we need to be able to find the original packet that initiated the forward
        // so we can properly propagate the acknowledgement or timeout back to the source chain.
        // Without this reply handling, we would lose track of forwarded packets.
        FORWARD_REPLY_ID => {
            if let SubMsgResult::Ok(reply_data) = reply.result {
                let sent_packet = serde_json_wasm::from_slice::<Packet>(
                    #[allow(deprecated)]
                    reply_data.data.clone().unwrap_or_default().as_slice(),
                )
                .map_err(|error| ContractError::CouldNotDeserializeSentPacket {
                    error,
                    #[allow(deprecated)]
                    sent_packet_data: Vec::from(reply_data.data.unwrap_or_default()).into(),
                })?;
                let commitment_key = BatchPacketsPath::from_packets(&[sent_packet.clone()]).key();
                IN_FLIGHT_PACKET.save(
                    deps.storage,
                    commitment_key.into_bytes().into(),
                    &sent_packet,
                )?;
                Ok(Response::new())
            } else {
                Err(ContractError::ForwardedPacketMissingInReply)
            }
        }
        // This reply is triggered after we call a contract in eureka mode.
        // We need to handle this reply to capture the acknowledgement returned by the target
        // contract. In the eureka mode of multiplex operations, the target contract must return
        // an acknowledgement that needs to be forwarded back to the source chain. This reply
        // allows us to capture that acknowledgement and store it so it can be included in the
        // packet acknowledgement. Without this reply, we wouldn't be able to support the
        // eureka mode of multiplex operations.
        MULTIPLEX_REPLY_ID => {
            // Extract the acknowledgement from the reply
            match reply.result {
                SubMsgResult::Ok(reply_data) => {
                    #[allow(deprecated)]
                    let acknowledgement = reply_data.data.unwrap_or_default();

                    // If the acknowledgement is empty, we can't proceed
                    if acknowledgement.is_empty() {
                        return Err(ContractError::AsyncMultiplexUnsupported);
                    }

                    Ok(Response::new().add_message(wasm_execute(
                        env.contract.address,
                        &ExecuteMsg::InternalWriteAck {
                            ack: Vec::from(acknowledgement).into(),
                        },
                        vec![],
                    )?))
                }
                SubMsgResult::Err(error) => Err(ContractError::MultiplexError { error }),
            }
        }
        MM_FILL_REPLY_ID => {
            let market_maker = MARKET_MAKER.load(deps.storage)?;
            MARKET_MAKER.remove(deps.storage);
            match reply.result {
                SubMsgResult::Ok(_) => Ok(Response::new().add_message(wasm_execute(
                    env.contract.address,
                    &ExecuteMsg::InternalWriteAck {
                        ack: FungibleAssetOrderAck {
                            fill_type: FILL_TYPE_MARKETMAKER,
                            market_maker: Vec::from(market_maker).into(),
                        }
                        .abi_encode_params()
                        .into(),
                    },
                    vec![],
                )?)),
                // Leave a chance for another MM to fill by telling the top level handler to revert.
                SubMsgResult::Err(error) => Ok(Response::new()
                    .add_attribute("maker_execution_failure", error)
                    .add_message(wasm_execute(
                        env.contract.address,
                        &ExecuteMsg::InternalWriteAck {
                            ack: ACK_ERR_ONLY_MAKER.into(),
                        },
                        vec![],
                    )?)),
            }
        }
        // For any other reply ID, we don't know how to handle it, so we return an error.
        // This is a safety measure to ensure we don't silently ignore unexpected replies,
        // which could indicate a bug in the contract or an attempt to exploit it.
        _ => Err(ContractError::UnknownReply { id: reply.id }),
    }
}

/// Verifies that an instruction is valid before execution.
/// This is the main entry point for instruction validation that routes to specific verify functions
/// based on the instruction opcode.
pub fn verify_internal(
    deps: DepsMut,
    info: MessageInfo,
    funds: &mut Coins,
    channel_id: ChannelId,
    path: U256,
    instruction: &Instruction,
    response: &mut Response,
) -> Result<(), ContractError> {
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            if instruction.version != INSTR_VERSION_1 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            verify_fungible_asset_order(deps, info, funds, channel_id, path, &order, response)
        }
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            verify_batch(deps, info, funds, channel_id, path, &batch, response)
        }
        OP_FORWARD => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let forward = Forward::abi_decode_params(&instruction.operand, true)?;
            verify_forward(deps, info, funds, channel_id, &forward, response)
        }
        OP_MULTIPLEX => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            verify_multiplex(&multiplex, info.sender, response)
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

/// Verifies a fungible asset order instruction.
/// Checks token metadata matches and validates unwrapping conditions by comparing
/// the token origin path with the current path and channel.
pub fn verify_fungible_asset_order(
    deps: DepsMut,
    info: MessageInfo,
    funds: &mut Coins,
    channel_id: ChannelId,
    path: U256,
    order: &FungibleAssetOrder,
    response: &mut Response,
) -> Result<(), ContractError> {
    // Validate and get base token address
    let base_token_str =
        str::from_utf8(&order.base_token).map_err(|_| ContractError::InvalidBaseToken)?;

    // Query token metadata from the minter
    let minter = TOKEN_MINTER.load(deps.storage)?;
    let metadata = deps.querier.query::<MetadataResponse>(&QueryRequest::Wasm(
        cosmwasm_std::WasmQuery::Smart {
            contract_addr: minter.to_string(),
            msg: to_json_binary(&ucs03_zkgm_token_minter_api::QueryMsg::Metadata {
                denom: base_token_str.to_string(),
            })?,
        },
    ))?;

    // Verify metadata matches
    if metadata.name != order.base_token_name {
        return Err(ContractError::InvalidAssetName {
            expected: metadata.name,
            found: order.base_token_name.clone(),
        });
    }
    if metadata.symbol != order.base_token_symbol {
        return Err(ContractError::InvalidAssetSymbol {
            expected: metadata.symbol,
            found: order.base_token_symbol.clone(),
        });
    }
    if metadata.decimals != order.base_token_decimals {
        return Err(ContractError::InvalidAssetDecimals {
            expected: metadata.decimals,
            found: order.base_token_decimals,
        });
    }

    // Get the origin path for the base token
    let origin = TOKEN_ORIGIN.may_load(deps.storage, base_token_str.to_string())?;

    // Compute the wrapped token from the destination to source
    let (wrapped_token, _) = predict_wrapped_token(
        deps.as_ref(),
        &minter,
        path,
        channel_id,
        order.quote_token.to_vec().into(),
    )?;

    // Check if base token matches predicted wrapper
    let is_unwrapping = base_token_str == wrapped_token;

    // Get the intermediate path and destination channel from origin
    let (origin, intermediate_path, destination_channel_id) = if let Some(origin) = origin {
        let origin_u256 = U256::from_be_bytes(origin.to_be_bytes());
        let (intermediate_path, destination_channel_id) = pop_channel_from_path(origin_u256);
        (origin_u256, intermediate_path, destination_channel_id)
    } else {
        (U256::ZERO, U256::ZERO, None)
    };

    // Check if we're taking same path starting from same channel using wrapped asset
    let is_inverse_intermediate_path = path == reverse_channel_path(intermediate_path)?;
    let is_sending_back_to_same_channel = destination_channel_id == Some(channel_id);

    if is_inverse_intermediate_path && is_sending_back_to_same_channel && is_unwrapping {
        // Verify the origin path matches what's in the order
        if origin != order.base_token_path {
            return Err(ContractError::InvalidAssetOrigin {
                actual: order.base_token_path,
                expected: origin,
            });
        }
        // Burn tokens as we are going to unescrow on the counterparty
        *response = response.clone().add_message(make_wasm_msg(
            WrappedTokenMsg::BurnTokens {
                denom: base_token_str.to_string(),
                amount: Uint256::from_be_bytes(order.base_amount.to_be_bytes())
                    .try_into()
                    .map_err(|_| ContractError::AmountOverflow)?,
                burn_from_address: minter.clone(),
                sender: info.sender,
            },
            &minter,
            vec![],
        )?);
    } else {
        if !order.base_token_path.is_zero() {
            return Err(ContractError::InvalidAssetOrigin {
                actual: order.base_token_path,
                expected: U256::ZERO,
            });
        }
        // Escrow tokens as the counterparty will mint them
        let base_amount = Uint256::from_be_bytes(order.base_amount.to_be_bytes());
        increase_channel_balance(
            deps.storage,
            channel_id,
            path,
            base_token_str.to_string(),
            base_amount,
        )?;
        let mut funds_to_escrow = vec![];
        if !funds.amount_of(base_token_str).is_zero() {
            let native_denom = Coin {
                denom: base_token_str.to_string(),
                amount: base_amount
                    .try_into()
                    .map_err(|_| ContractError::AmountOverflow)?,
            };
            funds.sub(native_denom.clone())?;
            funds_to_escrow.push(native_denom);
        }
        *response = response.clone().add_message(make_wasm_msg(
            LocalTokenMsg::Escrow {
                from: info.sender.to_string(),
                denom: base_token_str.to_string(),
                recipient: minter.to_string(),
                amount: base_amount
                    .try_into()
                    .map_err(|_| ContractError::AmountOverflow)?,
            },
            &minter,
            funds_to_escrow,
        )?);
    }

    Ok(())
}

/// Verifies a batch instruction by checking each sub-instruction is allowed and valid.
/// Only certain instruction types are allowed in batches to prevent complex nested operations.
pub fn verify_batch(
    mut deps: DepsMut,
    info: MessageInfo,
    funds: &mut Coins,
    channel_id: ChannelId,
    path: U256,
    batch: &Batch,
    response: &mut Response,
) -> Result<(), ContractError> {
    for instruction in &batch.instructions {
        if !is_allowed_batch_instruction(instruction.opcode) {
            return Err(ContractError::InvalidBatchInstruction);
        }
        verify_internal(
            deps.branch(),
            info.clone(),
            funds,
            channel_id,
            path,
            instruction,
            response,
        )?;
    }
    Ok(())
}

/// Verifies a forward instruction by checking the sub-instruction is allowed and valid.
/// Forward instructions can contain batch, multiplex or fungible asset orders.
pub fn verify_forward(
    deps: DepsMut,
    info: MessageInfo,
    funds: &mut Coins,
    channel_id: ChannelId,
    forward: &Forward,
    response: &mut Response,
) -> Result<(), ContractError> {
    if !is_allowed_forward_instruction(forward.instruction.opcode) {
        return Err(ContractError::InvalidForwardInstruction);
    }

    // Verify the sub-instruction
    verify_internal(
        deps,
        info,
        funds,
        channel_id,
        forward.path,
        &forward.instruction,
        response,
    )?;

    Ok(())
}

/// Verifies a multiplex instruction by checking the sender matches the transaction sender.
/// This prevents unauthorized parties from impersonating others in multiplex operations.
pub fn verify_multiplex(
    multiplex: &Multiplex,
    sender: Addr,
    _response: &mut Response,
) -> Result<(), ContractError> {
    // Verify the sender matches msg.sender
    let multiplex_sender =
        str::from_utf8(&multiplex.sender).map_err(|_| ContractError::InvalidSender)?;
    if multiplex_sender != sender.as_str() {
        return Err(ContractError::InvalidMultiplexSender);
    }
    Ok(())
}

/// Checks if an opcode is allowed in a batch instruction
fn is_allowed_batch_instruction(opcode: u8) -> bool {
    opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER
}

/// Checks if an opcode is allowed in a forward instruction
fn is_allowed_forward_instruction(opcode: u8) -> bool {
    opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER || opcode == OP_BATCH
}

#[allow(clippy::too_many_arguments)]
pub fn send(
    mut deps: DepsMut,
    info: MessageInfo,
    channel_id: ChannelId,
    timeout_height: u64,
    timeout_timestamp: Timestamp,
    salt: H256,
    instruction: Instruction,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    let mut funds = Coins::try_from(info.funds.clone()).expect("impossible");
    // Verify the instruction
    verify_internal(
        deps.branch(),
        info.clone(),
        &mut funds,
        channel_id,
        U256::ZERO,
        &instruction,
        &mut response,
    )?;

    // Hash the salt with the sender to prevent collision between users.
    let hashed_salt = keccak256((info.sender.as_bytes(), salt).abi_encode());

    let config = CONFIG.load(deps.storage)?;
    Ok(response.add_message(wasm_execute(
        &config.ibc_host,
        &ibc_union_msg::msg::ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel_id: channel_id,
            timeout_height,
            timeout_timestamp,
            data: ZkgmPacket {
                salt: hashed_salt.into(),
                path: U256::ZERO,
                instruction,
            }
            .abi_encode_params()
            .into(),
        }),
        vec![],
    )?))
}

#[cosmwasm_schema::cw_serde]
pub struct TokenMinterMigration {
    // code id of the new token minter
    new_code_id: u64,
    // migrate message json that will directly be passed to migrate call
    // it will be the same as the `to_json_binary(&msg)`'s output
    msg: Binary,
}

// The current structure is expected to be backward compatible, only idempotent
// fields can be currently added.
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {
    // Provide `token_minter_migration` to also migrate the token minter
    token_minter_migration: Option<TokenMinterMigration>,
    // Whether to enable or disable rate limiting while migrating.
    rate_limit_disabled: bool,
}

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
        |deps, migrate_msg, _current_version| {
            CONFIG.update::<_, ContractError>(deps.storage, |mut config| {
                config.rate_limit_disabled = migrate_msg.rate_limit_disabled;
                Ok(config)
            })?;
            if let Some(token_minter_migration) = migrate_msg.token_minter_migration {
                let token_minter = TOKEN_MINTER.load(deps.storage)?;
                Ok((
                    Response::default().add_message(WasmMsg::Migrate {
                        contract_addr: token_minter.to_string(),
                        new_code_id: token_minter_migration.new_code_id,
                        msg: token_minter_migration.msg,
                    }),
                    None,
                ))
            } else {
                Ok((Response::default(), None))
            }
        },
    )
}

/// Creates a WasmMsg for interacting with the token minter contract.
/// This is a helper function to construct properly formatted wasm messages
/// for token minting, burning, and other token operations.
fn make_wasm_msg(
    msg: impl Into<ucs03_zkgm_token_minter_api::ExecuteMsg>,
    minter: impl Into<String>,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let msg = msg.into();
    Ok(CosmosMsg::Wasm(wasm_execute(minter, &msg, funds)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::PredictWrappedToken {
            path,
            channel_id,
            token,
        } => {
            let minter = TOKEN_MINTER.load(deps.storage)?;
            let (token, _) = predict_wrapped_token(
                deps,
                &minter,
                path.parse().map_err(ContractError::InvalidPath)?,
                channel_id,
                token,
            )?;
            Ok(to_json_binary(&PredictWrappedTokenResponse {
                wrapped_token: token.as_bytes().into(),
            })?)
        }
        QueryMsg::GetMinter {} => {
            let minter = TOKEN_MINTER.load(deps.storage)?;
            Ok(to_json_binary(&minter)?)
        }
        QueryMsg::GetTokenBucket { denom } => {
            let bucket = TOKEN_BUCKET.load(deps.storage, denom)?;
            Ok(to_json_binary(&bucket)?)
        }
        QueryMsg::GetChannelBalance {
            channel_id,
            path,
            denom,
        } => {
            let balance = CHANNEL_BALANCE.load(
                deps.storage,
                (channel_id.get().into(), path.to_be_bytes().into(), denom),
            )?;
            Ok(to_json_binary(&balance)?)
        }
        QueryMsg::GetConfig {} => {
            let config = CONFIG.load(deps.storage)?;
            Ok(to_json_binary(&config)?)
        }
    }
}

fn rate_limit(
    storage: &mut dyn Storage,
    denom: String,
    amount: impl Into<Uint256>,
    now: impl Into<Uint256>,
) -> Result<(), ContractError> {
    if CONFIG.load(storage)?.rate_limit_disabled {
        return Ok(());
    }
    TOKEN_BUCKET.update(storage, denom.clone(), |entry| match entry {
        Some(mut token_bucket) => {
            token_bucket.rate_limit(amount.into(), now.into())?;
            Ok(token_bucket)
        }
        None => Err(ContractError::TokenBucketIsAbsent {
            token: denom.clone(),
        }),
    })?;
    Ok(())
}

/// Increases the outstanding balance for a (channel, path, token) combination.
/// This is used when escrowing tokens to track how many tokens can be unescrowed later.
/// The balance is used to prevent double-spending and ensure token conservation across chains.
pub fn increase_channel_balance(
    storage: &mut dyn Storage,
    channel_id: ChannelId,
    path: U256,
    base_token: String,
    base_amount: Uint256,
) -> Result<(), ContractError> {
    CHANNEL_BALANCE.update(
        storage,
        (
            channel_id.raw(),
            path.to_be_bytes::<32>().to_vec(),
            base_token,
        ),
        |balance| match balance {
            Some(value) => value
                .checked_add(base_amount)
                .map_err(|_| ContractError::InvalidChannelBalance),
            None => Ok(base_amount),
        },
    )?;
    Ok(())
}

/// Decrease the outstanding balance of a (channel, path, token) combination.
/// This is used when unwrapping tokens to ensure we don't unescrow more tokens than were originally escrowed.
fn decrease_channel_balance(
    deps: DepsMut,
    channel_id: ChannelId,
    path: U256,
    token: String,
    amount: Uint256,
) -> Result<(), ContractError> {
    CHANNEL_BALANCE.update(
        deps.storage,
        (channel_id.raw(), path.to_be_bytes::<32>().to_vec(), token),
        |balance| match balance {
            Some(value) => value
                .checked_sub(amount)
                .map_err(|_| ContractError::InvalidChannelBalance),
            None => Err(ContractError::InvalidChannelBalance),
        },
    )?;
    Ok(())
}

pub fn derive_batch_salt(index: U256, salt: H256) -> H256 {
    keccak256((index, salt.get()).abi_encode())
}

pub fn dequeue_channel_from_path(path: U256) -> (U256, Option<ChannelId>) {
    if path == U256::ZERO {
        (U256::ZERO, None)
    } else {
        (
            path >> 32,
            // expect/Some is intentional, such that any bugs are loud and not silently accepted
            Some(
                ChannelId::from_raw(
                    u32::try_from(path & U256::from(u32::MAX)).expect("impossible"),
                )
                .expect("value is > 0 as path is > 0; qed;"),
            ),
        )
    }
}

/// Extract the last channel from a path and return the base path without it
pub fn pop_channel_from_path(path: U256) -> (U256, Option<ChannelId>) {
    if path == U256::ZERO {
        return (U256::ZERO, None);
    }
    // Find the highest non-zero 32-bit chunk (leftmost)
    let highest_index = (256 - path.leading_zeros() - 1) / 32;
    // Extract the channel ID from the highest non-zero slot
    let channel_id = get_channel_from_path(path, highest_index);
    // Clear that slot in the path
    let mask = !(U256::from(u32::MAX) << (highest_index * 32));
    let base_path = path & mask;
    (base_path, channel_id)
}

pub fn update_channel_path(path: U256, next_channel_id: ChannelId) -> Result<U256, ContractError> {
    if path == U256::ZERO {
        Ok(U256::from(next_channel_id.raw()))
    } else {
        let next_hop_index = (256 - path.leading_zeros()) / 32 + 1;
        if next_hop_index > 7 {
            return Err(ContractError::ChannelPathIsFull {
                path,
                next_hop_index,
            });
        }
        Ok(make_path_from_channel(next_channel_id, next_hop_index) | path)
    }
}

pub fn get_channel_from_path(path: U256, index: usize) -> Option<ChannelId> {
    ChannelId::from_raw(
        u32::try_from((path >> (32 * index)) & U256::from(u32::MAX)).expect("impossible"),
    )
}

pub fn make_path_from_channel(channel_id: ChannelId, index: usize) -> U256 {
    U256::from(channel_id.raw()) << (32 * index)
}

pub fn reverse_channel_path(mut path: U256) -> Result<U256, ContractError> {
    let mut reversed_channel_path = U256::ZERO;
    loop {
        let (tail, head) = pop_channel_from_path(path);
        if let Some(channel_id) = head {
            reversed_channel_path = update_channel_path(reversed_channel_path, channel_id)?;
        }
        if tail == U256::ZERO {
            break;
        }
        path = tail;
    }
    Ok(reversed_channel_path)
}

pub fn tint_forward_salt(salt: H256) -> H256 {
    (FORWARD_SALT_MAGIC | (U256::from_be_bytes(*salt.get()) & !FORWARD_SALT_MAGIC))
        .to_be_bytes()
        .into()
}

pub fn is_forwarded_packet(salt: H256) -> bool {
    (U256::from_be_bytes(*salt.get()) & FORWARD_SALT_MAGIC) == FORWARD_SALT_MAGIC
}

pub fn derive_forward_salt(salt: H256) -> H256 {
    tint_forward_salt(keccak256(salt.abi_encode()))
}

pub fn encode_multiplex_calldata(path: U256, sender: Bytes, contract_calldata: Bytes) -> Vec<u8> {
    (path, sender, contract_calldata).abi_encode()
}
