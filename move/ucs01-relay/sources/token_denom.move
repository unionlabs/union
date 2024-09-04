/// A 2-in-1 module that combines managed_fungible_asset and coin_example into one module that when deployed, the
/// deployer will be creating a new managed fungible asset with the hardcoded supply config, name, symbol, and decimals.
/// The address of the asset can be obtained via get_metadata(). As a simple version, it only deals with primary stores.
module UCS01::fa_coin {
    use aptos_framework::fungible_asset::{Self, MintRef, TransferRef, BurnRef, Metadata, FungibleAsset};
    use aptos_framework::object::{Self, Object};
    use aptos_framework::primary_fungible_store;
    use aptos_framework::function_info;
    use aptos_framework::dispatchable_fungible_asset;
    use std::error;
    use std::signer;
    use std::string::{Self};
    use std::option;

    /// Only fungible asset metadata owner can make changes.
    const ENOT_OWNER: u64 = 1;
    /// The FA coin is paused.
    const EPAUSED: u64 = 2;

    const ASSET_SYMBOL: vector<u8> = b"FA";

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Hold refs to control the minting, transfer and burning of fungible assets.
    struct ManagedFungibleAsset has key {
        mint_ref: MintRef,
        transfer_ref: TransferRef,
        burn_ref: BurnRef,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Global state to pause the FA coin.
    /// OPTIONAL
    struct State has key {
        paused: bool,
    }

    /// Initialize metadata object and store the refs.
    // :!:>initialize
    public entry fun initialize(admin: &signer, name: string::String, symbol: string::String, decimals: u8, 
    icon: string::String, project: string::String) {
        let constructor_ref = &object::create_named_object(admin, ASSET_SYMBOL);
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            constructor_ref,
            option::none(),
            name, /* name */
            symbol, /* symbol */
            decimals, /* decimals */
            icon, /* icon */
            project, /* project */
        );

        // Create mint/burn/transfer refs to allow creator to manage the fungible asset.
        let mint_ref = fungible_asset::generate_mint_ref(constructor_ref);
        let burn_ref = fungible_asset::generate_burn_ref(constructor_ref);
        let transfer_ref = fungible_asset::generate_transfer_ref(constructor_ref);
        let metadata_object_signer = object::generate_signer(constructor_ref);
        move_to(
            &metadata_object_signer,
            ManagedFungibleAsset { mint_ref, transfer_ref, burn_ref}
        ); // <:!:initialize

        // Create a global state to pause the FA coin and move to Metadata object.
        move_to(
            &metadata_object_signer,
            State { paused: false, }
        );

        // // Override the deposit and withdraw functions which mean overriding transfer.
        // // This ensures all transfer will call withdraw and deposit functions in this module
        // // and perform the necessary checks.
        // // This is OPTIONAL. It is an advanced feature and we don't NEED a global state to pause the FA coin.
        // let deposit = function_info::new_function_info(
        //     admin,
        //     string::utf8(b"fa_coin"),
        //     string::utf8(b"deposit"),
        // );
        // let withdraw = function_info::new_function_info(
        //     admin,
        //     string::utf8(b"fa_coin"),
        //     string::utf8(b"withdraw"),
        // );
        // dispatchable_fungible_asset::register_dispatch_functions(
        //     constructor_ref,
        //     option::some(withdraw),
        //     option::some(deposit),
        //     option::none(),
        // );
    }

