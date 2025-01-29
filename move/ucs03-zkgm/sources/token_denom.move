module zkgm::fa_coin {
    use aptos_framework::fungible_asset::{
        Self,
        MintRef,
        TransferRef,
        MutateMetadataRef,
        BurnRef,
        Metadata,
        FungibleAsset
    };
    use aptos_framework::object::{Self, Object};
    use aptos_framework::primary_fungible_store;
    use std::error;
    use std::signer;
    use std::string::{Self};
    use std::option;

    /// Only fungible asset metadata owner can make changes.
    const E_NOT_OWNER: u64 = 1;

    const ASSET_SYMBOL: vector<u8> = b"FA";
    const IBC_APP_SEED: vector<u8> = b"ibc-union-app-v1";

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Hold refs to control the minting, transfer and burning of fungible assets.
    struct ManagedFungibleAsset has key {
        mint_ref: MintRef,
        transfer_ref: TransferRef,
        burn_ref: BurnRef,
        mutate_metadata_ref: MutateMetadataRef
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Global state to pause the FA coin.
    /// OPTIONAL
    struct State has key {
        paused: bool
    }

    /// Initialize metadata object and store the refs.
    public entry fun initialize(
        admin: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
        icon: string::String,
        project: string::String,
        asset_seed: vector<u8>
    ) {
        let constructor_ref = &object::create_named_object(admin, asset_seed);
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            constructor_ref,
            option::none(),
            name,
            symbol,
            decimals,
            icon,
            project
        );

        // Create mint/burn/transfer refs to allow creator to manage the fungible asset.
        let mint_ref = fungible_asset::generate_mint_ref(constructor_ref);
        let burn_ref = fungible_asset::generate_burn_ref(constructor_ref);
        let transfer_ref = fungible_asset::generate_transfer_ref(constructor_ref);
        let mutate_metadata_ref =
            fungible_asset::generate_mutate_metadata_ref(constructor_ref);
        let metadata_object_signer = object::generate_signer(constructor_ref);
        move_to(
            &metadata_object_signer,
            ManagedFungibleAsset { mint_ref, transfer_ref, burn_ref, mutate_metadata_ref }
        );

        // Create a global state to pause the FA coin and move to Metadata object.
        move_to(&metadata_object_signer, State { paused: false });
    }

    #[view]
    /// Return the address of the managed fungible asset that's created when this module is deployed.
    public fun get_metadata(asset_seed: vector<u8>): Object<Metadata> {
        let asset_address = object::create_object_address(&get_owner_addr(), asset_seed);
        object::address_to_object<Metadata>(asset_address)
    }

    #[view]
    /// Return the address of the managed fungible asset that's created when this module is deployed.
    public fun get_metadata_address(asset_seed: vector<u8>): address {
        object::create_object_address(&get_owner_addr(), asset_seed)
    }

    #[view]
    public fun get_owner_addr(): address {
        object::create_object_address(&@zkgm, IBC_APP_SEED)
    }

    #[view]
    public fun decimals(asset_seed: vector<u8>): u8 {
        let asset_metadata = get_metadata(asset_seed);
        fungible_asset::decimals<Metadata>(asset_metadata)
    }

    #[view]
    public fun symbol(asset_seed: vector<u8>): string::String {
        let asset_metadata = get_metadata(asset_seed);
        fungible_asset::symbol<Metadata>(asset_metadata)
    }

    #[view]
    public fun name(asset_seed: vector<u8>): string::String {
        let asset_metadata = get_metadata(asset_seed);
        fungible_asset::name<Metadata>(asset_metadata)
    }

    #[view]
    public fun name_with_metadata(asset: Object<Metadata>): string::String {
        fungible_asset::name<Metadata>(asset)
    }

    #[view]
    public fun symbol_with_metadata(asset: Object<Metadata>): string::String {
        fungible_asset::symbol<Metadata>(asset)
    }

    /// Deposit function override to ensure that the account is not denylisted and the FA coin is not paused.
    /// OPTIONAL
    public fun deposit<T: key>(
        store: Object<T>, fa: FungibleAsset, transfer_ref: &TransferRef
    ) {
        fungible_asset::deposit_with_ref(transfer_ref, store, fa);
    }

    /// Withdraw function override to ensure that the account is not denylisted and the FA coin is not paused.
    /// OPTIONAL
    public fun withdraw<T: key>(
        store: Object<T>, amount: u64, transfer_ref: &TransferRef
    ): FungibleAsset {
        fungible_asset::withdraw_with_ref(transfer_ref, store, amount)
    }

    /// Mint as the owner of metadata object.
    public entry fun mint_with_metadata(
        admin: &signer,
        to: address,
        amount: u64,
        asset: Object<Metadata>
    ) acquires ManagedFungibleAsset {
        let managed_fungible_asset = authorized_borrow_refs(admin, asset);
        let to_wallet = primary_fungible_store::ensure_primary_store_exists(to, asset);
        let fa = fungible_asset::mint(&managed_fungible_asset.mint_ref, amount);
        fungible_asset::deposit_with_ref(
            &managed_fungible_asset.transfer_ref, to_wallet, fa
        );
    }

    /// Mint as the owner of metadata object.
    public entry fun mint(
        admin: &signer,
        to: address,
        amount: u64,
        asset_seed: vector<u8>
    ) acquires ManagedFungibleAsset {
        let asset = get_metadata(asset_seed);
        let managed_fungible_asset = authorized_borrow_refs(admin, asset);
        let to_wallet = primary_fungible_store::ensure_primary_store_exists(to, asset);
        let fa = fungible_asset::mint(&managed_fungible_asset.mint_ref, amount);
        fungible_asset::deposit_with_ref(
            &managed_fungible_asset.transfer_ref, to_wallet, fa
        );
    }

    public entry fun update_with_metadata(
        admin: &signer,
        name: option::Option<string::String>,
        symbol: option::Option<string::String>,
        decimals: option::Option<u8>,
        icon_uri: option::Option<string::String>,
        project_uri: option::Option<string::String>,
        asset: Object<Metadata>
    ) acquires ManagedFungibleAsset {
        let managed_fungible_asset = authorized_borrow_refs(admin, asset);
        fungible_asset::mutate_metadata(
            &managed_fungible_asset.mutate_metadata_ref,
            name,
            symbol,
            decimals,
            icon_uri,
            project_uri
        );
    }

    /// Transfer as the owner of metadata object.
    public entry fun transfer(
        admin: &signer,
        from: address,
        to: address,
        amount: u64,
        asset_seed: vector<u8>
    ) acquires ManagedFungibleAsset {
        let asset = get_metadata(asset_seed);
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let from_wallet = primary_fungible_store::primary_store(from, asset);
        let to_wallet = primary_fungible_store::ensure_primary_store_exists(to, asset);
        let fa = withdraw(from_wallet, amount, transfer_ref);
        deposit(to_wallet, fa, transfer_ref);
    }

    public entry fun burn(
        admin: &signer,
        from: address,
        amount: u64,
        asset_seed: vector<u8>
    ) acquires ManagedFungibleAsset {
        let asset = get_metadata(asset_seed);
        let burn_ref = &authorized_borrow_refs(admin, asset).burn_ref;
        let from_wallet = primary_fungible_store::primary_store(from, asset);
        fungible_asset::burn_from(burn_ref, from_wallet, amount);
    }

    public entry fun burn_with_metadata(
        admin: &signer,
        from: address,
        amount: u64,
        asset: Object<Metadata>
    ) acquires ManagedFungibleAsset {
        let burn_ref = &authorized_borrow_refs(admin, asset).burn_ref;
        let from_wallet = primary_fungible_store::primary_store(from, asset);
        fungible_asset::burn_from(burn_ref, from_wallet, amount);
    }

    /// Freeze an account so it cannot transfer or receive fungible assets.
    public entry fun freeze_account(
        admin: &signer, account: address, asset_seed: vector<u8>
    ) acquires ManagedFungibleAsset {
        let asset = get_metadata(asset_seed);
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let wallet = primary_fungible_store::ensure_primary_store_exists(account, asset);
        fungible_asset::set_frozen_flag(transfer_ref, wallet, true);
    }

    /// Unfreeze an account so it can transfer or receive fungible assets.
    public entry fun unfreeze_account(
        admin: &signer, account: address, asset_seed: vector<u8>
    ) acquires ManagedFungibleAsset {
        let asset = get_metadata(asset_seed);
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let wallet = primary_fungible_store::ensure_primary_store_exists(account, asset);
        fungible_asset::set_frozen_flag(transfer_ref, wallet, false);
    }

    inline fun authorized_borrow_refs(
        owner: &signer, asset: Object<Metadata>
    ): &ManagedFungibleAsset acquires ManagedFungibleAsset {
        assert!(
            object::is_owner(asset, signer::address_of(owner)),
            error::permission_denied(E_NOT_OWNER)
        );
        borrow_global<ManagedFungibleAsset>(object::object_address(&asset))
    }

    const TEST_NAME: vector<u8> = b"Test Coin";
    const TEST_SYMBOL: vector<u8> = b"TST";
    const TEST_DECIMALS: u8 = 8;
    const TEST_ICON: vector<u8> = b"https://example.com/icon.png";
    const TEST_PROJECT: vector<u8> = b"Test Project";

    #[test(creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57)]
    public fun test_burn_with_metadata(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let asset_metadata = get_metadata(ASSET_SYMBOL);

        let balance_of_face_before =
            primary_fungible_store::balance(@0xface, asset_metadata);
        assert!(balance_of_face_before == 0, 101);
        mint_with_metadata(creator, @0xface, 1000, asset_metadata);
        let balance_of_face = primary_fungible_store::balance(@0xface, asset_metadata);
        assert!(balance_of_face == 1000, 102);

        burn_with_metadata(creator, @0xface, 500, asset_metadata);
        let balance_of_face_after =
            primary_fungible_store::balance(@0xface, asset_metadata);
        assert!(balance_of_face_after == 500, 103);

    }

    #[test(creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57)]
    public fun test_initialize(creator: &signer) {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let asset_metadata = get_metadata(ASSET_SYMBOL);
        let name = fungible_asset::name<Metadata>(asset_metadata);
        let symbol = fungible_asset::symbol<Metadata>(asset_metadata);
        let decimals = fungible_asset::decimals<Metadata>(asset_metadata);

        assert!(name == string::utf8(TEST_NAME), 101);
        assert!(symbol == string::utf8(TEST_SYMBOL), 102);
        assert!(decimals == TEST_DECIMALS, 103);
    }

    #[test(creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57)]
    public fun test_update_metadata(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let asset_metadata = get_metadata(ASSET_SYMBOL);
        let name = fungible_asset::name<Metadata>(asset_metadata);
        let symbol = fungible_asset::symbol<Metadata>(asset_metadata);
        let decimals = fungible_asset::decimals<Metadata>(asset_metadata);

        assert!(name == string::utf8(TEST_NAME), 101);
        assert!(symbol == string::utf8(TEST_SYMBOL), 102);
        assert!(decimals == TEST_DECIMALS, 103);

        let updated_name = string::utf8(b"Updated Name");
        let updated_symbol = string::utf8(b"UPD");
        let updated_decimals = 4;

        update_with_metadata(
            creator,
            option::some(updated_name),
            option::some(updated_symbol),
            option::some(updated_decimals),
            option::none(),
            option::none(),
            asset_metadata
        );

        let name = fungible_asset::name<Metadata>(asset_metadata);
        let symbol = fungible_asset::symbol<Metadata>(asset_metadata);
        let decimals = fungible_asset::decimals<Metadata>(asset_metadata);

        assert!(name == updated_name, 101);
        assert!(symbol == updated_symbol, 102);
        assert!(decimals == updated_decimals, 103);
    }

    #[test(creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57)]
    public fun test_mint_with_authorized_user(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let recipient = @0xface;
        let _creator_address = signer::address_of(creator);

        mint(creator, recipient, 1000, ASSET_SYMBOL);

        let asset_metadata = get_metadata(ASSET_SYMBOL);

        let recipient_balance = primary_fungible_store::balance(
            recipient, asset_metadata
        );

        assert!(recipient_balance == 1000, 201);
    }

    #[
        test(
            creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57,
            aaron = @0xface
        )
    ]
    #[expected_failure(abort_code = 0x50001, location = Self)]
    fun test_mint_with_unauthorized_user(
        creator: &signer, aaron: &signer
    ) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let recipient = @0xface;
        let _creator_address = signer::address_of(creator);

        mint(aaron, recipient, 1000, ASSET_SYMBOL);
    }

    #[test(creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57)]
    public fun test_burn_with_authorized_user(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let recipient = @0xface;

        mint(creator, recipient, 1000, ASSET_SYMBOL);
        burn(creator, recipient, 500, ASSET_SYMBOL);

        let asset_metadata = get_metadata(ASSET_SYMBOL);

        let recipient_balance = primary_fungible_store::balance(
            recipient, asset_metadata
        );

        assert!(recipient_balance == 500, 301);
    }

    #[test(creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57)]
    public fun test_transfer_with_authorized_user(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let sender = @0xface;
        let recipient = @0xdead;

        mint(creator, sender, 1000, ASSET_SYMBOL);
        transfer(creator, sender, recipient, 500, ASSET_SYMBOL);

        let asset_metadata = get_metadata(ASSET_SYMBOL);

        let sender_balance = primary_fungible_store::balance(sender, asset_metadata);
        let recipient_balance = primary_fungible_store::balance(
            recipient, asset_metadata
        );

        assert!(sender_balance == 500, 401);
        assert!(recipient_balance == 500, 402);
    }

    #[
        test(
            creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57,
            aaron = @0xface
        )
    ]
    #[expected_failure(abort_code = 0x50001, location = Self)]
    public fun test_transfer_with_unauthorized_user(
        creator: &signer, aaron: &signer
    ) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let sender = @0xface;
        let recipient = @0xdead;

        // Mint tokens to the sender first
        mint(creator, sender, 1000, ASSET_SYMBOL);

        // Attempt to transfer tokens from sender to recipient with unauthorized user
        transfer(aaron, sender, recipient, 500, ASSET_SYMBOL);
    }

    #[
        test(
            creator = @0x28873b2d4265e6e14bc0739ef876dce858f06380905279ed090b82d0c75f6e57,
            alice = @0x1234,
            bob = @0x5678
        )
    ]
    public fun test_to_relay(
        creator: &signer, alice: &signer, bob: address
    ) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            ASSET_SYMBOL
        );

        let asset = get_metadata(ASSET_SYMBOL);
        let asset_addr = get_metadata_address(ASSET_SYMBOL);

        let asset2 = object::address_to_object<Metadata>(asset_addr);

        assert!(asset2 == asset, 402);
        let alice_addr = signer::address_of(alice);
        mint(creator, alice_addr, 1000, ASSET_SYMBOL);
        primary_fungible_store::ensure_primary_store_exists(bob, asset);

        let alice_balance = primary_fungible_store::balance(alice_addr, asset);

        assert!(alice_balance == 1000, 402);

        let _to_wallet = primary_fungible_store::primary_store(bob, asset);

        // zkgm::relay::tx(alice, bob, 10, asset);

        let fa = primary_fungible_store::withdraw(alice, asset, 10);
        primary_fungible_store::deposit(bob, fa);

        let alice_balance_after = primary_fungible_store::balance(alice_addr, asset);
        let bob_balance = primary_fungible_store::balance(bob, asset);

        assert!(alice_balance_after == 990, 402);
        assert!(bob_balance == 10, 402);
    }
}
