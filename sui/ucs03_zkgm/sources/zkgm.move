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

module zkgm::zkgm_relay {
    use std::string::{Self, String};
    use std::type_name::{Self};

    use sui::balance::{Self};
    use sui::bcs;
    use sui::clock::Clock; 
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};
    use sui::hash;
    use sui::object_bag::{Self, ObjectBag};
    use sui::table::{Self, Table};

    use ibc::commitment;
    use ibc::ibc;
    use ibc::packet::{Self, Packet};

    use zkgm::ack;
    use zkgm::batch::{Self, Batch};
    use zkgm::batch_ack;
    use zkgm::forward::{Self, Forward};
    use zkgm::fungible_asset_metadata::{Self, FungibleAssetMetadata};
    use zkgm::fungible_asset_order::{Self, FungibleAssetOrder};
    use zkgm::fungible_asset_order_ack;
    use zkgm::helper;
    use zkgm::instruction::{Self, Instruction};
    use zkgm::sui_fungible_asset_metadata;
    use zkgm::zkgm_packet;

    // Constants
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const ACK_SUCCESS: u256 = 1;

    const INSTR_VERSION_0: u8 = 0x00;
    #[allow(unused_const)]
    const INSTR_VERSION_1: u8 = 0x01;
    const INSTR_VERSION_2: u8 = 0x02;

    const OP_FORWARD: u8 = 0x00;
    const OP_MULTIPLEX: u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;
    // const OP_STAKE: u8 = 0x04;
    // const OP_UNSTAKE: u8 = 0x05;
    // const OP_WITHDRAW_STAKE: u8 = 0x06;
    // const OP_WITHDRAW_REWARDS: u8 = 0x07;

    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";
    
    const FUNGIBLE_ASSET_METADATA_TYPE_IMAGE: u8 = 0x00;
    const FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE: u8 = 0x01;
    const FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP: u8 = 0x02;

    public struct TreasuryCapWithMetadata<phantom T> has key, store {
        id: UID,
        cap: TreasuryCap<T>,
        name: String,
        symbol: String,
        decimals: u8,
        icon_url: Option<String>,
        description: String,
        owner: address
    }

    // Errors
    const ACK_ERR_ONLYMAKER: vector<u8> = b"DEADC0DE";
    const E_INVALID_IBC_VERSION: u64 = 3;
    const E_INFINITE_GAME: u64 = 4;
    const E_UNSUPPORTED_VERSION: u64 = 5;
    const E_UNKNOWN_SYSCALL: u64 = 6;
    const E_INVALID_AMOUNT: u64 = 10;
    const E_INVALID_FILL_TYPE: u64 = 12;
    const E_ONLY_MAKER: u64 = 15;
    const E_NO_MULTIPLEX_OPERATION: u64 = 17;
    const E_INVALID_FORWARD_INSTRUCTION: u64 = 18;
    const E_NO_TREASURY_CAPABILITY: u64 = 20;
    const E_INVALID_BASE_AMOUNT: u64 = 22;
    const E_NO_COIN_IN_BAG: u64 = 23;
    const E_CHANNEL_BALANCE_PAIR_NOT_FOUND: u64 = 25;
    const E_ANOTHER_TOKEN_IS_REGISTERED: u64 = 26;
    const E_INVALID_BATCH_INSTRUCTION: u64 = 27;
    const E_BATCH_MUST_BE_SYNC: u64 = 28;
    const E_ACK_AND_PACKET_LENGTH_MISMATCH: u64 = 29;
    const E_INVALID_FORWARD_DESTINATION_CHANNEL_ID: u64 = 30;
    const E_INVALID_METADATA_TYPE: u64 = 31;
    const E_UNWRAP_BASE_AMOUNT_SMALLER_THAN_QUOTE_AMOUNT: u64 = 32;
    const E_UNWRAP_METADATA_INVALID: u64 = 33;
    const E_UNAUTHORIZED: u64 = 34;
    const E_INVALID_METADATA: u64 = 35;
    const E_ONLY_ONE_SESSION_IS_ALLOWED: u64 = 36;
    const E_ALL_INSTRUCTIONS_ARE_RUN: u64 = 37;
    const E_ACK_SIZE_MISMATCHING: u64 = 38;
    const E_EXECUTION_NOT_COMPLETE: u64 = 39;
    const E_INVALID_PACKET_HASH: u64 = 40;

    public struct IbcAppWitness has drop {}

    public struct RelayStore has key {
        id: UID,
        in_flight_packet: Table<vector<u8>, Packet>,
        channel_balance: Table<ChannelBalancePair, u256>,
        token_origin: Table<vector<u8>, u256>,
        type_name_t_to_capability: ObjectBag,
        bag_to_coin: ObjectBag,
        wrapped_denom_to_t: Table<vector<u8>, String>,
        session: Option<ExecutionCtx>,
    }

    public struct ChannelBalancePair has copy, drop, store {
        channel: u32,
        path: u256,
        token: vector<u8>,
        metadata_image: vector<u8>,
    }

    public struct Session {}

    public struct ExecutionCtx has store, drop {
        instruction_set: vector<vector<Instruction>>,
        // not by instruction but by set
        cursor: u64,
        acks: vector<vector<u8>>,
        packet_hash: vector<u8>,
        path: u256,
        salt: vector<u8>,
        ibc_packet: Packet,
    }

    fun init(ctx: &mut TxContext) {
        let id = object::new(ctx);

        transfer::share_object(RelayStore {
            id: id,
            in_flight_packet: table::new(ctx),
            channel_balance: table::new(ctx),
            token_origin: table::new(ctx),
            type_name_t_to_capability: object_bag::new(ctx),
            bag_to_coin: object_bag::new(ctx),
            wrapped_denom_to_t: table::new(ctx),
            session: option::none(),
        });
    }

    public entry fun channel_open_init(
        ibc_store: &mut ibc::IBCStore,
        port_id: String,
        counterparty_port_id: vector<u8>,
        connection_id: u32, 
        version: String
    ) {
        ibc::channel_open_init(
            ibc_store,
            port_id,
            counterparty_port_id,
            connection_id,
            version,
            IbcAppWitness {}
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_IBC_VERSION
        };
    }

    public entry fun channel_open_try(
        ibc_store: &mut ibc::IBCStore,
        port_id: String,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_IBC_VERSION
        };

        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_IBC_VERSION
        };

        ibc::channel_open_try(
            ibc_store,
            port_id,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height,
            IbcAppWitness {}
        );
    }

    public entry fun channel_open_ack(
        ibc_store: &mut ibc::IBCStore,
        port_id: String,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        // Store the channel_id
        ibc::channel_open_ack(
            ibc_store,
            port_id,
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height,
            IbcAppWitness {}
        );
        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_IBC_VERSION
        };
    }

    public entry fun channel_open_confirm(
        ibc_store: &mut ibc::IBCStore,
        channel_id: u32, 
        proof_ack: vector<u8>, 
        proof_height: u64
    )  {
        ibc::channel_open_confirm(
            ibc_store,
            channel_id,
            proof_ack,
            proof_height,
            IbcAppWitness {}
        );
    }

    public entry fun send_with_coin<T>(
        zkgm: &mut RelayStore,
        ibc_store: &mut ibc::IBCStore,
        clock: &Clock,
        coin: Coin<T>,
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>,
        version: u8,
        opcode: u8,
        operand: vector<u8>,
        ctx: &mut TxContext
    ) {
        let instruction = instruction::new(version, opcode, operand);
        let sender = ctx.sender();
        let coin = zkgm.verify_internal<T>(
            ibc_store,
            option::some(coin),
            sender,
            channel_id,
            0,
            instruction,
            ctx
        );
        // This guarantees that the coin is used by some instruction
        coin.destroy_none();

        let zkgm_pack = zkgm_packet::new(salt, 0, instruction);
        ibc_store.send_packet(
            clock,
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack),
            IbcAppWitness {},
            ctx
        );
    }

    /// When receiving a packet, the relayers **must** call this to begin
    /// a receiving session. Receiving is done in multi-steps.
    public fun begin_recv(
        zkgm: &mut RelayStore,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
    ): Session {
        if (zkgm.session.is_some()) {
            abort E_ONLY_ONE_SESSION_IS_ALLOWED
        };

        let zkgm_packet = zkgm_packet::decode(&packet_data);

        let ibc_packet =
            packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );
        let packet_hash = commitment::commit_packet(&ibc_packet);

        zkgm.session = option::some(ExecutionCtx {
            instruction_set: partition_instructions(instruction::decode(&packet_data, &mut 0)),
            cursor: 0,
            acks: vector::empty(),
            packet_hash,
            path: zkgm_packet.path(),
            salt: zkgm_packet.salt(),
            ibc_packet
        });

        Session {}
    }

    /// Partitions the instructions such that all the parts can at most have a single
    /// instruction that requires a generic type T. This drastically reduces the cost of
    /// having one `moveCall` per instruction within the PTB.
    ///
    /// Some examples:
    /// 1. Batch [ Forward, FAO<T>, Forward ] -> [ Forward, FAO<T>, Forward ]
    /// 2. Batch [ Forward, FAO<T>, Forward, FAO<T>, Forward ] -> [ Forward, FAO<T>, Forward ], [ FAO<T>, Forward ]
    ///
    /// *NOTE*: We also do not allow nested batching since it greatly complicates the
    /// execution for absolutely no benefit.
    #[allow(implicit_const_copy)]
    fun partition_instructions(instruction: Instruction): vector<vector<Instruction>> {
        if (instruction.opcode() == OP_BATCH) {
            if (instruction.version() != INSTR_VERSION_0) {
                abort E_UNSUPPORTED_VERSION
            };

            let instructions: vector<Instruction> = *(batch::decode(instruction.operand()).instructions());

            let mut seen_t = false;
            let mut i = 0;
            let mut partitions = vector::empty();
            let mut temp_instrs = vector::empty(); 
            while (i == instructions.length()) {
                if (!helper::is_allowed_batch_instruction(instructions[i].opcode())) {
                    abort E_INVALID_BATCH_INSTRUCTION
                } else if (instructions[i].opcode() == OP_FUNGIBLE_ASSET_ORDER) {
                    // if we already hit an instr with T, it's time to partition
                    if (seen_t == true) {
                        partitions.push_back(temp_instrs);
                        temp_instrs = vector::empty(); 
                    } else {
                        seen_t = true;
                    };
                };

                temp_instrs.push_back(instructions[i]);

                i = i + 1;
            };

            partitions
        } else {
            vector[vector[instruction]]
        }
    }

    public entry fun recv_packet_2<T>(
        ibc: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        clock: &Clock,
        relayer: address,
        relayer_msg: vector<u8>,
        ctx: &mut TxContext
    ) {
        // aborts if there is not a session
        let instr_length = {
            let exec_ctx = zkgm.session.borrow();
            assert!(exec_ctx.cursor + 1 <= exec_ctx.instruction_set.length(), E_ALL_INSTRUCTIONS_ARE_RUN);
            exec_ctx.instruction_set[exec_ctx.cursor].length()
        };

        let mut acks = vector::empty();

        let mut i = 0;
        while (i < instr_length) {
            let (ack, err) = zkgm.execute_internal_2<T>(
                ibc,
                i,
                clock,
                relayer,
                relayer_msg,
                false,
                ctx
            );
            assert!(err == 0, err);
            zkgm.session.borrow_mut().acks.push_back(ack);
            acks.push_back(ack);

            i = i + 1;
        };

        zkgm.session.borrow_mut().cursor = zkgm.session.borrow().cursor + 1;
    }

    public fun end_recv(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        clock: &Clock,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        relayer_msg: vector<u8>,
        session: Session,
    ) {
        let Session { .. } = session;

        let exec_ctx = zkgm.session.extract();

        let ibc_packet =
            packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        let packet_hash = commitment::commit_packet(&ibc_packet);

        assert!(exec_ctx.packet_hash == packet_hash, E_INVALID_PACKET_HASH);
        assert!(exec_ctx.cursor == exec_ctx.instruction_set.length(), E_EXECUTION_NOT_COMPLETE);

        let ack = if (exec_ctx.instruction_set[0].length() == 1 && exec_ctx.instruction_set.length() == 1) {
            assert!(exec_ctx.acks.length() == 1, E_ACK_SIZE_MISMATCHING);

            exec_ctx.acks[0]
        } else {
            batch_ack::new(exec_ctx.acks).encode()
        };

        ibc.recv_packet(
            clock,
            vector[ibc_packet],
            relayer,
            vector[relayer_msg],
            proof,
            proof_height,
            vector[ack],
            IbcAppWitness {}
        );
    }

    

    fun execute_internal_2<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        instr_idx: u64,
        clock: &Clock,
        relayer: address,
        relayer_msg: vector<u8>,
        intent: bool,
        ctx: &mut TxContext
    ): (vector<u8>, u64)  {
        let exec_ctx = zkgm.session.borrow();

        let instruction = exec_ctx.instruction_set[exec_ctx.cursor][instr_idx];
        let version = instruction.version();

        match (instruction.opcode()) {
            OP_FUNGIBLE_ASSET_ORDER => {
                if (version != INSTR_VERSION_2) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };
                zkgm.execute_fungible_asset_order<T>(
                    exec_ctx.ibc_packet,
                    relayer,
                    relayer_msg,
                    exec_ctx.path,
                    fungible_asset_order::decode(instruction.operand()),
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
                    exec_ctx.ibc_packet,
                    relayer,
                    relayer_msg,
                    exec_ctx.salt,
                    exec_ctx.path,
                    instruction.version(),
                    forward::decode(instruction.operand()),
                    intent,
                    ctx,
                )
            },
            OP_MULTIPLEX => (vector::empty(), E_NO_MULTIPLEX_OPERATION),
            _ => (vector::empty(), E_UNKNOWN_SYSCALL)
        }
    }



    public entry fun recv_packet<T>(
        ibc_store: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        clock: &Clock,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_data: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        relayer_msgs: vector<vector<u8>>,
        ctx: &mut TxContext
    ) {
        let mut packets = vector::empty();
        let mut acks = vector::empty();
        let mut i = 0;
        while (i < packet_source_channels.length()) {
            packets.push_back(packet::new(
                packet_source_channels[i],
                packet_destination_channels[i],
                packet_data[i],
                packet_timeout_heights[i],
                packet_timeout_timestamps[i],
            ));
            acks.push_back(zkgm.process_receive<T>(
                ibc_store,
                clock,              
                packets[i],
                relayer,
                relayer_msgs[i],
                false,
                ctx
            ));
            i = i + 1;
        };

        ibc::recv_packet(
            ibc_store,
            clock,
            packets,
            relayer,
            relayer_msgs,
            proof,
            proof_height,
            acks,
            IbcAppWitness {}
        );
    }

    public entry fun acknowledge_packet<T>(
        ibc: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_data: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        relayer: address,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &mut TxContext
    ) {
        assert!(acknowledgements.length() == packet_source_channels.length(), E_ACK_AND_PACKET_LENGTH_MISMATCH);

        let mut packets: vector<Packet> = vector::empty();
        let mut i = 0;
        while (i < packet_source_channels.length()) {
            packets.push_back(packet::new(
                packet_source_channels[i],
                packet_destination_channels[i],
                packet_data[i],
                packet_timeout_heights[i],
                packet_timeout_timestamps[i],
            ));
            zkgm.on_acknowledge_packet<T>(
                ibc,
                packets[i],
                acknowledgements[i],
                relayer,
                ctx
            );
            i = i + 1;
        };
        
        ibc.acknowledge_packet(
            packets,
            acknowledgements,
            proof,
            proof_height,
            relayer,
            IbcAppWitness {}
        );
    }

    public entry fun timeout_packet<T>(
        ibc_store: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        ctx: &mut TxContext
    ) {
        let ibc_packet =
            packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        let zkgm_packet = zkgm_packet::decode(ibc_packet.data());

        if (helper::is_forwarded_packet(zkgm_packet.salt())) {
            let packet_hash = commitment::commit_packet(&ibc_packet)  ;
            if (zkgm.in_flight_packet.contains(packet_hash)) {
                let parent = zkgm.in_flight_packet.remove(packet_hash);
                ibc_store.write_acknowledgement(
                    parent,
                    ack::failure(ACK_EMPTY).encode(),
                    IbcAppWitness {}
                );
                return
            };
        };

        ibc_store.timeout_packet(
            ibc_packet,
            proof,
            proof_height
        );

        zkgm.timeout_internal<T>(
            ibc_packet,
            relayer,
            zkgm_packet.path(),
            zkgm_packet.instruction(),
            ctx,
        );
    }

    public entry fun channel_close_init(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public entry fun channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public entry fun register_capability<T>(
        zkgm: &mut RelayStore,
        mut capability: TreasuryCap<T>,
        metadata: &CoinMetadata<T>,
        owner: address,
        ctx: &mut TxContext,
    ) {
        if (owner != @0x0) {
            assert!(ctx.sender() == owner, E_UNAUTHORIZED);         
        };
    
        let supply = coin::supply(&mut capability);
        if (balance::supply_value(supply) != 0 ) {
            abort 0
        };

        let typename_t = type_name::get<T>();
        zkgm.type_name_t_to_capability.add(string::from_ascii(typename_t.into_string()), TreasuryCapWithMetadata {
            id: object::new(ctx),
            cap: capability,
            name: coin::get_name(metadata),
            symbol: string::from_ascii(coin::get_symbol(metadata)),
            decimals: coin::get_decimals(metadata),
            icon_url: coin::get_icon_url(metadata).map!(|url| string::utf8(url.inner_url().into_bytes())),
            description: coin::get_description(metadata),
            owner
        });
    }

    fun process_receive<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        clock: &Clock,
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        intent: bool,
        ctx: &mut TxContext
    ): vector<u8> {
        let raw_zkgm_packet = packet.data();
        let zkgm_packet = zkgm_packet::decode(raw_zkgm_packet);

        let (ack, err) =
            zkgm.execute_internal<T>(
                ibc,
                clock,
                packet,
                relayer,
                relayer_msg,
                zkgm_packet.salt(),
                zkgm_packet.path(),
                zkgm_packet.instruction(),
                intent,
                ctx
            );

        if (err == 0) {
            if (ack.is_empty()) {
                return vector::empty()
            };

            // Special case where we should avoid the packet from being
            // received entirely as it is only fillable by a market maker.
            if (ack == ACK_ERR_ONLYMAKER) {
                abort E_ONLY_MAKER
            };

            ack::success(ack).encode()
        } else {
            ack::failure(bcs::to_bytes(&err)).encode()
        }
    }

    fun execute_internal<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        clock: &Clock,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        instruction: Instruction,
        intent: bool,
        ctx: &mut TxContext
    ): (vector<u8>, u64)  {
        let version = instruction.version();

        match (instruction.opcode()) {
            OP_FUNGIBLE_ASSET_ORDER => {
                if (version != INSTR_VERSION_2) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };
                zkgm.execute_fungible_asset_order<T>(
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    path,
                    fungible_asset_order::decode(instruction.operand()),
                    intent,
                    ctx
                )
            },
            OP_BATCH => {
                if (version != INSTR_VERSION_0) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };

                zkgm.execute_batch<T>(
                    ibc,
                    clock,
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    salt,
                    path,
                    batch::decode(instruction.operand()),
                    intent,
                    ctx,                    
                )
            },
            OP_FORWARD => {
                if (version != INSTR_VERSION_0) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };

                zkgm.execute_forward(
                    ibc,
                    clock,
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    salt,
                    path,
                    instruction.version(),
                    forward::decode(instruction.operand()),
                    intent,
                    ctx,
                )
            },
            OP_MULTIPLEX => (vector::empty(), E_NO_MULTIPLEX_OPERATION),
            _ => (vector::empty(), E_UNKNOWN_SYSCALL)
        }
    }

    

    fun market_maker_fill<T>(
        zkgm: &mut RelayStore,
        relayer_msg: vector<u8>,
        _quote_token: vector<u8>,
        receiver: address,
        quote_amount: u64,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        if (quote_amount != 0){
            // TODO(aeryz): handle NATIVE_TOKEN_ERC_7528_ADDRESS case            
            // TODO(aeryz): make sure that distribute here is correct
            zkgm.distribute_coin<T>(receiver, quote_amount, ctx);
        };
        (fungible_asset_order_ack::new(
            FILL_TYPE_MARKETMAKER,
            relayer_msg
        ).encode(), 0)
    }

    fun compute_salt(path: u256, channel: u32, base_token: vector<u8>, metadata: vector<u8>): vector<u8> {
        let mut data: vector<u8> = bcs::to_bytes(&path);
        data.append(bcs::to_bytes(&channel));
        data.append(base_token);
        data.append(hash::keccak256(&metadata));

        hash::keccak256(&data)
    }

    fun compute_salt_from_metadata_image(path: u256, channel: u32, base_token: vector<u8>, metadata_image: vector<u8>): vector<u8> {
        let mut data: vector<u8> = bcs::to_bytes(&path);
        data.append(bcs::to_bytes(&channel));
        data.append(base_token);
        data.append(metadata_image);

        hash::keccak256(&data)
    }

    fun distribute_coin<T>(
        relay_store: &mut RelayStore,
        receiver: address,
        amount: u64,
        ctx: &mut TxContext
    ) {
        let typename_t = type_name::get<T>();
        let key = typename_t.into_string();
        if(!relay_store.bag_to_coin.contains(string::from_ascii(key))) {
            abort E_NO_COIN_IN_BAG
        };
        let coin = relay_store.bag_to_coin.borrow_mut<String, Coin<T>>(string::from_ascii(key));

        let transferred_coin = coin.split<T>(amount, ctx);
        transfer::public_transfer(transferred_coin, receiver);
    }

    fun protocol_fill_mint<T>(
        zkgm: &mut RelayStore,
        _channel_id: u32,
        _path: u256,
        wrapped_token: vector<u8>,
        receiver: address,
        relayer: address,
        order: &FungibleAssetOrder,
        metadata: Option<FungibleAssetMetadata>,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let quote_amount = order.quote_amount();

        let fee = (order.base_amount() - order.quote_amount()) as u64;
        // if this token is minted for the first time, then we need to ensure that its always minting the same T
        if (!zkgm.claim_wrapped_denom<T>(wrapped_token, metadata)) {
            return (vector::empty(), E_ANOTHER_TOKEN_IS_REGISTERED)
        };
        let capability = zkgm.get_treasury_cap<T>();
        if (quote_amount > 0) {
            coin::mint_and_transfer<T>(capability, quote_amount as u64, receiver, ctx);
        };
        if (fee > 0){
            coin::mint_and_transfer<T>(capability, fee, relayer, ctx);
        };

        (fungible_asset_order_ack::new(
            FILL_TYPE_PROTOCOL,
            ACK_EMPTY
        ).encode(), 0)
    }

    fun protocol_fill_unescrow<T>(
        zkgm: &mut RelayStore,
        channel_id: u32,
        path: u256,
        quote_token: vector<u8>,
        metadata_image: vector<u8>,
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
            metadata_image,
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

        (fungible_asset_order_ack::new(
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
            IbcAppWitness {},
            ctx
        );

        // Guaranteed to be unique by the above sendPacket
        let commitment_key = commitment::batch_packets_commitment_key(
            commitment::commit_packet(&sent_packet)
        );
        zkgm.in_flight_packet.add(commitment_key, ibc_packet);

        (ACK_EMPTY, 0)
    }

    fun execute_batch<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        clock: &Clock,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        batch: Batch,
        intent: bool,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let l = batch.instructions().length();
        let mut acks = vector::empty();

        let mut i = 0;
        while (i < l) {
            let instruction = batch.instructions()[i];  
            if (!helper::is_allowed_batch_instruction(instruction.opcode())) {
                return (vector::empty(), E_INVALID_BATCH_INSTRUCTION)
            };

            let (ack, err) = zkgm.execute_internal<T>(
                ibc,
                clock,
                ibc_packet,
                relayer,
                relayer_msg,
                helper::derive_batch_salt(i, salt),
                path,
                instruction,
                intent,
                ctx
            );

            if (err != 0) {
                return (vector::empty(), err)
            };

            if (ack.is_empty()) {
                return (vector::empty(), E_BATCH_MUST_BE_SYNC)
            } else if (ack == ACK_ERR_ONLYMAKER){
                return (ack, 0)
            };

            acks.push_back(ack);

            i = i + 1;
        };

        (batch_ack::new(acks).encode(), 0)
    }


    fun execute_fungible_asset_order<T>(
        zkgm: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        path: u256,
        order: FungibleAssetOrder,
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
        if (intent) {
            return zkgm.market_maker_fill<T>(
                relayer_msg,
                quote_token,
                receiver,
                order.quote_amount() as u64,
                ctx,
            )
        };

        let base_amount_covers_quote_amount = order.base_amount() >= order.quote_amount();

        let (wrapped_token, metadata) = match (order.metadata_type()) {
            FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE => {
                let wrapped_token = compute_salt(
                    path,
                    ibc_packet.destination_channel_id(),
                    *order.base_token(),
                    *order.metadata()
                );
                let metadata = fungible_asset_metadata::decode(order.metadata());
                (wrapped_token, option::some(metadata))
                
            },
            FUNGIBLE_ASSET_METADATA_TYPE_IMAGE => {
                // we expect the metadata to be a 32-byte hash
                if (order.metadata().length() != 32) {
                    return (vector::empty(), E_UNWRAP_METADATA_INVALID)
                };
                let wrapped_token = compute_salt_from_metadata_image(
                    path,
                    ibc_packet.destination_channel_id(),
                    *order.base_token(),
                    *order.metadata()
                );
                (wrapped_token, option::none())
            },
            FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP => {
                if (!base_amount_covers_quote_amount) {
                    return (vector::empty(), E_UNWRAP_BASE_AMOUNT_SMALLER_THAN_QUOTE_AMOUNT)   
                };

                // we expect the metadata to be a 32-byte hash
                if (order.metadata().length() != 32) {
                    return (vector::empty(), E_UNWRAP_METADATA_INVALID)
                };

                // TODO: add rate limit here later
                return zkgm.protocol_fill_unescrow<T>(
                    ibc_packet.destination_channel_id(), 
                    path, 
                    quote_token, 
                    *order.metadata(),
                    receiver, 
                    relayer, 
                    order.base_amount() as u64, 
                    order.quote_amount() as u64, 
                    ctx
                )
            },
            _ => return (vector::empty(), E_INVALID_METADATA_TYPE)
        };

        if (quote_token == wrapped_token && base_amount_covers_quote_amount) {
            // TODO: rate limit
            zkgm.save_token_origin(wrapped_token, path, ibc_packet.destination_channel_id());
            
            // We expect the token to be deployed already here and the treasury cap is registered previously with type T
            zkgm.protocol_fill_mint<T>(
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
                relayer_msg, 
                quote_token, 
                receiver, 
                order.quote_amount() as u64,
                ctx
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
            OP_FUNGIBLE_ASSET_ORDER => {
                if (version == INSTR_VERSION_2) {
                    zkgm.verify_fungible_asset_order<T>(
                        coin.destroy_some(),
                        sender,
                        channel_id,
                        path,
                        fungible_asset_order::decode(instruction.operand()),
                        ctx
                    );
                    option::none<Coin<T>>()
                } else {
                    abort E_UNSUPPORTED_VERSION
                }
            },
            OP_BATCH => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.verify_batch<T>(
                    ibc_store,
                    coin,
                    sender,
                    channel_id,
                    path,
                    batch::decode(instruction.operand()),
                    ctx,
                )
            },
            OP_FORWARD => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.verify_forward<T>(
                    ibc_store,
                    coin,
                    sender,
                    channel_id,
                    forward::decode(instruction.operand()),
                    ctx,
                )
            },
            OP_MULTIPLEX => abort E_NO_MULTIPLEX_OPERATION,
            _ => abort E_UNKNOWN_SYSCALL,
        }
    }

    fun verify_fungible_asset_order<T>(
        zkgm: &mut RelayStore,
        coin: Coin<T>,
        _sender: address,
        channel_id: u32,
        path: u256,
        order: FungibleAssetOrder,
        _ctx: &mut TxContext
    ) {
        let base_token = *order.base_token();
        let base_amount = order.base_amount();

        if(coin.balance().value() != base_amount as u64){
            abort E_INVALID_BASE_AMOUNT
        };
        
        let mut origin = 0;    
        if (zkgm.token_origin.contains(base_token)) {
            origin = *zkgm.token_origin.borrow(base_token);
        };

        let (intermediate_channel_path, destination_channel_id) =
            helper::pop_channel_from_path(origin);

        let metadata_image = match (order.metadata_type()) {
            FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP | FUNGIBLE_ASSET_METADATA_TYPE_IMAGE => {
                assert!(order.metadata().length() == 32, E_INVALID_METADATA);
                *order.metadata()
            },
            FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE => hash::keccak256(order.metadata()),
            _ => abort E_INVALID_METADATA_TYPE
        };

        let wrapped_token = compute_salt_from_metadata_image(
            intermediate_channel_path,
            channel_id,
            *order.quote_token(),
            metadata_image
        );

        let is_inverse_intermediate_path = path == helper::reverse_channel_path(intermediate_channel_path);
        let is_sending_back_to_same_channel = destination_channel_id == channel_id;
        let is_unwrapping = base_token == wrapped_token;
        if (is_inverse_intermediate_path
            && is_sending_back_to_same_channel
            && is_unwrapping) {

            // Ensure we specificy that we unwrap in the metadata tag.
            assert!(order.metadata_type() == FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP, E_INVALID_METADATA_TYPE);
            // We don't have to verify that metadataImage matches the stored one
            // because the prediction would fail otherwise and we would fall
            // back in the else branch.
            let capability = zkgm.get_treasury_cap<T>();
            coin::burn<T>(capability, coin);
        } else {
            // IMAGE_UNWRAP is not allowed
            assert!(order.metadata_type() != FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP, E_INVALID_METADATA_TYPE);

            zkgm.increase_outstanding(
                channel_id, 
                path, 
                base_token,
                metadata_image,
                order.base_amount()
            );
            zkgm.save_coin_to_bag<T>(coin);

            // TODO(aeryz): handle gas station here
        }   

    }

    fun verify_forward<T>(
        zkgm: &mut RelayStore,
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
            ibc_store,
            coin,
            sender,
            channel_id,
            forward_packet.path(),
            *instruction,
            ctx
        )
    }

    fun verify_batch<T>(
        zkgm: &mut RelayStore,
        ibc_store: &mut ibc::IBCStore,
        mut coin: Option<Coin<T>>,
        sender: address,
        channel_id: u32,
        path: u256,
        batch: Batch,
        ctx: &mut TxContext
    ): Option<Coin<T>> {
        let l = batch.instructions().length();
        let mut i = 0;
        while (i < l) {
            let instruction = batch.instructions()[i];
            assert!(helper::is_allowed_batch_instruction(instruction.opcode()), E_INVALID_BATCH_INSTRUCTION);

            coin = zkgm.verify_internal<T>(
                ibc_store,
                coin,
                sender,
                channel_id,
                path,
                instruction,
                ctx
            );

            i = i + 1;
        };

        coin
    }

    fun on_acknowledge_packet<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        ibc_packet: Packet,
        ack: vector<u8>,
        relayer: address,
        ctx: &mut TxContext,
    ) {
        let zkgm_packet = zkgm_packet::decode(ibc_packet.data());
        if (helper::is_forwarded_packet(zkgm_packet.salt())) {
            let packet_hash = commitment::commit_packet(&ibc_packet);

            if (zkgm.in_flight_packet.contains(packet_hash)) {
                let parent = zkgm.in_flight_packet.remove(packet_hash);
                ibc.write_acknowledgement(parent, ack, IbcAppWitness {});
                return
            };
        };

        let zkgm_ack = ack::decode(&ack);
        zkgm.acknowledge_internal<T>(
            ibc_packet,
            relayer,
            zkgm_packet.path(),
            zkgm_packet.salt(),
            zkgm_packet.instruction(),
            zkgm_ack.tag() == ACK_SUCCESS,
            *zkgm_ack.inner_ack(),
            ctx
        );
    }

    fun acknowledge_internal<T>(
        zkgm: &mut RelayStore,
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
            OP_FUNGIBLE_ASSET_ORDER => {
                if (version == INSTR_VERSION_2) {
                    let order = fungible_asset_order::decode(instruction.operand());
                    
                    zkgm.acknowledge_fungible_asset_order<T>(
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
            OP_BATCH => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION)  ;
                zkgm.acknowledge_batch<T>(
                    ibc_packet,
                    relayer,
                    path,
                    salt,
                    batch::decode(instruction.operand()),
                    success,
                    inner_ack,
                    ctx
                )
            },
            OP_FORWARD => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.acknowledge_forward<T>(
                    ibc_packet,
                    relayer,
                    salt,
                    forward::decode(instruction.operand()),
                    success,
                    inner_ack,
                    ctx,
                );
            },
            OP_MULTIPLEX => {
                abort E_NO_MULTIPLEX_OPERATION
            },            
            _ => abort E_UNKNOWN_SYSCALL
        };
    }

    fun acknowledge_batch<T>(
        zkgm: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        salt: vector<u8>,
        batch: Batch,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        let l = batch.instructions().length();
        let batch_ack = batch_ack::decode(&ack);

        let mut i = 0;
        while (i < l) {
            let mut syscall_ack = ack;
            if (success) {
                syscall_ack = batch_ack.acknowledgements()[i];
            };

            zkgm.acknowledge_internal<T>(
                ibc_packet,
                relayer,
                path,
                helper::derive_batch_salt(i, salt),
                batch.instructions()[i],
                success,
                syscall_ack,
                ctx
            );

            i = i + 1;
        };
    }

    fun acknowledge_fungible_asset_order<T>(
        zkgm: &mut RelayStore,
        ibc_packet: Packet,
        _relayer: address,
        path: u256,
        _salt: vector<u8>,
        order: FungibleAssetOrder,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        if (success) {
            let asset_order_ack = fungible_asset_order_ack::decode(&ack);
            if (asset_order_ack.fill_type() == FILL_TYPE_PROTOCOL) {
                // The protocol filled, fee was paid to relayer.
            } else if(
                asset_order_ack.fill_type() == FILL_TYPE_MARKETMAKER
            ) {
                let market_maker = bcs::new(*asset_order_ack.market_maker()).peel_address();

                if (order.metadata_type() == FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP) {
                    let capability = zkgm.get_treasury_cap<T>();
                    coin::mint_and_transfer<T>(capability, order.base_amount() as u64, market_maker, ctx);
                } else {
                    let metadata_image = match (order.metadata_type()) {
                        FUNGIBLE_ASSET_METADATA_TYPE_IMAGE => *order.metadata(),
                        FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE => hash::keccak256(order.metadata()),
                        _ => abort E_INVALID_METADATA_TYPE
                    };
                    let res = zkgm.decrease_outstanding(
                        ibc_packet.source_channel_id(), 
                        path, 
                        *order.base_token(), 
                        metadata_image,
                        order.base_amount() as u256
                    );
                    assert!(res == 0, res);
                    zkgm.distribute_coin<T>(market_maker, order.base_amount() as u64, ctx);
                }
                
            } else {
                abort E_INVALID_FILL_TYPE
            };
        } else {
            zkgm.refund<T>(
                ibc_packet.source_channel_id(), 
                path, 
                order,
                ctx
            )
        };
    }


    fun acknowledge_forward<T>(
        zkgm: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        forward_packet: Forward,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        zkgm.acknowledge_internal<T>(
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
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ) {
        let version = instruction::version(&instruction);

        match (instruction.opcode()) {
            OP_FUNGIBLE_ASSET_ORDER => {
                if (version == INSTR_VERSION_2) {
                    zkgm.timeout_fungible_asset_order<T>(
                        ibc_packet,
                        path,
                        fungible_asset_order::decode(instruction.operand()),
                        ctx
                    );
                } else {
                    abort E_UNSUPPORTED_VERSION
                };
                      
            },
            OP_BATCH => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.timeout_batch<T>(
                    ibc_packet,
                    relayer,
                    path,
                    batch::decode(instruction.operand()),
                    ctx
                );
            },
            OP_FORWARD => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
                zkgm.timeout_forward<T>(
                    ibc_packet,
                    relayer,
                    path,
                    forward::decode(instruction.operand()),
                    ctx
                );
            },
            OP_MULTIPLEX => {
                abort E_NO_MULTIPLEX_OPERATION
            },
            _ => abort E_UNKNOWN_SYSCALL
        };
    }

    fun timeout_batch<T>(
        zkgm: &mut RelayStore,
        packet: Packet,
        relayer: address,
        path: u256,
        batch: Batch,
        ctx: &mut TxContext
    ) {
        let l = batch.instructions().length();
        let mut i = 0;
        while (i < l) {
            zkgm.timeout_internal<T>(
                packet,
                relayer,
                path,
                batch.instructions()[i],
                ctx
            );
            i = i + 1;
        }
    }

    fun timeout_fungible_asset_order<T>(
        zkgm: &mut RelayStore,
        packet: Packet,
        path: u256,
        order: FungibleAssetOrder,
        ctx: &mut TxContext
    ) {
        zkgm.refund<T>(
            packet.source_channel_id(), 
            path,
            order,
            ctx
        );
    }

    fun refund<T>(
        zkgm: &mut RelayStore,
        source_channel: u32,
        path: u256,
        order: FungibleAssetOrder,
        ctx: &mut TxContext
    ) {
        let sender = bcs::new(*order.sender()).peel_address();
        let capability = zkgm.get_treasury_cap<T>();

        if (order.metadata_type() == FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP) {
            coin::mint_and_transfer<T>(capability, order.base_amount() as u64, sender, ctx);
        } else {
            let metadata_image = match (order.metadata_type()) {
                FUNGIBLE_ASSET_METADATA_TYPE_IMAGE => *order.metadata(),
                FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE => hash::keccak256(order.metadata()),
                _ => abort E_INVALID_METADATA_TYPE
            };

            
            zkgm.decrease_outstanding(
                source_channel,
                path, 
                *order.base_token(),
                metadata_image,
                order.base_amount() as u256
            );
            
            zkgm.distribute_coin<T>(sender, order.base_amount() as u64, ctx);
        };        
    }

    fun timeout_forward<T>(
        zkgm: &mut RelayStore,
        packet: Packet,
        relayer: address,
        path: u256,
        forward_packet: Forward,
        ctx: &mut TxContext
    ) {
        zkgm.timeout_internal<T>(
            packet, 
            relayer, 
            path, 
            *forward_packet.instruction(), 
            ctx
        )
    }

    fun get_treasury_cap<T>(
        zkgm: &mut RelayStore
    ): &mut TreasuryCap<T> {
        let typename_t = type_name::get<T>();
        let key = string::from_ascii(typename_t.into_string());
        if (!zkgm.type_name_t_to_capability.contains(key)) {
            abort E_NO_TREASURY_CAPABILITY             
        };
        &mut zkgm.type_name_t_to_capability.borrow_mut<String, TreasuryCapWithMetadata<T>>(key).cap
    }

    fun claim_wrapped_denom<T>(
        zkgm: &mut RelayStore,
        wrapped_denom: vector<u8>,
        metadata: Option<FungibleAssetMetadata>,
    ): bool {
        let typename_t = type_name::get<T>();
        let key = string::from_ascii(typename_t.into_string());
        if (!zkgm.wrapped_denom_to_t.contains(wrapped_denom)) {
            if (metadata.is_none()) {
                // Means a hash is provided. We can't do the necessary checks when we don't know the full
                // metadata.
                return false                
            };

            let metadata = metadata.destroy_some();
            let sui_metadata = sui_fungible_asset_metadata::decode(*metadata.initializer());
        
            let t = zkgm.type_name_t_to_capability.borrow<String, TreasuryCapWithMetadata<T>>(key);

            if (t.name != sui_metadata.name()
                || t.symbol != sui_metadata.symbol()
                || t.decimals != sui_metadata.decimals()
                || t.owner != sui_metadata.owner()
                || &t.icon_url != sui_metadata.icon_url()
                || &t.description != sui_metadata.description()) {
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
        let typename_t = type_name::get<T>();
        let key = type_name::into_string(typename_t);
        if(relay_store.bag_to_coin.contains(string::from_ascii(key))) {
            let self_coin = relay_store.bag_to_coin.borrow_mut(string::from_ascii(key));
            coin::join(self_coin, coin)
        } else{
            relay_store.bag_to_coin.add(string::from_ascii(key), coin)
        }
    }

    fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }

    fun save_token_origin(
        zkgm: &mut RelayStore,
        wrapped_token: vector<u8>,
        path: u256,
        channel_id: u32
    ) {
        let updated_channel_path = helper::update_channel_path(path, channel_id);
        if (!zkgm.token_origin.contains(wrapped_token)) {
            zkgm.token_origin.add(wrapped_token, updated_channel_path);
        };
    }
}
