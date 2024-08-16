#[test_only]
module IBC::ChannelTest {

    use std::signer;
    use std::vector;
    use aptos_std::string::{Self, String};
    use aptos_std::any;
    use IBC::height;
    use std::hash;
    use IBC::Core;
    use std::bcs;
    use IBC::connection_end::{Self, Version};
    use IBC::IBCCommitment;
    use IBC::channel;
    use IBC::packet;

    const E_GENERATE_CLIENT_IDENTIFIER: u64 = 3001;
    const E_GET_CLIENT_IMPL: u64 = 3002;
    const E_CREATE_CLIENT: u64 = 3003;

    const E_ACKNOWLEDGEMENT_IS_EMPTY: u64 = 1028;
    const E_ACKNOWLEDGEMENT_ALREADY_EXISTS: u64 = 1029;

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
        let (_,_,_,channel_id,_,_,_) = Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));

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
    let (_,_,_,channel_id,_,_,_) = Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));

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
        let (_,_,_,channel_id,_,_,_) = Core::channel_open_init(string::utf8(b"port-0"), channel, signer::address_of(alice));

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
        let (_,_,_,channel_id,_,_,_,_) = Core::channel_open_try(string::utf8(b"port-0"), channel, string::utf8(b"1"), proof_init, proof_height);

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

    #[test(alice = @IBC)]
    public fun test_write_acknowledgement_success(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock channel and set it in the IBCStore
        let connection_id = string::utf8(b"connection-0");
        let channel_id = string::utf8(b"channel-0");
        let port_id = string::utf8(b"port-0");
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), channel_id);
        let channel_data = channel::new(3, 2, counterparty, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Set the capability to allow writing the acknowledgment
        let relayer_addr = signer::address_of(alice);
        Core::claim_capability(IBCCommitment::channel_capability_path(port_id, channel_id), relayer_addr);

        // Create a mock packet
        let packet_data = packet::new(
            1,
            string::utf8(b""),
            string::utf8(b""),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        // Create an acknowledgment
        let acknowledgement = vector::singleton(1u8);

        // Call write_acknowledgement function
        Core::write_acknowledgement(alice, packet_data, acknowledgement);

        // Verify that the acknowledgment was written
        let ack_commitment_key = IBCCommitment::packet_acknowledgement_commitment_key(port_id, channel_id, 1);
        let stored_ack = Core::get_commitment(ack_commitment_key);
        assert!(stored_ack == hash::sha2_256(acknowledgement), 1101);
    }
    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1020)] // E_UNAUTHORIZED
    public fun test_write_acknowledgement_unauthorized(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock channel and set it in the IBCStore
        let connection_id = string::utf8(b"connection-0");
        let channel_id = string::utf8(b"channel-0");
        let port_id = string::utf8(b"port-0");
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), channel_id);
        let channel_data = channel::new(3, 2, counterparty, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Create a mock packet
        let packet_data = packet::new(
            1,
            string::utf8(b""),
            string::utf8(b""),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        // Create an acknowledgment
        let acknowledgement = vector::singleton(1u8);

        // Call write_acknowledgement function without setting capability
        Core::write_acknowledgement(alice, packet_data, acknowledgement);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1028)] // E_ACKNOWLEDGEMENT_IS_EMPTY
    public fun test_write_acknowledgement_empty_ack(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock channel and set it in the IBCStore
        let connection_id = string::utf8(b"connection-0");
        let channel_id = string::utf8(b"channel-0");
        let port_id = string::utf8(b"port-0");
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), channel_id);
        let channel_data = channel::new(3, 2, counterparty, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Set the capability to allow writing the acknowledgment
        let relayer_addr = signer::address_of(alice);
        Core::claim_capability(IBCCommitment::channel_capability_path(port_id, channel_id), relayer_addr);

        // Create a mock packet
        let packet_data = packet::new(
            1,
            string::utf8(b""),
            string::utf8(b""),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        // Create an empty acknowledgment
        let empty_acknowledgement = vector::empty<u8>();

        // Call write_acknowledgement function with empty acknowledgment
        Core::write_acknowledgement(alice, packet_data, empty_acknowledgement);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1029)] // E_ACKNOWLEDGEMENT_ALREADY_EXISTS
    public fun test_write_acknowledgement_already_exists(alice: &signer) {
        // Initialize IBCStore for testing
        Core::create_ibc_store(alice);

        // Prepare a mock channel and set it in the IBCStore
        let connection_id = string::utf8(b"connection-0");
        let channel_id = string::utf8(b"channel-0");
        let port_id = string::utf8(b"port-0");
        let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), channel_id);
        let channel_data = channel::new(3, 2, counterparty, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Set the capability to allow writing the acknowledgment
        let relayer_addr = signer::address_of(alice);
        Core::claim_capability(IBCCommitment::channel_capability_path(port_id, channel_id), relayer_addr);

        // Create a mock packet
        let packet_data = packet::new(
            1,
            string::utf8(b""),
            string::utf8(b""),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        // Create an acknowledgment
        let acknowledgement = vector::singleton(1u8);

        // Call write_acknowledgement function once
        Core::write_acknowledgement(alice, packet_data, acknowledgement);

        // Call write_acknowledgement function again to trigger the already exists error
        Core::write_acknowledgement(alice, packet_data, acknowledgement);
    }

    #[test(alice = @IBC)]
    public fun test_acknowledge_packet_success(alice: &signer) {
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
        let port_id = string::utf8(b"counterparty-port");
        let channel_id = string::utf8(b"counterparty-channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b"counterparty-channel-0"));
        let channel_data = channel::new(3, 2, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Verify that the channel is set correctly in the store
        let stored_channel = Core::get_channel_from_store(port_id, channel_id);
        assert!(channel::state(&stored_channel) == 3, 1102); // Ensure the channel is in STATE_OPEN

        // Set a packet commitment for the mock packet
        let packet_sequence = 0;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"counterparty-port"),
            string::utf8(b"counterparty-channel-0"),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        let packet_commitment_key = IBCCommitment::packet_commitment_key(port_id, channel_id, packet_sequence);
        let packet_commitment_value = hash::sha2_256(packet::commitment(&packet_data));
        Core::set_commitment(packet_commitment_key, packet_commitment_value);

        // Prepare mock proof data
        let proof_height = height::new(0, 1);
        let proof = any::pack(vector::empty<u8>());
        let acknowledgement = vector::singleton(1u8);

        // Call acknowledge_packet function
        Core::acknowledge_packet(packet_data, acknowledgement, proof, proof_height);

        // Validate that the packet commitment has been removed
        let retrieved_commitment = Core::get_commitment(packet_commitment_key);
        assert!(vector::length(&retrieved_commitment) == 0, 1101);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1032)] // E_PACKET_COMMITMENT_NOT_FOUND
    public fun test_acknowledge_packet_commitment_not_found(alice: &signer) {
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
        let port_id = string::utf8(b"counterparty-port");
        let channel_id = string::utf8(b"counterparty-channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b"counterparty-channel-0"));
        let channel_data = channel::new(3, 2, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Prepare a packet without setting a corresponding commitment
        let packet_sequence = 0;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"counterparty-port"),
            string::utf8(b"counterparty-channel-0"),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        // Prepare mock proof data
        let proof_height = height::new(0, 1);
        let proof = any::pack(vector::empty<u8>());
        let acknowledgement = vector::singleton(1u8);

        // Call acknowledge_packet function (should abort with E_PACKET_COMMITMENT_NOT_FOUND)
        Core::acknowledge_packet(packet_data, acknowledgement, proof, proof_height);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1033)] // E_INVALID_PACKET_COMMITMENT
    public fun test_acknowledge_packet_invalid_commitment(alice: &signer) {
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
        let port_id = string::utf8(b"counterparty-port");
        let channel_id = string::utf8(b"counterparty-channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b"counterparty-channel-0"));
        let channel_data = channel::new(3, 2, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN
        Core::set_channel(port_id, channel_id, channel_data);

        // Set an incorrect packet commitment for the mock packet
        let packet_sequence = 0;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"counterparty-port"),
            string::utf8(b"counterparty-channel-0"),
            port_id,
            channel_id,
            vector::singleton(99u8), // Incorrect data to cause commitment mismatch
            height::new(0, 1),
            1000000000);

        let packet_commitment_key = IBCCommitment::packet_commitment_key(port_id, channel_id, packet_sequence);
        let incorrect_commitment_value = hash::sha2_256(vector::singleton(88u8)); // Incorrect commitment value
        Core::set_commitment(packet_commitment_key, incorrect_commitment_value);

        // Prepare mock proof data
        let proof_height = height::new(0, 1);
        let proof = any::pack(vector::empty<u8>());
        let acknowledgement = vector::singleton(1u8);

        // Call acknowledge_packet function (should abort with E_INVALID_PACKET_COMMITMENT)
        Core::acknowledge_packet(packet_data, acknowledgement, proof, proof_height);
    }

    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1026)] // E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH
    public fun test_acknowledge_packet_sequence_mismatch(alice: &signer) {
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
        let port_id = string::utf8(b"counterparty-port");
        let channel_id = string::utf8(b"counterparty-channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b"counterparty-channel-0"));
        let channel_data = channel::new(3, 2, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN, ORDERED
        Core::set_channel(port_id, channel_id, channel_data);

        // Set a correct packet commitment for the mock packet but with a different sequence
        let packet_sequence = 1;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"counterparty-port"),
            string::utf8(b"counterparty-channel-0"),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            1000000000);

        let packet_commitment_key = IBCCommitment::packet_commitment_key(port_id, channel_id, packet_sequence);
        let packet_commitment_value = hash::sha2_256(packet::commitment(&packet_data));
        Core::set_commitment(packet_commitment_key, packet_commitment_value);

        // Set an expected sequence mismatch
        let next_sequence_ack_key = IBCCommitment::next_sequence_ack_commitment_key(port_id, channel_id);
        Core::set_commitment(next_sequence_ack_key, bcs::to_bytes(&2u64)); // Set to 2 instead of 1

        // Prepare mock proof data
        let proof_height = height::new(0, 1);
        let proof = any::pack(vector::empty<u8>());
        let acknowledgement = vector::singleton(1u8);

        // Call acknowledge_packet function (should abort with E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH)
        Core::acknowledge_packet(packet_data, acknowledgement, proof, proof_height);
    }

    #[test(alice = @IBC)]
    public fun test_timeout_packet_success_ordered(alice: &signer) {
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
        let port_id = string::utf8(b"port-0");
        let channel_id = string::utf8(b"channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"port-0"), string::utf8(b"channel-0"));
        let channel_data = channel::new(3, 2, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN, ORDERED
        Core::set_channel(port_id, channel_id, channel_data);

        // Set a packet commitment for the mock packet
        let packet_sequence = 0;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"port-0"),
            string::utf8(b"channel-0"),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            0
        );

        let packet_commitment_key = IBCCommitment::packet_commitment_key(port_id, channel_id, packet_sequence);
        let packet_commitment_value = hash::sha2_256(packet::commitment(&packet_data));
        Core::set_commitment(packet_commitment_key, packet_commitment_value);

        // Prepare mock proof data
        let proof_height = height::new(0, 2);
        let proof = any::pack(vector::empty<u8>());

        // Call timeout_packet function
        Core::timeout_packet(
            port_id,
            channel_id,
            packet_data,
            proof,
            proof_height,
            2 // next_sequence_recv
        );

        // Validate that the packet commitment has been removed
        let retrieved_commitment = Core::get_commitment(packet_commitment_key);
        assert!(vector::length(&retrieved_commitment) == 0, 1101);
    }
    #[test(alice = @IBC)]
    public fun test_timeout_packet_success_unordered(alice: &signer) {
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
        let port_id = string::utf8(b"port-0");
        let channel_id = string::utf8(b"channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"port-0"), string::utf8(b"channel-0"));
        let channel_data = channel::new(3, 1, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN, UNORDERED
        Core::set_channel(port_id, channel_id, channel_data);

        // Set a packet commitment for the mock packet
        let packet_sequence = 0;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"port-0"),
            string::utf8(b"channel-0"),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            0
        );

        let packet_commitment_key = IBCCommitment::packet_commitment_key(port_id, channel_id, packet_sequence);
        let packet_commitment_value = hash::sha2_256(packet::commitment(&packet_data));
        Core::set_commitment(packet_commitment_key, packet_commitment_value);

        // Prepare mock proof data
        let proof_height = height::new(0, 2);
        let proof = any::pack(vector::empty<u8>());

        // Call timeout_packet function
        Core::timeout_packet(
            port_id,
            channel_id,
            packet_data,
            proof,
            proof_height,
            0 // next_sequence_recv is not relevant for UNORDERED
        );

        // Validate that the packet commitment has been removed
        let retrieved_commitment = Core::get_commitment(packet_commitment_key);
        assert!(vector::length(&retrieved_commitment) == 0, 1102);
    }
    
    #[test(alice = @IBC)]
    #[expected_failure(abort_code = 1032)] // E_PACKET_COMMITMENT_NOT_FOUND
    public fun test_timeout_packet_commitment_not_found(alice: &signer) {
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
        let port_id = string::utf8(b"port-0");
        let channel_id = string::utf8(b"channel-0");
        let counterparty_channel = channel::new_counterparty(string::utf8(b"port-0"), string::utf8(b"channel-0"));
        let channel_data = channel::new(3, 2, counterparty_channel, vector::singleton(connection_id), string::utf8(b"1")); // STATE_OPEN, ORDERED
        Core::set_channel(port_id, channel_id, channel_data);

        // Prepare a packet without setting a corresponding commitment
        let packet_sequence = 0;
        let packet_data = packet::new(
            packet_sequence,
            string::utf8(b"port-0"),
            string::utf8(b"channel-0"),
            port_id,
            channel_id,
            vector::empty<u8>(),
            height::new(0, 1),
            0
        );

        // Prepare mock proof data
        let proof_height = height::new(0, 2);
        let proof = any::pack(vector::empty<u8>());

        // Call timeout_packet function, should abort due to missing commitment
        Core::timeout_packet(
            port_id,
            channel_id,
            packet_data,
            proof,
            proof_height,
            2 // next_sequence_recv
        );
    }

}
