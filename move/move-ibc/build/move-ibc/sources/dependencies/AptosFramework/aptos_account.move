module aptos_framework::aptos_account {
    use aptos_framework::account::{Self, new_event_handle};
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::create_signer::create_signer;
    use aptos_framework::event::{EventHandle, emit_event, emit};
    use aptos_framework::fungible_asset::{Self, Metadata, BurnRef};
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object;

    use std::error;
    use std::features;
    use std::signer;
    use std::vector;

    friend aptos_framework::genesis;
    friend aptos_framework::resource_account;
    friend aptos_framework::transaction_fee;
    friend aptos_framework::transaction_validation;

    /// Account does not exist.
    const EACCOUNT_NOT_FOUND: u64 = 1;
    /// Account is not registered to receive APT.
    const EACCOUNT_NOT_REGISTERED_FOR_APT: u64 = 2;
    /// Account opted out of receiving coins that they did not register to receive.
    const EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS: u64 = 3;
    /// Account opted out of directly receiving NFT tokens.
    const EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS: u64 = 4;
    /// The lengths of the recipients and amounts lists don't match.
    const EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH: u64 = 5;

    /// Configuration for whether an account can receive direct transfers of coins that they have not registered.
    ///
    /// By default, this is enabled. Users can opt-out by disabling at any time.
    struct DirectTransferConfig has key {
        allow_arbitrary_coin_transfers: bool,
        update_coin_transfer_events: EventHandle<DirectCoinTransferConfigUpdatedEvent>,
    }

    /// Event emitted when an account's direct coins transfer config is updated.
    struct DirectCoinTransferConfigUpdatedEvent has drop, store {
        new_allow_direct_transfers: bool,
    }

    #[event]
    struct DirectCoinTransferConfigUpdated has drop, store {
        account: address,
        new_allow_direct_transfers: bool,
    }

    ///////////////////////////////////////////////////////////////////////////
    /// Basic account creation methods.
    ///////////////////////////////////////////////////////////////////////////

    public entry fun create_account(auth_key: address) {
        let account_signer = account::create_account(auth_key);
        register_apt(&account_signer);
    }

    /// Batch version of APT transfer.
    public entry fun batch_transfer(source: &signer, recipients: vector<address>, amounts: vector<u64>) {
        let recipients_len = vector::length(&recipients);
        assert!(
            recipients_len == vector::length(&amounts),
            error::invalid_argument(EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH),
        );

        vector::enumerate_ref(&recipients, |i, to| {
            let amount = *vector::borrow(&amounts, i);
            transfer(source, *to, amount);
        });
    }

    /// Convenient function to transfer APT to a recipient account that might not exist.
    /// This would create the recipient account first, which also registers it to receive APT, before transferring.
    public entry fun transfer(source: &signer, to: address, amount: u64) {
        if (!account::exists_at(to)) {
            create_account(to)
        };

        if (features::operations_default_to_fa_apt_store_enabled()) {
            fungible_transfer_only(source, to, amount)
        } else {
            // Resource accounts can be created without registering them to receive APT.
            // This conveniently does the registration if necessary.
            if (!coin::is_account_registered<AptosCoin>(to)) {
                coin::register<AptosCoin>(&create_signer(to));
            };
            coin::transfer<AptosCoin>(source, to, amount)
        }
    }

    /// Batch version of transfer_coins.
    public entry fun batch_transfer_coins<CoinType>(
        from: &signer, recipients: vector<address>, amounts: vector<u64>) acquires DirectTransferConfig {
        let recipients_len = vector::length(&recipients);
        assert!(
            recipients_len == vector::length(&amounts),
            error::invalid_argument(EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH),
        );

        vector::enumerate_ref(&recipients, |i, to| {
            let amount = *vector::borrow(&amounts, i);
            transfer_coins<CoinType>(from, *to, amount);
        });
    }

    /// Convenient function to transfer a custom CoinType to a recipient account that might not exist.
    /// This would create the recipient account first and register it to receive the CoinType, before transferring.
    public entry fun transfer_coins<CoinType>(from: &signer, to: address, amount: u64) acquires DirectTransferConfig {
        deposit_coins(to, coin::withdraw<CoinType>(from, amount));
    }

    /// Convenient function to deposit a custom CoinType into a recipient account that might not exist.
    /// This would create the recipient account first and register it to receive the CoinType, before transferring.
    public fun deposit_coins<CoinType>(to: address, coins: Coin<CoinType>) acquires DirectTransferConfig {
        if (!account::exists_at(to)) {
            create_account(to);
            spec {
                assert coin::spec_is_account_registered<AptosCoin>(to);
                assume aptos_std::type_info::type_of<CoinType>() == aptos_std::type_info::type_of<AptosCoin>() ==>
                    coin::spec_is_account_registered<CoinType>(to);
            };
        };
        if (!coin::is_account_registered<CoinType>(to)) {
            assert!(
                can_receive_direct_coin_transfers(to),
                error::permission_denied(EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS),
            );
            coin::register<CoinType>(&create_signer(to));
        };
        coin::deposit<CoinType>(to, coins)
    }

    public fun assert_account_exists(addr: address) {
        assert!(account::exists_at(addr), error::not_found(EACCOUNT_NOT_FOUND));
    }

    public fun assert_account_is_registered_for_apt(addr: address) {
        assert_account_exists(addr);
        assert!(coin::is_account_registered<AptosCoin>(addr), error::not_found(EACCOUNT_NOT_REGISTERED_FOR_APT));
    }

    /// Set whether `account` can receive direct transfers of coins that they have not explicitly registered to receive.
    public entry fun set_allow_direct_coin_transfers(account: &signer, allow: bool) acquires DirectTransferConfig {
        let addr = signer::address_of(account);
        if (exists<DirectTransferConfig>(addr)) {
            let direct_transfer_config = borrow_global_mut<DirectTransferConfig>(addr);
            // Short-circuit to avoid emitting an event if direct transfer config is not changing.
            if (direct_transfer_config.allow_arbitrary_coin_transfers == allow) {
                return
            };

            direct_transfer_config.allow_arbitrary_coin_transfers = allow;

            if (std::features::module_event_migration_enabled()) {
                emit(DirectCoinTransferConfigUpdated { account: addr, new_allow_direct_transfers: allow });
            };
            emit_event(
                &mut direct_transfer_config.update_coin_transfer_events,
                DirectCoinTransferConfigUpdatedEvent { new_allow_direct_transfers: allow });
        } else {
            let direct_transfer_config = DirectTransferConfig {
                allow_arbitrary_coin_transfers: allow,
                update_coin_transfer_events: new_event_handle<DirectCoinTransferConfigUpdatedEvent>(account),
            };
            if (std::features::module_event_migration_enabled()) {
                emit(DirectCoinTransferConfigUpdated { account: addr, new_allow_direct_transfers: allow });
            };
            emit_event(
                &mut direct_transfer_config.update_coin_transfer_events,
                DirectCoinTransferConfigUpdatedEvent { new_allow_direct_transfers: allow });
            move_to(account, direct_transfer_config);
        };
    }

    #[view]
    /// Return true if `account` can receive direct transfers of coins that they have not explicitly registered to
    /// receive.
    ///
    /// By default, this returns true if an account has not explicitly set whether the can receive direct transfers.
    public fun can_receive_direct_coin_transfers(account: address): bool acquires DirectTransferConfig {
        !exists<DirectTransferConfig>(account) ||
            borrow_global<DirectTransferConfig>(account).allow_arbitrary_coin_transfers
    }

    public(friend) fun register_apt(account_signer: &signer) {
        if (features::new_accounts_default_to_fa_apt_store_enabled()) {
            ensure_primary_fungible_store_exists(signer::address_of(account_signer));
        } else {
            coin::register<AptosCoin>(account_signer);
        }
    }

    /// APT Primary Fungible Store specific specialized functions,
    /// Utilized internally once migration of APT to FungibleAsset is complete.

    /// Convenient function to transfer APT to a recipient account that might not exist.
    /// This would create the recipient APT PFS first, which also registers it to receive APT, before transferring.
    /// TODO: once migration is complete, rename to just "transfer_only" and make it an entry function (for cheapest way
    /// to transfer APT) - if we want to allow APT PFS without account itself
    fun fungible_transfer_only(
        source: &signer, to: address, amount: u64
    ) {
        let sender_store = ensure_primary_fungible_store_exists(signer::address_of(source));
        let recipient_store = ensure_primary_fungible_store_exists(to);

        // use internal APIs, as they skip:
        // - owner, frozen and dispatchable checks
        // as APT cannot be frozen or have dispatch, and PFS cannot be transfered
        // (PFS could potentially be burned. regular transfer would permanently unburn the store.
        // Ignoring the check here has the equivalent of unburning, transfers, and then burning again)
        fungible_asset::deposit_internal(recipient_store, fungible_asset::withdraw_internal(sender_store, amount));
    }

    /// Is balance from APT Primary FungibleStore at least the given amount
    public(friend) fun is_fungible_balance_at_least(account: address, amount: u64): bool {
        let store_addr = primary_fungible_store_address(account);
        fungible_asset::is_address_balance_at_least(store_addr, amount)
    }

    /// Burn from APT Primary FungibleStore
    public(friend) fun burn_from_fungible_store(
        ref: &BurnRef,
        account: address,
        amount: u64,
    ) {
        // Skip burning if amount is zero. This shouldn't error out as it's called as part of transaction fee burning.
        if (amount != 0) {
            let store_addr = primary_fungible_store_address(account);
            fungible_asset::address_burn_from(ref, store_addr, amount);
        };
    }

    /// Ensure that APT Primary FungibleStore exists (and create if it doesn't)
    inline fun ensure_primary_fungible_store_exists(owner: address): address {
        let store_addr = primary_fungible_store_address(owner);
        if (fungible_asset::store_exists(store_addr)) {
            store_addr
        } else {
            object::object_address(&primary_fungible_store::create_primary_store(owner, object::address_to_object<Metadata>(@aptos_fungible_asset)))
        }
    }

    /// Address of APT Primary Fungible Store
    inline fun primary_fungible_store_address(account: address): address {
        object::create_user_derived_object_address(account, @aptos_fungible_asset)
    }

    // tests

    #[test_only]
    use aptos_std::from_bcs;
    #[test_only]
    use std::string::utf8;
    #[test_only]
    use aptos_framework::account::create_account_for_test;

    #[test_only]
    struct FakeCoin {}

    #[test(alice = @0xa11ce, core = @0x1)]
    public fun test_transfer(alice: &signer, core: &signer) {
        let bob = from_bcs::to_address(x"0000000000000000000000000000000000000000000000000000000000000b0b");
        let carol = from_bcs::to_address(x"00000000000000000000000000000000000000000000000000000000000ca501");

        let (burn_cap, mint_cap) = aptos_framework::aptos_coin::initialize_for_test(core);
        create_account(signer::address_of(alice));
        coin::deposit(signer::address_of(alice), coin::mint(10000, &mint_cap));
        transfer(alice, bob, 500);
        assert!(coin::balance<AptosCoin>(bob) == 500, 0);
        transfer(alice, carol, 500);
        assert!(coin::balance<AptosCoin>(carol) == 500, 1);
        transfer(alice, carol, 1500);
        assert!(coin::balance<AptosCoin>(carol) == 2000, 2);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    #[test(alice = @0xa11ce, core = @0x1)]
    public fun test_transfer_to_resource_account(alice: &signer, core: &signer) {
        let (resource_account, _) = account::create_resource_account(alice, vector[]);
        let resource_acc_addr = signer::address_of(&resource_account);
        let (burn_cap, mint_cap) = aptos_framework::aptos_coin::initialize_for_test(core);
        assert!(!coin::is_account_registered<AptosCoin>(resource_acc_addr), 0);

        create_account(signer::address_of(alice));
        coin::deposit(signer::address_of(alice), coin::mint(10000, &mint_cap));
        transfer(alice, resource_acc_addr, 500);
        assert!(coin::balance<AptosCoin>(resource_acc_addr) == 500, 1);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    #[test(from = @0x123, core = @0x1, recipient_1 = @0x124, recipient_2 = @0x125)]
    public fun test_batch_transfer(from: &signer, core: &signer, recipient_1: &signer, recipient_2: &signer) {
        let (burn_cap, mint_cap) = aptos_framework::aptos_coin::initialize_for_test(core);
        create_account(signer::address_of(from));
        let recipient_1_addr = signer::address_of(recipient_1);
        let recipient_2_addr = signer::address_of(recipient_2);
        create_account(recipient_1_addr);
        create_account(recipient_2_addr);
        coin::deposit(signer::address_of(from), coin::mint(10000, &mint_cap));
        batch_transfer(
            from,
            vector[recipient_1_addr, recipient_2_addr],
            vector[100, 500],
        );
        assert!(coin::balance<AptosCoin>(recipient_1_addr) == 100, 0);
        assert!(coin::balance<AptosCoin>(recipient_2_addr) == 500, 1);
        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    #[test(from = @0x1, to = @0x12)]
    public fun test_direct_coin_transfers(from: &signer, to: &signer) acquires DirectTransferConfig {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
            from,
            utf8(b"FC"),
            utf8(b"FC"),
            10,
            true,
        );
        create_account_for_test(signer::address_of(from));
        create_account_for_test(signer::address_of(to));
        deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
        // Recipient account did not explicit register for the coin.
        let to_addr = signer::address_of(to);
        transfer_coins<FakeCoin>(from, to_addr, 500);
        assert!(coin::balance<FakeCoin>(to_addr) == 500, 0);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_freeze_cap(freeze_cap);
    }

    #[test(from = @0x1, recipient_1 = @0x124, recipient_2 = @0x125)]
    public fun test_batch_transfer_coins(
        from: &signer, recipient_1: &signer, recipient_2: &signer) acquires DirectTransferConfig {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
            from,
            utf8(b"FC"),
            utf8(b"FC"),
            10,
            true,
        );
        create_account_for_test(signer::address_of(from));
        let recipient_1_addr = signer::address_of(recipient_1);
        let recipient_2_addr = signer::address_of(recipient_2);
        create_account_for_test(recipient_1_addr);
        create_account_for_test(recipient_2_addr);
        deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
        batch_transfer_coins<FakeCoin>(
            from,
            vector[recipient_1_addr, recipient_2_addr],
            vector[100, 500],
        );
        assert!(coin::balance<FakeCoin>(recipient_1_addr) == 100, 0);
        assert!(coin::balance<FakeCoin>(recipient_2_addr) == 500, 1);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_freeze_cap(freeze_cap);
    }

    #[test(user = @0x123)]
    public fun test_set_allow_direct_coin_transfers(user: &signer) acquires DirectTransferConfig {
        let addr = signer::address_of(user);
        create_account_for_test(addr);
        set_allow_direct_coin_transfers(user, true);
        assert!(can_receive_direct_coin_transfers(addr), 0);
        set_allow_direct_coin_transfers(user, false);
        assert!(!can_receive_direct_coin_transfers(addr), 1);
        set_allow_direct_coin_transfers(user, true);
        assert!(can_receive_direct_coin_transfers(addr), 2);
    }

    #[test(from = @0x1, to = @0x12)]
    public fun test_direct_coin_transfers_with_explicit_direct_coin_transfer_config(
        from: &signer, to: &signer) acquires DirectTransferConfig {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
            from,
            utf8(b"FC"),
            utf8(b"FC"),
            10,
            true,
        );
        create_account_for_test(signer::address_of(from));
        create_account_for_test(signer::address_of(to));
        set_allow_direct_coin_transfers(from, true);
        deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
        // Recipient account did not explicit register for the coin.
        let to_addr = signer::address_of(to);
        transfer_coins<FakeCoin>(from, to_addr, 500);
        assert!(coin::balance<FakeCoin>(to_addr) == 500, 0);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_freeze_cap(freeze_cap);
    }

    #[test(from = @0x1, to = @0x12)]
    #[expected_failure(abort_code = 0x50003, location = Self)]
    public fun test_direct_coin_transfers_fail_if_recipient_opted_out(
        from: &signer, to: &signer) acquires DirectTransferConfig {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
            from,
            utf8(b"FC"),
            utf8(b"FC"),
            10,
            true,
        );
        create_account_for_test(signer::address_of(from));
        create_account_for_test(signer::address_of(to));
        set_allow_direct_coin_transfers(from, false);
        deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
        // This should fail as the to account has explicitly opted out of receiving arbitrary coins.
        transfer_coins<FakeCoin>(from, signer::address_of(to), 500);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_freeze_cap(freeze_cap);
    }

    #[test(user = @0xcafe)]
    fun test_primary_fungible_store_address(
        user: &signer,
    ) {
        use aptos_framework::fungible_asset::Metadata;
        use aptos_framework::aptos_coin;

        aptos_coin::ensure_initialized_with_apt_fa_metadata_for_test();

        let apt_metadata = object::address_to_object<Metadata>(@aptos_fungible_asset);
        let user_addr = signer::address_of(user);
        assert!(primary_fungible_store_address(user_addr) == primary_fungible_store::primary_store_address(user_addr, apt_metadata), 1);

        ensure_primary_fungible_store_exists(user_addr);
        assert!(primary_fungible_store::primary_store_exists(user_addr, apt_metadata), 2);
    }
}
