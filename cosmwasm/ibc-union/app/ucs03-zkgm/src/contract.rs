use core::str;

use alloy::{primitives::U256, sol_types::SolValue};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, to_json_string, wasm_execute, Addr, Binary,
    CodeInfoResponse, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply,
    Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128, Uint256, WasmMsg,
};
use ibc_union_msg::{
    module::IbcUnionMsg,
    msg::{MsgSendPacket, MsgWriteAcknowledgement},
};
use ibc_union_spec::{path::BatchPacketsPath, types::Packet};
use ucs03_zkgm_token_minter_api::{
    LocalTokenMsg, Metadata, MetadataResponse, WrappedTokenMsg, DISPATCH_EVENT, DISPATCH_EVENT_ATTR,
};
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bytes, H256},
};
use unionlabs_cosmwasm_upgradable::UpgradeMsg;

use crate::{
    com::{
        Ack, Batch, BatchAck, Forward, FungibleAssetOrder, FungibleAssetOrderAck, Instruction,
        Multiplex, ZkgmPacket, ACK_ERR_ONLY_MAKER, FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL,
        FORWARD_SALT_MAGIC, INSTR_VERSION_0, OP_BATCH, OP_FORWARD, OP_FUNGIBLE_ASSET_ORDER,
        OP_MULTIPLEX, TAG_ACK_FAILURE, TAG_ACK_SUCCESS,
    },
    msg::{EurekaMsg, ExecuteMsg, InitMsg, PredictWrappedTokenResponse, QueryMsg},
    state::{
        BATCH_EXECUTION_ACKS, CHANNEL_BALANCE, CONFIG, EXECUTING_PACKET, EXECUTING_PACKET_IS_BATCH,
        EXECUTION_ACK, HASH_TO_FOREIGN_TOKEN, IN_FLIGHT_PACKET, TOKEN_MINTER, TOKEN_ORIGIN,
    },
    ContractError,
};

pub const PROTOCOL_VERSION: &str = "ucs03-zkgm-0";

