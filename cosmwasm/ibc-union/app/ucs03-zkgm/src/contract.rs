use core::str;

use alloy::sol_types::SolValue;
use base58::ToBase58;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, to_json_string, wasm_execute, Addr, Coin, CosmosMsg, DepsMut, Env, MessageInfo,
    QueryRequest, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128, Uint256,
};
use ibc_union_msg::{
    module::IbcUnionMsg,
    msg::{MsgSendPacket, MsgWriteAcknowledgement},
};
use ibc_union_spec::types::Packet;
use token_factory_api::{Metadata, MetadataResponse, TokenFactoryMsg, TokenFactoryQuery};
use ucs03_zkgm_token_minter_api::LocalTokenMsg;
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bytes, H256},
};

use crate::{
    com::{
        Ack, Batch, BatchAck, FungibleAssetOrder, FungibleAssetOrderAck, Instruction, Multiplex,
        ZkgmPacket, ACK_ERR_ONLY_MAKER, FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL, OP_BATCH,
        OP_FUNGIBLE_ASSET_ORDER, OP_MULTIPLEX, TAG_ACK_FAILURE, TAG_ACK_SUCCESS, ZKGM_VERSION_0,
    },
    msg::{EurekaMsg, ExecuteMsg, InitMsg, MigrateMsg},
    state::{
        CHANNEL_BALANCE, CONFIG, EXECUTING_PACKET, EXECUTION_ACK, HASH_TO_FOREIGN_TOKEN,
        TOKEN_ORIGIN,
    },
    ContractError,
};

pub const PROTOCOL_VERSION: &str = "ucs03-zkgm-0";

pub const REPLY_ID: u64 = 0x1337;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
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
            let ibc_host = CONFIG.load(deps.storage)?.ibc_host;
            if info.sender != ibc_host {
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
                    packet,
                    relayer,
                    relayer_msg,
                } => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    if EXECUTING_PACKET.exists(deps.storage) {
                        Err(ContractError::AlreadyExecuting)
                    } else {
                        EXECUTING_PACKET.save(deps.storage, &packet)?;
                        Ok(Response::default().add_submessage(SubMsg::reply_always(
                            wasm_execute(
                                env.contract.address,
                                &ExecuteMsg::ExecutePacket {
                                    packet,
                                    relayer,
                                    relayer_msg,
                                },
                                vec![],
                            )?,
                            REPLY_ID,
                        )))
                    }
                }
                IbcUnionMsg::OnAcknowledgementPacket {
                    packet,
                    acknowledgement,
                    relayer,
                } => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    acknowledge_packet(deps, env, info, packet, relayer, acknowledgement)
                }
                IbcUnionMsg::OnTimeoutPacket { packet, relayer } => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    timeout_packet(deps, env, info, packet, relayer)
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
        ExecuteMsg::BatchExecute { msgs } => {
            if info.sender != env.contract.address {
                Err(ContractError::OnlySelf)
            } else {
                Ok(Response::default().add_messages(msgs))
            }
        }
        ExecuteMsg::ExecutePacket {
            packet,
            relayer,
            relayer_msg,
        } => {
            if info.sender != env.contract.address {
                Err(ContractError::OnlySelf)
            } else {
                execute_packet(deps, env, info, packet, relayer, relayer_msg)
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
    }
}

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

fn timeout_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
) -> Result<Response, ContractError> {
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    timeout_internal(
        deps,
        env,
        info,
        packet,
        relayer,
        zkgm_packet.salt.into(),
        zkgm_packet.path,
        zkgm_packet.instruction,
    )
}