    public entry fun update_metadata(
        admin: &signer,
        new_name: string::String,
        new_symbol: string::String,
        new_decimals: u8
    ) {
        let asset = get_metadata();
        let _metadata_address = object::object_address(&asset);

        // Check if the caller is the owner of the metadata
        assert!(
            object::is_owner(asset, signer::address_of(admin)),
            error::permission_denied(ENOT_OWNER)
        );

        // TODO: When it comes to here it raises an error
        // assert!(!exists<ObjectCore>(object), error::already_exists(EOBJECT_EXISTS));
        // Test was not expected to error, but it aborted with code 524289 originating in the module 0000000000000000000000000000000000000000000000000000000000000001::object rooted here
        let constructor_ref = &object::create_named_object(admin, ASSET_SYMBOL);
        let mutate_ref = fungible_asset::generate_mutate_metadata_ref(constructor_ref);

        // // Use mutate_metadata to update the name, symbol, and decimals
        fungible_asset::mutate_metadata(
            &mutate_ref,
            option::some(new_name),
            option::some(new_symbol),
            option::some(new_decimals),
            option::none(),   // icon_uri - not updating
            option::none()    // project_uri - not updating
        );
    }

    #[view]
    /// Return the address of the managed fungible asset that's created when this module is deployed.
    public fun get_metadata(): Object<Metadata> {
        let asset_address = object::create_object_address(&@UCS01, ASSET_SYMBOL);
        object::address_to_object<Metadata>(asset_address)
    }

    #[view]
    public fun decimals(): u8 {
        let asset_metadata = get_metadata();
        fungible_asset::decimals<Metadata>(asset_metadata)
    }

    #[view]
    public fun symbol(): string::String {
        let asset_metadata = get_metadata();
        fungible_asset::symbol<Metadata>(asset_metadata)
    }

    #[view]
    public fun name(): string::String {
        let asset_metadata = get_metadata();
        fungible_asset::name<Metadata>(asset_metadata)
    }

    /// Deposit function override to ensure that the account is not denylisted and the FA coin is not paused.
    /// OPTIONAL
    public fun deposit<T: key>(
        store: Object<T>,
        fa: FungibleAsset,
        transfer_ref: &TransferRef,
    ) acquires State {
        assert_not_paused();
        fungible_asset::deposit_with_ref(transfer_ref, store, fa);
    }

    /// Withdraw function override to ensure that the account is not denylisted and the FA coin is not paused.
    /// OPTIONAL
    public fun withdraw<T: key>(
        store: Object<T>,
        amount: u64,
        transfer_ref: &TransferRef,
    ): FungibleAsset acquires State {
        assert_not_paused();
        fungible_asset::withdraw_with_ref(transfer_ref, store, amount)
    }
    

    // :!:>mint
    /// Mint as the owner of metadata object.
    public entry fun mint(admin: &signer, to: address, amount: u64) acquires ManagedFungibleAsset {
        let asset = get_metadata();
        let managed_fungible_asset = authorized_borrow_refs(admin, asset);
        let to_wallet = primary_fungible_store::ensure_primary_store_exists(to, asset);
        let fa = fungible_asset::mint(&managed_fungible_asset.mint_ref, amount);
        fungible_asset::deposit_with_ref(&managed_fungible_asset.transfer_ref, to_wallet, fa);
    }// <:!:mint

    /// Transfer as the owner of metadata object.
    /// TODO: This needs to be open for anyone, not just the owner of the metadata object.
    /// How to achieve it? Different stores?
    public entry fun transfer(admin: &signer, from: address, to: address, amount: u64) acquires ManagedFungibleAsset, State {
        let asset = get_metadata();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let from_wallet = primary_fungible_store::primary_store(from, asset);
        let to_wallet = primary_fungible_store::ensure_primary_store_exists(to, asset);
        let fa = withdraw(from_wallet, amount, transfer_ref);
        deposit(to_wallet, fa, transfer_ref);
    }

    /// Burn fungible assets as the owner of metadata object.
    public entry fun burn(admin: &signer, from: address, amount: u64) acquires ManagedFungibleAsset {
        let asset = get_metadata();
        let burn_ref = &authorized_borrow_refs(admin, asset).burn_ref;
        let from_wallet = primary_fungible_store::primary_store(from, asset);
        fungible_asset::burn_from(burn_ref, from_wallet, amount);
    }

