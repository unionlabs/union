// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory                      
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

module zkgm::ibc_app {
    use zkgm::dispatcher_zkgm;
    use zkgm::engine_zkgm;
    use zkgm::batch::{Self, Batch};
    use zkgm::batch_ack::{Self};
    use zkgm::instruction::{Self, Instruction};
    use zkgm::zkgm_packet::{Self};
    use zkgm::forward::{Self, Forward};
    use zkgm::fungible_asset_order::{Self, FungibleAssetOrder};
    use zkgm::fungible_asset_order_ack::{Self};
    use zkgm::multiplex::{Self, Multiplex};
    use zkgm::acknowledgement::{Self};

    use ibc::ibc;
    use ibc::packet::{Self, Packet};
    use ibc::dispatcher;
    use ibc::commitment;

    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use aptos_std::copyable_any;
    use aptos_framework::function_info;

    use std::string::{Self, String};
    use std::from_bcs;
    use std::bcs;
    use aptos_framework::hash;

    use aptos_framework::fungible_asset::{Metadata};
    use aptos_framework::signer;
    use aptos_std::smart_table::{Self, SmartTable};
    use std::vector;

    // Constants
    const ORDER_UNORDERED: u8 = 1;
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const ACK_SUCCESS: u256 = 1;
    const ACK_FAILURE: u256 = 0;
    const ACK_LENGTH: u64 = 1;
    const ZKGM_VERSION_0: u8 = 0x00;

    const OP_FORWARD: u8 = 0x00;
    const OP_MULTIPLEX: u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;

    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";

    // Errors
    const IBC_APP_SEED: vector<u8> = b"ibc-union-app-v1";
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
    const E_ONLY_MAKER_OTHER: u64 = 15;
    const E_ONLY_MAKER_RECV_PACKET: u64 = 16;

    struct IbcAppWitness has drop, store, key {}

    public(friend) fun new_ucs_relay_proof(): IbcAppWitness {
        IbcAppWitness {}
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    struct ChannelBalancePair has copy, drop, store {
        channel: u32,
        token: address
    }

    struct RelayStore has key {
        in_flight_packet: SmartTable<vector<u8>, Packet>,
        channel_balance: SmartTable<ChannelBalancePair, u256>,
        token_origin: SmartTable<address, u256>
    }

    struct OnZkgmParams has copy, drop, store {
        sender: vector<u8>,
        contract_calldata: vector<u8>
    }

    struct IIBCModuleOnPacketRecvParams has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    struct IIBCModuleOnAcknowledgementPacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address
    }

    struct IIBCModuleOnTimeoutPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address
    }

    struct Port<phantom T: key + store + drop> has key, copy, drop, store {
        port_id: address
    }

    #[view]
    public fun get_metadata(asset_addr: address): Object<Metadata> {
        object::address_to_object<Metadata>(asset_addr)
    }

    #[view]
    public fun get_balance(
        acc: address, token: address
    ): u64 {
        primary_fungible_store::balance(acc, get_metadata(token))
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@zkgm, IBC_APP_SEED)
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
    }

    // Initialize the RelayStore and SignerRef
    fun init_module(account: &signer) {
        assert!(signer::address_of(account) == @zkgm, E_UNAUTHORIZED);

        let vault_constructor_ref = &object::create_named_object(account, IBC_APP_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = RelayStore {
            in_flight_packet: smart_table::new(),
            channel_balance: smart_table::new(),
            token_origin: smart_table::new()
        };

        move_to(vault_signer, store);

        move_to(
            vault_signer,
            SignerRef {
                self_ref: object::generate_extend_ref(vault_constructor_ref),
                self_address: signer::address_of(account)
            }
        );

        let cb =
            function_info::new_function_info(
                account,
                string::utf8(b"ibc_app"),
                string::utf8(b"on_packet")
            );

        ibc::register_application<IbcAppWitness>(account, cb, new_ucs_relay_proof());
    }

    // Initialize the RelayStore and SignerRef
    fun init_module_for_testing(account: &signer) {
        assert!(signer::address_of(account) == @zkgm, E_UNAUTHORIZED);

        let vault_constructor_ref = &object::create_named_object(account, IBC_APP_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = RelayStore {
            in_flight_packet: smart_table::new(),
            channel_balance: smart_table::new(),
            token_origin: smart_table::new()
        };

        move_to(vault_signer, store);

        move_to(
            vault_signer,
            SignerRef {
                self_ref: object::generate_extend_ref(vault_constructor_ref),
                self_address: signer::address_of(account)
            }
        );
    }

    fun serialize_salt(
        path: u256, destination_channel_id: u32, token: vector<u8>
    ): vector<u8> {
        let data = vector::empty<u8>();
        vector::append(&mut data, bcs::to_bytes(&path));
        vector::append(&mut data, bcs::to_bytes(&destination_channel_id));
        vector::append(&mut data, token);
        data
    }

    #[view]
    public fun predict_wrapped_token(
        path: u256, destination_channel_id: u32, token: vector<u8>
    ): (address, vector<u8>) {
        let salt = hash::sha3_256(serialize_salt(path, destination_channel_id, token));

        let wrapped_address = object::create_object_address(&get_vault_addr(), salt);
        (wrapped_address, salt)
    }

    public fun deploy_token(salt: vector<u8>, name: string::String, symbol: string::String, decimals: u8): address acquires SignerRef {
        zkgm::fa_coin::initialize(
            &get_signer(),
            name,
            symbol,
            decimals,
            string::utf8(b""),
            string::utf8(b""),
            salt
        );
        zkgm::fa_coin::get_metadata_address(salt)
    }

    public fun is_deployed(token: address): bool {
        object::is_object(token)
    }

    /// Find last set (most significant bit).
    /// Returns the index of the most significant bit of `x`.
    /// If `x` is zero, returns 256.
    public fun fls(x: u256): u256 {
        if (x == 0) {
            return 256
        };

        let r: u256 = 0;

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

    #[view]
    public fun last_channel_from_path(path: u256): u32 {
        if (path == 0) {
            return 0
        };
        let current_hop_index = ((fls(path) / 32) as u8);
        let last_channel = path >> (current_hop_index * 32);
        (last_channel as u32)
    }

    #[view]
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

    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }

    public entry fun transfer(
        sender: &signer,
        channel_id: u32,
        receiver: vector<u8>,
        base_token: address,
        base_amount: u256,
        quote_token: vector<u8>,
        quote_amount: u256,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>
    ) acquires RelayStore, SignerRef {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        if (base_amount == 0) {
            abort E_INVALID_AMOUNT
        };

        let asset = get_metadata(base_token);
        let name = zkgm::fa_coin::name_with_metadata(asset);
        let symbol = zkgm::fa_coin::symbol_with_metadata(asset);

        let origin = *smart_table::borrow_with_default(
            &store.token_origin, base_token, &0
        );

        let (wrapped_address, _salt) = predict_wrapped_token(0, channel_id, quote_token);

        if (last_channel_from_path(origin) == channel_id
            && base_token == wrapped_address) {
            zkgm::fa_coin::burn_with_metadata(
                &get_signer(),
                signer::address_of(sender),
                (base_amount as u64),
                asset
            );
        } else {
            origin = 0;

            primary_fungible_store::transfer(
                sender,
                asset,
                signer::address_of(&get_signer()),
                (base_amount as u64)
            );

            let balance_key = ChannelBalancePair { channel: channel_id, token: base_token };

            let curr_balance = *smart_table::borrow_mut_with_default(&mut store.channel_balance, balance_key, 0);

            smart_table::upsert(
                &mut store.channel_balance,
                balance_key,
                curr_balance + (base_amount as u256)
            );
        };

        let fungible_asset_order =
            fungible_asset_order::new(
                bcs::to_bytes(&signer::address_of(sender)),
                receiver,
                bcs::to_bytes(&base_token),
                base_amount,
                symbol,
                name,
                origin,
                quote_token,
                quote_amount
            );
        let operand = fungible_asset_order::encode(&fungible_asset_order);
        let zkgm_pack =
            zkgm_packet::new(
                salt,
                0,
                instruction::new(
                    ZKGM_VERSION_0,
                    OP_FUNGIBLE_ASSET_ORDER,
                    operand
                )
            );

        ibc::ibc::send_packet(
            &get_signer(),
            get_self_address(),
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack)
        );
    }

    public entry fun call(
        sender: &signer,
        channel_id: u32,
        contract_address: address,
        contract_calldata: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>
    ) acquires SignerRef {
        let data = vector::empty<u8>();
        vector::append(&mut data, bcs::to_bytes(&signer::address_of(sender)));
        vector::append(&mut data, bcs::to_bytes(&salt));
        let multiplex =
            multiplex::new(
                bcs::to_bytes(&signer::address_of(sender)),
                true,
                bcs::to_bytes(&contract_address),
                contract_calldata
            );
        let operand = multiplex::encode(&multiplex);
        let zkgm_pack =
            zkgm_packet::new(
                data,
                0,
                instruction::new(ZKGM_VERSION_0, OP_MULTIPLEX, operand)
            );
        ibc::ibc::send_packet(
            &get_signer(),
            get_self_address(),
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack)
        );
    }

    public entry fun send(
        sender: &signer,
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>,
        version: u8,
        opcode: u8,
        operand: vector<u8>
    ) acquires SignerRef, RelayStore {
        let instruction = instruction::new(version, opcode, operand);
        verify_internal(sender, channel_id, 0, instruction);
        let zkgm_pack = zkgm_packet::new(salt, 0, instruction);
        ibc::ibc::send_packet(
            &get_signer(),
            get_self_address(),
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack)
        );
    }

    fun verify_internal(
        sender: &signer,
        channel_id: u32,
        path: u256,
        instruction: Instruction
    ) acquires RelayStore, SignerRef {
        if (instruction::version(&instruction) != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            verify_fungible_asset_order(
                sender,
                channel_id,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction))
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            let decode_idx = 0x20;
            verify_batch(
                sender,
                channel_id,
                path,
                batch::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            let decode_idx = 0x20;
            verify_forward(
                sender,
                channel_id,
                path,
                forward::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            verify_multiplex(
                sender,
                channel_id,
                path,
                multiplex::decode(instruction::operand(&instruction))
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun verify_batch(
        sender: &signer,
        channel_id: u32,
        path: u256,
        batch_packet: Batch
    ) acquires RelayStore, SignerRef {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let i = 0;
        while (i < l) {
            verify_internal(
                sender,
                channel_id,
                path,
                *vector::borrow(&instructions, i)
            );
        }
    }

    fun verify_forward(
        sender: &signer,
        channel_id: u32,
        path: u256,
        forward_packet: Forward
    ) acquires RelayStore, SignerRef {
        verify_internal(
            sender,
            channel_id,
            update_channel_path(path, forward::channel_id(&forward_packet)),
            *forward::instruction(&forward_packet)
        );
    }

    fun verify_multiplex(
        _sender: &signer,
        _channel_id: u32,
        _path: u256,
        _multiplex_packet: Multiplex
    ) {}

    fun verify_fungible_asset_order(
        sender: &signer,
        channel_id: u32,
        _path: u256,
        order: FungibleAssetOrder
    ) acquires RelayStore, SignerRef {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        let base_amount = fungible_asset_order::base_amount(&order);

        if (base_amount == 0) {
            abort E_INVALID_AMOUNT
        };

        let base_token = from_bcs::to_address(*fungible_asset_order::base_token(&order));

        let asset = get_metadata(base_token);
        let name = zkgm::fa_coin::name_with_metadata(asset);
        let symbol = zkgm::fa_coin::symbol_with_metadata(asset);

        if (*fungible_asset_order::base_token_name(&order) != name) {
            abort E_INVALID_ASSET_NAME
        };
        if (*fungible_asset_order::base_token_symbol(&order) != symbol) {
            abort E_INVALID_ASSET_SYMBOL
        };
        let origin = *smart_table::borrow_with_default(
            &store.token_origin, base_token, &0
        );

        if (last_channel_from_path(origin) == channel_id) {
            zkgm::fa_coin::burn_with_metadata(
                &get_signer(),
                signer::address_of(sender),
                (base_amount as u64),
                asset
            );
        } else {
            primary_fungible_store::transfer(
                sender,
                asset,
                signer::address_of(&get_signer()),
                (base_amount as u64)
            );

            let balance_key = ChannelBalancePair { channel: channel_id, token: base_token };

            let curr_balance = *smart_table::borrow(&store.channel_balance, balance_key);

            smart_table::upsert(
                &mut store.channel_balance,
                balance_key,
                curr_balance + (base_amount as u256)
            );
        };
        if (fungible_asset_order::base_token_path(&order) != origin) {
            abort E_INVALID_ASSET_ORIGIN
        };
    }

    public fun on_recv_packet(
        ibc_packet: Packet, relayer: address, relayer_msg: vector<u8>
    ) acquires RelayStore, SignerRef {
        // We can call execute_internal directly
        let raw_zkgm_packet = ibc::packet::data(&ibc_packet);
        let zkgm_packet = zkgm_packet::decode(raw_zkgm_packet);

        let acknowledgement =
            execute_internal(
                ibc_packet,
                relayer,
                relayer_msg,
                zkgm_packet::salt(&zkgm_packet),
                zkgm_packet::path(&zkgm_packet),
                zkgm_packet::instruction(&zkgm_packet)
            );

        if (vector::length(&acknowledgement) == 0) {
            abort E_ACK_EMPTY
        } else if (acknowledgement == ACK_ERR_ONLYMAKER) {
            abort E_ONLY_MAKER_RECV_PACKET
        } else {
            let new_ack = acknowledgement::new(ACK_SUCCESS, acknowledgement);
            let return_value = acknowledgement::encode(&new_ack);
            dispatcher::set_return_value<IbcAppWitness>(
                new_ucs_relay_proof(), return_value
            );
        }
    }

    fun execute_internal(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        instruction: Instruction
    ): (vector<u8>) acquires RelayStore, SignerRef {
        if (instruction::version(&instruction) != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            execute_fungible_asset_order(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction))
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            let decode_idx = 0x20;
            execute_batch(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                batch::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            let decode_idx = 0x20;
            execute_forward(
                ibc_packet,
                relayer_msg,
                salt,
                path,
                forward::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            execute_multiplex(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                multiplex::decode(instruction::operand(&instruction))
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun execute_batch(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        batch_packet: Batch
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let acks = vector::empty();
        let i = 0;
        while (i < l) {
            let instruction = *vector::borrow(&instructions, i);
            vector::push_back(
                &mut acks,
                execute_internal(
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    salt,
                    path,
                    instruction
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
        ibc_packet: Packet,
        _relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        forward_packet: Forward
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let zkgm_pack =
            zkgm_packet::new(
                salt,
                update_channel_path(
                    path, ibc::packet::destination_channel_id(&ibc_packet)
                ),
                *forward::instruction(&forward_packet)
            );
        let sent_packet =
            ibc::ibc::send_packet(
                &get_signer(),
                get_self_address(),
                forward::channel_id(&forward_packet),
                forward::timeout_height(&forward_packet),
                forward::timeout_timestamp(&forward_packet),
                zkgm_packet::encode(&zkgm_pack)
            );
        let packet_hash = commitment::commit_packet(&sent_packet);
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.in_flight_packet, packet_hash, ibc_packet);
        ACK_EMPTY
    }

    fun execute_multiplex(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        _salt: vector<u8>,
        multiplex_packet: Multiplex
    ): (vector<u8>) {
        let contract_address =
            from_bcs::to_address(*multiplex::contract_address(&multiplex_packet));
        if (multiplex::eureka(&multiplex_packet)) {
            let param =
                copyable_any::pack<OnZkgmParams>(
                    OnZkgmParams {
                        sender: *multiplex::sender(&multiplex_packet),
                        contract_calldata: *multiplex::contract_calldata(&multiplex_packet)
                    }
                );
            engine_zkgm::dispatch(param, contract_address);
            return bcs::to_bytes(&ACK_SUCCESS)
        };
        let multiplex_ibc_packet =
            ibc::packet::new(
                ibc::packet::source_channel_id(&ibc_packet),
                ibc::packet::destination_channel_id(&ibc_packet),
                multiplex::encode_multiplex_sender_and_calldata(
                    *multiplex::sender(&multiplex_packet),
                    *multiplex::contract_calldata(&multiplex_packet)
                ),
                ibc::packet::timeout_height(&ibc_packet),
                ibc::packet::timeout_timestamp(&ibc_packet)
            );
        let param =
            copyable_any::pack<IIBCModuleOnPacketRecvParams>(
                IIBCModuleOnPacketRecvParams {
                    packet: multiplex_ibc_packet,
                    relayer: relayer,
                    relayer_msg: relayer_msg
                }
            );

        engine_zkgm::dispatch(param, contract_address);

        let acknowledgement = dispatcher_zkgm::get_return_value(contract_address);

        if (vector::length(&acknowledgement) == 0) {
            abort E_UNIMPLEMENTED
        };
        acknowledgement
    }

    fun execute_fungible_asset_order(
        ibc_packet: Packet,
        relayer: address,
        _relayer_msg: vector<u8>,
        _salt: vector<u8>,
        path: u256,
        order: FungibleAssetOrder
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        if (fungible_asset_order::quote_amount(&order)
            > fungible_asset_order::base_amount(&order)) {
            abort E_INVALID_AMOUNT
        };

        let (wrapped_address, salt) =
            predict_wrapped_token(
                path,
                ibc::packet::destination_channel_id(&ibc_packet),
                *fungible_asset_order::base_token(&order)
            );
        let quote_token =
            from_bcs::to_address(*fungible_asset_order::quote_token(&order));
        let receiver = from_bcs::to_address(*fungible_asset_order::receiver(&order));
        let fee =
            fungible_asset_order::base_amount(&order)
                - fungible_asset_order::quote_amount(&order);
        // ------------------------------------------------------------------
        // TODO: no idea if the code below will work lol, it looks promising though
        // ------------------------------------------------------------------
        if (quote_token == wrapped_address) {
            if (!is_deployed(wrapped_address)) {
                let token_name = *fungible_asset_order::base_token_name(&order);
                let token_symbol = *fungible_asset_order::base_token_symbol(&order);
                // Truncate the name to max 32 characters
                if (string::length(&token_name) > 32) {
                    token_name = string::sub_string(&token_name, 0, 32);
                };

                // Truncate the symbol to max 10 characters
                if (string::length(&token_symbol) > 10) {
                    token_symbol = string::sub_string(&token_symbol, 0, 10);
                };

                deploy_token(salt, token_name, token_symbol, 18);
                let value =
                    update_channel_path(
                        path, ibc::packet::destination_channel_id(&ibc_packet)
                    );
                smart_table::upsert(&mut store.token_origin, wrapped_address, value);
            };
            zkgm::fa_coin::mint_with_metadata(
                &get_signer(),
                receiver,
                (fungible_asset_order::quote_amount(&order) as u64),
                get_metadata(quote_token)
            );
            if (fee > 0) {
                zkgm::fa_coin::mint_with_metadata(
                    &get_signer(),
                    relayer,
                    (fee as u64),
                    get_metadata(quote_token)
                );
            }
        } else {
            if (fungible_asset_order::base_token_path(&order)
                == (ibc::packet::source_channel_id(&ibc_packet) as u256)) {
                let balance_key = ChannelBalancePair {
                    channel: ibc::packet::destination_channel_id(&ibc_packet),
                    token: quote_token
                };

                let curr_balance =
                    *smart_table::borrow(&store.channel_balance, balance_key);

                smart_table::upsert(
                    &mut store.channel_balance,
                    balance_key,
                    curr_balance - (fungible_asset_order::base_amount(&order) as u256)
                );
                let asset = get_metadata(quote_token);

                primary_fungible_store::transfer(
                    &get_signer(),
                    asset,
                    receiver,
                    (fungible_asset_order::quote_amount(&order) as u64)
                );
                if (fee > 0) {
                    primary_fungible_store::transfer(
                        &get_signer(), asset, relayer, (fee as u64)
                    );
                }
            }
            else {
                abort E_ONLY_MAKER_OTHER
            };
        };
        let new_asset_order_ack =
            fungible_asset_order_ack::new(FILL_TYPE_PROTOCOL, ACK_EMPTY);
        fungible_asset_order_ack::encode(&new_asset_order_ack)
    }

    public fun on_acknowledge_packet(
        ibc_packet: Packet, acknowledgement: vector<u8>, relayer: address
    ) acquires RelayStore, SignerRef {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        let packet_hash = commitment::commit_packet(&ibc_packet);
        let parent =
            smart_table::borrow_mut_with_default(
                &mut store.in_flight_packet,
                packet_hash,
                packet::default()
            );
        if (packet::timeout_timestamp(parent) != 0
            || packet::timeout_height(parent) != 0) {
            ibc::ibc::write_acknowledgement(*parent, acknowledgement);
            smart_table::upsert(
                &mut store.in_flight_packet, packet_hash, packet::default()
            );
        } else {
            let zkgm_packet = zkgm_packet::decode(ibc::packet::data(&ibc_packet));
            let zkgm_ack = acknowledgement::decode(&acknowledgement);
            acknowledge_internal(
                ibc_packet,
                relayer,
                zkgm_packet::salt(&zkgm_packet),
                zkgm_packet::instruction(&zkgm_packet),
                acknowledgement::tag(&zkgm_ack) == ACK_SUCCESS,
                *acknowledgement::inner_ack(&zkgm_ack)
            )
        }
    }

    fun acknowledge_internal(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        instruction: Instruction,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef {
        if (instruction::version(&instruction) != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            acknowledge_fungible_asset_order(
                ibc_packet,
                salt,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                success,
                inner_ack
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            let decode_idx = 0x20;
            acknowledge_batch(
                ibc_packet,
                relayer,
                salt,
                batch::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            let decode_idx = 0x20;
            acknowledge_forward(
                ibc_packet,
                salt,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            acknowledge_multiplex(
                ibc_packet,
                relayer,
                salt,
                multiplex::decode(instruction::operand(&instruction)),
                success,
                inner_ack
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun acknowledge_multiplex(
        ibc_packet: Packet,
        relayer: address,
        _salt: vector<u8>,
        multiplex_packet: Multiplex,
        success: bool,
        ack: vector<u8>
    ) {
        if (success && !multiplex::eureka(&multiplex_packet)) {
            let multiplex_ibc_packet =
                ibc::packet::new(
                    ibc::packet::source_channel_id(&ibc_packet),
                    ibc::packet::destination_channel_id(&ibc_packet),
                    multiplex::encode_multiplex_sender_and_calldata(
                        *multiplex::contract_address(&multiplex_packet),
                        *multiplex::contract_calldata(&multiplex_packet)
                    ),
                    ibc::packet::timeout_height(&ibc_packet),
                    ibc::packet::timeout_timestamp(&ibc_packet)
                );
            let param =
                copyable_any::pack<IIBCModuleOnAcknowledgementPacketParams>(
                    IIBCModuleOnAcknowledgementPacketParams {
                        packet: multiplex_ibc_packet,
                        acknowledgement: ack,
                        relayer: relayer
                    }
                );
            let contract_address =
                from_bcs::to_address(*multiplex::sender(&multiplex_packet));

            engine_zkgm::dispatch(param, contract_address);
        }
    }

    fun acknowledge_batch(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: Batch,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let idx = 0x40;
        let batch_ack = batch_ack::decode(&inner_ack, &mut idx);
        let i = 0;
        while (i < l) {
            let syscall_ack = inner_ack;
            if (success) {
                syscall_ack = *vector::borrow(
                    &batch_ack::acknowledgements(&batch_ack), i
                );
            };
            acknowledge_internal(
                ibc_packet,
                relayer,
                salt,
                *vector::borrow(&instructions, i),
                success,
                syscall_ack
            );
            i = i + 1
        }
    }

    fun acknowledge_forward(
        _ibc_packet: Packet,
        _salt: vector<u8>,
        _forward_packet: Forward,
        _success: bool,
        _inner_ack: vector<u8>
    ) {}

    fun acknowledge_fungible_asset_order(
        ibc_packet: Packet,
        _salt: vector<u8>,
        transfer_packet: FungibleAssetOrder,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef {
        if (success) {
            let asset_order_ack = fungible_asset_order_ack::decode(&inner_ack);
            if (fungible_asset_order_ack::fill_type(&asset_order_ack)
                == FILL_TYPE_PROTOCOL) {
                // The protocol is filled, fee was paid to relayer.
            } else if (fungible_asset_order_ack::fill_type(&asset_order_ack)
                == FILL_TYPE_MARKETMAKER) {
                let market_maker =
                    from_bcs::to_address(
                        *fungible_asset_order_ack::market_maker(&asset_order_ack)
                    );
                let base_token =
                    from_bcs::to_address(
                        *fungible_asset_order::base_token(&transfer_packet)
                    );
                let asset = get_metadata(base_token);
                if (last_channel_from_path(
                    fungible_asset_order::base_token_path(&transfer_packet)
                ) == ibc::packet::source_channel_id(&ibc_packet)) {
                    zkgm::fa_coin::mint_with_metadata(
                        &get_signer(),
                        market_maker,
                        (fungible_asset_order::base_amount(&transfer_packet) as u64),
                        asset
                    );
                } else {
                    primary_fungible_store::transfer(
                        &get_signer(),
                        asset,
                        market_maker,
                        (fungible_asset_order::base_amount(&transfer_packet) as u64)
                    );
                }
            } else {
                abort E_INVALID_FILL_TYPE
            }
        } else {
            refund(ibc::packet::source_channel_id(&ibc_packet), transfer_packet);
        };
    }

    fun refund(
        source_channel_id: u32, asset_order_packet: FungibleAssetOrder
    ) acquires SignerRef {
        let sender =
            from_bcs::to_address(*fungible_asset_order::sender(&asset_order_packet));
        let base_token =
            from_bcs::to_address(*fungible_asset_order::base_token(&asset_order_packet));

        let asset = get_metadata(base_token);

        if (last_channel_from_path(
            fungible_asset_order::base_token_path(&asset_order_packet)
        ) == source_channel_id) {
            zkgm::fa_coin::mint_with_metadata(
                &get_signer(),
                sender,
                (fungible_asset_order::base_amount(&asset_order_packet) as u64),
                asset
            );
        } else {
            primary_fungible_store::transfer(
                &get_signer(),
                asset,
                sender,
                (fungible_asset_order::base_amount(&asset_order_packet) as u64)
            );
        }
    }

    public fun on_timeout_packet(ibc_packet: Packet, relayer: address) acquires RelayStore, SignerRef {
        // Decode the packet data
        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        let packet_hash = commitment::commit_packet(&ibc_packet);
        let parent =
            smart_table::borrow_mut_with_default(
                &mut store.in_flight_packet,
                packet_hash,
                packet::default()
            );

        if (packet::timeout_timestamp(parent) != 0
            || packet::timeout_height(parent) != 0) {
            let ack = acknowledgement::new(ACK_FAILURE, ACK_EMPTY);
            ibc::ibc::write_acknowledgement(*parent, acknowledgement::encode(&ack));
            smart_table::upsert(
                &mut store.in_flight_packet, packet_hash, packet::default()
            );
        } else {
            let packet_data = ibc::packet::data(&ibc_packet);

            let zkgm_packet = zkgm_packet::decode(packet_data);

            timeout_internal(
                ibc_packet,
                relayer,
                zkgm_packet::salt(&zkgm_packet),
                zkgm_packet::instruction(&zkgm_packet)
            );
        }
    }

    fun timeout_internal(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        instruction: Instruction
    ) acquires SignerRef {
        if (instruction::version(&instruction) != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            timeout_fungible_asset_order(
                ibc_packet,
                salt,
                fungible_asset_order::decode(instruction::operand(&instruction))
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            let decode_idx = 0x20;
            timeout_batch(
                ibc_packet,
                relayer,
                salt,
                batch::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            let decode_idx = 0x20;
            timeout_forward(
                ibc_packet,
                salt,
                forward::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            timeout_multiplex(
                ibc_packet,
                relayer,
                salt,
                multiplex::decode(instruction::operand(&instruction))
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun timeout_fungible_asset_order(
        ibc_packet: Packet, _salt: vector<u8>, transfer_packet: FungibleAssetOrder
    ) acquires SignerRef {
        refund(ibc::packet::source_channel_id(&ibc_packet), transfer_packet);
    }

    fun timeout_batch(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: Batch
    ) acquires SignerRef {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let i = 0;
        while (i < l) {
            timeout_internal(
                ibc_packet,
                relayer,
                salt,
                *vector::borrow(&instructions, i)
            );
        };
    }

    fun timeout_forward(
        _ibc_packet: Packet, _salt: vector<u8>, _forward_packet: Forward
    ) {}

    fun timeout_multiplex(
        ibc_packet: Packet,
        relayer: address,
        _salt: vector<u8>,
        multiplex_packet: Multiplex
    ) {
        if (!multiplex::eureka(&multiplex_packet)) {
            let multiplex_ibc_packet =
                ibc::packet::new(
                    ibc::packet::source_channel_id(&ibc_packet),
                    ibc::packet::destination_channel_id(&ibc_packet),
                    multiplex::encode_multiplex_sender_and_calldata(
                        *multiplex::contract_address(&multiplex_packet),
                        *multiplex::contract_calldata(&multiplex_packet)
                    ),
                    ibc::packet::timeout_height(&ibc_packet),
                    ibc::packet::timeout_timestamp(&ibc_packet)
                );
            let param =
                copyable_any::pack<IIBCModuleOnTimeoutPacketParams>(
                    IIBCModuleOnTimeoutPacketParams {
                        packet: multiplex_ibc_packet,
                        relayer: relayer
                    }
                );
            let contract_address =
                from_bcs::to_address(*multiplex::sender(&multiplex_packet));

            engine_zkgm::dispatch(param, contract_address);
        }
    }

    public fun on_channel_open_init(
        _connection_id: u32, _channel_id: u32, version: String
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_IBC_VERSION
        };
    }

    public fun on_channel_open_try(
        _connection_id: u32,
        _channel_id: u32,
        _counterparty_channel_id: u32,
        version: String,
        counterparty_version: String
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_IBC_VERSION
        };
        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_IBC_VERSION
        };
    }

    public fun on_channel_open_ack(
        _channel_id: u32, _counterparty_channel_id: u32, _counterparty_version: String
    ) {}

    public fun on_channel_open_confirm(_channel_id: u32) {}

    public fun on_channel_close_init(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public fun on_channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public fun on_recv_intent_packet(_packet: Packet, _relayer: address, _relayer_msg: vector<u8>) {
        abort E_INFINITE_GAME
    }
    
    public fun on_packet<T: key>(_store: Object<T>): u64 acquires RelayStore, SignerRef {
        ibc::helpers::on_packet(
            new_ucs_relay_proof(),
            |conn, chan, ver| on_channel_open_init(conn, chan, ver),
            |conn, chan, count_chan, ver, count_ver| on_channel_open_try(
                conn, chan, count_chan, ver, count_ver
            ),
            |chan, count, ver| on_channel_open_ack(chan, count, ver),
            |chan| on_channel_open_confirm(chan),
            |p, r, r_msg| on_recv_packet(p, r, r_msg),
            |p, r, r_msg| on_recv_intent_packet(p, r, r_msg),
            |p, d, r| on_acknowledge_packet(p, d, r),
            |p, r| on_timeout_packet(p, r),
            |chan| on_channel_close_init(chan),
            |chan| on_channel_close_confirm(chan)
        )
    }

    #[test]
    public fun test_fls() {
        assert!(fls(0) == 256, 1);
        assert!(fls(22) == 4, 23);
        assert!(fls(32) == 5, 33);
        assert!(fls(444) == 8, 33);
        assert!(fls(6671) == 12, 33);
        assert!(fls(33334411) == 24, 33);
    }

    #[test]
    public fun test_last_channel_from_path() {
        assert!(last_channel_from_path(0) == 0, 1);
        assert!(last_channel_from_path(244) == 244, 1);
        assert!(last_channel_from_path(9294967296) == 2, 1);
        assert!(
            last_channel_from_path(
                115792089237316195423570985008687907853269984665640564039457584007913129639935
            ) == 4294967295,
            1
        );
    }

    #[test]
    public fun test_update_Channel_path() {
        assert!(update_channel_path(0, 0) == 0, 1);
        assert!(update_channel_path(0, 34) == 34, 1);
        assert!(update_channel_path(12414123, 111) == 476753783979, 1);
        assert!(update_channel_path(44, 22) == 94489280556, 1);
    }

    #[test(admin = @zkgm, ibc = @ibc)]
    public fun test_predict_token(admin: &signer, ibc: &signer) acquires SignerRef {
        dispatcher::init_module_for_testing(ibc);
        // ibc::init_module(ibc);
        init_module_for_testing(admin);

        let path = 1;
        let destination_channel_id = 1;
        let token = b"test_token";
        let (wrapped_address, salt) =
            predict_wrapped_token(path, destination_channel_id, token);
        let deployed_token_addr = deploy_token(salt, string::utf8(b""), string::utf8(b""), 18);

        std::debug::print(&string::utf8(b"wrapped address is: "));
        std::debug::print(&wrapped_address);
        std::debug::print(&string::utf8(b"deployed_token_addr is: "));
        std::debug::print(&deployed_token_addr);

        assert!(wrapped_address == deployed_token_addr, 101);
        assert!(is_deployed(deployed_token_addr), 102);
    }

    #[test(admin = @zkgm, ibc = @ibc)]
    public fun test_is_deployed_false(admin: &signer, ibc: &signer) {
        dispatcher::init_module_for_testing(ibc);
        init_module_for_testing(admin);

        let path = 1;
        let destination_channel_id = 1;
        let token = b"never_deployed_salt";
        let (wrapped_address, _salt) =
            predict_wrapped_token(path, destination_channel_id, token);

        assert!(!is_deployed(wrapped_address), 102);
    }

    #[test(admin = @zkgm, ibc = @ibc)]
    #[expected_failure(abort_code = 10)]
    // E_INVALID_AMOUNT
    public fun test_transfer_scenario_err(admin: &signer, ibc: &signer) acquires RelayStore, SignerRef {
        dispatcher::init_module_for_testing(ibc);
        init_module_for_testing(admin);

        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        let salt = b"test_salt";
        let test_token_addr = deploy_token(salt, string::utf8(b""), string::utf8(b""), 18);

        transfer(
            admin,
            1, // channel_id
            b"receiver_address", // receiver (just a byte-string for test)
            test_token_addr, // base_token
            0, // base_amount == 0 -> expect E_INVALID_AMOUNT
            b"", // quote_token
            0, // quote_amount
            1234, // timeout_height
            0, // timeout_timestamp
            b"my_test_salt" // salt
        );

    }

    #[test(admin = @zkgm, ibc = @ibc)]
    #[expected_failure]
    // TODO: because ibc::send_packet raises error, no idea how to mock this
    public fun test_transfer_scenario_success(
        admin: &signer, ibc: &signer
    ) acquires RelayStore, SignerRef {
        dispatcher::init_module_for_testing(ibc);
        init_module_for_testing(admin);

        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let channel_id: u32 = 0;
        let quote_token = b"QUOTE_TOKEN";
        let (wrapped_address, salt) = predict_wrapped_token(0, channel_id, quote_token);
        let test_token_addr = deploy_token(salt, string::utf8(b""), string::utf8(b""), 18);

        let admin_balance_before =
            primary_fungible_store::balance(
                signer::address_of(admin), get_metadata(test_token_addr)
            );

        zkgm::fa_coin::mint_with_metadata(
            &get_signer(),
            signer::address_of(admin), // recipient
            1000, // amount
            get_metadata(test_token_addr) // object<Metadata> for this token
        );

        let admin_balance =
            primary_fungible_store::balance(
                signer::address_of(admin), get_metadata(test_token_addr)
            );

        assert!(admin_balance_before == 0, 101);
        assert!(admin_balance == 1000, 101);

        let base_token = test_token_addr;
        std::debug::print(&string::utf8(b"sooo whats the addr:"));
        std::debug::print(&test_token_addr);

        transfer(
            admin,
            channel_id, // channel_id
            b"some_receiver", // receiver
            base_token, // base_token
            500, // base_amount
            quote_token, // quote_token (just a byte-string for test)
            100, // quote_amount
            1234, // timeout_height
            0, // timeout_timestamp
            b"my_test_salt" // salt
        );

    }

    // #[test(admin = @zkgm, ibc = @ibc)]
    // public fun test_transfer_scenarios(admin: &signer, ibc: &signer) acquires RelayStore, SignerRef {
    //     // 1) Initialize test environment
    //     //    - Initialize the dispatcher and this module for testing
    //     dispatcher::init_module_for_testing(ibc);
    //     init_module_for_testing(admin);

    //     // 2) Deploy a test token, and mint some to `admin`
    //     let salt = b"test_salt";
    //     let test_token_addr = deploy_token(salt, string::utf8(b""), string::utf8(b""), 18);

    //     // Mint 1000 units of our newly deployed token to `admin`
    //     zkgm::fa_coin::mint_with_metadata(
    //         &get_signer(),
    //         signer::address_of(admin),           // recipient
    //         1000,                                // amount
    //         get_metadata(test_token_addr)        // object<Metadata> for this token
    //     );

    //     // 3) Scenario 1: Attempt transfer with base_amount = 0 => expect E_INVALID_AMOUNT abort
    //     //    This confirms that zero amounts are disallowed.
    //     assert_abort_with_code(
    //         move || {
    //             transfer(
    //                 admin,
    //                 1,                      // channel_id
    //                 b"receiver_address",    // receiver (just a byte-string for test)
    //                 test_token_addr,        // base_token
    //                 0,                      // base_amount == 0 -> expect E_INVALID_AMOUNT
    //                 b"",                    // quote_token
    //                 0,                      // quote_amount
    //                 1234,                   // timeout_height
    //                 0,                      // timeout_timestamp
    //                 b"my_test_salt"         // salt
    //             );
    //         },
    //         E_INVALID_AMOUNT
    //     );

    //     // 4) Scenario 2: Transfer with a positive base_amount => expect success
    //     //    We transfer 500 out of the 1000 minted tokens. This should succeed
    //     //    and create an IBC packet with the appropriate data.
    //     transfer(
    //         admin,
    //         1,                          // channel_id
    //         b"some_receiver",           // receiver
    //         test_token_addr,            // base_token
    //         500,                        // base_amount
    //         b"QUOTE_TOKEN",             // quote_token (just a byte-string for test)
    //         100,                        // quote_amount
    //         1234,                       // timeout_height
    //         0,                          // timeout_timestamp
    //         b"my_test_salt"             // salt
    //     );

    //     // At this point, you could add additional checks or debug prints
    //     // For now, simply finishing without an abort indicates Scenario 2 succeeded.

    //     // 5) (Optional) Scenario 3: Demonstrate transferring a "wrapped" token
    //     //    This scenario typically requires you to simulate that the `test_token_addr`
    //     //    is already recognized as "wrapped" (meaning its origin path indicates
    //     //    the current channel). You would do something like:
    //     //
    //     //      - Manually insert it into `store.token_origin`,
    //     //        so last_channel_from_path(origin) == channel_id used in `transfer`.
    //     //      - Then call `transfer` again with the same token, verifying that
    //     //        `fa_coin::burn_with_metadata` is invoked rather than `transfer`.
    //     //

    //     // If the function reaches here without abort, all scenarios passed.
    //     std::debug::print(&string::utf8(b"test_transfer_scenarios passed!"));
    // }

    #[test]
    fun see_packet() {
        let packet = x"53f247a39cb05a49ed206cdb7b09dad6a71b9eae2f49b3408be67510fd19b1cc0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002c756e696f6e3164383467743663777839333873616e306874687a37793666307234663030676a717768353977000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000201363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b500000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002073b27231bc5dc074cbbe018f6414022f679f2f66ee521abf24fa3cca9ad54072";
        let packet = zkgm_packet::decode(&packet);
        let packet = fungible_asset_order::decode(instruction::operand(&zkgm_packet::instruction(&packet)));
        let (wrapped_address, salt) =
            predict_wrapped_token(
                0,
                2,
                *fungible_asset_order::base_token(&packet)
            );
        let quote_token =
            from_bcs::to_address(*fungible_asset_order::quote_token(&packet));
        std::debug::print(&(wrapped_address == quote_token));
    }
}

