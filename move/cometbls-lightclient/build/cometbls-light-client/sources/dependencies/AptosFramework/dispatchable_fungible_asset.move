/// This defines the fungible asset module that can issue fungible asset of any `Metadata` object. The
/// metadata object can be any object that equipped with `Metadata` resource.
///
/// The dispatchable_fungible_asset wraps the existing fungible_asset module and adds the ability for token issuer
/// to customize the logic for withdraw and deposit operations. For example:
///
/// - Deflation token: a fixed percentage of token will be destructed upon transfer.
/// - Transfer allowlist: token can only be transfered to addresses in the allow list.
/// - Predicated transfer: transfer can only happen when some certain predicate has been met.
/// - Loyalty token: a fixed loyalty will be paid to a designated address when a fungible asset transfer happens
///
/// The api listed here intended to be an in-place replacement for defi applications that uses fungible_asset api directly
/// and is safe for non-dispatchable (aka vanilla) fungible assets as well.
///
/// See AIP-73 for further discussion
///
module aptos_framework::dispatchable_fungible_asset {
    use aptos_framework::fungible_asset::{Self, FungibleAsset, TransferRef};
    use aptos_framework::function_info::{Self, FunctionInfo};
    use aptos_framework::object::{Self, ConstructorRef, Object};

    use std::error;
    use std::features;
    use std::option::{Self, Option};

    /// TransferRefStore doesn't exist on the fungible asset type.
    const ESTORE_NOT_FOUND: u64 = 1;
    /// Recipient is not getting the guaranteed value;
    const EAMOUNT_MISMATCH: u64 = 2;
    /// Feature is not activated yet on the network.
    const ENOT_ACTIVATED: u64 = 3;
    /// Dispatch target is not loaded.
    const ENOT_LOADED: u64 = 4;

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct TransferRefStore has key {
        transfer_ref: TransferRef
    }

    public fun register_dispatch_functions(
        constructor_ref: &ConstructorRef,
        withdraw_function: Option<FunctionInfo>,
        deposit_function: Option<FunctionInfo>,
        derived_balance_function: Option<FunctionInfo>,
    ) {
        fungible_asset::register_dispatch_functions(
            constructor_ref,
            withdraw_function,
            deposit_function,
            derived_balance_function,
        );
        let store_obj = &object::generate_signer(constructor_ref);
        move_to<TransferRefStore>(
            store_obj,
            TransferRefStore {
                transfer_ref: fungible_asset::generate_transfer_ref(constructor_ref),
            }
        );
    }

    public fun register_derive_supply_dispatch_function(
        constructor_ref: &ConstructorRef,
        dispatch_function: Option<FunctionInfo>
    ) {
        fungible_asset::register_derive_supply_dispatch_function(
            constructor_ref,
            dispatch_function
        );
    }

    /// Withdraw `amount` of the fungible asset from `store` by the owner.
    ///
    /// The semantics of deposit will be governed by the function specified in DispatchFunctionStore.
    public fun withdraw<T: key>(
        owner: &signer,
        store: Object<T>,
        amount: u64,
    ): FungibleAsset acquires TransferRefStore {
        fungible_asset::withdraw_sanity_check(owner, store, false);
        let func_opt = fungible_asset::withdraw_dispatch_function(store);
        if (option::is_some(&func_opt)) {
            assert!(
                features::dispatchable_fungible_asset_enabled(),
                error::aborted(ENOT_ACTIVATED)
            );
            let start_balance = fungible_asset::balance(store);
            let func = option::borrow(&func_opt);
            function_info::load_module_from_function(func);
            let fa = dispatchable_withdraw(
                store,
                amount,
                borrow_transfer_ref(store),
                func,
            );
            let end_balance = fungible_asset::balance(store);
            assert!(amount <= start_balance - end_balance, error::aborted(EAMOUNT_MISMATCH));
            fa
        } else {
            fungible_asset::withdraw_internal(object::object_address(&store), amount)
        }
    }

