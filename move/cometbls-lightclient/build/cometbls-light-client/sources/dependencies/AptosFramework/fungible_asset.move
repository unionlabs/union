/// This defines the fungible asset module that can issue fungible asset of any `Metadata` object. The
/// metadata object can be any object that equipped with `Metadata` resource.
module aptos_framework::fungible_asset {
    use aptos_framework::aggregator_v2::{Self, Aggregator};
    use aptos_framework::create_signer;
    use aptos_framework::event;
    use aptos_framework::function_info::{Self, FunctionInfo};
    use aptos_framework::object::{Self, Object, ConstructorRef, DeleteRef, ExtendRef};
    use std::string;
    use std::features;

    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::string::String;

    friend aptos_framework::coin;
    friend aptos_framework::primary_fungible_store;
    friend aptos_framework::aptos_account;

    friend aptos_framework::dispatchable_fungible_asset;

    /// Amount cannot be zero.
    const EAMOUNT_CANNOT_BE_ZERO: u64 = 1;
    /// The transfer ref and the fungible asset do not match.
    const ETRANSFER_REF_AND_FUNGIBLE_ASSET_MISMATCH: u64 = 2;
    /// Store is disabled from sending and receiving this fungible asset.
    const ESTORE_IS_FROZEN: u64 = 3;
    /// Insufficient balance to withdraw or transfer.
    const EINSUFFICIENT_BALANCE: u64 = 4;
    /// The fungible asset's supply has exceeded maximum.
    const EMAX_SUPPLY_EXCEEDED: u64 = 5;
    /// Fungible asset do not match when merging.
    const EFUNGIBLE_ASSET_MISMATCH: u64 = 6;
    /// The mint ref and the store do not match.
    const EMINT_REF_AND_STORE_MISMATCH: u64 = 7;
    /// Account is not the store's owner.
    const ENOT_STORE_OWNER: u64 = 8;
    /// Transfer ref and store do not match.
    const ETRANSFER_REF_AND_STORE_MISMATCH: u64 = 9;
    /// Burn ref and store do not match.
    const EBURN_REF_AND_STORE_MISMATCH: u64 = 10;
    /// Fungible asset and store do not match.
    const EFUNGIBLE_ASSET_AND_STORE_MISMATCH: u64 = 11;
    /// Cannot destroy non-empty fungible assets.
    const EAMOUNT_IS_NOT_ZERO: u64 = 12;
    /// Burn ref and fungible asset do not match.
    const EBURN_REF_AND_FUNGIBLE_ASSET_MISMATCH: u64 = 13;
    /// Cannot destroy fungible stores with a non-zero balance.
    const EBALANCE_IS_NOT_ZERO: u64 = 14;
    /// Name of the fungible asset metadata is too long
    const ENAME_TOO_LONG: u64 = 15;
    /// Symbol of the fungible asset metadata is too long
    const ESYMBOL_TOO_LONG: u64 = 16;
    /// Decimals is over the maximum of 32
    const EDECIMALS_TOO_LARGE: u64 = 17;
    /// Fungibility is only available for non-deletable objects.
    const EOBJECT_IS_DELETABLE: u64 = 18;
    /// URI for the icon of the fungible asset metadata is too long
    const EURI_TOO_LONG: u64 = 19;
    /// The fungible asset's supply will be negative which should be impossible.
    const ESUPPLY_UNDERFLOW: u64 = 20;
    /// Supply resource is not found for a metadata object.
    const ESUPPLY_NOT_FOUND: u64 = 21;
    /// Flag for Concurrent Supply not enabled
    const ECONCURRENT_SUPPLY_NOT_ENABLED: u64 = 22;
    /// Flag for the existence of fungible store.
    const EFUNGIBLE_STORE_EXISTENCE: u64 = 23;
    /// Account is not the owner of metadata object.
    const ENOT_METADATA_OWNER: u64 = 24;
    /// Provided withdraw function type doesn't meet the signature requirement.
    const EWITHDRAW_FUNCTION_SIGNATURE_MISMATCH: u64 = 25;
    /// Provided deposit function type doesn't meet the signature requirement.
    const EDEPOSIT_FUNCTION_SIGNATURE_MISMATCH: u64 = 26;
    /// Provided derived_balance function type doesn't meet the signature requirement.
    const EDERIVED_BALANCE_FUNCTION_SIGNATURE_MISMATCH: u64 = 27;
    /// Invalid withdraw/deposit on dispatchable token. The specified token has a dispatchable function hook.
    /// Need to invoke dispatchable_fungible_asset::withdraw/deposit to perform transfer.
    const EINVALID_DISPATCHABLE_OPERATIONS: u64 = 28;
    /// Trying to re-register dispatch hook on a fungible asset.
    const EALREADY_REGISTERED: u64 = 29;
    /// Fungible metadata does not exist on this account.
    const EFUNGIBLE_METADATA_EXISTENCE: u64 = 30;
    /// Cannot register dispatch hook for APT.
    const EAPT_NOT_DISPATCHABLE: u64 = 31;
    /// Flag for Concurrent Supply not enabled
    const ECONCURRENT_BALANCE_NOT_ENABLED: u64 = 32;
    /// Provided derived_supply function type doesn't meet the signature requirement.
    const EDERIVED_SUPPLY_FUNCTION_SIGNATURE_MISMATCH: u64 = 33;

    //
    // Constants
    //

