module 0x1::ClientTest {

    use std::signer;
    use std::account;
    use std::vector;
    use std::debug;
    use aptos_std::string::{Self as StringModule, String};
    use aptos_framework::coin::Coin;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::event;
    use ContractClientAddress::Client::{Self as ClientModule, MsgCreateClient};


    use aptos_framework::smart_table::{Self as SmartTable, SmartTable};
    const E_GENERATE_CLIENT_IDENTIFIER: u64 = 3001;
    const E_GET_CLIENT_IMPL: u64 = 3002;
    const E_CREATE_CLIENT: u64 = 3003;

    // Test for the register_client function
    #[test(alice = @0x444444)]
    public fun test_register_client(alice: &signer) {
        // // Initialize the IBCStore in Alice's account
        ClientModule::create_ibc_store(alice);

        // // Define a client type and address for testing
        let client_type = StringModule::utf8(b"test_client");
        let client_address = @0x123;

        // Register a new client type
        ClientModule::register_client(alice, client_type, client_address);

        let get_client = ClientModule::get_client(alice, client_type);
        assert!(get_client == client_address, E_GET_CLIENT_IMPL);
    }

    #[test(alice = @0x444444)]
    #[expected_failure(abort_code = 1002)]
    public fun test_get_client_fail(alice: &signer) {
        ClientModule::create_ibc_store(alice);

        let client_type = StringModule::utf8(b"test_client");
        let client_address = @0x123;

        // Register a new client type
        ClientModule::register_client(alice, client_type, client_address);

        let unknown_client_type = StringModule::utf8(b"unknown_client_type");
        let get_client = ClientModule::get_client(alice, unknown_client_type); // expect it to abort with E_CLIENT_IMPL_NOT_FOUND
    }

    // Test for the register_client function with same client_type 2 times
    #[test(alice = @0x444444)]
    #[expected_failure(abort_code = 1001)]
    public fun test_register_client_fail_on_duplicate(alice: &signer) {
        // Initialize the IBCStore in Alice's account
        ClientModule::create_ibc_store(alice);

        // Define a client type and address for testing
        let client_type = StringModule::utf8(b"test_client");
        let client_address = @0x123;

        // Register a new client type
        ClientModule::register_client(alice, client_type, client_address);

        // Attempt to register the same client type again, expecting it to abort with E_CLIENT_ALREADY_EXISTS
        ClientModule::register_client(alice, client_type, client_address);
    }


    #[test(alice = @0x444444)]
    public fun test_generate_client_identifier(alice: &signer) {
        ClientModule::create_ibc_store(alice);

        let client_type = StringModule::utf8(b"test_client");

        // Generate the first identifier
        let identifier1 = ClientModule::generate_client_identifier(alice, client_type);
        let expected_identifier1 = StringModule::utf8(b"test_client-0"); 
        debug::print(&identifier1);
        assert!(identifier1 == expected_identifier1, E_GENERATE_CLIENT_IDENTIFIER);

        // Generate the second identifier
        let identifier2 = ClientModule::generate_client_identifier(alice, client_type);
        let expected_identifier2 = StringModule::utf8(b"test_client-1");
        assert!(identifier2 == expected_identifier2, E_GENERATE_CLIENT_IDENTIFIER);
    }

    #[test(alice = @0x444444)]
    #[expected_failure(abort_code = 9999)]
    public fun test_create_client_lc_failed(alice: &signer) {
        ClientModule::create_ibc_store(alice);

        // Register the mock client type and address
        let client_type = StringModule::utf8(b"mock_client");
        let mock_address = @0x2; // This would be the address of the MockLightClient in a real scenario
        ClientModule::register_client(alice, client_type, mock_address);

        // Create a MsgCreateClient message using the factory function
        let msg = ClientModule::new_msg_create_client(
            client_type,
            StringModule::utf8(b""),
            StringModule::utf8(b""),
            @0x3
        );

        ClientModule::create_client(alice, msg);
    }

    #[test(alice = @0x444444)]
    public fun test_create_client(alice: &signer) {
        ClientModule::create_ibc_store(alice);

        // Register the mock client type and address
        let client_type = StringModule::utf8(b"mock_client");
        let mock_address = @0x2; // This would be the address of the MockLightClient in a real scenario
        ClientModule::register_client(alice, client_type, mock_address);

        // Create a MsgCreateClient message using the factory function
        let msg = ClientModule::new_msg_create_client(
            client_type,
            StringModule::utf8(b"state_bytes"),
            StringModule::utf8(b"consensus_bytes"),
            @0x3
        );

        let client_id = ClientModule::create_client(alice, msg);
        assert!(client_id == client_type, E_CREATE_CLIENT);

    }
}
