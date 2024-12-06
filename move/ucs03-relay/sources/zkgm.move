module ibc::zkgm {
    use ibc::ibc;
    use ibc::helpers;
    use ibc::packet::{Packet};
    use ibc::dispatcher;
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use aptos_std::copyable_any;
    use std::event;
    use aptos_framework::function_info;

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
    const VERSION: vector<u8> = b"ucs03-relay-1";
    const ACK_SUCCESS: vector<u8> = b"1";
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;
    use std::option;

    // Errors
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const E_INVALID_BYTES_ADDRESS: u64 = 1;
    const E_UNAUTHORIZED: u64 = 2;
    const E_INVALID_ACKNOWLEDGEMENT: u64 = 3;
    const E_INVALID_PROTOCOL_VERSION: u64 = 4;
    const E_INVALID_COUNTERPARTY_PROTOCOL_VERSION: u64 = 6;
    const E_INVALID_AMOUNT: u64 = 7;
    const E_UNSTOPPABLE: u64 = 8;

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
        tag: u64,
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

        move_to(
            vault_signer,
            SignerRef {
                self_ref: object::generate_extend_ref(vault_constructor_ref),
                self_address: signer::address_of(account)
            }
        );
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
    }
}