use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
    wasm_execute, Addr, Binary, Coin, Coins, Deps, DepsMut, Empty, Env, MessageInfo, OwnedDeps,
    Response, StdError, StdResult, Uint256,
};
use cw20::{Cw20Coin, Cw20QueryMsg, TokenInfoResponse};
use cw20_token_minter::contract::save_native_token;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, SudoMsg};
use cw_storage_plus::Map;
use ibc_union_msg::module::IbcUnionMsg;
use ibc_union_spec::{path::commit_packets, ChannelId, ConnectionId, Packet};
use unionlabs::primitives::{Bytes, H256};

use crate::{
    com::{
        Ack, Batch, Forward, FungibleAssetOrder, FungibleAssetOrderAck, Instruction, Multiplex,
        ZkgmPacket, FILL_TYPE_PROTOCOL, FORWARD_SALT_MAGIC, INSTR_VERSION_0, INSTR_VERSION_1,
        OP_BATCH, OP_FORWARD, OP_FUNGIBLE_ASSET_ORDER, OP_MULTIPLEX, TAG_ACK_FAILURE,
        TAG_ACK_SUCCESS,
    },
    contract::{
        dequeue_channel_from_path, execute, increase_channel_balance, instantiate,
        is_forwarded_packet, migrate, pop_channel_from_path, query, reply, reverse_channel_path,
        tint_forward_salt, update_channel_path, verify_batch, verify_forward, verify_internal,
        verify_multiplex, PROTOCOL_VERSION,
    },
    msg::{
        Config, ExecuteMsg, InitMsg, PredictWrappedTokenResponse, QueryMsg, TokenMinterInitParams,
    },
    state::{CHANNEL_BALANCE, CONFIG, EXECUTING_PACKET, TOKEN_ORIGIN},
    ContractError,
};

const DEFAULT_IBC_HOST: &str = "blabla";

#[test]
fn test_dequeue_channel_from_path_ok_1() {
    let a = ChannelId!(42);
    let path = update_channel_path(U256::ZERO, a).unwrap();
    assert_eq!(dequeue_channel_from_path(path).1.unwrap(), a);
}

#[test]
fn test_dequeue_channel_from_path_ok_2() {
    let a = ChannelId!(42);
    let b = ChannelId!(123);
    let path1 = update_channel_path(U256::ZERO, a).unwrap();
    let path2 = update_channel_path(path1, b).unwrap();
    let (remaining_path, last_channel) = dequeue_channel_from_path(path2);
    assert_eq!(last_channel.unwrap(), a);
    assert_eq!(dequeue_channel_from_path(remaining_path).1.unwrap(), b);
}

#[test]
fn test_dequeue_channel_from_path_ok_3() {
    let a = ChannelId!(0xDEAD);
    let b = ChannelId!(0xC0DE);
    let c = ChannelId!(0xBEEF);
    let path1 = update_channel_path(U256::ZERO, a).unwrap();
    let path2 = update_channel_path(path1, b).unwrap();
    let path3 = update_channel_path(path2, c).unwrap();

    let (remaining_path1, last_channel1) = dequeue_channel_from_path(path3);
    assert_eq!(last_channel1.unwrap(), a);

    let (remaining_path2, last_channel2) = dequeue_channel_from_path(remaining_path1);
    assert_eq!(last_channel2.unwrap(), b);

    let (remaining_path3, last_channel3) = dequeue_channel_from_path(remaining_path2);
    assert_eq!(last_channel3.unwrap(), c);
    assert_eq!(remaining_path3, U256::ZERO);
}

#[test]
fn test_channel_path_ok() {
    let a = ChannelId!(1);
    let b = ChannelId!(2);
    let c = ChannelId!(3);
    let d = ChannelId!(4);
    let e = ChannelId!(5);
    let f = ChannelId!(6);
    let g = ChannelId!(7);
    let h = ChannelId!(8);

    let path1 = update_channel_path(U256::ZERO, a).unwrap();
    let path2 = update_channel_path(path1, b).unwrap();
    let path3 = update_channel_path(path2, c).unwrap();
    let path4 = update_channel_path(path3, d).unwrap();
    let path5 = update_channel_path(path4, e).unwrap();
    let path6 = update_channel_path(path5, f).unwrap();
    let path7 = update_channel_path(path6, g).unwrap();
    let path8 = update_channel_path(path7, h).unwrap();

    let expected = U256::from(a.raw())
        | U256::from(b.raw()) << 32
        | U256::from(c.raw()) << 64
        | U256::from(d.raw()) << 96
        | U256::from(e.raw()) << 128
        | U256::from(f.raw()) << 160
        | U256::from(g.raw()) << 192
        | U256::from(h.raw()) << 224;

    assert_eq!(path8, expected);
}

#[test]
fn test_reverse_channel_path_complete_ok() {
    let a = ChannelId!(1);
    let b = ChannelId!(2);
    let c = ChannelId!(3);
    let d = ChannelId!(4);
    let e = ChannelId!(5);
    let f = ChannelId!(6);
    let g = ChannelId!(7);
    let h = ChannelId!(8);

    let path = update_channel_path(
        update_channel_path(
            update_channel_path(
                update_channel_path(
                    update_channel_path(
                        update_channel_path(
                            update_channel_path(update_channel_path(U256::ZERO, a).unwrap(), b)
                                .unwrap(),
                            c,
                        )
                        .unwrap(),
                        d,
                    )
                    .unwrap(),
                    e,
                )
                .unwrap(),
                f,
            )
            .unwrap(),
            g,
        )
        .unwrap(),
        h,
    )
    .unwrap();

    let reversed = reverse_channel_path(path).unwrap();

    let expected = U256::from(h.raw())
        | U256::from(g.raw()) << 32
        | U256::from(f.raw()) << 64
        | U256::from(e.raw()) << 96
        | U256::from(d.raw()) << 128
        | U256::from(c.raw()) << 160
        | U256::from(b.raw()) << 192
        | U256::from(a.raw()) << 224;

    assert_eq!(reversed, expected);
}

#[test]
fn test_reverse_channel_path_partial_ok() {
    let a = ChannelId!(1);
    let b = ChannelId!(2);
    let path = update_channel_path(update_channel_path(U256::ZERO, a).unwrap(), b).unwrap();
    let reversed = reverse_channel_path(path).unwrap();
    let expected = U256::from(b.raw()) | U256::from(a.raw()) << 32;
    assert_eq!(reversed, expected);
}

#[test]
fn test_channel_path_full() {
    let mut path = U256::ZERO;
    for i in 1..=8 {
        path = update_channel_path(path, i.try_into().unwrap()).unwrap();
    }

    // Trying to add one more should fail with ChannelPathIsFull
    let result = update_channel_path(path, ChannelId!(9));
    assert!(result.is_err());
    match result {
        Err(ContractError::ChannelPathIsFull { .. }) => {}
        _ => panic!("Expected ChannelPathIsFull error"),
    }
}

