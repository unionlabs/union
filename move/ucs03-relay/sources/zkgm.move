module ucs03::zkgm_relay {
    use ibc::ibc;
    use ibc::helpers;
    use ibc::packet::{Self, Packet};
    use ibc::dispatcher;
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use aptos_std::copyable_any;
    use aptos_framework::function_info;
    use ibc::commitment;
    use ucs03::ethabi;
    use ucs03::dispatcher_zkgm;
    use ucs03::engine_zkgm;
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
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const ACK_SUCCESS: u256 = 1;
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;
    const ZKGM_VERSION_0: u8 = 0x00;
    const SYSCALL_FORWARD: u8 = 0x00;
    const SYSCALL_MULTIPLEX: u8 = 0x01;
    const SYSCALL_BATCH: u8 = 0x02;
    const SYSCALL_FUNGIBLE_ASSET_TRANSFER: u8 = 0x03;
    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";

    // Errors
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
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

    struct OnZkgmParams has copy, drop, store {
        sender: vector<u8>,
        contract_calldata: vector<u8>,
    }

    struct IIBCModuleOnRecvPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    struct IIBCModuleOnAcknowledgementPacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address,
    }

    struct IIBCModuleOnTimeoutPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address,
    }

    struct FungibleAssetTransferPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        sent_token: vector<u8>,
        sent_token_prefix: u256,
        sent_symbol: string::String,
        sent_name: string::String,
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
        fill_type: u256,
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

    struct Port<phantom T: key + store + drop> has key, copy, drop, store {
        port_id: address
    }


    public fun get_metadata(asset_addr: address): Object<Metadata> {
        object::address_to_object<Metadata>(asset_addr)
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

        let cb =
            function_info::new_function_info(
                account,
                string::utf8(b"zkgm_relay"),
                string::utf8(b"on_packet")
            );

        dispatcher::register<ZKGMProof>(cb, new_ucs_relay_proof(), bcs::to_bytes(&signer::address_of(account)));
        // ibc::register_application<ZKGMProof>(account, cb, new_ucs_relay_proof());
    }

    public fun register_application<T: key + store + drop>(
        zkgm_app: &signer, cb: FunctionInfo, type: T
    ) acquires SignerRef {
        dispatcher_zkgm::register<T>(cb, type, bcs::to_bytes(&signer::address_of(zkgm_app)));
        move_to(
            &get_signer(),
            Port<T> { port_id: signer::address_of(zkgm_app) }
        );
    }

    public fun decode_ack(buf: vector<u8>): Acknowledgement {
        let index = 0x20;
        let tag = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let inner_ack = ethabi::decode_vector<u8>(
            &buf,
            &mut index,
            |buf, index| {
                (ethabi::decode_uint(buf, index) as u8)
            }
        );
    
        Acknowledgement {
            tag: tag,
            inner_ack: inner_ack
        }
    }

    public fun encode_ack(packet: &Acknowledgement): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u256>(&mut buf, packet.tag);

        let version_offset = 0x40;
        ethabi::encode_uint<u32>(&mut buf, version_offset);
        
        ethabi::encode_vector<u8>(
            &mut buf,
            &packet.inner_ack,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun encode_asset_transfer_ack(ack: &AssetTransferAcknowledgement): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u256>(&mut buf, ack.fill_type);

        let version_offset = 0x40;
        ethabi::encode_uint<u32>(&mut buf, version_offset);
        
        ethabi::encode_vector<u8>(
            &mut buf,
            &ack.market_maker,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun decode_asset_transfer_ack(buf: vector<u8>): AssetTransferAcknowledgement {
        let index = 0x20;
        let fill_type = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let market_maker = ethabi::decode_vector<u8>(
            &buf,
            &mut index,
            |buf, index| {
                (ethabi::decode_uint(buf, index) as u8)
            }
        );

        AssetTransferAcknowledgement {
            fill_type: fill_type,
            market_maker: market_maker
        }
    }

    public fun decode_batch_ack(buf: vector<u8>): BatchAcknowledgement {
        let index = 0x40;
        let main_arr_length = ethabi::decode_uint(&buf, &mut index);
        index = index + (0x20 * main_arr_length as u64);

        let idx = 0;
        let acknowledgements = vector::empty();
        while (idx < main_arr_length) {
            let inner_vec = ethabi::decode_vector<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );
            vector::push_back(&mut acknowledgements, inner_vec);
            idx = idx+1;
        };

        BatchAcknowledgement {
            acknowledgements: acknowledgements
        }
    }

    public fun encode_batch_ack(ack: &BatchAcknowledgement): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&ack.acknowledgements);
        ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2){
            if (ack_arr_len == 1){
                ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                ethabi::encode_vector<u8>(
                    &mut buf,
                    vector::borrow(&ack.acknowledgements, 0),
                    |some_variable, data| {
                        ethabi::encode_uint<u8>(some_variable, *data);
                    }
                );
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let idx = 1;
        let prev_val = initial_stage;
        ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length = vector::length(vector::borrow(&ack.acknowledgements, idx-1));
            ethabi::encode_uint<u32>(&mut buf, prev_val + 0x20 * (prev_length+1 as u32));
            prev_val = prev_val + 0x20 * (prev_length+1 as u32);
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            ethabi::encode_vector<u8>(
                &mut buf,
                vector::borrow(&ack.acknowledgements, idx),
                |some_variable, data| {
                    ethabi::encode_uint<u8>(some_variable, *data);
                }
            );
            idx = idx + 1;
        };

        buf
    }

    public fun decode_batch_packet(buf: vector<u8>): BatchPacket {
        let index = 0x40;
        let main_arr_length = ethabi::decode_uint(&buf, &mut index);
        index = index + (0x20 * main_arr_length as u64);

        let idx = 0;
        let syscall_packets = vector::empty();
        while (idx < main_arr_length) {
            let inner_vec = ethabi::decode_vector<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );
            vector::push_back(&mut syscall_packets, inner_vec);
            idx = idx+1;
        };

        BatchPacket {
            syscall_packets: syscall_packets
        }
    }

    public fun encode_batch_packet(ack: &BatchPacket): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&ack.syscall_packets);
        ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2){
            if (ack_arr_len == 1){
                ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                ethabi::encode_vector<u8>(
                    &mut buf,
                    vector::borrow(&ack.syscall_packets, 0),
                    |some_variable, data| {
                        ethabi::encode_uint<u8>(some_variable, *data);
                    }
                );
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let idx = 1;
        let prev_val = initial_stage;
        ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length = vector::length(vector::borrow(&ack.syscall_packets, idx-1));
            ethabi::encode_uint<u32>(&mut buf, prev_val + 0x20 * (prev_length+1 as u32));
            prev_val = prev_val + 0x20 * (prev_length+1 as u32);
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            ethabi::encode_vector<u8>(
                &mut buf,
                vector::borrow(&ack.syscall_packets, idx),
                |some_variable, data| {
                    ethabi::encode_uint<u8>(some_variable, *data);
                }
            );
            idx = idx + 1;
        };

        buf
    }

    public fun decode_syscall(buf: vector<u8>): SyscallPacket {
        let index = 0x20;
        let version = ethabi::decode_uint(&buf, &mut index);
        let index_syscall = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;

        let packet = ethabi::decode_vector<u8>(
            &buf,
            &mut index,
            |buf, index| {
                (ethabi::decode_uint(buf, index) as u8)
            }
        );

        SyscallPacket {
            version: (version as u8),
            index: (index_syscall as u8),
            packet: packet
        }
    }

    public fun encode_forward(packet: &ForwardPacket): vector<u8>{
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u32>(&mut buf, packet.channel_id);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_height);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_timestamp);
        ethabi::encode_uint<u8>(&mut buf, 0x80);
        ethabi::encode_bytes(&mut buf, &packet.syscall_packet);
        buf
    }

    public fun decode_forward(buf: vector<u8>): ForwardPacket {
        let index = 0x20;
        let channel_id = ethabi::decode_uint(&buf, &mut index);
        let timeout_height = ethabi::decode_uint(&buf, &mut index);
        let timeout_timestamp = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let syscall_packet = ethabi::decode_bytes(&buf, &mut index);

        ForwardPacket {
            channel_id: (channel_id as u32),
            timeout_height: (timeout_height as u64),
            timeout_timestamp: (timeout_timestamp as u64),
            syscall_packet: syscall_packet
        }
    }

    public fun decode_multiplex(buf: vector<u8>): MultiplexPacket {
        let index = 0x40;
        let eureka = ethabi::decode_uint(&buf, &mut index) == 1;
        index = index + 0x20*2;
        let sender = ethabi::decode_bytes(&buf, &mut index);
        let contract_address = ethabi::decode_bytes(&buf, &mut index);
        let contract_calldata = ethabi::decode_bytes(&buf, &mut index);

        MultiplexPacket {
            sender: sender,
            eureka: eureka,
            contract_address: contract_address,
            contract_calldata: contract_calldata
        }
    }

    public fun encode_multiplex_sender_and_calldata(
        sender: vector<u8>,
        contract_calldata: vector<u8>
    ): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x40);
        let length_of_first = vector::length(&sender);
        ethabi::encode_uint<u64>(&mut buf, ((length_of_first / 32)  * 0x20) + 0x80) ;
        ethabi::encode_bytes(&mut buf, &sender);
        ethabi::encode_bytes(&mut buf, &contract_calldata);
        buf
    }


    public fun decode_fungible_asset_transfer(buf: vector<u8>): FungibleAssetTransferPacket {
        let index = 0x80;
        let sent_token_prefix = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x40;
        let sent_amount = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let ask_amount = ethabi::decode_uint(&buf, &mut index);
        let only_maker = (ethabi::decode_uint(&buf, &mut index) == 1);
        let sender = ethabi::decode_bytes(&buf, &mut index);
        let receiver = ethabi::decode_bytes(&buf, &mut index);
        let sent_token = ethabi::decode_bytes(&buf, &mut index);
        let sent_symbol = ethabi::decode_string(&buf, &mut index);
        let sent_name = ethabi::decode_string(&buf, &mut index);
        let ask_token = ethabi::decode_bytes(&buf, &mut index);

        FungibleAssetTransferPacket {
            sender: sender,
            receiver: receiver,
            sent_token: sent_token,
            sent_token_prefix: sent_token_prefix,
            sent_symbol: sent_symbol,
            sent_name: sent_name,
            sent_amount: (sent_amount as u64),
            ask_token: ask_token,
            ask_amount: (ask_amount as u64),
            only_maker: only_maker
        }
    }

    public fun encode_packet(packet: &ZkgmPacket): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x60);
        ethabi::encode_uint<u256>(&mut buf, packet.path);


        let version_offset = 0x20 * 4;
        ethabi::encode_uint<u32>(&mut buf, version_offset +  ((vector::length(&packet.salt) * 0x20) as u32));
        
        ethabi::encode_vector<u8>(
            &mut buf,
            &packet.salt,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        ethabi::encode_vector<u8>(
            &mut buf,
            &packet.syscall,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun decode_packet(buf: vector<u8>): ZkgmPacket {
        let index = 0x40;
        let packet_path = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let salt = ethabi::decode_vector<u8>(
            &buf,
            &mut index,
            |buf, index| {
                (ethabi::decode_uint(buf, index) as u8)
            }
        );
        let syscall = ethabi::decode_vector<u8>(
            &buf,
            &mut index,
            |buf, index| {
                (ethabi::decode_uint(buf, index) as u8)
            }
        );

        ZkgmPacket {
            salt: salt,
            path: packet_path,
            syscall: syscall
        }
    }


    public fun predict_wrapped_token(
        path: u256,
        destination_channel: u32,
        token: vector<u8>
    ): (address, vector<u8>) {
        let salt = hash::sha3_256(serialize_salt(path, destination_channel, token));

        let wrapped_address = object::create_object_address(&get_vault_addr(), salt);
        (wrapped_address, salt)
    }

    public fun deploy_token(
        salt: vector<u8>
    ): address acquires SignerRef {
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
        path: u256,
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
        if (path == 0){
            return 0
        };
        let current_hop_index = ((fls(path) / 32) as u8);
        let last_channel = path >> (current_hop_index * 32);
        (last_channel as u32)
    }

    public fun update_channel_path(path: u256, next_channel_id: u32): u256 {
        if (path == 0){
            return (next_channel_id as u256)
        };
        let next_hop_index = ((fls(path) / 32) as u8) + 1;
        if (next_hop_index > 7){
            abort E_INVALID_HOPS
        };

        let next_channel = (((next_channel_id as u256) << (next_hop_index * 32)) as u256) | path;
        (next_channel as u256)
    }

    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }


    public fun on_channel_open_init(
        _connection_id: u32,
        _channel_id: u32,
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
        _channel_id: u32, _counterparty_channel_id: u32, _counterparty_version: String
    ) { }

    public fun on_channel_open_confirm(_channel_id: u32) {}

    public fun on_channel_close_init(_channel_id: u32) {
        abort E_INFINITE_GAME
    }

    public fun on_channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
    }
    public fun on_recv_packet<T: key + store + drop>(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    ) acquires RelayStore, SignerRef {
        // We can call execute_internal directly
        let raw_zkgm_packet = ibc::packet::data(&ibc_packet);
        let zkgm_packet = decode_packet(*raw_zkgm_packet);
        
        let acknowledgement = execute_internal<T>(ibc_packet, relayer, relayer_msg, zkgm_packet.salt, zkgm_packet.path, decode_syscall(zkgm_packet.syscall));
        
        if (vector::length(&acknowledgement) == 0) {
            abort E_ACK_EMPTY
        } else if(acknowledgement == ACK_ERR_ONLYMAKER) {
            abort E_ONLY_MAKER
        } else {
            let return_value = encode_ack(&Acknowledgement {
                tag: ACK_SUCCESS,
                inner_ack: acknowledgement
            });
            dispatcher_zkgm::set_return_value<ZKGMProof>(new_ucs_relay_proof(), return_value);
        }
    }

    public fun on_acknowledge_packet(
        ibc_packet: Packet, acknowledgement: vector<u8>, relayer: address
    ) acquires RelayStore, SignerRef {
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
                relayer,
                zkgm_packet.salt,
                decode_syscall(zkgm_packet.syscall),
                zkgm_ack.tag == ACK_SUCCESS,
                zkgm_ack.inner_ack
            )
        }
    }

    fun acknowledge_internal(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        syscall_packet: SyscallPacket,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef {
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
                relayer,
                salt,
                decode_batch_packet(syscall_packet.packet),
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
                relayer,
                salt,
                decode_multiplex(syscall_packet.packet),
                success,
                inner_ack
            );
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun acknowledge_fungible_asset_transfer(
        ibc_packet: Packet,
        _salt: vector<u8>,
        transfer_packet: FungibleAssetTransferPacket,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef {
        if (success) {
            let asset_transfer_ack = decode_asset_transfer_ack(inner_ack);
            if (asset_transfer_ack.fill_type == FILL_TYPE_PROTOCOL ){
                // The protocol is filled, fee was paid to relayer.
            } else if (asset_transfer_ack.fill_type == FILL_TYPE_MARKETMAKER) {
                let market_maker = from_bcs::to_address(asset_transfer_ack.market_maker);
                let sent_token = from_bcs::to_address(transfer_packet.sent_token);
                let asset = get_metadata(sent_token);
                if (last_channel_from_path(transfer_packet.sent_token_prefix) == ibc::packet::source_channel(&ibc_packet)){
                    ucs03::fa_coin::mint_with_metadata(
                        &get_signer(),
                        market_maker,
                        transfer_packet.sent_amount,
                        asset
                    );
                } else {
                    primary_fungible_store::transfer(
                        &get_signer(),
                        asset,
                        market_maker,
                        transfer_packet.sent_amount
                    );
                }
            } else {
                abort E_INVALID_FILL_TYPE
            }
        } else{
            refund(ibc::packet::source_channel(&ibc_packet), transfer_packet);
        };
    }

    fun acknowledge_batch(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: BatchPacket,
        success: bool,
        inner_ack: vector<u8>
    ) acquires SignerRef {
        let l = vector::length(&batch_packet.syscall_packets);
        let batch_ack = decode_batch_ack(inner_ack);
        let i = 0;
        while (i < l){
            let syscall_ack = inner_ack;
            if (success){
                syscall_ack = *vector::borrow(&batch_ack.acknowledgements, i);
            };
            acknowledge_internal(
                ibc_packet,
                relayer,
                salt,
                decode_syscall(*vector::borrow(&batch_packet.syscall_packets, i)),
                success,
                syscall_ack
            );
        }
    }

    fun acknowledge_forward(
        _ibc_packet: Packet,
        _salt: vector<u8>,
        _forward_packet: ForwardPacket,
        _success: bool,
        _inner_ack: vector<u8>
    ) {    }

    fun acknowledge_multiplex(
        ibc_packet: Packet,
        relayer: address,
        _salt: vector<u8>,
        multiplex_packet: MultiplexPacket,
        success: bool,
        ack: vector<u8>
    ) {
        if (success && !multiplex_packet.eureka) {
            let multiplex_ibc_packet = ibc::packet::new(
                ibc::packet::source_channel(&ibc_packet),
                ibc::packet::destination_channel(&ibc_packet),
                encode_multiplex_sender_and_calldata(multiplex_packet.contract_address, multiplex_packet.contract_calldata),
                ibc::packet::timeout_height(&ibc_packet),
                ibc::packet::timeout_timestamp(&ibc_packet)
            );
            let param = copyable_any::pack<IIBCModuleOnAcknowledgementPacketParams>(
                IIBCModuleOnAcknowledgementPacketParams{
                    packet: multiplex_ibc_packet,
                    acknowledgement: ack,
                    relayer: relayer
                }
            );
            let contract_address = from_bcs::to_address(multiplex_packet.sender);

            engine_zkgm::dispatch(param, contract_address);
        }
    }



    public fun on_timeout_packet(ibc_packet: Packet, relayer: address) acquires SignerRef {
        // Decode the packet data
        let packet_data = ibc::packet::data(&ibc_packet);

        let zkgm_packet = decode_packet(*packet_data);

        timeout_internal(
            ibc_packet,
            relayer,
            zkgm_packet.salt,
            decode_syscall(zkgm_packet.syscall)
        );
    }

    fun timeout_internal(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        syscall_packet: SyscallPacket
    ) acquires SignerRef {
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
                relayer,
                salt,
                decode_batch_packet(syscall_packet.packet)
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
                relayer,
                salt,
                decode_multiplex(syscall_packet.packet)
            );
        }  else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun timeout_fungible_asset_transfer(
        ibc_packet: Packet,
        _salt: vector<u8>,
        transfer_packet: FungibleAssetTransferPacket
    ) acquires SignerRef {
        refund(ibc::packet::source_channel(&ibc_packet), transfer_packet);
    }

    fun refund(
        source_channel: u32,
        asset_transfer_packet: FungibleAssetTransferPacket
    ) acquires SignerRef {
        let sender = from_bcs::to_address(asset_transfer_packet.sender);
        let sent_token = from_bcs::to_address(asset_transfer_packet.sender);

        let asset = get_metadata(sent_token);

        if (last_channel_from_path(asset_transfer_packet.sent_token_prefix) == source_channel){
            ucs03::fa_coin::mint_with_metadata(
                &get_signer(), sender, asset_transfer_packet.sent_amount, asset
            );
        } else {
            primary_fungible_store::transfer(
                &get_signer(),
                asset,
                sender,
                asset_transfer_packet.sent_amount
            );
        }
    }

    fun timeout_batch(
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: BatchPacket
    ) acquires SignerRef {
        let l = vector::length(&batch_packet.syscall_packets);
        let i = 0;
        while (i < l){
            timeout_internal(
                ibc_packet,
                relayer,
                salt,
                decode_syscall(*vector::borrow(&batch_packet.syscall_packets, i))
            );
        };
    }

    fun timeout_forward(
        _ibc_packet: Packet,
        _salt: vector<u8>,
        _forward_packet: ForwardPacket
    ) {

    }

    fun timeout_multiplex(
        ibc_packet: Packet,
        relayer: address,
        _salt: vector<u8>,
        multiplex_packet: MultiplexPacket
    ) {
        if (!multiplex_packet.eureka) {
            let multiplex_ibc_packet = ibc::packet::new(
                ibc::packet::source_channel(&ibc_packet),
                ibc::packet::destination_channel(&ibc_packet),
                encode_multiplex_sender_and_calldata(multiplex_packet.contract_address, multiplex_packet.contract_calldata),
                ibc::packet::timeout_height(&ibc_packet),
                ibc::packet::timeout_timestamp(&ibc_packet)
            );
            let param = copyable_any::pack<IIBCModuleOnTimeoutPacketParams>(
                IIBCModuleOnTimeoutPacketParams{
                    packet: multiplex_ibc_packet,
                    relayer: relayer
                }
            );
            let contract_address = from_bcs::to_address(multiplex_packet.sender);

            engine_zkgm::dispatch(param, contract_address);
        }
    }


    public entry fun execute<T: key + store + drop>(
        //ibc_packet: Packet,
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,

        relayer: address,
        relayer_msg: vector<u8>,
        raw_zkgm_packet: vector<u8>
    ) acquires RelayStore, SignerRef {
        // no need to check msg.sender since its not public entry function
        // sender will be address(this) anyway
        let ibc_packet = ibc::packet::new(
            source_channel,
            destination_channel,
            data,
            timeout_height,
            timeout_timestamp
        );

        let zkgm_packet = decode_packet(raw_zkgm_packet);
        execute_internal<T>(ibc_packet, relayer, relayer_msg, zkgm_packet.salt, zkgm_packet.path, decode_syscall(zkgm_packet.syscall));
    }

    fun execute_internal<T: key + store + drop>(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        syscall_packet: SyscallPacket
    ): (vector<u8>) acquires RelayStore, SignerRef {
        if (syscall_packet.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (syscall_packet.index == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            execute_fungible_asset_transfer(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                decode_fungible_asset_transfer(syscall_packet.packet)
            )
        } else if (syscall_packet.index == SYSCALL_BATCH) {
            execute_batch<T>(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                decode_batch_packet(syscall_packet.packet)
            )
        } else if (syscall_packet.index == SYSCALL_FORWARD) {
            execute_forward(
                ibc_packet,
                relayer_msg,
                salt,
                path,
                decode_forward(syscall_packet.packet)
            )
        } else if (syscall_packet.index == SYSCALL_MULTIPLEX) {
            execute_multiplex(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                decode_multiplex(syscall_packet.packet)
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun execute_fungible_asset_transfer(
        ibc_packet: Packet,
        relayer: address,
        _relayer_msg: vector<u8>,
        _salt: vector<u8>,
        path: u256,
        transfer_packet: FungibleAssetTransferPacket
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        if (transfer_packet.only_maker) {
            return ACK_ERR_ONLYMAKER
        };
        if (transfer_packet.ask_amount > transfer_packet.sent_amount) {
            abort E_INVALID_AMOUNT
        };
        let (wrapped_address, salt) = predict_wrapped_token(
                                        path,
                                        ibc::packet::destination_channel(&ibc_packet),
                                        transfer_packet.sent_token
                                    );
        let ask_token = from_bcs::to_address(transfer_packet.ask_token);
        let receiver  = from_bcs::to_address(transfer_packet.receiver);
        let fee = transfer_packet.sent_amount - transfer_packet.ask_amount;
        // ------------------------------------------------------------------
        // TODO: no idea if the code below will work lol, it looks promising though
        // ------------------------------------------------------------------
        if (ask_token == wrapped_address) {
            if (!is_deployed(wrapped_address)) {
                deploy_token(
                    salt
                );
                let value = update_channel_path(path, ibc::packet::destination_channel(&ibc_packet));
                smart_table::upsert(&mut store.token_origin, wrapped_address, value);
            };
            ucs03::fa_coin::mint_with_metadata(
                &get_signer(),
                receiver,
                transfer_packet.ask_amount,
                get_metadata(ask_token)
            );
            if (fee > 0) {
                ucs03::fa_coin::mint_with_metadata(
                    &get_signer(),
                    relayer,
                    fee,
                    get_metadata(ask_token)
                );
            }
        } else {
            if (transfer_packet.sent_token_prefix == (ibc::packet::source_channel(&ibc_packet) as u256)) {
                let balance_key = ChannelBalancePair {
                    channel: ibc::packet::destination_channel(&ibc_packet),
                    token: ask_token
                };

                let curr_balance = *smart_table::borrow(
                    &store.channel_balance,
                    balance_key
                );

                smart_table::upsert(
                    &mut store.channel_balance,
                    balance_key,
                    curr_balance - (transfer_packet.ask_amount as u256)
                );
                let asset = get_metadata(ask_token);

                primary_fungible_store::transfer(
                    &get_signer(),
                    asset,
                    receiver,
                    transfer_packet.ask_amount
                );
                if (fee > 0){
                    primary_fungible_store::transfer(
                        &get_signer(),
                        asset,
                        relayer,
                        fee
                    );
                }
            };
        };
        encode_asset_transfer_ack(
            &AssetTransferAcknowledgement {
                fill_type: FILL_TYPE_PROTOCOL,
                market_maker: ACK_EMPTY
            }
        )
    }

    fun execute_batch<T: key + store + drop>(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        batch_packet: BatchPacket
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let l = vector::length(&batch_packet.syscall_packets);
        let acks = vector::empty();
        let i = 0;
        while (i < l){
            let syscall_packet = decode_syscall(*vector::borrow(&batch_packet.syscall_packets, i));
            vector::push_back(
                &mut acks,
                execute_internal<T>(
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    salt,
                    path,
                    syscall_packet
                )
            );
            if (vector::length(vector::borrow(&acks, i)) == 0){
                abort E_BATCH_MUST_BE_SYNC
            };
        };
        encode_batch_ack(&BatchAcknowledgement { acknowledgements: acks })
    }

    fun execute_forward(
        ibc_packet: Packet,
        _relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        forward_packet: ForwardPacket
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let sent_packet = ibc::ibc::send_packet(
            &get_signer(),
            get_self_address(),
            forward_packet.channel_id,
            forward_packet.timeout_height,
            forward_packet.timeout_timestamp,
            encode_packet(
                &ZkgmPacket {
                    salt: salt,
                    path: update_channel_path(path, ibc::packet::destination_channel(&ibc_packet)),
                    syscall: forward_packet.syscall_packet
                })
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
        multiplex_packet: MultiplexPacket
    ): (vector<u8>) {
        let contract_address = from_bcs::to_address(multiplex_packet.contract_address);
        if (multiplex_packet.eureka) {
            let param = copyable_any::pack<OnZkgmParams>(
                OnZkgmParams{
                    sender: multiplex_packet.sender,
                    contract_calldata: multiplex_packet.contract_calldata
                }
            );
            engine_zkgm::dispatch(param, contract_address);
            return bcs::to_bytes(&ACK_SUCCESS)
        };
        let multiplex_ibc_packet = ibc::packet::new(
            ibc::packet::source_channel(&ibc_packet),
            ibc::packet::destination_channel(&ibc_packet),
            encode_multiplex_sender_and_calldata(multiplex_packet.sender, multiplex_packet.contract_calldata),
            ibc::packet::timeout_height(&ibc_packet),
            ibc::packet::timeout_timestamp(&ibc_packet)
        );
        let param = copyable_any::pack<IIBCModuleOnRecvPacketParams>(
            IIBCModuleOnRecvPacketParams{
                packet: multiplex_ibc_packet,
                relayer: relayer,
                relayer_msg: relayer_msg
            }
        );

        engine_zkgm::dispatch(param, contract_address);

        let acknowledgement = dispatcher_zkgm::get_return_value(contract_address);

        if (vector::length(&acknowledgement) == 0){
            abort E_UNIMPLEMENTED
        };
        acknowledgement
    }


    public entry fun send(
        sender: &signer,
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>,
        raw_syscall: vector<u8>
    ) acquires SignerRef, RelayStore {
        verify_internal(sender, channel_id, 0, raw_syscall);
        ibc::ibc::send_packet(
            &get_signer(),
            get_self_address(),
            channel_id,
            timeout_height,
            timeout_timestamp,
            encode_packet(
                &ZkgmPacket {
                    salt: salt,
                    path: 0,
                    syscall: raw_syscall
                }
            )
        );
    }

    fun verify_internal(
        sender: &signer,
        channel_id: u32,
        path: u256,
        raw_syscall: vector<u8>
    ) acquires RelayStore, SignerRef {
        let syscall_packet = decode_syscall(raw_syscall);
        if (syscall_packet.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (syscall_packet.index == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            verify_fungible_asset_transfer(sender, channel_id, path, decode_fungible_asset_transfer(syscall_packet.packet))
        } else if (syscall_packet.index == SYSCALL_BATCH) {
            verify_batch(sender, channel_id, path, decode_batch_packet(syscall_packet.packet))
        } else if (syscall_packet.index == SYSCALL_FORWARD) {
            verify_forward(sender, channel_id, path, decode_forward(syscall_packet.packet))
        } else if (syscall_packet.index == SYSCALL_MULTIPLEX) {
            verify_multiplex(sender, channel_id, path, decode_multiplex(syscall_packet.packet))
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun verify_fungible_asset_transfer(
        sender: &signer,
        channel_id: u32,
        _path: u256,
        transfer_packet: FungibleAssetTransferPacket
    ) acquires RelayStore, SignerRef {
        let store = borrow_global<RelayStore>(get_vault_addr());

        let sent_token = from_bcs::to_address(transfer_packet.sent_token);

        let asset = get_metadata(sent_token);
        let name = ucs03::fa_coin::name_with_metadata(asset);
        let symbol = ucs03::fa_coin::symbol_with_metadata(asset);

        if (transfer_packet.sent_name != name){
            abort E_INVALID_ASSET_NAME
        };
        if (transfer_packet.sent_symbol != symbol){
            abort E_INVALID_ASSET_SYMBOL
        };
        let origin = *smart_table::borrow_with_default(
            &store.token_origin, sent_token, &0
        );

        if (last_channel_from_path(origin) == channel_id) {
            ucs03::fa_coin::burn_with_metadata(
                &get_signer(),
                signer::address_of(sender),
                transfer_packet.sent_amount,
                asset
            );
        } else {
            primary_fungible_store::transfer(
                sender,
                asset,
                signer::address_of(&get_signer()),
                transfer_packet.sent_amount
            );
        };
        if (!transfer_packet.only_maker && transfer_packet.sent_token_prefix != origin) {
            abort E_INVALID_ASSET_ORIGIN
        }
    }

    fun verify_batch(
        sender: &signer,
        channel_id: u32,
        path: u256,
        batch_packet: BatchPacket
    ) acquires RelayStore, SignerRef {
        let l = vector::length(&batch_packet.syscall_packets);
        let i = 0;
        while (i < l){
            verify_internal(
                sender,
                channel_id,
                path,
                *vector::borrow(&batch_packet.syscall_packets, i)
            );
        }
    }

    fun verify_forward(
        sender: &signer,
        channel_id: u32,
        path: u256,
        forward_packet: ForwardPacket
    ) acquires RelayStore, SignerRef {
        verify_internal(
            sender,
            channel_id,
            update_channel_path(path, forward_packet.channel_id),
            forward_packet.syscall_packet
        );
    }

    fun verify_multiplex(
        _sender: &signer,
        _channel_id: u32,
        _path: u256,
        _multiplex_packet: MultiplexPacket
    ) { }

    // public fun on_packet<T: key, P: key + store + drop>(_store: Object<T>): u64 acquires RelayStore, SignerRef {
    //     let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     let my_signer = get_signer(); // just to require SignerRef and RelayStore for testing

    //     let value: copyable_any::Any = dispatcher::get_data(new_ucs_relay_proof());
    //     let type_name_output = *copyable_any::type_name(&value);
    //     std::debug::print(&type_name_output);
    //     0
    // }

    public fun on_packet<T: key, P: key + store + drop>(_store: Object<T>): u64 acquires RelayStore, SignerRef {
        let value: copyable_any::Any = dispatcher::get_data(new_ucs_relay_proof());
        let type_name_output = *copyable_any::type_name(&value);

        if (type_name_output == std::type_info::type_name<helpers::RecvPacketParamsZKGM>()) {
            let (pack, relayer, relayer_msg) =
                helpers::on_recv_packet_zkgm_deconstruct(
                    copyable_any::unpack<helpers::RecvPacketParamsZKGM>(value)
                );
            on_recv_packet<P>(pack, relayer, relayer_msg);
        } else if (type_name_output
            == std::type_info::type_name<helpers::AcknowledgePacketParamsZKGM>()) {
            let (pack, acknowledgement, relayer) =
                helpers::on_acknowledge_packet_deconstruct_zkgm(
                    copyable_any::unpack<helpers::AcknowledgePacketParamsZKGM>(value)
                );
            on_acknowledge_packet(pack, acknowledgement, relayer);
        } else if (type_name_output
            == std::type_info::type_name<helpers::TimeoutPacketParamsZKGM>()) {
            let (pack, relayer) =
                helpers::on_timeout_packet_deconstruct_zkgm(
                    copyable_any::unpack<helpers::TimeoutPacketParamsZKGM>(value)
                );
            on_timeout_packet(pack, relayer);
        } else if (type_name_output
            == std::type_info::type_name<helpers::ChannelOpenInitParams>()) {
            let (connection_id, channel_id, version) =
                helpers::on_channel_open_init_deconstruct(
                    copyable_any::unpack<helpers::ChannelOpenInitParams>(value)
                );
            on_channel_open_init(connection_id, channel_id, version);
        } else if (type_name_output
            == std::type_info::type_name<helpers::ChannelOpenTryParams>()) {
            let (
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            ) =
                helpers::on_channel_open_try_deconstruct(
                    copyable_any::unpack<helpers::ChannelOpenTryParams>(value)
                );
            on_channel_open_try(
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            );
        } else if (type_name_output
            == std::type_info::type_name<helpers::ChannelOpenAckParams>()) {
            let (channel_id, counterparty_channel_id, counterparty_version) =
                helpers::on_channel_open_ack_deconstruct(
                    copyable_any::unpack<helpers::ChannelOpenAckParams>(value)
                );
            on_channel_open_ack(
                channel_id, counterparty_channel_id, counterparty_version
            );
        } else if (type_name_output
            == std::type_info::type_name<helpers::ChannelOpenConfirmParams>()) {
            let channel_id =
                helpers::on_channel_open_confirm_deconstruct(
                    copyable_any::unpack<helpers::ChannelOpenConfirmParams>(value)
                );
            on_channel_open_confirm(channel_id);
        } else if (type_name_output
            == std::type_info::type_name<helpers::ChannelCloseInitParams>()) {
            let channel_id =
                helpers::on_channel_close_init_deconstruct(
                    copyable_any::unpack<helpers::ChannelCloseInitParams>(value)
                );
            on_channel_close_init(channel_id);
        } else if (type_name_output
            == std::type_info::type_name<helpers::ChannelCloseConfirmParams>()) {
            let channel_id =
                helpers::on_channel_close_confirm_deconstruct(
                    copyable_any::unpack<helpers::ChannelCloseConfirmParams>(value)
                );
            on_channel_close_confirm(channel_id);
        } else {
            std::debug::print(
                &string::utf8(b"Invalid function type detected in on_packet function!")
            );
        };
        0
    }

    #[test(admin = @ucs03, ibc = @ibc)]
    public fun test_predict_token(admin: &signer, ibc: &signer) acquires SignerRef {
        dispatcher::init_module_for_testing(ibc);
        // ibc::init_module(ibc);
        init_module_for_testing(admin);

        let path = 1;
        let destination_channel = 1;
        let token = b"test_token";
        let (wrapped_address, salt) = predict_wrapped_token(path, destination_channel, token);
        let deployed_token_addr = deploy_token(salt);

        std::debug::print(&string::utf8(b"wrapped address is: "));
        std::debug::print(&wrapped_address);
        std::debug::print(&string::utf8(b"deployed_token_addr is: "));
        std::debug::print(&deployed_token_addr);

        assert!(wrapped_address == deployed_token_addr, 101);
        assert!(is_deployed(deployed_token_addr), 102);
    }

    #[test(admin = @ucs03, ibc = @ibc)]
    public fun test_is_deployed_false(admin: &signer, ibc: &signer) {
        dispatcher::init_module_for_testing(ibc);
        init_module_for_testing(admin);


        let path = 1;
        let destination_channel = 1;
        let token = b"never_deployed_salt";
        let (wrapped_address, _salt) = predict_wrapped_token(path, destination_channel, token);

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

    #[test]
    fun test_zkgm_encode_decode() {
        let output = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000032dcd60000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let zkgm_data = ZkgmPacket {
            salt: b"helloo",
            path: 3333334,
            syscall: b"hellloo"
        };

        let zkgm_bytes = encode_packet(&zkgm_data);
        assert!(zkgm_bytes == output, 0);

        let zkgm_data_decoded = decode_packet(zkgm_bytes);
        assert!(zkgm_data_decoded.salt == b"helloo", 1);
        assert!(zkgm_data_decoded.path == 3333334, 2);
        assert!(zkgm_data_decoded.syscall == b"hellloo", 3);
    }


    #[test]
    fun test_decode_syscall(){
        let output = x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000007100000000000000000000000000000000000000000000000000000000000000f40000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        
        let syscall_data_decoded = decode_syscall(output);
        assert!(syscall_data_decoded.version == 113, 1);
        assert!(syscall_data_decoded.index == 244, 2);
        assert!(syscall_data_decoded.packet == b"hellloo", 3);
    }

    #[test]
    fun test_encode_decode_ack() {
        let output = x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let ack_data = Acknowledgement {
            tag: 7788909223344,
            inner_ack: b"hellloo"
        };

        let ack_bytes = encode_ack(&ack_data);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode_ack(ack_bytes);
        assert!(ack_data_decoded.tag == 7788909223344, 1);
        assert!(ack_data_decoded.inner_ack == b"hellloo", 3);
    }

    #[test]
    fun test_encode_decode_asset_transfer_ack() {
        let output = x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let ack_data = AssetTransferAcknowledgement {
            fill_type: 7788909223344,
            market_maker: b"hellloo"
        };

        let ack_bytes = encode_asset_transfer_ack(&ack_data);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode_asset_transfer_ack(ack_bytes);
        assert!(ack_data_decoded.fill_type == 7788909223344, 1);
        assert!(ack_data_decoded.market_maker == b"hellloo", 3);
    }    
    
    #[test]
    fun test_encode_decode_batch_ack() {
        // ---------------- TEST 1 ----------------
        let output = x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006900000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065";
        let outer_arr = vector::empty();
        vector::push_back(
            &mut outer_arr,
            b"hello"
        );
        vector::push_back(
            &mut outer_arr,
            b"hi"
        );
        vector::push_back(
            &mut outer_arr,
            b"hehe"
        );
        let ack_data = BatchAcknowledgement {
            acknowledgements: outer_arr  
        };
        let ack_bytes = encode_batch_ack(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"hi", 3);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 2) == b"hehe", 4);


        // ---------------- TEST 2 ----------------
        let output2 = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000069";
        let outer_arr = vector::empty();
        vector::push_back(
            &mut outer_arr,
            b"hello"
        );
        vector::push_back(
            &mut outer_arr,
            b"hi"
        );
        let ack_data2 = BatchAcknowledgement {
            acknowledgements: outer_arr  
        };
        let ack_bytes2 = encode_batch_ack(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes2);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"hi", 3);



        // ---------------- TEST 3 ----------------
        let output3 = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000360000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000007300000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();
        let idx = 0;
        vector::push_back(
            &mut outer_arr,
            b"xdddd"
        );
        vector::push_back(
            &mut outer_arr,
            b"test"
        );
        while (idx <10){
            vector::push_back(
                &mut outer_arr,
                b""
            );
            idx = idx + 1;
        };

        let ack_data3 = BatchAcknowledgement {
            acknowledgements: outer_arr  
        };
        let ack_bytes3 = encode_batch_ack(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes3);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 12, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"xdddd", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"test", 3);



        // ---------------- TEST 4 ----------------
        let output4 = x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064";
        let outer_arr = vector::empty();
        vector::push_back(
            &mut outer_arr,
            b"xdddd"
        );
        
        let ack_data4 = BatchAcknowledgement {
            acknowledgements: outer_arr  
        };
        let ack_bytes4 = encode_batch_ack(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes4);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"xdddd", 2);

        // ---------------- TEST 5 ----------------
        let output5 = x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();
        
        let ack_data5 = BatchAcknowledgement {
            acknowledgements: outer_arr  
        };
        let ack_bytes5 = encode_batch_ack(&ack_data5);
        assert!(ack_bytes5 == output5, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes5);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 0, 1);

    }

    #[test]
    fun test_encode_decode_batch_packet() {
        // ---------------- TEST 1 ----------------
        let output = x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006900000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065";
        let outer_arr = vector::empty();
        vector::push_back(
            &mut outer_arr,
            b"hello"
        );
        vector::push_back(
            &mut outer_arr,
            b"hi"
        );
        vector::push_back(
            &mut outer_arr,
            b"hehe"
        );
        let ack_data = BatchPacket {
            syscall_packets: outer_arr  
        };
        let ack_bytes = encode_batch_packet(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 1) == b"hi", 3);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 2) == b"hehe", 4);


        // ---------------- TEST 2 ----------------
        let output2 = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000069";
        let outer_arr = vector::empty();
        vector::push_back(
            &mut outer_arr,
            b"hello"
        );
        vector::push_back(
            &mut outer_arr,
            b"hi"
        );
        let ack_data2 = BatchPacket {
            syscall_packets: outer_arr  
        };
        let ack_bytes2 = encode_batch_packet(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes2);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 1) == b"hi", 3);



        // ---------------- TEST 3 ----------------
        let output3 = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000360000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000007300000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();
        let idx = 0;
        vector::push_back(
            &mut outer_arr,
            b"xdddd"
        );
        vector::push_back(
            &mut outer_arr,
            b"test"
        );
        while (idx <10){
            vector::push_back(
                &mut outer_arr,
                b""
            );
            idx = idx + 1;
        };

        let ack_data3 = BatchPacket {
            syscall_packets: outer_arr  
        };
        let ack_bytes3 = encode_batch_packet(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes3);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 12, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"xdddd", 2);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 1) == b"test", 3);



        // ---------------- TEST 4 ----------------
        let output4 = x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064";
        let outer_arr = vector::empty();
        vector::push_back(
            &mut outer_arr,
            b"xdddd"
        );
        
        let ack_data4 = BatchPacket {
            syscall_packets: outer_arr  
        };
        let ack_bytes4 = encode_batch_packet(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes4);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"xdddd", 2);

        // ---------------- TEST 5 ----------------
        let output5 = x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();
        
        let ack_data5 = BatchPacket {
            syscall_packets: outer_arr  
        };
        let ack_bytes5 = encode_batch_packet(&ack_data5);
        assert!(ack_bytes5 == output5, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes5);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 0, 1);

    }

    #[test]
    fun test_encode_decode_forward_packet() {
        let output = x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000005161500000000000000000000000000000000000000000000000000000000000056ce0000000000000000000000000000000000000000000000000000000000002b670000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000005a4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000";
        let forward_data = ForwardPacket {
            channel_id: 333333,
            timeout_height: 22222,
            timeout_timestamp: 11111,
            syscall_packet: b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData"
        };

        let ack_bytes = encode_forward(&forward_data);
        assert!(ack_bytes == output, 0);

        let forward_data_decoded = decode_forward(ack_bytes);
        assert!(forward_data_decoded.channel_id == forward_data.channel_id, 0);
        assert!(forward_data_decoded.timeout_height == forward_data.timeout_height, 1);
        assert!(forward_data_decoded.timeout_timestamp == forward_data.timeout_timestamp, 2);
        assert!(forward_data_decoded.syscall_packet == forward_data.syscall_packet, 3);
    }  

    #[test]
    fun test_decode_multiplex() {
        let output = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000002a3078354233384461366137303163353638353435644366634230334663423837356635366265646443340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000617468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c626200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005a4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000";
        let multiplex_pack = MultiplexPacket{
            sender: b"0x5B38Da6a701c568545dCfcB03FcB875f56beddC4",
            eureka: true,
            contract_address: b"thisismyaddresszlulthisismyaddresszlulthisismyaddresszlulthisismyaddresszlulthisismyaddresszlulbb",
            contract_calldata: b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData"
        };

        let multiplex_decoded = decode_multiplex(output);
        assert!(multiplex_decoded.sender == multiplex_pack.sender, 0);
        assert!(multiplex_decoded.eureka == multiplex_pack.eureka, 1);
        assert!(multiplex_decoded.contract_address == multiplex_pack.contract_address, 2);
        assert!(multiplex_decoded.contract_calldata == multiplex_pack.contract_calldata, 3);
    }  

    #[test]
    fun test_decode_fungible_asset_transfer_pack() {
        
        let output = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000007c37bdc730000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000007a1200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f307853656e646572416464726573730000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001130785265636569766572416464726573730000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012307853656e74546f6b656e4164647265737300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014535353535353535353535353535353594d424f4c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000155454545454545454545454546f6b656e204e616d6500000000000000000000000000000000000000000000000000000000000000000000000000000000000011307841736b546f6b656e41646472657373000000000000000000000000000000";
        let fatp = FungibleAssetTransferPacket {
            sender: b"0xSenderAddress",
            receiver: b"0xReceiverAddress",
            sent_token: b"0xSentTokenAddress",
            sent_token_prefix: 33344445555,
            sent_symbol: string::utf8(b"SSSSSSSSSSSSSSSYMBOL"),
            sent_name: string::utf8(b"TTTTTTTTTTTToken Name"),
            sent_amount: 1000000, // Example token amount
            ask_token: b"0xAskTokenAddress",
            ask_amount: 500000, // Example ask amount
            only_maker: false
        };

        let fatp_decoded = decode_fungible_asset_transfer(output);
        assert!(fatp.sender == fatp_decoded.sender, 0);
        assert!(fatp.receiver == fatp_decoded.receiver, 0);
        assert!(fatp.sent_token == fatp_decoded.sent_token, 0);
        assert!(fatp.sent_token_prefix == fatp_decoded.sent_token_prefix, 0);
        assert!(fatp.sent_symbol == fatp_decoded.sent_symbol, 0);
        assert!(fatp.sent_name == fatp_decoded.sent_name, 0);
        assert!(fatp.sent_amount == fatp_decoded.sent_amount, 0);
        assert!(fatp.ask_token == fatp_decoded.ask_token, 0);
        assert!(fatp.ask_amount == fatp_decoded.ask_amount, 0);
        assert!(fatp.only_maker == fatp_decoded.only_maker, 0);
    }  
    

    #[test]
    fun test_encode_multiplex_sender_and_calldata() {
        let output = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000d23078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010e4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000000000000000000000000000";
        
        let ack_bytes = encode_multiplex_sender_and_calldata(b"0x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC4",
                                                            b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData");
        assert!(ack_bytes == output, 0);
    }  
}

