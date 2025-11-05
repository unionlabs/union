// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
// 

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module zkgm::zkgm {
    use std::string::{Self, String};
    use std::type_name;

    use sui::bcs;
    use sui::clock::Clock; 
    use sui::coin::{Self, Coin};
    use sui::event;
    use sui::object_bag::{Self, ObjectBag};
    use sui::table::{Self, Table};

    use ibc::commitment;
    use ibc::ibc;
    use ibc::packet::{Self, Packet};

    use owned_vault::owned_vault::{Self, OwnedVault};

    use escrow_vault::escrow_vault::{Self, EscrowVault};

    use zkgm::ack::{Self, Ack};
    use zkgm::batch;
    use zkgm::batch_ack;
    use zkgm::forward::{Self, Forward};
    use zkgm::token_metadata::{Self, TokenMetadata};
    use zkgm::token_order::{Self, TokenOrderV2};
    use zkgm::token_order_ack;
    use zkgm::helper;
    use zkgm::instruction::{Self, Instruction};
    use zkgm::sui_token_metadata;
    use zkgm::zkgm_packet;
    use zkgm::solver_metadata;

    #[test_only]
    use sui::test_scenario;
    

    // Constants
    const ACK_SUCCESS: u256 = 1;

    const INSTR_VERSION_0: u8 = 0x00;
    #[allow(unused_const)]
    const INSTR_VERSION_1: u8 = 0x01;
    const INSTR_VERSION_2: u8 = 0x02;

    const OP_FORWARD: u8 = 0x00;
    const OP_CALL : u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_TOKEN_ORDER: u8 = 0x03;

    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";
    
    const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
    const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
    const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;
    const TOKEN_ORDER_KIND_SOLVE: u8 = 0x03;

    // Errors
    const ACK_ERR_ONLYMAKER: vector<u8> = b"DEADC0DE";
    const E_UNSUPPORTED_VERSION: u64 = 5;
    const E_UNKNOWN_SYSCALL: u64 = 6;
    const E_INVALID_AMOUNT: u64 = 10;
    const E_INVALID_FILL_TYPE: u64 = 12;
    const E_NO_CALL_OPERATION: u64 = 17;
    const E_INVALID_FORWARD_INSTRUCTION: u64 = 18;
    const E_NO_COIN_IN_BAG: u64 = 23;
    const E_CHANNEL_BALANCE_PAIR_NOT_FOUND: u64 = 25;
    const E_ANOTHER_TOKEN_IS_REGISTERED: u64 = 26;
    const E_INVALID_BATCH_INSTRUCTION: u64 = 27;
    const E_INVALID_FORWARD_DESTINATION_CHANNEL_ID: u64 = 30;
    const E_INVALID_TOKEN_ORDER_KIND: u64 = 31;
    const E_UNWRAP_BASE_AMOUNT_SMALLER_THAN_QUOTE_AMOUNT: u64 = 32;
    const E_UNWRAP_METADATA_INVALID: u64 = 33;
    // const E_UNAUTHORIZED: u64 = 34;
    const E_INVALID_METADATA: u64 = 35;
    const E_INVALID_UNESCROW: u64 = 38;
    const E_ACK_SIZE_MISMATCHING: u64 = 42;
    const E_EXECUTION_NOT_COMPLETE: u64 = 43;
    const E_INVALID_SOLVER_ADDRESS: u64 = 45;
    const E_INVALID_QUOTE_TOKEN: u64 = 46;
    const E_EXECUTION_ALREADY_COMPLETE: u64 = 47;
    // const E_ONLY_MAKER: u64 = 0xdeadc0de;

    const OWNED_VAULT_OBJECT_KEY: vector<u8> = b"ucs03-zkgm-owned-vault";
    const ESCROW_VAULT_OBJECT_KEY: vector<u8> = b"ucs03-zkgm-escrow-vault";
    const ESCROW_VAULT_OBJECT_KEY_ADDR: vector<u8> = b"ucs03-zkgm-escrow-vault\0\0\0\0\0\0\0\0\0";

    public struct RelayStore has key {
        id: UID,
        in_flight_packet: Table<vector<u8>, Packet>,
        channel_balance: Table<ChannelBalancePair, u256>,
        token_origin: Table<vector<u8>, u256>,
        wrapped_denom_to_t: Table<vector<u8>, String>,
        object_store: ObjectBag,
        port: ibc::Port<address>,
    }

    public struct CreateWrappedToken has copy, drop, store {
        path: u256,
        channel_id: u32,
        base_token: vector<u8>,
        quote_token: vector<u8>,
        native_token: vector<u8>,
        metadata: vector<u8>,
        kind: u8
    }

    public struct ChannelBalancePair has copy, drop, store {
        channel: u32,
        path: u256,
        token: vector<u8>,
        metadata_image: vector<u8>,
    }

    public struct Session {}

    public struct ZkgmPacketCtx has drop {
        instruction_set: vector<Instruction>,
        // not by instruction but by set
        cursor: u64,
        acks: vector<vector<u8>>,
        path: u256,
        salt: vector<u8>,
    }

    public struct RecvCtx {
        packet_ctxs: vector<ZkgmPacketCtx>,
        packets: vector<Packet>,
        cursor: u64,
    }

    public struct ZkgmPacketAckCtx has drop {
        instruction_set: vector<Instruction>,
        // not by instruction but by set
        cursor: u64,
        path: u256,
        salt: vector<u8>,        
    }

    public struct SendCtx {
        channel_id: u32,
        salt: vector<u8>,
        instructions: vector<Instruction>
    }

    public struct AckCtx {
        packet_ctxs: vector<ZkgmPacketAckCtx>,
        packets: vector<Packet>,
        cursor: u64,
        acks: vector<Ack>,
        raw_acks: vector<vector<u8>>,
    }

    public struct TimeoutCtx {
        packet_ctx: ZkgmPacketAckCtx,
        packet: Packet,
    }

    fun init(ctx: &mut TxContext) {
        let id = object::new(ctx);

        let port = ibc::create_port(@zkgm, object::uid_to_address(&id), ctx);

        transfer::share_object(RelayStore {
            id: id,
            in_flight_packet: table::new(ctx),
            channel_balance: table::new(ctx),
            token_origin: table::new(ctx),
            object_store: object_bag::new(ctx),
            wrapped_denom_to_t: table::new(ctx),
            port,
        });
    }

    public fun register_owned_vault_cap(
        zkgm: &mut RelayStore,
        vault_cap: owned_vault::ZkgmCap,
    ) {
        zkgm.object_store.add(OWNED_VAULT_OBJECT_KEY, vault_cap);
    }

    public fun register_escrow_vault_cap(
        zkgm: &mut RelayStore,
        vault_cap: escrow_vault::ZkgmCap,
    ) {
        zkgm.object_store.add(ESCROW_VAULT_OBJECT_KEY, vault_cap);
    }

    public fun begin_send(
        channel_id: u32,
        salt: vector<u8>,
    ): SendCtx {
        SendCtx {
            channel_id,
            salt,
            instructions: vector::empty()
        }
    }

    public fun send_with_coin<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        ibc_store: &mut ibc::IBCStore,
        coin: Coin<T>,
        version: u8,
        opcode: u8,
        operand: vector<u8>,
        mut send_ctx: SendCtx,
        ctx: &mut TxContext
    ): SendCtx {
        let instruction = instruction::new(version, opcode, operand);
        let sender = ctx.sender();
        let coin = zkgm.verify_internal<T>(
            vault,
            ibc_store,
            option::some(coin),
            sender,
            send_ctx.channel_id,
            0,
            instruction,
            ctx
        );
        // This guarantees that the coin is used by some instruction
        coin.destroy_none();

        send_ctx.instructions.push_back(instruction);

        send_ctx
    }

    public fun end_send(
        zkgm: &RelayStore,
        ibc_store: &mut ibc::IBCStore,
        clock: &Clock,
        timeout_height: u64,
        timeout_timestamp: u64,
        send_ctx: SendCtx,
        ctx: &mut TxContext,
    ) {
        let SendCtx { channel_id, salt, instructions } = send_ctx;

        let instruction = if (instructions.length() > 1) {
            instructions.do_ref!(|instr| {
                assert!(helper::is_allowed_batch_instruction(instr.opcode()), E_INVALID_BATCH_INSTRUCTION);
            });
            instruction::new(INSTR_VERSION_0, OP_BATCH, batch::new(instructions).encode())
        } else {
            instructions[0]
        };

        ibc_store.send_packet(
            clock,
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_packet::new(salt, 0, instruction)),
            &zkgm.port,
            ctx
        );
    }

    /// When receiving a packet, the relayers **must** call this to begin
    /// a receiving session. Receiving is done in multi-steps.
    public fun begin_recv(
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_data: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
    ): RecvCtx {
        let mut packet_ctxs = vector::empty();
        let packet_len = packet_source_channels.length();
        let mut i = 0;
        let mut packets = vector::empty();
        while (i < packet_len) {
            let ibc_packet = packet::new(
                packet_source_channels[i],
                packet_destination_channels[i],
                packet_data[i],
                packet_timeout_heights[i],
                packet_timeout_timestamps[i]
            );
            let zkgm_packet = zkgm_packet::decode(ibc_packet.data());
            packet_ctxs.push_back(
                ZkgmPacketCtx {
                    instruction_set: extract_batch(zkgm_packet.instruction()),
                    cursor: 0,
                    acks: vector::empty(),
                    path: zkgm_packet.path(),
                    salt: zkgm_packet.salt(),
                }
            );
            packets.push_back(ibc_packet);
            i = i + 1;
        };

        RecvCtx {
            packet_ctxs,
            packets,
            cursor: 0
        }
    }

    fun extract_batch(instruction: Instruction): vector<Instruction> {
        if (instruction.opcode() == OP_BATCH) {
            if (instruction.version() != INSTR_VERSION_0) {
                abort E_UNSUPPORTED_VERSION
            };

            let instructions = batch::decode(instruction.operand()).instructions();
            let instr_len = instructions.length();
            let mut i = 0;
            while (i < instr_len) {
                if (!helper::is_allowed_batch_instruction(instructions.borrow(i).opcode())) {
                    abort E_INVALID_BATCH_INSTRUCTION
                };
                i = i + 1;  
            };

            *instructions
        } else {
            vector[instruction]
        }
    }

    public fun recv_packet<T>(
        ibc: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        clock: &Clock,
        relayer: address,
        relayer_msg: vector<u8>,
        mut exec_ctx: RecvCtx,
        ctx: &mut TxContext
    ): RecvCtx {
        // aborts if there is not a session
        let packet_cursor = exec_ctx.cursor;
        let packet = exec_ctx.packets[exec_ctx.cursor];
        let zkgm_packet = exec_ctx.packet_ctxs.borrow_mut(packet_cursor);

        let instr_len = zkgm_packet.instruction_set.length();
        let mut type_is_exhausted = false;
        while (zkgm_packet.cursor < instr_len) {
            let instruction = zkgm_packet.instruction_set[zkgm_packet.cursor];

            // if we previously run an instruction where type type `T` is used, we should
            // return exec_ctx and expect to be called with the appropriate type again.
            if (instruction.opcode() == OP_TOKEN_ORDER) {
                if (type_is_exhausted) {
                    return exec_ctx
                } else {
                    type_is_exhausted = true;
                };
            };
            
            let (ack, err) = zkgm.execute_internal<T>(
                vault,
                escrow_vault,
                ibc,
                packet,
                zkgm_packet,
                instruction,
                clock,
                relayer,
                relayer_msg,
                false,
                ctx
            );
            // TODO(aeryz): should we abort here?
            if (err != 0) {
                abort err
            };
            zkgm_packet.acks.push_back(ack);

            zkgm_packet.cursor = zkgm_packet.cursor + 1;
        };

        if (zkgm_packet.cursor == zkgm_packet.instruction_set.length()) {
            exec_ctx.cursor = exec_ctx.cursor + 1;  
        };

        exec_ctx
    }

    public fun end_recv(
        zkgm: &RelayStore,
        ibc: &mut ibc::IBCStore,
        clock: &Clock,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        relayer_msg: vector<u8>,
        exec_ctx: RecvCtx,
        ctx: &TxContext,
    ) {
        // make sure all packets are run
        assert!(exec_ctx.cursor == exec_ctx.packet_ctxs.length() , E_EXECUTION_NOT_COMPLETE);
        // make sure all instructions in the final packet are run
        let final_packet = exec_ctx.packet_ctxs.borrow(exec_ctx.packet_ctxs.length() - 1);
        assert!(final_packet.cursor == final_packet.instruction_set.length(), E_EXECUTION_NOT_COMPLETE);

        let mut i = 0;
        let mut acks = vector::empty();
        let packets_len = exec_ctx.packet_ctxs.length();

        while (i < packets_len) {
            let packet_ctx = exec_ctx.packet_ctxs.borrow(i);
            let ack = if (packet_ctx.instruction_set.length() == 1) {
                assert!(packet_ctx.acks.length() == 1, E_ACK_SIZE_MISMATCHING);

                packet_ctx.acks[0]
                if (!packet_ctx.acks[0].is_empty()) {
                    ack::success(packet_ctx.acks[0]).encode()
                } else {
                    ack::failure(b"").encode()
                }
            } else {
                ack::success(batch_ack::new(packet_ctx.acks).encode()).encode()
            };
            acks.push_back(ack);

            i = i + 1;
        };

        ibc.recv_packet(
            clock,
            exec_ctx.packets,
            relayer,
            vector[relayer_msg],
            proof,
            proof_height,
            acks,
            &zkgm.port,
            ctx
        );

        // dropping the execution ctx by decontstructing it
        let RecvCtx { .. } = exec_ctx;
    }

    fun execute_internal<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc: &mut ibc::IBCStore,
        packet: Packet,
        packet_ctx: &ZkgmPacketCtx,
        instruction: Instruction,
        clock: &Clock,
        relayer: address,
        relayer_msg: vector<u8>,
        intent: bool,
        ctx: &mut TxContext
    ): (vector<u8>, u64)  {
        let version = instruction.version();

        match (instruction.opcode()) {
            OP_TOKEN_ORDER => {
                if (version != INSTR_VERSION_2) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };
                zkgm.execute_token_order<T>(
                    vault,
                    escrow_vault,
                    packet,
                    relayer,
                    relayer_msg,
                    packet_ctx.path,
                    token_order::decode(instruction.operand()),
                    intent,
                    ctx
                )
            },
            OP_FORWARD => {
                if (version != INSTR_VERSION_0) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };

                zkgm.execute_forward(
                    ibc,
                    clock,
                    packet,
                    relayer,
                    relayer_msg,
                    packet_ctx.salt,
                    packet_ctx.path,
                    instruction.version(),
                    forward::decode(instruction.operand()),
                    intent,
                    ctx,
                )
            },
            OP_CALL => (vector::empty(), E_NO_CALL_OPERATION),
            _ => (vector::empty(), E_UNKNOWN_SYSCALL)
        }
    }

    public fun begin_ack(
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_data: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
    ): AckCtx {
        let mut packet_ctxs = vector::empty();
        let packet_len = packet_source_channels.length();
        let mut i = 0;
        let mut packets = vector::empty();
        let mut acks = vector::empty();
        while (i < packet_len) {
            let ibc_packet = packet::new(
                packet_source_channels[i],
                packet_destination_channels[i],
                packet_data[i],
                packet_timeout_heights[i],
                packet_timeout_timestamps[i]
            );
            let zkgm_packet = zkgm_packet::decode(ibc_packet.data());
            packet_ctxs.push_back(
                ZkgmPacketAckCtx {
                    instruction_set: extract_batch(zkgm_packet.instruction()),
                    cursor: 0,
                    path: zkgm_packet.path(),
                    salt: zkgm_packet.salt(),
                }
            );
            packets.push_back(ibc_packet);
            acks.push_back(ack::decode(&acknowledgements[i]));
            i = i + 1;
        };

        AckCtx {
            packet_ctxs,
            packets,
            acks,
            raw_acks: acknowledgements,
            cursor: 0
        }
    }

    public fun acknowledge_packet<T>(
        ibc: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        relayer: address,
        mut ack_ctx: AckCtx,
        ctx: &mut TxContext
    ): AckCtx {
        // aborts if there is not a session
        let packet_cursor = ack_ctx.cursor;
        let packet = ack_ctx.packets[packet_cursor];
        let ack = ack_ctx.acks[packet_cursor];
        let raw_ack = ack_ctx.raw_acks[packet_cursor];
        let zkgm_packet = ack_ctx.packet_ctxs.borrow_mut(packet_cursor);

        let instr_len = zkgm_packet.instruction_set.length();
        let mut type_is_exhausted = false;

        let batch_ack = if (instr_len > 1) {
            batch_ack::decode(ack.inner_ack()).acknowledgements()
        } else {
            vector::empty()
        };

        while (zkgm_packet.cursor < instr_len) {
            let instruction = zkgm_packet.instruction_set[zkgm_packet.cursor];

            // if we previously run an instruction where type type `T` is used, we should
            // return ack_ctx and expect to be called with the appropriate type again.
            if (instruction.opcode() == OP_TOKEN_ORDER) {
                if (type_is_exhausted) {
                    return ack_ctx
                } else {
                    type_is_exhausted = true;
                };
            };

            if (helper::is_forwarded_packet(zkgm_packet.salt)) {
                let packet_hash = commitment::commit_packet(&packet);

                if (zkgm.in_flight_packet.contains(packet_hash)) {
                    let parent = zkgm.in_flight_packet.remove(packet_hash);
                    ibc.write_acknowledgement(parent, raw_ack, &zkgm.port);
                    zkgm_packet.cursor = zkgm_packet.cursor + 1;
                    continue
                };
            };

            zkgm.acknowledge_internal<T>(
                vault,
                escrow_vault,
                packet,
                relayer,
                zkgm_packet.path,
                if (instr_len > 1) {
                    helper::derive_batch_salt(zkgm_packet.cursor, zkgm_packet.salt)
                } else {
                    zkgm_packet.salt
                },
                instruction,
                ack.tag() == ACK_SUCCESS,
                if (instr_len > 1 && ack.tag() == ACK_SUCCESS) {
                    batch_ack[zkgm_packet.cursor]
                } else {
                    *ack.inner_ack()
                },
                ctx
            );
            
            zkgm_packet.cursor = zkgm_packet.cursor + 1;
        };

        if (zkgm_packet.cursor == zkgm_packet.instruction_set.length()) {
            ack_ctx.cursor = ack_ctx.cursor + 1;  
        };

        ack_ctx
    }
    
    public fun end_ack(
        zkgm: &RelayStore,
        ibc: &mut ibc::IBCStore,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        ack_ctx: AckCtx,
        ctx: &TxContext,
    ) {
        // make sure all packets are run
        assert!(ack_ctx.cursor == ack_ctx.packet_ctxs.length() , E_EXECUTION_NOT_COMPLETE);
        // make sure all instructions in the final packet are run
        let final_packet = ack_ctx.packet_ctxs.borrow(ack_ctx.packet_ctxs.length() - 1);
        assert!(final_packet.cursor == final_packet.instruction_set.length(), E_EXECUTION_NOT_COMPLETE);

        ibc.acknowledge_packet(
            ack_ctx.packets,
            ack_ctx.raw_acks,
            proof,
            proof_height,
            relayer,
            &zkgm.port,
            ctx
        );

        // dropping the ctx by decontstructing it
        let AckCtx { .. } = ack_ctx;
    }

    
    public fun begin_timeout(
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
    ): TimeoutCtx {
        let zkgm_packet = zkgm_packet::decode(&packet_data);
        TimeoutCtx {
            packet_ctx: ZkgmPacketAckCtx {
                instruction_set: extract_batch(zkgm_packet.instruction()),
                cursor: 0,
                path: zkgm_packet.path(),
                salt: zkgm_packet.salt(),
            },
            packet: packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp,
            ),
        }
    }

    public fun timeout_packet<T>(
        ibc: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        relayer: address,
        mut timeout_ctx: TimeoutCtx,
        ctx: &mut TxContext
    ): TimeoutCtx {
        let packet = timeout_ctx.packet;
        let zkgm_packet = &mut timeout_ctx.packet_ctx;
        let instr_len = zkgm_packet.instruction_set.length();

        assert!(zkgm_packet.cursor < instr_len, E_EXECUTION_ALREADY_COMPLETE);

        let mut type_is_exhausted = false;

        while (zkgm_packet.cursor < instr_len) {
            let instruction = zkgm_packet.instruction_set[zkgm_packet.cursor];

            // if we previously run an instruction where type type `T` is used, we should
            // return timeout_ctx and expect to be called with the appropriate type again.
            if (instruction.opcode() == OP_TOKEN_ORDER) {
                if (type_is_exhausted) {
                    return timeout_ctx
                } else {
                    type_is_exhausted = true;
                };
            };

            if (helper::is_forwarded_packet(zkgm_packet.salt)) {
                let packet_hash = commitment::commit_packet(&packet);

                if (zkgm.in_flight_packet.contains(packet_hash)) {
                    let parent = zkgm.in_flight_packet.remove(packet_hash);
                    ibc.write_acknowledgement(parent, ack::failure(ACK_EMPTY).encode(), &zkgm.port);
                    zkgm_packet.cursor = zkgm_packet.cursor + 1;
                    continue
                };
            };

            zkgm.timeout_internal<T>(
                vault,
                packet,
                relayer,
                zkgm_packet.path,
                instruction,
                ctx,
            );
            
            zkgm_packet.cursor = zkgm_packet.cursor + 1;
        };

        timeout_ctx
    }
    
    public fun end_timeout(
        ibc: &mut ibc::IBCStore,
        zkgm: &RelayStore,
        proof: vector<u8>,
        proof_height: u64,
        timeout_ctx: TimeoutCtx,
        ctx: &TxContext,
    ) {
        // make sure all instructions in the packet are run
        assert!(timeout_ctx.packet_ctx.cursor == timeout_ctx.packet_ctx.instruction_set.length(), E_EXECUTION_NOT_COMPLETE);

        ibc.timeout_packet(
            timeout_ctx.packet,
            proof,
            proof_height,
            &zkgm.port,
            ctx,
        );

        // dropping the ctx by decontstructing it
        let TimeoutCtx { .. } = timeout_ctx;
    }

    fun market_maker_fill<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        path: u256,
        receiver: address,
        order: TokenOrderV2,
        intent: bool,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        if (order.kind() == TOKEN_ORDER_KIND_SOLVE) {
            return zkgm.solver_fill<T>(
                vault,
                escrow_vault,
                ibc_packet,
                order,
                path,
                relayer,
                intent,
                ctx,
            )
        } else {
            let quote_amount = order.quote_amount() as u64;
            if (quote_amount > 0){
                // TODO(aeryz): handle NATIVE_TOKEN_ERC_7528_ADDRESS case            
                // TODO(aeryz): make sure that distribute here is correct
                zkgm.distribute_coin<T>(receiver, quote_amount, ctx);
            };
        };

        (token_order_ack::new(
            FILL_TYPE_MARKETMAKER,
            relayer_msg
        ).encode(), 0)
    }

    fun solver_fill<T>(
        zkgm: &RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc_packet: Packet,
        order: TokenOrderV2,
        path: u256,
        relayer: address,
        intent: bool,
        ctx: &mut TxContext,
    ): (vector<u8>, u64) {
        let metadata = solver_metadata::decode(order.metadata());

        let quote_token = *order.quote_token();

        if (type_name::with_defining_ids<T>().into_string().into_bytes() != quote_token) {
            return (vector::empty(), E_INVALID_QUOTE_TOKEN)
        };
        
        let solver = metadata.solver_address();

        let (res, err) = match (solver) {
            OWNED_VAULT_OBJECT_KEY => vault.solve<T>(
                zkgm.object_store.borrow(OWNED_VAULT_OBJECT_KEY),
                ibc_packet,
                *order.base_token(),
                quote_token,
                order.base_amount(),
                order.quote_amount(),
                *order.receiver(),
                path,
                relayer,
                vector::empty(),
                intent,
                ctx
            ),
            ESCROW_VAULT_OBJECT_KEY => escrow_vault.solve<T>(
                zkgm.object_store.borrow(ESCROW_VAULT_OBJECT_KEY),
                ibc_packet,
                *order.base_token(),
                quote_token,
                order.base_amount(),
                order.quote_amount(),
                *order.receiver(),
                path,
                relayer,
                vector::empty(),
                intent,
                ctx
            ),
            _ => (vector::empty(), E_INVALID_SOLVER_ADDRESS)
        };

        if (err != 0) {
            (vector::empty(), err)
        } else {
            (token_order_ack::new(
                FILL_TYPE_MARKETMAKER,
                res
            ).encode(), 0)
        }
    }

    fun split_coin<T>(
        relay_store: &mut RelayStore,
        amount: u64,
        ctx: &mut TxContext
    ): Coin<T> {
        let typename_t = type_name::with_defining_ids<T>();
        let key = typename_t.into_string();
        if(!relay_store.object_store.contains(string::from_ascii(key))) {
            abort E_NO_COIN_IN_BAG
        };
        let coin = relay_store.object_store.borrow_mut<String, Coin<T>>(string::from_ascii(key));

        coin.split<T>(amount, ctx)
    }

    fun distribute_coin<T>(
        relay_store: &mut RelayStore,
        receiver: address,
        amount: u64,
        ctx: &mut TxContext
    ) {
        let transferred_coin = relay_store.split_coin<T>(amount, ctx);
        transfer::public_transfer(transferred_coin, receiver);
    }

    fun protocol_fill_mint<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        _channel_id: u32,
        _path: u256,
        wrapped_token: vector<u8>,
        receiver: address,
        relayer: address,
        order: &TokenOrderV2,
        metadata: Option<TokenMetadata>,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let quote_amount = order.quote_amount();

        let fee = (order.base_amount() - order.quote_amount()) as u64;
        // if this token is minted for the first time, then we need to ensure that its always minting the same T
        if (!zkgm.claim_wrapped_denom<T>(vault, wrapped_token, metadata)) {
            return (vector::empty(), E_ANOTHER_TOKEN_IS_REGISTERED)
        };
        if (quote_amount > 0) {
            vault.mint<T>(
                zkgm.object_store.borrow(OWNED_VAULT_OBJECT_KEY),
                quote_amount as u64,
                receiver,
                ctx
            );
        };
        if (fee > 0){
            vault.mint<T>(
                zkgm.object_store.borrow(OWNED_VAULT_OBJECT_KEY),
                fee,
                relayer,
                ctx
            );
        };

        (token_order_ack::new(
            FILL_TYPE_PROTOCOL,
            ACK_EMPTY
        ).encode(), 0)
    }

    fun protocol_fill_unescrow<T>(
        zkgm: &mut RelayStore,
        channel_id: u32,
        path: u256,
        base_token: vector<u8>,
        quote_token: vector<u8>,
        receiver: address,
        relayer: address,
        base_amount: u64,
        quote_amount: u64,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let fee = base_amount - quote_amount;

        // If the base token path is being unwrapped, it's going to be non-zero.
        if (zkgm.decrease_outstanding(
            channel_id,
            helper::reverse_channel_path(path), 
            quote_token, 
            base_token,
            base_amount as u256
        ) != 0) {
            return (vector::empty(), 0)
        };

        // TODO(aeryz): handle quote_token == NATIVE_TOKEN_ERC_7528_ADDRESS for gas station

        // Here we just need to split our coins to the receiver and the relayer
        if(quote_amount > 0) {
            zkgm.distribute_coin<T>(receiver, quote_amount, ctx)
        };

        if(fee > 0){
            zkgm.distribute_coin<T>(relayer, fee, ctx)
        };

        (token_order_ack::new(
            FILL_TYPE_PROTOCOL,
            ACK_EMPTY
        ).encode(), 0)
    }

    fun execute_forward(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        clock: &Clock,
        ibc_packet: Packet,
        _relayer: address,
        _relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        version: u8,
        forward: Forward,
        intent: bool,
        ctx: &TxContext
    ): (vector<u8>, u64) {
        if (!helper::is_allowed_forward(forward.instruction().opcode())) {
            return (vector::empty(), E_INVALID_FORWARD_INSTRUCTION)
        };

        // We cannot allow market makers to fill packets containing forward
        // instruction. This would allow them to submit of a proof and fill via the
        // protocol on destination for a fake forward.

        // Instead, they must first fill on destination the orders, awaits finality
        // to settle the forward, then cascade acknowledge.
        if (intent) {
            return (ACK_ERR_ONLYMAKER, 0)
        };

        let (tail_path, previous_destination_channel_id) = helper::dequeue_channel_from_path(forward.path());
        let (continuation_path, next_source_channel_id) = helper::dequeue_channel_from_path(tail_path);
        if (ibc_packet.destination_channel_id() != previous_destination_channel_id) {
            return (vector::empty(), E_INVALID_FORWARD_DESTINATION_CHANNEL_ID)
        };

        let next_instruction = if (continuation_path == 0) {
            *forward.instruction()
        } else {
            instruction::new(
                version,
                OP_FORWARD,
                forward::new(
                    continuation_path,
                    forward.timeout_height(),
                    forward.timeout_timestamp(),
                    *forward.instruction(),
                ).encode()
            )
        };

        let sent_packet = ibc.send_packet(
            clock,
            next_source_channel_id,
            forward.timeout_height(),
            forward.timeout_timestamp(),
            zkgm_packet::new(
                helper::derive_forward_salt(salt),
                helper::update_channel_path(
                    helper::update_channel_path(
                        path, previous_destination_channel_id,
                    ),
                    next_source_channel_id,
                ),
                next_instruction
            ).encode(),
            &zkgm.port,
            ctx
        );

        // Guaranteed to be unique by the above sendPacket
        let commitment_key = commitment::batch_packets_commitment_key(
            commitment::commit_packet(&sent_packet)
        );
        zkgm.in_flight_packet.add(commitment_key, ibc_packet);

        (ACK_EMPTY, 0)
    }

    fun execute_token_order<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        path: u256,
        order: TokenOrderV2,
        intent: bool,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let quote_token = *order.quote_token();
        let receiver = bcs::new(*order.receiver()).peel_address();

        // For intent packets, the protocol is not allowed to provide any fund
        // as the packet has not been checked for membership poof. Instead, we
        // know the market maker will be repaid on the source chain, if and only
        // if the currently executing packet hash had been registered as sent on
        // the source. In other words, the market maker is unable to lie.
        if (intent || order.kind() == TOKEN_ORDER_KIND_SOLVE) {
            return zkgm.market_maker_fill<T>(
                vault,
                escrow_vault,
                ibc_packet,
                relayer,
                relayer_msg,
                path,
                receiver,
                order,
                intent,
                ctx,
            )
        };

        let base_amount_covers_quote_amount = order.base_amount() >= order.quote_amount();

        let (wrapped_token, metadata) = match (order.kind()) {
            TOKEN_ORDER_KIND_UNESCROW => {
                if (!base_amount_covers_quote_amount) {
                    return (vector::empty(), E_UNWRAP_BASE_AMOUNT_SMALLER_THAN_QUOTE_AMOUNT)   
                };

                // TODO: add rate limit here later
                return zkgm.protocol_fill_unescrow<T>(
                    ibc_packet.destination_channel_id(), 
                    path, 
                    *order.base_token(),
                    quote_token, 
                    receiver, 
                    relayer, 
                    order.base_amount() as u64, 
                    order.quote_amount() as u64, 
                    ctx
                )
            },
            TOKEN_ORDER_KIND_ESCROW => {
                // we expect the metadata to be a 32-byte hash
                if (order.metadata().length() != 32) {
                    return (vector::empty(), E_UNWRAP_METADATA_INVALID)
                };
                let wrapped_token = helper::compute_salt_from_metadata_image(
                    path,
                    ibc_packet.destination_channel_id(),
                    *order.base_token(),
                    *order.metadata()
                );
                (wrapped_token, option::none())
            },
            TOKEN_ORDER_KIND_INITIALIZE => {
                let wrapped_token = helper::compute_salt(
                    path,
                    ibc_packet.destination_channel_id(),
                    *order.base_token(),
                    *order.metadata()
                );
                let metadata = token_metadata::decode(order.metadata());
                (wrapped_token, option::some(metadata))
                
            },
            _ => return (vector::empty(), E_INVALID_TOKEN_ORDER_KIND)
        };

        if (quote_token == wrapped_token && base_amount_covers_quote_amount) {
            // TODO: rate limit
            if (!zkgm.save_token_origin(wrapped_token, path, ibc_packet.destination_channel_id())) {
                event::emit(CreateWrappedToken {
                    path,
                    channel_id: ibc_packet.destination_channel_id(), 
                    base_token: *order.base_token(),
                    quote_token,
                    native_token: type_name::with_defining_ids<T>().into_string().into_bytes(),
                    metadata: *order.metadata(),
                    kind: order.kind()
                });
            };
            
            // We expect the token to be deployed already here and the treasury cap is registered previously with type T
            zkgm.protocol_fill_mint<T>(
                vault,
                ibc_packet.destination_channel_id(), 
                path, 
                wrapped_token, 
                receiver, 
                relayer, 
                &order,
                metadata,
                ctx
            )
        } else {            
            // We also allow market makers to fill orders after finality. This
            // allow orders that combines protocol and mm filling (wrapped vs
            // non wrapped assets).
            zkgm.market_maker_fill<T>(
                vault,
                escrow_vault,
                ibc_packet,
                relayer,
                relayer_msg,
                path,
                receiver,
                order,
                intent,
                ctx,
            )
        }
    }

    fun add_or_update_table<T: drop + store + copy, P: drop + store>(table: &mut Table<T, P>, key: T, value: P) {
        if (table.contains(key)) {
            let val = table.borrow_mut(key);
            *val = value;
        } else {
            table.add(key, value);
        }
    }

    fun decrease_outstanding(
        relay_store: &mut RelayStore,
        channel: u32,
        path: u256,
        token: vector<u8>,
        metadata_image: vector<u8>,
        amount: u256
    ): u64 {
        let pair = ChannelBalancePair {
            channel,
            path,
            token,
            metadata_image,    
        };
        if(!relay_store.channel_balance.contains(pair)) {
            return E_CHANNEL_BALANCE_PAIR_NOT_FOUND
        };
        let channel_balance = *relay_store.channel_balance.borrow(pair);
        if (channel_balance < amount) {
            return E_INVALID_AMOUNT
        };
        let new_balance = channel_balance - amount;
        add_or_update_table<ChannelBalancePair, u256>(
            &mut relay_store.channel_balance,
            pair,
            new_balance
        );

        0
    }
        
    fun increase_outstanding(
        relay_store: &mut RelayStore,
        channel: u32,
        path: u256,
        token: vector<u8>,
        metadata_image: vector<u8>,
        amount: u256
    ) {
        let pair = ChannelBalancePair{
            channel,
            path,
            token,
            metadata_image,
        };

        let mut channel_balance = 0;
        if(relay_store.channel_balance.contains(pair)) {
            channel_balance = *relay_store.channel_balance.borrow(pair);
        };
        let new_balance = channel_balance + amount;
        add_or_update_table<ChannelBalancePair, u256>(
            &mut relay_store.channel_balance,
            pair,
            new_balance
        );
    }

    fun verify_internal<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        ibc_store: &mut ibc::IBCStore,
        coin: Option<Coin<T>>,
        sender: address,
        channel_id: u32,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ): Option<Coin<T>> {
        let version = instruction.version();
        match (instruction.opcode()) {
            OP_TOKEN_ORDER => {
                if (version == INSTR_VERSION_2) {
                    zkgm.verify_token_order<T>(
                        vault,
                        coin.destroy_some(),
                        sender,
                        channel_id,
                        path,
                        token_order::decode(instruction.operand()),
                        ctx
                    );
                    option::none<Coin<T>>()
                } else {
                    abort E_UNSUPPORTED_VERSION
                }
            },
            OP_FORWARD => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.verify_forward<T>(
                    vault,
                    ibc_store,
                    coin,
                    sender,
                    channel_id,
                    forward::decode(instruction.operand()),
                    ctx,
                )
            },
            OP_CALL => abort E_NO_CALL_OPERATION,
            _ => abort E_UNKNOWN_SYSCALL,
        }
    }

    fun verify_token_order<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        coin: Coin<T>,
        _sender: address,
        channel_id: u32,
        path: u256,
        order: TokenOrderV2,
        _: &TxContext
    ) {
        let base_token = *order.base_token();

        if (order.kind() == TOKEN_ORDER_KIND_UNESCROW) {
            let mut origin = 0;    
            if (zkgm.token_origin.contains(base_token)) {
                origin = *zkgm.token_origin.borrow(base_token);
            };
            let (intermediate_channel_path, destination_channel_id) =
                helper::pop_channel_from_path(origin);
            let is_inverse_intermediate_path = path == helper::reverse_channel_path(intermediate_channel_path);
            let is_sending_back_to_same_channel = destination_channel_id == channel_id;

            assert!(order.metadata().length() == 32, E_INVALID_METADATA);
            let wrapped_token = helper::compute_salt_from_metadata_image(
                intermediate_channel_path,
                channel_id,
                *order.quote_token(),
                *order.metadata()
            );

            let is_unwrapping = order.base_token() == wrapped_token;

            if (
                !(
                    is_unwrapping && is_inverse_intermediate_path
                        && is_sending_back_to_same_channel
                )
            ) {
                abort E_INVALID_UNESCROW
            };
            
            // We don't have to verify that metadataImage matches the stored one
            // because the prediction would fail otherwise and we would fall
            // back in the else branch.
            vault.burn<T>(
                zkgm.object_store.borrow<_, owned_vault::ZkgmCap>(OWNED_VAULT_OBJECT_KEY),
                coin
            );
        } else {
            
            zkgm.increase_outstanding(
                channel_id, 
                path, 
                base_token,
                *order.quote_token(),
                order.base_amount()
            );
            zkgm.save_coin_to_bag<T>(coin);

            // TODO(aeryz): handle gas station here
        };
    }

    fun verify_forward<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        ibc_store: &mut ibc::IBCStore,
        coin: Option<Coin<T>>,
        sender: address,
        channel_id: u32,
        forward_packet: Forward,
        ctx: &mut TxContext
    ): Option<Coin<T>> {
        let instruction = forward_packet.instruction();

        if(!helper::is_allowed_forward(instruction.opcode())) {
            abort E_INVALID_FORWARD_INSTRUCTION
        };
        zkgm.verify_internal<T>(
            vault,
            ibc_store,
            coin,
            sender,
            channel_id,
            forward_packet.path(),
            *instruction,
            ctx
        )
    }

    fun acknowledge_internal<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        salt: vector<u8>,
        instruction: Instruction,
        success: bool,
        inner_ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        let version = instruction.version();

        match (instruction.opcode()) {
            OP_TOKEN_ORDER => {
                if (version == INSTR_VERSION_2) {
                    let order = token_order::decode(instruction.operand());
                    
                    zkgm.acknowledge_token_order<T>(
                        vault,
                        escrow_vault,
                        ibc_packet,
                        relayer,
                        path,
                        salt,
                        order,
                        success,
                        inner_ack,
                        ctx
                    );
                } else {
                    abort E_UNSUPPORTED_VERSION
                };
            },
            OP_FORWARD => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.acknowledge_forward<T>(
                    vault,
                    escrow_vault,
                    ibc_packet,
                    relayer,
                    salt,
                    forward::decode(instruction.operand()),
                    success,
                    inner_ack,
                    ctx,
                );
            },
            OP_CALL => {
                abort E_NO_CALL_OPERATION
            },            
            _ => abort E_UNKNOWN_SYSCALL
        };
    }

    fun acknowledge_token_order<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc_packet: Packet,
        _relayer: address,
        path: u256,
        _salt: vector<u8>,
        order: TokenOrderV2,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        if (success) {
            let asset_order_ack = token_order_ack::decode(&ack);
            if (asset_order_ack.fill_type() == FILL_TYPE_PROTOCOL) {
                // The protocol filled, fee was paid to relayer.
            } else if(
                asset_order_ack.fill_type() == FILL_TYPE_MARKETMAKER
            ) {
                let market_maker = bcs::new(*asset_order_ack.market_maker()).peel_address();

                if (order.kind() == TOKEN_ORDER_KIND_UNESCROW) {
                    vault.mint<T>(
                        zkgm.object_store.borrow(OWNED_VAULT_OBJECT_KEY),
                        order.base_amount() as u64,
                        market_maker,
                        ctx
                    );
                } else {
                    let res = zkgm.decrease_outstanding(
                        ibc_packet.source_channel_id(), 
                        path, 
                        *order.base_token(), 
                        *order.quote_token(),
                        order.base_amount() as u256
                    );
                    assert!(res == 0, res);
                    if (market_maker == @0x0) {
                        let coin = {
                            let coin: &mut Coin<T> = zkgm.object_store.borrow_mut(
                                string::from_ascii(type_name::with_defining_ids<T>().into_string())
                            );
                            coin.split<T>(order.base_amount() as u64, ctx)
                        };
                        vault.burn<T>(
                            zkgm.object_store.borrow(OWNED_VAULT_OBJECT_KEY),
                            coin
                        );
                    } else if (bcs::to_bytes(&market_maker) == ESCROW_VAULT_OBJECT_KEY_ADDR) {
                        let coin = zkgm.split_coin<T>(order.base_amount() as u64, ctx);
                        escrow_vault.escrow<T>(
                            zkgm.object_store.borrow(ESCROW_VAULT_OBJECT_KEY),
                            coin
                        );
                    } else {
                        zkgm.distribute_coin<T>(market_maker, order.base_amount() as u64, ctx);
                    };
                }
            } else {
                abort E_INVALID_FILL_TYPE
            };
        } else {
            zkgm.refund<T>(
                vault,
                ibc_packet.source_channel_id(), 
                path, 
                order,
                ctx
            )
        };
    }


    fun acknowledge_forward<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        escrow_vault: &mut EscrowVault,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        forward_packet: Forward,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        zkgm.acknowledge_internal<T>(
            vault,
            escrow_vault,
            ibc_packet,
            relayer,
            forward_packet.path(),
            salt,
            *forward_packet.instruction(),
            success,
            ack,
            ctx
        )
    }

    fun timeout_internal<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ) {
        let version = instruction.version();

        match (instruction.opcode()) {
            OP_TOKEN_ORDER => {
                if (version == INSTR_VERSION_2) {
                    zkgm.timeout_token_order<T>(
                        vault,
                        ibc_packet,
                        path,
                        token_order::decode(instruction.operand()),
                        ctx
                    );
                } else {
                    abort E_UNSUPPORTED_VERSION
                };
                      
            },
            OP_FORWARD => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.timeout_forward<T>(
                    vault,
                    ibc_packet,
                    relayer,
                    path,
                    forward::decode(instruction.operand()),
                    ctx
                );
            },
            OP_CALL => {
                abort E_NO_CALL_OPERATION
            },
            _ => abort E_UNKNOWN_SYSCALL
        };
    }

    fun timeout_token_order<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        packet: Packet,
        path: u256,
        order: TokenOrderV2,
        ctx: &mut TxContext
    ) {
        zkgm.refund<T>(
            vault,
            packet.source_channel_id(), 
            path,
            order,
            ctx
        );
    }

    fun refund<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        source_channel: u32,
        path: u256,
        order: TokenOrderV2,
        ctx: &mut TxContext
    ) {
        let sender = bcs::new(*order.sender()).peel_address();

        if (order.kind() == TOKEN_ORDER_KIND_UNESCROW) {
            vault.mint<T>(
                zkgm.object_store.borrow(OWNED_VAULT_OBJECT_KEY),
                order.base_amount() as u64, sender, ctx
            );
        } else {
            zkgm.decrease_outstanding(
                source_channel,
                path, 
                *order.base_token(),
                *order.quote_token(),
                order.base_amount() as u256
            );
            
            zkgm.distribute_coin<T>(sender, order.base_amount() as u64, ctx);
        };        
    }

    fun timeout_forward<T>(
        zkgm: &mut RelayStore,
        vault: &mut OwnedVault,
        packet: Packet,
        relayer: address,
        path: u256,
        forward_packet: Forward,
        ctx: &mut TxContext
    ) {
        zkgm.timeout_internal<T>(
            vault,
            packet, 
            relayer, 
            path, 
            *forward_packet.instruction(), 
            ctx
        )
    }

    fun claim_wrapped_denom<T>(
        zkgm: &mut RelayStore,
        vault: &OwnedVault,
        wrapped_denom: vector<u8>,
        metadata: Option<TokenMetadata>,
    ): bool {
        let typename_t = type_name::with_defining_ids<T>();
        let key = string::from_ascii(typename_t.into_string());
        if (!zkgm.wrapped_denom_to_t.contains(wrapped_denom)) {
            if (metadata.is_none()) {
                // Means a hash is provided. We can't do the necessary checks when we don't know the full
                // metadata.
                return false                
            };

            let metadata = metadata.destroy_some();
            let sui_metadata = sui_token_metadata::decode(*metadata.initializer());
        
            let m = vault.get_metadata<T>();

            if (m.name() != sui_metadata.name()
                || m.symbol() != sui_metadata.symbol()
                || m.decimals() != sui_metadata.decimals()
                || m.owner() != sui_metadata.owner()
                || m.icon_url() != sui_metadata.icon_url()
                || m.description() != sui_metadata.description()) {
                return false
            };
            
            zkgm.wrapped_denom_to_t.add(wrapped_denom, key);
            true
        } else {
            let claimed_key = zkgm.wrapped_denom_to_t.borrow(wrapped_denom);
            claimed_key == key
        }
    }

    fun save_coin_to_bag<T>(
        relay_store: &mut RelayStore,
        coin: Coin<T>
    ) {
        let typename_t = type_name::with_defining_ids<T>();
        let key = type_name::into_string(typename_t);
        if(relay_store.object_store.contains(string::from_ascii(key))) {
            let self_coin = relay_store.object_store.borrow_mut(string::from_ascii(key));
            coin::join(self_coin, coin)
        } else{
            relay_store.object_store.add(string::from_ascii(key), coin)
        }
    }

    fun save_token_origin(
        zkgm: &mut RelayStore,
        wrapped_token: vector<u8>,
        path: u256,
        channel_id: u32
    ): bool {
        let updated_channel_path = helper::update_channel_path(path, channel_id);
        if (!zkgm.token_origin.contains(wrapped_token)) {
            zkgm.token_origin.add(wrapped_token, updated_channel_path);
            false
        } else {
            true
        }
    }

    public(package) fun port(zkgm: &RelayStore): &ibc::Port<address> {
        &zkgm.port
    }

    #[test_only]
    public fun init_for_tests(ctx: &mut TxContext) {
        let id = object::new(ctx);

        transfer::share_object(RelayStore {
            id: id,
            in_flight_packet: table::new(ctx),
            channel_balance: table::new(ctx),
            token_origin: table::new(ctx),
            object_store: object_bag::new(ctx),
            wrapped_denom_to_t: table::new(ctx),
        });
    }

    #[test]
    fun test_is_valid_version_true() {
        assert!(is_valid_version(string::utf8(b"ucs03-zkgm-0")), 1)
    }

    #[test]
    fun test_is_valid_version_false() {
        assert!(!is_valid_version(string::utf8(b"ucs03-zkgm-1")), 1)
    }

    #[test]
    fun test_increase_then_decrease_outstanding_ok() {
        let mut t = test_scenario::begin(@0x0);

        t.next_tx(@0x0);
        init(t.ctx());

        t.next_tx(@0x0);
        let mut store = t.take_shared<RelayStore>();

        let channel: u32 = 7;
        let path: u256 = 0xAA;
        let token: vector<u8> = b"TKN";
        let meta: vector<u8> = b"IMG";
        let amt: u256 = 5;

        increase_outstanding(&mut store, channel, path, token, meta, amt);
        let rc1 = decrease_outstanding(&mut store, channel, path, token, meta, amt);
        assert!(rc1 == 0, rc1);

        test_scenario::return_shared(store);
        t.end();
    }

    #[test]
    fun test_decrease_outstanding_pair_not_found_code() {
        let mut t = test_scenario::begin(@0x0);

        t.next_tx(@0x0);
        init(t.ctx());

        t.next_tx(@0x0);
        let mut store = t.take_shared<RelayStore>();

        let rc = decrease_outstanding(
            &mut store,
            1,
            0x1,
            b"A",
            b"B",
            1
        );
        assert!(rc == E_CHANNEL_BALANCE_PAIR_NOT_FOUND, rc);

        test_scenario::return_shared(store);
        t.end();
    }

    #[test]
    fun test_send_flow_escrow_sui_single_tx() {
        use std::string;
        use std::ascii;
        use std::type_name;
        use sui::test_scenario;
        use sui::clock;
        use sui::clock::Clock;
        use sui::coin;
        use sui::bcs;

        let mut t = test_scenario::begin(@0x0);
        let (mut ibc, mut zkgm, mut owned_vault, escrow_vault) = prepare_test_ctx(&mut t);
        let zkgm_cap = test_scenario::take_from_sender<owned_vault::ZkgmCap>(&t);
        zkgm.register_owned_vault_cap(zkgm_cap);
      
        let amount: u64 = 1_000;
        let sui_coin = coin::mint_for_testing<sui::sui::SUI>(amount, t.ctx());

        let sender = t.sender();
        let receiver = t.sender();
        let base_token = b"BASE";
        let quote_token = std::type_name::with_defining_ids<sui::sui::SUI>().into_string().into_bytes();
        let md = vector[
            0u8,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,1,
            2,3,4,5,6,7,8,9,
            10,11,12,13,14,15,16,17
        ];

        let order = zkgm::token_order::new(
            sui::bcs::to_bytes(&sender),
            sui::bcs::to_bytes(&receiver),
            base_token,
            amount as u256,
            quote_token,
            0,
            0x01,
            x"000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756e696f6e31793035653070326a6376686a7a66376b63717372717839336434673375393368633268796b6171386872766b717270356c7472737361677a7964"
        );
        
        let instr = zkgm::instruction::new(
            INSTR_VERSION_2,
            OP_TOKEN_ORDER,
            order.encode()
        );

        let salt = b"SALT";
        let mut send_ctx = begin_send(1, salt);
        send_ctx = send_with_coin<sui::sui::SUI>(
            &mut zkgm,
            &mut owned_vault,
            &mut ibc,
            sui_coin,
            INSTR_VERSION_2,
            OP_TOKEN_ORDER,
            order.encode(),
            send_ctx,
            t.ctx()
        );


        let clk_ref = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk_ref) * 1_000_000;
        let timeout_ns = now_ns + 1_000_000_000;
        zkgm::zkgm::end_send(
            &mut ibc,
            &clk_ref,
            0,
            timeout_ns,
            send_ctx,
            t.ctx()
        );

        
        let key = string::from_ascii(type_name::with_defining_ids<sui::sui::SUI>().into_string());
        assert!(zkgm.object_store.contains(key), 1);
        let bag_coin: &Coin<sui::sui::SUI> = zkgm.object_store.borrow(key);
        assert!(coin::value(bag_coin) == amount, 2);

        let pair = ChannelBalancePair {
            channel: 1,
            path: 0,
            token: base_token,
            metadata_image: quote_token,
        };
        assert!(zkgm.channel_balance.contains(pair), 3);
        let tracked: u256 = *zkgm.channel_balance.borrow(pair);
        assert!(tracked == (amount as u256), 4);
        
        end_test(
            clk_ref,
            ibc,
            zkgm,
            owned_vault,
            escrow_vault,
        );

        t.end();
    }

    #[test]
    fun test_recv_flow_solve_sui_single_tx() {
        use sui::test_scenario;
        use sui::clock;

        let mut t = test_scenario::begin(@0x0);

        let (mut ibc, mut zkgm, mut owned_vault, mut escrow_vault) = prepare_test_ctx(&mut t);
        let zkgm_cap = test_scenario::take_from_sender<escrow_vault::ZkgmCap>(&t);

        let base_token = b"SUI";

        let coin = coin::mint_for_testing<sui::sui::SUI>(100_000, t.ctx());        

        escrow_vault.set_fungible_counterparty<sui::sui::SUI>(
            0,
            1,
            base_token,
            b"beneficiary",
            t.ctx(),
        );

        escrow_vault.escrow<sui::sui::SUI>(
            &zkgm_cap,
            coin
        );

        zkgm.register_escrow_vault_cap(zkgm_cap);

        let amount: u64 = 1_000;
        let quote_token = type_name::with_defining_ids<sui::sui::SUI>().into_string().into_bytes();

        let sender = t.sender();  
        let receiver = t.sender(); 

        let solver_md = solver_metadata::new(
            ESCROW_VAULT_OBJECT_KEY,
            x"",
        );
        let solver_md = solver_md.encode();

        let order = zkgm::token_order::new(
            bcs::to_bytes(&sender),     
            bcs::to_bytes(&receiver), 
            base_token,                
            (amount as u256),          
            quote_token,                
            (amount as u256),          
            0x03,  
            solver_md                     
        );

        let instr = zkgm::instruction::new(
            zkgm::zkgm::INSTR_VERSION_2,
            zkgm::zkgm::OP_TOKEN_ORDER,
            order.encode()
        );

        let salt = b"SALT_SOLVE";
        let path: u256 = 0; 
        let packet_bytes = zkgm::zkgm_packet::new(
            salt,
            path,
            instr
        ).encode();

        let clk_ref = t.take_shared<Clock>();
        let mut rctx = begin_recv(
            vector[1],
            vector[1],
            vector[packet_bytes],
            vector[0],
            vector[clock::timestamp_ms(&clk_ref) * 1_000_000 + 1_000_000_000]
        );

        let relayer: address = @0xCAFE;
        let relayer_msg = b"MM-proof-or-data";

        rctx = recv_packet<sui::sui::SUI>(
            &mut ibc,
            &mut zkgm,
            &mut owned_vault,
            &mut escrow_vault,
            &clk_ref,
            relayer,
            relayer_msg,
            rctx,
            t.ctx()
        );

        end_recv(
            &mut ibc,
            &clk_ref,
            b"proof",
            1,           
            relayer,
            relayer_msg,
            rctx
        );

        end_test(
            clk_ref,
            ibc,
            zkgm,
            owned_vault,
            escrow_vault,
        );
        t.end();
    }

    #[test_only]
    fun prepare_test_ctx(t: &mut test_scenario::Scenario): (ibc::IBCStore, RelayStore, OwnedVault, EscrowVault) {
        use std::ascii;
        use sui::test_scenario;
        use sui::clock;

        init_for_tests(t.ctx());
        ibc::init_for_tests(t.ctx());
        owned_vault::init_for_tests(t.ctx());
        escrow_vault::init_for_tests(t.ctx());


        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc = t.take_shared<ibc::IBCStore>();
        let mut zkgm = t.take_shared<zkgm::zkgm::RelayStore>();
        let mut owned_vault = t.take_shared<owned_vault::OwnedVault>();
        let mut escrow_vault = t.take_shared<escrow_vault::EscrowVault>();

        // Open client/connection/channel like in send test
        let mut clk = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk, 1_000);
        clock::share_for_testing(clk);

        ibc.create_client(
            string::utf8(b"cometbls"),
            b"cs",
            b"cons",
            t.ctx()
        );
        ibc.connection_open_init(1, 2);
        ibc.connection_open_ack(1, 9, b"p", 1);

        let mut port = type_name::with_defining_ids<zkgm::zkgm::IbcAppWitness>().into_string();
        port.append(ascii::string(b"::any"));
        ibc.channel_open_init(
            port.to_string(),
            b"cp-port",
            1,
            string::utf8(b"ucs03-zkgm-0"),
            zkgm::zkgm::IbcAppWitness {}
        );
        ibc.channel_open_ack(
            1,
            string::utf8(b"ucs03-zkgm-0"),
            1,
            b"p",
            1,
            zkgm::zkgm::IbcAppWitness {}
        );

        (ibc, zkgm, owned_vault, escrow_vault) 
    }

    #[test_only]
    fun end_test<A: key, B: key, C: key, D: key, E: key>(a: A, b: B, c: C, d: D, e: E) {
        test_scenario::return_shared(a);
        test_scenario::return_shared(b);
        test_scenario::return_shared(c);
        test_scenario::return_shared(d);
        test_scenario::return_shared(e);
    }
    
}
