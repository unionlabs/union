/// This module provides a way for creators of fungible assets to enable support for creating primary (deterministic)
/// stores for their users. This is useful for assets that are meant to be used as a currency, as it allows users to
/// easily create a store for their account and deposit/withdraw/transfer fungible assets to/from it.
///
/// The transfer flow works as below:
/// 1. The sender calls `transfer` on the fungible asset metadata object to transfer `amount` of fungible asset to
///   `recipient`.
/// 2. The fungible asset metadata object calls `ensure_primary_store_exists` to ensure that both the sender's and the
/// recipient's primary stores exist. If either doesn't, it will be created.
/// 3. The fungible asset metadata object calls `withdraw` on the sender's primary store to withdraw `amount` of
/// fungible asset from it. This emits a withdraw event.
/// 4. The fungible asset metadata object calls `deposit` on the recipient's primary store to deposit `amount` of
/// fungible asset to it. This emits an deposit event.
module aptos_framework::primary_fungible_store {
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::fungible_asset::{Self, FungibleAsset, FungibleStore, Metadata, MintRef, TransferRef, BurnRef};
    use aptos_framework::object::{Self, Object, ConstructorRef, DeriveRef};

    use std::option::Option;
    use std::signer;
    use std::string::String;

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// A resource that holds the derive ref for the fungible asset metadata object. This is used to create primary
    /// stores for users with deterministic addresses so that users can easily deposit/withdraw/transfer fungible
    /// assets.
    struct DeriveRefPod has key {
        metadata_derive_ref: DeriveRef,
    }

    /// Create a fungible asset with primary store support. When users transfer fungible assets to each other, their
    /// primary stores will be created automatically if they don't exist. Primary stores have deterministic addresses
    /// so that users can easily deposit/withdraw/transfer fungible assets.
    public fun create_primary_store_enabled_fungible_asset(
        constructor_ref: &ConstructorRef,
        maximum_supply: Option<u128>,
        name: String,
        symbol: String,
        decimals: u8,
        icon_uri: String,
        project_uri: String,
    ) {
        fungible_asset::add_fungibility(
            constructor_ref,
            maximum_supply,
            name,
            symbol,
            decimals,
            icon_uri,
            project_uri,
        );
        let metadata_obj = &object::generate_signer(constructor_ref);
        move_to(metadata_obj, DeriveRefPod {
            metadata_derive_ref: object::generate_derive_ref(constructor_ref),
        });
    }

    /// Ensure that the primary store object for the given address exists. If it doesn't, create it.
    public fun ensure_primary_store_exists<T: key>(
        owner: address,
        metadata: Object<T>,
    ): Object<FungibleStore> acquires DeriveRefPod {
        let store_addr = primary_store_address(owner, metadata);
        if (fungible_asset::store_exists(store_addr)) {
            object::address_to_object(store_addr)
        } else {
            create_primary_store(owner, metadata)
        }
    }

    /// Create a primary store object to hold fungible asset for the given address.
    public fun create_primary_store<T: key>(
        owner_addr: address,
        metadata: Object<T>,
    ): Object<FungibleStore> acquires DeriveRefPod {
        let metadata_addr = object::object_address(&metadata);
        object::address_to_object<Metadata>(metadata_addr);
        let derive_ref = &borrow_global<DeriveRefPod>(metadata_addr).metadata_derive_ref;
        let constructor_ref = &object::create_user_derived_object(owner_addr, derive_ref);
        // Disable ungated transfer as deterministic stores shouldn't be transferrable.
        let transfer_ref = &object::generate_transfer_ref(constructor_ref);
        object::disable_ungated_transfer(transfer_ref);

        fungible_asset::create_store(constructor_ref, metadata)
    }

    #[view]
    /// Get the address of the primary store for the given account.
    public fun primary_store_address<T: key>(owner: address, metadata: Object<T>): address {
        let metadata_addr = object::object_address(&metadata);
        object::create_user_derived_object_address(owner, metadata_addr)
    }

    #[view]
    /// Get the primary store object for the given account.
    public fun primary_store<T: key>(owner: address, metadata: Object<T>): Object<FungibleStore> {
        let store = primary_store_address(owner, metadata);
        object::address_to_object<FungibleStore>(store)
    }

    #[view]
    /// Return whether the given account's primary store exists.
    public fun primary_store_exists<T: key>(account: address, metadata: Object<T>): bool {
        fungible_asset::store_exists(primary_store_address(account, metadata))
    }

    /// Get the address of the primary store for the given account.
    /// Use instead of the corresponding view functions for dispatchable hooks to avoid circular dependencies of modules.
    public inline fun primary_store_address_inlined<T: key>(owner: address, metadata: Object<T>): address {
        let metadata_addr = object::object_address(&metadata);
        object::create_user_derived_object_address(owner, metadata_addr)
    }

