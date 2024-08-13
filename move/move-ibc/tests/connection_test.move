#[test_only]
module IBC::ConnectionTest {
    use aptos_std::string::{Self, String};
    use aptos_framework::vector;
    use IBC::Core;
    use IBC::height;
    use aptos_std::any;
    use IBC::connection_end::{Version, Self};

    #[test]
    public fun test_default_ibc_version() {
        let version = Core::default_ibc_version();
        let expected_identifier = string::utf8(b"1");
        let expected_features = vector::empty<String>();
        vector::push_back(&mut expected_features, string::utf8(b"ORDER_ORDERED"));
        vector::push_back(&mut expected_features, string::utf8(b"ORDER_UNORDERED"));

        assert!(*connection_end::version_identifier(&version) == expected_identifier, 1001);
        let len = vector::length(connection_end::version_features(&version));
        assert!(len == 2, 1002);
        assert!(vector::contains(connection_end::version_features(&version), &string::utf8(b"ORDER_ORDERED")), 1003);
        assert!(vector::contains(connection_end::version_features(&version), &string::utf8(b"ORDER_UNORDERED")), 1004);
    }

    #[test]
    public fun test_set_supported_versions() {
        let supported_versions = vector::empty<Version>();
        let dst = vector::empty<Version>();
        vector::push_back(&mut supported_versions, Core::default_ibc_version());
        Core::set_supported_versions(supported_versions, &mut dst);

        let len = vector::length(&dst);
        assert!(len == 1, 2001);
    }

    #[test]
    public fun test_is_supported_version() {
        let supported_versions = vector::empty<Version>();
        vector::push_back(&mut supported_versions, Core::default_ibc_version());

        let version = Core::default_ibc_version();
        let is_supported = Core::is_supported_version(&supported_versions, &version);
        assert!(is_supported, 3001);

        let non_matching_version = connection_end::new_version(string::utf8(b"2"), vector::empty<String>());
        let is_not_supported = Core::is_supported_version(&supported_versions, &non_matching_version);
        assert!(!is_not_supported, 3002);
    }

    #[test]
    public fun test_find_supported_version() {
        let supported_versions = vector::empty<Version>();
        let version = Core::default_ibc_version();

        vector::push_back(&mut supported_versions, version);

        
        let (found_version, found) = Core::find_supported_version(&supported_versions, &version);
        
        assert!(found, 4001);
        assert!(connection_end::version_identifier(&found_version) == connection_end::version_identifier(&version), 4002);

        let non_matching_version = connection_end::new_version(string::utf8(b"2"), vector::empty<String>());
        let (_, not_found) = Core::find_supported_version(&supported_versions, &non_matching_version);
        assert!(!not_found, 4003);
    }

    #[test]
    public fun test_verify_proposed_version() {
        let supported_version = Core::default_ibc_version();
        let matching_proposed_version = Core::default_ibc_version();
        let non_matching_proposed_version = connection_end::new_version(
            string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_CUSTOM"))
        );

        let is_verified = Core::verify_proposed_version(&supported_version, &matching_proposed_version);
        assert!(is_verified, 5001);

        let is_not_verified = Core::verify_proposed_version(&supported_version, &non_matching_proposed_version);
        assert!(!is_not_verified, 5002);
    }

    #[test]
    public fun test_pick_version_success() {
        let supported_versions = vector::empty<Version>();
        let counterparty_versions = vector::empty<Version>();

        let version1 = Core::default_ibc_version();
        let version2 = connection_end::new_version(string::utf8(b"2"), vector::singleton(string::utf8(b"ORDER_ORDERED")));

        vector::push_back(&mut supported_versions, version1);
        vector::push_back(&mut counterparty_versions, version1);
        vector::push_back(&mut supported_versions, version2);

        let picked_version = Core::pick_version(&supported_versions, &counterparty_versions);

        assert!(*connection_end::version_identifier(&picked_version) == string::utf8(b"1"), 6001);
        assert!(vector::length(connection_end::version_features(&picked_version)) > 0, 6002);
    }