#[test]
fn test_pop_channel_from_path_ok_1() {
    let a = ChannelId!(42);
    let path = update_channel_path(U256::ZERO, a).unwrap();
    let (base_path, channel_id) = pop_channel_from_path(path);
    assert_eq!(channel_id.unwrap(), a);
    assert_eq!(base_path, U256::ZERO);
}

#[test]
fn test_pop_channel_from_path_ok_2() {
    let a = ChannelId!(42);
    let b = ChannelId!(123);
    let path1 = update_channel_path(U256::ZERO, a).unwrap();
    let path2 = update_channel_path(path1, b).unwrap();

    let (base_path, channel_id) = pop_channel_from_path(path2);
    assert_eq!(channel_id.unwrap(), b);
    assert_eq!(base_path, path1);
}

#[test]
fn test_pop_channel_from_path_ok_3() {
    let a = ChannelId!(0xDEAD);
    let b = ChannelId!(0xC0DE);
    let c = ChannelId!(0xBEEF);
    let path1 = update_channel_path(U256::ZERO, a).unwrap();
    let path2 = update_channel_path(path1, b).unwrap();
    let path3 = update_channel_path(path2, c).unwrap();

    let (base_path1, channel_id1) = pop_channel_from_path(path3);
    assert_eq!(channel_id1.unwrap(), c);
    assert_eq!(base_path1, path2);

    let (base_path2, channel_id2) = pop_channel_from_path(base_path1);
    assert_eq!(channel_id2.unwrap(), b);
    assert_eq!(base_path2, path1);

    let (base_path3, channel_id3) = pop_channel_from_path(base_path2);
    assert_eq!(channel_id3.unwrap(), a);
    assert_eq!(base_path3, U256::ZERO);
}

#[test]
fn test_pop_channel_from_path_complex() {
    let a = ChannelId!(1);
    let b = ChannelId!(2);
    let c = ChannelId!(3);
    let d = ChannelId!(4);
    let e = ChannelId!(5);
    let f = ChannelId!(6);
    let g = ChannelId!(7);
    let h = ChannelId!(8);

    let path1 = update_channel_path(U256::ZERO, a).unwrap();
    let path2 = update_channel_path(path1, b).unwrap();
    let path3 = update_channel_path(path2, c).unwrap();
    let path4 = update_channel_path(path3, d).unwrap();
    let path5 = update_channel_path(path4, e).unwrap();
    let path6 = update_channel_path(path5, f).unwrap();
    let path7 = update_channel_path(path6, g).unwrap();
    let path8 = update_channel_path(path7, h).unwrap();

    let expected_base_path = update_channel_path(
        update_channel_path(
            update_channel_path(
                update_channel_path(
                    update_channel_path(
                        update_channel_path(update_channel_path(U256::ZERO, a).unwrap(), b)
                            .unwrap(),
                        c,
                    )
                    .unwrap(),
                    d,
                )
                .unwrap(),
                e,
            )
            .unwrap(),
            f,
        )
        .unwrap(),
        g,
    )
    .unwrap();

    let (base_path, channel_id) = pop_channel_from_path(path8);
    assert_eq!(channel_id.unwrap(), h);
    assert_eq!(base_path, expected_base_path);
}

#[test]
fn test_pop_channel_from_path_zero() {
    let (base_path, channel_id) = pop_channel_from_path(U256::ZERO);
    assert_eq!(channel_id, None);
    assert_eq!(base_path, U256::ZERO);
}

#[test]
fn test_tint_forward_salt_ok() {
    let salt = H256::from([
        0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD,
        0xEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB,
        0xCD, 0xEF,
    ]);
    assert!(!is_forwarded_packet(salt));
    assert!(is_forwarded_packet(tint_forward_salt(salt)));
}

#[test]
fn test_tint_forward_salt_idempotent() {
    let salt = H256::from([
        0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21, 0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43,
        0x21, 0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21, 0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65,
        0x43, 0x21,
    ]);
    let tinted = tint_forward_salt(salt);
    assert_eq!(tint_forward_salt(tinted), tinted);
}

#[test]
fn test_tint_forward_salt_preserves_data() {
    let salt = H256::from([
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA,
        0xBE, 0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE,
        0xBA, 0xBE,
    ]);
    let tinted = tint_forward_salt(salt);
    // Check that non-magic bits are preserved
    let mask = !FORWARD_SALT_MAGIC;
    assert_eq!(
        U256::from_be_bytes(*salt.get()) & mask,
        U256::from_be_bytes(*tinted.get()) & mask
    );
}

#[test]
fn test_verify_forward_ok() {
    let sender = Addr::unchecked(DEFAULT_IBC_HOST);

    let forward = Forward {
        path: update_channel_path(
            update_channel_path(U256::ZERO, ChannelId!(10)).unwrap(),
            ChannelId!(1),
        )
        .unwrap(),
        timeout_height: u64::MAX,
        timeout_timestamp: 0,
        instruction: Instruction {
            version: INSTR_VERSION_0,
            opcode: OP_MULTIPLEX,
            operand: Multiplex {
                sender: sender.as_bytes().to_vec().into(),
                eureka: false,
                contract_address: sender.as_bytes().to_vec().into(),
                contract_calldata: vec![].into(),
            }
            .abi_encode_params()
            .into(),
        },
    };

    let (mut deps, _, info, _) = init();
    let mut coins = Default::default();
    let mut response = Response::new();

    assert_eq!(
        verify_forward(
            deps.as_mut(),
            info,
            &mut coins,
            ChannelId!(1),
            &forward,
            &mut response
        ),
        Ok(())
    );

    assert_eq!(response, Response::new());
}

#[test]
fn test_verify_forward_invalid_version() {
    let sender = Addr::unchecked(DEFAULT_IBC_HOST);

    let forward = Forward {
        path: update_channel_path(
            update_channel_path(U256::ZERO, ChannelId!(10)).unwrap(),
            ChannelId!(1),
        )
        .unwrap(),
        timeout_height: u64::MAX,
        timeout_timestamp: 0,
        instruction: Instruction {
            version: INSTR_VERSION_1,
            opcode: OP_MULTIPLEX,
            operand: Multiplex {
                sender: sender.as_bytes().to_vec().into(),
                eureka: false,
                contract_address: sender.as_bytes().to_vec().into(),
                contract_calldata: vec![].into(),
            }
            .abi_encode_params()
            .into(),
        },
    };

    let (mut deps, _, info, _) = init();
    let mut coins = Default::default();

    assert_eq!(
        verify_forward(
            deps.as_mut(),
            info,
            &mut coins,
            ChannelId!(1),
            &forward,
            &mut Response::new()
        ),
        Err(ContractError::UnsupportedVersion {
            version: INSTR_VERSION_1
        })
    );
}

