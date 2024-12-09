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
    use std::option;

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
    public fun decode_asset_transfer_ack(buf: vector<u8>): AssetTransferAcknowledgement {
        // TODO: implement this
        AssetTransferAcknowledgement {
            fill_type: 0,
            market_maker: vector::empty()
        }
    }
    public fun decode_batch_ack(buf: vector<u8>): BatchAcknowledgement {
        // TODO: implement this
        BatchAcknowledgement {
            acknowledgements: vector::empty()
        }
    }

    public fun encode_batch_ack(ack: &BatchAcknowledgement): vector<u8> {
        // TODO: implement this
        let buf = vector::empty<u8>();
        buf
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
            sent_symbol: string::utf8(b""),
            sent_name: string::utf8(b""),
            sent_amount: 0,
            ask_token: vector::empty(),
            ask_amount: 0,
            only_maker: false
        }
    }

    public fun encode_packet(packet: &ZkgmPacket): vector<u8> {
        // TODO Implement this
        let buf = vector::empty<u8>();
        buf
    }

    public fun encode_asset_transfer_ack(ack: &AssetTransferAcknowledgement): vector<u8> {
        // TODO Implement this
        let buf = vector::empty<u8>();
        buf
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
        path: u256,
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
    public fun on_recv_packet(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    ) acquires RelayStore, SignerRef {
        // We can call execute_internal directly
        let raw_zkgm_packet = ibc::packet::data(&ibc_packet);
        let zkgm_packet = decode_packet(*raw_zkgm_packet);
        execute_internal(ibc_packet, relayer, relayer_msg, zkgm_packet.salt, zkgm_packet.path, decode_syscall(zkgm_packet.syscall));
    }

    public fun on_acknowledge_packet(
        ibc_packet: Packet, acknowledgement: vector<u8>
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
    ) acquires SignerRef {
        if (success) {
            let asset_transfer_ack = decode_asset_transfer_ack(inner_ack);
            if (asset_transfer_ack.fill_type == FILL_TYPE_PROTOCOL ){
                // The protocol is filled, fee was paid to relayer.
            } else if (asset_transfer_ack.fill_type == FILL_TYPE_MARKETMAKER) {
                // TODO: implement here
            } else {
                abort E_INVALID_FILL_TYPE
            }
        } else{
            refund(ibc::packet::source_channel(&ibc_packet), transfer_packet);
        };
    }

    fun acknowledge_batch(
        ibc_packet: Packet,
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
                salt,
                decode_syscall(*vector::borrow(&batch_packet.syscall_packets, i)),
                success,
                syscall_ack
            );
        }
    }

    fun acknowledge_forward(
        ibc_packet: Packet,
        salt: vector<u8>,
        forward_packet: ForwardPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {    }

    fun acknowledge_multiplex(
        ibc_packet: Packet,
        salt: vector<u8>,
        multiplex_packet: MultiplexPacket,
        success: bool,
        inner_ack: vector<u8>
    ) {
        if (success && !multiplex_packet.eureka) {ibc::packet::new(
                0, // TODO: sequence is no longer exist
                ibc::packet::source_channel(&ibc_packet),
                ibc::packet::destination_channel(&ibc_packet),
                multiplex_packet.contract_calldata, // TODO: abi.encode contract_addr & contractcalldata
                ibc::packet::timeout_height(&ibc_packet),
                ibc::packet::timeout_timestamp(&ibc_packet)
            );
            // TODO: Call
            // IIBCModule(address(bytes20(multiplexPacket.sender)))
            //  .onAcknowledgementPacket(multiplexIbcPacket, ack, relayer);
            // not sure how?
        }
    }



    public fun on_timeout_packet(ibc_packet: Packet) acquires SignerRef {
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
        salt: vector<u8>,
        batch_packet: BatchPacket
    ) acquires SignerRef {
        let l = vector::length(&batch_packet.syscall_packets);
        let i = 0;
        while (i < l){
            timeout_internal(
                ibc_packet,
                salt,
                decode_syscall(*vector::borrow(&batch_packet.syscall_packets, i))
            );
        };
    }

    fun timeout_forward(
        ibc_packet: Packet,
        salt: vector<u8>,
        forward_packet: ForwardPacket
    ) {

    }

    fun timeout_multiplex(
        ibc_packet: Packet,
        salt: vector<u8>,
        multiplex_packet: MultiplexPacket
    ) {
        if (!multiplex_packet.eureka) {
            let multiplex_ibc_packet = ibc::packet::new(
                0, // TODO: sequence is no longer exist
                ibc::packet::source_channel(&ibc_packet),
                ibc::packet::destination_channel(&ibc_packet),
                multiplex_packet.contract_calldata, // TODO: abi.encode contract_addr & contractcalldata
                ibc::packet::timeout_height(&ibc_packet),
                ibc::packet::timeout_timestamp(&ibc_packet)
            );
            // TODO: Call
            // IIBCModule(address(bytes20(multiplexPacket.sender)))
            //  .onTimeoutPacket(multiplexIbcPacket, relayer);
            // not sure how?
        }
    }


    public entry fun execute(
        //ibc_packet: Packet,
        sequence: u64, // TODO: fix tihs sequence thing later
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
            sequence,
            source_channel,
            destination_channel,
            data,
            timeout_height,
            timeout_timestamp
        );

        let zkgm_packet = decode_packet(raw_zkgm_packet);
        execute_internal(ibc_packet, relayer, relayer_msg, zkgm_packet.salt, zkgm_packet.path, decode_syscall(zkgm_packet.syscall));
    }

    fun execute_internal(
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
            execute_batch(
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                decode_batch(syscall_packet.packet)
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
                relayer_msg,
                salt,
                path,
                decode_multiplex(syscall_packet.packet)
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun execute_fungible_asset_transfer(
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        transfer_packet: FungibleAssetTransferPacket
    ): (vector<u8>) acquires RelayStore, SignerRef {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        if (transfer_packet.only_maker) {
            return ACK_ERR_ONLYMAKER;
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
                    path,
                    ibc::packet::destination_channel(&ibc_packet),
                    transfer_packet.sent_token
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

    fun execute_batch(
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
                execute_internal(
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
        relayer_msg: vector<u8>,
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
        let packet_hash = commitment::commit_packet(&ibc_packet);
        // TODO: send_packet returns packet now instead of u64
        // Fix it later `commitment::commit_packet(&ibc_packet);`  is completely wrong
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.in_flight_packet, packet_hash, ibc_packet);
        ACK_EMPTY
    }

    fun execute_multiplex(
        ibc_packet: Packet,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        multiplex_packet: MultiplexPacket
    ): (vector<u8>) {
        let contract_address = from_bcs::to_address(multiplex_packet.contract_address);
        if (multiplex_packet.eureka) {
            // TODO: No idea how to do this?
            /*
                IEurekaModule(contractAddress).onZkgm(
                    multiplexPacket.sender, multiplexPacket.contractCalldata
                );
            */
            return bcs::to_bytes(&ACK_SUCCESS);
        };
        let multiplex_ibc_packet = ibc::packet::new(
            0, // TODO: sequence is no longer exist
            ibc::packet::source_channel(&ibc_packet),
            ibc::packet::destination_channel(&ibc_packet),
            multiplex_packet.contract_calldata, // TODO: abi.encode multiplexPacket.sender & multiplexPacket.contractCalldata
            ibc::packet::timeout_height(&ibc_packet),
            ibc::packet::timeout_timestamp(&ibc_packet)
        );
        // TODO: Call
        // bytes memory acknowledgement = IIBCModule(contractAddress)
        //  .onRecvPacket(multiplexIbcPacket, relayer, relayerMsg);
        // not sure how?
        let acknowledgement = vector::empty();
        if (vector::length(&acknowledgement) == 0){
            abort E_UNIMPLEMENTED;
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
            verify_batch(sender, channel_id, path, decode_batch(syscall_packet.packet))
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
        path: u256,
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
            forward_packet.channel_id,
            update_channel_path(path, forward_packet.channel_id),
            forward_packet.syscall_packet
        );
    }

    fun verify_multiplex(
        sender: &signer,
        channel_id: u32,
        path: u256,
        multiplex_packet: MultiplexPacket
    ) { }

    // public entry fun on_packet(
    //     ibc_packet: Packet,
    //     relayer_msg: vector<u8>
    // ) {
    //     let packet_data = ibc::packet::data(&ibc_packet);
    //     let zkgm_packet = decode_packet(*packet_data);
    //     execute_internal(ibc_packet, relayer_msg, zkgm_packet.salt, zkgm_packet.path, decode_syscall(zkgm_packet.syscall))
    // }


    // public fun on_packet<T: key>(_store: Object<T>): u64 acquires RelayStore, SignerRef {
    //     let value: copyable_any::Any = dispatcher::get_data(new_ucs_relay_proof());
    //     let type_name_output = *copyable_any::type_name(&value);

    //     if (type_name_output == std::type_info::type_name<ibc::RecvPacketParams>()) {
    //         let (pack) =
    //             helpers::on_recv_packet_deconstruct(
    //                 copyable_any::unpack<ibc::RecvPacketParams>(value)
    //             );
    //         on_recv_packet(pack);
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::AcknowledgePacketParams>()) {
    //         let (pack, acknowledgement) =
    //             helpers::on_acknowledge_packet_deconstruct(
    //                 copyable_any::unpack<ibc::AcknowledgePacketParams>(value)
    //             );
    //         on_acknowledge_packet(pack, acknowledgement);
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::TimeoutPacketParams>()) {
    //         let (pack) =
    //             helpers::on_timeout_packet_deconstruct(
    //                 copyable_any::unpack<ibc::TimeoutPacketParams>(value)
    //             );
    //         on_timeout_packet(pack);
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::ChannelOpenInitParams>()) {
    //         let (connection_id, channel_id, version) =
    //             helpers::on_channel_open_init_deconstruct(
    //                 copyable_any::unpack<ibc::ChannelOpenInitParams>(value)
    //             );
    //         on_channel_open_init(connection_id, channel_id, version);
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::ChannelOpenTryParams>()) {
    //         let (
    //             connection_id,
    //             channel_id,
    //             counterparty_channel_id,
    //             version,
    //             counterparty_version
    //         ) =
    //             helpers::on_channel_open_try_deconstruct(
    //                 copyable_any::unpack<ibc::ChannelOpenTryParams>(value)
    //             );
    //         on_channel_open_try(
    //             connection_id,
    //             channel_id,
    //             counterparty_channel_id,
    //             version,
    //             counterparty_version
    //         );
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::ChannelOpenAckParams>()) {
    //         let (channel_id, counterparty_channel_id, counterparty_version) =
    //             helpers::on_channel_open_ack_deconstruct(
    //                 copyable_any::unpack<ibc::ChannelOpenAckParams>(value)
    //             );
    //         on_channel_open_ack(
    //             channel_id, counterparty_channel_id, counterparty_version
    //         );
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::ChannelOpenConfirmParams>()) {
    //         let channel_id =
    //             helpers::on_channel_open_confirm_deconstruct(
    //                 copyable_any::unpack<ibc::ChannelOpenConfirmParams>(value)
    //             );
    //         on_channel_open_confirm(channel_id);
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::ChannelCloseInitParams>()) {
    //         let channel_id =
    //             helpers::on_channel_close_init_deconstruct(
    //                 copyable_any::unpack<ibc::ChannelCloseInitParams>(value)
    //             );
    //         on_channel_close_init(channel_id);
    //     } else if (type_name_output
    //         == std::type_info::type_name<ibc::ChannelCloseConfirmParams>()) {
    //         let channel_id =
    //             helpers::on_channel_close_confirm_deconstruct(
    //                 copyable_any::unpack<ibc::ChannelCloseConfirmParams>(value)
    //             );
    //         on_channel_close_confirm(channel_id);
    //     } else {
    //         std::debug::print(
    //             &string::utf8(b"Invalid function type detected in on_packet function!")
    //         );
    //     };
    //     0
    // }


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