    /// Get the primary store object for the given account.
    /// Use instead of the corresponding view functions for dispatchable hooks to avoid circular dependencies of modules.
    public inline fun primary_store_inlined<T: key>(owner: address, metadata: Object<T>): Object<FungibleStore> {
        let store = primary_store_address_inlined(owner, metadata);
        object::address_to_object(store)
    }

    /// Return whether the given account's primary store exists.
    /// Use instead of the corresponding view functions for dispatchable hooks to avoid circular dependencies of modules.
    public inline fun primary_store_exists_inlined<T: key>(account: address, metadata: Object<T>): bool {
        fungible_asset::store_exists(primary_store_address_inlined(account, metadata))
    }

    #[view]
    /// Get the balance of `account`'s primary store.
    public fun balance<T: key>(account: address, metadata: Object<T>): u64 {
        if (primary_store_exists(account, metadata)) {
            fungible_asset::balance(primary_store(account, metadata))
        } else {
            0
        }
    }

    #[view]
    public fun is_balance_at_least<T: key>(account: address, metadata: Object<T>, amount: u64): bool {
        if (primary_store_exists(account, metadata)) {
            fungible_asset::is_balance_at_least(primary_store(account, metadata), amount)
        } else {
            amount == 0
        }
    }

    #[view]
    /// Return whether the given account's primary store is frozen.
    public fun is_frozen<T: key>(account: address, metadata: Object<T>): bool {
        if (primary_store_exists(account, metadata)) {
            fungible_asset::is_frozen(primary_store(account, metadata))
        } else {
            false
        }
    }

    /// Withdraw `amount` of fungible asset from the given account's primary store.
    public fun withdraw<T: key>(owner: &signer, metadata: Object<T>, amount: u64): FungibleAsset acquires DeriveRefPod {
        let store = ensure_primary_store_exists(signer::address_of(owner), metadata);
        // Check if the store object has been burnt or not. If so, unburn it first.
        may_be_unburn(owner, store);
        dispatchable_fungible_asset::withdraw(owner, store, amount)
    }

    /// Deposit fungible asset `fa` to the given account's primary store.
    public fun deposit(owner: address, fa: FungibleAsset) acquires DeriveRefPod {
        let metadata = fungible_asset::asset_metadata(&fa);
        let store = ensure_primary_store_exists(owner, metadata);
        dispatchable_fungible_asset::deposit(store, fa);
    }

    /// Deposit fungible asset `fa` to the given account's primary store.
    public(friend) fun force_deposit(owner: address, fa: FungibleAsset) acquires DeriveRefPod {
        let metadata = fungible_asset::asset_metadata(&fa);
        let store = ensure_primary_store_exists(owner, metadata);
        fungible_asset::deposit_internal(object::object_address(&store), fa);
    }

    /// Transfer `amount` of fungible asset from sender's primary store to receiver's primary store.
    public entry fun transfer<T: key>(
        sender: &signer,
        metadata: Object<T>,
        recipient: address,
        amount: u64,
    ) acquires DeriveRefPod {
        let sender_store = ensure_primary_store_exists(signer::address_of(sender), metadata);
        // Check if the sender store object has been burnt or not. If so, unburn it first.
        may_be_unburn(sender, sender_store);
        let recipient_store = ensure_primary_store_exists(recipient, metadata);
        dispatchable_fungible_asset::transfer(sender, sender_store, recipient_store, amount);
    }

    /// Transfer `amount` of fungible asset from sender's primary store to receiver's primary store.
    /// Use the minimum deposit assertion api to make sure receipient will receive a minimum amount of fund.
    public entry fun transfer_assert_minimum_deposit<T: key>(
        sender: &signer,
        metadata: Object<T>,
        recipient: address,
        amount: u64,
        expected: u64,
    ) acquires DeriveRefPod {
        let sender_store = ensure_primary_store_exists(signer::address_of(sender), metadata);
        // Check if the sender store object has been burnt or not. If so, unburn it first.
        may_be_unburn(sender, sender_store);
        let recipient_store = ensure_primary_store_exists(recipient, metadata);
        dispatchable_fungible_asset::transfer_assert_minimum_deposit(
            sender,
            sender_store,
            recipient_store,
            amount,
            expected
        );
    }

    /// Mint to the primary store of `owner`.
    public fun mint(mint_ref: &MintRef, owner: address, amount: u64) acquires DeriveRefPod {
        let primary_store = ensure_primary_store_exists(owner, fungible_asset::mint_ref_metadata(mint_ref));
        fungible_asset::mint_to(mint_ref, primary_store, amount);
    }