#[test]
fn test_verify_forward_invalid_instruction() {
    let forward = Forward {
        path: update_channel_path(
            update_channel_path(U256::ZERO, ChannelId!(10)).unwrap(),
            ChannelId!(1),
        )
        .unwrap(),
        timeout_height: u64::MAX,
        timeout_timestamp: 0,
        instruction: Instruction {
            version: INSTR_VERSION_0,
            opcode: OP_FORWARD,
            operand: Default::default(),
        },
    };

    let (mut deps, _, info, _) = init();
    let mut coins = Default::default();

    assert_eq!(
        verify_forward(
            deps.as_mut(),
            info,
            &mut coins,
            ChannelId!(1),
            &forward,
            &mut Response::new()
        ),
        Err(ContractError::InvalidForwardInstruction)
    );
}

#[test]
fn test_verify_multiplex_sender_ok() {
    let sender = Addr::unchecked("sender");
    // Test with matching sender
    let multiplex = Multiplex {
        sender: sender.as_bytes().to_vec().into(),
        eureka: false,
        contract_address: Addr::unchecked("contract").as_bytes().to_vec().into(),
        contract_calldata: vec![].into(),
    };
    let mut response = Response::new();
    assert_eq!(
        verify_multiplex(&multiplex, sender.clone(), &mut response),
        Ok(())
    );
    assert_eq!(response, Response::new());
}

#[test]
fn test_verify_multiplex_invalid_sender() {
    let sender = Addr::unchecked("sender");
    // Test with matching sender
    let multiplex = Multiplex {
        sender: sender.as_bytes().to_vec().into(),
        eureka: false,
        contract_address: Addr::unchecked("contract").as_bytes().to_vec().into(),
        contract_calldata: vec![].into(),
    };
    let wrong_sender = Addr::unchecked("wrong_sender");
    let result = verify_multiplex(&multiplex, wrong_sender, &mut Response::new());
    assert!(matches!(result, Err(ContractError::InvalidMultiplexSender)));
}

#[test]
fn test_verify_batch_ok() {
    let sender = Addr::unchecked(DEFAULT_IBC_HOST);
    // Test with matching sender
    let multiplex = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_MULTIPLEX,
        operand: Multiplex {
            sender: sender.as_bytes().to_vec().into(),
            eureka: false,
            contract_address: Addr::unchecked("contract").as_bytes().to_vec().into(),
            contract_calldata: vec![].into(),
        }
        .abi_encode_params()
        .into(),
    };

    let (mut deps, _, info, _) = init();
    let mut response = Response::new();
    let mut funds = Coins::try_from(info.funds.clone()).unwrap();
    let result = verify_batch(
        deps.as_mut(),
        info,
        &mut funds,
        ChannelId!(1),
        U256::ZERO,
        &Batch {
            instructions: vec![multiplex],
        },
        &mut response,
    );

    assert_eq!(result, Ok(()));
}

#[test]
fn test_verify_batch_invalid_instruction() {
    let (mut deps, _, info, _) = init();
    let mut response = Response::new();
    let mut funds = Coins::try_from(info.funds.clone()).unwrap();
    let result = verify_batch(
        deps.as_mut(),
        info,
        &mut funds,
        ChannelId!(1),
        U256::ZERO,
        &Batch {
            instructions: vec![Instruction {
                version: INSTR_VERSION_0,
                opcode: OP_BATCH,
                operand: vec![].into(),
            }],
        },
        &mut response,
    );

    assert_eq!(result, Err(ContractError::InvalidBatchInstruction));
}

#[test]
fn test_verify_internal_unsupported_version() {
    let mut deps = mock_dependencies();
    let info = message_info(&Addr::unchecked("sender"), &[]);

    let instruction = Instruction {
        version: 99, // Unsupported version
        opcode: OP_MULTIPLEX,
        operand: vec![].into(),
    };

    let mut response = Response::new();
    let mut funds = Coins::try_from(info.funds.clone()).unwrap();
    let result = verify_internal(
        deps.as_mut(),
        info,
        &mut funds,
        ChannelId!(1),
        U256::ZERO,
        &instruction,
        &mut response,
    );
    assert!(matches!(
        result,
        Err(ContractError::UnsupportedVersion { version: 99 })
    ));
}

#[test]
fn test_verify_internal_unknown_opcode() {
    let mut deps = mock_dependencies();
    let info = message_info(&Addr::unchecked("sender"), &[]);

    let instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: 99, // Unknown opcode
        operand: vec![].into(),
    };

    let mut response = Response::new();
    let mut funds = Coins::try_from(info.funds.clone()).unwrap();
    let result = verify_internal(
        deps.as_mut(),
        info,
        &mut funds,
        ChannelId!(1),
        U256::ZERO,
        &instruction,
        &mut response,
    );
    assert!(matches!(
        result,
        Err(ContractError::UnknownOpcode { opcode: 99 })
    ));
}

#[test]
fn test_execute_internal_batch_only_self() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("sender"), &[]);
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::InternalBatch { messages: vec![] },
    );
    assert_eq!(result, Err(ContractError::OnlySelf));
}

#[test]
fn test_execute_internal_execute_only_self() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("sender"), &[]);
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::InternalExecutePacket {
            caller: Addr::unchecked(""),
            packet: Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(10),
                data: Default::default(),
                timeout_height: Default::default(),
                timeout_timestamp: Default::default(),
            },
            relayer: Addr::unchecked(""),
            relayer_msg: Default::default(),
            intent: false,
        },
    );
    assert_eq!(result, Err(ContractError::OnlySelf));
}

#[test]
fn test_execute_internal_write_ack_only_self() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("sender"), &[]);
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::InternalWriteAck {
            ack: Default::default(),
        },
    );
    assert_eq!(result, Err(ContractError::OnlySelf));
}

fn init() -> (
    OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    Env,
    MessageInfo,
    Config,
) {
    let mut deps = mock_dependencies();
    deps.api = MockApi::default().with_prefix("union");
    let env = mock_env();
    let ibc_host = Addr::unchecked(DEFAULT_IBC_HOST);
    let info = message_info(&ibc_host, &[]);
    let config = Config {
        admin: Addr::unchecked(""),
        ibc_host,
        token_minter_code_id: 0,
        rate_limit_admin: Addr::unchecked("blabla"),
        rate_limit_operators: vec![],
        rate_limit_disabled: false,
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    (deps, env, info, config)
}

#[test]
fn test_on_recv_packet_only_ibc() {
    let (mut deps, env, mut info, _) = init();
    info.sender = Addr::unchecked("not_ibc");
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
            caller: "".into(),
            packet: Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(10),
                data: Default::default(),
                timeout_height: Default::default(),
                timeout_timestamp: Default::default(),
            },
            relayer: "".into(),
            relayer_msg: Default::default(),
        }),
    );
    assert_eq!(result, Err(ContractError::OnlyIBCHost));
}

#[test]
fn test_on_recv_packet_invalid_caller() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
            caller: "".into(),
            packet: Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(10),
                data: Default::default(),
                timeout_height: Default::default(),
                timeout_timestamp: Default::default(),
            },
            relayer: "".into(),
            relayer_msg: Default::default(),
        }),
    );
    assert_eq!(
        result,
        Err(ContractError::Std(StdError::generic_err(
            "Error decoding bech32"
        )))
    );
}

