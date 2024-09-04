module UCS01::Relay {    
    use IBC::Core;
    use IBC::channel;
    use IBC::height;
    use IBC::packet::{Self, Packet};
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};

    use std::string::{Self, String};
    use std::string_utils;
    use std::from_bcs;
    use aptos_framework::fungible_asset::{Self, MintRef, TransferRef, BurnRef, Metadata, FungibleAsset};
    use std::bcs;
    use aptos_framework::signer;
    use aptos_std::smart_table::{Self, SmartTable};
    use std::vector;
    use aptos_framework::coin;
    use UCS01::fa_coin;

    const ASSET_SYMBOL: vector<u8> = b"FA";
    // Constants
    const ORDER_UNORDERED: u8 = 1;
    const VERSION: vector<u8> = b"ucs01-relay-1";
    const ACK_SUCCESS: u8 = 1;
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;

    // Errors 
    const VAULT_SEED: vector<u8> = b"Relay Store Vault";
    const E_INVALID_BYTES_ADDRESS: u64 = 1;
    const E_UNAUTHORIZED: u64 = 2;
    const E_INVALID_ACKNOWLEDGEMENT: u64 = 3;
    const E_INVALID_PROTOCOL_VERSION: u64 = 4;
    const E_INVALID_PROTOCOL_ORDERING: u64 = 5;
    const E_INVALID_COUNTERPARTY_PROTOCOL_VERSION: u64 = 6;
    const E_INVALID_AMOUNT: u64 = 7;
    const E_UNSTOPPABLE: u64 = 8;

    struct Token has copy, drop, store {
        denom: String,
        amount: u128,
    }

    struct RelayPacket has copy, drop, store {
        sender: address,
        receiver: address,
        tokens: vector<Token>,
        extension: String,
    }

    struct DenomToAddressPair has copy, drop, store {
        source_channel: String,
        denom: String,
    }

    struct AddressToDenomPair has copy, drop, store {
        source_channel: String,
        denom: address,
    }

    struct OutstandingPair has copy, drop, store {
        source_channel: String,
        token: address,
    }

    struct RelayStore has key {
        denom_to_address: SmartTable<DenomToAddressPair, address>,
        address_to_denom: SmartTable<AddressToDenomPair, String>,
        outstanding: SmartTable<OutstandingPair, u64>,
    }



    struct SignerRef has key {
        self_ref: object::ExtendRef,
    }

    // Events
    #[event]
    struct DenomCreated has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        denom: String,
        token: address,
    }

    #[event]
    struct Received has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: address,
        receiver: address,
        denom: String,
        token: address,
        amount: u128,
    }

    #[event]
    struct Sent has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: address,
        receiver: address,
        denom: String,
        token: address,
        amount: u128,
    }

    #[event]
    struct Refunded has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: address,
        receiver: address,
        denom: String,
        token: address,
        amount: u128,
    }

    // View/Pure Functions
    public fun is_valid_version(version: String): bool {
        let version_bytes = string::bytes(&version);
        *version_bytes == VERSION
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

        let i = 0;
        while (i < prefix_len) {
            if (vector::borrow(&s_bytes, i) != vector::borrow(&prefix_bytes, i)) {
                return false
            };
            i = i + 1;
        };
        true
    }


    public fun is_from_channel(port_id: String, channel_id: String, denom: String): bool {
        let prefix = make_denom_prefix(port_id, channel_id);
        starts_with(denom, prefix)
    }

    public fun make_denom_prefix(port_id: String, channel_id: String): String {
        let prefix = port_id;
        string::append_utf8(&mut prefix, b"/");
        string::append(&mut prefix, channel_id);
        string::append_utf8(&mut prefix, b"/");
        prefix
    }

    public fun make_foreign_denom(port_id: String, channel_id: String, denom: String): String {
        let foreign_denom = make_denom_prefix(port_id, channel_id);
        string::append(&mut foreign_denom, denom);
        foreign_denom
    }

    public fun get_denom_address(source_channel: String, denom: String): address acquires RelayStore {
        let store = borrow_global<RelayStore>(get_vault_addr());
        let pair = DenomToAddressPair { source_channel, denom };
        *smart_table::borrow_with_default(&store.denom_to_address, pair, &@0x0)
    }

    public fun get_outstanding(source_channel: String, token: address): u64 acquires RelayStore {
        let store = borrow_global<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        *smart_table::borrow_with_default(&store.outstanding, pair, &0)
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@UCS01, VAULT_SEED)
    }

    public fun get_ucs_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun increase_outstanding(source_channel: String, token: address, amount: u64) acquires RelayStore {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        let current_outstanding = smart_table::borrow_mut_with_default(&mut store.outstanding, pair, 0);
        *current_outstanding = *current_outstanding + amount;
    }

    public fun decrease_outstanding(source_channel: String, token: address, amount: u64) acquires RelayStore {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        let current_outstanding = smart_table::borrow_mut(&mut store.outstanding, pair);
        *current_outstanding = *current_outstanding - amount;
    }



    // Initialize the RelayStore and SignerRef
    public fun initialize_store(account: &signer) {
        assert!(signer::address_of(account) == @UCS01, E_UNAUTHORIZED);

        let vault_constructor_ref = &object::create_named_object(account, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = RelayStore {
            denom_to_address: smart_table::new<DenomToAddressPair, address>(),
            address_to_denom: smart_table::new(),
            outstanding: smart_table::new(),
        };

        move_to(vault_signer, store);

        move_to(vault_signer, SignerRef {
            self_ref: object::generate_extend_ref(vault_constructor_ref)
        });
    }

    public fun chan_open_init(
        port_id: String,
        connection_hops: vector<String>,
        ordering: u8,
        counterparty: channel::Counterparty,
        version: String,
    ) acquires SignerRef {
        Core::channel_open_init(
            &get_ucs_signer(),
            port_id,
            connection_hops,
            ordering,
            counterparty,
            version,
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (ordering != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };
    }


    public fun chan_open_try(
        port_id: String,
        connection_hops: vector<String>,
        ordering: u8,
        counterparty: channel::Counterparty,
        counterparty_version: String,
        version: String,
        proof_init: vector<u8>,
        proof_height: height::Height,
    ) acquires SignerRef {
        Core::channel_open_try(
            &get_ucs_signer(),
            port_id,
            connection_hops,
            ordering,
            counterparty,
            counterparty_version,
            version,
            proof_init,
            proof_height,
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (ordering != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };

        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public fun chan_open_ack(
        port_id: String,
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: vector<u8>,
        proof_height: height::Height
    ) acquires SignerRef {
        // Store the channel_id
        Core::channel_open_ack(
            &get_ucs_signer(),
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            proof_height
        );
        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public fun chan_open_confirm(
        port_id: String,
        channel_id: String,
        proof_ack: vector<u8>,
        proof_height: height::Height
    ) acquires SignerRef {
        Core::channel_open_confirm(
            &get_ucs_signer(),
            port_id,
            channel_id,
            proof_ack,
            proof_height
        );
    }
    public fun chan_close_init(
        _port_id: String,
        _channel_id: String
    ) {
        abort E_UNSTOPPABLE
    }

    public fun chan_close_confirm(
        _port_id: String,
        _channel_id: String
    ) {
        abort E_UNSTOPPABLE
    }

    public fun timeout_packet(
        _packet: Packet
    )  {
        // TODO: call refund tokens
        // refund_tokens(sequence(packet), source_channel(packet), decode(packet.data));
    }

     public fun send_token(
        sender: &signer,
        token: Object<Metadata>,
        source_channel: String,
        denom: String,
        amount: u64,
    ): address acquires RelayStore {
        if (amount == 0) {
            abort E_INVALID_AMOUNT;
        };

        let store = borrow_global<RelayStore>(get_vault_addr());

        let pair = DenomToAddressPair { source_channel, denom };
        let token_address = *smart_table::borrow_with_default(&store.denom_to_address, pair, &@0x0);

        if (token_address == @0x0) {
            // transferring to the zero address is basically burning
            primary_fungible_store::transfer(sender, token, @zero_account, amount);
        } else {
            primary_fungible_store::transfer(sender, token, @UCS01, amount);
            increase_outstanding(source_channel, token_address, amount);
        };
        token_address
    }
    
    #[test]
    public fun test_is_valid_version() {
        let valid_version = string::utf8(b"ucs01-relay-1");
        let invalid_version = string::utf8(b"invalid-version");
        
        // Test with valid version
        assert!(is_valid_version(valid_version), 100);

        // Test with invalid version
        assert!(!is_valid_version(invalid_version), 101);
    }


    #[test]
    public fun test_is_from_channel() {
        let port_id = string::utf8(b"port-1");
        let channel_id = string::utf8(b"channel-1");
        let valid_denom = string::utf8(b"port-1/channel-1/denom");
        let invalid_denom = string::utf8(b"other-port/other-channel/denom");
        
        // Test with valid denom
        assert!(is_from_channel(port_id, channel_id, valid_denom), 200);

        // Test with invalid denom
        assert!(!is_from_channel(port_id, channel_id, invalid_denom), 201);
    }

    #[test]
    public fun test_make_denom_prefix() {
        let port_id = string::utf8(b"port-1");
        let channel_id = string::utf8(b"channel-1");
        let expected_prefix = string::utf8(b"port-1/channel-1/");
        
        let result = make_denom_prefix(port_id, channel_id);
        assert!(result == expected_prefix, 300);
    }

    #[test]
    public fun test_make_foreign_denom() {
        let port_id = string::utf8(b"port-1");
        let channel_id = string::utf8(b"channel-1");
        let denom = string::utf8(b"denom");
        let expected_foreign_denom = string::utf8(b"port-1/channel-1/denom");
        
        let result = make_foreign_denom(port_id, channel_id, denom);
        assert!(result == expected_foreign_denom, 400);
    }

    #[test(admin = @UCS01)]
    public fun test_get_denom_address(admin: &signer) acquires RelayStore {
        // Initialize the store in the admin's account
        initialize_store(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = string::utf8(b"denom-1");
        let expected_address: address = @0x1;

        let pair = DenomToAddressPair {
            source_channel,
            denom,
        };
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.denom_to_address, pair, expected_address);
    
        // Test getting the address
        let result = get_denom_address(source_channel, denom);
        assert!(result == expected_address, 500);
    }

    #[test(admin = @UCS01)]    
    public fun test_get_outstanding(admin: &signer) acquires RelayStore {
        // Initialize the store in the admin's account
        initialize_store(admin);

        let source_channel = string::utf8(b"channel-1");
        let token = @0x1;
        let expected_amount: u64 = 1000;

        // Set up the mapping in the Relay module (this is usually done through an entry function)
        let pair = OutstandingPair {
            source_channel: source_channel,
            token: token,
        };
        
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.outstanding, pair, expected_amount);

        // Test getting the outstanding amount
        let result = get_outstanding(source_channel, token);
        assert!(result == expected_amount, 600);
    }
    
    #[test(admin = @UCS01)]    
    public fun test_increase_outstanding(admin: &signer) acquires RelayStore {
        // Initialize the store
        let source_channel = string::utf8(b"channel-1");
        let token_address: address = @0x1;
        let initial_amount: u64 = 1000;
        
        // Initialize the store in the admin's account
        initialize_store(admin);

        // Increase outstanding amount
        increase_outstanding(source_channel, token_address, initial_amount);

        // Verify that the outstanding amount is updated correctly
        let outstanding_amount = get_outstanding(source_channel, token_address);
        assert!(outstanding_amount == initial_amount, 700);
    }

    #[test(admin = @UCS01)]    
    public fun test_decrease_outstanding(admin: &signer) acquires RelayStore {
        // Initialize the store
        let source_channel = string::utf8(b"channel-1");
        let token_address: address = @0x1;
        let initial_amount: u64 = 1000;
        let decrease_amount: u64 = 400;

        // Initialize the store in the admin's account
        initialize_store(admin);

        // First, increase outstanding amount
        increase_outstanding(source_channel, token_address, initial_amount);

        // Decrease the outstanding amount
        decrease_outstanding(source_channel, token_address, decrease_amount);

        // Verify that the outstanding amount is updated correctly
        let outstanding_amount = get_outstanding(source_channel, token_address);
        let expected_amount = initial_amount - decrease_amount;
        assert!(outstanding_amount == expected_amount, 701);
    }

    const TEST_NAME: vector<u8> = b"Test Coin";
    const TEST_SYMBOL: vector<u8> = b"TST";
    const TEST_DECIMALS: u8 = 8;
    const TEST_ICON: vector<u8> = b"https://example.com/icon.png";
    const TEST_PROJECT: vector<u8> = b"Test Project";



    #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    public fun test_send_token_valid_address(admin: &signer, bob: &signer, alice: address) acquires RelayStore {
        // Initialize the store
        initialize_store(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = string::utf8(b"test-denom");
        let amount: u64 = 1000;

        // Upsert denom to address pair
        let pair = DenomToAddressPair {
            source_channel,
            denom,
        };
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.denom_to_address, pair, alice);

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let asset = UCS01::fa_coin::get_metadata();
        let bob_addr = signer::address_of(bob);
        UCS01::fa_coin::mint(admin, bob_addr, amount);

        // Send tokens
        let result_address = send_token(bob, asset, source_channel, denom, amount);

        // Verify the result and outstanding balance
        assert!(result_address == alice, 100);
        let outstanding_balance = get_outstanding(source_channel, alice);
        assert!(outstanding_balance == amount, 101);

        let bob_balance = primary_fungible_store::balance(bob_addr, asset);
        let ucs01_balance = primary_fungible_store::balance(@UCS01, asset);
        assert!(bob_balance == 0, 102);
        assert!(ucs01_balance == 1000, 102);
    }

    #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    public fun test_send_token_burn(admin: &signer, bob: &signer, alice: address) acquires RelayStore {
        // Initialize the store
        initialize_store(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = string::utf8(b"burn-denom");
        let amount: u64 = 1000;

        // Upsert denom to address pair
        let pair = DenomToAddressPair {
            source_channel,
            denom,
        };
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.denom_to_address, pair, @0x0); // added 0x0 so it'll be burned

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let asset = UCS01::fa_coin::get_metadata();
        let bob_addr = signer::address_of(bob);
        UCS01::fa_coin::mint(admin, bob_addr, amount);

        // Send tokens
        let result_address = send_token(bob, asset, source_channel, denom, amount);

        // Verify the result and outstanding balance
        assert!(result_address == @0x0, 100); 
        let outstanding_balance = get_outstanding(source_channel, alice);
        assert!(outstanding_balance == 0, 101);

        let bob_balance = primary_fungible_store::balance(bob_addr, asset);
        let ucs01_balance = primary_fungible_store::balance(@UCS01, asset);
        assert!(bob_balance == 0, 102);
        assert!(ucs01_balance == 0, 102);
    }

    #[test(admin = @UCS01, alice = @0x1234)]
    #[expected_failure(abort_code = E_INVALID_AMOUNT)]
    public fun test_send_zero_amount(admin: &signer, alice: address) acquires RelayStore {
        // Initialize the store
        initialize_store(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = string::utf8(b"zero-amount-denom");

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let asset = UCS01::fa_coin::get_metadata();

        // Attempt to send zero amount
        send_token(admin, asset, source_channel, denom, 0);
    }
}