    #[test]
    #[expected_failure(abort_code = 1007)]
    public fun test_pick_version_failure() {
        let supported_versions = vector::empty<Version>();
        let counterparty_versions = vector::empty<Version>();

        let version1 = connection_end::new_version(string::utf8(b"2"), vector::singleton(string::utf8(b"ORDER_ORDERED")));
        let version2 = connection_end::new_version(string::utf8(b"3"), vector::singleton(string::utf8(b"ORDER_UNORDERED")));

        vector::push_back(&mut supported_versions, version1);
        vector::push_back(&mut counterparty_versions, version2);

        Core::pick_version(&supported_versions, &counterparty_versions);
    }

    #[test]
    public fun test_copy_versions_success() {
        let src_versions = vector::empty<Version>();
        let dst_versions = vector::empty<Version>();

        let version1 = Core::default_ibc_version();
        let version2 = connection_end::new_version(string::utf8(b"2"), vector::singleton(string::utf8(b"ORDER_ORDERED")));

        vector::push_back(&mut src_versions, version1);
        vector::push_back(&mut src_versions, version2);

        Core::copy_versions(&src_versions, &mut dst_versions);

        assert!(vector::length(&dst_versions) == vector::length(&src_versions), 7001);
        assert!(*connection_end::version_identifier(vector::borrow(&dst_versions, 0)) == string::utf8(b"1"), 7002);
        assert!(*connection_end::version_identifier(vector::borrow(&dst_versions, 1)) == string::utf8(b"2"), 7003);
    }

    #[test]
    public fun test_copy_versions_mismatch_length() {
        let src_versions = vector::empty<Version>();
        let dst_versions = vector::empty<Version>();

        let version1 = Core::default_ibc_version();
        let version2 = connection_end::new_version(string::utf8(b"2"), vector::singleton(string::utf8(b"ORDER_ORDERED")));

        vector::push_back(&mut src_versions, version1);
        vector::push_back(&mut src_versions, version2);
        vector::push_back(&mut dst_versions, version1);

        Core::copy_versions(&src_versions, &mut dst_versions);

        assert!(vector::length(&dst_versions) == vector::length(&src_versions), 7004);
        assert!(*connection_end::version_identifier(vector::borrow(&dst_versions, 0)) == string::utf8(b"1"), 7005);
        assert!(*connection_end::version_identifier(vector::borrow(&dst_versions, 1)) == string::utf8(b"2"), 7006);
    }

    #[test]
    public fun test_verify_client_state_success() {
        let client_id = string::utf8(b"client-0");
        let height = height::new(0, 0);
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            b"ibc",
        );
        let connection = connection_end::new(
            client_id,
            vector::empty<Version>(),
            0,
            0,
            counterparty
        );
        let path = vector::empty<u8>();
        let proof = any::pack(vector::empty<u8>());
        let client_state_bytes = vector::empty<u8>();