#[test]
fn test_on_recv_packet_invalid_relayer() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
            caller: "union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua".into(),
            packet: Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(10),
                data: Default::default(),
                timeout_height: Default::default(),
                timeout_timestamp: Default::default(),
            },
            relayer: "".into(),
            relayer_msg: Default::default(),
        }),
    );
    assert_eq!(
        result,
        Err(ContractError::Std(StdError::generic_err(
            "Error decoding bech32"
        )))
    );
}

#[test]
fn test_on_recv_packet_save_packet() {
    let (mut deps, env, info, _) = init();
    let packet = Packet {
        source_channel_id: ChannelId!(1),
        destination_channel_id: ChannelId!(10),
        data: Default::default(),
        timeout_height: Default::default(),
        timeout_timestamp: Default::default(),
    };
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
            caller: "union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua".into(),
            packet: packet.clone(),
            relayer: "union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua".into(),
            relayer_msg: Default::default(),
        }),
    );
    assert_eq!(
        packet,
        EXECUTING_PACKET.load(deps.as_mut().storage).unwrap()
    );
    assert!(result.is_ok());
}

#[test]
fn test_on_recv_packet_nonreentrant() {
    let (mut deps, env, info, _) = init();
    let msg = ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
        caller: "union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua".into(),
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(10),
            data: Default::default(),
            timeout_height: Default::default(),
            timeout_timestamp: Default::default(),
        },
        relayer: "union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua".into(),
        relayer_msg: Default::default(),
    });
    let result = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(result.is_ok());
    let result = execute(deps.as_mut(), env, info, msg.clone());
    assert_eq!(result, Err(ContractError::AlreadyExecuting));
}

fn zkgm_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(execute, instantiate, query)
            .with_migrate(migrate)
            .with_reply(reply),
    )
}

fn cw20_minter_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            cw20_token_minter::contract::execute,
            cw20_token_minter::contract::instantiate,
            cw20_token_minter::contract::query,
        )
        .with_migrate(cw20_token_minter::contract::migrate),
    )
}

fn cw20_base_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        )
        .with_migrate(cw20_base::contract::migrate),
    )
}

// Dummy migration contract simulating our deterministic proxy bytecode
fn migrator_instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> {
    Ok(Default::default())
}
fn migrator_execute(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> {
    Ok(Default::default())
}
fn migrator_query(_: Deps, _: Env, _: Empty) -> StdResult<Binary> {
    Ok(Default::default())
}

fn migrator_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(
        migrator_execute,
        migrator_instantiate,
        migrator_query,
    ))
}

// Mocked ibc host
const PACKET_ACK: Map<[u8; 32], Bytes> = Map::new("packet_ack");

fn ibc_host_instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> {
    Ok(Default::default())
}
fn ibc_host_execute(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: ibc_union_msg::msg::ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ibc_union_msg::msg::ExecuteMsg::WriteAcknowledgement(msg_write_acknowledgement) => {
            PACKET_ACK.save(
                deps.storage,
                commit_packets(&[msg_write_acknowledgement.packet]).into(),
                &msg_write_acknowledgement.acknowledgement,
            )?;
        }
        _ => {
            panic!()
        }
    }
    Ok(Default::default())
}
fn ibc_host_query(_: Deps, _: Env, _: Empty) -> StdResult<Binary> {
    Ok(Default::default())
}
fn ibc_host_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(
        ibc_host_execute,
        ibc_host_instantiate,
        ibc_host_query,
    ))
}

fn mock_app() -> App {
    AppBuilder::new()
        .with_api(MockApi::default().with_prefix("union"))
        .build(|_, _, _| {})
}

#[allow(dead_code)]
struct TestState {
    app: App,
    ibc_host_code_id: u64,
    proxy_code_id: u64,
    cw20_base_code_id: u64,
    cw20_minter_code_id: u64,
    zkgm_code_id: u64,
    ibc_host: Addr,
    minter: Addr,
    zkgm: Addr,
    rate_limiter: Addr,
}

impl TestState {
    fn balance_of(&self, token: &Bytes, address: Addr) -> u128 {
        self.app
            .wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                std::str::from_utf8(token).unwrap(),
                &cw20_base::msg::QueryMsg::Balance {
                    address: address.to_string(),
                },
            )
            .unwrap()
            .balance
            .u128()
    }
}

fn init_test_state(admin: Addr) -> TestState {
    let mut app = mock_app();
    let ibc_host_code_id = app.store_code(ibc_host_contract());
    let proxy_code_id = app.store_code(migrator_contract());
    let cw20_base_code_id = app.store_code(cw20_base_contract());
    let cw20_minter_code_id = app.store_code(cw20_minter_contract());
    let zkgm_code_id = app.store_code(zkgm_contract());
    let ibc_host = app
        .instantiate_contract(
            ibc_host_code_id,
            admin.clone(),
            &Empty {},
            &[],
            "ibc-host",
            Some(admin.clone().to_string()),
        )
        .unwrap();
    let zkgm = app
        .instantiate2_contract(
            proxy_code_id,
            admin.clone(),
            &Empty {},
            &[],
            "zkgm",
            admin.clone().to_string(),
            b"zkgm",
        )
        .unwrap();
    let rate_limiter = Addr::unchecked("union1ml67yhc5kp8qrxssfnqz8pxqvjyln5fus654vk");
    app.migrate_contract(
        admin.clone(),
        zkgm.clone(),
        &frissitheto::UpgradeMsg::Init::<_, ()>(InitMsg {
            config: Config {
                admin,
                ibc_host: ibc_host.clone(),
                token_minter_code_id: cw20_minter_code_id,
                rate_limit_admin: Addr::unchecked("hola"),
                rate_limit_operators: vec![rate_limiter.clone()],
                rate_limit_disabled: false,
            },
            minter_init_params: TokenMinterInitParams::Cw20 {
                cw20_base_code_id,
                dummy_code_id: proxy_code_id,
            },
        }),
        zkgm_code_id,
    )
    .unwrap();
    let minter = app
        .wrap()
        .query_wasm_smart(zkgm.clone(), &QueryMsg::GetMinter {})
        .unwrap();
    TestState {
        app,
        ibc_host_code_id,
        proxy_code_id,
        cw20_base_code_id,
        cw20_minter_code_id,
        zkgm_code_id,
        ibc_host,
        minter,
        zkgm,
        rate_limiter,
    }
}

#[test]
fn test_deploy_via_proxy() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    init_test_state(admin);
}