    /// Deposit `amount` of the fungible asset to `store`.
    ///
    /// The semantics of deposit will be governed by the function specified in DispatchFunctionStore.
    public fun deposit<T: key>(store: Object<T>, fa: FungibleAsset) acquires TransferRefStore {
        fungible_asset::deposit_sanity_check(store, false);
        let func_opt = fungible_asset::deposit_dispatch_function(store);
        if (option::is_some(&func_opt)) {
            assert!(
                features::dispatchable_fungible_asset_enabled(),
                error::aborted(ENOT_ACTIVATED)
            );
            let func = option::borrow(&func_opt);
            function_info::load_module_from_function(func);
            dispatchable_deposit(
                store,
                fa,
                borrow_transfer_ref(store),
                func
            )
        } else {
            fungible_asset::deposit_internal(object::object_address(&store), fa)
        }
    }

    /// Transfer an `amount` of fungible asset from `from_store`, which should be owned by `sender`, to `receiver`.
    /// Note: it does not move the underlying object.
    public entry fun transfer<T: key>(
        sender: &signer,
        from: Object<T>,
        to: Object<T>,
        amount: u64,
    ) acquires TransferRefStore {
        let fa = withdraw(sender, from, amount);
        deposit(to, fa);
    }

    /// Transfer an `amount` of fungible asset from `from_store`, which should be owned by `sender`, to `receiver`.
    /// The recipient is guranteed to receive asset greater than the expected amount.
    /// Note: it does not move the underlying object.
    public entry fun transfer_assert_minimum_deposit<T: key>(
        sender: &signer,
        from: Object<T>,
        to: Object<T>,
        amount: u64,
        expected: u64
    ) acquires TransferRefStore {
        let start = fungible_asset::balance(to);
        let fa = withdraw(sender, from, amount);
        deposit(to, fa);
        let end = fungible_asset::balance(to);
        assert!(end - start >= expected, error::aborted(EAMOUNT_MISMATCH));
    }

    #[view]
    /// Get the derived value of store using the overloaded hook.
    ///
    /// The semantics of value will be governed by the function specified in DispatchFunctionStore.
    public fun derived_balance<T: key>(store: Object<T>): u64 {
        let func_opt = fungible_asset::derived_balance_dispatch_function(store);
        if (option::is_some(&func_opt)) {
            assert!(
                features::dispatchable_fungible_asset_enabled(),
                error::aborted(ENOT_ACTIVATED)
            );
            let func = option::borrow(&func_opt);
            function_info::load_module_from_function(func);
            dispatchable_derived_balance(store, func)
        } else {
            fungible_asset::balance(store)
        }
    }

    #[view]
    /// Get the derived supply of the fungible asset using the overloaded hook.
    ///
    /// The semantics of supply will be governed by the function specified in DeriveSupplyDispatch.
    public fun derived_supply<T: key>(metadata: Object<T>): Option<u128> {
        let func_opt = fungible_asset::derived_supply_dispatch_function(metadata);
        if (option::is_some(&func_opt)) {
            assert!(
                features::dispatchable_fungible_asset_enabled(),
                error::aborted(ENOT_ACTIVATED)
            );
            let func = option::borrow(&func_opt);
            function_info::load_module_from_function(func);
            dispatchable_derived_supply(metadata, func)
        } else {
            fungible_asset::supply(metadata)
        }
    }

    inline fun borrow_transfer_ref<T: key>(metadata: Object<T>): &TransferRef acquires TransferRefStore {
        let metadata_addr = object::object_address(
            &fungible_asset::store_metadata(metadata)
        );
        assert!(
            exists<TransferRefStore>(metadata_addr),
            error::not_found(ESTORE_NOT_FOUND)
        );
        &borrow_global<TransferRefStore>(metadata_addr).transfer_ref
    }

    native fun dispatchable_withdraw<T: key>(
        store: Object<T>,
        amount: u64,
        transfer_ref: &TransferRef,
        function: &FunctionInfo,
    ): FungibleAsset;

    native fun dispatchable_deposit<T: key>(
        store: Object<T>,
        fa: FungibleAsset,
        transfer_ref: &TransferRef,
        function: &FunctionInfo,
    );

    native fun dispatchable_derived_balance<T: key>(
        store: Object<T>,
        function: &FunctionInfo,
    ): u64;

    native fun dispatchable_derived_supply<T: key>(
        store: Object<T>,
        function: &FunctionInfo,
    ): Option<u128>;
}
