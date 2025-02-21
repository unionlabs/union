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
    use zkgm::fungible_token::{Self, FUNGIBLE_TOKEN};
    use zkgm::batch::{Self, Batch};
    use zkgm::batch_ack::{Self};
    use zkgm::instruction::{Self, Instruction};
    use zkgm::zkgm_packet::{Self};
    use zkgm::forward::{Self, Forward};
    use zkgm::fungible_asset_order::{Self, FungibleAssetOrder};
    use zkgm::fungible_asset_order_ack::{Self};
    use zkgm::multiplex::{Self, Multiplex};
    use zkgm::acknowledgement::{Self};
    use zkgm::zkgm_ethabi;
    
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};

    use std::string::{Self, String, utf8};
    use sui::table::{Self, Table};
    use ibc::commitment;
    use sui::bcs;
    use sui::clock;
    use sui::address::{to_string};
    use sui::event;

    // Constants
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const ACK_SUCCESS: u256 = 1;
    const ACK_FAILURE: u256 = 0;
    const ACK_LENGTH: u64 = 1;
    const INSTR_VERSION_0: u8 = 0x00;
    const INSTR_VERSION_1: u8 = 0x01;

    const OP_FORWARD: u8 = 0x00;
    const OP_MULTIPLEX: u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;

    const SYSCALL_FORWARD: u8 = 0x00;
    const SYSCALL_MULTIPLEX: u8 = 0x01;
    const SYSCALL_BATCH: u8 = 0x02;
    const SYSCALL_FUNGIBLE_ASSET_TRANSFER: u8 = 0x03;
    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";


    // Errors
    const ACK_ERR_ONLYMAKER: vector<u8> = b"DEADC0DE";
    const E_UNAUTHORIZED: u64 = 1;
    const E_INVALID_HOPS: u64 = 2;
    const E_INVALID_IBC_VERSION: u64 = 3;
    const E_INFINITE_GAME: u64 = 4;
    const E_UNSUPPORTED_VERSION: u64 = 5;
    const E_UNKNOWN_SYSCALL: u64 = 6;
    const E_INVALID_ASSET_NAME: u64 = 7;
    const E_INVALID_ASSET_SYMBOL: u64 = 8;
    const E_INVALID_ASSET_ORIGIN: u64 = 9;
    const E_INVALID_AMOUNT: u64 = 10;
    const E_BATCH_MUST_BE_SYNC: u64 = 11;
    const E_INVALID_FILL_TYPE: u64 = 12;
    const E_UNIMPLEMENTED: u64 = 13;
    const E_ACK_EMPTY: u64 = 14;
    const E_ONLY_MAKER: u64 = 15;
    const E_BATCH_MISMATCH: u64 = 16;

    public struct ZkgmPacket has copy, drop, store {
        salt: vector<u8>,
        path: u256,
        instruction: Instruction
    }

    // struct Instruction has copy, drop, store {
    //     version: u8,
    //     opcode: u8,
    //     operand: vector<u8>
    // }

    public struct SyscallPacket has copy, drop, store {
        version: u8,
        index: u8,
        packet: vector<u8>
    }

    #[event]
    public struct OnZkgmCall has copy, drop, store {
        sender: vector<u8>,
        contract_calldata: vector<u8>,
        contract_address: vector<u8>
    }

    public struct OnZkgmParams has copy, drop, store {
        sender: vector<u8>,
        contract_calldata: vector<u8>
    }

    public struct IIBCModuleOnRecvPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    #[event]
    public struct OnIIBCModuleOnRecvPacketCall has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        contract_address: vector<u8>
    }

    public struct IIBCModuleOnAcknowledgementPacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address
    }


    #[event]
    public struct OnIIBCModuleOnAcknowledgementPacketCall has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address,
        contract_address: vector<u8>
    }

    public struct IIBCModuleOnTimeoutPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address
    }

    #[event]
    public struct OnIIBCModuleOnTimeoutPacketCall has copy, drop, store {
        packet: Packet,
        relayer: address,
        contract_address: vector<u8>
    }



    public struct Acknowledgement has copy, drop, store {
        tag: u256,
        inner_ack: vector<u8>
    }

    public struct BatchAcknowledgement has copy, drop, store {
        acknowledgements: vector<vector<u8>>
    }

    public struct AssetTransferAcknowledgement has copy, drop, store {
        fill_type: u256,
        market_maker: vector<u8>
    }

    public struct ChannelBalancePair has copy, drop, store {
        channel: u32,
        token: address
    }

    public struct RelayStore has key {
        id: UID,
        in_flight_packet: Table<vector<u8>, Packet>,
        channel_balance: Table<ChannelBalancePair, u256>,
        token_origin: Table<address, u256>,
        address_to_treasurycap: Table<address, TreasuryCap<FUNGIBLE_TOKEN>>,
        // address_to_coin: Table<address, Coin<FUNGIBLE_TOKEN>>

    }

    // Events
    #[event]
    public struct DenomCreated has copy, drop, store {
        channel_id: u32,
        denom: String,
        token: address
    }

    #[event]
    public struct Received has copy, drop, store {
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    public struct Sent has copy, drop, store {
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    public struct Refunded has copy, drop, store {
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }
    
    fun init(ctx: &mut TxContext) {
        let id = object::new(ctx);

        transfer::share_object(RelayStore {
            id: id,
            in_flight_packet: table::new(ctx),
            channel_balance: table::new(ctx),
            token_origin: table::new(ctx),
            address_to_treasurycap: table::new(ctx),
            // address_to_coin: table::new(ctx)
        });
    }
    fun get_treasury_cap_mut(relay_store: &mut RelayStore, denom_address: address): &mut TreasuryCap<FUNGIBLE_TOKEN> {
        relay_store.address_to_treasurycap.borrow_mut(denom_address)
    }

    // fun get_coin_mut(relay_store: &mut RelayStore, denom_address: address): &mut Coin<FUNGIBLE_TOKEN> {
    //     relay_store.address_to_coin.borrow_mut(denom_address)
    // }


    // public entry fun insert_pair(
    //     relay_store: &mut RelayStore,
    //     denom_address: address,
    //     treasury_cap: TreasuryCap<FUNGIBLE_TOKEN>,
    //     coin: Coin<FUNGIBLE_TOKEN>
    // ) {
    //     relay_store.address_to_treasurycap.add(denom_address, treasury_cap);
    //     relay_store.address_to_coin.add(denom_address, coin);
    // }

    
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
        let s_bytes: vector<u8> = *string::bytes(&s);
        let prefix_bytes: vector<u8> = *string::bytes(&prefix);

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
        counterparty_port_id: vector<u8>,
        connection_id: u32, 
        version: String
    ) {
        ibc::channel_open_init(
            ibc_store,
            utf8(b"&get_signer()"),
            counterparty_port_id,
            connection_id,
            version
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_IBC_VERSION
        };

    }
    public entry fun channel_open_try(
        ibc_store: &mut ibc::IBCStore,
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
            utf8(b"&get_signer()"),
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );
    }

    public entry fun channel_open_ack(
        ibc_store: &mut ibc::IBCStore,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        // Store the channel_id
        ibc::channel_open_ack(
            ibc_store,
            utf8(b"&get_signer()"),
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height
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
            utf8(b"&get_signer()"),
            channel_id,
            proof_ack,
            proof_height
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
        let s_bytes = string::bytes(&s);
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


    public entry fun recv_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        clock: &clock::Clock,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        relayer_msg: vector<u8>,
        ctx: &mut TxContext
    ) {
        let mut packets: vector<Packet> = vector::empty();
        let mut i = 0;
        while (i < vector::length(&packet_source_channels)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };
        ibc::recv_packet(
            ibc_store,
            clock,
            packets,
            proof,
            proof_height,
            vector[1]
        );
        while (i < vector::length(&packets)) {
            let ibc_packet = *vector::borrow(&packets, i);
            let raw_zkgm_packet = packet::data(&ibc_packet);
            let zkgm_packet = zkgm_packet::decode(raw_zkgm_packet);

            let acknowledgement =
                execute_internal(
                    ibc_store,
                    relay_store,
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    zkgm_packet::salt(&zkgm_packet),
                    zkgm_packet::path(&zkgm_packet),
                    zkgm_packet::instruction(&zkgm_packet),
                    ctx
                );

            if (vector::length(&acknowledgement) == 0) {
                abort E_ACK_EMPTY
            } else if (acknowledgement == ACK_ERR_ONLYMAKER) {
                abort E_ONLY_MAKER
            } else {
                // TODO: what to do here?
                let new_ack = acknowledgement::new(ACK_SUCCESS, acknowledgement);
                let return_value = acknowledgement::encode(&new_ack);
                // dispatcher_zkgm::set_return_value<ZKGMProof>(
                //     new_ucs_relay_proof(), return_value
                // );
            }

        };
    }


    fun execute_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ): (vector<u8>)  {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            execute_fungible_asset_order(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            execute_batch(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                batch::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            execute_forward(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer_msg,
                salt,
                path,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            execute_multiplex(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                multiplex::decode(instruction::operand(&instruction)),
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    // TODO: Voyager need to call this after token deployment
    // with wrapped address & value
    public entry fun update_token_origin(
        relay_store: &mut RelayStore,
        token: address,
        channel_id: u256
    ) {
        add_or_update_table<address, u256>(&mut relay_store.token_origin, token, channel_id);
    }

    fun execute_fungible_asset_order(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        _relayer_msg: vector<u8>,
        _salt: vector<u8>,
        path: u256,
        order: FungibleAssetOrder,
        ctx: &mut TxContext
    ): (vector<u8>) {
        // TODO: this function will be a problem for us
        // investigate it later.
        vector::empty()
    }

    fun execute_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        batch_packet: Batch,
        ctx: &mut TxContext
    ): (vector<u8>) {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let mut acks = vector::empty();
        let mut i = 0;
        while (i < l) {
            let instruction = *vector::borrow(&instructions, i);
            vector::push_back(
                &mut acks,
                execute_internal(
                    ibc_store,
                    relay_store,
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    salt,
                    path,
                    instruction,
                    ctx
                )
            );
            if (vector::length(vector::borrow(&acks, i)) == 0) {
                abort E_BATCH_MUST_BE_SYNC
            };
        };
        let batch_ack = batch_ack::new(acks);
        batch_ack::encode(&batch_ack)
    }

    fun execute_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        _relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        forward_packet: Forward,
        ctx: &mut TxContext
    ): (vector<u8>) {

        let zkgm_pack =
            zkgm_packet::new(
                salt,
                update_channel_path(
                    path, packet::destination_channel_id(&ibc_packet)
                ),
                *forward::instruction(&forward_packet)
            );
        let sent_packet =
            ibc::send_packet(
                ibc_store,
                forward::channel_id(&forward_packet),
                forward::timeout_height(&forward_packet),
                forward::timeout_timestamp(&forward_packet),
                zkgm_packet::encode(&zkgm_pack)
            );


        let packet_hash = commitment::commit_packet(&sent_packet);
        add_or_update_table<vector<u8>, Packet>(&mut relay_store.in_flight_packet, packet_hash, sent_packet);
        ACK_EMPTY
    }

    fun add_or_update_table<T: drop + store + copy, P: drop + store>(table: &mut Table<T, P>, key: T, mut value: P) {
        if (table.contains(key)) {
            let mut val = table.borrow_mut(key);
            *val = value;
        } else {
            table.add(key, value);
        }
    }

    fun execute_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        _salt: vector<u8>,
        multiplex_packet: Multiplex,
        ctx: &mut TxContext
    ): (vector<u8>) {
        let contract_address =
            bcs::new(*multiplex::contract_address(&multiplex_packet)).peel_address();

        if (multiplex::eureka(&multiplex_packet)) {
            // TODO: discuss this, is it ok to do?
            // event::emit(
            //     OnZkgmCall {
            //         sender: *multiplex::sender(&multiplex_packet),
            //         contract_calldata: *multiplex::contract_calldata(&multiplex_packet),
            //         contract_address: contract_address
            //     }
            // );
            return bcs::to_bytes(&ACK_SUCCESS)
        };
        let multiplex_ibc_packet =
            packet::new(
                packet::source_channel_id(&ibc_packet),
                packet::destination_channel_id(&ibc_packet),
                multiplex::encode_multiplex_sender_and_calldata(
                    *multiplex::sender(&multiplex_packet),
                    *multiplex::contract_calldata(&multiplex_packet)
                ),
                packet::timeout_height(&ibc_packet),
                packet::timeout_timestamp(&ibc_packet)
            );

        // TODO: How do we return something from this? investigate
        // event::emit(
        //     OnIIBCModuleOnRecvPacketCall {
        //         packet: multiplex_ibc_packet,
        //         relayer: relayer,
        //         relayer_msg: relayer_msg,
        //         contract_address: multiplex_packet.contract_address
        //     }
        // );
        vector::empty() // TODO: investigate
        // Here is the original implementation in aptos
        // let param =
        //     copyable_any::pack<IIBCModuleOnRecvPacketParams>(
        //         IIBCModuleOnRecvPacketParams {
        //             packet: multiplex_ibc_packet,
        //             relayer: relayer,
        //             relayer_msg: relayer_msg
        //         }
        //     );

        // engine_zkgm::dispatch(param, contract_address);

        // let acknowledgement = dispatcher_zkgm::get_return_value(contract_address);

        // if (vector::length(&acknowledgement) == 0) {
        //     abort E_UNIMPLEMENTED
        // };
        // acknowledgement
    }

    public entry fun send(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
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
        verify_internal(ibc_store, relay_store, sender, channel_id, 0, instruction, ctx);

        let zkgm_pack = zkgm_packet::new(salt, 0, instruction);
        ibc::send_packet(
            ibc_store,
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack)
        );
    }
    fun verify_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ){
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            verify_fungible_asset_order(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            verify_batch(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                batch::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            verify_forward(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            verify_multiplex(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                multiplex::decode(instruction::operand(&instruction)),
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        };
    }

    fun verify_fungible_asset_order(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        transfer_packet: FungibleAssetOrder,
        ctx: &mut TxContext
    ){
        // let sent_token = transfer_packet.sent_token;
        // let treasury_cap = relay_store.address_to_treasurycap.borrow_mut(sent_token);

        // TODO: implement this further, can't take coin as argument because of that
        // copy issue

    }

    fun verify_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        batch_packet: Batch,
        ctx: &mut TxContext
    ){
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);

        let mut i = 0;
        while (i < l) {
            verify_internal(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                *vector::borrow(&instructions, i),
                ctx
            );
            i = i + 1;
        }

    }

    fun verify_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        forward_packet: Forward,
        ctx: &mut TxContext
    ){
        verify_internal(
            ibc_store,
            relay_store,
            sender,
            channel_id,
            update_channel_path(path, forward::channel_id(&forward_packet)),
            *forward::instruction(&forward_packet),
            ctx
        );
    }

    fun verify_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        multiplex_packet: Multiplex,
        ctx: &mut TxContext
    ){

    }

    public entry fun acknowledge_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        relayer: address,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &mut TxContext
    ) {
        let mut packets: vector<Packet> = vector::empty();
        let mut i = 0;
        while (i < vector::length(&packet_source_channels)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };
        ibc::acknowledge_packet(
            ibc_store,
            packets,
            acknowledgements,
            proof,
            proof_height
        );

        let mut i = 0;
        while (i < vector::length(&packet_source_channels)) {
            let ibc_packet = *vector::borrow(&packets, i);
            let acknowledgement = *vector::borrow(&acknowledgements, i);
            let packet_hash = commitment::commit_packet(&ibc_packet);
            let parent = relay_store.in_flight_packet.borrow(packet_hash);

            if (packet::timeout_timestamp(parent) != 0 ||
                packet::timeout_height(parent) != 0) {
                    ibc::write_acknowledgement(
                        ibc_store,
                        *parent,
                        acknowledgement
                    );
                    add_or_update_table<vector<u8>, Packet>(
                        &mut relay_store.in_flight_packet,
                        packet_hash,
                        packet::default()
                    );
            } else {
                let zkgm_packet = zkgm_packet::decode(packet::data(&ibc_packet));
                let zkgm_ack = acknowledgement::decode(&acknowledgement);
                acknowledge_internal(
                    ibc_store,
                    relay_store,
                    ibc_packet,
                    relayer,
                    zkgm_packet::salt(&zkgm_packet),
                    zkgm_packet::instruction(&zkgm_packet),
                    acknowledgement::tag(&zkgm_ack) == ACK_SUCCESS,
                    *acknowledgement::inner_ack(&zkgm_ack),
                    ctx
                )
            };
            i = i + 1;
        };
    }

    fun acknowledge_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        instruction: Instruction,
        success: bool,
        inner_ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            acknowledge_fungible_asset_order(
                ibc_store,
                relay_store,
                ibc_packet,
                salt,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                success,
                inner_ack,
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            acknowledge_batch(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                batch::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack,
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            acknowledge_forward(
                ibc_store,
                relay_store,
                ibc_packet,
                salt,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack,
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            acknowledge_multiplex(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                multiplex::decode(instruction::operand(&instruction)),
                success,
                inner_ack,
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun acknowledge_fungible_asset_order(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        salt: vector<u8>,
        transfer_packet: FungibleAssetOrder,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        // TODO: fill it later
    }

    fun acknowledge_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: Batch,
        success: bool,
        inner_ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let mut idx = 0x40;
        let batch_ack = batch_ack::decode(&inner_ack, &mut idx);
        let mut i = 0;
        while (i < l) {
            let mut syscall_ack = inner_ack;
            if (success) {
                syscall_ack = *vector::borrow(
                    &batch_ack::acknowledgements(&batch_ack), i
                );
            };
            acknowledge_internal(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                *vector::borrow(&instructions, i),
                success,
                syscall_ack,
                ctx
            );
            i = i + 1;
        }
    }

    fun acknowledge_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        salt: vector<u8>,
        forward_packet: Forward,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {

    }

    fun acknowledge_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        multiplex_packet: Multiplex,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        if (success && !multiplex::eureka(&multiplex_packet)) {
            let multiplex_ibc_packet =
                packet::new(
                    packet::source_channel_id(&ibc_packet),
                    packet::destination_channel_id(&ibc_packet),
                    multiplex::encode_multiplex_sender_and_calldata(
                        *multiplex::contract_address(&multiplex_packet),
                        *multiplex::contract_calldata(&multiplex_packet)
                    ),
                    packet::timeout_height(&ibc_packet),
                    packet::timeout_timestamp(&ibc_packet)
                );
            // TODO: verify this
            // event::emit(
            //     OnIIBCModuleOnAcknowledgementPacketCall {
            //         packet: multiplex_ibc_packet,
            //         acknowledgement: ack,
            //         relayer: relayer,
            //         contract_address: multiplex_packet.sender
            //     }
            // )
            // let param =
            //     copyable_any::pack<IIBCModuleOnAcknowledgementPacketParams>(
            //         IIBCModuleOnAcknowledgementPacketParams {
            //             packet: multiplex_ibc_packet,
            //             acknowledgement: ack,
            //             relayer: relayer
            //         }
            //     );
            // let contract_address = from_bcs::to_address(multiplex_packet.sender);

            // engine_zkgm::dispatch(param, contract_address);
        }
    }
    public entry fun timeout_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        next_sequence_receive: u64,
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

        ibc::timeout_packet(
            ibc_store,
            packet,
            proof,
            proof_height,
            next_sequence_receive
        );

        let packet_hash = commitment::commit_packet(&packet);

        let parent = relay_store.in_flight_packet.borrow(packet_hash);
        let parent = relay_store.in_flight_packet.borrow(packet_hash);
        if (packet::timeout_timestamp(parent) != 0 ||
            packet::timeout_height(parent) != 0) {
                let ack = acknowledgement::new(ACK_FAILURE, ACK_EMPTY);
                ibc::write_acknowledgement(ibc_store, *parent, acknowledgement::encode(&ack));
                add_or_update_table<vector<u8>, Packet>(
                    &mut relay_store.in_flight_packet,
                    packet_hash,
                    packet::default()
                );
        } else {
            let zkgm_packet = zkgm_packet::decode(&packet_data);
            timeout_internal(
                ibc_store,
                relay_store,
                packet,
                relayer,
                zkgm_packet::salt(&zkgm_packet),
                zkgm_packet::instruction(&zkgm_packet),
                ctx
            )
        }
    }

    fun timeout_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        instruction: Instruction,
        ctx: &mut TxContext
    ) {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            timeout_fungible_asset_order(
                ibc_store,
                relay_store,
                ibc_packet,
                salt,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            timeout_batch(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                batch::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let mut decode_idx = 0x20;
            timeout_forward(
                ibc_store,
                relay_store,
                ibc_packet,
                salt,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                ctx
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            timeout_multiplex(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                multiplex::decode(instruction::operand(&instruction)),
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun timeout_fungible_asset_order(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        _salt: vector<u8>,
        transfer_packet: FungibleAssetOrder,
        ctx: &mut TxContext
    ) {
        refund(packet::source_channel_id(&packet), transfer_packet, ctx);
    }

    fun refund(
        channel_id: u32,
        transfer_packet: FungibleAssetOrder,
        ctx: &mut TxContext
    ) {
        // TODO: Fill it later
    }

    fun timeout_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: Batch,
        ctx: &mut TxContext
    ) {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let mut i = 0;
        while (i < l) {
            timeout_internal(
                ibc_store,
                relay_store,
                packet,
                relayer,
                salt,
                *vector::borrow(&instructions, i),
                ctx
            );
            i = i + 1;
        }
    }

    fun timeout_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        _salt: vector<u8>,
        forward_packet: Forward,
        ctx: &mut TxContext
    ) {

    }

    fun timeout_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        _salt: vector<u8>,
        multiplex_packet: Multiplex,
        ctx: &mut TxContext
    ) {
        if (!multiplex::eureka(&multiplex_packet)) {
            let multiplex_ibc_packet =
                packet::new(
                    packet::source_channel_id(&ibc_packet),
                    packet::destination_channel_id(&ibc_packet),
                    multiplex::encode_multiplex_sender_and_calldata(
                        *multiplex::contract_address(&multiplex_packet),
                        *multiplex::contract_calldata(&multiplex_packet)
                    ),
                    packet::timeout_height(&ibc_packet),
                    packet::timeout_timestamp(&ibc_packet)
                );
            // TODO: verify this

            // event::emit(
            //     OnIIBCModuleOnTimeoutPacketCall{
            //         packet: multiplex_ibc_packet,
            //         relayer: relayer,
            //         contract_address: multiplex_packet.sender
            //     }
            // );
            // let param =
            //     copyable_any::pack<IIBCModuleOnTimeoutPacketParams>(
            //         IIBCModuleOnTimeoutPacketParams {
            //             packet: multiplex_ibc_packet,
            //             relayer: relayer
            //         }
            //     );
            // let contract_address = from_bcs::to_address(multiplex_packet.sender);

            // engine_zkgm::dispatch(param, contract_address);

        }
    }

    // public entry fun execute(
    //     ibc_store: &mut ibc::IBCStore,
    //     relay_store: &mut RelayStore,
    //     source_channel: u32,
    //     destination_channel: u32,
    //     data: vector<u8>,
    //     timeout_height: u64,
    //     timeout_timestamp: u64,
    //     relayer: address,
    //     relayer_msg: vector<u8>,
    //     raw_zkgm_packet: vector<u8>,
    //     ctx: &mut TxContext
    // ) {
    //     let ibc_packet =
    //         packet::new(
    //             source_channel,
    //             destination_channel,
    //             data,
    //             timeout_height,
    //             timeout_timestamp
    //         );

    //     let zkgm_packet = decode_packet(raw_zkgm_packet);
    //     execute_internal(
    //         ibc_store,
    //         relay_store,
    //         ibc_packet,
    //         relayer,
    //         relayer_msg,
    //         zkgm_packet.salt,
    //         zkgm_packet.path,
    //         zkgm_packet.instruction,
    //         ctx
    //     );
    // }
}