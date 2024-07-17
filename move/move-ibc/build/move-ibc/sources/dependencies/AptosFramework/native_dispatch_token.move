#[test_only]
module 0xcafe::native_dispatch_token {
    use aptos_framework::fungible_asset::{FungibleAsset, TransferRef};
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::object::{ConstructorRef, Object};
    use aptos_framework::function_info;

    use std::option;
    use std::signer;
    use std::string;

    public fun initialize(account: &signer, constructor_ref: &ConstructorRef) {
        assert!(signer::address_of(account) == @0xcafe, 1);
        let withdraw = function_info::new_function_info(
            account,
            string::utf8(b"native_dispatch_token"),
            string::utf8(b"withdraw"),
        );

        dispatchable_fungible_asset::register_dispatch_functions(
            constructor_ref,
            option::some(withdraw),
            option::none(),
            option::none(),
        );
    }

    public native fun withdraw<T: key>(
        store: Object<T>,
        _amount: u64,
        transfer_ref: &TransferRef,
    ): FungibleAsset;
}
