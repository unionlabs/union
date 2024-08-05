#[test_only]
module 0xcafe::permissioned_token {
    use aptos_framework::fungible_asset::{Self, FungibleAsset, TransferRef};
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::object::{Self, ConstructorRef, Object};
    use aptos_framework::function_info;

    use std::option;
    use std::signer;
    use std::string;
    use std::vector;

    /// Provided withdraw function type doesn't meet the signature requirement.
    const EWITHDRAW_NOT_ALLOWED: u64 = 1;

    struct AllowlistStore has key {
        allowed_sender: vector<address>,
    }

    public fun initialize(account: &signer, constructor_ref: &ConstructorRef, allowed_sender: vector<address>) {
        assert!(signer::address_of(account) == @0xcafe, 1);
        move_to<AllowlistStore>(account, AllowlistStore { allowed_sender });

        let withdraw = function_info::new_function_info(
            account,
            string::utf8(b"permissioned_token"),
            string::utf8(b"withdraw"),
        );

        dispatchable_fungible_asset::register_dispatch_functions(
            constructor_ref,
            option::some(withdraw),
            option::none(),
            option::none(),
        );
    }

    public fun add_to_allow_list(account: &signer, new_address: address) acquires AllowlistStore {
        assert!(signer::address_of(account) == @0xcafe, 1);
        let allowed_sender = borrow_global_mut<AllowlistStore>(@0xcafe);
        if(!vector::contains(&allowed_sender.allowed_sender, &new_address)) {
            vector::push_back(&mut allowed_sender.allowed_sender, new_address);
        }
    }

    public fun withdraw<T: key>(
        store: Object<T>,
        amount: u64,
        transfer_ref: &TransferRef,
    ): FungibleAsset acquires AllowlistStore {
        assert!(vector::contains(
            &borrow_global<AllowlistStore>(@0xcafe).allowed_sender,
            &object::object_address(&store)
        ), EWITHDRAW_NOT_ALLOWED);

        fungible_asset::withdraw_with_ref(transfer_ref, store, amount)
    }
}
