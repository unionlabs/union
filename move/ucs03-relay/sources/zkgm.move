module ibc::zkgm {
    use ibc::ibc;
    use ibc::helpers;
    use ibc::packet::{Self, Packet};
    use ibc::dispatcher;
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use aptos_std::copyable_any;
    use std::event;
    use aptos_framework::function_info;
    use ibc::commitment;

    use std::string::{Self, String};
    use std::string_utils;
    use std::from_bcs;
    use std::bcs;
    use aptos_framework::hash;
    use aptos_framework::account;


    use aptos_framework::fungible_asset::{Metadata};
    use aptos_framework::signer;
    use aptos_std::smart_table::{Self, SmartTable};
    use std::vector;
    use ucs03::ethabi;

    // Constants
    const ORDER_UNORDERED: u8 = 1;
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const ACK_SUCCESS: u256 = 1;
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;
    const ZKGM_VERSION_0: u8 = 0x00;
    const SYSCALL_FORWARD: u8 = 0x00;
    const SYSCALL_MULTIPLEX: u8 = 0x01;
    const SYSCALL_BATCH: u8 = 0x02;
    const SYSCALL_FUNGIBLE_ASSET_TRANSFER: u8 = 0x03;
    use std::option;

    // Errors
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const E_UNAUTHORIZED: u64 = 1;
    const E_INVALID_HOPS: u64 = 2;
    const E_INVALID_IBC_VERSION: u64 = 3;
    const E_INFINITE_GAME: u64 = 4;
    const E_UNSUPPORTED_VERSION: u64 = 5;
    const E_UNKNOWN_SYSCALL: u64 = 6;

    struct ZKGMProof has drop, store, key {}

    public(friend) fun new_ucs_relay_proof(): ZKGMProof {
        ZKGMProof {}
    }

    struct ZkgmPacket has copy, drop, store {
        salt: vector<u8>,
        path: u64,
        syscall: vector<u8>,
    }

    struct SyscallPacket has copy, drop, store {
        version: u8,
        index: u8,
        packet: vector<u8>,
    }

    struct ForwardPacket has copy, drop, store {
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        syscall_packet: vector<u8>,
    }

    struct MultiplexPacket has copy, drop, store {
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>,
    }

    struct BatchPacket has copy, drop, store {
        syscall_packets: vector<vector<u8>>,
    }

    struct FungibleAssetTransferPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        sent_token: vector<u8>,
        sent_token_prefix: u64,
        sent_symbol: vector<u8>,
        sent_name: vector<u8>,
        sent_amount: u64,
        ask_token: vector<u8>,
        ask_amount: u64,
        only_maker: bool,
    }

    struct Acknowledgement has copy, drop, store {
        tag: u256,
        inner_ack: vector<u8>,
    }

    struct BatchAcknowledgement has copy, drop, store {
        acknowledgements: vector<vector<u8>>,
    }

    struct AssetTransferAcknowledgement has copy, drop, store {
        fill_type: u64,
        market_maker: vector<u8>,
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

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@ucs03, IBC_APP_SEED)
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
        assert!(signer::address_of(account) == @ucs03, E_UNAUTHORIZED);

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

        // let cb =
        //     function_info::new_function_info(
        //         account,
        //         string::utf8(b"relay_app"),
        //         string::utf8(b"on_packet")
        //     );

        // ibc::register_application<ZKGMProof>(account, cb, new_ucs_relay_proof());
    }

    public fun decode_ack(buf: vector<u8>): Acknowledgement {
        // TODO: Implement this
        Acknowledgement {
            tag: 0,
            inner_ack: vector::empty()
        }
    }


    public fun decode_packet(buf: vector<u8>): ZkgmPacket {
        // TODO: Implement this
        ZkgmPacket {
            salt: vector::empty(),
            path: 0,
            syscall: vector::empty()
        }
    }

    public fun decode_syscall(buf: vector<u8>): SyscallPacket {
        // TODO: Implement this
        SyscallPacket {
            version: 0,
            index: 0,
            packet: vector::empty()
        }
    }

    public fun decode_forward(buf: vector<u8>): ForwardPacket {
        // TODO Implement this
        ForwardPacket {
            channel_id: 0,
            timeout_height: 0,
            timeout_timestamp: 0,
            syscall_packet: vector::empty()
        }
    }

    public fun decode_multiplex(buf: vector<u8>): MultiplexPacket {
        // TODO Implement this
        MultiplexPacket {
            sender: vector::empty(),
            eureka: false,
            contract_address: vector::empty(),
            contract_calldata: vector::empty()
        }
    }

    public fun decode_batch(buf: vector<u8>): BatchPacket {
        // TODO Implement this
        BatchPacket {
            syscall_packets: vector::empty()
        }
    }

    public fun decode_fungible_asset_transfer(buf: vector<u8>): FungibleAssetTransferPacket {
        // TODO Implement this
        FungibleAssetTransferPacket {
            sender: vector::empty(),
            receiver: vector::empty(),
            sent_token: vector::empty(),
            sent_token_prefix: 0,
            sent_symbol: vector::empty(),
            sent_name: vector::empty(),
            sent_amount: 0,
            ask_token: vector::empty(),
            ask_amount: 0,
            only_maker: false
        }
    }

    public fun encode_packet(packet: &ZkgmPacket): vector<u8> {
        let buf = vector::empty<u8>();
        buf
    }

    public fun predict_wrapped_token(
        path: u64,
        destination_channel: u32,
        token: vector<u8>
    ): (address, vector<u8>) {
        let salt = hash::sha3_256(serialize_salt(path, destination_channel, token));

        let wrapped_address = object::create_object_address(&get_vault_addr(), salt);
        (wrapped_address, salt)
    }

    public fun deploy_token(
        path: u64,
        destination_channel: u32,
        token: vector<u8>
    ): address acquires SignerRef {
        let salt = hash::sha3_256(serialize_salt(path, destination_channel, token));

        ucs03::fa_coin::initialize(
            &get_signer(),
            string::utf8(b""),
            string::utf8(b""),
            18,
            string::utf8(b""),
            string::utf8(b""),
            salt
        );
        ucs03::fa_coin::get_metadata_address(
            salt
        )
    }

    fun serialize_salt(
        path: u64,
        destination_channel: u32,
        token: vector<u8>
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
            return 256;
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
        if (path == 0){
            return 0;
        };
        let current_hop_index = ((fls(path) / 32) as u8);
        let last_channel = path >> (current_hop_index * 32);
        (last_channel as u32)
    }

    public fun update_channel_path(path: u256, next_channel_id: u32): u256 {
        if (path == 0){
            return (next_channel_id as u256);
        };
        let next_hop_index = ((fls(path) / 32) as u8) + 1;
        if (next_hop_index > 7){
            abort E_INVALID_HOPS;
        };

        let next_channel = (((next_channel_id as u256) << (next_hop_index * 32)) as u256) | path;
        (next_channel as u256)
    }

    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }


    public fun on_channel_open_init(
        connection_id: u32,
        channel_id: u32,
        version: String
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
        _channel_id: u32, _counterparty_channel_id: u32, counterparty_version: String
    ) { }

    public fun on_channel_open_confirm(_channel_id: u32) {}

    public fun on_channel_close_init(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public fun on_channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public fun on_acknowledge_packet(
        ibc_packet: Packet, acknowledgement: vector<u8>
    ) acquires RelayStore{
        let store = borrow_global_mut<RelayStore>(get_vault_addr());

        let packet_hash = commitment::commit_packet(&ibc_packet);
        let parent = smart_table::borrow_mut_with_default(
                        &mut store.in_flight_packet,
                        packet_hash,
                        packet::default()
                    );
        if (packet::timeout_timestamp(parent) != 0 || packet::timeout_height(parent) != 0) {
            ibc::ibc::write_acknowledgement(*parent, acknowledgement);
            smart_table::upsert(&mut store.in_flight_packet, packet_hash, packet::default());
        } else {
            let zkgm_packet = decode_packet(*ibc::packet::data(&ibc_packet));
            let zkgm_ack = decode_ack(acknowledgement);
            acknowledge_internal(
                ibc_packet,
                zkgm_packet.salt,
                decode_syscall(zkgm_packet.syscall),
                zkgm_ack.tag == ACK_SUCCESS,
                zkgm_ack.inner_ack
            )
        }
    }

    fun acknowledge_internal(
        ibc_packet: Packet,
        salt: vector<u8>,
        syscall_packet: SyscallPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {
        if (syscall_packet.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };

        if (syscall_packet.index == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            acknowledge_fungible_asset_transfer(
                ibc_packet,
                salt,
                decode_fungible_asset_transfer(syscall_packet.packet),
                success,
                inner_ack
            );
        } else if (syscall_packet.index == SYSCALL_BATCH) {
            acknowledge_batch(
                ibc_packet,
                salt,
                decode_batch(syscall_packet.packet),
                success,
                inner_ack
            );
        } else if (syscall_packet.index == SYSCALL_FORWARD) {
            acknowledge_forward(
                ibc_packet,
                salt,
                decode_forward(syscall_packet.packet),
                success,
                inner_ack
            );
        } else if (syscall_packet.index == SYSCALL_MULTIPLEX) {
            acknowledge_multiplex(
                ibc_packet,
                salt,
                decode_multiplex(syscall_packet.packet),
                success,
                inner_ack
            );
        } else {
            abort E_UNKNOWN_SYSCALL;
        }
    }

    fun acknowledge_fungible_asset_transfer(
        ibc_packet: Packet,
        salt: vector<u8>,
        transfer_packet: FungibleAssetTransferPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {
        // TODO: implement this
    }

    fun acknowledge_batch(
        ibc_packet: Packet,
        salt: vector<u8>,
        batch_packet: BatchPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {
        // TODO: implement this
    }

    fun acknowledge_forward(
        ibc_packet: Packet,
        salt: vector<u8>,
        forward_packet: ForwardPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {
        // TODO: implement this
    }

    fun acknowledge_multiplex(
        ibc_packet: Packet,
        salt: vector<u8>,
        multiplex_packet: MultiplexPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {
        // TODO: implement this
    }



    public fun on_timeout_packet(ibc_packet: Packet) {
        // Decode the packet data
        let packet_data = ibc::packet::data(&ibc_packet);

        let zkgm_packet = decode_packet(*packet_data);

        timeout_internal(
            ibc_packet,
            zkgm_packet.salt,
            decode_syscall(zkgm_packet.syscall)
        );
    }

    fun timeout_internal(
        ibc_packet: Packet,
        salt: vector<u8>,
        syscall_packet: SyscallPacket
    ) {
        if (syscall_packet.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };

        if (syscall_packet.index == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            timeout_fungible_asset_transfer(
                ibc_packet,
                salt,
                decode_fungible_asset_transfer(syscall_packet.packet)
            );
        } else if (syscall_packet.index == SYSCALL_BATCH) {
            timeout_batch(
                ibc_packet,
                salt,
                decode_batch(syscall_packet.packet)
            );
        } else if (syscall_packet.index == SYSCALL_FORWARD) {
            timeout_forward(
                ibc_packet,
                salt,
                decode_forward(syscall_packet.packet)
            );
        } else if (syscall_packet.index == SYSCALL_MULTIPLEX) {
            timeout_multiplex(
                ibc_packet,
                salt,
                decode_multiplex(syscall_packet.packet)
            );
        }  else {
            abort E_UNKNOWN_SYSCALL;
        }
    }

    fun timeout_fungible_asset_transfer(
        ibc_packet: Packet,
        salt: vector<u8>,
        transfer_packet: FungibleAssetTransferPacket
    ) {
        // TODO: Implement this
    }

    fun timeout_batch(
        ibc_packet: Packet,
        salt: vector<u8>,
        batch_packet: BatchPacket
    ) {
        // TODO: Implement this
    }

    fun timeout_forward(
        ibc_packet: Packet,
        salt: vector<u8>,
        forward_packet: ForwardPacket
    ) {
        // TODO: Implement this
    }

    fun timeout_multiplex(
        ibc_packet: Packet,
        salt: vector<u8>,
        multiplex_packet: MultiplexPacket
    ) {
        // TODO: Implement this
    }

    #[test(admin = @ucs03)]
    public fun test_predict_token(admin: &signer) acquires SignerRef {
        init_module(admin);

        let path = 1;
        let destination_channel = 1;
        let token = b"test_token";
        let (wrapped_address, salt) = predict_wrapped_token(path, destination_channel, token);
        let deployed_token_addr = deploy_token(path, destination_channel, token);

        std::debug::print(&string::utf8(b"wrapped address is: "));
        std::debug::print(&wrapped_address);
        std::debug::print(&string::utf8(b"deployed_token_addr is: "));
        std::debug::print(&deployed_token_addr);

        assert!(wrapped_address == deployed_token_addr, 101);
        assert!(is_deployed(deployed_token_addr), 102);
    }

    #[test(admin = @ucs03)]
    public fun test_is_deployed_false(admin: &signer) {
        init_module(admin);

        let path = 1;
        let destination_channel = 1;
        let token = b"never_deployed_salt";
        let (wrapped_address, salt) = predict_wrapped_token(path, destination_channel, token);

        assert!(!is_deployed(wrapped_address), 102);
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
        assert!(last_channel_from_path(115792089237316195423570985008687907853269984665640564039457584007913129639935) == 4294967295, 1);
    }

    #[test]
    public fun test_update_Channel_path() {
        assert!(update_channel_path(0,0) == 0, 1);
        assert!(update_channel_path(0,34) == 34, 1);
        assert!(update_channel_path(12414123,111) == 476753783979, 1);
        assert!(update_channel_path(44, 22) == 94489280556, 1);
    }
}