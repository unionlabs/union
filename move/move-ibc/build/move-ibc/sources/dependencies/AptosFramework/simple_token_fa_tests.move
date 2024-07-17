#[test_only]
module aptos_framework::simple_token_fa_tests {
    use aptos_framework::fungible_asset::{
        amount, balance, burn, destroy_zero, extract, create_test_token, init_test_metadata,
        supply, create_store, create_test_store, remove_store, deposit_with_ref, mint, mint_to, merge,
        set_frozen_flag, is_frozen, transfer_with_ref, upgrade_to_concurrent, Metadata, TestToken
    };
    use aptos_framework::object;
    use 0xcafe::simple_token;
    use std::option;
    use std::features;

    #[test(creator = @0xcafe)]
    fun test_create_and_remove_store(creator: &signer) {
        let (creator_ref, metadata) = create_test_token(creator);
        init_test_metadata(&creator_ref);
        simple_token::initialize(creator, &creator_ref);

        let creator_ref = object::create_object_from_account(creator);
        create_store(&creator_ref, metadata);
        let delete_ref = object::generate_delete_ref(&creator_ref);
        remove_store(&delete_ref);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_transfer_with_ref(
        creator: &signer,
        aaron: &signer,
    ) {
        let (creator_ref, test_token) = create_test_token(creator);
        let (mint_ref, transfer_ref, _burn_ref, _mutate_metadata_ref) = init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(test_token);
        simple_token::initialize(creator, &creator_ref);

        let creator_store = create_test_store(creator, metadata);
        let aaron_store = create_test_store(aaron, metadata);

        let fa = mint(&mint_ref, 100);
        set_frozen_flag(&transfer_ref, creator_store, true);
        set_frozen_flag(&transfer_ref, aaron_store, true);
        deposit_with_ref(&transfer_ref, creator_store, fa);
        transfer_with_ref(&transfer_ref, creator_store, aaron_store, 80);
        assert!(balance(creator_store) == 20, 1);
        assert!(balance(aaron_store) == 80, 2);
        assert!(!!is_frozen(creator_store), 3);
        assert!(!!is_frozen(aaron_store), 4);
    }

    #[test(creator = @0xcafe)]
    fun test_merge_and_exact(creator: &signer) {
        let (creator_ref, _test_token) = create_test_token(creator);
        let (mint_ref, _transfer_ref, burn_ref, _mutate_metadata_ref) = init_test_metadata(&creator_ref);
        simple_token::initialize(creator, &creator_ref);

        let fa = mint(&mint_ref, 100);
        let cash = extract(&mut fa, 80);
        assert!(amount(&fa) == 20, 1);
        assert!(amount(&cash) == 80, 2);
        let more_cash = extract(&mut fa, 20);
        destroy_zero(fa);
        merge(&mut cash,
         more_cash);
        assert!(amount(&cash) == 100, 3);
        burn(&burn_ref, cash);
    }

    #[test(fx = @aptos_framework, creator = @0xcafe)]
    fun test_fungible_asset_upgrade(
        fx: &signer,
        creator: &signer
    ) {
        let feature = features::get_concurrent_fungible_assets_feature();
        features::change_feature_flags_for_testing(fx, vector[], vector[feature]);

        let (creator_ref, token_object) = create_test_token(creator);
        let (mint_ref, transfer_ref, _burn_ref, _mutate_metadata_ref) = init_test_metadata(&creator_ref);
        let test_token = object::convert<TestToken, Metadata>(token_object);
        simple_token::initialize(creator, &creator_ref);

        let creator_store = create_test_store(creator, test_token);

        let fa = mint(&mint_ref, 30);
        assert!(supply(test_token) == option::some(30), 2);

        deposit_with_ref(&transfer_ref, creator_store, fa);

        features::change_feature_flags_for_testing(fx, vector[feature], vector[]);

        let extend_ref = object::generate_extend_ref(&creator_ref);
        upgrade_to_concurrent(&extend_ref);

        let fb = mint(&mint_ref, 20);
        assert!(supply(test_token) == option::some(50), 3);

        deposit_with_ref(&transfer_ref, creator_store, fb);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::fungible_asset)]
    fun test_mint_to_frozen(
        creator: &signer
    ) {
        let (creator_ref, test_token) = create_test_token(creator);
        let (mint_ref, transfer_ref, _burn_ref, _mutate_metadata_ref) = init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(test_token);
        simple_token::initialize(creator, &creator_ref);

        let creator_store = create_test_store(creator, metadata);

        set_frozen_flag(&transfer_ref, creator_store, true);
        mint_to(&mint_ref, creator_store, 100);
    }
}