#[test]
fn test_recv_packet_invalid_failure_ack() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let packet = Packet {
        source_channel_id: ChannelId!(1),
        destination_channel_id: ChannelId!(10),
        data: Default::default(),
        timeout_height: Default::default(),
        timeout_timestamp: Default::default(),
    };
    let caller = "union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua".to_string();
    let msg = ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
        caller: caller.clone(),
        packet: packet.clone(),
        relayer: caller,
        relayer_msg: Default::default(),
    });
    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm, &msg, vec![]).unwrap().into(),
        )
        .unwrap();
    let ack = PACKET_ACK
        .load(
            st.app.contract_storage(&st.ibc_host).as_ref(),
            commit_packets(&[packet]).into(),
        )
        .unwrap();
    assert_eq!(
        ack,
        Ack {
            tag: TAG_ACK_FAILURE,
            inner_ack: Default::default(),
        }
        .abi_encode_params()
    )
}

#[allow(dead_code)]
struct IncomingOrderBuilder {
    caller: Addr,
    relayer: Addr,
    source_channel_id: ChannelId,
    destination_channel_id: ChannelId,
    salt: H256,
    path: U256,
    base_token: Bytes,
    base_token_symbol: String,
    base_token_name: String,
    base_token_decimals: u8,
    base_token_path: U256,
    base_amount: u128,
    quote_token: Bytes,
    quote_amount: u128,
    sender: Bytes,
    receiver: Addr,
}

#[allow(dead_code)]
impl IncomingOrderBuilder {
    fn new(quote_token: Bytes) -> Self {
        // host
        let caller = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
        let relayer = Addr::unchecked("union1ml67yhc5kp8qrxssfnqz8pxqvjyln5fus654vk");
        // packet
        let source_channel_id = ChannelId!(1);
        let destination_channel_id = ChannelId!(10);
        let salt = Default::default();
        // order
        let path = U256::ZERO;
        let base_token = Bytes::from(hex_literal::hex!("00AABBCCDDEEFF"));
        let base_token_symbol = "USDC".into();
        let base_token_name = "Circle USD".into();
        let base_token_decimals = 8;
        let base_token_path = U256::ZERO;
        let base_amount = 0xCAFEBABEu128;
        let quote_amount = 0xCAFEBABEu128;
        let sender = vec![].into();
        let receiver = Addr::unchecked("union1g0jxmy25g5t6qdagq2dkclux7c46kwym8decfw");
        Self {
            caller,
            relayer,
            source_channel_id,
            destination_channel_id,
            salt,
            path,
            base_token,
            base_token_symbol,
            base_token_name,
            base_token_decimals,
            base_token_path,
            base_amount,
            quote_token,
            quote_amount,
            sender,
            receiver,
        }
    }

    fn with_source_channel_id(mut self, source_channel_id: impl Into<ChannelId>) -> Self {
        self.source_channel_id = source_channel_id.into();
        self
    }

    fn with_destination_channel_id(mut self, destination_channel_id: impl Into<ChannelId>) -> Self {
        self.destination_channel_id = destination_channel_id.into();
        self
    }

    fn with_caller(mut self, caller: impl Into<Addr>) -> Self {
        self.caller = caller.into();
        self
    }

    fn with_path(mut self, path: impl Into<U256>) -> Self {
        self.path = path.into();
        self
    }

    fn with_base_token_path(mut self, base_token_path: impl Into<U256>) -> Self {
        self.base_token_path = base_token_path.into();
        self
    }

    fn with_base_token(mut self, base_token: impl Into<Bytes>) -> Self {
        self.base_token = base_token.into();
        self
    }

    fn with_base_amount(mut self, base_amount: impl Into<u128>) -> Self {
        self.base_amount = base_amount.into();
        self
    }

    fn with_quote_token(mut self, quote_token: impl Into<Bytes>) -> Self {
        self.quote_token = quote_token.into();
        self
    }

    fn with_quote_amount(mut self, quote_amount: impl Into<u128>) -> Self {
        self.quote_amount = quote_amount.into();
        self
    }

    fn build(self) -> (Self, ExecuteMsg, Packet) {
        let packet = Packet {
            source_channel_id: self.source_channel_id,
            destination_channel_id: self.destination_channel_id,
            data: ZkgmPacket {
                salt: self.salt.into(),
                path: self.path,
                instruction: Instruction {
                    version: INSTR_VERSION_1,
                    opcode: OP_FUNGIBLE_ASSET_ORDER,
                    operand: FungibleAssetOrder {
                        sender: self.sender.clone().into_vec().into(),
                        receiver: self
                            .receiver
                            .clone()
                            .into_string()
                            .as_bytes()
                            .to_vec()
                            .into(),
                        base_token: self.base_token.clone().into_vec().into(),
                        base_amount: self.base_amount.try_into().unwrap(),
                        base_token_symbol: self.base_token_symbol.clone(),
                        base_token_name: self.base_token_name.clone(),
                        base_token_decimals: self.base_token_decimals,
                        base_token_path: self.base_token_path,
                        quote_token: self.quote_token.clone().into_vec().into(),
                        quote_amount: self.quote_amount.try_into().unwrap(),
                    }
                    .abi_encode_params()
                    .into(),
                },
            }
            .abi_encode_params()
            .into(),
            timeout_height: Default::default(),
            timeout_timestamp: Default::default(),
        };
        let msg = ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
            caller: self.caller.clone().into(),
            packet: packet.clone(),
            relayer: self.relayer.clone().into(),
            relayer_msg: Default::default(),
        });
        (self, msg, packet)
    }
}