        let result = Core::verify_client_state(
            &connection,
            height,
            path,
            proof,
            client_state_bytes
        );
        assert!(result, 8001);
    }

    #[test]
    public fun test_verify_connection_state_success() {
        let client_id = string::utf8(b"client-0");
        let height = height::new(0, 0);
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            b"ibc",
        );
        let connection = connection_end::new(
            client_id,
            vector::empty<Version>(),
            0,
            0,
            counterparty
        );
        let counterparty_connection = connection_end::new(
            string::utf8(b"counterparty-client"),
            vector::empty<Version>(),
            1,
            0,
            connection_end::new_counterparty(
                string::utf8(b"client-0"),
                string::utf8(b"connection-0"),
                b"ibc",
            )
        );
        let proof = any::pack(vector::empty<u8>());
        let connection_id = string::utf8(b"connection-0");

        let result = Core::verify_connection_state(
            &connection,
            height,
            proof,
            connection_id,
            counterparty_connection
        );
        assert!(result, 9001);
    }

    #[test(alice = @IBC)]
    public fun test_generate_connection_identifier(alice: &signer) {
        Core::create_ibc_store(alice);
        let expected_identifier1 = string::utf8(b"connection-0");
        let expected_identifier2 = string::utf8(b"connection-1");

        let identifier1 = Core::generate_connection_identifier();
        assert!(identifier1 == expected_identifier1, 10001);

        let identifier2 = Core::generate_connection_identifier();
        assert!(identifier2 == expected_identifier2, 10002);
    }

    #[test(alice = @IBC)]
    public fun test_connection_open_init_success(alice: &signer) {
        Core::create_ibc_store(alice);

        let client_id = string::utf8(b"client-0");
        let version = Core::default_ibc_version();
        let delay_period: u64 = 0;
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            b"ibc",
        );

        let connection_id_1 = Core::connection_open_init(
            client_id,
            version,
            counterparty,
            delay_period
        );

        let connection = Core::get_connection(connection_id_1);
        let commitment = Core::get_connection_commitment(connection_id_1);

        assert!(*connection_end::client_id(&connection) == client_id, 1001);
        assert!(vector::length(connection_end::versions(&connection)) > 0, 1002);
        assert!(connection_end::state(&connection) == 1, 1003);
        assert!(connection_end::delay_period(&connection) == delay_period, 1004);
        assert!(vector::length(&commitment) > 0, 1005);

        // Create a second connection
        let client_id_2 = string::utf8(b"client-1");
        let version_2 = Core::default_ibc_version();
        let delay_period_2: u64 = 0;
        let counterparty_2 = connection_end::new_counterparty(
            string::utf8(b"counterparty-client-1"),
            string::utf8(b"connection-1"),
            b"",
        );

        let connection_id_2 = Core::connection_open_init(
            client_id_2,
            version_2,
            counterparty_2,
            delay_period_2
        );

        let connection_2 = Core::get_connection(connection_id_2);
        let commitment_2 = Core::get_connection_commitment(connection_id_2);

        assert!(*connection_end::client_id(&connection_2) == client_id_2, 2001);
        assert!(vector::length(connection_end::versions(&connection_2)) > 0, 2002);
        assert!(connection_end::state(&connection_2) == 1, 2003);
        assert!(connection_end::delay_period(&connection_2) == delay_period_2, 2004);
        assert!(vector::length(&commitment_2) > 0, 2005);

        // Ensure that the connection IDs are unique
        assert!(connection_id_1 != connection_id_2, 2006);
    }

    #[test(alice = @IBC)]
    public fun test_connection_open_try_success(alice: &signer) {
        // Initialize the IBC store
        Core::create_ibc_store(alice);

        let client_id = string::utf8(b"client-0");
        let version = Core::default_ibc_version();
        let delay_period: u64 = 0;
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            b"",
        );

        // Mock data for proof
        let proof_height = height::new(1, 1);
        let proof_init = any::pack(vector::empty<u8>());
        let proof_client = any::pack(vector::empty<u8>());
        let proof_consensus = vector::empty<u8>();
        let client_state_bytes = vector::empty<u8>();
        let counterparty_versions = vector::singleton(version);

        // Call connection_open_try
        let connection_id = Core::connection_open_try(
            counterparty,
            delay_period,
            client_id,
            client_state_bytes,
            counterparty_versions,
            proof_init,
            proof_client,
            proof_consensus,
            proof_height,
            proof_height
        );

        // Fetch the connection and commitment from the store
        let connection = Core::get_connection(connection_id);
        let commitment = Core::get_connection_commitment(connection_id);

        // Assertions
        assert!(*connection_end::client_id(&connection) == client_id, 1001);
        assert!(vector::length(connection_end::versions(&connection)) > 0, 1002);
        assert!(connection_end::state(&connection) == 2, 1003); // STATE_TRYOPEN
        assert!(connection_end::delay_period(&connection) == delay_period, 1004);
        assert!(vector::length(&commitment) > 0, 1005);

        // Verify that the counterparty fields are set correctly
        assert!(*connection_end::conn_counterparty_client_id(&connection) == string::utf8(b"counterparty-client"), 1006);
        assert!(*connection_end::conn_counterparty_connection_id(&connection) == string::utf8(b"connection-0"), 1007);

        // Add second connection to check if the connection state is increasing
        let new_client_id = string::utf8(b"client-1");
        let new_counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client-1"),
            string::utf8(b"connection-1"),
            b"",
        );


        let proof_init2 = any::pack(vector::empty<u8>());
        let proof_client2 = any::pack(vector::empty<u8>());

        let new_counterparty_versions = vector::singleton(version);
        let new_connection_id = Core::connection_open_try(
            new_counterparty,
            delay_period,
            new_client_id,
            client_state_bytes,
            new_counterparty_versions,
            proof_init2,
            proof_client2,
            proof_consensus,
            proof_height,
            proof_height
        );

        let new_connection = Core::get_connection(new_connection_id);

        // Assertions for the second connection
        assert!(*connection_end::client_id(&new_connection) == new_client_id, 1008);
        assert!(vector::length(connection_end::versions(&new_connection)) > 0, 1009);
        assert!(connection_end::state(&new_connection) == 2, 1010); // STATE_TRYOPEN
        assert!(connection_end::delay_period(&new_connection) == delay_period, 1011);
    }


    #[test(alice = @IBC)]   
    public fun test_connection_open_ack_success(alice: &signer) {
        // Initialize the IBC store
        Core::create_ibc_store(alice);

        let client_id = string::utf8(b"client-0");
        let version = Core::default_ibc_version();
        let delay_period: u64 = 0;
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            b"",
        );

        // Call connection_open_init
        let connection_id = Core::connection_open_init(
            client_id,
            version,
            counterparty,
            delay_period
        );

        // Mock data for proof
        let proof_height = height::new(1, 1);
        let proof_try = any::pack(vector::empty<u8>());
        let proof_client = any::pack(vector::empty<u8>());
        let proof_consensus = vector::empty<u8>();
        let client_state_bytes = vector::empty<u8>();
        let counterparty_connection_id = string::utf8(b"connection-0");

        // Call connection_open_ack
        Core::connection_open_ack(
            connection_id,
            client_state_bytes,
            version,
            proof_try,
            proof_client,
            proof_consensus,
            counterparty_connection_id,
            proof_height,
            proof_height
        );

        // Fetch the connection and commitment from the store
        let connection = Core::get_connection(connection_id);
        let commitment = Core::get_connection_commitment(connection_id);

        // Assertions
        assert!(*connection_end::client_id(&connection) == client_id, 1001);
        assert!(vector::length(connection_end::versions(&connection)) > 0, 1002);
        assert!(connection_end::state(&connection) == 3, 1003); // STATE_OPEN
        assert!(connection_end::delay_period(&connection) == delay_period, 1004);
        assert!(vector::length(&commitment) > 0, 1005);

        // Verify that the counterparty fields are set correctly
        assert!(*connection_end::conn_counterparty_client_id(&connection) == string::utf8(b"counterparty-client"), 1006);
        assert!(*connection_end::conn_counterparty_connection_id(&connection) == string::utf8(b"connection-0"), 1007);

        // Add second connection to check if the connection state is increasing
        let new_client_id = string::utf8(b"client-1");
        let new_counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client-1"),
            string::utf8(b"connection-1"),
            b"",
        );
        let new_connection_id = Core::connection_open_init(
            new_client_id,
            version,
            new_counterparty,
            delay_period
        );

        // Mock data for proof for the new connection
        let new_proof_height = height::new(2, 1);
        let new_proof_try = any::pack(vector::empty<u8>());
        let new_proof_client = any::pack(vector::empty<u8>());
        let new_proof_consensus = vector::empty<u8>();
        let new_client_state_bytes = vector::empty<u8>();
        let new_counterparty_connection_id = string::utf8(b"connection-1");

        // Call connection_open_ack for the new connection
        Core::connection_open_ack(
            new_connection_id,
            new_client_state_bytes,
            version,
            new_proof_try,
            new_proof_client,
            new_proof_consensus,
            new_counterparty_connection_id,
            new_proof_height,
            new_proof_height
        );

        // Fetch the new connection and commitment from the store
        let new_connection = Core::get_connection(new_connection_id);
        let new_commitment = Core::get_connection_commitment(new_connection_id);

        // Assertions for the new connection
        assert!(*connection_end::client_id(&new_connection) == new_client_id, 2001);
        assert!(vector::length(connection_end::versions(&new_connection)) > 0, 2002);
        assert!(connection_end::state(&new_connection) == 3, 2003); // STATE_OPEN
        assert!(connection_end::delay_period(&new_connection) == delay_period, 2004);
        assert!(vector::length(&new_commitment) > 0, 2005);
        assert!(*connection_end::conn_counterparty_client_id(&new_connection) == string::utf8(b"counterparty-client-1"), 2006);
        assert!(*connection_end::conn_counterparty_connection_id(&new_connection) == string::utf8(b"connection-1"), 2007);
    }
   

    #[test(alice = @IBC)]   
    public fun test_connection_open_confirm_success(alice: &signer) {
        Core::create_ibc_store(alice);

        let client_id = string::utf8(b"client-0");
        let version = Core::default_ibc_version();
        let delay_period: u64 = 0;
        let counterparty_prefix = vector::empty<u8>();
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            counterparty_prefix,
        );

        // Mock data for proof
        let proof_height = height::new(1, 1);
        let proof_init = any::pack(vector::empty<u8>());
        let proof_client = any::pack(vector::empty<u8>());
        let client_state_bytes = vector::empty<u8>();

        // Call connection_open_try
        let connection_id = Core::connection_open_try(
            counterparty,
            delay_period,
            client_id,
            client_state_bytes,
            vector::singleton(version),
            proof_init,
            proof_client,
            vector::empty<u8>(),  // proof_consensus (not used in this test)
            proof_height,
            proof_height
        );

        // Call connection_open_confirm
        Core::connection_open_confirm(
            connection_id,
            any::pack(vector::empty<u8>()),  // proofAck
            proof_height
        );

        // Fetch the connection and commitment from the store
        let updated_connection = Core::get_connection(connection_id);
        let commitment = Core::get_connection_commitment(connection_id);

        // Assertions
        assert!(*connection_end::client_id(&updated_connection) == client_id, 1001);
        assert!(vector::length(connection_end::versions(&updated_connection)) > 0, 1002);
        assert!(connection_end::state(&updated_connection) == 3, 1003); // STATE_OPEN
        assert!(connection_end::delay_period(&updated_connection) == delay_period, 1004);
        assert!(vector::length(&commitment) > 0, 1005);

        // Verify that the counterparty fields are set correctly
        assert!(*connection_end::conn_counterparty_client_id(&updated_connection) == string::utf8(b"counterparty-client"), 1006);
        assert!(*connection_end::conn_counterparty_connection_id(&updated_connection) == string::utf8(b"connection-0"), 1007);
    }

    #[test(alice = @IBC)]   
    #[expected_failure(abort_code = 1008)] // E_INVALID_CONNECTION_STATE
    public fun test_connection_open_confirm_failure_invalid_state(alice: &signer) {
        Core::create_ibc_store(alice);

        let client_id = string::utf8(b"client-0");
        let version = Core::default_ibc_version();
        let delay_period: u64 = 0;
        let counterparty_prefix = vector::empty<u8>();
        let counterparty = connection_end::new_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            counterparty_prefix
        );

        // Mock data for proof
        let proof_height = height::new(1, 1);
        let proof_ack = any::pack(vector::empty<u8>());
        let connection_id = Core::connection_open_init(
            client_id,
            version,
            counterparty,
            delay_period
        );

        // Call connection_open_confirm without calling connection_open_try
        Core::connection_open_confirm(
            connection_id,
            proof_ack,
            proof_height
        );
    }

    // TODO: Uncomment this test when light client is implemented

    // #[test(alice = @IBC, relayer=@mock_relayer_address)]   
    // #[expected_failure(abort_code = 1010)] // E_INVALID_PROOF
    // public fun test_connection_open_confirm_failure_invalid_proof(alice: &signer, relayer:address) {
    //     Core::create_ibc_store(alice);
    //     let client_id = string::utf8(b"client-0");
    //     let version = Core::default_ibc_version();
    //     let delay_period: u64 = 0;
    //     let counterparty_prefix = Core::new_merkleprefix(vector::empty<u8>());
    //     let counterparty = Core::new_connection_counterparty(
    //         string::utf8(b"counterparty-client"),
    //         string::utf8(b"connection-0"),
    //         counterparty_prefix
    //     );

    //     // Mock data for proof
    //     let proof_height = height::new(1, 1);
    //     let proof_init = any::pack(vector::empty<u8>());
    //     let proof_client = any::pack(vector::empty<u8>());
    //     let client_state_bytes = vector::empty<u8>();


    //     // Call connection_open_try
    //     let connection_id_try = Core::connection_open_try(
    //         counterparty,
    //         delay_period,
    //         client_id,
    //         client_state_bytes,
    //         vector::singleton(version),
    //         proof_init,
    //         proof_client,
    //         vector::empty<u8>(),  // proof_consensus (not used in this test)
    //         proof_height,
    //         proof_height,
    //         relayer
    //     );

    //     // Call connection_open_confirm with invalid proof
    //     let invalid_proof = b"invalid_proof";
    //     Core::connection_open_confirm(
    //         connection_id_try,
    //         invalid_proof,
    //         proof_height,
    //         relayer
    //     );

    //     // This won't fail because light client is not implemented
    //     // and it returns true in any case so we can't check the proof
    // }

}
