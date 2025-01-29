module zkgm::zkgm_relay {
    use zkgm::zkgm_helpers;
    use zkgm::ethabi;
    use zkgm::dispatcher_zkgm;
    use zkgm::engine_zkgm;
    use zkgm::instruction::{Instruction, encode as encode_instruction, decode as decode_instruction};

    use ibc::ibc;
    use ibc::helpers;
    use ibc::packet::{Self, Packet};
    use ibc::dispatcher;
    use ibc::commitment;

    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use aptos_std::copyable_any;
    use aptos_framework::function_info;
    use aptos_framework::function_info::FunctionInfo;

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
    const VERSION: vector<u8> = b"zkgm-zkgm-0";
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
    const E_ONLY_MAKER: u64 = 15;

    struct ZKGMProof has drop, store, key {}

    public(friend) fun new_ucs_relay_proof(): ZKGMProof {
        ZKGMProof {}
    }

    struct ZkgmPacket has copy, drop, store {
        salt: vector<u8>,
        path: u256,
        instruction: Instruction
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

    struct Port<phantom T: key + store + drop> has key, copy, drop, store {
        port_id: address
    }

    public fun get_metadata(asset_addr: address): Object<Metadata> {
        object::address_to_object<Metadata>(asset_addr)
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
                string::utf8(b"zkgm_relay"),
                string::utf8(b"on_packet")
            );

        ibc::register_application<ZKGMProof>(account, cb, new_ucs_relay_proof());
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
        path: u256, destination_channel: u32, token: vector<u8>
    ): vector<u8> {
        let data = vector::empty<u8>();
        vector::append(&mut data, bcs::to_bytes(&path));
        vector::append(&mut data, bcs::to_bytes(&destination_channel));
        vector::append(&mut data, token);
        data
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

    public fun last_channel_from_path(path: u256): u32 {
        if (path == 0) {
            return 0
        };
        let current_hop_index = ((fls(path) / 32) as u8);
        let last_channel = path >> (current_hop_index * 32);
        (last_channel as u32)
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

    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }

    public entry fun transfer() {
        // TODO: fill this
    }

    public entry fun call() {
        // TODO: fill this
    }

    // public entry fun send(
    //     sender: &signer,
    //     channel_id: u32,
    //     timeout_height: u64,
    //     timeout_timestamp: u64,
    //     salt: vector<u8>,
    //     version: u8,
    //     opcode: u8,
    //     operand: vector<u8>
    // ) acquires SignerRef, RelayStore {
    //     let instruction = Instruction { version: version, opcode: opcode, operand: operand };
    //     verify_internal(sender, channel_id, 0, instruction);
    //     ibc::ibc::send_packet(
    //         &get_signer(),
    //         get_self_address(),
    //         channel_id,
    //         timeout_height,
    //         timeout_timestamp,
    //         encode_packet(&ZkgmPacket { salt: salt, path: 0, instruction: instruction })
    //     );
    // }

    // fun verify_internal(
    //     sender: &signer,
    //     channel_id: u32,
    //     path: u256,
    //     instruction: Instruction
    // ) acquires RelayStore, SignerRef {
    //     if (instruction.version != ZKGM_VERSION_0) {
    //         abort E_UNSUPPORTED_VERSION
    //     };
    //     if (instruction.opcode == OP_FUNGIBLE_ASSET_ORDER) {
    //         verify_fungible_asset_transfer(
    //             sender,
    //             channel_id,
    //             path,
    //             decode_fungible_asset_order(instruction.operand)
    //         )
    //     } else if (instruction.opcode == OP_BATCH) {
    //         verify_batch(
    //             sender,
    //             channel_id,
    //             path,
    //             decode_batch_packet(instruction.operand)
    //         )
    //     } else if (instruction.opcode == SYSCALL_FORWARD) {
    //         verify_forward(
    //             sender,
    //             channel_id,
    //             path,
    //             decode_forward(instruction.operand)
    //         )
    //     } else if (instruction.opcode == SYSCALL_MULTIPLEX) {
    //         verify_multiplex(
    //             sender,
    //             channel_id,
    //             path,
    //             decode_multiplex(instruction.operand)
    //         )
    //     } else {
    //         abort E_UNKNOWN_SYSCALL
    //     }
    // }

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

}