#[test]
fn test_recv_packet_native_new_wrapped() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin.clone());
    let path = U256::ZERO;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = st
        .app
        .wrap()
        .query_wasm_smart::<PredictWrappedTokenResponse>(
            st.zkgm.clone(),
            &QueryMsg::PredictWrappedToken {
                path: path.to_string(),
                channel_id: destination_channel_id,
                token: base_token.clone(),
            },
        )
        .unwrap()
        .wrapped_token;
    let quote_token_addr = Addr::unchecked(std::str::from_utf8(&quote_token).unwrap());
    assert!(st.app.contract_data(&quote_token_addr).is_err());
    let (order, msg, packet) = IncomingOrderBuilder::new(quote_token)
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .build();
    st.app
        .execute(
            st.rate_limiter.clone(),
            wasm_execute(
                st.zkgm.clone(),
                &ExecuteMsg::SetBucketConfig {
                    denom: std::str::from_utf8(&order.quote_token).unwrap().into(),
                    capacity: order.quote_amount.into(),
                    refill_rate: 1u32.into(),
                    reset: false,
                },
                vec![],
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();
    assert_eq!(
        PACKET_ACK
            .load(
                st.app.contract_storage(&st.ibc_host).as_ref(),
                commit_packets(&[packet]).into(),
            )
            .unwrap(),
        Ack {
            tag: TAG_ACK_SUCCESS,
            inner_ack: FungibleAssetOrderAck {
                fill_type: FILL_TYPE_PROTOCOL,
                market_maker: Default::default()
            }
            .abi_encode_params()
            .into(),
        }
        .abi_encode_params()
    );
    let deployed_contract = st.app.contract_data(&quote_token_addr).unwrap();
    assert_eq!(deployed_contract.code_id, st.cw20_base_code_id);
    assert_eq!(deployed_contract.admin, Some(admin));

    let token_info_response: TokenInfoResponse = st
        .app
        .wrap()
        .query_wasm_smart(quote_token_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(token_info_response.name, order.base_token_name);
    assert_eq!(token_info_response.symbol, order.base_token_symbol);
    assert_eq!(token_info_response.decimals, order.base_token_decimals);
    assert_eq!(token_info_response.total_supply.u128(), order.base_amount);
}

#[test]
fn test_recv_packet_native_new_wrapped_relative_supply() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let path = U256::ZERO;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = st
        .app
        .wrap()
        .query_wasm_smart::<PredictWrappedTokenResponse>(
            st.zkgm.clone(),
            &QueryMsg::PredictWrappedToken {
                path: path.to_string(),
                channel_id: destination_channel_id,
                token: base_token.clone(),
            },
        )
        .unwrap()
        .wrapped_token;
    let quote_token_addr = Addr::unchecked(std::str::from_utf8(&quote_token).unwrap());
    assert!(st.app.contract_data(&quote_token_addr).is_err());
    let (order, msg, _) = IncomingOrderBuilder::new(quote_token.clone())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .build();
    st.app
        .execute(
            st.rate_limiter.clone(),
            wasm_execute(
                st.zkgm.clone(),
                &ExecuteMsg::SetBucketConfig {
                    denom: std::str::from_utf8(&order.quote_token).unwrap().into(),
                    capacity: order.quote_amount.into(),
                    refill_rate: 1u32.into(),
                    reset: false,
                },
                vec![],
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();
    assert_eq!(
        st.balance_of(&quote_token, order.receiver),
        order.quote_amount
    );
}

#[test]
fn test_recv_packet_native_new_wrapped_split_fee() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let path = U256::ZERO;
    let destination_channel_id = ChannelId!(10);
    let base_amount = 1000u128;
    let quote_amount = 900u128;
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = st
        .app
        .wrap()
        .query_wasm_smart::<PredictWrappedTokenResponse>(
            st.zkgm.clone(),
            &QueryMsg::PredictWrappedToken {
                path: path.to_string(),
                channel_id: destination_channel_id,
                token: base_token.clone(),
            },
        )
        .unwrap()
        .wrapped_token;
    let (order, msg, _) = IncomingOrderBuilder::new(quote_token.clone())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .with_base_amount(base_amount)
        .with_quote_amount(quote_amount)
        .build();
    st.app
        .execute(
            st.rate_limiter.clone(),
            wasm_execute(
                st.zkgm.clone(),
                &ExecuteMsg::SetBucketConfig {
                    denom: std::str::from_utf8(&order.quote_token).unwrap().into(),
                    capacity: order.quote_amount.into(),
                    refill_rate: 1u32.into(),
                    reset: false,
                },
                vec![],
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();
    assert_eq!(
        st.balance_of(&quote_token, order.receiver),
        order.quote_amount
    );
    assert_eq!(
        st.balance_of(&quote_token, order.relayer),
        order.base_amount - order.quote_amount
    );
}

#[test]
fn test_recv_packet_native_new_wrapped_origin_set() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let path = update_channel_path(
        update_channel_path(U256::ZERO, ChannelId!(9)).unwrap(),
        ChannelId!(4),
    )
    .unwrap();
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = st
        .app
        .wrap()
        .query_wasm_smart::<PredictWrappedTokenResponse>(
            st.zkgm.clone(),
            &QueryMsg::PredictWrappedToken {
                path: path.to_string(),
                channel_id: destination_channel_id,
                token: base_token.clone(),
            },
        )
        .unwrap()
        .wrapped_token;
    let (order, msg, _) = IncomingOrderBuilder::new(quote_token.clone())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .build();
    st.app
        .execute(
            st.rate_limiter.clone(),
            wasm_execute(
                st.zkgm.clone(),
                &ExecuteMsg::SetBucketConfig {
                    denom: std::str::from_utf8(&order.quote_token).unwrap().into(),
                    capacity: order.quote_amount.into(),
                    refill_rate: 1u32.into(),
                    reset: false,
                },
                vec![],
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();
    let quote_token_addr = Addr::unchecked(std::str::from_utf8(&quote_token).unwrap());
    let token_origin = TOKEN_ORIGIN
        .load(
            st.app.contract_storage(&st.zkgm).as_ref(),
            quote_token_addr.into(),
        )
        .unwrap();
    assert_eq!(
        U256::from_be_bytes(token_origin.to_be_bytes()),
        update_channel_path(path, destination_channel_id).unwrap()
    );
}

#[test]
fn test_recv_packet_native_base_dont_cover_quote_only_maker() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let path = U256::ZERO;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = st
        .app
        .wrap()
        .query_wasm_smart::<PredictWrappedTokenResponse>(
            st.zkgm.clone(),
            &QueryMsg::PredictWrappedToken {
                path: path.to_string(),
                channel_id: destination_channel_id,
                token: base_token.clone(),
            },
        )
        .unwrap()
        .wrapped_token;
    let quote_token_addr = Addr::unchecked(std::str::from_utf8(&quote_token).unwrap());
    assert!(st.app.contract_data(&quote_token_addr).is_err());
    let (_, msg, _) = IncomingOrderBuilder::new(quote_token)
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        // Base not covering the quote
        .with_base_amount(0x1337u32)
        .with_quote_amount(0x1338u32)
        .build();
    let err = st
        .app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .err()
        .unwrap();
    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::OnlyMaker
    );
}

#[test]
fn test_recv_packet_native_to_native_only_maker() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let path = U256::ZERO;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = Bytes::from(b"muno");
    let quote_token_addr = Addr::unchecked(std::str::from_utf8(&quote_token).unwrap());
    assert!(st.app.contract_data(&quote_token_addr).is_err());
    let (_, msg, _) = IncomingOrderBuilder::new(quote_token)
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .build();
    let err = st
        .app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .err()
        .unwrap();
    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::OnlyMaker
    );
}

#[test]
fn test_recv_packet_native_quote_maker_fill_ok() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin);
    let path = U256::ZERO;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));
    let quote_token = Bytes::from(b"muno");
    let quote_token_addr = Addr::unchecked(std::str::from_utf8(&quote_token).unwrap());
    assert!(st.app.contract_data(&quote_token_addr).is_err());
    let (order, msg, _) = IncomingOrderBuilder::new(quote_token)
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .build();
    let quote_coin = Coin::new(order.quote_amount, quote_token_addr.clone());
    assert_eq!(
        st.app
            .wrap()
            .query_balance(&order.receiver, &quote_token_addr)
            .unwrap(),
        Coin::new(0u32, quote_token_addr.clone())
    );
    st.app
        .sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: st.ibc_host.clone().to_string(),
                amount: vec![quote_coin.clone()],
            },
        ))
        .unwrap();
    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![quote_coin.clone()])
                .unwrap()
                .into(),
        )
        .unwrap();
    assert_eq!(
        st.app
            .wrap()
            .query_balance(order.receiver, quote_token_addr)
            .unwrap(),
        quote_coin
    );
}