    /// Burn from the primary store of `owner`.
    public fun burn(burn_ref: &BurnRef, owner: address, amount: u64) {
        let primary_store = primary_store(owner, fungible_asset::burn_ref_metadata(burn_ref));
        fungible_asset::burn_from(burn_ref, primary_store, amount);
    }

    /// Freeze/Unfreeze the primary store of `owner`.
    public fun set_frozen_flag(transfer_ref: &TransferRef, owner: address, frozen: bool) acquires DeriveRefPod {
        let primary_store = ensure_primary_store_exists(owner, fungible_asset::transfer_ref_metadata(transfer_ref));
        fungible_asset::set_frozen_flag(transfer_ref, primary_store, frozen);
    }

    /// Withdraw from the primary store of `owner` ignoring frozen flag.
    public fun withdraw_with_ref(transfer_ref: &TransferRef, owner: address, amount: u64): FungibleAsset {
        let from_primary_store = primary_store(owner, fungible_asset::transfer_ref_metadata(transfer_ref));
        fungible_asset::withdraw_with_ref(transfer_ref, from_primary_store, amount)
    }

    /// Deposit from the primary store of `owner` ignoring frozen flag.
    public fun deposit_with_ref(transfer_ref: &TransferRef, owner: address, fa: FungibleAsset) acquires DeriveRefPod {
        let from_primary_store = ensure_primary_store_exists(
            owner,
            fungible_asset::transfer_ref_metadata(transfer_ref)
        );
        fungible_asset::deposit_with_ref(transfer_ref, from_primary_store, fa);
    }

    /// Transfer `amount` of FA from the primary store of `from` to that of `to` ignoring frozen flag.
    public fun transfer_with_ref(
        transfer_ref: &TransferRef,
        from: address,
        to: address,
        amount: u64
    ) acquires DeriveRefPod {
        let from_primary_store = primary_store(from, fungible_asset::transfer_ref_metadata(transfer_ref));
        let to_primary_store = ensure_primary_store_exists(to, fungible_asset::transfer_ref_metadata(transfer_ref));
        fungible_asset::transfer_with_ref(transfer_ref, from_primary_store, to_primary_store, amount);
    }

    fun may_be_unburn(owner: &signer, store: Object<FungibleStore>) {
        if (object::is_burnt(store)) {
            object::unburn(owner, store);
        };
    }

    #[test_only]
    use aptos_framework::fungible_asset::{
        create_test_token,
        generate_mint_ref,
        generate_burn_ref,
        generate_transfer_ref
    };
    #[test_only]
    use std::string;
    #[test_only]
    use std::option;

    #[test_only]
    public fun init_test_metadata_with_primary_store_enabled(
        constructor_ref: &ConstructorRef
    ): (MintRef, TransferRef, BurnRef) {
        create_primary_store_enabled_fungible_asset(
            constructor_ref,
            option::some(100), // max supply
            string::utf8(b"TEST COIN"),
            string::utf8(b"@T"),
            0,
            string::utf8(b"http://example.com/icon"),
            string::utf8(b"http://example.com"),
        );
        let mint_ref = generate_mint_ref(constructor_ref);
        let burn_ref = generate_burn_ref(constructor_ref);
        let transfer_ref = generate_transfer_ref(constructor_ref);
        (mint_ref, transfer_ref, burn_ref)
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    fun test_default_behavior(creator: &signer, aaron: &signer) acquires DeriveRefPod {
        let (creator_ref, metadata) = create_test_token(creator);
        init_test_metadata_with_primary_store_enabled(&creator_ref);
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
    ) acquires DeriveRefPod {
        let (creator_ref, metadata) = create_test_token(creator);
        let (mint_ref, transfer_ref, burn_ref) = init_test_metadata_with_primary_store_enabled(&creator_ref);
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
    ) acquires DeriveRefPod {
        let (creator_ref, metadata) = create_test_token(creator);
        let (mint_ref, _transfer_ref, _) = init_test_metadata_with_primary_store_enabled(&creator_ref);
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

    #[test(user_1 = @0xcafe, user_2 = @0xface)]
    fun test_transfer_to_burnt_store(
        user_1: &signer,
        user_2: &signer,
    ) acquires DeriveRefPod {
        let (creator_ref, metadata) = create_test_token(user_1);
        let (mint_ref, _, _) = init_test_metadata_with_primary_store_enabled(&creator_ref);
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
    ) acquires DeriveRefPod {
        let (creator_ref, metadata) = create_test_token(user_1);
        let (mint_ref, _, _) = init_test_metadata_with_primary_store_enabled(&creator_ref);
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
}
