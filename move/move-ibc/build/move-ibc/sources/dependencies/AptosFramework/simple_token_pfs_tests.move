#[test_only]
module aptos_framework::simple_token_pfs_tests {
    use aptos_framework::fungible_asset::{create_test_token};
    use aptos_framework::primary_fungible_store::{
        balance, burn, deposit, mint, primary_store, transfer, transfer_assert_minimum_deposit,
        withdraw, init_test_metadata_with_primary_store_enabled, is_frozen, set_frozen_flag,
        transfer_with_ref, deposit_with_ref, withdraw_with_ref, primary_store_exists,
        ensure_primary_store_exists
    };
    use aptos_framework::object;
    use 0xcafe::simple_token;
    use std::signer;

    // Copied from primary_fungible_store tests.
    #[test(user_1 = @0xcafe, user_2 = @0xface)]
    fun test_transfer_to_burnt_store(
        user_1: &signer,
        user_2: &signer,
    ) {
        let (creator_ref, metadata) = create_test_token(user_1);
        let (mint_ref, _, _) = init_test_metadata_with_primary_store_enabled(&creator_ref);
        simple_token::initialize(user_1, &creator_ref);

        let user_1_address = signer::address_of(user_1);
        let user_2_address = signer::address_of(user_2);
        mint(&mint_ref, user_1_address, 100);
        transfer(user_1, metadata, user_2_address, 80);

        // User 2 burns their primary store but should still be able to transfer afterward.
        let user_2_primary_store = primary_store(user_2_address, metadata);
        object::burn(user_2, user_2_primary_store);
        assert!(object::is_burnt(user_2_primary_store), 0);
        // Balance still works
        assert!(balance(user_2_address, metadata) == 80, 0);
        // Deposit still works
        transfer(user_1, metadata, user_2_address, 20);
        transfer(user_2, metadata, user_1_address, 90);
        assert!(balance(user_2_address, metadata) == 10, 0);
    }

    #[test(user_1 = @0xcafe, user_2 = @0xface)]
    fun test_withdraw_from_burnt_store(
        user_1: &signer,
        user_2: &signer,
    ) {
        let (creator_ref, metadata) = create_test_token(user_1);
        let (mint_ref, _, _) = init_test_metadata_with_primary_store_enabled(&creator_ref);
        simple_token::initialize(user_1, &creator_ref);

        let user_1_address = signer::address_of(user_1);
        let user_2_address = signer::address_of(user_2);
        mint(&mint_ref, user_1_address, 100);
        transfer(user_1, metadata, user_2_address, 80);

        // User 2 burns their primary store but should still be able to withdraw afterward.
        let user_2_primary_store = primary_store(user_2_address, metadata);
        object::burn(user_2, user_2_primary_store);
        assert!(object::is_burnt(user_2_primary_store), 0);
        let coins = withdraw(user_2, metadata, 70);
        assert!(balance(user_2_address, metadata) == 10, 0);
        deposit(user_2_address, coins);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_default_behavior(creator: &signer, aaron: &signer) {
        let (creator_ref, metadata) = create_test_token(creator);
        init_test_metadata_with_primary_store_enabled(&creator_ref);
        simple_token::initialize(creator, &creator_ref);

        let creator_address = signer::address_of(creator);
        let aaron_address = signer::address_of(aaron);
        assert!(!primary_store_exists(creator_address, metadata), 1);
        assert!(!primary_store_exists(aaron_address, metadata), 2);
        assert!(balance(creator_address, metadata) == 0, 3);
        assert!(balance(aaron_address, metadata) == 0, 4);
        assert!(!is_frozen(creator_address, metadata), 5);
        assert!(!is_frozen(aaron_address, metadata), 6);
        ensure_primary_store_exists(creator_address, metadata);
        ensure_primary_store_exists(aaron_address, metadata);
        assert!(primary_store_exists(creator_address, metadata), 7);
        assert!(primary_store_exists(aaron_address, metadata), 8);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_basic_flow(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, metadata) = create_test_token(creator);
        let (mint_ref, transfer_ref, burn_ref) = init_test_metadata_with_primary_store_enabled(&creator_ref);
        simple_token::initialize(creator, &creator_ref);

        let creator_address = signer::address_of(creator);
        let aaron_address = signer::address_of(aaron);
        assert!(balance(creator_address, metadata) == 0, 1);
        assert!(balance(aaron_address, metadata) == 0, 2);
        mint(&mint_ref, creator_address, 100);
        transfer(creator, metadata, aaron_address, 80);
        let fa = withdraw(aaron, metadata, 10);
        deposit(creator_address, fa);
        assert!(balance(creator_address, metadata) == 30, 3);
        assert!(balance(aaron_address, metadata) == 70, 4);
        set_frozen_flag(&transfer_ref, aaron_address, true);
        assert!(is_frozen(aaron_address, metadata), 5);
        let fa = withdraw_with_ref(&transfer_ref, aaron_address, 30);
        deposit_with_ref(&transfer_ref, aaron_address, fa);
        transfer_with_ref(&transfer_ref, aaron_address, creator_address, 20);
        set_frozen_flag(&transfer_ref, aaron_address, false);
        assert!(!is_frozen(aaron_address, metadata), 6);
        burn(&burn_ref, aaron_address, 50);
        assert!(balance(aaron_address, metadata) == 0, 7);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_basic_flow_with_min_balance(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, metadata) = create_test_token(creator);
        let (mint_ref, _transfer_ref, _) = init_test_metadata_with_primary_store_enabled(&creator_ref);
        simple_token::initialize(creator, &creator_ref);

        let creator_address = signer::address_of(creator);
        let aaron_address = signer::address_of(aaron);
        assert!(balance(creator_address, metadata) == 0, 1);
        assert!(balance(aaron_address, metadata) == 0, 2);
        mint(&mint_ref, creator_address, 100);
        transfer_assert_minimum_deposit(creator, metadata, aaron_address, 80, 80);
        let fa = withdraw(aaron, metadata, 10);
        deposit(creator_address, fa);
        assert!(balance(creator_address, metadata) == 30, 3);
        assert!(balance(aaron_address, metadata) == 70, 4);
    }
}