#[test]
fn test_recv_packet_native_unwrap_wrapped_token_ok() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin.clone());
    let path = U256::ONE;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));

    let wrapped_token = st
        .app
        .instantiate_contract(
            st.cw20_base_code_id,
            admin.clone(),
            &cw20_base::msg::InstantiateMsg {
                name: "muno".to_string(),
                symbol: "muno".to_string(),
                decimals: 8,
                initial_balances: vec![Cw20Coin {
                    address: st.minter.to_string(),
                    amount: (0xCAFEBABEu128 + 1000).into(),
                }],
                mint: None,
                marketing: None,
            },
            &[],
            "muno-token",
            Some(admin.clone().to_string()),
        )
        .unwrap();

    increase_channel_balance(
        st.app.contract_storage_mut(&st.zkgm).as_mut(),
        destination_channel_id,
        reverse_channel_path(path).unwrap(),
        wrapped_token.to_string(),
        0xCAFEBABEu128.into(),
    )
    .unwrap();

    let msg = ExecuteMsg::SetBucketConfig {
        denom: wrapped_token.to_string(),
        capacity: 0xCAFEBABEu128.into(),
        refill_rate: 1u128.into(),
        reset: false,
    };

    st.app
        .execute(
            Addr::unchecked("union1ml67yhc5kp8qrxssfnqz8pxqvjyln5fus654vk"),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();

    let (order, msg, packet) = IncomingOrderBuilder::new(wrapped_token.as_bytes().into())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .with_base_token_path(reverse_channel_path(path).unwrap())
        .build();

    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();

    // make sure transfer succeeded with the correct ack
    assert_eq!(
        PACKET_ACK
            .load(
                st.app.contract_storage(&st.ibc_host).as_ref(),
                commit_packets(&[packet]).into(),
            )
            .unwrap(),
        Ack {
            tag: TAG_ACK_SUCCESS,
            inner_ack: FungibleAssetOrderAck {
                fill_type: FILL_TYPE_PROTOCOL,
                market_maker: Default::default()
            }
            .abi_encode_params()
            .into(),
        }
        .abi_encode_params()
    );

    // balance is reduced to 1000
    assert_eq!(
        st.balance_of(&wrapped_token.as_bytes().into(), st.minter.clone()),
        1000
    );

    // receiver's balance is now 0xCAFEBABE
    assert_eq!(
        st.balance_of(&wrapped_token.as_bytes().into(), order.receiver),
        0xCAFEBABE
    );

    let channel_balance = CHANNEL_BALANCE
        .load(
            st.app.contract_storage(&st.zkgm).as_ref(),
            (
                destination_channel_id.raw(),
                reverse_channel_path(path)
                    .unwrap()
                    .to_be_bytes::<32>()
                    .to_vec(),
                wrapped_token.to_string(),
            ),
        )
        .unwrap();

    // outstanding is now 0
    assert_eq!(channel_balance, Uint256::zero());
}

#[test]
fn test_recv_packet_native_unwrap_native_token_ok() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin.clone());
    let path = U256::ONE;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));

    let wrapped_token = "muno";

    st.app
        .sudo(SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
            to_address: st.minter.to_string(),
            amount: vec![Coin::new(0xCAFEBABEu128, wrapped_token)],
        }))
        .unwrap();

    increase_channel_balance(
        st.app.contract_storage_mut(&st.zkgm).as_mut(),
        destination_channel_id,
        reverse_channel_path(path).unwrap(),
        wrapped_token.to_string(),
        0xCAFEBABEu128.into(),
    )
    .unwrap();

    save_native_token(
        st.app.contract_storage_mut(&st.minter).as_mut(),
        wrapped_token,
    );

    let msg = ExecuteMsg::SetBucketConfig {
        denom: wrapped_token.to_string(),
        capacity: 0xCAFEBABEu128.into(),
        refill_rate: 1u128.into(),
        reset: false,
    };

    st.app
        .execute(
            Addr::unchecked("union1ml67yhc5kp8qrxssfnqz8pxqvjyln5fus654vk"),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();

    let (order, msg, packet) = IncomingOrderBuilder::new(wrapped_token.as_bytes().into())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .with_base_token_path(reverse_channel_path(path).unwrap())
        .build();

    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();

    let balance = st
        .app
        .wrap()
        .query_balance(order.receiver, wrapped_token)
        .unwrap();
    assert_eq!(balance.amount.u128(), 0xCAFEBABEu128);

    // make sure transfer succeeded with the correct ack
    assert_eq!(
        PACKET_ACK
            .load(
                st.app.contract_storage(&st.ibc_host).as_ref(),
                commit_packets(&[packet]).into(),
            )
            .unwrap(),
        Ack {
            tag: TAG_ACK_SUCCESS,
            inner_ack: FungibleAssetOrderAck {
                fill_type: FILL_TYPE_PROTOCOL,
                market_maker: Default::default()
            }
            .abi_encode_params()
            .into(),
        }
        .abi_encode_params()
    );

    let channel_balance = CHANNEL_BALANCE
        .load(
            st.app.contract_storage(&st.zkgm).as_ref(),
            (
                destination_channel_id.raw(),
                reverse_channel_path(path)
                    .unwrap()
                    .to_be_bytes::<32>()
                    .to_vec(),
                wrapped_token.to_string(),
            ),
        )
        .unwrap();

    // outstanding is now 0
    assert_eq!(channel_balance, Uint256::zero());
}

#[test]
fn test_recv_packet_native_unwrap_channel_no_outstanding() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin.clone());
    let path = U256::ONE;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));

    let wrapped_token = st
        .app
        .instantiate_contract(
            st.cw20_base_code_id,
            admin.clone(),
            &cw20_base::msg::InstantiateMsg {
                name: "muno".to_string(),
                symbol: "muno".to_string(),
                decimals: 8,
                initial_balances: vec![Cw20Coin {
                    address: st.minter.to_string(),
                    amount: (0xCAFEBABEu128 + 1000).into(),
                }],
                mint: None,
                marketing: None,
            },
            &[],
            "muno-token",
            Some(admin.clone().to_string()),
        )
        .unwrap();

    increase_channel_balance(
        st.app.contract_storage_mut(&st.zkgm).as_mut(),
        ChannelId!(20),
        reverse_channel_path(path).unwrap(),
        wrapped_token.to_string(),
        0xCAFEBABEu128.into(),
    )
    .unwrap();

    let (_, msg, packet) = IncomingOrderBuilder::new(wrapped_token.as_bytes().into())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .with_base_token_path(reverse_channel_path(path).unwrap())
        .build();

    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();

    // make sure transfer succeeded with the correct ack
    assert_eq!(
        PACKET_ACK
            .load(
                st.app.contract_storage(&st.ibc_host).as_ref(),
                commit_packets(&[packet]).into(),
            )
            .unwrap(),
        Ack {
            tag: TAG_ACK_FAILURE,
            inner_ack: Default::default()
        }
        .abi_encode_params()
    );
    let balance: cw20::BalanceResponse = st
        .app
        .wrap()
        .query_wasm_smart(
            &wrapped_token,
            &Cw20QueryMsg::Balance {
                address: st.minter.to_string(),
            },
        )
        .unwrap();
    // no transfer is made
    assert_eq!(balance.balance.u128(), 0xCAFEBABEu128 + 1000);
}

