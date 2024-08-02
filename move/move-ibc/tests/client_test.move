module IBC::ClientTest {

    use std::signer;
    use std::account;
    use std::vector;
    use std::debug;
    use aptos_std::string::{Self as StringModule, String};
    use aptos_framework::coin::Coin;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::event;
    use aptos_std::any::{Self, Any};
    use IBC::Core;


    use aptos_framework::smart_table::{Self as SmartTable, SmartTable};
    const E_GENERATE_CLIENT_IDENTIFIER: u64 = 3001;
    const E_GET_CLIENT_IMPL: u64 = 3002;
    const E_CREATE_CLIENT: u64 = 3003;


    #[test(alice = @IBC)]
    public fun test_generate_client_identifier(alice: &signer) {
        Core::create_ibc_store(alice);

        let client_type = StringModule::utf8(b"test_client");

        // Generate the first identifier
        let identifier1 = Core::generate_client_identifier(client_type);
        let expected_identifier1 = StringModule::utf8(b"test_client-0"); 
        assert!(identifier1 == expected_identifier1, E_GENERATE_CLIENT_IDENTIFIER);

        // Generate the second identifier
        let identifier2 = Core::generate_client_identifier(client_type);
        let expected_identifier2 = StringModule::utf8(b"test_client-1");
        assert!(identifier2 == expected_identifier2, E_GENERATE_CLIENT_IDENTIFIER);
    }

    #[test(alice = @IBC)]
    public fun test_create_client(alice: &signer) {
        Core::create_ibc_store(alice);

        // Register the mock client type and address
        let client_type = StringModule::utf8(b"mock_client");
        let mock_address = @0x2; // This would be the address of the MockLightClient in a real scenario

        let expected_client_id = StringModule::utf8(b"mock_client-0");

        let client_id = Core::create_client(
            client_type,
            any::pack(3),
            any::pack(3),
            @0x3
        );
        assert!(client_id == expected_client_id, E_CREATE_CLIENT);


        // Register the mock client type and address
        let client_type2 = StringModule::utf8(b"another_client");
        let mock_address2 = @0x2; // This would be the address of the MockLightClient in a real scenario

        let expected_client_id2 = StringModule::utf8(b"another_client-1");

        let client_id2 = Core::create_client(
            client_type2,
            any::pack(3),
            any::pack(3),
            @0x3
        );
        assert!(client_id2 == expected_client_id2, E_CREATE_CLIENT);

    }
}
