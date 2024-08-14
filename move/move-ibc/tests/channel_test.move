#[test_only]
module IBC::ChannelTest {

    use std::signer;
    use std::vector;
    use aptos_std::string::{Self, String};
    use aptos_std::any;
    use IBC::height;
    use IBC::Core;
    use IBC::connection_end::{Self, Version};
    use IBC::channel;

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


        let version = connection_end::new_version(string::utf8(b"1"), features);
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
        let counterparty = connection_end::new_counterparty(string::utf8(b"counterparty-client"), connection_id, b"");
        let connection = connection_end::new(string::utf8(b"client-0"), vector::empty<Version>(), 3, 0, counterparty);

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
        let _ = Core::generate_channel_identifier();

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
        let counterparty = connection_end::new_counterparty(string::utf8(b"counterparty-client"), connection_id, vector::empty<u8>());
        let connection = connection_end::new(string::utf8(b"client-0"), vector::empty<Version>(), 3, 0, counterparty);

        Core::set_connection(connection_id, connection);

        // Test ensure_connection_state function
        let retrieved_connection_end = Core::ensure_connection_state(connection_id);
        assert!(connection_end::state(&retrieved_connection_end) == 3, 5002);
        assert!(*connection_end::client_id(&retrieved_connection_end) == string::utf8(b"client-0"), 5003);
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
        let version = connection_end::new_version(string::utf8(b"1"), features);
        let counterparty = connection_end::new_counterparty(string::utf8(b"counterparty-client"), connection_id, vector::empty<u8>());
        let connection = connection_end::new(string::utf8(b"client-0"), vector::singleton(version), 3, 0, counterparty);

        Core::set_connection(connection_id, connection);

        // Test ensure_connection_feature function
        let connection_hops = vector::singleton(connection_id);
        let (retrieved_connection_id, retrieved_connection_end) = Core::ensure_connection_feature(connection_hops, ORDER_ORDERED);
        assert!(retrieved_connection_id == connection_id, 6001);
        assert!(connection_end::state(&retrieved_connection_end) == 3, 6002);
        assert!(*connection_end::client_id(&retrieved_connection_end) == string::utf8(b"client-0"), 6003);
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

        let lowercase_string = string::utf8(b"port-0-also-ok");
        let is_lowercase = Core::is_lowercase(&lowercase_string);
        assert!(is_lowercase, 7001);
    }

    #[test(alice = @IBC)]
    public fun test_channel_open_init(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(connection_end::new_version(string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_ORDERED")))),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // debug::print(&string::utf8(b"Connection set in store"));

        // Prepare a mock channel
        let connection_hops = vector::singleton(connection_id);
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(1, ORDER_ORDERED, counterparty, connection_hops, string::utf8(b"1"));

        // Call channel_open_init function
        let channel_id = Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));

        // Validate that the channel was added to the store
        let stored_channel = Core::get_channel_from_store(string::utf8(b"port-0"), channel_id);

        // Validate that the stored channel matches the expected channel
        assert!(channel::state(&stored_channel) == 1, 8001);
        assert!(channel::ordering(&stored_channel) == ORDER_ORDERED, 8002);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1016)] //E_INVALID_CHANNEL_STATE
    public fun test_channel_open_init_invalid_state(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(connection_end::new_version(string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_ORDERED")))),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel with an invalid state (not STATE_INIT)
        let connection_hops = vector::singleton(connection_id);
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(3, ORDER_ORDERED, counterparty, connection_hops, string::utf8(b"1")); // Invalid state

        Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1017)] //E_COUNTERPARTY_CHANNEL_NOT_EMPTY
    public fun test_channel_open_init_non_empty_counterparty_channel_id(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(connection_end::new_version(string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_ORDERED")))),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel with a non-empty counterparty channel ID
        let connection_hops = vector::singleton(connection_id);
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b"channel-1"));
        let channel = channel::new(1, ORDER_ORDERED, counterparty, connection_hops, string::utf8(b"1")); // Non-empty counterparty channel ID

        Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));
    }
    #[test(alice = @IBC)]
