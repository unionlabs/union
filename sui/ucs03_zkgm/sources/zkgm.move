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
    use ibc::ibc;
    use ibc::packet::{Self, Packet};
    use zkgm::instruction::{Self, Instruction};
    use zkgm::zkgm_packet::{Self};
    use zkgm::forward::{Self, Forward};
    use zkgm::fungible_asset_order::{Self, FungibleAssetOrder};
    use zkgm::fungible_asset_order_ack::{Self};
    use zkgm::ack;
    use zkgm::batch::{Self, Batch};
    use zkgm::batch_ack;
    use sui::clock::Clock;
    
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};

    use std::string::{Self, String};
    use sui::table::{Self, Table};
    use ibc::commitment;
    use sui::bcs;
    use sui::clock;
    use sui::event;
    use sui::object_bag::{Self, ObjectBag};
    use std::type_name::{Self};
    use sui::balance::{Self};
    use sui::hash::{Self};

    // Constants
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const ACK_SUCCESS: u256 = 1;
    const ACK_FAILURE: u256 = 0;

    const INSTR_VERSION_0: u8 = 0x00;
    const INSTR_VERSION_1: u8 = 0x01;

    const OP_FORWARD: u8 = 0x00;
    const OP_MULTIPLEX: u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;
    const OP_STAKE: u8 = 0x04;
    const OP_UNSTAKE: u8 = 0x05;
    const OP_WITHDRAW_STAKE: u8 = 0x06;
    const OP_WITHDRAW_REWARDS: u8 = 0x07;

    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";

    const FORWARD_SALT_MAGIC: u256 = 0xC0DE00000000000000000000000000000000000000000000000000000000BABE;


    // Errors
    const ACK_ERR_ONLYMAKER: vector<u8> = b"DEADC0DE";
    const E_INVALID_HOPS: u64 = 2;
    const E_INVALID_IBC_VERSION: u64 = 3;
    const E_INFINITE_GAME: u64 = 4;
    const E_UNSUPPORTED_VERSION: u64 = 5;
    const E_UNKNOWN_SYSCALL: u64 = 6;
    const E_INVALID_ASSET_NAME: u64 = 7;
    const E_INVALID_ASSET_SYMBOL: u64 = 8;
    const E_INVALID_ASSET_ORIGIN: u64 = 9;
    const E_INVALID_AMOUNT: u64 = 10;
    const E_INVALID_FILL_TYPE: u64 = 12;
    const E_ACK_EMPTY: u64 = 14;
    const E_ONLY_MAKER: u64 = 15;
    const E_NO_BATCH_OPERATION: u64 = 16;
    const E_NO_MULTIPLEX_OPERATION: u64 = 17;
    const E_ERR_INVALID_FORWARD_INSTRUCTION: u64 = 18;
    const E_NO_EXECUTE_OPERATION: u64 = 19;
    const E_NO_TREASURY_CAPABILITY: u64 = 20;
    const E_INVALID_ASSET_DECIMAL: u64 = 21;
    const E_INVALID_BASE_AMOUNT: u64 = 22;
    const E_NO_COIN_IN_BAG: u64 = 23;
    const E_CHANNEL_BALANCE_PAIR_NOT_FOUND: u64 = 25;
    const E_ANOTHER_TOKEN_IS_REGISTERED: u64 = 26;
    const E_INVALID_BATCH_INSTRUCTION: u64 = 27;
    const E_BATCH_MUST_BE_SYNC: u64 = 28;
    const E_ACK_AND_PACKET_LENGTH_MISMATCH: u64 = 29;
    const E_NOT_IMPLEMENTED: u64 = 333222111;


    public struct IbcAppWitness has drop {}

    public struct RelayStore has key {
        id: UID,
        in_flight_packet: Table<vector<u8>, Packet>,
        channel_balance: Table<ChannelBalancePair, u256>,
        token_origin: Table<vector<u8>, u256>,
        type_name_t_to_capability: ObjectBag,
        bag_to_coin: ObjectBag,
        wrapped_denom_to_t: Table<vector<u8>, String>
    }

    public struct ChannelBalancePair has copy, drop, store {
        channel: u32,
        path: u256,
        token: vector<u8>
    }

    public fun type_name_contains_capability(
        relay_store: &RelayStore,
        typename_t: string::String
    ): bool {
        let capability = relay_store.type_name_t_to_capability.contains(typename_t);
        return capability
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
            wrapped_denom_to_t: table::new(ctx)
        });
    }

    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }

    public fun update_channel_path(path: u256, next_channel_id: u32): u256 {
        if (path == 0) {
            return (next_channel_id as u256)
        };
        let next_hop_index = ((fls(path) / 32) as u8) + 1;
        if (next_hop_index > 7) {
            abort E_INVALID_HOPS
        };

        let next_channel = (((next_channel_id as u256) << (next_hop_index * 32)) as u256)
            | path;
        (next_channel as u256)
    }

    public fun reverse_channel_path(mut path: u256): u256 {
        let mut reversed_path = 0;
        loop {
            // body always runs once
            let (tail, head) = pop_channel_from_path(path);
            reversed_path = update_channel_path(reversed_path, head);
            path = tail;
            // exit once path == 0
            if (path == 0) {
                break
            }
        };
        reversed_path
    }


    public fun pop_channel_from_path(path: u256) : (u256, u32){
        if (path == 0) {
            return (0, 0)
        };
        let current_hop_index = ((fls(path) / 32)) as u8;
        let clear_shift = ((8-current_hop_index) * 32) as u8;
        return (
            (path << clear_shift) >> clear_shift,
            (path >> (current_hop_index * 32)) as u32
        )

    }

    /// Find last set (most significant bit).
    /// Returns the index of the most significant bit of `x`.
    /// If `x` is zero, returns 256.
    public fun fls(mut x: u256): u256 {
        if (x == 0) {
            return 256
        };

        let mut r: u256 = 0;

        // Check higher 128 bits
        if (x > 0xffffffffffffffffffffffffffffffff) {
            r = 128;
            x = x >> 128;
        };

        // Check higher 64 bits
        if (x > 0xffffffffffffffff) {
            r = r + 64;
            x = x >> 64;
        };

        // Check higher 32 bits
        if (x > 0xffffffff) {
            r = r + 32;
            x = x >> 32;
        };

        // Check higher 16 bits
        if (x > 0xffff) {
            r = r + 16;
            x = x >> 16;
        };

        // Check higher 8 bits
        if (x > 0xff) {
            r = r + 8;
            x = x >> 8;
        };

        // Check higher 4 bits
        if (x > 0xf) {
            r = r + 4;
            x = x >> 4;
        };

        // Check higher 2 bits
        if (x > 0x3) {
            r = r + 2;
            x = x >> 2;
        };

        // Check higher 1 bit
        if (x > 0x1) {
            r = r + 1;
        };

        r
    }


    public fun starts_with(s: String, prefix: String): bool {
        let s_len = string::length(&s);
        let prefix_len = string::length(&prefix);

        if (prefix_len > s_len) {
            return false
        };

        // Convert String to vector<u8>
        let s_bytes: vector<u8> = *std::string::as_bytes(&s);
        let prefix_bytes: vector<u8> = *std::string::as_bytes(&prefix);

        let mut i = 0;
        while (i < prefix_len) {
            if (vector::borrow(&s_bytes, i) != vector::borrow(&prefix_bytes, i)) {
                return false
            };
            i = i + 1;
        };
        true
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

    public entry fun channel_close_init(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public entry fun channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    /// Function to trim a prefix from the string if it starts with that prefix
    public fun trim_prefix(s: String, prefix: String): String {
        // Check if the string starts with the prefix
        if (!starts_with(s, prefix)) {
            return s // If the prefix doesn't match, return the string as-is
        };

        // Get the lengths
        let prefix_len = string::length(&prefix);
        let s_len = string::length(&s);

        // Get the bytes of the string and create a new trimmed vector
        let s_bytes = std::string::as_bytes(&s);
        let mut trimmed_bytes = vector::empty<u8>();

        // Manually copy elements starting from prefix_len to s_len
        let mut i = prefix_len;
        while (i < s_len) {
            vector::push_back(&mut trimmed_bytes, s_bytes[i]);
            i = i + 1;
        };


        // Convert the trimmed vector back to a string
        string::utf8(trimmed_bytes)
    }


    public entry fun register_capability<T>(
        relay_store: &mut RelayStore,
        mut capability: TreasuryCap<T>
    ) {
        let supply = coin::supply(&mut capability);
        if (balance::supply_value(supply) != 0 ) {
            abort 0
        };
        let typename_t = type_name::get<T>();
        let key = type_name::into_string(typename_t);
        relay_store.type_name_t_to_capability.add(string::from_ascii(key), capability)
    }

    public entry fun save_token_origin(
        relay_store: &mut RelayStore,
        wrapped_token: vector<u8>,
        path: u256,
        channel_id: u32
    ) {
        let updated_channel_path = update_channel_path(path, channel_id);
        relay_store.token_origin.add(wrapped_token, updated_channel_path);
    }

    public entry fun recv_packet<T>(
        ibc_store: &mut ibc::IBCStore,
        zkgm: &mut RelayStore,
        clock: &clock::Clock,
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

    fun process_receive<T>(
        zkgm: &mut RelayStore,
        clock: &clock::Clock,
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
                return vector::empty();
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
                match (version) {
                    INSTR_VERSION_1 => {
                        // TODO(aeryz): fix
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
                    _ => (vector::empty(), E_UNSUPPORTED_VERSION)
                }
            },
            OP_BATCH => {
                if (version != INSTR_VERSION_0) {
                    return (vector::empty(), E_UNSUPPORTED_VERSION)  
                };

                zkgm.execute_batch<T>(
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
            OP_FORWARD => (vector::empty(), E_NO_EXECUTE_OPERATION),
            OP_MULTIPLEX => (vector::empty(), E_NO_MULTIPLEX_OPERATION),
            _ => (vector::empty(), E_UNKNOWN_SYSCALL)
        }
    }

    fun market_maker_fill<T>(
        zkgm: &mut RelayStore,
        relayer_msg: vector<u8>,
        quote_token: vector<u8>,
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

    public fun compute_salt(path: u256, channel: u32, base_token: vector<u8>): vector<u8> {
        let mut data: vector<u8> = bcs::to_bytes(&path);
        data.append(bcs::to_bytes(&channel));
        data.append(base_token);

        hash::keccak256(&data)
    }

    public fun distribute_coin<T>(
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

    public fun protocol_fill_mint<T>(
        relay_store: &mut RelayStore,
        channel_id: u32,
        path: u256,
        wrapped_token: vector<u8>,
        receiver: address,
        relayer: address,
        base_amount: u64,
        quote_amount: u64,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let fee = base_amount - quote_amount;
        // if this token is minted for the first time, then we need to ensure that its always minting the same T
        if (!relay_store.claim_wrapped_denom<T>(wrapped_token)) {
            return (vector::empty(), E_ANOTHER_TOKEN_IS_REGISTERED); 
        };
        let capability = relay_store.get_treasury_cap<T>();
        if (quote_amount > 0) {
            coin::mint_and_transfer<T>(capability, quote_amount, receiver, ctx);
        };
        if (fee > 0){
            coin::mint_and_transfer<T>(capability, fee, relayer, ctx);
        };

        (fungible_asset_order_ack::new(
            FILL_TYPE_PROTOCOL,
            ACK_EMPTY
        ).encode(), 0)
    }

    public fun protocol_fill_unescrow<T>(
        zkgm: &mut RelayStore,
        channel_id: u32,
        path: u256,
        quote_token: vector<u8>,
        receiver: address,
        relayer: address,
        base_amount: u64,
        quote_amount: u64,
        ctx: &mut TxContext
    ): (vector<u8>, u64) {
        let fee = base_amount - quote_amount;

        if (zkgm.decrease_outstanding(
            channel_id,
            reverse_channel_path(path), 
            quote_token, 
            (quote_amount + fee)as u256
        ) != 0) {
            return (vector::empty(), 0);
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

    fun execute_batch<T>(
        zkgm: &mut RelayStore,
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
            if (!is_allowed_batch_instruction(instruction.opcode())) {
                return (vector::empty(), E_INVALID_BATCH_INSTRUCTION);
            };

            let (ack, err) = zkgm.execute_internal<T>(
                ibc_packet,
                relayer,
                relayer_msg,
                derive_batch_salt(i, salt),
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

        if (intent) {
            return zkgm.market_maker_fill<T>(
                relayer_msg,
                quote_token,
                receiver,
                order.quote_amount() as u64,
                ctx,
            );
        };

        let wrapped_token = compute_salt(
            path,
            ibc_packet.destination_channel_id(),
            *order.base_token()
        );
        let base_amount = order.base_amount();
        let quote_amount = order.quote_amount();
        let base_amount_covers_quote_amount = base_amount >= quote_amount;

        if (quote_token == wrapped_token && base_amount_covers_quote_amount) {
            // TODO: add rate limit here later
            
            // We expect the token to be deployed already here and the treasury cap is registered previously with type T
            zkgm.protocol_fill_mint<T>(
                ibc_packet.destination_channel_id(), 
                path, 
                wrapped_token, 
                receiver, 
                relayer, 
                base_amount as u64, 
                quote_amount as u64, 
                ctx
            )
        } else if (order.base_token_path() != 0 && base_amount_covers_quote_amount) {
            // TODO: add rate limit here later
            zkgm.protocol_fill_unescrow<T>(
                ibc_packet.destination_channel_id(), 
                path, 
                quote_token, 
                receiver, 
                relayer, 
                base_amount as u64, 
                quote_amount as u64, 
                ctx
            )
        } else {
            zkgm.market_maker_fill<T>(
                relayer_msg, 
                quote_token, 
                receiver, 
                quote_amount as u64,
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
    public fun channel_balance(
        relay_store: &mut RelayStore,
        channel_id: u32,
        path: u256,
        token: vector<u8>
    ): u256 {
        let pair = ChannelBalancePair{
            channel: channel_id,
            path: path,
            token: token
        };
        event::emit(pair);
        if(!relay_store.channel_balance.contains(pair)) {
            abort E_CHANNEL_BALANCE_PAIR_NOT_FOUND
        };
        *relay_store.channel_balance.borrow(pair)
    }
    
    fun decrease_outstanding(
        relay_store: &mut RelayStore,
        channel_id: u32,
        path: u256,
        token: vector<u8>,
        amount: u256
    ): u64 {
        let pair = ChannelBalancePair {
            channel: channel_id,
            path: path,
            token: token
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
        channel_id: u32,
        path: u256,
        token: vector<u8>,
        amount: u256
    ) {
        let pair = ChannelBalancePair{
            channel: channel_id,
            path: path,
            token: token
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



    public entry fun send<T>(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        clock: &Clock,
        coin: Coin<T>,
        metadata: &CoinMetadata<T>,
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
        let sender = tx_context::sender(ctx);
        verify_internal<T>(ibc_store, relay_store, coin, metadata, sender, channel_id, 0, instruction, ctx);

        let zkgm_pack = zkgm_packet::new(salt, 0, instruction);
        ibc::send_packet(
            ibc_store,
            clock,
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack),
            IbcAppWitness {},
            ctx
        );
    }
    fun verify_internal<T>(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        coin: Coin<T>,
        metadata: &CoinMetadata<T>,
        sender: address,
        channel_id: u32,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ){
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            verify_fungible_asset_order<T>(
                ibc_store,
                relay_store,
                coin,
                metadata,
                sender,
                channel_id,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            abort E_NO_BATCH_OPERATION
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            verify_forward<T>(
                ibc_store,
                relay_store,
                coin,
                metadata,
                sender,
                channel_id,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            abort E_NO_MULTIPLEX_OPERATION
        } else {
            abort E_UNKNOWN_SYSCALL
        };
    }

    fun get_treasury_cap<T>(
        relay_store: &mut RelayStore
    ): &mut TreasuryCap<T> {
        let typename_t = type_name::get<T>();
        let key = string::from_ascii(type_name::into_string(typename_t));
        if(!type_name_contains_capability(relay_store, key)) {
            abort E_NO_TREASURY_CAPABILITY
        };
        let capability: &mut TreasuryCap<T> = relay_store.type_name_t_to_capability.borrow_mut(key);
        return capability
    }

    fun claim_wrapped_denom<T>(
        relay_store: &mut RelayStore,
        wrapped_denom: vector<u8>
    ): bool {
        let typename_t = type_name::get<T>();
        let key = string::from_ascii(type_name::into_string(typename_t));
        if (!relay_store.wrapped_denom_to_t.contains(wrapped_denom)) {
            relay_store.wrapped_denom_to_t.add(wrapped_denom, key);
            true
        } else {
            let claimed_key = relay_store.wrapped_denom_to_t.borrow(wrapped_denom);
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

    fun verify_fungible_asset_order<T>(
        _ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        coin: Coin<T>,
        metadata: &CoinMetadata<T>,
        _sender: address,
        channel_id: u32,
        path: u256,
        order: FungibleAssetOrder,
        _ctx: &mut TxContext
    ){

        let base_token = fungible_asset_order::base_token(&order);
        let base_amount = fungible_asset_order::base_amount(&order);
        let token_name = fungible_asset_order::base_token_name(&order);
        let token_symbol = fungible_asset_order::base_token_symbol(&order);
        let token_decimals = fungible_asset_order::base_token_decimals(&order);

        if (token_name != coin::get_name(metadata)) {
            abort E_INVALID_ASSET_NAME
        };

        if (token_symbol != string::from_ascii(coin::get_symbol(metadata))) {
            abort E_INVALID_ASSET_SYMBOL
        };

        if (token_decimals != coin::get_decimals(metadata)) {
            abort E_INVALID_ASSET_DECIMAL
        };

        if(balance::value(coin::balance(&coin)) != base_amount as u64){
            abort E_INVALID_BASE_AMOUNT
        };

        
        let mut origin = 0;    
        if(relay_store.token_origin.contains(*base_token)) {
            origin = *relay_store.token_origin.borrow(*base_token);
        };

        let (intermediate_channel_path, destination_channel_id) =
            pop_channel_from_path(origin);

        let wrapped_token = compute_salt(
            intermediate_channel_path,
            channel_id,
            *fungible_asset_order::quote_token(&order)
        );

        let is_inverse_intermediate_path = path == reverse_channel_path(intermediate_channel_path);
        let is_sending_back_to_same_channel = destination_channel_id == channel_id;
        let is_unwrapping = base_token == wrapped_token;
        let base_token_path = fungible_asset_order::base_token_path(&order);
        if (is_inverse_intermediate_path && is_sending_back_to_same_channel && is_unwrapping) {
            if(origin != base_token_path) {
                abort E_INVALID_ASSET_ORIGIN
            };
            let capability = get_treasury_cap<T>(relay_store);
            coin::burn<T>(capability, coin);
        } else {
            if (base_token_path != 0) {
                abort E_INVALID_ASSET_ORIGIN
            };
            increase_outstanding(
                relay_store, 
                channel_id, 
                path, 
                *base_token, 
                fungible_asset_order::base_amount(&order)
            );
            // There can not be a scenario where base_token == NATIVE_TOKEN_ERC_7528_ADDRESS
            // And here if i don't do anything, the COIN will be ours anyway. We just need to 
            // merge that one
            save_coin_to_bag<T>(relay_store, coin);
        }   

    }

    fun verify_forward<T>(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        coin: Coin<T>,
        metadata: &CoinMetadata<T>,
        sender: address,
        channel_id: u32,
        forward_packet: Forward,
        ctx: &mut TxContext
    ){
        let instruction = forward::instruction(&forward_packet);
        let opcode = instruction::opcode(instruction);

        let is_allowed_forward = opcode == OP_MULTIPLEX || 
                            opcode == OP_BATCH || 
                            opcode == OP_FUNGIBLE_ASSET_ORDER;
        if(!is_allowed_forward) {
            abort E_ERR_INVALID_FORWARD_INSTRUCTION
        };
        verify_internal<T>(
            ibc_store,
            relay_store,
            coin,
            metadata,
            sender,
            channel_id,
            forward::path(&forward_packet),
            *instruction,
            ctx
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

    fun on_acknowledge_packet<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
        ibc_packet: Packet,
        ack: vector<u8>,
        relayer: address,
        ctx: &mut TxContext,
    ) {
        let zkgm_packet = zkgm_packet::decode(ibc_packet.data());
        if (is_forwarded_packet(zkgm_packet.salt())) {
            let packet_hash = commitment::commit_packet(&ibc_packet);

            if (zkgm.in_flight_packet.contains(packet_hash)) {
                let parent = zkgm.in_flight_packet.remove(packet_hash);
                ibc.write_acknowledgement(parent, ack, IbcAppWitness {});
                return
            };
        };

        let zkgm_ack = ack::decode(&ack);
        zkgm.acknowledge_internal<T>(
            ibc,
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
        ibc: &mut ibc::IBCStore,
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
                if (version == INSTR_VERSION_1) {
                    let order = fungible_asset_order::decode(instruction.operand());
                    
                    zkgm.acknowledge_fungible_asset_order<T>(
                        ibc,
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
                    abort E_UNSUPPORTED_VERSION;
                };
            },
            OP_BATCH => {
                assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION)  ;
                zkgm.acknowledge_batch<T>(
                    ibc,
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
            _ => abort 1
        };
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {

        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            abort E_NO_BATCH_OPERATION
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            zkgm.acknowledge_forward<T>(
                ibc,
                ibc_packet,
                relayer,
                salt,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack,
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            abort E_NO_MULTIPLEX_OPERATION
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun acknowledge_batch<T>(
        zkgm: &mut RelayStore,
        ibc: &mut ibc::IBCStore,
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
        let mut idx = 0;
        let batch_ack = batch_ack::decode(&ack, &mut idx);

        let mut i = 0;
        while (i < l) {
            let mut syscall_ack = ack;
            if (success) {
                syscall_ack = batch_ack.acknowledgements()[i];
            };

            zkgm.acknowledge_internal<T>(
                ibc,
                ibc_packet,
                relayer,
                path,
                derive_batch_salt(i, salt),
                batch.instructions()[i],
                success,
                syscall_ack,
                ctx
            );
        };
    }

    fun acknowledge_fungible_asset_order<T>(
        zkgm: &mut RelayStore,
        ibc_store: &mut ibc::IBCStore,
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
                
                if (order.base_token_path() != 0) {
                    let capability = zkgm.get_treasury_cap<T>();
                    coin::mint_and_transfer<T>(capability, order.base_amount() as u64, market_maker, ctx);
                } else {
                    let res = zkgm.decrease_outstanding(
                        ibc_packet.source_channel_id(), 
                        path, 
                        *order.base_token(), 
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
        ibc: &mut ibc::IBCStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        forward_packet: Forward,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        zkgm.acknowledge_internal<T>(
            ibc,
            ibc_packet,
            relayer,
            forward::path(&forward_packet),
            salt,
            *forward::instruction(&forward_packet),
            success,
            ack,
            ctx
        )
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
        let packet =
            packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        ibc_store.timeout_packet(
            packet,
            proof,
            proof_height
        );

        let packet_hash = commitment::commit_packet(&packet);

        let parent = zkgm.in_flight_packet.borrow(packet_hash);
        if (packet::timeout_timestamp(parent) != 0 ||
            packet::timeout_height(parent) != 0) {
                let ack = ack::failure(ACK_EMPTY);
                ibc::write_acknowledgement(ibc_store, *parent, ack::encode(&ack), IbcAppWitness {});
                add_or_update_table<vector<u8>, Packet>(
                    &mut zkgm.in_flight_packet,
                    packet_hash,
                    packet::default()
                );
        } else {
            let zkgm_packet = zkgm_packet::decode(&packet_data);
            zkgm.timeout_internal<T>(
                ibc_store,
                packet,
                relayer,
                zkgm_packet::path(&zkgm_packet),
                zkgm_packet::instruction(&zkgm_packet),
                ctx
            )
        }
    }

    fun timeout_internal<T>(
        zkgm: &mut RelayStore,
        ibc_store: &mut ibc::IBCStore,
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ) {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            let order = fungible_asset_order::decode(instruction::operand(&instruction));
            zkgm.timeout_fungible_asset_order<T>(
                ibc_store,
                ibc_packet,
                path,
                order,
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            abort E_NO_BATCH_OPERATION
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            zkgm.timeout_forward<T>(
                ibc_store,
                ibc_packet,
                relayer,
                path,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            abort E_NO_MULTIPLEX_OPERATION
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun timeout_fungible_asset_order<T>(
        zkgm: &mut RelayStore,
        ibc_store: &mut ibc::IBCStore,
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
        if (order.base_token_path() != 0) {
            coin::mint_and_transfer<T>(capability, order.base_amount() as u64, sender, ctx);
        } else {
            zkgm.decrease_outstanding(
                source_channel,
                path, 
                *order.base_token(), 
                order.base_amount() as u256
            );
            
            zkgm.distribute_coin<T>(sender, order.base_amount() as u64, ctx);
        }
    }

    fun timeout_forward<T>(
        zkgm: &mut RelayStore,
        ibc_store: &mut ibc::IBCStore,
        packet: Packet,
        relayer: address,
        path: u256,
        forward_packet: Forward,
        ctx: &mut TxContext
    ) {
        zkgm.timeout_internal<T>(
            ibc_store, 
            packet, 
            relayer, 
            path, 
            *forward::instruction(&forward_packet), 
            ctx
        )
    }

    fun is_allowed_batch_instruction(
        opcode: u8
    ): bool {
        opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER
            || opcode == OP_STAKE || opcode == OP_UNSTAKE
            || opcode == OP_WITHDRAW_STAKE
    }

    fun is_forwarded_packet(mut salt: vector<u8>): bool {
        salt.reverse();
        let salt_u256 = bcs::new(salt).peel_u256();

        (salt_u256 & FORWARD_SALT_MAGIC) == FORWARD_SALT_MAGIC
    }

    fun derive_batch_salt(
        index: u64,
        salt: vector<u8>
    ): vector<u8> {
        let mut data: vector<u8> = bcs::to_bytes(&(index as u256));
        data.append(salt);
        hash::keccak256(&data)
    }

}
