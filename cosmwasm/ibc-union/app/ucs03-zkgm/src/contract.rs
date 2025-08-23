use core::str;
use std::str::FromStr;

use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use chrono::DateTime;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, to_json_string, wasm_execute, Addr, BankMsg, Binary,
    CodeInfoResponse, Coin, Coins, CosmosMsg, DecCoin, Decimal256, Deps, DepsMut, DistributionMsg,
    Empty, Env, Event, MessageInfo, QueryRequest, Reply, Response, StakingMsg, StdError, StdResult,
    Storage, SubMsg, SubMsgResponse, SubMsgResult, Uint128, Uint256, WasmMsg,
};
use frissitheto::UpgradeMsg;
use ibc_union_msg::{
    module::IbcUnionMsg,
    msg::{MsgSendPacket, MsgWriteAcknowledgement},
};
use ibc_union_spec::{path::BatchPacketsPath, ChannelId, MustBeZero, Packet, Timestamp};
use ucs03_zkgm_token_minter_api::{
    new_wrapped_token_event, LocalTokenMsg, Metadata, MetadataResponse, WrappedTokenKind,
    WrappedTokenMsg,
};
use unionlabs::{
    ethereum::keccak256,
    primitives::{encoding::HexPrefixed, Bytes, H256},
};

use crate::{
    com::{
        Ack, Batch, BatchAck, Call, Forward, Instruction, SolverMetadata, Stake, TokenMetadata,
        TokenOrderAck, TokenOrderV1, TokenOrderV2, Unstake, UnstakeAck, WithdrawRewards,
        WithdrawRewardsAck, WithdrawStake, WithdrawStakeAck, ZkgmPacket, ACK_ERR_ONLY_MAKER,
        FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL, FORWARD_SALT_MAGIC, INSTR_VERSION_0,
        INSTR_VERSION_1, INSTR_VERSION_2, OP_BATCH, OP_CALL, OP_FORWARD, OP_STAKE, OP_TOKEN_ORDER,
        OP_UNSTAKE, OP_WITHDRAW_REWARDS, OP_WITHDRAW_STAKE, TAG_ACK_FAILURE, TAG_ACK_SUCCESS,
        TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE, TOKEN_ORDER_KIND_SOLVE,
        TOKEN_ORDER_KIND_UNESCROW,
    },
    msg::{
        Config, ExecuteMsg, InitMsg, PredictWrappedTokenResponse, QueryMsg, SolverMsg,
        V1ToV2Migration, V1ToV2WrappedMigration, ZkgmMsg,
    },
    state::{
        BATCH_EXECUTION_ACKS, CHANNEL_BALANCE_V2, CONFIG, DEPRECATED_CHANNEL_BALANCE_V1,
        EXECUTING_PACKET, EXECUTING_PACKET_IS_BATCH, EXECUTION_ACK, HASH_TO_FOREIGN_TOKEN,
        IN_FLIGHT_PACKET, MARKET_MAKER, METADATA_IMAGE_OF, TOKEN_BUCKET, TOKEN_MINTER,
        TOKEN_ORIGIN,
    },
    token_bucket::TokenBucket,
    ContractError,
};

pub const PROTOCOL_VERSION: &str = "ucs03-zkgm-0";

pub const EXECUTE_REPLY_ID: u64 = 0x1337;
pub const TOKEN_INIT_REPLY_ID: u64 = 0xbeef;
pub const FORWARD_REPLY_ID: u64 = 0xbabe;
pub const MULTIPLEX_REPLY_ID: u64 = 0xface;
pub const MM_RELAYER_FILL_REPLY_ID: u64 = 0xdead;
pub const MM_SOLVER_FILL_REPLY_ID: u64 = 0xb0cad0;
pub const UNSTAKE_REPLY_ID: u64 = 0xc0de;

pub const ZKGM_TOKEN_MINTER_LABEL: &str = "zkgm-token-minter";
pub const ZKGM_CW_ACCOUNT_LABEL: &str = "zkgm-cw-account";

pub const SOLVER_EVENT: &str = "solver";
pub const SOLVER_EVENT_MARKET_MAKER_ATTR: &str = "market_maker";

/// Instantiate `ucs03-zkgm`.
///
/// This will instantiate the minter contract with the provided [`TokenMinterInitMsg`][ucs03_zkgm_token_minter_api::TokenMinterInitMsg]. The admin of the minter contract is set to `ucs03-zkgm`. All migrations for the minter will be threaded through the `ucs03-zkgm` migrate entrypoint.
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

