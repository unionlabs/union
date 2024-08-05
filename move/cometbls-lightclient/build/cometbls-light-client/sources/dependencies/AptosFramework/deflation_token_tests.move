#[test_only]
module 0xcafe::deflation_token_tests {
    use aptos_framework::account;
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::function_info;
    use aptos_framework::fungible_asset::{Self, Metadata, TestToken};
    use aptos_framework::object;
    use aptos_framework::primary_fungible_store;
    use 0xcafe::deflation_token;
    use std::option;
    use std::string;
    use std::signer;

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_deflation_e2e_basic_flow(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);

        deflation_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::is_store_dispatchable(creator_store), 1);
        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        // Deposit
        dispatchable_fungible_asset::deposit(creator_store, fa);
        // Withdraw
        let fa = dispatchable_fungible_asset::withdraw(creator, creator_store, 5);
        assert!(fungible_asset::supply(metadata) == option::some(100), 3);
        dispatchable_fungible_asset::deposit(aaron_store, fa);

        assert!(fungible_asset::balance(creator_store) == 95, 42);
        assert!(fungible_asset::balance(aaron_store) == 5, 42);

        // Withdrawing 10 token will cause 1 token to be burned.
        let fa = dispatchable_fungible_asset::withdraw(creator, creator_store, 10);
        assert!(fungible_asset::supply(metadata) == option::some(99), 3);
        dispatchable_fungible_asset::deposit(aaron_store, fa);

        assert!(fungible_asset::balance(creator_store) == 84, 42);
        assert!(fungible_asset::balance(aaron_store) == 15, 42);

        dispatchable_fungible_asset::transfer(creator, creator_store, aaron_store, 10);
        assert!(fungible_asset::balance(creator_store) == 73, 42);
        assert!(fungible_asset::balance(aaron_store) == 25, 42);

        dispatchable_fungible_asset::transfer_assert_minimum_deposit(creator, creator_store, aaron_store, 10, 10);
        assert!(fungible_asset::balance(creator_store) == 62, 42);
        assert!(fungible_asset::balance(aaron_store) == 35, 42);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x70002, location = aptos_framework::dispatchable_fungible_asset)]
    fun test_deflation_assert_min_deposit(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);

        deflation_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        dispatchable_fungible_asset::deposit(creator_store, fa);

        dispatchable_fungible_asset::transfer_assert_minimum_deposit(creator, creator_store, aaron_store, 10, 11);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x1001C, location = aptos_framework::fungible_asset)]
    fun test_deflation_fa_deposit(
        creator: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);

        deflation_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        dispatchable_fungible_asset::deposit(creator_store, fa);

        // Withdraw would fail if using existing FA api.
        let fa = fungible_asset::withdraw(creator, creator_store, 5);
        fungible_asset::deposit(creator_store, fa);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x1001C, location = aptos_framework::fungible_asset)]
    fun test_deflation_fa_withdraw(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);

        deflation_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        // Deposit
        dispatchable_fungible_asset::deposit(creator_store, fa);
        // Withdraw would fail if using existing FA api.
        let fa = fungible_asset::withdraw(creator, creator_store, 5);
        assert!(fungible_asset::supply(metadata) == option::some(100), 3);
        dispatchable_fungible_asset::deposit(aaron_store, fa);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x8001D, location = aptos_framework::fungible_asset)]
    fun test_double_init(
        creator: &signer,
    ) {
        let (creator_ref, _) = fungible_asset::create_test_token(creator);
        let (_, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        deflation_token::initialize(creator, &creator_ref);

        let withdraw = function_info::new_function_info(
            creator,
            string::utf8(b"deflation_token"),
            string::utf8(b"withdraw"),
        );

        // Re-registering the overload function should yield an error
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::none(),
            option::none()
        );
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x10019, location = aptos_framework::fungible_asset)]
    fun test_register_bad_withdraw(
        creator: &signer,
    ) {
        let (creator_ref, _) = fungible_asset::create_test_token(creator);

        let withdraw = function_info::new_function_info(
            creator,
            string::utf8(b"deflation_token"),
            string::utf8(b"initialize"),
        );

        // Change the deposit and withdraw function. Should give a type mismatch error.
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::none(),
            option::none()
        );
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x1001A, location = aptos_framework::fungible_asset)]
    fun test_register_bad_deposit(
        creator: &signer,
    ) {
        let (creator_ref, _) = fungible_asset::create_test_token(creator);

        let withdraw = function_info::new_function_info(
            creator,
            string::utf8(b"deflation_token"),
            string::utf8(b"withdraw"),
        );

        // Change the deposit and withdraw function. Should give a type mismatch error.
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::some(withdraw),
            option::none()
        );
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x1001B, location = aptos_framework::fungible_asset)]
    fun test_register_bad_value(
        creator: &signer,
    ) {
        let (creator_ref, _) = fungible_asset::create_test_token(creator);

        let withdraw = function_info::new_function_info(
            creator,
            string::utf8(b"deflation_token"),
            string::utf8(b"withdraw"),
        );

        // Change the deposit and withdraw function. Should give a type mismatch error.
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::none(),
            option::some(withdraw)
        );
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(major_status=1081, location = aptos_framework::function_info)]
    fun test_register_bad_withdraw_non_exist(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, _) = fungible_asset::create_test_token(creator);

        let withdraw = function_info::new_function_info(
            aaron,
            string::utf8(b"deflation_token"),
            string::utf8(b"withdraw"),
        );

        // Change the deposit and withdraw function. Should give a type mismatch error.
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::none(),
            option::none()
        );
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code=2, location = aptos_framework::function_info)]
    fun test_register_bad_withdraw_non_exist_2(
        creator: &signer,
    ) {
        let (creator_ref, _) = fungible_asset::create_test_token(creator);

        let withdraw = function_info::new_function_info(
            creator,
            string::utf8(b"deflation_token"),
            string::utf8(b"withdraw2"),
        );

        // Change the deposit and withdraw function. Should give a type mismatch error.
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::none(),
            option::none()
        );
    }

    #[test(creator = @0xcafe)]
    fun test_calling_overloadable_api_on_regular_fa(
        creator: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        // Deposit would succeed
        dispatchable_fungible_asset::deposit(creator_store, fa);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code=0x6001E, location = aptos_framework::fungible_asset)]
    fun test_register_on_non_metadata_object(
        creator: &signer,
    ) {
        account::create_account_for_test(signer::address_of(creator));
        let creator_ref = object::create_named_object(creator, b"TEST");
         let withdraw = function_info::new_function_info(
            creator,
            string::utf8(b"deflation_token"),
            string::utf8(b"withdraw"),
        );

        // Change the deposit and withdraw function. Should give a type mismatch error.
        dispatchable_fungible_asset::register_dispatch_functions(
            &creator_ref,
            option::some(withdraw),
            option::none(),
            option::none()
        );
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_basic_flow_primary_fa(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint_ref, transfer_ref, burn_ref) = primary_fungible_store::init_test_metadata_with_primary_store_enabled(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        deflation_token::initialize(creator, &creator_ref);
        let creator_address = signer::address_of(creator);
        let aaron_address = signer::address_of(aaron);
        assert!(primary_fungible_store::balance(creator_address, metadata) == 0, 1);
        assert!(primary_fungible_store::balance(aaron_address, metadata) == 0, 2);
        primary_fungible_store::mint(&mint_ref, creator_address, 100);
        primary_fungible_store::transfer(creator, metadata, aaron_address, 80);

        assert!(primary_fungible_store::balance(creator_address, metadata) == 12, 3);
        assert!(primary_fungible_store::balance(aaron_address, metadata) == 80, 4);

        let fa = primary_fungible_store::withdraw(aaron, metadata, 10);
        primary_fungible_store::deposit(creator_address, fa);
        assert!(primary_fungible_store::balance(creator_address, metadata) == 22, 3);
        assert!(primary_fungible_store::balance(aaron_address, metadata) == 69, 4);

        primary_fungible_store::set_frozen_flag(&transfer_ref, aaron_address, true);
        assert!(primary_fungible_store::is_frozen(aaron_address, metadata), 5);
        let fa = primary_fungible_store::withdraw_with_ref(&transfer_ref, aaron_address, 30);

        assert!(primary_fungible_store::balance(creator_address, metadata) == 22, 3);
        assert!(primary_fungible_store::balance(aaron_address, metadata) == 39, 4);

        primary_fungible_store::deposit_with_ref(&transfer_ref, aaron_address, fa);

        assert!(primary_fungible_store::balance(aaron_address, metadata) == 69, 4);
        primary_fungible_store::transfer_with_ref(&transfer_ref, aaron_address, creator_address, 20);

        assert!(primary_fungible_store::balance(creator_address, metadata) == 42, 3);
        assert!(primary_fungible_store::balance(aaron_address, metadata) == 49, 4);
        primary_fungible_store::set_frozen_flag(&transfer_ref, aaron_address, false);
        assert!(!primary_fungible_store::is_frozen(aaron_address, metadata), 6);

        primary_fungible_store::burn(&burn_ref, aaron_address, 49);
        assert!(primary_fungible_store::balance(aaron_address, metadata) == 0, 7);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::fungible_asset)]
    fun test_deflation_set_frozen(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, transfer_ref, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);

        deflation_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        // Deposit
        dispatchable_fungible_asset::deposit(creator_store, fa);
        // Withdraw
        let fa = dispatchable_fungible_asset::withdraw(creator, creator_store, 5);
        assert!(fungible_asset::supply(metadata) == option::some(100), 3);
        dispatchable_fungible_asset::deposit(aaron_store, fa);

        assert!(fungible_asset::balance(creator_store) == 95, 42);
        assert!(fungible_asset::balance(aaron_store) == 5, 42);

        fungible_asset::set_frozen_flag(&transfer_ref, aaron_store, true);

        let fa = dispatchable_fungible_asset::withdraw(creator, creator_store, 5);
        dispatchable_fungible_asset::deposit(aaron_store, fa);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50008, location = aptos_framework::fungible_asset)]
    fun test_deflation_wrong_withdraw(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);

        deflation_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);

        // Withdraw
        let fa = dispatchable_fungible_asset::withdraw(aaron, creator_store, 5);
        assert!(fungible_asset::supply(metadata) == option::some(100), 3);
        dispatchable_fungible_asset::deposit(aaron_store, fa);
    }
}
