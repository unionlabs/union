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
    use zkgm::lib;

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
    const INSTR_VERSION_0: u8 = 0x00;
    const INSTR_VERSION_1: u8 = 0x01;

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
    const E_INVALID_BATCH_INSTRUCTION: u64 = 17;
    const E_INVALID_FORWARD_INSTRUCTION: u64 = 18;
    const E_INVALID_MULTIPLEX_SENDER: u64 = 19;
    const E_INVALID_FORWARD_DESTINATION_CHANNEL_ID: u64 = 20;

    struct IbcAppWitness has drop, store, key {}

    public(friend) fun witness(): IbcAppWitness {
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

    public entry fun call(
        sender: &signer,
        channel_id: u32,
        contract_address: address,
        contract_calldata: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>
    ) {
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
                instruction::new(INSTR_VERSION_0, OP_MULTIPLEX, operand)
            );
        ibc::ibc::send_packet(
            witness(),
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
            witness(),
            channel_id,
            timeout_height,
            timeout_timestamp,
            zkgm_packet::encode(&zkgm_pack)
        );
    }

    #[view]
    public fun get_metadata(asset_addr: address): Object<Metadata> {
        object::address_to_object<Metadata>(asset_addr)
    }

    #[view]
    public fun get_balance(acc: address, token: address): u64 {
        primary_fungible_store::balance(acc, get_metadata(token))
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@zkgm, IBC_APP_SEED)
    }

    #[view]
    public fun predict_wrapped_token(
        path: u256, destination_channel_id: u32, token: vector<u8>
    ): (address, vector<u8>) {
        let salt = hash::sha3_256(serialize_salt(path, destination_channel_id, token));

        let wrapped_address = object::create_object_address(&get_vault_addr(), salt);
        (wrapped_address, salt)
    }

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

        ibc::register_application<IbcAppWitness>(account, cb, witness());
    }

    fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
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

    fun deploy_token(
        salt: vector<u8>,
        name: string::String,
        symbol: string::String,
        decimals: u8
    ): address acquires SignerRef {
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

    fun is_deployed(token: address): bool {
        object::is_object(token)
    }

    fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }

    fun verify_internal(
        sender: &signer,
        channel_id: u32,
        path: u256,
        instruction: Instruction
    ) acquires RelayStore, SignerRef {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            verify_fungible_asset_order(
                sender,
                channel_id,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction))
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            verify_batch(
                sender,
                channel_id,
                path,
                batch::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            verify_forward(
                sender,
                channel_id,
                path,
                forward::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
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
            let instruction = *vector::borrow(&instructions, i);
            if (lib::is_allowed_batch_instruction(instruction::opcode(&instruction))) {
                abort E_INVALID_BATCH_INSTRUCTION
            };
            verify_internal(sender, channel_id, path, instruction);
            i = i + 1;
        }
    }

    fun verify_forward(
        sender: &signer,
        channel_id: u32,
        path: u256,
        forward_packet: Forward
    ) acquires RelayStore, SignerRef {
        let instruction = *forward::instruction(&forward_packet);
        if (lib::is_allowed_forward_instruction(instruction::opcode(&instruction))) {
            abort E_INVALID_FORWARD_INSTRUCTION
        };
        verify_internal(
            sender,
            channel_id,
            forward::path(&forward_packet),
            instruction
        );
    }

    fun verify_multiplex(
        sender: &signer,
        _channel_id: u32,
        _path: u256,
        multiplex_packet: Multiplex
    ) {
        if (from_bcs::to_address(*multiplex::sender(&multiplex_packet))
            != signer::address_of(sender)) {
            abort E_INVALID_MULTIPLEX_SENDER
        };
    }

    fun verify_fungible_asset_order(
        sender: &signer,
        channel_id: u32,
        path: u256,
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

        let (intermediate_channel_path, destination_channel_id) =
            lib::pop_channel_from_path(origin);
        let (wrapped_token, _) =
            predict_wrapped_token(
                intermediate_channel_path,
                channel_id,
                *fungible_asset_order::quote_token(&order)
            );
        let is_inverse_intermediate_path =
            path == lib::reverse_channel_path(intermediate_channel_path);
        let is_sending_back_to_same_channel = destination_channel_id == channel_id;
        let is_unwrapping = base_token == wrapped_token;

        if (is_inverse_intermediate_path
            && is_sending_back_to_same_channel
            && is_unwrapping) {
            if (fungible_asset_order::base_token_path(&order) != origin) {
                abort E_INVALID_ASSET_ORIGIN
            };

            zkgm::fa_coin::burn_with_metadata(
                &get_signer(),
                signer::address_of(sender),
                (base_amount as u64),
                asset
            );
        } else {
            if (fungible_asset_order::base_token_path(&order) != 0) {
                abort E_INVALID_ASSET_ORIGIN
            };

            primary_fungible_store::transfer(
                sender,
                asset,
                signer::address_of(&get_signer()),
                (base_amount as u64)
            );

            increase_outstanding(
                store,
                channel_id,
                path,
                base_token,
                base_amount
            );
        }
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
            dispatcher::set_return_value<IbcAppWitness>(witness(), return_value);
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
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            execute_fungible_asset_order(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction))
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
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
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            execute_forward(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                instruction::version(&instruction),
                forward::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            execute_multiplex(
                ibc_packet,
                relayer,
                relayer_msg,
                path,
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
        relayer: address,
        _relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        version: u8,
        forward_packet: Forward
    ): (vector<u8>) acquires RelayStore {
        let (tail_path, previous_destination_channel_id) =
            lib::dequeue_channel_from_path(forward::path(&forward_packet));
        let (continuation_path, next_source_channel_id) =
            lib::dequeue_channel_from_path(tail_path);

        if (packet::destination_channel_id(&ibc_packet)
            != previous_destination_channel_id) {
            abort E_INVALID_FORWARD_DESTINATION_CHANNEL_ID
        };

        let next_instruction =
            if (continuation_path == 0) {
                *forward::instruction(&forward_packet)
            } else {
                instruction::new(
                    version,
                    OP_FORWARD,
                    forward::encode(
                        &forward::new(
                            continuation_path,
                            forward::timeout_height(&forward_packet),
                            forward::timeout_timestamp(&forward_packet),
                            *forward::instruction(&forward_packet)
                        )
                    )
                )
            };

        let sent_packet =
            ibc::ibc::send_packet(
                witness(),
                next_source_channel_id,
                forward::timeout_height(&forward_packet),
                forward::timeout_timestamp(&forward_packet),
                zkgm_packet::encode(
                    &zkgm_packet::new(
                        lib::derive_forward_salt(salt),
                        lib::update_channel_path(
                            lib::update_channel_path(
                                path, previous_destination_channel_id
                            ),
                            next_source_channel_id
                        ),
                        next_instruction
                    )
                )
            );

        let commitment_key =
            commitment::batch_packets_commitment_key(
                commitment::commit_packet(&sent_packet)
            );
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.in_flight_packet, commitment_key, ibc_packet);

        ACK_EMPTY
    }

    fun execute_multiplex(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        path: u256,
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
                multiplex::encode_calldata(
                    path,
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
        if (quote_token == wrapped_address) {
            if (!is_deployed(wrapped_address)) {
                let token_name = *fungible_asset_order::base_token_name(&order);
                let token_symbol = *fungible_asset_order::base_token_symbol(&order);
                deploy_token(
                    salt,
                    token_name,
                    token_symbol,
                    fungible_asset_order::base_token_decimals(&order)
                );
                let value =
                    lib::update_channel_path(
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
            } else {
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
            ibc::ibc::write_acknowledgement(witness(), *parent, acknowledgement);
            smart_table::upsert(
                &mut store.in_flight_packet, packet_hash, packet::default()
            );
        } else {
            let zkgm_packet = zkgm_packet::decode(ibc::packet::data(&ibc_packet));
            let zkgm_ack = acknowledgement::decode(&acknowledgement);
            acknowledge_internal(
                ibc_packet,
                relayer,
                zkgm_packet::path(&zkgm_packet),
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
        path: u256,
        salt: vector<u8>,
        instruction: Instruction,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef, RelayStore {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            acknowledge_fungible_asset_order(
                ibc_packet,
                path,
                salt,
                fungible_asset_order::decode(instruction::operand(&instruction)),
                success,
                inner_ack
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            acknowledge_batch(
                ibc_packet,
                relayer,
                path,
                salt,
                batch::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            acknowledge_forward(
                ibc_packet,
                relayer,
                salt,
                forward::decode(instruction::operand(&instruction), &mut decode_idx),
                success,
                inner_ack
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            acknowledge_multiplex(
                ibc_packet,
                relayer,
                path,
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
        path: u256,
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
                    multiplex::encode_calldata(
                        path,
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
        path: u256,
        salt: vector<u8>,
        batch_packet: Batch,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef, RelayStore {
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
                path,
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
        _relayer: address,
        _salt: vector<u8>,
        _forward_packet: Forward,
        _success: bool,
        _inner_ack: vector<u8>
    ) {}

    fun acknowledge_fungible_asset_order(
        ibc_packet: Packet,
        path: u256,
        _salt: vector<u8>,
        transfer_packet: FungibleAssetOrder,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef, RelayStore {
        if (success) {
            let base_token =
                from_bcs::to_address(
                    *fungible_asset_order::base_token(&transfer_packet)
                );
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
                let asset = get_metadata(base_token);
                if (fungible_asset_order::base_token_path(&transfer_packet) != 0) {
                    zkgm::fa_coin::mint_with_metadata(
                        &get_signer(),
                        market_maker,
                        (fungible_asset_order::base_amount(&transfer_packet) as u64),
                        asset
                    );

                } else {
                    decrease_outstanding(
                        borrow_global_mut<RelayStore>(get_vault_addr()),
                        packet::source_channel_id(&ibc_packet),
                        fungible_asset_order::base_token_path(&transfer_packet),
                        base_token,
                        fungible_asset_order::base_amount(&transfer_packet)
                    );
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
            refund(ibc::packet::source_channel_id(&ibc_packet), path, transfer_packet);
        };
    }

    fun refund(
        source_channel_id: u32, path: u256, asset_order_packet: FungibleAssetOrder
    ) acquires SignerRef, RelayStore {
        let sender =
            from_bcs::to_address(*fungible_asset_order::sender(&asset_order_packet));
        let base_token =
            from_bcs::to_address(*fungible_asset_order::base_token(&asset_order_packet));

        let asset = get_metadata(base_token);

        if (fungible_asset_order::base_token_path(&asset_order_packet) != 0) {
            zkgm::fa_coin::mint_with_metadata(
                &get_signer(),
                sender,
                (fungible_asset_order::base_amount(&asset_order_packet) as u64),
                asset
            );
        } else {
            let base_amount = fungible_asset_order::base_amount(&asset_order_packet);
            decrease_outstanding(
                borrow_global_mut<RelayStore>(get_vault_addr()),
                source_channel_id,
                path,
                base_token,
                base_amount
            );
            primary_fungible_store::transfer(
                &get_signer(),
                asset,
                sender,
                ((base_amount) as u64)
            );
        }
    }

    public fun on_timeout_packet(ibc_packet: Packet, relayer: address) acquires RelayStore, SignerRef {
        // Decode the packet data
        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        let zkgm_packet = zkgm_packet::decode(packet::data(&ibc_packet));

        if (lib::is_forwarded_packet(zkgm_packet::salt(&zkgm_packet))) {
            let packet_hash = commitment::commit_packet(&ibc_packet);
            let parent =
                smart_table::borrow_mut_with_default(
                    &mut store.in_flight_packet,
                    packet_hash,
                    packet::default()
                );
            if (packet::timeout_timestamp(parent) != 0
                || packet::timeout_height(parent) != 0) {
                return;
            }
        };

        timeout_internal(
            ibc_packet,
            relayer,
            zkgm_packet::path(&zkgm_packet),
            zkgm_packet::instruction(&zkgm_packet)
        );
    }

    fun timeout_internal(
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        instruction: Instruction
    ) acquires SignerRef, RelayStore {
        let version = instruction::version(&instruction);
        if (instruction::opcode(&instruction) == OP_FUNGIBLE_ASSET_ORDER) {
            assert!(version == INSTR_VERSION_1, E_UNSUPPORTED_VERSION);
            timeout_fungible_asset_order(
                ibc_packet,
                path,
                fungible_asset_order::decode(instruction::operand(&instruction))
            )
        } else if (instruction::opcode(&instruction) == OP_BATCH) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            timeout_batch(
                ibc_packet,
                relayer,
                path,
                batch::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_FORWARD) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            let decode_idx = 0x20;
            timeout_forward(
                ibc_packet,
                relayer,
                forward::decode(instruction::operand(&instruction), &mut decode_idx)
            )
        } else if (instruction::opcode(&instruction) == OP_MULTIPLEX) {
            assert!(version == INSTR_VERSION_0, E_UNSUPPORTED_VERSION);
            timeout_multiplex(
                ibc_packet,
                relayer,
                path,
                multiplex::decode(instruction::operand(&instruction))
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun timeout_fungible_asset_order(
        ibc_packet: Packet, path: u256, transfer_packet: FungibleAssetOrder
    ) acquires SignerRef, RelayStore {
        refund(ibc::packet::source_channel_id(&ibc_packet), path, transfer_packet);
    }

    fun timeout_batch(
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        batch_packet: Batch
    ) acquires SignerRef, RelayStore {
        let instructions = batch::instructions(&batch_packet);
        let l = vector::length(&instructions);
        let i = 0;
        while (i < l) {
            timeout_internal(
                ibc_packet,
                relayer,
                path,
                *vector::borrow(&instructions, i)
            );
        };
    }

    fun timeout_forward(
        ibc_packet: Packet, relayer: address, forward_packet: Forward
    ) acquires SignerRef, RelayStore {
        timeout_internal(
            ibc_packet,
            relayer,
            forward::path(&forward_packet),
            *forward::instruction(&forward_packet)
        );
    }

    fun timeout_multiplex(
        ibc_packet: Packet,
        relayer: address,
        path: u256,
        multiplex_packet: Multiplex
    ) {
        if (!multiplex::eureka(&multiplex_packet)) {
            let multiplex_ibc_packet =
                ibc::packet::new(
                    ibc::packet::source_channel_id(&ibc_packet),
                    ibc::packet::destination_channel_id(&ibc_packet),
                    multiplex::encode_calldata(
                        path,
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

    fun on_channel_open_init(
        _connection_id: u32, _channel_id: u32, version: String
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_IBC_VERSION
        };
    }

    fun on_channel_open_try(
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

    fun on_channel_open_ack(
        _channel_id: u32, _counterparty_channel_id: u32, _counterparty_version: String
    ) {}

    fun on_channel_open_confirm(_channel_id: u32) {}

    fun on_channel_close_init(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    fun on_channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    fun on_recv_intent_packet(
        _packet: Packet, _relayer: address, _relayer_msg: vector<u8>
    ) {
        abort E_INFINITE_GAME
    }

    fun increase_outstanding(
        store: &mut RelayStore,
        source_channel_id: u32,
        path: u256,
        token: address,
        amount: u256
    ) {
        let balance_key = ChannelBalancePair { channel: source_channel_id, token };

        let curr_balance = *smart_table::borrow(&store.channel_balance, balance_key);

        smart_table::upsert(
            &mut store.channel_balance,
            balance_key,
            curr_balance + (amount as u256)
        );
    }

    fun decrease_outstanding(
        store: &mut RelayStore,
        source_channel_id: u32,
        path: u256,
        token: address,
        amount: u256
    ) {
        let balance_key = ChannelBalancePair { channel: source_channel_id, token };

        let curr_balance = *smart_table::borrow(&store.channel_balance, balance_key);

        smart_table::upsert(
            &mut store.channel_balance,
            balance_key,
            curr_balance - (amount as u256)
        );
    }

    public fun on_packet<T: key>(_store: Object<T>): u64 acquires RelayStore, SignerRef {
        ibc::helpers::on_packet(
            witness(),
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

    #[test_only]
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
        let deployed_token_addr =
            deploy_token(salt, string::utf8(b""), string::utf8(b""), 18);

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
        let packet =
            x"82c1e7c9642e7ecbb7bbe659eff187e8ee6691fd7c840b09a89ec6b126c8ca3b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000000c800000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000000c80000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000204d8a66ece11f6352224942bd1dabc456b4bb5316124f02b9a7b6292ad61f77770000000000000000000000000000000000000000000000000000000000000040756e696f6e31677968347464377639366d7563723465616b7364326d7367306a76306d636e396135796a38357678356c376874793374753970737178736a79320000000000000000000000000000000000000000000000000000000000000004414e414d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000963616e696d616e616d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020feec3232793d275bfc623cc14f7306904b080746752fefce94e87dfe0bcf4962";
        let raw_packet = zkgm_packet::decode(&packet);
        std::debug::print(&raw_packet);
        let packet =
            fungible_asset_order::decode(
                instruction::operand(&zkgm_packet::instruction(&raw_packet))
            );
        std::debug::print(&packet);
    }
}