public fun test_channel_open_ack(alice: &signer) {
    // Initialize IBCStore for testing
    Core::create_ibc_store(alice);

    // Prepare a mock connection and set it in the IBCStore
    let client_id = string::utf8(b"client-0");
    let connection_id = string::utf8(b"connection-0");
    let counterparty = connection_end::new_counterparty(
        string::utf8(b"counterparty-client"),
        connection_id,
        b"",
    );
    let connection = connection_end::new(
        client_id,
        vector::singleton(connection_end::new_version(string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_ORDERED")))),
        3, // STATE_OPEN
        0,
        counterparty
    );
    Core::set_connection(connection_id, connection);

    // Prepare a mock channel
    let connection_hops = vector::singleton(connection_id);
    let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
    let channel = channel::new(1, ORDER_ORDERED, counterparty, connection_hops, string::utf8(b"1"));
    let channel_id = Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));

    // Prepare mock proof data
    let proof_height = height::new(0, 1);
    let proof_try = any::pack(vector::empty<u8>());

    // Call channel_open_ack function
    Core::channel_open_ack(
        string::utf8(b"port-0"),
        channel_id,
        string::utf8(b"counterparty-channel-0"),
        string::utf8(b"counterparty-version-0"),
        proof_try,
        proof_height
    );

    // Validate that the channel state has been updated to STATE_OPEN
    let stored_channel = Core::get_channel_from_store(string::utf8(b"port-0"), channel_id);
    assert!(channel::state(&stored_channel) == 3, 9001); // STATE_OPEN
    assert!(*channel::version(&stored_channel) == string::utf8(b"counterparty-version-0"), 9002);
    
    assert!(*channel::chan_counterparty_channel_id(&stored_channel) == string::utf8(b"counterparty-channel-0"), 9003);
}

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1016)] // E_INVALID_CHANNEL_STATE
    public fun test_channel_open_ack_invalid_state(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(connection_end::new_version(string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_ORDERED")))),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel with an invalid state (not STATE_INIT)
        let connection_hops = vector::singleton(connection_id);
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(3, ORDER_ORDERED, counterparty, connection_hops, string::utf8(b"1"));
        let channel_id = Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));

        // Prepare mock proof data
        let proof_height = height::new(0, 1);
        let proof_try = any::pack(vector::empty<u8>());

        // Call channel_open_ack function
        Core::channel_open_ack(
            string::utf8(b"port-0"),
            channel_id,
            string::utf8(b"counterparty-channel-0"),
            string::utf8(b"counterparty-version-0"),
            proof_try,
            proof_height
        );
    }
    #[test(alice = @IBC)]
    public fun test_channel_open_confirm_success(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(Core::default_ibc_version()),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel
        let connection_hops = vector::singleton(connection_id);
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(2, 1, counterparty_channel, connection_hops, string::utf8(b"1")); // STATE_TRYOPEN
        let port_id = string::utf8(b"port-0");
        let channel_id = string::utf8(b"channel-0");
        Core::set_channel(port_id, channel_id, channel);

        // Prepare proof and height
        let proof_height = height::new(1, 1);
        let proof_ack = any::pack(vector::empty<u8>());

        // Call channel_open_confirm function
        Core::channel_open_confirm(port_id, channel_id, proof_ack, proof_height);

        // Validate the channel state after confirmation
        let updated_channel = Core::get_channel_from_store(port_id, channel_id);
        assert!(channel::state(&updated_channel) == 3, 1001); // STATE_OPEN
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1016)] // E_INVALID_CHANNEL_STATE
    public fun test_channel_open_confirm_invalid_state(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(Core::default_ibc_version()),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel with an invalid state
        let connection_hops = vector::singleton(connection_id);
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(1, 1, counterparty_channel, connection_hops, string::utf8(b"1")); // STATE_INIT (invalid state for confirm)
        let port_id = string::utf8(b"port-0");
        let channel_id = string::utf8(b"channel-0");
        Core::set_channel(port_id, channel_id, channel);

        // Prepare proof and height
        let proof_height = height::new(1, 1);
        let proof_ack = any::pack(vector::empty<u8>());

        // Call channel_open_confirm function
        Core::channel_open_confirm(port_id, channel_id, proof_ack, proof_height);
    }

    #[test(alice = @IBC)]
    public fun test_channel_open_try_success(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(Core::default_ibc_version()),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel
        let connection_hops = vector::singleton(connection_id);
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(2, ORDER_ORDERED, counterparty_channel, connection_hops, string::utf8(b"1"));

        // Mock data for proof
        let proof_height = height::new(1, 1);
        let proof_init = any::pack(vector::empty<u8>());

        // Call channel_open_try function
        let channel_id = Core::channel_open_try(string::utf8(b"port-0"), channel, string::utf8(b"1"), proof_init, proof_height);

        // Validate that the channel was added to the store
        let stored_channel = Core::get_channel_from_store(string::utf8(b"port-0"), channel_id);

        // Validate that the stored channel matches the expected channel
        assert!(channel::state(&stored_channel) == 2, 8001);
        assert!(channel::ordering(&stored_channel) == ORDER_ORDERED, 8002);
        assert!(*channel::version(&stored_channel) == string::utf8(b"1"), 8003);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1016)] // E_INVALID_CHANNEL_STATE
    public fun test_channel_open_try_invalid_state(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock connection and set it in the IBCStore
        let client_id = string::utf8(b"client-0");
        let connection_id = string::utf8(b"connection-0");
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            connection_id,
            b"",
        );
        let connection = connection_end::new(
            client_id,
            vector::singleton(Core::default_ibc_version()),
            3, // STATE_OPEN
            0,
            counterparty
        );
        Core::set_connection(connection_id, connection);

        // Prepare a mock channel with an invalid state (not STATE_TRYOPEN)
        let connection_hops = vector::singleton(connection_id);
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
        let channel = channel::new(1, ORDER_ORDERED, counterparty_channel, connection_hops, string::utf8(b"1")); // Invalid state

        // Mock data for proof
        let proof_height = height::new(1, 1);
        let proof_init = any::pack(vector::empty<u8>());

        // Call channel_open_try function, should abort due to invalid state
        Core::channel_open_try(string::utf8(b"port-0"), channel, string::utf8(b"1"), proof_init, proof_height);
    }

    #[test(alice = @IBC)]
    public fun test_claim_capability_success(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Claim a new capability
        let capability_name = string::utf8(b"capability-0");
        let addr = signer::address_of(alice);
        Core::claim_capability(capability_name, addr);

        // Verify the capability was claimed
        let claimed_addr = Core::get_capability_from_store(capability_name);
        assert!(claimed_addr == addr, 9001);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1014)] // E_CAPABILITY_ALREADY_CLAIMED
    public fun test_claim_capability_already_claimed(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Claim a new capability
        let capability_name = string::utf8(b"capability-0");
        let addr = signer::address_of(alice);
        Core::claim_capability(capability_name, addr);

        // Attempt to claim the same capability again, should abort
        Core::claim_capability(capability_name, addr);
    }

}