#[test]
fn test_recv_packet_native_unwrap_path_no_outstanding() {
    let admin = Addr::unchecked("union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua");
    let mut st = init_test_state(admin.clone());
    let path = U256::ONE;
    let destination_channel_id = ChannelId!(10);
    let base_token = Bytes::from(hex_literal::hex!("DEAFBABE"));

    let wrapped_token = st
        .app
        .instantiate_contract(
            st.cw20_base_code_id,
            admin.clone(),
            &cw20_base::msg::InstantiateMsg {
                name: "muno".to_string(),
                symbol: "muno".to_string(),
                decimals: 8,
                initial_balances: vec![Cw20Coin {
                    address: st.minter.to_string(),
                    amount: (0xCAFEBABEu128 + 1000).into(),
                }],
                mint: None,
                marketing: None,
            },
            &[],
            "muno-token",
            Some(admin.clone().to_string()),
        )
        .unwrap();

    increase_channel_balance(
        st.app.contract_storage_mut(&st.zkgm).as_mut(),
        destination_channel_id,
        "100".parse().unwrap(),
        wrapped_token.to_string(),
        0xCAFEBABEu128.into(),
    )
    .unwrap();

    let (_, msg, packet) = IncomingOrderBuilder::new(wrapped_token.as_bytes().into())
        .with_base_token(base_token)
        .with_destination_channel_id(destination_channel_id)
        .with_path(path)
        .with_base_token_path(reverse_channel_path(path).unwrap())
        .build();

    st.app
        .execute(
            st.ibc_host.clone(),
            wasm_execute(st.zkgm.clone(), &msg, vec![]).unwrap().into(),
        )
        .unwrap();

    // make sure transfer succeeded with the correct ack
    assert_eq!(
        PACKET_ACK
            .load(
                st.app.contract_storage(&st.ibc_host).as_ref(),
                commit_packets(&[packet]).into(),
            )
            .unwrap(),
        Ack {
            tag: TAG_ACK_FAILURE,
            inner_ack: Default::default()
        }
        .abi_encode_params()
    );
    let balance: cw20::BalanceResponse = st
        .app
        .wrap()
        .query_wasm_smart(
            &wrapped_token,
            &Cw20QueryMsg::Balance {
                address: st.minter.to_string(),
            },
        )
        .unwrap();
    // no transfer is made
    assert_eq!(balance.balance.u128(), 0xCAFEBABEu128 + 1000);
}

#[test]
fn test_on_channel_open_init_ok() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenInit {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: PROTOCOL_VERSION.to_string(),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Ok(Response::default()));
}

#[test]
fn test_on_channel_open_init_invalid_version() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenInit {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: "im-invalid".to_string(),
            relayer: "".to_string(),
        }),
    );

    assert!(matches!(
        result,
        Err(ContractError::InvalidIbcVersion { .. })
    ));
}

#[test]
fn test_on_channel_open_init_only_ibc() {
    let (mut deps, env, mut info, _) = init();
    info.sender = Addr::unchecked("not_ibc");
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenInit {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: PROTOCOL_VERSION.to_string(),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Err(ContractError::OnlyIBCHost));
}

#[test]
fn test_on_channel_open_try_ok() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenTry {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: PROTOCOL_VERSION.to_string(),
            counterparty_version: PROTOCOL_VERSION.to_string(),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Ok(Response::default()));
}

#[test]
fn test_on_channel_open_try_invalid_version() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenTry {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: "im-invalid".to_string(),
            counterparty_version: PROTOCOL_VERSION.to_string(),
            relayer: "".to_string(),
        }),
    );

    assert!(matches!(
        result,
        Err(ContractError::InvalidIbcVersion { .. })
    ));
}

#[test]
fn test_on_channel_open_try_invalid_counterparty_version() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenTry {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: PROTOCOL_VERSION.to_string(),
            counterparty_version: "im-invalid".to_string(),
            relayer: "".to_string(),
        }),
    );

    assert!(matches!(
        result,
        Err(ContractError::InvalidIbcVersion { .. })
    ));
}

#[test]
fn test_on_channel_open_try_only_ibc() {
    let (mut deps, env, mut info, _) = init();
    info.sender = Addr::unchecked("not_ibc");
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenTry {
            caller: "".into(),
            connection_id: ConnectionId!(1),
            channel_id: ChannelId!(1),
            version: PROTOCOL_VERSION.to_string(),
            counterparty_version: PROTOCOL_VERSION.to_string(),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Err(ContractError::OnlyIBCHost));
}

#[test]
fn test_on_channel_open_ack_and_confirm_noop() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenAck {
            caller: "".into(),
            channel_id: ChannelId!(1),
            counterparty_version: PROTOCOL_VERSION.to_string(),
            relayer: "".to_string(),
            counterparty_channel_id: ChannelId!(2),
        }),
    );

    assert_eq!(result, Ok(Response::default()));

    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenConfirm {
            caller: "".into(),
            channel_id: ChannelId!(1),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Ok(Response::default()));
}

#[test]
fn test_on_channel_close_init_impossible() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelCloseInit {
            caller: "".into(),
            channel_id: ChannelId!(1),
            relayer: "".to_string(),
        }),
    );

    assert!(matches!(result, Err(ContractError::Std(..))));
}

#[test]
fn test_on_channel_close_init_only_ibc() {
    let (mut deps, env, mut info, _) = init();
    info.sender = Addr::unchecked("not_ibc");
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelCloseInit {
            caller: "".into(),
            channel_id: ChannelId!(1),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Err(ContractError::OnlyIBCHost));
}

#[test]
fn test_on_channel_close_confirm_impossible() {
    let (mut deps, env, info, _) = init();
    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelCloseConfirm {
            caller: "".into(),
            channel_id: ChannelId!(1),
            relayer: "".to_string(),
        }),
    );

    assert!(matches!(result, Err(ContractError::Std(..))));
}

#[test]
fn test_on_channel_close_confirm_only_ibc() {
    let (mut deps, env, mut info, _) = init();
    info.sender = Addr::unchecked("not_ibc");
    let result = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::IbcUnionMsg(IbcUnionMsg::OnChannelCloseConfirm {
            caller: "".into(),
            channel_id: ChannelId!(1),
            relayer: "".to_string(),
        }),
    );

    assert_eq!(result, Err(ContractError::OnlyIBCHost));
}