pub fn cw_account_salt() -> String {
    format!("{PROTOCOL_VERSION}/{ZKGM_CW_ACCOUNT_LABEL}")
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
            timeout_timestamp,
            timeout_height: _,
            salt,
            instruction,
        } => send(
            deps,
            info,
            channel_id,
            timeout_timestamp,
            salt,
            Instruction::abi_decode_params_validate(&instruction)?,
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
        ExecuteMsg::MigrateV1ToV2 {
            balance_migrations,
            wrapped_migrations,
        } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != config.admin {
                return Err(ContractError::OnlyAdmin);
            }
            migrate_v1_to_v2(deps, balance_migrations, wrapped_migrations)
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

/// Verifies a token order v2 instruction.
/// Handles different metadata types and validates unwrapping conditions.
pub fn verify_token_order_v2(
    deps: DepsMut,
    info: MessageInfo,
    funds: &mut Coins,
    channel_id: ChannelId,
    path: U256,
    order: &TokenOrderV2,
    response: &mut Response,
) -> Result<(), ContractError> {
    // Validate and get base token address
    let base_token_str =
        str::from_utf8(&order.base_token).map_err(|_| ContractError::InvalidBaseToken)?;

    // Get the token minter contract
    let minter = TOKEN_MINTER.load(deps.storage)?;

    if order.kind == TOKEN_ORDER_KIND_UNESCROW {
        // Get the origin path for the base token (from EVM: tokenOrigin[baseToken])
        let origin = TOKEN_ORIGIN.may_load(deps.storage, base_token_str.to_string())?;
        let (intermediate_channel_path, destination_channel_id) = if let Some(origin) = origin {
            let origin_u256 = U256::from_be_bytes(origin.to_be_bytes());
            pop_channel_from_path(origin_u256)
        } else {
            (U256::ZERO, None)
        };

        let is_inverse_intermediate_path = path == reverse_channel_path(intermediate_channel_path)?;
        let is_sending_back_to_same_channel = destination_channel_id == Some(channel_id);

        // Predict V1 wrapped token (EVM: _predictWrappedToken)
        let (wrapped_token_v1, _) = predict_wrapped_token(
            deps.as_ref(),
            &minter,
            intermediate_channel_path,
            channel_id,
            order.quote_token.to_vec().into(),
        )?;

        // Predict V2 wrapped token (EVM: _predictWrappedTokenFromMetadataImageV2)
        let metadata_image = METADATA_IMAGE_OF
            .may_load(deps.storage, base_token_str.to_string())?
            .unwrap_or_default();

        let (wrapped_token_v2, _) = predict_wrapped_token_from_metadata_image_v2(
            deps.as_ref(),
            &minter,
            intermediate_channel_path,
            channel_id,
            order.quote_token.to_vec().into(),
            metadata_image,
        )?;

        // Check if we're unwrapping (EVM: isUnwrappingV1 || isUnwrappingV2)
        let is_unwrapping_v1 = base_token_str == wrapped_token_v1;
        let is_unwrapping_v2 = base_token_str == wrapped_token_v2;
        let is_unwrapping = is_unwrapping_v1 || is_unwrapping_v2;

        if !(is_unwrapping && is_inverse_intermediate_path && is_sending_back_to_same_channel) {
            return Err(ContractError::InvalidTokenOrderUnescrow);
        }

        // Burn the wrapped token (EVM: IZkgmERC20(baseToken).burn(msg.sender, order.baseAmount))
        let base_amount = Uint256::from_be_bytes(order.base_amount.to_be_bytes());

        let mut funds_to_burn = vec![];
        if !funds.amount_of(base_token_str).is_zero() {
            let native_denom = Coin {
                denom: base_token_str.to_string(),
                amount: base_amount
                    .try_into()
                    .map_err(|_| ContractError::AmountOverflow)?,
            };
            funds.sub(native_denom.clone())?;
            funds_to_burn.push(native_denom);
        }

        *response = response.clone().add_message(make_wasm_msg(
            WrappedTokenMsg::BurnTokens {
                denom: base_token_str.to_string(),
                amount: base_amount
                    .try_into()
                    .map_err(|_| ContractError::AmountOverflow)?,
                burn_from_address: minter.clone(),
                sender: info.sender,
            },
            &minter,
            funds_to_burn,
        )?);
    } else {
        // Escrow tokens (EVM: _increaseOutstandingV2 + transfer logic)
        let base_amount = Uint256::from_be_bytes(order.base_amount.to_be_bytes());

        increase_channel_balance_v2(
            deps.storage,
            channel_id,
            path,
            base_token_str.to_string(),
            order.quote_token.clone().into(),
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
    let zkgm_packet = ZkgmPacket::abi_decode_params_validate(&packet.data)?;
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
        OP_TOKEN_ORDER => match instruction.version {
            INSTR_VERSION_1 => {
                let order = TokenOrderV1::abi_decode_params_validate(&instruction.operand)?;
                refund(deps, path, packet.source_channel_id, order)
            }
            INSTR_VERSION_2 => {
                let order = TokenOrderV2::abi_decode_params_validate(&instruction.operand)?;
                refund_v2(deps, path, packet.source_channel_id, order)
            }
            _ => Err(ContractError::UnsupportedVersion {
                version: instruction.version,
            }),
        },
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let mut response = Response::new();
            let batch = Batch::abi_decode_params_validate(&instruction.operand)?;
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
        OP_CALL => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Call::abi_decode_params_validate(&instruction.operand)?;
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
    multiplex: Call,
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
            timeout_height: MustBeZero,
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
    let zkgm_packet = ZkgmPacket::abi_decode_params_validate(&packet.data)?;
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
    let ack = Ack::abi_decode_params_validate(&ack)?;
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
        ack.inner_ack.into(),
    )
}

/// Handles the internal acknowledgement logic for a packet.
/// Processes acknowledgements based on instruction type and success status.
/// For successful acknowledgements, executes the appropriate success handlers.
/// For failed acknowledgements, executes refund/cleanup actions.
#[allow(clippy::too_many_arguments)]
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
        OP_TOKEN_ORDER => match instruction.version {
            INSTR_VERSION_1 => {
                let order = TokenOrderV1::abi_decode_params_validate(&instruction.operand)?;
                let order_ack = if successful {
                    Some(TokenOrderAck::abi_decode_params_validate(&ack)?)
                } else {
                    None
                };
                acknowledge_fungible_asset_order(
                    deps, env, info, packet, relayer, salt, path, order, order_ack,
                )
            }
            INSTR_VERSION_2 => {
                let order = TokenOrderV2::abi_decode_params_validate(&instruction.operand)?;
                let order_ack = if successful {
                    Some(TokenOrderAck::abi_decode_params_validate(&ack)?)
                } else {
                    None
                };
                acknowledge_fungible_asset_order_v2(
                    deps, env, info, packet, relayer, salt, path, order, order_ack,
                )
            }
            _ => Err(ContractError::UnsupportedVersion {
                version: instruction.version,
            }),
        },
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let mut response = Response::new();
            let batch = Batch::abi_decode_params_validate(&instruction.operand)?;
            let batch_ack = if successful {
                Some(BatchAck::abi_decode_params_validate(&ack)?)
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
        OP_CALL => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Call::abi_decode_params_validate(&instruction.operand)?;
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
    order: TokenOrderV1,
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
            decrease_channel_balance_v2(
                deps,
                source_channel,
                path,
                base_denom.clone(),
                order.quote_token.clone().into(),
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

fn refund_v2(
    deps: DepsMut,
    path: U256,
    source_channel: ChannelId,
    order: TokenOrderV2,
) -> Result<Response, ContractError> {
    // Extract sender and base token (EVM: address sender = address(bytes20(order.sender)))
    let sender = deps
        .api
        .addr_validate(str::from_utf8(&order.sender).map_err(|_| ContractError::InvalidSender)?)
        .map_err(|_| ContractError::UnableToValidateSender)?;
    let base_denom = String::from_utf8(order.base_token.to_vec())
        .map_err(|_| ContractError::InvalidBaseToken)?;
    let base_amount =
        u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;

    let minter = TOKEN_MINTER.load(deps.storage)?;
    let mut messages = Vec::<CosmosMsg>::new();

    if order.kind == TOKEN_ORDER_KIND_UNESCROW {
        // Mint tokens back to sender (EVM: IZkgmERC20(address(baseToken)).mint(sender, order.baseAmount))
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
        // Decrease channel balance and unescrow (EVM: _decreaseOutstandingV2 + safeTransfer)
        decrease_channel_balance_v2(
            deps,
            source_channel,
            path,
            base_denom.clone(),
            order.quote_token.clone().into(),
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

    Ok(Response::new().add_messages(messages))
}

#[allow(clippy::too_many_arguments)]
fn acknowledge_fungible_asset_order_v2(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    packet: Packet,
    _relayer: Addr,
    _salt: H256,
    path: U256,
    order: TokenOrderV2,
    order_ack: Option<TokenOrderAck>,
) -> Result<Response, ContractError> {
    if let Some(ack) = order_ack {
        // Successful acknowledgement
        match ack.fill_type {
            // The protocol filled, fee was paid to relayer.
            FILL_TYPE_PROTOCOL => Ok(Response::new()),
            // A market maker filled, we pay with the sent asset.
            FILL_TYPE_MARKETMAKER => {
                let market_maker = deps
                    .api
                    .addr_validate(
                        str::from_utf8(ack.market_maker.as_ref())
                            .map_err(|_| ContractError::InvalidReceiver)?,
                    )
                    .map_err(|_| ContractError::UnableToValidateMarketMaker)?;
                let base_denom = String::from_utf8(order.base_token.to_vec())
                    .map_err(|_| ContractError::InvalidBaseToken)?;
                let base_amount =
                    u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;

                let minter = TOKEN_MINTER.load(deps.storage)?;
                let mut messages = Vec::<CosmosMsg>::new();

                if order.kind == TOKEN_ORDER_KIND_UNESCROW {
                    // Mint tokens to market maker (EVM: IZkgmERC20(address(baseToken)).mint(marketMaker, order.baseAmount))
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
                    // Decrease channel balance and transfer (EVM: _decreaseOutstandingV2 + safeTransfer)
                    decrease_channel_balance_v2(
                        deps,
                        packet.source_channel_id,
                        path,
                        base_denom.clone(),
                        order.quote_token.clone().into(),
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

                Ok(Response::new().add_messages(messages))
            }
            fill_type => Err(ContractError::InvalidFillType { fill_type }),
        }
    } else {
        // Failed acknowledgement - refund (EVM: _refundV2(ibcPacket.sourceChannelId, path, order))
        refund_v2(deps, path, packet.source_channel_id, order)
    }
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
    order: TokenOrderV1,
    order_ack: Option<TokenOrderAck>,
) -> Result<Response, ContractError> {
    if let Some(ack) = order_ack {
        // Successful acknowledgement
        match ack.fill_type {
            FILL_TYPE_PROTOCOL => {
                // The protocol filled, fee was paid to relayer.
                Ok(Response::new())
            }
            FILL_TYPE_MARKETMAKER => {
                // A market maker filled, we pay with the sent asset.
                let market_maker = deps
                    .api
                    .addr_validate(
                        str::from_utf8(ack.market_maker.as_ref())
                            .map_err(|_| ContractError::InvalidReceiver)?,
                    )
                    .map_err(|_| ContractError::UnableToValidateMarketMaker)?;
                let base_denom = String::from_utf8(order.base_token.to_vec())
                    .map_err(|_| ContractError::InvalidBaseToken)?;
                let base_amount =
                    u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;

                let minter = TOKEN_MINTER.load(deps.storage)?;
                let mut messages = Vec::<CosmosMsg>::new();

                if !order.base_token_path.is_zero() {
                    // Mint tokens to market maker (EVM: IZkgmERC20(address(baseToken)).mint(marketMaker, order.baseAmount))
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
                    // Decrease channel balance and transfer (EVM: _decreaseOutstandingV2 + safeTransfer)
                    decrease_channel_balance_v2(
                        deps,
                        packet.source_channel_id,
                        path,
                        base_denom.clone(),
                        order.quote_token.clone().into(),
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

                Ok(Response::new().add_messages(messages))
            }
            fill_type => Err(ContractError::InvalidFillType { fill_type }),
        }
    } else {
        // Failed acknowledgement - refund (EVM: _refund(ibcPacket.sourceChannelId, path, order))
        refund(deps, path, packet.source_channel_id, order)
    }
}

#[allow(clippy::too_many_arguments)]
fn acknowledge_multiplex(
    deps: DepsMut,
    caller: Addr,
    packet: Packet,
    relayer: Addr,
    path: U256,
    multiplex: Call,
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
            timeout_height: MustBeZero,
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
    let zkgm_packet = ZkgmPacket::abi_decode_params_validate(&packet.data)?;
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
/// - Call operations
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
        OP_TOKEN_ORDER => match instruction.version {
            INSTR_VERSION_1 => {
                let order = TokenOrderV1::abi_decode_params_validate(&instruction.operand)?;
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
            INSTR_VERSION_2 => {
                let order = TokenOrderV2::abi_decode_params_validate(&instruction.operand)?;
                execute_fungible_asset_order_v2(
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
            _ => Err(ContractError::UnsupportedVersion {
                version: instruction.version,
            }),
        },
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let batch = Batch::abi_decode_params_validate(&instruction.operand)?;
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
        OP_CALL => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Call::abi_decode_params_validate(&instruction.operand)?;
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
            let forward = Forward::abi_decode_params_validate(&instruction.operand)?;
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
        OP_STAKE => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let stake = Stake::abi_decode_params_validate(&instruction.operand)?;
            execute_stake(deps, env, packet, stake, intent)
        }
        OP_UNSTAKE => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let unstake = Unstake::abi_decode_params_validate(&instruction.operand)?;
            execute_unstake(deps, env, packet, unstake, intent)
        }
        OP_WITHDRAW_STAKE => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let withdraw_stake = WithdrawStake::abi_decode_params_validate(&instruction.operand)?;
            execute_withdraw_stake(deps, env, packet, withdraw_stake, intent)
        }
        OP_WITHDRAW_REWARDS => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let withdraw_rewards =
                WithdrawRewards::abi_decode_params_validate(&instruction.operand)?;
            execute_withdraw_rewards(deps, env, packet, withdraw_rewards, intent)
        }
        _ => Err(ContractError::UnknownOpcode {
            opcode: instruction.opcode,
        }),
    }
}

fn calculate_stake_account_salt(channel_id: ChannelId, token_id: U256) -> Vec<u8> {
    keccak256((channel_id.raw(), token_id.to_be_bytes_vec()).abi_encode_params())
        .into_bytes()
        .to_vec()
}

fn predict_stake_account(
    deps: Deps,
    env: Env,
    channel_id: ChannelId,
    token_id: U256,
) -> Result<Addr, ContractError> {
    let Config { dummy_code_id, .. } = CONFIG.load(deps.storage)?;
    let code_hash = get_code_hash(deps, dummy_code_id)?;
    let token_addr = instantiate2_address(
        &code_hash.into_bytes(),
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        &calculate_stake_account_salt(channel_id, token_id),
    )?;
    Ok(deps.api.addr_humanize(&token_addr)?)
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
    if !is_allowed_forward_instruction(forward.instruction.opcode) {
        return Err(ContractError::InvalidForwardInstruction);
    }
    // We cannot allow market makers to fill packets containing forward
    // instruction. This would allow them to submit of a proof and fill via the
    // protocol on destination for a fake forward.

    // Instead, they must first fill on destination the orders, awaits finality
    // to settle the forward, then cascade acknowledge.
    if intent {
        return Ok(Response::new().add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: ACK_ERR_ONLY_MAKER.into(),
            },
            vec![],
        )?));
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
                timeout_height: 0,
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
    multiplex: Call,
    intent: bool,
) -> Result<Response, ContractError> {
    let contract_address = deps
        .api
        .addr_validate(
            str::from_utf8(&multiplex.contract_address)
                .map_err(|_| ContractError::InvalidContractAddress)?,
        )
        .map_err(|_| ContractError::UnableToValidateCallTarget)?;

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
            timeout_height: MustBeZero,
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
        if !is_allowed_batch_instruction(instruction.opcode) {
            return Err(ContractError::InvalidBatchInstruction);
        }
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
        MM_RELAYER_FILL_REPLY_ID,
    )))
}

#[allow(clippy::too_many_arguments)]
fn relayer_market_maker_fill_v2(
    deps: DepsMut,
    env: Env,
    funds: &mut Coins,
    caller: Addr,
    minter: Addr,
    order: TokenOrderV2,
) -> Result<Response, ContractError> {
    let quote_token_str = String::from_utf8(Vec::from(order.quote_token.clone()))
        .map_err(|_| ContractError::InvalidQuoteToken)?;

    let receiver = deps
        .api
        .addr_validate(
            str::from_utf8(order.receiver.as_ref()).map_err(|_| ContractError::InvalidReceiver)?,
        )
        .map_err(|_| ContractError::UnableToValidateReceiver)?;

    let quote_amount =
        u128::try_from(order.quote_amount).map_err(|_| ContractError::AmountOverflow)?;

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
        MM_RELAYER_FILL_REPLY_ID,
    )))
}