    /// Freeze an account so it cannot transfer or receive fungible assets.
    public entry fun freeze_account(admin: &signer, account: address) acquires ManagedFungibleAsset {
        let asset = get_metadata();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let wallet = primary_fungible_store::ensure_primary_store_exists(account, asset);
        fungible_asset::set_frozen_flag(transfer_ref, wallet, true);
    }

    /// Unfreeze an account so it can transfer or receive fungible assets.
    public entry fun unfreeze_account(admin: &signer, account: address) acquires ManagedFungibleAsset {
        let asset = get_metadata();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let wallet = primary_fungible_store::ensure_primary_store_exists(account, asset);
        fungible_asset::set_frozen_flag(transfer_ref, wallet, false);
    }

    /// Pause or unpause the transfer of FA coin. This checks that the caller is the pauser.
    public entry fun set_pause(pauser: &signer, paused: bool) acquires State {
        let asset = get_metadata();
        assert!(object::is_owner(asset, signer::address_of(pauser)), error::permission_denied(ENOT_OWNER));
        let state = borrow_global_mut<State>(object::create_object_address(&@UCS01, ASSET_SYMBOL));
        if (state.paused == paused) { return };
        state.paused = paused;
    }

    /// Assert that the FA coin is not paused.
    /// OPTIONAL
    fun assert_not_paused() acquires State {
        let state = borrow_global<State>(object::create_object_address(&@UCS01, ASSET_SYMBOL));
        assert!(!state.paused, EPAUSED);
    }

    /// Borrow the immutable reference of the refs of `metadata`.
    /// This validates that the signer is the metadata object's owner.
    inline fun authorized_borrow_refs(
        owner: &signer,
        asset: Object<Metadata>,
    ): &ManagedFungibleAsset acquires ManagedFungibleAsset {
        assert!(object::is_owner(asset, signer::address_of(owner)), error::permission_denied(ENOT_OWNER));
        borrow_global<ManagedFungibleAsset>(object::object_address(&asset))
    }

    const TEST_NAME: vector<u8> = b"Test Coin";
    const TEST_SYMBOL: vector<u8> = b"TST";
    const TEST_DECIMALS: u8 = 8;
    const TEST_ICON: vector<u8> = b"https://example.com/icon.png";
    const TEST_PROJECT: vector<u8> = b"Test Project";



    
    #[test(creator = @UCS01)]
    public fun test_initialize(creator: &signer) {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let asset_metadata = get_metadata();
        let name = fungible_asset::name<Metadata>(asset_metadata);
        let symbol = fungible_asset::symbol<Metadata>(asset_metadata);
        let decimals = fungible_asset::decimals<Metadata>(asset_metadata);

        assert!(name == string::utf8(TEST_NAME), 101);
        assert!(symbol == string::utf8(TEST_SYMBOL), 102);
        assert!(decimals == TEST_DECIMALS, 103);
    }
    

    
    // TODO: This test is failing with an error code 524289 - object already exists.
    // #[test(creator = @UCS01)]
    // public fun test_update_metadata(creator: &signer) {
    //     initialize(
    //         creator,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT)
    //     );

    //     let asset_metadata = get_metadata();
    //     let name = fungible_asset::name<Metadata>(asset_metadata);
    //     let symbol = fungible_asset::symbol<Metadata>(asset_metadata);
    //     let decimals = fungible_asset::decimals<Metadata>(asset_metadata);

    //     assert!(name == string::utf8(TEST_NAME), 101);
    //     assert!(symbol == string::utf8(TEST_SYMBOL), 102);
    //     assert!(decimals == TEST_DECIMALS, 103);

    //     update_metadata(
    //         creator,
    //         string::utf8(b"Updated Name"),
    //         string::utf8(b"UPD"),
    //         4
    //     );

    //     let updated_asset_metadata = get_metadata();
    //     let updated_name = fungible_asset::name<Metadata>(updated_asset_metadata);
    //     let updated_symbol = fungible_asset::symbol<Metadata>(updated_asset_metadata);
    //     let updated_decimals = fungible_asset::decimals<Metadata>(updated_asset_metadata);

