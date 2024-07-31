module 0x1::ClientTest {

    use std::signer;
    use std::account;
    use std::vector;
    use std::debug;
    use aptos_std::string::{Self as StringModule, String};
    use aptos_framework::coin::Coin;
    use aptos_framework::aptos_coin::AptosCoin;
    // use ContractClientAddress::Client::{self, IBCStore};
    use ContractClientAddress::Client;

    use aptos_framework::smart_table::{Self as SmartTable, SmartTable};
    const E_GENERATE_CLIENT_IDENTIFIER: u64 = 3001;
    const E_GET_CLIENT_IMPL: u64 = 3002;

    // Test for the register_client function
    #[test(alice = @0x444444)]
    public fun test_register_client(alice: &signer) {
        // // Initialize the IBCStore in Alice's account
        Client::create_ibc_store(alice);

        // // Define a client type and address for testing
        let client_type = StringModule::utf8(b"test_client");
        let client_address = @0x123;

        // Register a new client type
        Client::register_client(alice, client_type, client_address);

        let get_client = Client::get_client(alice, client_type);
        assert!(get_client == client_address, E_GET_CLIENT_IMPL);
    }

    #[test(alice = @0x444444)]
    #[expected_failure(abort_code = 1002)]
    public fun test_get_client_fail(alice: &signer) {
        Client::create_ibc_store(alice);

        let client_type = StringModule::utf8(b"test_client");
        let client_address = @0x123;

        // Register a new client type
        Client::register_client(alice, client_type, client_address);

        let unknown_client_type = StringModule::utf8(b"unknown_client_type");
        let get_client = Client::get_client(alice, unknown_client_type); // expect it to abort with E_CLIENT_IMPL_NOT_FOUND
    }

    // Test for the register_client function with same client_type 2 times
    #[test(alice = @0x444444)]
    #[expected_failure(abort_code = 1001)]
    public fun test_register_client_fail_on_duplicate(alice: &signer) {
        // Initialize the IBCStore in Alice's account
        Client::create_ibc_store(alice);

        // Define a client type and address for testing
        let client_type = StringModule::utf8(b"test_client");
        let client_address = @0x123;

        // Register a new client type
        Client::register_client(alice, client_type, client_address);

        // Attempt to register the same client type again, expecting it to abort with E_CLIENT_ALREADY_EXISTS
        Client::register_client(alice, client_type, client_address);
    }


    #[test(alice = @0x444444)]
    public fun test_generate_client_identifier(alice: &signer) {
        Client::create_ibc_store(alice);

        let client_type = StringModule::utf8(b"test_client");

        // Generate the first identifier
        let identifier1 = Client::generate_client_identifier(alice, client_type);
        let expected_identifier1 = StringModule::utf8(b"\"test_client\"-\"0\""); // TODO: Fix this syntax later
        assert!(identifier1 == expected_identifier1, E_GENERATE_CLIENT_IDENTIFIER);

        // Generate the second identifier
        let identifier2 = Client::generate_client_identifier(alice, client_type);
        let expected_identifier2 = StringModule::utf8(b"\"test_client\"-\"1\""); // TODO: Fix this syntax later
        assert!(identifier2 == expected_identifier2, E_GENERATE_CLIENT_IDENTIFIER);
    }

    // #[test(alice = @0x444444)]
    // #[expected_failure(abort_code = 1002)]
    // public fun test_create_client_impl_not_found(alice: &signer) {
    //     Client::create_ibc_store(alice);

    //     let client_type = b"test_client";

    //     let identifier1 = Client::generate_client_identifier(alice, client_type);
    //     debug::print(&identifier1);
    //     assert!(identifier1 == b"test_client-0", E_GENERATE_CLIENT_IDENTIFIER);

    //     let identifier2 = Client::generate_client_identifier(alice, client_type);
    //     assert!(identifier2 == b"test_client-1", E_GENERATE_CLIENT_IDENTIFIER);
    // }

}