#[allow(clippy::too_many_arguments)]
fn solver_market_maker_fill_v2(
    caller: Addr,
    relayer: Addr,
    relayer_msg: Bytes,
    path: U256,
    packet: Packet,
    order: TokenOrderV2,
    intent: bool,
) -> Result<Response, ContractError> {
    let metadata = SolverMetadata::abi_decode_params_validate(&order.metadata)?;
    let solver = String::from_utf8(Vec::from(metadata.solverAddress))
        .map_err(|_| ContractError::InvalidSolverAddress)?;
    Ok(Response::new().add_submessage(SubMsg::reply_always(
        wasm_execute(
            solver,
            &SolverMsg::DoSolve {
                packet,
                order: order.into(),
                path: Uint256::from_be_bytes(path.to_be_bytes()),
                caller,
                relayer,
                relayer_msg,
                intent,
            },
            vec![],
        )?,
        MM_SOLVER_FILL_REPLY_ID,
    )))
}

#[allow(clippy::too_many_arguments)]
fn market_maker_fill_v2(
    deps: DepsMut,
    env: Env,
    funds: &mut Coins,
    caller: Addr,
    relayer: Addr,
    relayer_msg: Bytes,
    minter: Addr,
    path: U256,
    packet: Packet,
    order: TokenOrderV2,
    intent: bool,
) -> Result<Response, ContractError> {
    match order.kind {
        TOKEN_ORDER_KIND_SOLVE => {
            solver_market_maker_fill_v2(caller, relayer, relayer_msg, path, packet, order, intent)
        }
        _ => {
            MARKET_MAKER.save(deps.storage, &relayer_msg)?;
            relayer_market_maker_fill_v2(deps, env, funds, caller, minter, order)
        }
    }
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
    order: TokenOrderV1,
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
        decrease_channel_balance_v2(
            deps,
            packet.destination_channel_id,
            reverse_channel_path(path)?,
            quote_token_str.clone(),
            order.base_token.clone().into(),
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
                ack: TokenOrderAck {
                    fill_type: FILL_TYPE_PROTOCOL,
                    market_maker: Default::default(),
                }
                .abi_encode_params()
                .into(),
            },
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn execute_fungible_asset_order_v2(
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
    order: TokenOrderV2,
    intent: bool,
) -> Result<Response, ContractError> {
    // Extract quote token and receiver (EVM: address quoteToken = address(bytes20(order.quoteToken)))
    let quote_token_str = String::from_utf8(Vec::from(order.quote_token.clone()))
        .map_err(|_| ContractError::InvalidQuoteToken)?;
    let receiver = deps
        .api
        .addr_validate(
            str::from_utf8(order.receiver.as_ref()).map_err(|_| ContractError::InvalidReceiver)?,
        )
        .map_err(|_| ContractError::UnableToValidateReceiver)?;

    // For intent packets, only market maker can fill
    if intent || order.kind == TOKEN_ORDER_KIND_SOLVE {
        let minter = TOKEN_MINTER.load(deps.storage)?;
        return market_maker_fill_v2(
            deps,
            env,
            funds,
            caller,
            relayer,
            relayer_msg,
            minter,
            path,
            packet,
            order,
            intent,
        );
    }

    let base_amount_covers_quote_amount = order.base_amount >= order.quote_amount;

    // Direct unescrow path (EVM: order.kind == TOKEN_ORDER_KIND_UNESCROW && baseAmountCoversQuoteAmount)
    if order.kind == TOKEN_ORDER_KIND_UNESCROW && base_amount_covers_quote_amount {
        let base_amount =
            u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;
        let quote_amount =
            u128::try_from(order.quote_amount).map_err(|_| ContractError::AmountOverflow)?;

        rate_limit(
            deps.storage,
            quote_token_str.clone(),
            quote_amount,
            env.block.time.seconds(),
        )?;

        protocol_fill_unescrow_v2(
            deps,
            env,
            packet.destination_channel_id,
            path,
            Vec::from(order.base_token.clone()).into(),
            quote_token_str,
            receiver,
            relayer,
            base_amount.into(),
            quote_amount.into(),
        )
    } else {
        let minter = TOKEN_MINTER.load(deps.storage)?;
        let mut wrapped_token: Option<String> = None;

        // Decode metadata (EVM: ZkgmLib.decodeTokenMetadata(order.metadata))
        let metadata = TokenMetadata::abi_decode_params_validate(&order.metadata).ok();

        if order.kind == TOKEN_ORDER_KIND_ESCROW {
            // Get metadata image for quote token (EVM: metadataImageOf[quoteToken])
            let metadata_image = METADATA_IMAGE_OF
                .may_load(deps.storage, quote_token_str.clone())?
                .unwrap_or_default();

            if metadata_image.is_zero() {
                // V1 prediction
                let (pred_wrapped, _) = predict_wrapped_token(
                    deps.as_ref(),
                    &minter,
                    path,
                    packet.destination_channel_id,
                    Vec::from(order.base_token.clone()).into(),
                )?;
                wrapped_token = Some(pred_wrapped);
            } else {
                // V2 prediction
                let (pred_wrapped, _) = predict_wrapped_token_from_metadata_image_v2(
                    deps.as_ref(),
                    &minter,
                    path,
                    packet.destination_channel_id,
                    Vec::from(order.base_token.clone()).into(),
                    metadata_image,
                )?;
                wrapped_token = Some(pred_wrapped);
            }
        } else if order.kind == TOKEN_ORDER_KIND_INITIALIZE {
            if let Some(ref metadata) = metadata {
                let (pred_wrapped, _) = predict_wrapped_token_v2(
                    deps.as_ref(),
                    &minter,
                    path,
                    packet.destination_channel_id,
                    Vec::from(order.base_token.clone()).into(),
                    metadata.clone(),
                )?;
                if quote_token_str != pred_wrapped {
                    return Err(ContractError::InvalidTokenOrderKind);
                }
                wrapped_token = Some(pred_wrapped);
            }
        }

        // Protocol fill if quote token matches wrapped token and base covers quote
        if let Some(wrapped_denom) = wrapped_token {
            if quote_token_str == wrapped_denom && base_amount_covers_quote_amount {
                let base_amount =
                    u128::try_from(order.base_amount).map_err(|_| ContractError::AmountOverflow)?;
                let quote_amount = u128::try_from(order.quote_amount)
                    .map_err(|_| ContractError::AmountOverflow)?;

                rate_limit(
                    deps.storage,
                    quote_token_str.clone(),
                    quote_amount,
                    env.block.time.seconds(),
                )?;

                // The asset can only be deployed if the metadata preimage is provided
                let can_deploy = order.kind == TOKEN_ORDER_KIND_INITIALIZE;

                return protocol_fill_mint(
                    deps,
                    env,
                    packet.destination_channel_id,
                    path,
                    Vec::from(order.base_token.clone()).into(),
                    wrapped_denom,
                    receiver,
                    relayer,
                    base_amount.into(),
                    quote_amount.into(),
                    metadata,
                    can_deploy,
                );
            }
        }

        // Fall back to market maker fill
        market_maker_fill_v2(
            deps,
            env,
            funds,
            caller,
            relayer,
            relayer_msg,
            minter,
            path,
            packet,
            order,
            intent,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn protocol_fill_mint(
    deps: DepsMut,
    env: Env,
    channel_id: ChannelId,
    path: U256,
    base_token: Bytes,
    wrapped_token: String,
    receiver: Addr,
    relayer: Addr,
    base_amount: Uint256,
    quote_amount: Uint256,
    metadata: Option<TokenMetadata>,
    can_deploy: bool,
) -> Result<Response, ContractError> {
    let minter = TOKEN_MINTER.load(deps.storage)?;
    let mut messages = Vec::new();

    let fee_amount = base_amount
        .checked_sub(quote_amount)
        .map_err(|_| ContractError::AmountOverflow)?;

    // Deploy wrapped token if needed and can deploy
    if can_deploy && !HASH_TO_FOREIGN_TOKEN.has(deps.storage, wrapped_token.clone()) {
        if let Some(metadata) = metadata {
            // Create the wrapped token if it doesn't exist
            HASH_TO_FOREIGN_TOKEN.save(
                deps.storage,
                wrapped_token.clone(),
                &Vec::from(base_token.clone()).into(),
            )?;

            // Create the token with metadata from the preimage
            messages.push(SubMsg::reply_never(make_wasm_msg(
                WrappedTokenMsg::CreateDenomV2 {
                    subdenom: wrapped_token.clone(),
                    path: path.to_be_bytes::<32>().to_vec().into(),
                    channel_id,
                    token: Vec::from(base_token.clone()).into(),
                    implementation: metadata.implementation.to_vec().into(),
                    initializer: metadata.initializer.to_vec().into(),
                },
                &minter,
                vec![],
            )?));

            // Save the token origin for future unwrapping
            TOKEN_ORIGIN.save(
                deps.storage,
                wrapped_token.clone(),
                &Uint256::from_be_bytes(update_channel_path(path, channel_id)?.to_be_bytes()),
            )?;

            // Save metadata image
            METADATA_IMAGE_OF.save(
                deps.storage,
                wrapped_token.clone(),
                &keccak256(metadata.abi_encode_params()),
            )?;
        }
    }

    // Mint the quote amount to the receiver
    if !quote_amount.is_zero() {
        messages.push(SubMsg::reply_never(make_wasm_msg(
            WrappedTokenMsg::MintTokens {
                denom: wrapped_token.clone(),
                amount: Uint128::try_from(quote_amount)
                    .map_err(|_| ContractError::AmountOverflow)?,
                mint_to_address: receiver,
            },
            &minter,
            vec![],
        )?));
    }

    // Mint any fee to the relayer
    if !fee_amount.is_zero() {
        messages.push(SubMsg::reply_never(make_wasm_msg(
            WrappedTokenMsg::MintTokens {
                denom: wrapped_token,
                amount: Uint128::try_from(fee_amount).map_err(|_| ContractError::AmountOverflow)?,
                mint_to_address: relayer,
            },
            &minter,
            vec![],
        )?));
    }

    // Return success acknowledgement with protocol fill type
    Ok(Response::new()
        .add_submessages(messages)
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: TokenOrderAck {
                    fill_type: FILL_TYPE_PROTOCOL,
                    market_maker: Default::default(),
                }
                .abi_encode_params()
                .into(),
            },
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn protocol_fill_unescrow_v2(
    deps: DepsMut,
    env: Env,
    channel_id: ChannelId,
    path: U256,
    base_token: Bytes,
    quote_token_str: String,
    receiver: Addr,
    relayer: Addr,
    base_amount: Uint256,
    quote_amount: Uint256,
) -> Result<Response, ContractError> {
    let minter = TOKEN_MINTER.load(deps.storage)?;
    let mut messages = Vec::new();

    let fee_amount = base_amount
        .checked_sub(quote_amount)
        .map_err(|_| ContractError::AmountOverflow)?;

    // If the base token path is being unwrapped, it's escrowed balance will be non zero.
    // EVM: _decreaseOutstandingV2(channelId, ZkgmLib.reverseChannelPath(path), quoteToken, baseToken, baseAmount)
    decrease_channel_balance_v2(
        deps,
        channel_id,
        reverse_channel_path(path)?,
        quote_token_str.clone(), // EVM param: quoteToken (address) -> EVM function param: baseToken
        base_token,              // EVM param: baseToken (bytes) -> EVM function param: quoteToken
        base_amount,
    )?;

    // Transfer the quote amount to the receiver
    if !quote_amount.is_zero() {
        messages.push(SubMsg::reply_never(make_wasm_msg(
            LocalTokenMsg::Unescrow {
                denom: quote_token_str.clone(),
                recipient: receiver.into_string(),
                amount: Uint128::try_from(quote_amount)
                    .map_err(|_| ContractError::AmountOverflow)?,
            },
            &minter,
            vec![],
        )?));
    }

    // Transfer any fee to the relayer
    if !fee_amount.is_zero() {
        messages.push(SubMsg::reply_never(make_wasm_msg(
            LocalTokenMsg::Unescrow {
                denom: quote_token_str,
                recipient: relayer.into_string(),
                amount: Uint128::try_from(fee_amount).map_err(|_| ContractError::AmountOverflow)?,
            },
            &minter,
            vec![],
        )?));
    }

    // Return success acknowledgement with protocol fill type
    Ok(Response::new()
        .add_submessages(messages)
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: TokenOrderAck {
                    fill_type: FILL_TYPE_PROTOCOL,
                    market_maker: Default::default(),
                }
                .abi_encode_params()
                .into(),
            },
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn execute_stake(
    deps: DepsMut,
    env: Env,
    packet: Packet,
    stake: Stake,
    intent: bool,
) -> Result<Response, ContractError> {
    // Market makers not allowed to fill staking requests.
    if intent {
        return Ok(Response::new().add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: ACK_ERR_ONLY_MAKER.into(),
            },
            vec![],
        )?));
    }

    let validator =
        str::from_utf8(&stake.validator).map_err(|_| ContractError::InvalidValidator)?;
    let governance_token = str::from_utf8(&stake.governance_token)
        .map_err(|_| ContractError::InvalidGovernanceToken)?;
    let stake_amount = u128::try_from(stake.amount).map_err(|_| ContractError::AmountOverflow)?;

    let stake_account = predict_stake_account(
        deps.as_ref(),
        env.clone(),
        packet.destination_channel_id,
        stake.token_id,
    )?;

    if deps
        .querier
        .query_wasm_contract_info(&stake_account)
        .is_ok()
    {
        return Err(ContractError::StakingAccountAlreadyExist {
            stake: Box::new(stake),
            account: stake_account,
        });
    }

    let mut messages = Vec::new();
    let config = CONFIG.load(deps.storage)?;

    // 1. Create the staking account
    messages.push(
        WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.dummy_code_id,
            label: format!(
                "ucs03-staking-account:{}-{}",
                packet.destination_channel_id, stake.token_id
            ),
            msg: to_json_binary(&cosmwasm_std::Empty {})?,
            funds: vec![],
            salt: Binary::new(calculate_stake_account_salt(
                packet.destination_channel_id,
                stake.token_id,
            )),
        }
        .into(),
    );
    messages.push(
        WasmMsg::Migrate {
            contract_addr: stake_account.to_string(),
            new_code_id: config.cw_account_code_id,
            msg: to_json_binary(&UpgradeMsg::<_, Empty>::Init(
                cw_account::msg::InstantiateMsg {
                    owner: env.contract.address.clone(),
                },
            ))?,
        }
        .into(),
    );

    // 2. Unescrow the gov tokens to the stake account.
    let minter = TOKEN_MINTER.load(deps.storage)?;
    decrease_channel_balance_v2(
        deps,
        packet.destination_channel_id,
        U256::ZERO,
        governance_token.into(),
        stake.governance_token_wrapped.0.to_vec().into(),
        stake_amount.into(),
    )?;

    messages.push(make_wasm_msg(
        LocalTokenMsg::Unescrow {
            denom: governance_token.into(),
            recipient: stake_account.clone().into(),
            amount: stake_amount.into(),
        },
        minter,
        vec![],
    )?);

    // 3. Delegate the token to the validator
    messages.push(
        wasm_execute(
            stake_account.clone(),
            &cw_account::msg::ExecuteMsg {
                messages: vec![StakingMsg::Delegate {
                    validator: validator.into(),
                    amount: Coin::new(stake_amount, governance_token),
                }
                .into()],
            },
            vec![],
        )?
        .into(),
    );

    Ok(Response::new()
        .add_messages(messages)
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: TAG_ACK_SUCCESS.abi_encode().into(),
            },
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn execute_unstake(
    deps: DepsMut,
    env: Env,
    packet: Packet,
    unstake: Unstake,
    intent: bool,
) -> Result<Response, ContractError> {
    // Market makers not allowed to fill unstaking requests.
    if intent {
        return Ok(Response::new().add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: ACK_ERR_ONLY_MAKER.into(),
            },
            vec![],
        )?));
    }

    let validator =
        str::from_utf8(&unstake.validator).map_err(|_| ContractError::InvalidValidator)?;
    let governance_token = str::from_utf8(&unstake.governance_token)
        .map_err(|_| ContractError::InvalidGovernanceToken)?;

    let stake_account = predict_stake_account(
        deps.as_ref(),
        env.clone(),
        packet.destination_channel_id,
        unstake.token_id,
    )?;

    let stake_amount = deps
        .querier
        .query_delegation(stake_account.clone(), validator)?
        .map(|delegation| delegation.amount.amount)
        .unwrap_or(0u128.into());

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        wasm_execute(
            stake_account.clone(),
            &cw_account::msg::ExecuteMsg {
                messages: vec![
                    // Withdraw the pending rewards because we won't earn any
                    // new reward after undelegating
                    DistributionMsg::WithdrawDelegatorReward {
                        validator: validator.into(),
                    }
                    .into(),
                    StakingMsg::Undelegate {
                        validator: validator.into(),
                        amount: Coin::new(stake_amount, governance_token),
                    }
                    .into(),
                ],
            },
            vec![],
        )?,
        UNSTAKE_REPLY_ID,
    )))
}