#[allow(clippy::too_many_arguments)]
fn timeout_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    salt: H256,
    _path: alloy::primitives::U256,
    instruction: Instruction,
) -> Result<Response, ContractError> {
    if instruction.version != ZKGM_VERSION_0 {
        return Err(ContractError::UnsupportedVersion {
            version: instruction.version,
        });
    }
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
            let order = FungibleAssetOrder::abi_decode_params(&instruction.operand, true)?;
            refund(deps, packet.source_channel_id, order)
        }
        OP_BATCH => {
            let mut response = Response::new();
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            for (i, instruction) in batch.instructions.into_iter().enumerate() {
                let sub_response = timeout_internal(
                    deps.branch(),
                    env.clone(),
                    info.clone(),
                    packet.clone(),
                    relayer.clone(),
                    keccak256(
                        (alloy::primitives::U256::try_from(i).unwrap(), salt.get()).abi_encode(),
                    ),
                    _path,
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
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            if !multiplex.eureka {
                // TODO: implement when execute is implemented
                Err(ContractError::Unimplemented)
            } else {
                Ok(Response::new())
            }
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn acknowledge_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    ack: Bytes,
) -> Result<Response, ContractError> {
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    let ack = Ack::abi_decode_params(&ack, true)?;
    acknowledge_internal(
        deps,
        env,
        info,
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
fn acknowledge_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    salt: H256,
    path: alloy::primitives::U256,
    instruction: Instruction,
    successful: bool,
    ack: Bytes,
) -> Result<Response, ContractError> {
    if instruction.version != ZKGM_VERSION_0 {
        return Err(ContractError::UnsupportedVersion {
            version: instruction.version,
        });
    }
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
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
                    packet.clone(),
                    relayer.clone(),
                    keccak256(
                        (alloy::primitives::U256::try_from(i).unwrap(), salt.get()).abi_encode(),
                    ),
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
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            if !multiplex.eureka {
                // TODO: implement when execute is implemented
                Err(ContractError::Unimplemented)
            } else {
                Ok(Response::new())
            }
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn refund(
    deps: DepsMut,
    source_channel: u32,
    order: FungibleAssetOrder,
) -> Result<Response, ContractError> {
    // 1. Native minter + sent native tokens: correct
    // 2. Cw20 minter + sent native tokens:
    let sender = deps
        .api
        .addr_validate(str::from_utf8(&order.sender).map_err(|_| ContractError::InvalidSender)?)
        .map_err(|_| ContractError::UnableToValidateSender)?;
    let base_amount =
        u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;
    let base_denom = String::from_utf8(order.base_token.to_vec())
        .map_err(|_| ContractError::InvalidBaseToken)?;
    let mut messages = Vec::<CosmosMsg>::new();
    let minter = CONFIG.load(deps.storage)?.token_minter;
    // TODO: handle forward path
    if order.base_token_path == source_channel.try_into().unwrap() {
        messages.push(make_wasm_msg(
            TokenFactoryMsg::MintTokens {
                denom: base_denom,
                amount: base_amount.into(),
                mint_to_address: sender.into_string(),
            },
            minter,
            vec![],
        )?);
    } else {
        messages.push(make_wasm_msg(
            LocalTokenMsg::Transfer {
                denom: base_denom,
                recipient: sender.into_string(),
                amount: base_amount.into(),
            },
            minter,
            vec![],
        )?);
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
    _path: alloy::primitives::U256,
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
                    let base_denom = String::from_utf8(order.base_token.to_vec())
                        .map_err(|_| ContractError::InvalidBaseToken)?;
                    let minter = CONFIG.load(deps.storage)?.token_minter;
                    // TODO: handle forward path
                    if order.base_token_path == packet.source_channel_id.try_into().unwrap() {
                        messages.push(make_wasm_msg(
                            TokenFactoryMsg::MintTokens {
                                denom: base_denom,
                                amount: base_amount.into(),
                                mint_to_address: market_maker.into_string(),
                            },
                            minter,
                            vec![],
                        )?);
                    } else {
                        messages.push(make_wasm_msg(
                            LocalTokenMsg::Transfer {
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
        None => refund(deps, packet.source_channel_id, order),
    }
}

fn execute_packet(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
) -> Result<Response, ContractError> {
    let zkgm_packet = ZkgmPacket::abi_decode_params(&packet.data, true)?;
    let (ack, response) = execute_internal(
        deps.branch(),
        env,
        info,
        packet,
        relayer,
        relayer_msg,
        zkgm_packet.salt.into(),
        zkgm_packet.path,
        zkgm_packet.instruction,
    )?;
    EXECUTION_ACK.save(deps.storage, &ack)?;
    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn execute_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    salt: H256,
    path: alloy::primitives::U256,
    instruction: Instruction,
) -> Result<(Bytes, Response), ContractError> {
    if instruction.version != ZKGM_VERSION_0 {
        return Err(ContractError::UnsupportedVersion {
            version: instruction.version,
        });
    }
    match instruction.opcode {
        OP_FUNGIBLE_ASSET_ORDER => {
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
            let batch = Batch::abi_decode_params(&instruction.operand, true)?;
            execute_batch(
                deps,
                env,
                info,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                batch,
            )
        }
        OP_MULTIPLEX => {
            let multiplex = Multiplex::abi_decode_params(&instruction.operand, true)?;
            execute_multiplex(
                deps,
                env,
                info,
                packet,
                relayer,
                relayer_msg,
                salt,
                path,
                multiplex,
            )
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn factory_denom(token: &str, contract: &str) -> String {
    format!("factory/{}/{}", contract, token)
}

fn predict_wrapped_denom(path: alloy::primitives::U256, channel: u32, token: Bytes) -> String {
    // TokenFactory denom name limit
    const MAX_DENOM_LENGTH: usize = 44;

    let token_hash = keccak256(
        [
            path.to_be_bytes_vec().as_ref(),
            channel.to_be_bytes().as_ref(),
            &token,
        ]
        .concat(),
    )
    .get()
    .to_base58();

    // https://en.wikipedia.org/wiki/Binary-to-text_encoding
    // Luckily, base58 encoding has ~0.73 efficiency:
    // (1 / 0.73) * 32 = 43.8356164384
    // TokenFactory denom name limit
    assert!(token_hash.len() <= MAX_DENOM_LENGTH);

    token_hash.to_string()
}

#[allow(clippy::too_many_arguments)]
fn execute_multiplex(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    packet: Packet,
    _relayer: Addr,
    _relayer_msg: Bytes,
    _salt: H256,
    _path: alloy::primitives::U256,
    multiplex: Multiplex,
) -> Result<(Bytes, Response), ContractError> {
    let contract_address = deps
        .api
        .addr_validate(
            str::from_utf8(&multiplex.contract_address)
                .map_err(|_| ContractError::InvalidContractAddress)?,
        )
        .map_err(|_| ContractError::UnableToValidateMultiplexTarget)?;
    if multiplex.eureka {
        Ok((
            TAG_ACK_SUCCESS.abi_encode().into(),
            Response::new().add_message(wasm_execute(
                contract_address,
                &EurekaMsg::OnZkgm {
                    channel_id: packet.destination_channel_id,
                    sender: multiplex.sender.to_vec().into(),
                    message: multiplex.contract_calldata.to_vec().into(),
                },
                vec![],
            )?),
        ))
    } else {
        // TODO: implement non eureka multiplexing, add a new msg `WriteAck` by
        // maintaining the hashing from packet to ack or thread the ack back
        // from the events directly? Beware we'll need to use submessage+reply
        // for execute_batch when implementing this as we need the ack to yield
        // the batch ack
        Err(ContractError::Unimplemented)
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_batch(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    packet: Packet,
    relayer: Addr,
    relayer_msg: Bytes,
    salt: H256,
    path: alloy::primitives::U256,
    batch: Batch,
) -> Result<(Bytes, Response), ContractError> {
    let mut response = Response::new();
    let mut acks = Vec::<Bytes>::with_capacity(batch.instructions.len());
    for (i, instruction) in batch.instructions.into_iter().enumerate() {
        let (ack, sub_response) = execute_internal(
            deps.branch(),
            env.clone(),
            info.clone(),
            packet.clone(),
            relayer.clone(),
            relayer_msg.clone(),
            keccak256((alloy::primitives::U256::try_from(i).unwrap(), salt.get()).abi_encode()),
            path,
            instruction,
        )?;
        response = response
            .add_attributes(sub_response.attributes)
            .add_events(sub_response.events)
            .add_submessages(sub_response.messages);
        acks.push(ack);
    }
    Ok((
        BatchAck {
            acknowledgements: acks.into_iter().map(Into::into).collect(),
        }
        .abi_encode_params()
        .into(),
        response,
    ))
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
    path: alloy::primitives::U256,
    order: FungibleAssetOrder,
) -> Result<(Bytes, Response), ContractError> {
    if order.quote_amount > order.base_amount {
        return Ok((ACK_ERR_ONLY_MAKER.into(), Response::new()));
    }
    let wrapped_denom = predict_wrapped_denom(
        path,
        packet.destination_channel_id,
        Vec::from(order.base_token.clone()).into(),
    );
    let quote_amount =
        u128::try_from(order.quote_amount).map_err(|_| ContractError::AmountOverflow)?;
    let fee_amount = order.base_amount - order.quote_amount;
    let fee_amount = u128::try_from(fee_amount).map_err(|_| ContractError::AmountOverflow)?;
    let receiver = deps
        .api
        .addr_validate(
            str::from_utf8(order.receiver.as_ref()).map_err(|_| ContractError::InvalidReceiver)?,
        )
        .map_err(|_| ContractError::UnableToValidateReceiver)?;
    let mut messages = Vec::<CosmosMsg>::new();
    let minter = CONFIG.load(deps.storage)?.token_minter;
    if order.quote_token.as_ref() == wrapped_denom.as_bytes() {
        // TODO: handle forwarding path
        let subdenom = factory_denom(&wrapped_denom, env.contract.address.as_str());
        if !HASH_TO_FOREIGN_TOKEN.has(deps.storage, subdenom.clone()) {
            HASH_TO_FOREIGN_TOKEN.save(
                deps.storage,
                subdenom.clone(),
                &Vec::from(order.base_token.clone()).into(),
            )?;
            messages.push(make_wasm_msg(
                TokenFactoryMsg::CreateDenom {
                    subdenom: wrapped_denom,
                },
                &minter,
                vec![],
            )?);
            messages.push(make_wasm_msg(
                TokenFactoryMsg::SetDenomMetadata {
                    denom: subdenom.clone(),
                    metadata: Metadata {
                        description: None,
                        denom_units: vec![],
                        base: None,
                        display: None,
                        name: Some(order.base_token_name),
                        symbol: Some(order.base_token_symbol),
                        uri: None,
                        uri_hash: None,
                    },
                },
                &minter,
                vec![],
            )?);
            TOKEN_ORIGIN.save(
                deps.storage,
                subdenom.clone(),
                &Uint256::from_u128(packet.destination_channel_id as _),
            )?;
        };
        messages.push(make_wasm_msg(
            TokenFactoryMsg::MintTokens {
                denom: subdenom.clone(),
                amount: quote_amount.into(),
                mint_to_address: receiver.into_string(),
            },
            &minter,
            vec![],
        )?);
        if fee_amount > 0 {
            messages.push(make_wasm_msg(
                TokenFactoryMsg::MintTokens {
                    denom: subdenom,
                    amount: fee_amount.into(),
                    mint_to_address: relayer.into_string(),
                },
                &minter,
                vec![],
            )?);
        }
    } else if order.base_token_path == alloy::primitives::U256::from(packet.source_channel_id) {
        let quote_token = String::from_utf8(Vec::from(order.quote_token))
            .map_err(|_| ContractError::InvalidQuoteToken)?;
        CHANNEL_BALANCE.update(
            deps.storage,
            (packet.destination_channel_id, quote_token.clone()),
            |balance| match balance {
                Some(value) => value
                    .checked_sub(quote_amount.into())
                    .map_err(|_| ContractError::InvalidChannelBalance),
                None => Err(ContractError::InvalidChannelBalance),
            },
        )?;
        messages.push(make_wasm_msg(
            LocalTokenMsg::Transfer {
                denom: quote_token.clone(),
                recipient: receiver.into_string(),
                amount: quote_amount.into(),
            },
            &minter,
            vec![],
        )?);
        if fee_amount > 0 {
            messages.push(make_wasm_msg(
                LocalTokenMsg::Transfer {
                    denom: quote_token.clone(),
                    recipient: relayer.into_string(),
                    amount: quote_amount.into(),
                },
                minter,
                vec![],
            )?);
        }
    } else {
        return Ok((ACK_ERR_ONLY_MAKER.into(), Response::new()));
    };
    Ok((
        FungibleAssetOrderAck {
            fill_type: FILL_TYPE_PROTOCOL,
            market_maker: Default::default(),
        }
        .abi_encode_params()
        .into(),
        Response::new().add_messages(messages),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != REPLY_ID {
        return Err(ContractError::UnknownReply { id: reply.id });
    }
    let ibc_host = CONFIG.load(deps.storage)?.ibc_host;
    let packet = EXECUTING_PACKET.load(deps.storage)?;
    EXECUTING_PACKET.remove(deps.storage);
    match reply.result {
        SubMsgResult::Ok(_) => {
            // If the execution succedeed ack is guaranteed to exist.
            let execution_ack = EXECUTION_ACK.load(deps.storage)?;
            EXECUTION_ACK.remove(deps.storage);
            match execution_ack {
                // Specific value when the execution must be replayed by a MM. No
                // side effects were executed. We break the TX for MMs to be able to
                // replay the packet.
                ack if ack.as_ref() == ACK_ERR_ONLY_MAKER => Err(ContractError::OnlyMaker),
                ack if !ack.is_empty() => {
                    let zkgm_ack = Ack {
                        tag: TAG_ACK_SUCCESS,
                        inner_ack: Vec::from(ack).into(),
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
                }
                // Async acknowledgement, we don't write anything
                _ => Ok(Response::new()),
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

#[allow(clippy::too_many_arguments)]
fn transfer(
    deps: DepsMut,
    env: Env,
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
    if base_amount.is_zero() {
        return Err(ContractError::InvalidAmount);
    }
    // If the origin exists, the preimage exists
    let unwrapped_asset = HASH_TO_FOREIGN_TOKEN.may_load(deps.storage, base_token.clone())?;
    let mut messages = Vec::<CosmosMsg>::new();
    // TODO: handle forward path
    let mut origin = TOKEN_ORIGIN.may_load(deps.storage, base_token.clone())?;
    let minter = CONFIG.load(deps.storage)?.token_minter;
    match origin {
        // Burn as we are going to unescrow on the counterparty
        Some(path)
            if path == Uint256::from(channel_id)
                && unwrapped_asset == Some(quote_token.clone()) =>
        {
            messages.push(make_wasm_msg(
                TokenFactoryMsg::BurnTokens {
                    denom: base_token.clone(),
                    amount: base_amount,
                    burn_from_address: env.contract.address.into_string(),
                },
                &minter,
                info.funds,
            )?)
        }
        // Escrow and update the balance, the counterparty will mint the token
        _ => {
            origin = None;
            messages.push(make_wasm_msg(
                LocalTokenMsg::TakeFunds {
                    from: env.contract.address.to_string(),
                    denom: base_token.clone(),
                    recipient: String::from_utf8_lossy(&receiver).to_string(),
                    amount: base_amount,
                },
                &minter,
                info.funds,
            )?);
            CHANNEL_BALANCE.update(deps.storage, (channel_id, base_token.clone()), |balance| {
                match balance {
                    Some(value) => value
                        .checked_add(base_amount.into())
                        .map_err(|_| ContractError::InvalidChannelBalance),
                    None => Ok(base_amount.into()),
                }
            })?;
        }
    };
    let denom_metadata = deps.querier.query::<MetadataResponse>(&QueryRequest::Wasm(
        cosmwasm_std::WasmQuery::Smart {
            contract_addr: minter.to_string(),
            msg: to_json_binary(&TokenFactoryQuery::Metadata {
                denom: base_token.clone(),
            })?,
        },
    ));
    let default_name = "".into();
    let default_symbol = base_token.clone();
    let (base_token_name, base_token_symbol) = match denom_metadata {
        Ok(MetadataResponse {
            metadata: Some(metadata),
        }) => (
            metadata.name.unwrap_or(default_name),
            metadata.symbol.unwrap_or(default_symbol),
        ),
        _ => (default_name, default_symbol),
    };
    let config = CONFIG.load(deps.storage)?;
    messages.push(
        wasm_execute(
            &config.ibc_host,
            &ibc_union_msg::msg::ExecuteMsg::PacketSend(MsgSendPacket {
                source_channel: channel_id,
                timeout_height,
                timeout_timestamp,
                data: ZkgmPacket {
                    salt: salt.into(),
                    path: alloy::primitives::U256::ZERO,
                    instruction: Instruction {
                        version: ZKGM_VERSION_0,
                        opcode: OP_FUNGIBLE_ASSET_ORDER,
                        operand: FungibleAssetOrder {
                            sender: info.sender.as_bytes().to_vec().into(),
                            receiver: Vec::from(receiver).into(),
                            base_token: base_token.as_bytes().to_vec().into(),
                            base_amount: base_amount.u128().try_into().expect("u256>u128"),
                            base_token_symbol,
                            base_token_name,
                            base_token_path: origin
                                .map(|x| alloy::primitives::U256::from_be_bytes(x.to_be_bytes()))
                                .unwrap_or(alloy::primitives::U256::ZERO),
                            quote_token: Vec::from(quote_token).into(),
                            quote_amount: alloy::primitives::U256::from_be_bytes(
                                quote_amount.to_be_bytes(),
                            ),
                        }
                        .abi_encode_params()
                        .into(),
                    },
                }
                .abi_encode_params()
                .into(),
            }),
            vec![],
        )?
        .into(),
    );
    Ok(Response::new().add_messages(messages))
}

fn make_wasm_msg(
    msg: impl Into<ucs03_zkgm_token_minter_api::ExecuteMsg>,
    minter: impl Into<String>,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let msg = msg.into();
    Ok(CosmosMsg::Wasm(wasm_execute(minter, &msg, funds)?))
}