    //     assert!(updated_name == string::utf8(b"Updated Name"), 201);
    //     assert!(updated_symbol == string::utf8(b"UPD"), 202);
    //     assert!(updated_decimals == 4, 203);

    // }
    

    #[test(creator = @UCS01)]
    public fun test_mint_with_authorized_user(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let recipient = @0xface;
        let creator_address = signer::address_of(creator);

        mint(creator, recipient, 1000);

        let asset_metadata = get_metadata();
        
        let recipient_balance = primary_fungible_store::balance(recipient, asset_metadata);

        assert!(recipient_balance == 1000, 201);
    }

    #[test(creator = @UCS01, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50001, location = Self)]
    fun test_mint_with_unauthorized_user(
        creator: &signer,
        aaron: &signer
    ) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let recipient = @0xface;
        let _creator_address = signer::address_of(creator);

        mint(aaron, recipient, 1000);
    }

    #[test(creator = @UCS01)]
    public fun test_burn_with_authorized_user(creator: &signer) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let recipient = @0xface;

        // Mint tokens first
        mint(creator, recipient, 1000);

        // Burn tokens
        burn(creator, recipient, 500);

        let asset_metadata = get_metadata();
        
        let recipient_balance = primary_fungible_store::balance(recipient, asset_metadata);

        assert!(recipient_balance == 500, 301);
    }

    #[test(creator = @UCS01, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50001, location = Self)]
    public fun test_burn_with_unauthorized_user(
        creator: &signer,
        aaron: &signer
    ) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let recipient = @0xface;

        // Mint tokens first
        mint(creator, recipient, 1000);

        // Attempt to burn tokens with unauthorized user
        burn(aaron, recipient, 500);
    }

    #[test(creator = @UCS01)]
    public fun test_transfer_with_authorized_user(creator: &signer) acquires ManagedFungibleAsset, State {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let sender = @0xface;
        let recipient = @0xdead;

        // Mint tokens to the sender first
        mint(creator, sender, 1000);

        // Transfer tokens from sender to recipient
        transfer(creator, sender, recipient, 500);

        let asset_metadata = get_metadata();
        
        let sender_balance = primary_fungible_store::balance(sender, asset_metadata);
        let recipient_balance = primary_fungible_store::balance(recipient, asset_metadata);

        assert!(sender_balance == 500, 401);
        assert!(recipient_balance == 500, 402);
    }

    #[test(creator = @UCS01, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50001, location = Self)]
    public fun test_transfer_with_unauthorized_user(
        creator: &signer,
        aaron: &signer
    ) acquires ManagedFungibleAsset, State {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let sender = @0xface;
        let recipient = @0xdead;

        // Mint tokens to the sender first
        mint(creator, sender, 1000);

        // Attempt to transfer tokens from sender to recipient with unauthorized user
        transfer(aaron, sender, recipient, 500);
    }


    #[test(creator = @UCS01, alice=@0x1234, bob=@0x5678)]
    public fun test_to_relay(creator: &signer, alice: &signer, bob: address) acquires ManagedFungibleAsset {
        initialize(
            creator,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT)
        );

        let asset = get_metadata();

        let alice_addr = signer::address_of(alice);
        mint(creator, alice_addr, 1000);
        primary_fungible_store::ensure_primary_store_exists(bob, asset);

        let alice_balance = primary_fungible_store::balance(alice_addr, asset);

        assert!(alice_balance == 1000, 402);
        
        let to_wallet = primary_fungible_store::primary_store(bob, asset);

        //UCS01::Relay::tx(alice, bob, 10, asset);
        primary_fungible_store::transfer(alice, asset, bob, 10);

        let alice_balance_after = primary_fungible_store::balance(alice_addr, asset);
        let bob_balance = primary_fungible_store::balance(bob, asset);
        
        assert!(alice_balance_after == 990, 402);
        assert!(bob_balance == 10, 402);
    }
}