#[allow(clippy::too_many_arguments)]
fn execute_withdraw_stake(
    deps: DepsMut,
    env: Env,
    packet: Packet,
    withdraw_stake: WithdrawStake,
    intent: bool,
) -> Result<Response, ContractError> {
    // Market makers not allowed to fill unstaking requests.
    if intent {
        return Ok(Response::new().add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: ACK_ERR_ONLY_MAKER.into(),
            },
            vec![],
        )?));
    }
    let minter = TOKEN_MINTER.load(deps.storage)?;

    let governance_token = str::from_utf8(&withdraw_stake.governance_token)
        .map_err(|_| ContractError::InvalidGovernanceToken)?;

    let stake_account = predict_stake_account(
        deps.as_ref(),
        env.clone(),
        packet.destination_channel_id,
        withdraw_stake.token_id,
    )?;

    let coin = deps
        .querier
        .query_balance(stake_account.clone(), governance_token)?;

    increase_channel_balance_v2(
        deps.storage,
        packet.destination_channel_id,
        U256::ZERO,
        governance_token.to_string(),
        withdraw_stake.governance_token_wrapped.0.to_vec().into(),
        coin.amount.u128().into(),
    )?;

    Ok(Response::new()
        .add_message(wasm_execute(
            stake_account.clone(),
            &cw_account::msg::ExecuteMsg {
                messages: vec![BankMsg::Send {
                    to_address: minter.into(),
                    amount: vec![coin.clone()],
                }
                .into()],
            },
            vec![],
        )?)
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: WithdrawStakeAck {
                    amount: U256::from(coin.amount.u128()),
                }
                .abi_encode_params()
                .into(),
            },
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn execute_withdraw_rewards(
    deps: DepsMut,
    env: Env,
    packet: Packet,
    withdraw_rewards: WithdrawRewards,
    intent: bool,
) -> Result<Response, ContractError> {
    // Market makers not allowed to fill unstaking requests.
    if intent {
        return Ok(Response::new().add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: ACK_ERR_ONLY_MAKER.into(),
            },
            vec![],
        )?));
    }
    let minter = TOKEN_MINTER.load(deps.storage)?;

    let validator =
        str::from_utf8(&withdraw_rewards.validator).map_err(|_| ContractError::InvalidValidator)?;
    let governance_token = str::from_utf8(&withdraw_rewards.governance_token)
        .map_err(|_| ContractError::InvalidGovernanceToken)?;

    let stake_account = predict_stake_account(
        deps.as_ref(),
        env.clone(),
        packet.destination_channel_id,
        withdraw_rewards.token_id,
    )?;

    let reward = deps
        .querier
        .query_delegation_rewards(stake_account.clone(), validator)?
        .into_iter()
        .find_map(|reward| {
            if reward.denom == governance_token {
                Some(reward)
            } else {
                None
            }
        })
        .unwrap_or(DecCoin::new(Decimal256::zero(), governance_token));

    let reward = Coin::new(
        Uint128::try_from(reward.amount.to_uint_floor()).expect("impossible"),
        governance_token,
    );

    increase_channel_balance_v2(
        deps.storage,
        packet.destination_channel_id,
        U256::ZERO,
        governance_token.to_string(),
        withdraw_rewards.governance_token_wrapped.0.to_vec().into(),
        reward.amount.u128().into(),
    )?;

    Ok(Response::new()
        .add_message(wasm_execute(
            stake_account.clone(),
            &cw_account::msg::ExecuteMsg {
                messages: vec![
                    DistributionMsg::WithdrawDelegatorReward {
                        validator: validator.into(),
                    }
                    .into(),
                    BankMsg::Send {
                        to_address: minter.into(),
                        amount: vec![reward.clone()],
                    }
                    .into(),
                ],
            },
            vec![],
        )?)
        .add_message(wasm_execute(
            env.contract.address,
            &ExecuteMsg::InternalWriteAck {
                ack: WithdrawRewardsAck {
                    amount: U256::from(reward.amount.u128()),
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
                                // this case (call/token order). We keep this assertion for future upgrades.
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
                        return Err(ContractError::AsyncCallUnsupported);
                    }

                    Ok(Response::new().add_message(wasm_execute(
                        env.contract.address,
                        &ExecuteMsg::InternalWriteAck {
                            ack: Vec::from(acknowledgement).into(),
                        },
                        vec![],
                    )?))
                }
                SubMsgResult::Err(error) => Err(ContractError::CallError { error }),
            }
        }
        MM_RELAYER_FILL_REPLY_ID => {
            let market_maker = MARKET_MAKER.load(deps.storage)?;
            MARKET_MAKER.remove(deps.storage);
            match reply.result {
                SubMsgResult::Ok(_) => Ok(Response::new().add_message(wasm_execute(
                    env.contract.address,
                    &ExecuteMsg::InternalWriteAck {
                        ack: TokenOrderAck {
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
        MM_SOLVER_FILL_REPLY_ID => {
            let extract_market_maker = |x: &SubMsgResponse| {
                x.events.iter().find_map(|e| {
                    if e.ty == format!("wasm-{}", SOLVER_EVENT) {
                        e.attributes
                            .iter()
                            .find(|a| a.key == SOLVER_EVENT_MARKET_MAKER_ATTR)
                            .and_then(|a| Bytes::<HexPrefixed>::from_str(&a.value).ok())
                    } else {
                        None
                    }
                })
            };
            match reply.result {
                SubMsgResult::Ok(reply_data) if extract_market_maker(&reply_data).is_some() => {
                    let market_maker = extract_market_maker(&reply_data).expect("impossible");
                    Ok(Response::new().add_message(wasm_execute(
                        env.contract.address,
                        &ExecuteMsg::InternalWriteAck {
                            ack: TokenOrderAck {
                                fill_type: FILL_TYPE_MARKETMAKER,
                                market_maker: market_maker.into(),
                            }
                            .abi_encode_params()
                            .into(),
                        },
                        vec![],
                    )?))
                }
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
                // Solver didn't provide a maker address.
                _ => Ok(Response::new().add_message(wasm_execute(
                    env.contract.address,
                    &ExecuteMsg::InternalWriteAck {
                        ack: ACK_ERR_ONLY_MAKER.into(),
                    },
                    vec![],
                )?)),
            }
        }
        UNSTAKE_REPLY_ID => match reply.result {
            SubMsgResult::Ok(response) => {
                let completion_time_utc_str = response
                    .events
                    .into_iter()
                    .find_map(|e| {
                        if e.ty == "unbond" {
                            e.attributes
                                .into_iter()
                                .find(|a| a.key == "completion_time")
                        } else {
                            None
                        }
                    })
                    .expect("impossible");
                let completion_time_utc =
                    DateTime::parse_from_rfc3339(&completion_time_utc_str.value)
                        .expect("impossible");
                Ok(Response::new().add_message(wasm_execute(
                    env.contract.address,
                    &ExecuteMsg::InternalWriteAck {
                        ack: UnstakeAck {
                            completion_time: U256::from(completion_time_utc.timestamp()),
                        }
                        .abi_encode_params()
                        .into(),
                    },
                    vec![],
                )?))
            }
            SubMsgResult::Err(_) => panic!("reply_on_success with error branch, impossible"),
        },
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
        OP_TOKEN_ORDER => match instruction.version {
            INSTR_VERSION_1 => {
                let order = TokenOrderV1::abi_decode_params_validate(&instruction.operand)?;
                verify_fungible_asset_order(deps, info, funds, channel_id, path, &order, response)
            }
            INSTR_VERSION_2 => {
                let order = TokenOrderV2::abi_decode_params_validate(&instruction.operand)?;
                verify_token_order_v2(deps, info, funds, channel_id, path, &order, response)
            }
            _ => Err(ContractError::UnsupportedVersion {
                version: instruction.version,
            }),
        },
        OP_BATCH => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let batch = Batch::abi_decode_params_validate(&instruction.operand)?;
            verify_batch(deps, info, funds, channel_id, path, &batch, response)
        }
        OP_FORWARD => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let forward = Forward::abi_decode_params_validate(&instruction.operand)?;
            verify_forward(deps, info, funds, channel_id, &forward, response)
        }
        OP_CALL => {
            if instruction.version > INSTR_VERSION_0 {
                return Err(ContractError::UnsupportedVersion {
                    version: instruction.version,
                });
            }
            let multiplex = Call::abi_decode_params_validate(&instruction.operand)?;
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
    order: &TokenOrderV1,
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

    let base_amount = Uint256::from_be_bytes(order.base_amount.to_be_bytes());
    let mut funds_to_attach = vec![];
    if !base_amount.is_zero() && !funds.amount_of(base_token_str).is_zero() {
        let native_denom = Coin {
            denom: base_token_str.to_string(),
            amount: base_amount
                .try_into()
                .map_err(|_| ContractError::AmountOverflow)?,
        };
        funds.sub(native_denom.clone())?;
        funds_to_attach.push(native_denom);
    }

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
            funds_to_attach,
        )?);
    } else {
        if !order.base_token_path.is_zero() {
            return Err(ContractError::InvalidAssetOrigin {
                actual: order.base_token_path,
                expected: U256::ZERO,
            });
        }
        increase_channel_balance_v2(
            deps.storage,
            channel_id,
            path,
            base_token_str.to_string(),
            order.quote_token.clone().into(),
            base_amount,
        )?;
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
            funds_to_attach,
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
    multiplex: &Call,
    sender: Addr,
    _response: &mut Response,
) -> Result<(), ContractError> {
    // Verify the sender matches msg.sender
    let multiplex_sender =
        str::from_utf8(&multiplex.sender).map_err(|_| ContractError::InvalidSender)?;
    if multiplex_sender != sender.as_str() {
        return Err(ContractError::InvalidCallSender);
    }
    Ok(())
}

/// Checks if an opcode is allowed in a batch instruction
fn is_allowed_batch_instruction(opcode: u8) -> bool {
    opcode == OP_CALL || opcode == OP_TOKEN_ORDER
}

/// Checks if an opcode is allowed in a forward instruction
fn is_allowed_forward_instruction(opcode: u8) -> bool {
    opcode == OP_CALL || opcode == OP_TOKEN_ORDER || opcode == OP_BATCH
}

#[allow(clippy::too_many_arguments)]
pub fn send(
    mut deps: DepsMut,
    info: MessageInfo,
    channel_id: ChannelId,
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
    dummy_code_id: Option<u64>,
    cw_account_code_id: Option<u64>,
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
                if let Some(dummy_code_id) = migrate_msg.dummy_code_id {
                    config.dummy_code_id = dummy_code_id;
                }
                if let Some(cw_account_code_id) = migrate_msg.cw_account_code_id {
                    config.cw_account_code_id = cw_account_code_id;
                }
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::PredictStakeAccount {
            channel_id,
            token_id,
        } => Ok(to_json_binary(
            predict_stake_account(
                deps,
                env,
                channel_id,
                U256::from_be_bytes(token_id.to_be_bytes()),
            )?
            .as_str(),
        )?),
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
                // HACK(benluelo): This previously returned hex bytes, but was accidentally changed. This is a hotfix until we decide on a format.
                wrapped_token: <Bytes>::from(token.into_bytes()).to_string(),
            })?)
        }
        QueryMsg::PredictWrappedTokenV2 {
            path,
            channel_id,
            token,
            metadata_image,
        } => {
            let minter = TOKEN_MINTER.load(deps.storage)?;
            let (token, _) = predict_wrapped_token_from_metadata_image_v2(
                deps,
                &minter,
                path.parse().map_err(ContractError::InvalidPath)?,
                channel_id,
                token,
                metadata_image,
            )?;
            Ok(to_json_binary(&PredictWrappedTokenResponse {
                wrapped_token: token,
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
            let balance = DEPRECATED_CHANNEL_BALANCE_V1.load(
                deps.storage,
                (channel_id.get().into(), path.to_be_bytes().into(), denom),
            )?;
            Ok(to_json_binary(&balance)?)
        }
        QueryMsg::GetChannelBalanceV2 {
            channel_id,
            path,
            base_token,
            quote_token,
        } => {
            let balance = CHANNEL_BALANCE_V2.load(
                deps.storage,
                (
                    channel_id.get().into(),
                    (
                        path.to_be_bytes().into(),
                        base_token,
                        quote_token.into_vec(),
                    ),
                ),
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

/// Decrease the outstanding balance of a (channel, path, base_token, quote_token) combination for V2 tokens.
/// Matches EVM: _decreaseOutstandingV2(uint32 sourceChannelId, uint256 path, address baseToken, bytes calldata quoteToken, uint256 amount)
pub fn decrease_channel_balance_v2(
    deps: DepsMut,
    channel_id: ChannelId,
    path: U256,
    base_token: String, // EVM: address baseToken
    quote_token: Bytes, // EVM: bytes calldata quoteToken
    amount: Uint256,
) -> Result<(), ContractError> {
    // Storage key: channelBalanceV2[sourceChannelId][path][baseToken][quoteToken]
    CHANNEL_BALANCE_V2.update(
        deps.storage,
        (
            channel_id.raw(),
            (
                path.to_be_bytes::<32>().to_vec(),
                base_token,
                quote_token.to_vec(),
            ),
        ),
        |balance| match balance {
            Some(value) => value
                .checked_sub(amount)
                .map_err(|_| ContractError::InvalidChannelBalance),
            None => Err(ContractError::InvalidChannelBalance),
        },
    )?;
    Ok(())
}

/// Increase the outstanding balance for a (channel, path, base_token, quote_token) combination for V2 tokens.
/// Matches EVM: _increaseOutstandingV2(uint32 sourceChannelId, uint256 path, address baseToken, bytes calldata quoteToken, uint256 amount)
pub fn increase_channel_balance_v2(
    storage: &mut dyn Storage,
    channel_id: ChannelId,
    path: U256,
    base_token: String, // EVM: address baseToken
    quote_token: Bytes, // EVM: bytes calldata quoteToken
    amount: Uint256,
) -> Result<(), ContractError> {
    // Storage key: channelBalanceV2[sourceChannelId][path][baseToken][quoteToken]
    CHANNEL_BALANCE_V2.update(
        storage,
        (
            channel_id.raw(),
            (
                path.to_be_bytes::<32>().to_vec(),
                base_token,
                quote_token.to_vec(),
            ),
        ),
        |balance| match balance {
            Some(value) => value
                .checked_add(amount)
                .map_err(|_| ContractError::InvalidChannelBalance),
            None => Ok(amount),
        },
    )?;
    Ok(())
}

/// Predict a wrapped token address for V2 tokens using metadata image
fn predict_wrapped_token_from_metadata_image_v2(
    deps: Deps,
    minter: &Addr,
    path: U256,
    channel_id: ChannelId,
    token: Bytes,
    metadata_image: H256,
) -> StdResult<(String, Bytes)> {
    let wrapped_token = deps
        .querier
        .query::<ucs03_zkgm_token_minter_api::PredictWrappedTokenResponse>(&QueryRequest::Wasm(
            cosmwasm_std::WasmQuery::Smart {
                contract_addr: minter.to_string(),
                msg: to_json_binary(
                    &ucs03_zkgm_token_minter_api::QueryMsg::PredictWrappedTokenV2 {
                        path: path.to_string(),
                        channel_id,
                        token: Binary::new(token.to_vec()),
                        metadata_image,
                    },
                )?,
            },
        ))?
        .wrapped_token;

    // Return both the string and bytes representation
    Ok((wrapped_token.clone(), wrapped_token.as_bytes().into()))
}

/// Predict a wrapped token address for V2 tokens using full metadata
fn predict_wrapped_token_v2(
    deps: Deps,
    minter: &Addr,
    path: U256,
    channel_id: ChannelId,
    token: Bytes,
    metadata: TokenMetadata,
) -> StdResult<(String, Bytes)> {
    // Hash the metadata to get the metadata image
    let metadata_bytes = metadata.abi_encode_params();
    let metadata_image = keccak256(metadata_bytes);

    // Use the metadata image to predict the token
    predict_wrapped_token_from_metadata_image_v2(
        deps,
        minter,
        path,
        channel_id,
        token,
        metadata_image,
    )
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

fn migrate_v1_to_v2(
    deps: DepsMut,
    balance_migrations: Vec<V1ToV2Migration>,
    wrapped_migrations: Vec<V1ToV2WrappedMigration>,
) -> Result<Response, ContractError> {
    for migration in balance_migrations {
        let key = (
            migration.channel_id.raw(),
            migration.path.to_be_bytes().to_vec(),
            migration.base_token.clone(),
        );

        let balance = DEPRECATED_CHANNEL_BALANCE_V1.may_load(deps.storage, key.clone())?;

        if let Some(balance) = balance {
            if balance.is_zero() {
                return Err(StdError::generic_err("no balance").into());
            }

            // Remove from V1 storage
            DEPRECATED_CHANNEL_BALANCE_V1.remove(deps.storage, key);

            // Add to V2 storage
            CHANNEL_BALANCE_V2.update(
                deps.storage,
                (
                    migration.channel_id.raw(),
                    (
                        migration.path.to_be_bytes().to_vec(),
                        migration.base_token,
                        migration.quote_token.to_vec(),
                    ),
                ),
                |existing_balance| match existing_balance {
                    Some(existing) => existing
                        .checked_add(balance)
                        .map_err(|_| ContractError::InvalidChannelBalance),
                    None => Ok(balance),
                },
            )?;
        } else {
            return Err(StdError::generic_err("no balance").into());
        }
    }

    let mut events = Vec::new();
    for migration in wrapped_migrations {
        let event = new_wrapped_token_event(
            U256::from_be_bytes(migration.path.to_be_bytes()),
            migration.channel_id,
            migration.base_token.into_vec(),
            &migration.quote_token,
            Vec::new(),
            WrappedTokenKind::Protocol,
        );
        events.push(event);
    }

    Ok(Response::new()
        .add_event(Event::new("v1_to_v2_migration"))
        .add_events(events))
}

pub fn encode_multiplex_calldata(path: U256, sender: Bytes, contract_calldata: Bytes) -> Vec<u8> {
    (path, sender, contract_calldata).abi_encode()
}
