module UCS01::Relay {    
    use IBC::Core;
    use IBC::channel;
    use IBC::height;
    use IBC::packet::{Self, Packet};
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use std::event;

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
        amount: u64,
    }

    struct LocalToken has copy, drop, store {
        denom: address,
        amount: u64,
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
        amount: u64,
    }

    #[event]
    struct Sent has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: address,
        receiver: address,
        denom: String,
        token: address,
        amount: u64,
    }

    #[event]
    struct Refunded has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: address,
        receiver: address,
        denom: String,
        token: address,
        amount: u64,
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

    public fun get_metadata(asset_addr: address): Object<Metadata> {
        object::address_to_object<Metadata>(asset_addr)
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

    // TODO: No idea if this encode_packet is correct or not and how to write decode_packet
    // take a look at here later.
    public fun encode_packet(packet: &RelayPacket): vector<u8> {
        let buf = vector::empty<u8>();

        let sender_bytes = bcs::to_bytes(&packet.sender);
        vector::append(&mut buf, sender_bytes);

        let receiver_bytes = bcs::to_bytes(&packet.receiver);
        vector::append(&mut buf, receiver_bytes);

        let num_tokens = vector::length(&packet.tokens);
        let num_tokens_bytes = bcs::to_bytes(&(num_tokens as u256));
        vector::reverse(&mut num_tokens_bytes);
        std::debug::print(&num_tokens);
        std::debug::print(&num_tokens_bytes);
        vector::append(&mut buf, num_tokens_bytes);

        std::debug::print(&buf);

        let i = 0;
        while (i < num_tokens) {
            let token = vector::borrow(&packet.tokens, i);
            
            let denom_bytes = bcs::to_bytes(&token.denom);
            let denom_len_bytes = bcs::to_bytes(&vector::length(&denom_bytes));
            vector::append(&mut buf, denom_len_bytes);
            vector::append(&mut buf, denom_bytes);

            let amount_bytes = bcs::to_bytes(&token.amount);
            vector::append(&mut buf, amount_bytes);

            i = i + 1;
        };

        let extension_bytes = bcs::to_bytes(&packet.extension);
        let extension_len_bytes = bcs::to_bytes(&vector::length(&extension_bytes));
        vector::append(&mut buf, extension_len_bytes);
        vector::append(&mut buf, extension_bytes);

        buf
    }

    public fun decode_packet(data: &vector<u8>): RelayPacket {
        let offset = 0;

        // Decode sender address (address size is 32 bytes)
        let sender_bytes = vector::slice(data, offset, 32);
        let sender: address = from_bcs::to_address(sender_bytes);
        offset = offset + 32;

        // Decode receiver address (address size is 32 bytes)
        let receiver_bytes = vector::slice(data, offset, 32);
        let receiver: address = from_bcs::to_address(receiver_bytes);
        offset = offset + 32;

        // Decode the number of tokens (u32 for simplicity)
        let num_tokens_bytes = vector::slice(data, offset, 4);
        let num_tokens: u32 = from_bcs::to_u32(num_tokens_bytes);
        offset = offset + 4;

        // Decode tokens
        let tokens: vector<Token> = vector::empty<Token>();
        let i = 0;
        while (i < num_tokens) {
            // Decode token denom length (u32 for simplicity)
            let denom_len_bytes = vector::slice(data, offset, 4);
            let denom_len: u64 = from_bcs::to_u64(denom_len_bytes);
            offset = offset + 4;

            // Decode token denom (String)
            let denom_bytes = vector::slice(data, offset, denom_len);
            let denom: String = from_bcs::to_string(denom_bytes);
            offset = offset + denom_len;

            // Decode token amount (u64)
            let amount_bytes = vector::slice(data, offset, 8);
            let amount: u64 = from_bcs::to_u64(amount_bytes);
            offset = offset + 8;

            // Push the decoded Token into the tokens vector
            let token = Token { denom, amount };
            vector::push_back(&mut tokens, token);
            i = i + 1;
        };

        // Decode extension length (u32 for simplicity)
        let extension_len_bytes = vector::slice(data, offset, 4);
        let extension_len: u64 = from_bcs::to_u64(extension_len_bytes);
        offset = offset + 4;

        // Decode extension (String)
        let extension_bytes = vector::slice(data, offset, extension_len);
        let extension: String = from_bcs::to_string(extension_bytes);

        // Return the decoded RelayPacket
        RelayPacket {
            sender,
            receiver,
            tokens,
            extension,
        }
    }

    
    // fun refund_tokens(
    //     sequence: u64,
    //     channel_id: String,
    //     packet: &RelayPacket
    // ) acquires RelayStore {
    //     let receiver = packet.receiver;
    //     let user_to_refund = packet.sender;

    //     let packet_tokens_length = vector::length(&packet.tokens);
    //     let i = 0;
    //     while (i < packet_tokens_length) {
    //         let token = vector::borrow(&packet.tokens, i);
    //         let denom_address = get_denom_address(channel_id, token.denom);

    //         if (denom_address != @0x0) {
    //             // The token originated from the remote chain (burnt on send), so refunding means minting.
    //             UCS01::fa_coin::mint(&get_ucs_signer(), user_to_refund, token.amount);
    //         } else {
    //             // The token originated from the local chain (escrowed on send), so refunding means unescrowing.
    //             denom_address = from_bcs::to_address(string::bytes(&token.denom));
    //             decrease_outstanding(channel_id, denom_address, token.amount);
    //             UCS01::fa_coin::transfer_escrowed_tokens(&get_ucs_signer(), user_to_refund, denom_address, token.amount);
    //         };

    //         // Emit a Refunded event
    //         event::emit(Refunded {
    //             packet_sequence: sequence,
    //             channel_id: channel_id,
    //             sender: user_to_refund,
    //             receiver: receiver,
    //             denom: token.denom,
    //             token: denom_address,
    //             amount: token.amount
    //         });

    //         i = i + 1;
    //     }
    // }



    public entry fun send(
        sender: &signer,
        source_channel: String,
        receiver: address,
        denom_list: vector<address>, 
        amount_list: vector<u64>, 
        extension: String,
        timeout_height_number: u64,
        timeout_height_height: u64,
        timeout_timestamp: u64
    ) acquires RelayStore {
        let num_tokens = vector::length(&denom_list);
        
        if(vector::length(&amount_list) != num_tokens) {
            abort E_INVALID_BYTES_ADDRESS;
        };

        let normalized_tokens: vector<Token> = vector::empty<Token>();

        let i = 0;
        while (i < num_tokens) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);
            
            let token_address = send_token(
                sender,
                source_channel,
                local_token_denom,
                local_token_amount
            );

            // Create a normalized Token struct and push to the vector
            let normalized_token = Token {
                denom: token_address,
                amount: local_token_amount
            };
            vector::push_back(&mut normalized_tokens, normalized_token);
            i = i + 1;
        };
        let packet: RelayPacket = RelayPacket {
            sender: signer::address_of(sender),
            receiver,
            tokens: normalized_tokens,
            extension
        };

        let timeout_height = height::new(timeout_height_number, timeout_height_height);

        let packet_sequence = IBC::Core::send_packet(
            sender,
            source_channel,
            timeout_height,
            timeout_timestamp,
            encode_packet(&packet)
        );

        let i = 0;
        while (i < num_tokens) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);
            let normalizedToken = *vector::borrow(&normalized_tokens, i);
            
            event::emit(Sent {
                packet_sequence: packet_sequence,
                channel_id: source_channel,
                sender: signer::address_of(sender),
                receiver: receiver,
                denom: normalizedToken.denom,
                token: local_token_denom,
                amount: local_token_amount
            });

        }
    }

     public fun send_token(
        sender: &signer,
        source_channel: String,
        denom: address,
        amount: u64,
    ): String acquires RelayStore {
        if (amount == 0) {
            abort E_INVALID_AMOUNT;
        };

        let store = borrow_global<RelayStore>(get_vault_addr());

        let pair = AddressToDenomPair { source_channel, denom };
        let token_address = *smart_table::borrow_with_default(&store.address_to_denom, pair, &string::utf8(b""));

        let token = get_metadata(denom);
        if (string::length(&token_address) == 0) {
            // transferring to the zero address is basically burning
            primary_fungible_store::transfer(sender, token, @zero_account, amount);
        } else {
            primary_fungible_store::transfer(sender, token, @UCS01, amount);
            increase_outstanding(source_channel, denom, amount);
            token_address = string_utils::to_string_with_canonical_addresses(&denom);
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
        let denom = UCS01::fa_coin::get_metadata_address();
        let amount: u64 = 1000;

        // Upsert denom to address pair
        let pair = AddressToDenomPair {
            source_channel,
            denom,
        };

        let new_denom = string::utf8(b"new-denom");
        let denom_str = string_utils::to_string_with_canonical_addresses(&denom);
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.address_to_denom, pair, new_denom);

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let asset_addr = UCS01::fa_coin::get_metadata_address();
        let asset = get_metadata(asset_addr);
        let bob_addr = signer::address_of(bob);
        UCS01::fa_coin::mint(admin, bob_addr, amount);

        // Send tokens
        let result_address = send_token(bob, source_channel, asset_addr, amount);

        // Verify the result and outstanding balance
        assert!(result_address == denom_str, 100);
        let outstanding_balance = get_outstanding(source_channel, denom);
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
        let denom = @0x111111;
        let amount: u64 = 1000;

        // Upsert denom to address pair
        let pair = AddressToDenomPair {
            source_channel,
            denom,
        };

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        
        let asset_addr = UCS01::fa_coin::get_metadata_address();
        let asset = get_metadata(asset_addr);
        let bob_addr = signer::address_of(bob);
        UCS01::fa_coin::mint(admin, bob_addr, amount);

        // Send tokens
        let result_address = send_token(bob, source_channel, asset_addr, amount);

        // Verify the result and outstanding balance
        assert!(string::length(&result_address) == 0, 100); 
        let outstanding_balance = get_outstanding(source_channel, denom);
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
        let denom = @0x111111;

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        
        let asset_addr = UCS01::fa_coin::get_metadata_address();
        let asset = get_metadata(asset_addr);

        // Attempt to send zero amount
        send_token(admin, source_channel, asset_addr, 0);
    }

    #[test]
    public fun test_encode() {
        let token = Token {
            denom: string::utf8(b"denom"),
            amount: 1000,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);
        let packet = RelayPacket {
            sender: @0x1111111111111111111111111111111111111111,
            receiver: @0x0000000000000000000000000000000000000033,
            tokens: tokens,
            extension: string::utf8(b"extension"),
        };
        let encoded = encode_packet(&packet);
        // let decoded = decode_packet(&encoded);
        
        // std::debug::print(&encoded);
        // assert!(decoded.ping == packet.ping, 1);
        // assert!(decoded.counterparty_timeout == packet.counterparty_timeout, 2);
    }

    // #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    // public fun test_send_tokens(admin: &signer, alice: &signer, bob: address) acquires RelayStore {
    //     initialize_store(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let denom_list = vector::empty<address>();
    //     let amount_list = vector::empty<u64>();

    //     UCS01::fa_coin::initialize(
    //         admin,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT)
    //     );
    //     let asset_addr = UCS01::fa_coin::get_metadata_address();
    //     UCS01::fa_coin::mint(admin, signer::address_of(alice), 1000);

    //     vector::push_back(&mut denom_list, asset_addr);
    //     vector::push_back(&mut amount_list, 500);  // Send half of the minted amount

    //     let extension = string::utf8(b"optional-extension");
    //     let timeout_height_number = 1;
    //     let timeout_height_height = 100;
    //     let timeout_timestamp = 100000;

    //     send(
    //         alice,
    //         source_channel,
    //         bob,
    //         denom_list,
    //         amount_list,
    //         extension,
    //         timeout_height_number,
    //         timeout_height_height,
    //         timeout_timestamp
    //     );

    //     let outstanding_balance = get_outstanding(source_channel, asset_addr);
    //     assert!(outstanding_balance == 500, 100);  // Only 500 tokens were sent

    //     let ucs01_balance = primary_fungible_store::balance(@UCS01, get_metadata(asset_addr));
    //     assert!(ucs01_balance == 500, 101);  // 500 tokens should be transferred to UCS01

    //     let alice_balance = primary_fungible_store::balance(signer::address_of(alice), get_metadata(asset_addr));
    //     assert!(alice_balance == 500, 102);  // Alice should have 500 tokens left after sending

}
