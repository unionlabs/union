#[test_only]
module 0xcafe::reentrant_token {
    use aptos_framework::fungible_asset::{FungibleAsset, TransferRef};
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::object::{ConstructorRef, Object};
    use aptos_framework::function_info;

    use std::option;
    use std::signer;
    use std::string;

    public fun initialize(account: &signer, constructor_ref: &ConstructorRef) {
        assert!(signer::address_of(account) == @0xcafe, 1);
        let deposit = function_info::new_function_info(
            account,
            string::utf8(b"reentrant_token"),
            string::utf8(b"deposit"),
        );

        dispatchable_fungible_asset::register_dispatch_functions(
            constructor_ref,
            option::none(),
            option::some(deposit),
            option::none()
        );
    }

    public fun deposit<T: key>(
        store: Object<T>,
        fa: FungibleAsset,
        _transfer_ref: &TransferRef,
    ) {
        // Re-entering into dispatchable_fungible_asset. Will be rejected by the MoveVM runtime.
        dispatchable_fungible_asset::deposit(store, fa);
    }
}
