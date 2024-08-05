module IBC::ChannelTest {

    use std::signer;
    use std::account;
    use std::vector;
    use std::debug;
    use aptos_std::string::{Self, String};
    use aptos_framework::coin::Coin;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::event;
    use aptos_std::any::{Self, Any};
    use IBC::height;
    use IBC::Core::{Self, Version};
    use IBC::LightClient;


    use aptos_framework::smart_table::{Self as SmartTable, SmartTable};
    const E_GENERATE_CLIENT_IDENTIFIER: u64 = 3001;
    const E_GET_CLIENT_IMPL: u64 = 3002;
    const E_CREATE_CLIENT: u64 = 3003;

    const ORDER_UNORDERED: u8 = 1;
    const ORDER_ORDERED: u8 = 2;
    const ORDER_INVALID: u8 = 3;

    #[test]
    public fun test_verify_supported_feature() {
        // Prepare the version with features
        let features = vector::empty<String>();
        vector::push_back(&mut features, string::utf8(b"FEATURE_A"));
        vector::push_back(&mut features, string::utf8(b"FEATURE_B"));


        let version = Core::new_version(string::utf8(b"1"), features);
        // Test case where the feature is supported
        let feature_a = string::utf8(b"FEATURE_A");
        let is_supported = Core::verify_supported_feature(&version, feature_a);
        assert!(is_supported, 1001);

        // Test case where the feature is not supported
        let feature_c = string::utf8(b"FEATURE_C");
        let is_not_supported = Core::verify_supported_feature(&version, feature_c);
        assert!(!is_not_supported, 1002);
    }


    #[test]
    public fun test_to_string() {
        // Test case for ORDER_UNORDERED
        let order_unordered = Core::to_string(ORDER_UNORDERED);
        assert!(order_unordered == string::utf8(b"ORDER_UNORDERED"), 2001);

        // Test case for ORDER_ORDERED
        let order_ordered = Core::to_string(ORDER_ORDERED);
        assert!(order_ordered == string::utf8(b"ORDER_ORDERED"), 2002);

        // Test case for invalid order
        let order_invalid = Core::to_string(ORDER_INVALID);
        assert!(order_invalid == string::utf8(b"ORDER_INVALID"), 2003);
    }

    #[test(alice = @IBC)]
    public fun test_get_counterparty_hops(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Mock connection ID
        let connection_id = string::utf8(b"connection-0");

        // Prepare counterparty and connection
        let counterparty = Core::new_connection_counterparty(string::utf8(b"counterparty-client"), connection_id, Core::new_merkleprefix(vector::empty<u8>()));
        let connection = Core::new_connection_end(string::utf8(b"client-0"), vector::empty<Version>(), 3, 0, counterparty);

        // Insert connection into the store
        Core::set_connection(connection_id, connection);

        // Test get_counterparty_hops function
        let hops = Core::get_counterparty_hops(connection_id);
        assert!(vector::length(&hops) == 1, 3001);
        assert!(*vector::borrow(&hops, 0) == connection_id, 3002);
    }

    #[test(alice = @IBC)]
    public fun test_generate_channel_identifier(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Test generate_channel_identifier function
        let channel_id = Core::generate_channel_identifier();

        // Verify the next sequence has incremented
        let next_sequence = Core::get_next_channel_sequence();
        assert!(next_sequence == 1, 4002);
    }

    #[test(alice = @IBC)]
    public fun test_ensure_connection_state(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Mock connection ID
        let connection_id = string::utf8(b"connection-0");

        // Prepare counterparty and connection
        let counterparty = Core::new_connection_counterparty(string::utf8(b"counterparty-client"), connection_id, Core::new_merkleprefix(vector::empty<u8>()));
        let connection = Core::new_connection_end(string::utf8(b"client-0"), vector::empty<Version>(), 3, 0, counterparty);

        Core::set_connection(connection_id, connection);

        // Test ensure_connection_state function
        let retrieved_connection_end = Core::ensure_connection_state(connection_id);
        let (client_id, versions, state, delay_period, counterparty) = Core::get_connection_end(retrieved_connection_end);
        assert!(state == 3, 5001);
        assert!(client_id == string::utf8(b"client-0"), 5002);
    }

    #[test(alice = @IBC)]
    public fun test_ensure_connection_feature(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Mock connection ID
        let connection_id = string::utf8(b"connection-0");

        // Prepare counterparty and connection
        let features = vector::empty<String>();
        vector::push_back(&mut features, string::utf8(b"ORDER_ORDERED"));
        let version = Core::new_version(string::utf8(b"1"), features);
        let counterparty = Core::new_connection_counterparty(string::utf8(b"counterparty-client"), connection_id, Core::new_merkleprefix(vector::empty<u8>()));
        let connection = Core::new_connection_end(string::utf8(b"client-0"), vector::singleton(version), 3, 0, counterparty);

        Core::set_connection(connection_id, connection);

        // Test ensure_connection_feature function
        let connection_hops = vector::singleton(connection_id);
        let (retrieved_connection_id, retrieved_connection_end) = Core::ensure_connection_feature(connection_hops, ORDER_ORDERED);
        let (client_id, versions, state, delay_period, counterparty) = Core::get_connection_end(retrieved_connection_end);
        assert!(retrieved_connection_id == connection_id, 6001);
        assert!(state == 3, 6002);
        assert!(client_id == string::utf8(b"client-0"), 6003);
    }

    #[test]
    public fun test_is_lowercase() {
        // Test case where the string is lowercase
        let lowercase_string = string::utf8(b"lowercase");
        let is_lowercase = Core::is_lowercase(&lowercase_string);
        assert!(is_lowercase, 7001);

        // Test case where the string is not lowercase
        let mixedcase_string = string::utf8(b"MixedCase");
        let is_not_lowercase = Core::is_lowercase(&mixedcase_string);
        assert!(!is_not_lowercase, 7002);

        // Test case where the string is not lowercase
        let mixedcase_string = string::utf8(b"ItsWrong");
        let is_not_lowercase = Core::is_lowercase(&mixedcase_string);
        assert!(!is_not_lowercase, 7002);


        let lowercase_string = string::utf8(b"thisshouldbecorrecttoo");
        let is_lowercase = Core::is_lowercase(&lowercase_string);
        assert!(is_lowercase, 7001);
    }
}