    const MAX_NAME_LENGTH: u64 = 32;
    const MAX_SYMBOL_LENGTH: u64 = 10;
    const MAX_DECIMALS: u8 = 32;
    const MAX_URI_LENGTH: u64 = 512;

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct Supply has key {
        current: u128,
        // option::none() means unlimited supply.
        maximum: Option<u128>,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct ConcurrentSupply has key {
        current: Aggregator<u128>,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Metadata of a Fungible asset
    struct Metadata has key, copy, drop {
        /// Name of the fungible metadata, i.e., "USDT".
        name: String,
        /// Symbol of the fungible metadata, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: String,
        /// Number of decimals used for display purposes.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        /// The Uniform Resource Identifier (uri) pointing to an image that can be used as the icon for this fungible
        /// asset.
        icon_uri: String,
        /// The Uniform Resource Identifier (uri) pointing to the website for the fungible asset.
        project_uri: String,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Defines a `FungibleAsset`, such that all `FungibleStore`s stores are untransferable at
    /// the object layer.
    struct Untransferable has key {}

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// The store object that holds fungible assets of a specific type associated with an account.
    struct FungibleStore has key {
        /// The address of the base metadata object.
        metadata: Object<Metadata>,
        /// The balance of the fungible metadata.
        balance: u64,
        /// If true, owner transfer is disabled that only `TransferRef` can move in/out from this store.
        frozen: bool,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct DispatchFunctionStore has key {
		withdraw_function: Option<FunctionInfo>,
		deposit_function: Option<FunctionInfo>,
        derived_balance_function: Option<FunctionInfo>,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct DeriveSupply has key {
        dispatch_function: Option<FunctionInfo>
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// The store object that holds concurrent fungible asset balance.
    struct ConcurrentFungibleBalance has key {
        /// The balance of the fungible metadata.
        balance: Aggregator<u64>,
    }

    /// FungibleAsset can be passed into function for type safety and to guarantee a specific amount.
    /// FungibleAsset is ephemeral and cannot be stored directly. It must be deposited back into a store.
    struct FungibleAsset {
        metadata: Object<Metadata>,
        amount: u64,
    }

    /// MintRef can be used to mint the fungible asset into an account's store.
    struct MintRef has drop, store {
        metadata: Object<Metadata>
    }

    /// TransferRef can be used to allow or disallow the owner of fungible assets from transferring the asset
    /// and allow the holder of TransferRef to transfer fungible assets from any account.
    struct TransferRef has drop, store {
        metadata: Object<Metadata>
    }

    /// BurnRef can be used to burn fungible assets from a given holder account.
    struct BurnRef has drop, store {
        metadata: Object<Metadata>
    }

    /// MutateMetadataRef can be used to directly modify the fungible asset's Metadata.
    struct MutateMetadataRef has drop, store {
        metadata: Object<Metadata>
    }

    #[event]
    /// Emitted when fungible assets are deposited into a store.
    struct Deposit has drop, store {
        store: address,
        amount: u64,
    }

    #[event]
    /// Emitted when fungible assets are withdrawn from a store.
    struct Withdraw has drop, store {
        store: address,
        amount: u64,
    }

    #[event]
    /// Emitted when a store's frozen status is updated.
    struct Frozen has drop, store {
        store: address,
        frozen: bool,
    }

    inline fun default_to_concurrent_fungible_supply(): bool {
        features::concurrent_fungible_assets_enabled()
    }

    inline fun allow_upgrade_to_concurrent_fungible_balance(): bool {
        features::concurrent_fungible_balance_enabled()
    }

    inline fun default_to_concurrent_fungible_balance(): bool {
        features::default_to_concurrent_fungible_balance_enabled()
    }

    /// Make an existing object fungible by adding the Metadata resource.
    /// This returns the capabilities to mint, burn, and transfer.
    /// maximum_supply defines the behavior of maximum supply when monitoring:
    ///   - option::none(): Monitoring unlimited supply
    ///     (width of the field - MAX_U128 is the implicit maximum supply)
    ///     if option::some(MAX_U128) is used, it is treated as unlimited supply.
    ///   - option::some(max): Monitoring fixed supply with `max` as the maximum supply.
    public fun add_fungibility(
        constructor_ref: &ConstructorRef,
        maximum_supply: Option<u128>,
        name: String,
        symbol: String,
        decimals: u8,
        icon_uri: String,
        project_uri: String,
    ): Object<Metadata> {
        assert!(!object::can_generate_delete_ref(constructor_ref), error::invalid_argument(EOBJECT_IS_DELETABLE));
        let metadata_object_signer = &object::generate_signer(constructor_ref);
        assert!(string::length(&name) <= MAX_NAME_LENGTH, error::out_of_range(ENAME_TOO_LONG));
        assert!(string::length(&symbol) <= MAX_SYMBOL_LENGTH, error::out_of_range(ESYMBOL_TOO_LONG));
        assert!(decimals <= MAX_DECIMALS, error::out_of_range(EDECIMALS_TOO_LARGE));
        assert!(string::length(&icon_uri) <= MAX_URI_LENGTH, error::out_of_range(EURI_TOO_LONG));
        assert!(string::length(&project_uri) <= MAX_URI_LENGTH, error::out_of_range(EURI_TOO_LONG));
        move_to(metadata_object_signer,
            Metadata {
                name,
                symbol,
                decimals,
                icon_uri,
                project_uri,
            }
        );

        if (default_to_concurrent_fungible_supply()) {
            let unlimited = option::is_none(&maximum_supply);
            move_to(metadata_object_signer, ConcurrentSupply {
                current: if (unlimited) {
                    aggregator_v2::create_unbounded_aggregator()
                } else {
                    aggregator_v2::create_aggregator(option::extract(&mut maximum_supply))
                },
            });
        } else {
            move_to(metadata_object_signer, Supply {
                current: 0,
                maximum: maximum_supply
            });
        };

        object::object_from_constructor_ref<Metadata>(constructor_ref)
    }

    /// Set that only untransferable stores can be created for this fungible asset.
    public fun set_untransferable(constructor_ref: &ConstructorRef) {
        let metadata_addr = object::address_from_constructor_ref(constructor_ref);
        assert!(exists<Metadata>(metadata_addr), error::not_found(EFUNGIBLE_METADATA_EXISTENCE));
        let metadata_signer = &object::generate_signer(constructor_ref);
        move_to(metadata_signer, Untransferable {});
    }


    #[view]
    /// Returns true if the FA is untransferable.
    public fun is_untransferable<T: key>(metadata: Object<T>): bool {
        exists<Untransferable>(object::object_address(&metadata))
    }

    /// Create a fungible asset store whose transfer rule would be overloaded by the provided function.
    public(friend) fun register_dispatch_functions(
        constructor_ref: &ConstructorRef,
        withdraw_function: Option<FunctionInfo>,
        deposit_function: Option<FunctionInfo>,
        derived_balance_function: Option<FunctionInfo>,
    ) {
        // Verify that caller type matches callee type so wrongly typed function cannot be registered.
        option::for_each_ref(&withdraw_function, |withdraw_function| {
            let dispatcher_withdraw_function_info = function_info::new_function_info_from_address(
                @aptos_framework,
                string::utf8(b"dispatchable_fungible_asset"),
                string::utf8(b"dispatchable_withdraw"),
            );

            assert!(
                function_info::check_dispatch_type_compatibility(
                    &dispatcher_withdraw_function_info,
                    withdraw_function
                ),
                error::invalid_argument(
                    EWITHDRAW_FUNCTION_SIGNATURE_MISMATCH
                )
            );
        });

        option::for_each_ref(&deposit_function, |deposit_function| {
            let dispatcher_deposit_function_info = function_info::new_function_info_from_address(
                @aptos_framework,
                string::utf8(b"dispatchable_fungible_asset"),
                string::utf8(b"dispatchable_deposit"),
            );
            // Verify that caller type matches callee type so wrongly typed function cannot be registered.
            assert!(
                function_info::check_dispatch_type_compatibility(
                    &dispatcher_deposit_function_info,
                    deposit_function
                ),
                error::invalid_argument(
                    EDEPOSIT_FUNCTION_SIGNATURE_MISMATCH
                )
            );
        });

        option::for_each_ref(&derived_balance_function, |balance_function| {
            let dispatcher_derived_balance_function_info = function_info::new_function_info_from_address(
                @aptos_framework,
                string::utf8(b"dispatchable_fungible_asset"),
                string::utf8(b"dispatchable_derived_balance"),
            );
            // Verify that caller type matches callee type so wrongly typed function cannot be registered.
            assert!(
                function_info::check_dispatch_type_compatibility(
                    &dispatcher_derived_balance_function_info,
                    balance_function
                ),
                error::invalid_argument(
                    EDERIVED_BALANCE_FUNCTION_SIGNATURE_MISMATCH
                )
            );
        });
        register_dispatch_function_sanity_check(constructor_ref);
        assert!(
            !exists<DispatchFunctionStore>(
                object::address_from_constructor_ref(constructor_ref)
            ),
            error::already_exists(EALREADY_REGISTERED)
        );

        let store_obj = &object::generate_signer(constructor_ref);

        // Store the overload function hook.
        move_to<DispatchFunctionStore>(
            store_obj,
            DispatchFunctionStore {
                withdraw_function,
                deposit_function,
                derived_balance_function,
            }
        );
    }

    /// Define the derived supply dispatch with the provided function.
    public(friend) fun register_derive_supply_dispatch_function(
        constructor_ref: &ConstructorRef,
        dispatch_function: Option<FunctionInfo>
    ) {
        // Verify that caller type matches callee type so wrongly typed function cannot be registered.
        option::for_each_ref(&dispatch_function, |supply_function| {
            let function_info = function_info::new_function_info_from_address(
                @aptos_framework,
                string::utf8(b"dispatchable_fungible_asset"),
                string::utf8(b"dispatchable_derived_supply"),
            );
            // Verify that caller type matches callee type so wrongly typed function cannot be registered.
            assert!(
                function_info::check_dispatch_type_compatibility(
                    &function_info,
                    supply_function
                ),
                error::invalid_argument(
                    EDERIVED_SUPPLY_FUNCTION_SIGNATURE_MISMATCH
                )
            );
        });
        register_dispatch_function_sanity_check(constructor_ref);
        assert!(
            !exists<DeriveSupply>(
                object::address_from_constructor_ref(constructor_ref)
            ),
            error::already_exists(EALREADY_REGISTERED)
        );


        let store_obj = &object::generate_signer(constructor_ref);

        // Store the overload function hook.
        move_to<DeriveSupply>(
            store_obj,
            DeriveSupply {
                dispatch_function
            }
        );
    }

    /// Check the requirements for registering a dispatchable function.
    inline fun register_dispatch_function_sanity_check(
        constructor_ref: &ConstructorRef,
    )  {
        // Cannot register hook for APT.
        assert!(
            object::address_from_constructor_ref(constructor_ref) != @aptos_fungible_asset,
            error::permission_denied(EAPT_NOT_DISPATCHABLE)
        );
        assert!(
            !object::can_generate_delete_ref(constructor_ref),
            error::invalid_argument(EOBJECT_IS_DELETABLE)
        );
        assert!(
            exists<Metadata>(
                object::address_from_constructor_ref(constructor_ref)
            ),
            error::not_found(EFUNGIBLE_METADATA_EXISTENCE),
        );
    }

    /// Creates a mint ref that can be used to mint fungible assets from the given fungible object's constructor ref.
    /// This can only be called at object creation time as constructor_ref is only available then.
    public fun generate_mint_ref(constructor_ref: &ConstructorRef): MintRef {
        let metadata = object::object_from_constructor_ref<Metadata>(constructor_ref);
        MintRef { metadata }
    }

    /// Creates a burn ref that can be used to burn fungible assets from the given fungible object's constructor ref.
    /// This can only be called at object creation time as constructor_ref is only available then.
    public fun generate_burn_ref(constructor_ref: &ConstructorRef): BurnRef {
        let metadata = object::object_from_constructor_ref<Metadata>(constructor_ref);
        BurnRef { metadata }
    }

    /// Creates a transfer ref that can be used to freeze/unfreeze/transfer fungible assets from the given fungible
    /// object's constructor ref.
    /// This can only be called at object creation time as constructor_ref is only available then.
    public fun generate_transfer_ref(constructor_ref: &ConstructorRef): TransferRef {
        let metadata = object::object_from_constructor_ref<Metadata>(constructor_ref);
        TransferRef { metadata }
    }

    /// Creates a mutate metadata ref that can be used to change the metadata information of fungible assets from the
    /// given fungible object's constructor ref.
    /// This can only be called at object creation time as constructor_ref is only available then.
    public fun generate_mutate_metadata_ref(constructor_ref: &ConstructorRef): MutateMetadataRef {
        let metadata = object::object_from_constructor_ref<Metadata>(constructor_ref);
        MutateMetadataRef { metadata }
    }

    #[view]
    /// Get the current supply from the `metadata` object.
    public fun supply<T: key>(metadata: Object<T>): Option<u128> acquires Supply, ConcurrentSupply {
        let metadata_address = object::object_address(&metadata);
        if (exists<ConcurrentSupply>(metadata_address)) {
            let supply = borrow_global<ConcurrentSupply>(metadata_address);
            option::some(aggregator_v2::read(&supply.current))
        } else if (exists<Supply>(metadata_address)) {
            let supply = borrow_global<Supply>(metadata_address);
            option::some(supply.current)
        } else {
            option::none()
        }
    }

    #[view]
    /// Get the maximum supply from the `metadata` object.
    /// If supply is unlimited (or set explicitly to MAX_U128), none is returned
    public fun maximum<T: key>(metadata: Object<T>): Option<u128> acquires Supply, ConcurrentSupply {
        let metadata_address = object::object_address(&metadata);
        if (exists<ConcurrentSupply>(metadata_address)) {
            let supply = borrow_global<ConcurrentSupply>(metadata_address);
            let max_value = aggregator_v2::max_value(&supply.current);
            if (max_value == MAX_U128) {
                option::none()
            } else {
                option::some(max_value)
            }
        } else if (exists<Supply>(metadata_address)) {
            let supply = borrow_global<Supply>(metadata_address);
            supply.maximum
        } else {
            option::none()
        }
    }

    #[view]
    /// Get the name of the fungible asset from the `metadata` object.
    public fun name<T: key>(metadata: Object<T>): String acquires Metadata {
        borrow_fungible_metadata(&metadata).name
    }

    #[view]
    /// Get the symbol of the fungible asset from the `metadata` object.
    public fun symbol<T: key>(metadata: Object<T>): String acquires Metadata {
        borrow_fungible_metadata(&metadata).symbol
    }

    #[view]
    /// Get the decimals from the `metadata` object.
    public fun decimals<T: key>(metadata: Object<T>): u8 acquires Metadata {
        borrow_fungible_metadata(&metadata).decimals
    }

    #[view]
    /// Get the icon uri from the `metadata` object.
    public fun icon_uri<T: key>(metadata: Object<T>): String acquires Metadata {
        borrow_fungible_metadata(&metadata).icon_uri
    }

    #[view]
    /// Get the project uri from the `metadata` object.
    public fun project_uri<T: key>(metadata: Object<T>): String acquires Metadata {
        borrow_fungible_metadata(&metadata).project_uri
    }

    #[view]
    /// Get the metadata struct from the `metadata` object.
    public fun metadata<T: key>(metadata: Object<T>): Metadata acquires Metadata {
        *borrow_fungible_metadata(&metadata)
    }

    #[view]
    /// Return whether the provided address has a store initialized.
    public fun store_exists(store: address): bool {
        store_exists_inline(store)
    }

    /// Return whether the provided address has a store initialized.
    inline fun store_exists_inline(store: address): bool {
        exists<FungibleStore>(store)
    }

    /// Return whether the provided address has a concurrent fungible balance initialized,
    /// at the fungible store address.
    inline fun concurrent_fungible_balance_exists_inline(store: address): bool {
        exists<ConcurrentFungibleBalance>(store)
    }

    /// Return the underlying metadata object
    public fun metadata_from_asset(fa: &FungibleAsset): Object<Metadata> {
        fa.metadata
    }

    #[view]
    /// Return the underlying metadata object.
    public fun store_metadata<T: key>(store: Object<T>): Object<Metadata> acquires FungibleStore {
        borrow_store_resource(&store).metadata
    }

    /// Return the `amount` of a given fungible asset.
    public fun amount(fa: &FungibleAsset): u64 {
        fa.amount
    }

    #[view]
    /// Get the balance of a given store.
    public fun balance<T: key>(store: Object<T>): u64 acquires FungibleStore, ConcurrentFungibleBalance {
        let store_addr = object::object_address(&store);
        if (store_exists_inline(store_addr)) {
            let store_balance = borrow_store_resource(&store).balance;
            if (store_balance == 0 && concurrent_fungible_balance_exists_inline(store_addr)) {
                let balance_resource = borrow_global<ConcurrentFungibleBalance>(store_addr);
                aggregator_v2::read(&balance_resource.balance)
            } else {
                store_balance
            }
        } else {
            0
        }
    }

    #[view]
    /// Check whether the balance of a given store is >= `amount`.
    public fun is_balance_at_least<T: key>(store: Object<T>, amount: u64): bool acquires FungibleStore, ConcurrentFungibleBalance {
        let store_addr = object::object_address(&store);
        is_address_balance_at_least(store_addr, amount)
    }

    /// Check whether the balance of a given store is >= `amount`.
    public(friend) fun is_address_balance_at_least(store_addr: address, amount: u64): bool acquires FungibleStore, ConcurrentFungibleBalance {
        if (store_exists_inline(store_addr)) {
            let store_balance = borrow_global<FungibleStore>(store_addr).balance;
            if (store_balance == 0 && concurrent_fungible_balance_exists_inline(store_addr)) {
                let balance_resource = borrow_global<ConcurrentFungibleBalance>(store_addr);
                aggregator_v2::is_at_least(&balance_resource.balance, amount)
            } else {
                store_balance >= amount
            }
        } else {
            amount == 0
        }
    }

    #[view]
    /// Return whether a store is frozen.
    ///
    /// If the store has not been created, we default to returning false so deposits can be sent to it.
    public fun is_frozen<T: key>(store: Object<T>): bool acquires FungibleStore {
        let store_addr = object::object_address(&store);
        store_exists_inline(store_addr) && borrow_global<FungibleStore>(store_addr).frozen
    }

    #[view]
    /// Return whether a fungible asset type is dispatchable.
    public fun is_store_dispatchable<T: key>(store: Object<T>): bool acquires FungibleStore {
        let fa_store = borrow_store_resource(&store);
        let metadata_addr = object::object_address(&fa_store.metadata);
        exists<DispatchFunctionStore>(metadata_addr)
    }

    public fun deposit_dispatch_function<T: key>(store: Object<T>): Option<FunctionInfo> acquires FungibleStore, DispatchFunctionStore {
        let fa_store = borrow_store_resource(&store);
        let metadata_addr = object::object_address(&fa_store.metadata);
        if(exists<DispatchFunctionStore>(metadata_addr)) {
            borrow_global<DispatchFunctionStore>(metadata_addr).deposit_function
        } else {
            option::none()
        }
    }

    fun has_deposit_dispatch_function(metadata: Object<Metadata>): bool acquires DispatchFunctionStore {
        let metadata_addr = object::object_address(&metadata);
        // Short circuit on APT for better perf
        if(metadata_addr != @aptos_fungible_asset && exists<DispatchFunctionStore>(metadata_addr)) {
            option::is_some(&borrow_global<DispatchFunctionStore>(metadata_addr).deposit_function)
        } else {
            false
        }
    }

    public fun withdraw_dispatch_function<T: key>(store: Object<T>): Option<FunctionInfo> acquires FungibleStore, DispatchFunctionStore {
        let fa_store = borrow_store_resource(&store);
        let metadata_addr = object::object_address(&fa_store.metadata);
        if(exists<DispatchFunctionStore>(metadata_addr)) {
            borrow_global<DispatchFunctionStore>(metadata_addr).withdraw_function
        } else {
            option::none()
        }
    }

    fun has_withdraw_dispatch_function(metadata: Object<Metadata>): bool acquires DispatchFunctionStore {
        let metadata_addr = object::object_address(&metadata);
        // Short circuit on APT for better perf
        if (metadata_addr != @aptos_fungible_asset && exists<DispatchFunctionStore>(metadata_addr)) {
            option::is_some(&borrow_global<DispatchFunctionStore>(metadata_addr).withdraw_function)
        } else {
            false
        }
    }

    public(friend) fun derived_balance_dispatch_function<T: key>(store: Object<T>): Option<FunctionInfo> acquires FungibleStore, DispatchFunctionStore {
        let fa_store = borrow_store_resource(&store);
        let metadata_addr = object::object_address(&fa_store.metadata);
        if (exists<DispatchFunctionStore>(metadata_addr)) {
            borrow_global<DispatchFunctionStore>(metadata_addr).derived_balance_function
        } else {
            option::none()
        }
    }

    public(friend) fun derived_supply_dispatch_function<T: key>(metadata: Object<T>): Option<FunctionInfo> acquires DeriveSupply {
        let metadata_addr = object::object_address(&metadata);
        if (exists<DeriveSupply>(metadata_addr)) {
            borrow_global<DeriveSupply>(metadata_addr).dispatch_function
        } else {
            option::none()
        }
    }

    public fun asset_metadata(fa: &FungibleAsset): Object<Metadata> {
        fa.metadata
    }

    /// Get the underlying metadata object from the `MintRef`.
    public fun mint_ref_metadata(ref: &MintRef): Object<Metadata> {
        ref.metadata
    }

    /// Get the underlying metadata object from the `TransferRef`.
    public fun transfer_ref_metadata(ref: &TransferRef): Object<Metadata> {
        ref.metadata
    }

    /// Get the underlying metadata object from the `BurnRef`.
    public fun burn_ref_metadata(ref: &BurnRef): Object<Metadata> {
        ref.metadata
    }

    /// Get the underlying metadata object from the `MutateMetadataRef`.
    public fun object_from_metadata_ref(ref: &MutateMetadataRef): Object<Metadata> {
        ref.metadata
    }

    /// Transfer an `amount` of fungible asset from `from_store`, which should be owned by `sender`, to `receiver`.
    /// Note: it does not move the underlying object.
    public entry fun transfer<T: key>(
        sender: &signer,
        from: Object<T>,
        to: Object<T>,
        amount: u64,
    ) acquires FungibleStore, DispatchFunctionStore, ConcurrentFungibleBalance {
        let fa = withdraw(sender, from, amount);
        deposit(to, fa);
    }

    /// Allow an object to hold a store for fungible assets.
    /// Applications can use this to create multiple stores for isolating fungible assets for different purposes.
    public fun create_store<T: key>(
        constructor_ref: &ConstructorRef,
        metadata: Object<T>,
    ): Object<FungibleStore> {
        let store_obj = &object::generate_signer(constructor_ref);
        move_to(store_obj, FungibleStore {
            metadata: object::convert(metadata),
            balance: 0,
            frozen: false,
        });

        if (is_untransferable(metadata)) {
            object::set_untransferable(constructor_ref);
        };

        if (default_to_concurrent_fungible_balance()) {
            move_to(store_obj, ConcurrentFungibleBalance {
                balance: aggregator_v2::create_unbounded_aggregator(),
            });
        };

        object::object_from_constructor_ref<FungibleStore>(constructor_ref)
    }

    /// Used to delete a store.  Requires the store to be completely empty prior to removing it
    public fun remove_store(delete_ref: &DeleteRef) acquires FungibleStore, FungibleAssetEvents, ConcurrentFungibleBalance {
        let store = &object::object_from_delete_ref<FungibleStore>(delete_ref);
        let addr = object::object_address(store);
        let FungibleStore { metadata: _, balance, frozen: _ }
            = move_from<FungibleStore>(addr);
        assert!(balance == 0, error::permission_denied(EBALANCE_IS_NOT_ZERO));

        if (concurrent_fungible_balance_exists_inline(addr)) {
            let ConcurrentFungibleBalance { balance } = move_from<ConcurrentFungibleBalance>(addr);
            assert!(aggregator_v2::read(&balance) == 0, error::permission_denied(EBALANCE_IS_NOT_ZERO));
        };

        // Cleanup deprecated event handles if exist.
        if (exists<FungibleAssetEvents>(addr)) {
            let FungibleAssetEvents {
                deposit_events,
                withdraw_events,
                frozen_events,
            } = move_from<FungibleAssetEvents>(addr);
            event::destroy_handle(deposit_events);
            event::destroy_handle(withdraw_events);
            event::destroy_handle(frozen_events);
        };
    }

    /// Withdraw `amount` of the fungible asset from `store` by the owner.
    public fun withdraw<T: key>(
        owner: &signer,
        store: Object<T>,
        amount: u64,
    ): FungibleAsset acquires FungibleStore, DispatchFunctionStore, ConcurrentFungibleBalance {
        withdraw_sanity_check(owner, store, true);
        withdraw_internal(object::object_address(&store), amount)
    }

    /// Check the permission for withdraw operation.
    public(friend) fun withdraw_sanity_check<T: key>(
        owner: &signer,
        store: Object<T>,
        abort_on_dispatch: bool,
    ) acquires FungibleStore, DispatchFunctionStore {
        assert!(object::owns(store, signer::address_of(owner)), error::permission_denied(ENOT_STORE_OWNER));
        let fa_store = borrow_store_resource(&store);
        assert!(
            !abort_on_dispatch || !has_withdraw_dispatch_function(fa_store.metadata),
            error::invalid_argument(EINVALID_DISPATCHABLE_OPERATIONS)
        );
        assert!(!fa_store.frozen, error::permission_denied(ESTORE_IS_FROZEN));
    }

    /// Deposit `amount` of the fungible asset to `store`.
    public fun deposit_sanity_check<T: key>(
        store: Object<T>,
        abort_on_dispatch: bool
    ) acquires FungibleStore, DispatchFunctionStore {
        let fa_store = borrow_store_resource(&store);
        assert!(
            !abort_on_dispatch || !has_deposit_dispatch_function(fa_store.metadata),
            error::invalid_argument(EINVALID_DISPATCHABLE_OPERATIONS)
        );
        assert!(!fa_store.frozen, error::permission_denied(ESTORE_IS_FROZEN));
    }

    /// Deposit `amount` of the fungible asset to `store`.
    public fun deposit<T: key>(store: Object<T>, fa: FungibleAsset) acquires FungibleStore, DispatchFunctionStore, ConcurrentFungibleBalance {
        deposit_sanity_check(store, true);
        deposit_internal(object::object_address(&store), fa);
    }

    /// Mint the specified `amount` of the fungible asset.
    public fun mint(ref: &MintRef, amount: u64): FungibleAsset acquires Supply, ConcurrentSupply {
        let metadata = ref.metadata;
        mint_internal(metadata, amount)
    }

    /// CAN ONLY BE CALLED BY coin.move for migration.
    public(friend) fun mint_internal(
        metadata: Object<Metadata>,
        amount: u64
    ): FungibleAsset acquires Supply, ConcurrentSupply {
        increase_supply(&metadata, amount);
        FungibleAsset {
            metadata,
            amount
        }
    }

    /// Mint the specified `amount` of the fungible asset to a destination store.
    public fun mint_to<T: key>(ref: &MintRef, store: Object<T>, amount: u64)
    acquires FungibleStore, Supply, ConcurrentSupply, DispatchFunctionStore, ConcurrentFungibleBalance {
        deposit_sanity_check(store, false);
        deposit_internal(object::object_address(&store), mint(ref, amount));
    }

    /// Enable/disable a store's ability to do direct transfers of the fungible asset.
    public fun set_frozen_flag<T: key>(
        ref: &TransferRef,
        store: Object<T>,
        frozen: bool,
    ) acquires FungibleStore {
        assert!(
            ref.metadata == store_metadata(store),
            error::invalid_argument(ETRANSFER_REF_AND_STORE_MISMATCH),
        );
        set_frozen_flag_internal(store, frozen)
    }

    public(friend) fun set_frozen_flag_internal<T: key>(
        store: Object<T>,
        frozen: bool
    ) acquires FungibleStore {
        let store_addr = object::object_address(&store);
        borrow_global_mut<FungibleStore>(store_addr).frozen = frozen;

        event::emit(Frozen { store: store_addr, frozen });
    }

    /// Burns a fungible asset
    public fun burn(ref: &BurnRef, fa: FungibleAsset) acquires Supply, ConcurrentSupply {
        assert!(
            ref.metadata == metadata_from_asset(&fa),
            error::invalid_argument(EBURN_REF_AND_FUNGIBLE_ASSET_MISMATCH)
        );
        burn_internal(fa);
    }

    /// CAN ONLY BE CALLED BY coin.move for migration.
    public(friend) fun burn_internal(
        fa: FungibleAsset
    ): u64 acquires Supply, ConcurrentSupply {
        let FungibleAsset {
            metadata,
            amount
        } = fa;
        decrease_supply(&metadata, amount);
        amount
    }

    /// Burn the `amount` of the fungible asset from the given store.
    public fun burn_from<T: key>(
        ref: &BurnRef,
        store: Object<T>,
        amount: u64
    ) acquires FungibleStore, Supply, ConcurrentSupply, ConcurrentFungibleBalance {
        // ref metadata match is checked in burn() call
        burn(ref, withdraw_internal(object::object_address(&store), amount));
    }

    public(friend) fun address_burn_from(
        ref: &BurnRef,
        store_addr: address,
        amount: u64
    ) acquires FungibleStore, Supply, ConcurrentSupply, ConcurrentFungibleBalance {
        // ref metadata match is checked in burn() call
        burn(ref, withdraw_internal(store_addr, amount));
    }

    /// Withdraw `amount` of the fungible asset from the `store` ignoring `frozen`.
    public fun withdraw_with_ref<T: key>(
        ref: &TransferRef,
        store: Object<T>,
        amount: u64
    ): FungibleAsset acquires FungibleStore, ConcurrentFungibleBalance {
        assert!(
            ref.metadata == store_metadata(store),
            error::invalid_argument(ETRANSFER_REF_AND_STORE_MISMATCH),
        );
        withdraw_internal(object::object_address(&store), amount)
    }

    /// Deposit the fungible asset into the `store` ignoring `frozen`.
    public fun deposit_with_ref<T: key>(
        ref: &TransferRef,
        store: Object<T>,
        fa: FungibleAsset
    ) acquires FungibleStore, ConcurrentFungibleBalance {
        assert!(
            ref.metadata == fa.metadata,
            error::invalid_argument(ETRANSFER_REF_AND_FUNGIBLE_ASSET_MISMATCH)
        );
        deposit_internal(object::object_address(&store), fa);
    }

    /// Transfer `amount` of the fungible asset with `TransferRef` even it is frozen.
    public fun transfer_with_ref<T: key>(
        transfer_ref: &TransferRef,
        from: Object<T>,
        to: Object<T>,
        amount: u64,
    ) acquires FungibleStore, ConcurrentFungibleBalance {
        let fa = withdraw_with_ref(transfer_ref, from, amount);
        deposit_with_ref(transfer_ref, to, fa);
    }

    /// Mutate specified fields of the fungible asset's `Metadata`.
    public fun mutate_metadata(
        metadata_ref: &MutateMetadataRef,
        name: Option<String>,
        symbol: Option<String>,
        decimals: Option<u8>,
        icon_uri: Option<String>,
        project_uri: Option<String>,
    ) acquires Metadata {
        let metadata_address = object::object_address(&metadata_ref.metadata);
        let mutable_metadata = borrow_global_mut<Metadata>(metadata_address);

        if (option::is_some(&name)){
            mutable_metadata.name = option::extract(&mut name);
        };
        if (option::is_some(&symbol)){
            mutable_metadata.symbol = option::extract(&mut symbol);
        };
        if (option::is_some(&decimals)){
            mutable_metadata.decimals = option::extract(&mut decimals);
        };
        if (option::is_some(&icon_uri)){
            mutable_metadata.icon_uri = option::extract(&mut icon_uri);
        };
        if (option::is_some(&project_uri)){
            mutable_metadata.project_uri = option::extract(&mut project_uri);
        };
    }

    /// Create a fungible asset with zero amount.
    /// This can be useful when starting a series of computations where the initial value is 0.
    public fun zero<T: key>(metadata: Object<T>): FungibleAsset {
        FungibleAsset {
            metadata: object::convert(metadata),
            amount: 0,
        }
    }

    /// Extract a given amount from the given fungible asset and return a new one.
    public fun extract(fungible_asset: &mut FungibleAsset, amount: u64): FungibleAsset {
        assert!(fungible_asset.amount >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));
        fungible_asset.amount = fungible_asset.amount - amount;
        FungibleAsset {
            metadata: fungible_asset.metadata,
            amount,
        }
    }

    /// "Merges" the two given fungible assets. The fungible asset passed in as `dst_fungible_asset` will have a value
    /// equal to the sum of the two (`dst_fungible_asset` and `src_fungible_asset`).
    public fun merge(dst_fungible_asset: &mut FungibleAsset, src_fungible_asset: FungibleAsset) {
        let FungibleAsset { metadata, amount } = src_fungible_asset;
        assert!(metadata == dst_fungible_asset.metadata, error::invalid_argument(EFUNGIBLE_ASSET_MISMATCH));
        dst_fungible_asset.amount = dst_fungible_asset.amount + amount;
    }

    /// Destroy an empty fungible asset.
    public fun destroy_zero(fungible_asset: FungibleAsset) {
        let FungibleAsset { amount, metadata: _ } = fungible_asset;
        assert!(amount == 0, error::invalid_argument(EAMOUNT_IS_NOT_ZERO));
    }

    public(friend) fun deposit_internal(store_addr: address, fa: FungibleAsset) acquires FungibleStore, ConcurrentFungibleBalance {
        let FungibleAsset { metadata, amount } = fa;
        assert!(exists<FungibleStore>(store_addr), error::not_found(EFUNGIBLE_STORE_EXISTENCE));
        let store = borrow_global_mut<FungibleStore>(store_addr);
        assert!(metadata == store.metadata, error::invalid_argument(EFUNGIBLE_ASSET_AND_STORE_MISMATCH));

        if (amount == 0) return;

        if (store.balance == 0 && concurrent_fungible_balance_exists_inline(store_addr)) {
            let balance_resource = borrow_global_mut<ConcurrentFungibleBalance>(store_addr);
            aggregator_v2::add(&mut balance_resource.balance, amount);
        } else {
            store.balance = store.balance + amount;
        };

        event::emit(Deposit { store: store_addr, amount });
    }

    /// Extract `amount` of the fungible asset from `store`.
    public(friend) fun withdraw_internal(
        store_addr: address,
        amount: u64,
    ): FungibleAsset acquires FungibleStore, ConcurrentFungibleBalance {
        assert!(exists<FungibleStore>(store_addr), error::not_found(EFUNGIBLE_STORE_EXISTENCE));

        let store = borrow_global_mut<FungibleStore>(store_addr);
        let metadata = store.metadata;
        if (amount != 0) {
            if (store.balance == 0 && concurrent_fungible_balance_exists_inline(store_addr)) {
                let balance_resource = borrow_global_mut<ConcurrentFungibleBalance>(store_addr);
                assert!(
                    aggregator_v2::try_sub(&mut balance_resource.balance, amount),
                    error::invalid_argument(EINSUFFICIENT_BALANCE)
                );
            } else {
                assert!(store.balance >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));
                store.balance = store.balance - amount;
            };

            event::emit<Withdraw>(Withdraw { store: store_addr, amount });
        };
        FungibleAsset { metadata, amount }
    }

    /// Increase the supply of a fungible asset by minting.
    fun increase_supply<T: key>(metadata: &Object<T>, amount: u64) acquires Supply, ConcurrentSupply {
        if (amount == 0) {
            return
        };
        let metadata_address = object::object_address(metadata);

        if (exists<ConcurrentSupply>(metadata_address)) {
            let supply = borrow_global_mut<ConcurrentSupply>(metadata_address);
            assert!(
                aggregator_v2::try_add(&mut supply.current, (amount as u128)),
                error::out_of_range(EMAX_SUPPLY_EXCEEDED)
            );
        } else if (exists<Supply>(metadata_address)) {
            let supply = borrow_global_mut<Supply>(metadata_address);
            if (option::is_some(&supply.maximum)) {
                let max = *option::borrow_mut(&mut supply.maximum);
                assert!(
                    max - supply.current >= (amount as u128),
                    error::out_of_range(EMAX_SUPPLY_EXCEEDED)
                )
            };
            supply.current = supply.current + (amount as u128);
        } else {
            abort error::not_found(ESUPPLY_NOT_FOUND)
        }
    }

    /// Decrease the supply of a fungible asset by burning.
    fun decrease_supply<T: key>(metadata: &Object<T>, amount: u64) acquires Supply, ConcurrentSupply {
        if (amount == 0) {
            return
        };
        let metadata_address = object::object_address(metadata);

        if (exists<ConcurrentSupply>(metadata_address)) {
            let supply = borrow_global_mut<ConcurrentSupply>(metadata_address);

            assert!(
                aggregator_v2::try_sub(&mut supply.current, (amount as u128)),
                error::out_of_range(ESUPPLY_UNDERFLOW)
            );
        } else if (exists<Supply>(metadata_address)) {
            assert!(exists<Supply>(metadata_address), error::not_found(ESUPPLY_NOT_FOUND));
            let supply = borrow_global_mut<Supply>(metadata_address);
            assert!(
                supply.current >= (amount as u128),
                error::invalid_state(ESUPPLY_UNDERFLOW)
            );
            supply.current = supply.current - (amount as u128);
        } else {
            assert!(false, error::not_found(ESUPPLY_NOT_FOUND));
        }
    }

    inline fun borrow_fungible_metadata<T: key>(
        metadata: &Object<T>
    ): &Metadata acquires Metadata {
        let addr = object::object_address(metadata);
        borrow_global<Metadata>(addr)
    }

    inline fun borrow_fungible_metadata_mut<T: key>(
        metadata: &Object<T>
    ): &mut Metadata acquires Metadata {
        let addr = object::object_address(metadata);
        borrow_global_mut<Metadata>(addr)
    }

    inline fun borrow_store_resource<T: key>(store: &Object<T>): &FungibleStore acquires FungibleStore {
        let store_addr = object::object_address(store);
        assert!(exists<FungibleStore>(store_addr), error::not_found(EFUNGIBLE_STORE_EXISTENCE));
        borrow_global<FungibleStore>(store_addr)
    }

    public fun upgrade_to_concurrent(
        ref: &ExtendRef,
    ) acquires Supply {
        let metadata_object_address = object::address_from_extend_ref(ref);
        let metadata_object_signer = object::generate_signer_for_extending(ref);
        assert!(
            features::concurrent_fungible_assets_enabled(),
            error::invalid_argument(ECONCURRENT_SUPPLY_NOT_ENABLED)
        );
        assert!(exists<Supply>(metadata_object_address), error::not_found(ESUPPLY_NOT_FOUND));
        let Supply {
            current,
            maximum,
        } = move_from<Supply>(metadata_object_address);

        let unlimited = option::is_none(&maximum);
        let supply = ConcurrentSupply {
            current: if (unlimited) {
                aggregator_v2::create_unbounded_aggregator_with_value(current)
            }
            else {
                aggregator_v2::create_aggregator_with_value(current, option::extract(&mut maximum))
            },
        };
        move_to(&metadata_object_signer, supply);
    }

    public entry fun upgrade_store_to_concurrent<T: key>(
        owner: &signer,
        store: Object<T>,
    ) acquires FungibleStore {
        assert!(object::owns(store, signer::address_of(owner)), error::permission_denied(ENOT_STORE_OWNER));
        assert!(!is_frozen(store), error::invalid_argument(ESTORE_IS_FROZEN));
        assert!(allow_upgrade_to_concurrent_fungible_balance(), error::invalid_argument(ECONCURRENT_BALANCE_NOT_ENABLED));
        ensure_store_upgraded_to_concurrent_internal(object::object_address(&store));
    }

    /// Ensure a known `FungibleStore` has `ConcurrentFungibleBalance`.
    fun ensure_store_upgraded_to_concurrent_internal(
        fungible_store_address: address,
    ) acquires FungibleStore {
        if (exists<ConcurrentFungibleBalance>(fungible_store_address)) {
            return
        };
        let store = borrow_global_mut<FungibleStore>(fungible_store_address);
        let balance = aggregator_v2::create_unbounded_aggregator_with_value(store.balance);
        store.balance = 0;
        let object_signer = create_signer::create_signer(fungible_store_address);
        move_to(&object_signer, ConcurrentFungibleBalance { balance });
    }

    #[test_only]
    use aptos_framework::account;

    #[test_only]
    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]

    struct TestToken has key {}

    #[test_only]
    public fun create_test_token(creator: &signer): (ConstructorRef, Object<TestToken>) {
        account::create_account_for_test(signer::address_of(creator));
        let creator_ref = object::create_named_object(creator, b"TEST");
        let object_signer = object::generate_signer(&creator_ref);
        move_to(&object_signer, TestToken {});

        let token = object::object_from_constructor_ref<TestToken>(&creator_ref);
        (creator_ref, token)
    }

    #[test_only]
    public fun init_test_metadata(constructor_ref: &ConstructorRef): (MintRef, TransferRef, BurnRef, MutateMetadataRef) {
        add_fungibility(
            constructor_ref,
            option::some(100) /* max supply */,
            string::utf8(b"TEST"),
            string::utf8(b"@@"),
            0,
            string::utf8(b"http://www.example.com/favicon.ico"),
            string::utf8(b"http://www.example.com"),
        );
        let mint_ref = generate_mint_ref(constructor_ref);
        let burn_ref = generate_burn_ref(constructor_ref);
        let transfer_ref = generate_transfer_ref(constructor_ref);
        let mutate_metadata_ref= generate_mutate_metadata_ref(constructor_ref);
        (mint_ref, transfer_ref, burn_ref, mutate_metadata_ref)
    }

    #[test_only]
    public fun create_fungible_asset(
        creator: &signer
    ): (MintRef, TransferRef, BurnRef, MutateMetadataRef, Object<Metadata>) {
        let (creator_ref, token_object) = create_test_token(creator);
        let (mint, transfer, burn, mutate_metadata) = init_test_metadata(&creator_ref);
        (mint, transfer, burn, mutate_metadata, object::convert(token_object))
    }

    #[test_only]
    public fun create_test_store<T: key>(owner: &signer, metadata: Object<T>): Object<FungibleStore> {
        let owner_addr = signer::address_of(owner);
        if (!account::exists_at(owner_addr)) {
            account::create_account_for_test(owner_addr);
        };
        create_store(&object::create_object_from_account(owner), metadata)
    }

    #[test(creator = @0xcafe)]
    fun test_metadata_basic_flow(creator: &signer) acquires Metadata, Supply, ConcurrentSupply {
        let (creator_ref, metadata) = create_test_token(creator);
        init_test_metadata(&creator_ref);
        assert!(supply(metadata) == option::some(0), 1);
        assert!(maximum(metadata) == option::some(100), 2);
        assert!(name(metadata) == string::utf8(b"TEST"), 3);
        assert!(symbol(metadata) == string::utf8(b"@@"), 4);
        assert!(decimals(metadata) == 0, 5);
        assert!(icon_uri(metadata) == string::utf8(b"http://www.example.com/favicon.ico"), 6);
        assert!(project_uri(metadata) == string::utf8(b"http://www.example.com"), 7);

        assert!(metadata(metadata) == Metadata {
            name: string::utf8(b"TEST"),
            symbol: string::utf8(b"@@"),
            decimals: 0,
            icon_uri: string::utf8(b"http://www.example.com/favicon.ico"),
            project_uri: string::utf8(b"http://www.example.com"),
        }, 8);

        increase_supply(&metadata, 50);
        assert!(supply(metadata) == option::some(50), 9);
        decrease_supply(&metadata, 30);
        assert!(supply(metadata) == option::some(20), 10);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x20005, location = Self)]
    fun test_supply_overflow(creator: &signer) acquires Supply, ConcurrentSupply {
        let (creator_ref, metadata) = create_test_token(creator);
        init_test_metadata(&creator_ref);
        increase_supply(&metadata, 101);
    }

    #[test(creator = @0xcafe)]
    fun test_create_and_remove_store(creator: &signer) acquires FungibleStore, FungibleAssetEvents, ConcurrentFungibleBalance {
        let (_, _, _, _, metadata) = create_fungible_asset(creator);
        let creator_ref = object::create_object_from_account(creator);
        create_store(&creator_ref, metadata);
        let delete_ref = object::generate_delete_ref(&creator_ref);
        remove_store(&delete_ref);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_e2e_basic_flow(
        creator: &signer,
        aaron: &signer,
    ) acquires FungibleStore, Supply, ConcurrentSupply, DispatchFunctionStore, ConcurrentFungibleBalance, Metadata {
        let (mint_ref, transfer_ref, burn_ref, mutate_metadata_ref, test_token) = create_fungible_asset(creator);
        let metadata = mint_ref.metadata;
        let creator_store = create_test_store(creator, metadata);
        let aaron_store = create_test_store(aaron, metadata);

        assert!(supply(test_token) == option::some(0), 1);
        // Mint
        let fa = mint(&mint_ref, 100);
        assert!(supply(test_token) == option::some(100), 2);
        // Deposit
        deposit(creator_store, fa);
        // Withdraw
        let fa = withdraw(creator, creator_store, 80);
        assert!(supply(test_token) == option::some(100), 3);
        deposit(aaron_store, fa);
        // Burn
        burn_from(&burn_ref, aaron_store, 30);
        assert!(supply(test_token) == option::some(70), 4);
        // Transfer
        transfer(creator, creator_store, aaron_store, 10);
        assert!(balance(creator_store) == 10, 5);
        assert!(balance(aaron_store) == 60, 6);

        set_frozen_flag(&transfer_ref, aaron_store, true);
        assert!(is_frozen(aaron_store), 7);
        // Mutate Metadata
        mutate_metadata(
            &mutate_metadata_ref,
            option::some(string::utf8(b"mutated_name")),
            option::some(string::utf8(b"mutated_symbol")),
            option::none(),
            option::none(),
            option::none()
        );
        assert!(name(metadata) == string::utf8(b"mutated_name"), 8);
        assert!(symbol(metadata) == string::utf8(b"mutated_symbol"), 9);
        assert!(decimals(metadata) == 0, 10);
        assert!(icon_uri(metadata) == string::utf8(b"http://www.example.com/favicon.ico"), 11);
        assert!(project_uri(metadata) == string::utf8(b"http://www.example.com"), 12);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x50003, location = Self)]
    fun test_frozen(
        creator: &signer
    ) acquires FungibleStore, Supply, ConcurrentSupply, DispatchFunctionStore, ConcurrentFungibleBalance {
        let (mint_ref, transfer_ref, _burn_ref, _mutate_metadata_ref,  _) = create_fungible_asset(creator);

        let creator_store = create_test_store(creator, mint_ref.metadata);
        let fa = mint(&mint_ref, 100);
        set_frozen_flag(&transfer_ref, creator_store, true);
        deposit(creator_store, fa);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x50003, location = Self)]
    fun test_mint_to_frozen(
        creator: &signer
    ) acquires FungibleStore, ConcurrentFungibleBalance, Supply, ConcurrentSupply, DispatchFunctionStore {
        let (mint_ref, transfer_ref, _burn_ref, _mutate_metadata_ref, _) = create_fungible_asset(creator);

        let creator_store = create_test_store(creator, mint_ref.metadata);
        set_frozen_flag(&transfer_ref, creator_store, true);
        mint_to(&mint_ref, creator_store, 100);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::object)]
    fun test_untransferable(
        creator: &signer
    ) {
        let (creator_ref, _) = create_test_token(creator);
        let (mint_ref, _, _, _) = init_test_metadata(&creator_ref);
        set_untransferable(&creator_ref);

        let creator_store = create_test_store(creator, mint_ref.metadata);
        object::transfer(creator, creator_store, @0x456);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_transfer_with_ref(
        creator: &signer,
        aaron: &signer,
    ) acquires FungibleStore, Supply, ConcurrentSupply, ConcurrentFungibleBalance {
        let (mint_ref, transfer_ref, _burn_ref, _mutate_metadata_ref, _) = create_fungible_asset(creator);
        let metadata = mint_ref.metadata;
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
    fun test_mutate_metadata(
        creator: &signer
    ) acquires Metadata {
        let (mint_ref, _transfer_ref, _burn_ref, mutate_metadata_ref, _) = create_fungible_asset(creator);
        let metadata = mint_ref.metadata;

        mutate_metadata(
            &mutate_metadata_ref,
            option::some(string::utf8(b"mutated_name")),
            option::some(string::utf8(b"mutated_symbol")),
            option::some(10),
            option::some(string::utf8(b"http://www.mutated-example.com/favicon.ico")),
            option::some(string::utf8(b"http://www.mutated-example.com"))
        );
        assert!(name(metadata) == string::utf8(b"mutated_name"), 1);
        assert!(symbol(metadata) == string::utf8(b"mutated_symbol"), 2);
        assert!(decimals(metadata) == 10, 3);
        assert!(icon_uri(metadata) == string::utf8(b"http://www.mutated-example.com/favicon.ico"), 4);
        assert!(project_uri(metadata) == string::utf8(b"http://www.mutated-example.com"), 5);
    }

    #[test(creator = @0xcafe)]
    fun test_partial_mutate_metadata(
        creator: &signer
    ) acquires Metadata {
        let (mint_ref, _transfer_ref, _burn_ref, mutate_metadata_ref, _) = create_fungible_asset(creator);
        let metadata = mint_ref.metadata;

        mutate_metadata(
            &mutate_metadata_ref,
            option::some(string::utf8(b"mutated_name")),
            option::some(string::utf8(b"mutated_symbol")),
            option::none(),
            option::none(),
            option::none()
        );
        assert!(name(metadata) == string::utf8(b"mutated_name"), 8);
        assert!(symbol(metadata) == string::utf8(b"mutated_symbol"), 9);
        assert!(decimals(metadata) == 0, 10);
        assert!(icon_uri(metadata) == string::utf8(b"http://www.example.com/favicon.ico"), 11);
        assert!(project_uri(metadata) == string::utf8(b"http://www.example.com"), 12);
    }

    #[test(creator = @0xcafe)]
    fun test_merge_and_exact(creator: &signer) acquires Supply, ConcurrentSupply {
        let (mint_ref, _transfer_ref, burn_ref, _mutate_metadata_ref, _) = create_fungible_asset(creator);
        let fa = mint(&mint_ref, 100);
        let cash = extract(&mut fa, 80);
        assert!(fa.amount == 20, 1);
        assert!(cash.amount == 80, 2);
        let more_cash = extract(&mut fa, 20);
        destroy_zero(fa);
        merge(&mut cash, more_cash);
        assert!(cash.amount == 100, 3);
        burn(&burn_ref, cash);
    }

    #[test(creator = @0xcafe)]
    #[expected_failure(abort_code = 0x10012, location = Self)]
    fun test_add_fungibility_to_deletable_object(creator: &signer) {
        account::create_account_for_test(signer::address_of(creator));
        let creator_ref = &object::create_object_from_account(creator);
        init_test_metadata(creator_ref);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x10006, location = Self)]
    fun test_fungible_asset_mismatch_when_merge(creator: &signer, aaron: &signer) {
        let (_, _, _, _, metadata1) = create_fungible_asset(creator);
        let (_, _, _, _, metadata2) = create_fungible_asset(aaron);
        let base = FungibleAsset {
            metadata: metadata1,
            amount: 1,
        };
        let addon = FungibleAsset {
            metadata: metadata2,
            amount: 1
        };
        merge(&mut base, addon);
        let FungibleAsset {
            metadata: _,
            amount: _
        } = base;
    }

    #[test(fx = @aptos_framework, creator = @0xcafe)]
    fun test_fungible_asset_upgrade(fx: &signer, creator: &signer) acquires Supply, ConcurrentSupply, FungibleStore, ConcurrentFungibleBalance {
        let supply_feature = features::get_concurrent_fungible_assets_feature();
        let balance_feature = features::get_concurrent_fungible_balance_feature();
        let default_balance_feature = features::get_default_to_concurrent_fungible_balance_feature();

        features::change_feature_flags_for_testing(fx, vector[], vector[supply_feature, balance_feature, default_balance_feature]);

        let (creator_ref, token_object) = create_test_token(creator);
        let (mint_ref, transfer_ref, _burn, _mutate_metadata_ref) = init_test_metadata(&creator_ref);
        let test_token = object::convert<TestToken, Metadata>(token_object);
        assert!(exists<Supply>(object::object_address(&test_token)), 1);
        assert!(!exists<ConcurrentSupply>(object::object_address(&test_token)), 2);
        let creator_store = create_test_store(creator, test_token);
        assert!(exists<FungibleStore>(object::object_address(&creator_store)), 3);
        assert!(!exists<ConcurrentFungibleBalance>(object::object_address(&creator_store)), 4);

        let fa = mint(&mint_ref, 30);
        assert!(supply(test_token) == option::some(30), 5);

        deposit_with_ref(&transfer_ref, creator_store, fa);
        assert!(exists<FungibleStore>(object::object_address(&creator_store)), 13);
        assert!(borrow_store_resource(&creator_store).balance == 30, 14);
        assert!(!exists<ConcurrentFungibleBalance>(object::object_address(&creator_store)), 15);

        features::change_feature_flags_for_testing(fx, vector[supply_feature, balance_feature], vector[default_balance_feature]);

        let extend_ref = object::generate_extend_ref(&creator_ref);
        // manual conversion of supply
        upgrade_to_concurrent(&extend_ref);
        assert!(!exists<Supply>(object::object_address(&test_token)), 6);
        assert!(exists<ConcurrentSupply>(object::object_address(&test_token)), 7);

        // assert conversion of balance
        upgrade_store_to_concurrent(creator, creator_store);
        let fb = withdraw_with_ref(&transfer_ref, creator_store, 20);
        // both store and new balance need to exist. Old balance should be 0.
        assert!(exists<FungibleStore>(object::object_address(&creator_store)), 9);
        assert!(borrow_store_resource(&creator_store).balance == 0, 10);
        assert!(exists<ConcurrentFungibleBalance>(object::object_address(&creator_store)), 11);
        assert!(aggregator_v2::read(&borrow_global<ConcurrentFungibleBalance>(object::object_address(&creator_store)).balance) == 10, 12);

        deposit_with_ref(&transfer_ref, creator_store, fb);
    }

    #[test(fx = @aptos_framework, creator = @0xcafe)]
    fun test_fungible_asset_default_concurrent(fx: &signer, creator: &signer) acquires Supply, ConcurrentSupply, FungibleStore, ConcurrentFungibleBalance {
        let supply_feature = features::get_concurrent_fungible_assets_feature();
        let balance_feature = features::get_concurrent_fungible_balance_feature();
        let default_balance_feature = features::get_default_to_concurrent_fungible_balance_feature();

        features::change_feature_flags_for_testing(fx, vector[supply_feature, balance_feature, default_balance_feature], vector[]);

        let (creator_ref, token_object) = create_test_token(creator);
        let (mint_ref, transfer_ref, _burn, _mutate_metadata_ref) = init_test_metadata(&creator_ref);
        let test_token = object::convert<TestToken, Metadata>(token_object);
        assert!(!exists<Supply>(object::object_address(&test_token)), 1);
        assert!(exists<ConcurrentSupply>(object::object_address(&test_token)), 2);
        let creator_store = create_test_store(creator, test_token);
        assert!(exists<FungibleStore>(object::object_address(&creator_store)), 3);
        assert!(exists<ConcurrentFungibleBalance>(object::object_address(&creator_store)), 4);

        let fa = mint(&mint_ref, 30);
        assert!(supply(test_token) == option::some(30), 5);

        deposit_with_ref(&transfer_ref, creator_store, fa);

        assert!(exists<FungibleStore>(object::object_address(&creator_store)), 9);
        assert!(borrow_store_resource(&creator_store).balance == 0, 10);
        assert!(exists<ConcurrentFungibleBalance>(object::object_address(&creator_store)), 11);
        assert!(aggregator_v2::read(&borrow_global<ConcurrentFungibleBalance>(object::object_address(&creator_store)).balance) == 30, 12);
    }

    #[deprecated]
    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct FungibleAssetEvents has key {
        deposit_events: event::EventHandle<DepositEvent>,
        withdraw_events: event::EventHandle<WithdrawEvent>,
        frozen_events: event::EventHandle<FrozenEvent>,
    }

    #[deprecated]
    struct DepositEvent has drop, store {
        amount: u64,
    }

    #[deprecated]
    struct WithdrawEvent has drop, store {
        amount: u64,
    }

    #[deprecated]
    struct FrozenEvent has drop, store {
        frozen: bool,
    }
}
