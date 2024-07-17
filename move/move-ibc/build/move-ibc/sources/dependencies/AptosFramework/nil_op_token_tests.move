#[test_only]
module aptos_framework::nil_op_token_tests {
    use aptos_framework::fungible_asset::{Self, Metadata, TestToken};
    use aptos_framework::dispatchable_fungible_asset;
    use 0xcafe::nil_op_token;
    use aptos_framework::object;
    use std::option;

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code=0x70002, location=aptos_framework::dispatchable_fungible_asset)]
    fun test_nil_op_token(
        creator: &signer,
    ) {
        let (creator_ref, token_object) = fungible_asset::create_test_token(creator);
        let (mint, _, _, _) = fungible_asset::init_test_metadata(&creator_ref);
        let metadata = object::convert<TestToken, Metadata>(token_object);

        let creator_store = fungible_asset::create_test_store(creator, metadata);

        nil_op_token::initialize(creator, &creator_ref);

        assert!(fungible_asset::supply(metadata) == option::some(0), 1);
        // Mint
        let fa = fungible_asset::mint(&mint, 100);
        assert!(fungible_asset::supply(metadata) == option::some(100), 2);
        // Deposit will cause an re-entrant call into dispatchable_fungible_asset
        dispatchable_fungible_asset::deposit(creator_store, fa);

        // Withdraw will fail because it's not drawing the basic amount.
        let fa = dispatchable_fungible_asset::withdraw(creator, creator_store, 10);
        dispatchable_fungible_asset::deposit(creator_store, fa);
    }
}