pub const EXECUTE_REPLY_ID: u64 = 0x1337;
pub const TOKEN_INIT_REPLY_ID: u64 = 0xbeef;
pub const ESCROW_REPLY_ID: u64 = 0xcafe;
pub const FORWARD_REPLY_ID: u64 = 0xbabe;
pub const MULTIPLEX_REPLY_ID: u64 = 0xface;

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
        msg: to_json_binary(&msg.minter_init_msg)?,
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
                } => {
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
                                },
                                vec![],
                            )?,
                            EXECUTE_REPLY_ID,
                        )))
                    }
                }
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
                x => Err(
                    StdError::generic_err(format!("not handled: {}", to_json_string(&x)?)).into(),
                ),
            }
        }
        ExecuteMsg::InternalExecutePacket {
            caller,
            packet,
            relayer,
            relayer_msg,
        } => {
            if info.sender != env.contract.address {
                Err(ContractError::OnlySelf)
            } else {
                execute_packet(deps, env, info, caller, packet, relayer, relayer_msg)
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
        ExecuteMsg::Transfer {
            channel_id,
            receiver,
            base_token,
            base_amount,
            quote_token,
            quote_amount,
            timeout_height,
            timeout_timestamp,
            salt,
        } => transfer(
            deps,
            env,
            info,
            channel_id,
            receiver,
            base_token,
            base_amount,
            quote_token,
            quote_amount,
            timeout_height,
            timeout_timestamp,
            salt,
        ),
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
            timeout_height,
            timeout_timestamp,
            salt,
            Instruction::abi_decode_params(&instruction, true)?,
        ),
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

        if IN_FLIGHT_PACKET
            .may_load(deps.storage, commitment_key.into_bytes().into())?
            .is_some()
        {
            // Erase the parent packet
            IN_FLIGHT_PACKET.remove(deps.storage, commitment_key.into_bytes().into());

            // Don't write any acknowledgement, the IBC stack will prevent replay and the parent will timeout.
            return Ok(Response::new());
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
            if instruction.version > INSTR_VERSION_0 {
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
                multiplex.sender,
                multiplex.contract_calldata.clone(),
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
                    channel_id: parent_packet.destination_channel_id,
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
            if instruction.version > INSTR_VERSION_0 {
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
    source_channel: u32,
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
                    mint_to_address: sender.into_string(),
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
                                mint_to_address: market_maker.into_string(),
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
            data: encode_multiplex_calldata(path, multiplex.sender, multiplex.contract_calldata)
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
fn execute_packet(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
) -> Result<Response, ContractError> {
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    execute_internal(
        deps.branch(),
        env,
        info,
        caller,
        packet,
        relayer,
        relayer_msg,
        zkgm_packet.salt.into(),
        zkgm_packet.path,
        zkgm_packet.instruction,
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
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    salt: H256,
    path: U256,
    instruction: Instruction,
) -> Result<Response, ContractError> {
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            execute_fungible_asset_order(
                deps,
                env,
                info,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                order,
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
                caller,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                batch,
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
    channel: u32,
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
                        channel,
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
) -> Result<Response, ContractError> {
    let (tail_path, previous_destination_channel_id) = dequeue_channel_from_path(forward.path);
    let (continuation_path, next_source_channel_id) = dequeue_channel_from_path(tail_path);

    if packet.destination_channel_id != previous_destination_channel_id {
        return Err(ContractError::InvalidForwardDestinationChannelId {
            actual: previous_destination_channel_id,
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
        source_channel: next_source_channel_id,
        timeout_height: forward.timeout_height,
        timeout_timestamp: forward.timeout_timestamp,
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
            data: encode_multiplex_calldata(path, multiplex.sender, multiplex.contract_calldata)
                .into(),
            timeout_height: packet.timeout_height,
            timeout_timestamp: packet.timeout_timestamp,
        };

        // Call the target contract with a reply to capture the acknowledgement
        Ok(Response::new().add_submessage(SubMsg::reply_on_success(
            wasm_execute(
                contract_address,
                &IbcUnionMsg::OnRecvPacket {
                    caller: caller.into(),
                    packet: multiplex_packet,
                    relayer: relayer.into(),
                    relayer_msg,
                },
                vec![],
            )?,
            MULTIPLEX_REPLY_ID,
        )))
    } else {
        // Standard mode - fire and forget
        Ok(Response::new()
            .add_message(wasm_execute(
                contract_address,
                &EurekaMsg::OnZkgm {
                    path: Uint256::from_be_bytes(path.to_be_bytes()),
                    source_channel_id: packet.source_channel_id,
                    destination_channel_id: packet.destination_channel_id,
                    sender: multiplex.sender.to_vec().into(),
                    message: multiplex.contract_calldata.to_vec().into(),
                },
                vec![],
            )?)
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
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    salt: H256,
    path: U256,
    batch: Batch,
) -> Result<Response, ContractError> {
    EXECUTING_PACKET_IS_BATCH.save(deps.storage, &batch.instructions.len())?;
    let mut response = Response::new();
    for (i, instruction) in batch.instructions.into_iter().enumerate() {
        let sub_response = execute_internal(
            deps.branch(),
            env.clone(),
            info.clone(),
            caller.clone(),
            packet.clone(),
            relayer.clone(),
            relayer_msg.clone(),
            derive_batch_salt(i.try_into().unwrap(), salt),
            path,
            instruction,
        )?;
        response = response
            .add_attributes(sub_response.attributes)
            .add_events(sub_response.events)
            .add_submessages(sub_response.messages);
    }
    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn execute_fungible_asset_order(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    _relayer_msg: Bytes,
    _salt: H256,
    path: U256,
    order: FungibleAssetOrder,
) -> Result<Response, ContractError> {
    // Get the token minter contract
    let minter = TOKEN_MINTER.load(deps.storage)?;

    // Predict the wrapped token denom based on path, destination channel, and base token
    let (wrapped_denom, _) = predict_wrapped_token(
        deps.as_ref(),
        &minter,
        path,
        packet.destination_channel_id,
        Vec::from(order.base_token.clone()).into(),
    )?;

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
    let base_covers_quote = base_amount >= quote_amount;
    let fee_amount = base_amount.saturating_sub(quote_amount);

    let mut messages = Vec::<SubMsg>::new();

    // Protocol Fill - If the quote token matches the wrapped version of the base token and base amount >= quote amount
    if quote_token_str == wrapped_denom && base_covers_quote {
        // For new assets: Deploy wrapped token contract and mint quote amount to receiver
        if !HASH_TO_FOREIGN_TOKEN.has(deps.storage, wrapped_denom.clone()) {
            // Create the wrapped token if it doesn't exist
            HASH_TO_FOREIGN_TOKEN.save(
                deps.storage,
                wrapped_denom.clone(),
                &Vec::from(order.base_token.clone()).into(),
            )?;

            // Create the token with metadata
            messages.push(SubMsg::new(make_wasm_msg(
                WrappedTokenMsg::CreateDenom {
                    subdenom: wrapped_denom.clone(),
                    metadata: Metadata {
                        name: order.base_token_name,
                        symbol: order.base_token_symbol,
                        decimals: order.base_token_decimals,
                    },
                    path: path.to_be_bytes_vec().into(),
                    channel: packet.destination_channel_id,
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
            messages.push(SubMsg::new(make_wasm_msg(
                WrappedTokenMsg::MintTokens {
                    denom: wrapped_denom.clone(),
                    amount: quote_amount.into(),
                    mint_to_address: receiver.into_string(),
                },
                &minter,
                vec![],
            )?));
        }

        // Mint any fee to the relayer
        if fee_amount > 0 {
            messages.push(SubMsg::new(make_wasm_msg(
                WrappedTokenMsg::MintTokens {
                    denom: wrapped_denom,
                    amount: fee_amount.into(),
                    mint_to_address: relayer.into_string(),
                },
                &minter,
                vec![],
            )?));
        }
    }
    // Unwrapping - For returning assets: Unwrap base token and transfer quote amount to receiver
    else if order.base_token_path != U256::ZERO && base_covers_quote {
        // Decrease the outstanding balance for this channel and path
        decrease_channel_balance(
            deps,
            packet.destination_channel_id,
            reverse_channel_path(path),
            quote_token_str.clone(),
            base_amount.into(),
        )?;

        // Transfer the quote amount to the receiver
        if quote_amount > 0 {
            messages.push(SubMsg::new(make_wasm_msg(
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
            messages.push(SubMsg::new(make_wasm_msg(
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
        // Return a special acknowledgement that indicates this order needs a market maker
        return Ok(Response::new().add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: ACK_ERR_ONLY_MAKER.into(),
            },
            vec![],
        )?));
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
        // This reply is triggered after we escrow tokens in the token minter contract.
        // We need to handle this reply to process any dispatch events that the token minter might have
        // emitted during the escrow operation. These events can contain additional messages that need
        // to be executed as part of the token transfer process, such as updating balances or
        // performing additional token operations.
        ESCROW_REPLY_ID => {
            let Some(dispatch) = reply
                .result
                .into_result()
                .expect("only if success")
                .events
                .into_iter()
                .find(|e| e.ty == format!("wasm-{DISPATCH_EVENT}"))
            else {
                return Ok(Response::new());
            };

            let Some(attr) = dispatch
                .attributes
                .into_iter()
                .find(|a| a.key == DISPATCH_EVENT_ATTR)
            else {
                return Ok(Response::new());
            };

            match serde_json_wasm::from_str::<Vec<WasmMsg>>(&attr.value) {
                Ok(msgs) => Ok(Response::new().add_messages(msgs)),
                Err(_) => Ok(Response::new()),
            }
        }
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
                                    channel_id: packet.destination_channel_id,
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
                                    channel_id: packet.destination_channel_id,
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
            if let SubMsgResult::Ok(reply_data) = reply.result {
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
            } else {
                Err(ContractError::AsyncMultiplexUnsupported)
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
    deps: Deps,
    info: MessageInfo,
    channel_id: u32,
    path: U256,
    instruction: &Instruction,
    response: &mut Response,
) -> Result<(), ContractError> {
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            if instruction.version != INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            verify_fungible_asset_order(deps, info, channel_id, path, &order, response)
        }
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            verify_batch(deps, info, channel_id, path, &batch, response)
        }
        OP_FORWARD => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let forward = Forward::abi_decode_params(&instruction.operand, true)?;
            verify_forward(deps, info, channel_id, &forward, response)
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
fn verify_fungible_asset_order(
    deps: Deps,
    info: MessageInfo,
    channel_id: u32,
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
        return Err(ContractError::InvalidAssetName);
    }
    if metadata.symbol != order.base_token_symbol {
        return Err(ContractError::InvalidAssetSymbol);
    }
    if metadata.decimals != order.base_token_decimals {
        return Err(ContractError::InvalidAssetDecimals);
    }

    // Get the origin path for the base token
    let origin = TOKEN_ORIGIN.may_load(deps.storage, base_token_str.to_string())?;

    // Compute the wrapped token from the destination to source
    let (wrapped_token, _) = predict_wrapped_token(
        deps,
        &minter,
        path,
        channel_id,
        order.quote_token.to_vec().into(),
    )?;

    // Check if base token matches predicted wrapper
    let is_unwrapping = base_token_str == wrapped_token;

    // Get the intermediate path and destination channel from origin
    let (intermediate_path, destination_channel_id) = if let Some(origin) = origin {
        let origin_u256 = U256::from_be_bytes(origin.to_be_bytes());
        pop_channel_from_path(origin_u256)
    } else {
        (U256::ZERO, 0)
    };

    // Check if we're taking same path starting from same channel using wrapped asset
    let is_inverse_intermediate_path = path == reverse_channel_path(intermediate_path);
    let is_sending_back_to_same_channel = destination_channel_id == channel_id;

    if is_inverse_intermediate_path && is_sending_back_to_same_channel && is_unwrapping {
        // Verify the origin path matches what's in the order
        if let Some(origin) = origin {
            let origin_u256 = U256::from_be_bytes(origin.to_be_bytes());
            if origin_u256 != order.base_token_path {
                return Err(ContractError::InvalidAssetOrigin);
            }
            // Burn tokens as we are going to unescrow on the counterparty
            *response = response.clone().add_message(make_wasm_msg(
                WrappedTokenMsg::BurnTokens {
                    denom: base_token_str.to_string(),
                    amount: Uint256::from_be_bytes(order.base_amount.to_be_bytes())
                        .try_into()
                        .map_err(|_| ContractError::AmountOverflow)?,
                    burn_from_address: minter.to_string(),
                    sender: info.sender,
                },
                &minter,
                info.funds,
            )?);
        } else {
            return Err(ContractError::InvalidAssetOrigin);
        }
    } else if !order.base_token_path.is_zero() {
        return Err(ContractError::InvalidAssetOrigin);
    } else {
        // Escrow tokens as the counterparty will mint them
        *response = response.clone().add_message(make_wasm_msg(
            LocalTokenMsg::Escrow {
                from: info.sender.to_string(),
                denom: base_token_str.to_string(),
                recipient: minter.to_string(),
                amount: Uint256::from_be_bytes(order.base_amount.to_be_bytes())
                    .try_into()
                    .map_err(|_| ContractError::AmountOverflow)?,
            },
            &minter,
            info.funds,
        )?);
    }

    Ok(())
}

/// Verifies a batch instruction by checking each sub-instruction is allowed and valid.
/// Only certain instruction types are allowed in batches to prevent complex nested operations.
fn verify_batch(
    deps: Deps,
    info: MessageInfo,
    channel_id: u32,
    path: U256,
    batch: &Batch,
    response: &mut Response,
) -> Result<(), ContractError> {
    for instruction in &batch.instructions {
        if !is_allowed_batch_instruction(instruction.opcode) {
            return Err(ContractError::InvalidBatchInstruction);
        }
        verify_internal(deps, info.clone(), channel_id, path, instruction, response)?;
    }
    Ok(())
}

/// Verifies a forward instruction by checking the sub-instruction is allowed and valid.
/// Forward instructions can contain batch, multiplex or fungible asset orders.
pub fn verify_forward(
    deps: Deps,
    info: MessageInfo,
    channel_id: u32,
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
    deps: DepsMut,
    info: MessageInfo,
    channel_id: u32,
    timeout_height: u64,
    timeout_timestamp: u64,
    salt: H256,
    instruction: Instruction,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    // Verify the instruction
    verify_internal(
        deps.as_ref(),
        info.clone(),
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
            source_channel: channel_id,
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

#[allow(clippy::too_many_arguments)]
fn transfer(
    mut deps: DepsMut,
    _: Env,
    info: MessageInfo,
    channel_id: u32,
    receiver: Bytes,
    base_token: String,
    base_amount: Uint128,
    quote_token: Bytes,
    quote_amount: Uint256,
    timeout_height: u64,
    timeout_timestamp: u64,
    salt: H256,
) -> Result<Response, ContractError> {
    // NOTE(aeryz): We don't check whether the funds are provided here. We check it in the
    // minter because cw20 token minter doesn't require funds to be given in the native form.
    let minter = TOKEN_MINTER.load(deps.storage)?;
    // If the origin exists, the preimage exists
    let unwrapped_asset = HASH_TO_FOREIGN_TOKEN.may_load(deps.storage, base_token.clone())?;
    let mut messages = Vec::<SubMsg>::new();
    // TODO: handle forward path
    let mut origin = TOKEN_ORIGIN.may_load(deps.storage, base_token.clone())?;
    match origin {
        // Burn as we are going to unescrow on the counterparty
        Some(path)
            if path == Uint256::from(channel_id)
                && unwrapped_asset == Some(quote_token.clone()) =>
        {
            messages.push(SubMsg::reply_on_success(
                make_wasm_msg(
                    WrappedTokenMsg::BurnTokens {
                        denom: base_token.clone(),
                        amount: base_amount,
                        burn_from_address: minter.to_string(),
                        sender: info.sender.clone(),
                    },
                    &minter,
                    info.funds,
                )?,
                ESCROW_REPLY_ID,
            ))
        }
        // Escrow and update the balance, the counterparty will mint the token
        _ => {
            origin = None;
            messages.push(SubMsg::reply_on_success(
                make_wasm_msg(
                    LocalTokenMsg::Escrow {
                        from: info.sender.to_string(),
                        denom: base_token.clone(),
                        recipient: minter.to_string(),
                        amount: base_amount,
                    },
                    &minter,
                    info.funds,
                )?,
                ESCROW_REPLY_ID,
            ));
            // Use path = 0 for local tokens, matching Solidity implementation
            increase_channel_balance(
                deps.branch(),
                channel_id,
                U256::ZERO,
                base_token.clone(),
                base_amount.into(),
            )?;
        }
    };
    let MetadataResponse {
        name: base_token_name,
        symbol: base_token_symbol,
        decimals: base_token_decimals,
    } = deps.querier.query::<MetadataResponse>(&QueryRequest::Wasm(
        cosmwasm_std::WasmQuery::Smart {
            contract_addr: minter.to_string(),
            msg: to_json_binary(&ucs03_zkgm_token_minter_api::QueryMsg::Metadata {
                denom: base_token.clone(),
            })?,
        },
    ))?;
    let config = CONFIG.load(deps.storage)?;
    messages.push(SubMsg::new(wasm_execute(
        &config.ibc_host,
        &ibc_union_msg::msg::ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel: channel_id,
            timeout_height,
            timeout_timestamp,
            data: ZkgmPacket {
                salt: salt.into(),
                path: U256::ZERO,
                instruction: Instruction {
                    version: INSTR_VERSION_0,
                    opcode: OP_FUNGIBLE_ASSET_ORDER,
                    operand: FungibleAssetOrder {
                        sender: info.sender.as_bytes().to_vec().into(),
                        receiver: Vec::from(receiver).into(),
                        base_token: base_token.as_bytes().to_vec().into(),
                        base_amount: base_amount.u128().try_into().expect("u256>u128"),
                        base_token_symbol,
                        base_token_name,
                        base_token_decimals,
                        base_token_path: origin
                            .map(|x| U256::from_be_bytes(x.to_be_bytes()))
                            .unwrap_or(U256::ZERO),
                        quote_token: Vec::from(quote_token).into(),
                        quote_amount: U256::from_be_bytes(quote_amount.to_be_bytes()),
                    }
                    .abi_encode_params()
                    .into(),
                },
            }
            .abi_encode_params()
            .into(),
        }),
        vec![],
    )?));
    Ok(Response::new().add_submessages(messages))
}

#[cosmwasm_schema::cw_serde]
pub struct TokenMinterMigration {
    // code id of the new token minter
    new_code_id: u64,
    // migrate message json that will directly be passed to migrate call
    // it will be the same as the `to_json_binary(&msg)`'s output
    msg: Binary,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {
    // Provide `token_minter_migration` to also migrate the token minter
    token_minter_migration: Option<TokenMinterMigration>,
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
            channel,
            token,
        } => {
            let minter = TOKEN_MINTER.load(deps.storage)?;
            let (token, _) = predict_wrapped_token(
                deps,
                &minter,
                path.parse().map_err(ContractError::InvalidPath)?,
                channel,
                token,
            )?;
            Ok(to_json_binary(&PredictWrappedTokenResponse {
                wrapped_token: token.as_bytes().into(),
            })?)
        }
    }
}

/// Increases the outstanding balance for a (channel, path, token) combination.
/// This is used when escrowing tokens to track how many tokens can be unescrowed later.
/// The balance is used to prevent double-spending and ensure token conservation across chains.
fn increase_channel_balance(
    deps: DepsMut,
    channel_id: u32,
    path: U256,
    base_token: String,
    base_amount: Uint256,
) -> Result<(), ContractError> {
    CHANNEL_BALANCE.update(
        deps.storage,
        (channel_id, path.to_be_bytes::<32>().to_vec(), base_token),
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
    channel_id: u32,
    path: U256,
    token: String,
    amount: Uint256,
) -> Result<(), ContractError> {
    CHANNEL_BALANCE.update(
        deps.storage,
        (channel_id, path.to_be_bytes::<32>().to_vec(), token),
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

pub fn dequeue_channel_from_path(path: U256) -> (U256, u32) {
    if path == U256::ZERO {
        (U256::ZERO, 0)
    } else {
        (
            path >> 32,
            u32::try_from(path & U256::from(u32::MAX)).expect("impossible"),
        )
    }
}

/// Extract the last channel from a path and return the base path without it
pub fn pop_channel_from_path(path: U256) -> (U256, u32) {
    if path == U256::ZERO {
        return (U256::ZERO, 0);
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

pub fn update_channel_path(path: U256, next_channel_id: u32) -> Result<U256, ContractError> {
    if path == U256::ZERO {
        Ok(U256::from(next_channel_id))
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

pub fn get_channel_from_path(path: U256, index: usize) -> u32 {
    u32::try_from((path >> (32 * index)) & U256::from(u32::MAX)).expect("impossible")
}

pub fn make_path_from_channel(channel_id: u32, index: usize) -> U256 {
    U256::from(channel_id) << (32 * index)
}

pub fn reverse_channel_path(path: U256) -> U256 {
    make_path_from_channel(get_channel_from_path(path, 0), 7)
        | make_path_from_channel(get_channel_from_path(path, 1), 6)
        | make_path_from_channel(get_channel_from_path(path, 2), 5)
        | make_path_from_channel(get_channel_from_path(path, 3), 4)
        | make_path_from_channel(get_channel_from_path(path, 4), 3)
        | make_path_from_channel(get_channel_from_path(path, 5), 2)
        | make_path_from_channel(get_channel_from_path(path, 6), 1)
        | make_path_from_channel(get_channel_from_path(path, 7), 0)
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

pub fn encode_multiplex_calldata(
    path: U256,
    sender: alloy::primitives::Bytes,
    contract_calldata: alloy::primitives::Bytes,
) -> Vec<u8> {
    (path, sender, contract_calldata).abi_encode()
}
