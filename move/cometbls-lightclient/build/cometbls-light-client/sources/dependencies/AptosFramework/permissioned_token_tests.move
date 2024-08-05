#[test_only]
module 0xcafe::permissioned_token_tests {
    use aptos_framework::fungible_asset::{Self, Metadata, TestToken};
    use aptos_framework::dispatchable_fungible_asset;
    use 0xcafe::permissioned_token;
    use aptos_framework::object;
    use std::option;

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_permissioned_e2e_basic_flow(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);
        let allowed_sender = vector[
            object::object_address(&creator_store),
            object::object_address(&aaron_store),
        ];

        permissioned_token::initialize(creator, &creator_ref, allowed_sender);

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

        let fa = dispatchable_fungible_asset::withdraw(creator, creator_store, 10);
        assert!(fungible_asset::supply(metadata) == option::some(100), 3);
        dispatchable_fungible_asset::deposit(aaron_store, fa);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 1, location = 0xcafe::permissioned_token)]
    fun test_permissioned_disallowed_sender(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);
        let allowed_sender = vector[
            object::object_address(&creator_store),
        ];

        permissioned_token::initialize(creator, &creator_ref, allowed_sender);

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

        // aaron_store is not allowed to perform withdraw according to the allowlist rule
        let fa = dispatchable_fungible_asset::withdraw(aaron, aaron_store, 10);
        assert!(fungible_asset::supply(metadata) == option::some(100), 3);
        dispatchable_fungible_asset::deposit(creator_store, fa);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_permissioned_update_disallowed_sender(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);
        let aaron_store = fungible_asset::create_test_store(aaron, metadata);
        let allowed_sender = vector[
            object::object_address(&creator_store),
        ];

        permissioned_token::initialize(creator, &creator_ref, allowed_sender);

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

        permissioned_token::add_to_allow_list(creator, object::object_address(&aaron_store));
        // aaron_store is now allowed to perform withdraw
        let fa = dispatchable_fungible_asset::withdraw(aaron, aaron_store, 1);
        dispatchable_fungible_asset::deposit(creator_store, fa);
    }
}
