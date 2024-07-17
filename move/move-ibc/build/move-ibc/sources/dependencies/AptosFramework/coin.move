/// This module provides the foundation for typesafe Coins.
module aptos_framework::coin {
    use std::error;
    use std::features;
    use std::option::{Self, Option};
    use std::signer;
    use std::string::{Self, String};
    use aptos_std::table::{Self, Table};

    use aptos_framework::account;
    use aptos_framework::aggregator_factory;
    use aptos_framework::aggregator::{Self, Aggregator};
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::guid;
    use aptos_framework::optional_aggregator::{Self, OptionalAggregator};
    use aptos_framework::system_addresses;

    use aptos_framework::fungible_asset::{Self, FungibleAsset, Metadata, MintRef, TransferRef, BurnRef};
    use aptos_framework::object::{Self, Object, object_address};
    use aptos_framework::primary_fungible_store;
    use aptos_std::type_info::{Self, TypeInfo, type_name};
    use aptos_framework::create_signer;

    friend aptos_framework::aptos_coin;
    friend aptos_framework::genesis;
    friend aptos_framework::transaction_fee;

    //
    // Errors.
    //

    /// Address of account which is used to initialize a coin `CoinType` doesn't match the deployer of module
    const ECOIN_INFO_ADDRESS_MISMATCH: u64 = 1;

    /// `CoinType` is already initialized as a coin
    const ECOIN_INFO_ALREADY_PUBLISHED: u64 = 2;

    /// `CoinType` hasn't been initialized as a coin
    const ECOIN_INFO_NOT_PUBLISHED: u64 = 3;

    /// Deprecated. Account already has `CoinStore` registered for `CoinType`
    const ECOIN_STORE_ALREADY_PUBLISHED: u64 = 4;

    /// Account hasn't registered `CoinStore` for `CoinType`
    const ECOIN_STORE_NOT_PUBLISHED: u64 = 5;

    /// Not enough coins to complete transaction
    const EINSUFFICIENT_BALANCE: u64 = 6;

    /// Cannot destroy non-zero coins
    const EDESTRUCTION_OF_NONZERO_TOKEN: u64 = 7;

    /// CoinStore is frozen. Coins cannot be deposited or withdrawn
    const EFROZEN: u64 = 10;

    /// Cannot upgrade the total supply of coins to different implementation.
    const ECOIN_SUPPLY_UPGRADE_NOT_SUPPORTED: u64 = 11;

    /// Name of the coin is too long
    const ECOIN_NAME_TOO_LONG: u64 = 12;

    /// Symbol of the coin is too long
    const ECOIN_SYMBOL_TOO_LONG: u64 = 13;

    /// The value of aggregatable coin used for transaction fees redistribution does not fit in u64.
    const EAGGREGATABLE_COIN_VALUE_TOO_LARGE: u64 = 14;

    /// Error regarding paired coin type of the fungible asset metadata.
    const EPAIRED_COIN: u64 = 15;

    /// Error regarding paired fungible asset metadata of a coin type.
    const EPAIRED_FUNGIBLE_ASSET: u64 = 16;

    /// The coin type from the map does not match the calling function type argument.
    const ECOIN_TYPE_MISMATCH: u64 = 17;

    /// The feature of migration from coin to fungible asset is not enabled.
    const ECOIN_TO_FUNGIBLE_ASSET_FEATURE_NOT_ENABLED: u64 = 18;

    /// PairedFungibleAssetRefs resource does not exist.
    const EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND: u64 = 19;

    /// The MintRefReceipt does not match the MintRef to be returned.
    const EMINT_REF_RECEIPT_MISMATCH: u64 = 20;

    /// The MintRef does not exist.
    const EMINT_REF_NOT_FOUND: u64 = 21;

    /// The TransferRefReceipt does not match the TransferRef to be returned.
    const ETRANSFER_REF_RECEIPT_MISMATCH: u64 = 22;

    /// The TransferRef does not exist.
    const ETRANSFER_REF_NOT_FOUND: u64 = 23;

    /// The BurnRefReceipt does not match the BurnRef to be returned.
    const EBURN_REF_RECEIPT_MISMATCH: u64 = 24;

    /// The BurnRef does not exist.
    const EBURN_REF_NOT_FOUND: u64 = 25;

    /// The migration process from coin to fungible asset is not enabled yet.
    const EMIGRATION_FRAMEWORK_NOT_ENABLED: u64 = 26;

    /// The coin converison map is not created yet.
    const ECOIN_CONVERSION_MAP_NOT_FOUND: u64 = 27;

    /// APT pairing is not eanbled yet.
    const EAPT_PAIRING_IS_NOT_ENABLED: u64 = 28;

    //
    // Constants
    //

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    /// Core data structures

    /// Main structure representing a coin/token in an account's custody.
    struct Coin<phantom CoinType> has store {
        /// Amount of coin this address has.
        value: u64,
    }

    /// Represents a coin with aggregator as its value. This allows to update
    /// the coin in every transaction avoiding read-modify-write conflicts. Only
    /// used for gas fees distribution by Aptos Framework (0x1).
    struct AggregatableCoin<phantom CoinType> has store {
        /// Amount of aggregatable coin this address has.
        value: Aggregator,
    }

    /// Maximum possible aggregatable coin value.
    const MAX_U64: u128 = 18446744073709551615;

    /// A holder of a specific coin types and associated event handles.
    /// These are kept in a single resource to ensure locality of data.
    struct CoinStore<phantom CoinType> has key {
        coin: Coin<CoinType>,
        frozen: bool,
        deposit_events: EventHandle<DepositEvent>,
        withdraw_events: EventHandle<WithdrawEvent>,
    }

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    /// Configuration that controls the behavior of total coin supply. If the field
    /// is set, coin creators are allowed to upgrade to parallelizable implementations.
    struct SupplyConfig has key {
        allow_upgrades: bool,
    }

    /// Information about a specific coin type. Stored on the creator of the coin's account.
    struct CoinInfo<phantom CoinType> has key {
        name: String,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: String,
        /// Number of decimals used to get its user representation.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        /// Amount of this coin type in existence.
        supply: Option<OptionalAggregator>,
    }


    #[event]
    /// Module event emitted when some amount of a coin is deposited into an account.
    struct CoinDeposit has drop, store {
        coin_type: String,
        account: address,
        amount: u64,
    }

    #[event]
    /// Module event emitted when some amount of a coin is withdrawn from an account.
    struct CoinWithdraw has drop, store {
        coin_type: String,
        account: address,
        amount: u64,
    }

    // DEPRECATED, NEVER USED
    #[deprecated]
    #[event]
    struct Deposit<phantom CoinType> has drop, store {
        account: address,
        amount: u64,
    }

    // DEPRECATED, NEVER USED
    #[deprecated]
    #[event]
    struct Withdraw<phantom CoinType> has drop, store {
        account: address,
        amount: u64,
    }

    /// Event emitted when some amount of a coin is deposited into an account.
    struct DepositEvent has drop, store {
        amount: u64,
    }

    /// Event emitted when some amount of a coin is withdrawn from an account.
    struct WithdrawEvent has drop, store {
        amount: u64,
    }


    #[event]
    /// Module event emitted when the event handles related to coin store is deleted.
    struct CoinEventHandleDeletion has drop, store {
        event_handle_creation_address: address,
        deleted_deposit_event_handle_creation_number: u64,
        deleted_withdraw_event_handle_creation_number: u64,
    }

    #[event]
    /// Module event emitted when a new pair of coin and fungible asset is created.
    struct PairCreation has drop, store {
        coin_type: TypeInfo,
        fungible_asset_metadata_address: address,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// The flag the existence of which indicates the primary fungible store is created by the migration from CoinStore.
    struct MigrationFlag has key {}

    /// Capability required to mint coins.
    struct MintCapability<phantom CoinType> has copy, store {}

    /// Capability required to freeze a coin store.
    struct FreezeCapability<phantom CoinType> has copy, store {}

    /// Capability required to burn coins.
    struct BurnCapability<phantom CoinType> has copy, store {}

    /// The mapping between coin and fungible asset.
    struct CoinConversionMap has key {
        coin_to_fungible_asset_map: Table<TypeInfo, Object<Metadata>>,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// The paired coin type info stored in fungible asset metadata object.
    struct PairedCoinType has key {
        type: TypeInfo,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// The refs of the paired fungible asset.
    struct PairedFungibleAssetRefs has key {
        mint_ref_opt: Option<MintRef>,
        transfer_ref_opt: Option<TransferRef>,
        burn_ref_opt: Option<BurnRef>,
    }

    /// The hot potato receipt for flash borrowing MintRef.
    struct MintRefReceipt {
        metadata: Object<Metadata>,
    }

    /// The hot potato receipt for flash borrowing TransferRef.
    struct TransferRefReceipt {
        metadata: Object<Metadata>,
    }

    /// The hot potato receipt for flash borrowing BurnRef.
    struct BurnRefReceipt {
        metadata: Object<Metadata>,
    }

    #[view]
    /// Get the paired fungible asset metadata object of a coin type. If not exist, return option::none().
    public fun paired_metadata<CoinType>(): Option<Object<Metadata>> acquires CoinConversionMap {
        if (exists<CoinConversionMap>(@aptos_framework) && features::coin_to_fungible_asset_migration_feature_enabled(
        )) {
            let map = &borrow_global<CoinConversionMap>(@aptos_framework).coin_to_fungible_asset_map;
            let type = type_info::type_of<CoinType>();
            if (table::contains(map, type)) {
                return option::some(*table::borrow(map, type))
            }
        };
        option::none()
    }

    public entry fun create_coin_conversion_map(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);
        if (!exists<CoinConversionMap>(@aptos_framework)) {
            move_to(aptos_framework, CoinConversionMap {
                coin_to_fungible_asset_map: table::new(),
            })
        };
    }

    /// Create APT pairing by passing `AptosCoin`.
    public entry fun create_pairing<CoinType>(
        aptos_framework: &signer
    ) acquires CoinConversionMap, CoinInfo {
        system_addresses::assert_aptos_framework(aptos_framework);
        create_and_return_paired_metadata_if_not_exist<CoinType>(true);
    }

    inline fun is_apt<CoinType>(): bool {
        type_info::type_name<CoinType>() == string::utf8(b"0x1::aptos_coin::AptosCoin")
    }

    inline fun create_and_return_paired_metadata_if_not_exist<CoinType>(allow_apt_creation: bool): Object<Metadata> {
        assert!(
            features::coin_to_fungible_asset_migration_feature_enabled(),
            error::invalid_state(EMIGRATION_FRAMEWORK_NOT_ENABLED)
        );
        assert!(exists<CoinConversionMap>(@aptos_framework), error::not_found(ECOIN_CONVERSION_MAP_NOT_FOUND));
        let map = borrow_global_mut<CoinConversionMap>(@aptos_framework);
        let type = type_info::type_of<CoinType>();
        if (!table::contains(&map.coin_to_fungible_asset_map, type)) {
            let is_apt = is_apt<CoinType>();
            assert!(!is_apt || allow_apt_creation, error::invalid_state(EAPT_PAIRING_IS_NOT_ENABLED));
            let metadata_object_cref =
                if (is_apt) {
                    object::create_sticky_object_at_address(@aptos_framework, @aptos_fungible_asset)
                } else {
                    object::create_named_object(
                        &create_signer::create_signer(@aptos_fungible_asset),
                        *string::bytes(&type_info::type_name<CoinType>())
                    )
                };
            primary_fungible_store::create_primary_store_enabled_fungible_asset(
                &metadata_object_cref,
                option::none(),
                name<CoinType>(),
                symbol<CoinType>(),
                decimals<CoinType>(),
                string::utf8(b""),
                string::utf8(b""),
            );

            let metadata_object_signer = &object::generate_signer(&metadata_object_cref);
            let type = type_info::type_of<CoinType>();
            move_to(metadata_object_signer, PairedCoinType { type });
            let metadata_obj = object::object_from_constructor_ref(&metadata_object_cref);

            table::add(&mut map.coin_to_fungible_asset_map, type, metadata_obj);
            event::emit(PairCreation {
                coin_type: type,
                fungible_asset_metadata_address: object_address(&metadata_obj)
            });

            // Generates all three refs
            let mint_ref = fungible_asset::generate_mint_ref(&metadata_object_cref);
            let transfer_ref = fungible_asset::generate_transfer_ref(&metadata_object_cref);
            let burn_ref = fungible_asset::generate_burn_ref(&metadata_object_cref);
            move_to(metadata_object_signer,
                PairedFungibleAssetRefs {
                    mint_ref_opt: option::some(mint_ref),
                    transfer_ref_opt: option::some(transfer_ref),
                    burn_ref_opt: option::some(burn_ref),
                }
            );
        };
        *table::borrow(&map.coin_to_fungible_asset_map, type)
    }

    /// Get the paired fungible asset metadata object of a coin type, create if not exist.
    public(friend) fun ensure_paired_metadata<CoinType>(): Object<Metadata> acquires CoinConversionMap, CoinInfo {
        create_and_return_paired_metadata_if_not_exist<CoinType>(false)
    }

    #[view]
    /// Get the paired coin type of a fungible asset metadata object.
    public fun paired_coin(metadata: Object<Metadata>): Option<TypeInfo> acquires PairedCoinType {
        let metadata_addr = object::object_address(&metadata);
        if (exists<PairedCoinType>(metadata_addr)) {
            option::some(borrow_global<PairedCoinType>(metadata_addr).type)
        } else {
            option::none()
        }
    }

    /// Conversion from coin to fungible asset
    public fun coin_to_fungible_asset<CoinType>(
        coin: Coin<CoinType>
    ): FungibleAsset acquires CoinConversionMap, CoinInfo {
        let metadata = ensure_paired_metadata<CoinType>();
        let amount = burn_internal(coin);
        fungible_asset::mint_internal(metadata, amount)
    }

    /// Conversion from fungible asset to coin. Not public to push the migration to FA.
    fun fungible_asset_to_coin<CoinType>(
        fungible_asset: FungibleAsset
    ): Coin<CoinType> acquires CoinInfo, PairedCoinType {
        let metadata_addr = object::object_address(&fungible_asset::metadata_from_asset(&fungible_asset));
        assert!(
            object::object_exists<PairedCoinType>(metadata_addr),
            error::not_found(EPAIRED_COIN)
        );
        let coin_type_info = borrow_global<PairedCoinType>(metadata_addr).type;
        assert!(coin_type_info == type_info::type_of<CoinType>(), error::invalid_argument(ECOIN_TYPE_MISMATCH));
        let amount = fungible_asset::burn_internal(fungible_asset);
        mint_internal<CoinType>(amount)
    }

    inline fun assert_paired_metadata_exists<CoinType>(): Object<Metadata> {
        let metadata_opt = paired_metadata<CoinType>();
        assert!(option::is_some(&metadata_opt), error::not_found(EPAIRED_FUNGIBLE_ASSET));
        option::destroy_some(metadata_opt)
    }

    #[view]
    /// Check whether `MintRef` has not been taken.
    public fun paired_mint_ref_exists<CoinType>(): bool acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        option::is_some(&borrow_global<PairedFungibleAssetRefs>(metadata_addr).mint_ref_opt)
    }

    /// Get the `MintRef` of paired fungible asset of a coin type from `MintCapability`.
    public fun get_paired_mint_ref<CoinType>(
        _: &MintCapability<CoinType>
    ): (MintRef, MintRefReceipt) acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        let mint_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).mint_ref_opt;
        assert!(option::is_some(mint_ref_opt), error::not_found(EMINT_REF_NOT_FOUND));
        (option::extract(mint_ref_opt), MintRefReceipt { metadata })
    }

    /// Return the `MintRef` with the hot potato receipt.
    public fun return_paired_mint_ref(mint_ref: MintRef, receipt: MintRefReceipt) acquires PairedFungibleAssetRefs {
        let MintRefReceipt { metadata } = receipt;
        assert!(
            fungible_asset::mint_ref_metadata(&mint_ref) == metadata,
            error::invalid_argument(EMINT_REF_RECEIPT_MISMATCH)
        );
        let metadata_addr = object_address(&metadata);
        let mint_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).mint_ref_opt;
        option::fill(mint_ref_opt, mint_ref);
    }

    #[view]
    /// Check whether `TransferRef` still exists.
    public fun paired_transfer_ref_exists<CoinType>(): bool acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        option::is_some(&borrow_global<PairedFungibleAssetRefs>(metadata_addr).transfer_ref_opt)
    }

    /// Get the TransferRef of paired fungible asset of a coin type from `FreezeCapability`.
    public fun get_paired_transfer_ref<CoinType>(
        _: &FreezeCapability<CoinType>
    ): (TransferRef, TransferRefReceipt) acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        let transfer_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).transfer_ref_opt;
        assert!(option::is_some(transfer_ref_opt), error::not_found(ETRANSFER_REF_NOT_FOUND));
        (option::extract(transfer_ref_opt), TransferRefReceipt { metadata })
    }

    /// Return the `TransferRef` with the hot potato receipt.
    public fun return_paired_transfer_ref(
        transfer_ref: TransferRef,
        receipt: TransferRefReceipt
    ) acquires PairedFungibleAssetRefs {
        let TransferRefReceipt { metadata } = receipt;
        assert!(
            fungible_asset::transfer_ref_metadata(&transfer_ref) == metadata,
            error::invalid_argument(ETRANSFER_REF_RECEIPT_MISMATCH)
        );
        let metadata_addr = object_address(&metadata);
        let transfer_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).transfer_ref_opt;
        option::fill(transfer_ref_opt, transfer_ref);
    }

    #[view]
    /// Check whether `BurnRef` has not been taken.
    public fun paired_burn_ref_exists<CoinType>(): bool acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        option::is_some(&borrow_global<PairedFungibleAssetRefs>(metadata_addr).burn_ref_opt)
    }

    /// Get the `BurnRef` of paired fungible asset of a coin type from `BurnCapability`.
    public fun get_paired_burn_ref<CoinType>(
        _: &BurnCapability<CoinType>
    ): (BurnRef, BurnRefReceipt) acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        let burn_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).burn_ref_opt;
        assert!(option::is_some(burn_ref_opt), error::not_found(EBURN_REF_NOT_FOUND));
        (option::extract(burn_ref_opt), BurnRefReceipt { metadata })
    }

    // Permanently convert to BurnRef, and take it from the pairing.
    // (i.e. future calls to borrow/convert BurnRef will fail)
    public fun convert_and_take_paired_burn_ref<CoinType>(
        burn_cap: BurnCapability<CoinType>
    ): BurnRef acquires CoinConversionMap, PairedFungibleAssetRefs {
        destroy_burn_cap(burn_cap);
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        let burn_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).burn_ref_opt;
        assert!(option::is_some(burn_ref_opt), error::not_found(EBURN_REF_NOT_FOUND));
        option::extract(burn_ref_opt)
    }

    /// Return the `BurnRef` with the hot potato receipt.
    public fun return_paired_burn_ref(
        burn_ref: BurnRef,
        receipt: BurnRefReceipt
    ) acquires PairedFungibleAssetRefs {
        let BurnRefReceipt { metadata } = receipt;
        assert!(
            fungible_asset::burn_ref_metadata(&burn_ref) == metadata,
            error::invalid_argument(EBURN_REF_RECEIPT_MISMATCH)
        );
        let metadata_addr = object_address(&metadata);
        let burn_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).burn_ref_opt;
        option::fill(burn_ref_opt, burn_ref);
    }

    inline fun borrow_paired_burn_ref<CoinType>(
        _: &BurnCapability<CoinType>
    ): &BurnRef acquires CoinConversionMap, PairedFungibleAssetRefs {
        let metadata = assert_paired_metadata_exists<CoinType>();
        let metadata_addr = object_address(&metadata);
        assert!(exists<PairedFungibleAssetRefs>(metadata_addr), error::internal(EPAIRED_FUNGIBLE_ASSET_REFS_NOT_FOUND));
        let burn_ref_opt = &mut borrow_global_mut<PairedFungibleAssetRefs>(metadata_addr).burn_ref_opt;
        assert!(option::is_some(burn_ref_opt), error::not_found(EBURN_REF_NOT_FOUND));
        option::borrow(burn_ref_opt)
    }

    //
    // Total supply config
    //

    /// Publishes supply configuration. Initially, upgrading is not allowed.
    public(friend) fun initialize_supply_config(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);
        move_to(aptos_framework, SupplyConfig { allow_upgrades: false });
    }

    /// This should be called by on-chain governance to update the config and allow
    /// or disallow upgradability of total supply.
    public fun allow_supply_upgrades(aptos_framework: &signer, allowed: bool) acquires SupplyConfig {
        system_addresses::assert_aptos_framework(aptos_framework);
        let allow_upgrades = &mut borrow_global_mut<SupplyConfig>(@aptos_framework).allow_upgrades;
        *allow_upgrades = allowed;
    }

    //
    //  Aggregatable coin functions
    //

    /// Creates a new aggregatable coin with value overflowing on `limit`. Note that this function can
    /// only be called by Aptos Framework (0x1) account for now because of `create_aggregator`.
    public(friend) fun initialize_aggregatable_coin<CoinType>(aptos_framework: &signer): AggregatableCoin<CoinType> {
        let aggregator = aggregator_factory::create_aggregator(aptos_framework, MAX_U64);
        AggregatableCoin<CoinType> {
            value: aggregator,
        }
    }

    /// Returns true if the value of aggregatable coin is zero.
    public(friend) fun is_aggregatable_coin_zero<CoinType>(coin: &AggregatableCoin<CoinType>): bool {
        let amount = aggregator::read(&coin.value);
        amount == 0
    }

    /// Drains the aggregatable coin, setting it to zero and returning a standard coin.
    public(friend) fun drain_aggregatable_coin<CoinType>(coin: &mut AggregatableCoin<CoinType>): Coin<CoinType> {
        spec {
            // TODO: The data invariant is not properly assumed from CollectedFeesPerBlock.
            assume aggregator::spec_get_limit(coin.value) == MAX_U64;
        };
        let amount = aggregator::read(&coin.value);
        assert!(amount <= MAX_U64, error::out_of_range(EAGGREGATABLE_COIN_VALUE_TOO_LARGE));
        spec {
            update aggregate_supply<CoinType> = aggregate_supply<CoinType> - amount;
        };
        aggregator::sub(&mut coin.value, amount);
        spec {
            update supply<CoinType> = supply<CoinType> + amount;
        };
        Coin<CoinType> {
            value: (amount as u64),
        }
    }

    /// Merges `coin` into aggregatable coin (`dst_coin`).
    public(friend) fun merge_aggregatable_coin<CoinType>(
        dst_coin: &mut AggregatableCoin<CoinType>,
        coin: Coin<CoinType>
    ) {
        spec {
            update supply<CoinType> = supply<CoinType> - coin.value;
        };
        let Coin { value } = coin;
        let amount = (value as u128);
        spec {
            update aggregate_supply<CoinType> = aggregate_supply<CoinType> + amount;
        };
        aggregator::add(&mut dst_coin.value, amount);
    }

    /// Collects a specified amount of coin form an account into aggregatable coin.
    public(friend) fun collect_into_aggregatable_coin<CoinType>(
        account_addr: address,
        amount: u64,
        dst_coin: &mut AggregatableCoin<CoinType>,
    ) acquires CoinStore, CoinConversionMap, CoinInfo, PairedCoinType {
        // Skip collecting if amount is zero.
        if (amount == 0) {
            return
        };

        let (coin_amount_to_collect, fa_amount_to_collect) = calculate_amount_to_withdraw<CoinType>(
            account_addr,
            amount
        );
        let coin = if (coin_amount_to_collect > 0) {
            let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
            extract(&mut coin_store.coin, coin_amount_to_collect)
        } else {
            zero()
        };
        if (fa_amount_to_collect > 0) {
            let store_addr = primary_fungible_store::primary_store_address(
                account_addr,
                option::destroy_some(paired_metadata<CoinType>())
            );
            let fa = fungible_asset::withdraw_internal(store_addr, fa_amount_to_collect);
            merge(&mut coin, fungible_asset_to_coin<CoinType>(fa));
        };
        merge_aggregatable_coin(dst_coin, coin);
    }

    inline fun calculate_amount_to_withdraw<CoinType>(
        account_addr: address,
        amount: u64
    ): (u64, u64) {
        let coin_balance = coin_balance<CoinType>(account_addr);
        if (coin_balance >= amount) {
            (amount, 0)
        } else {
            let metadata = paired_metadata<CoinType>();
            if (option::is_some(&metadata) && primary_fungible_store::primary_store_exists(
                account_addr,
                option::destroy_some(metadata)
            ))
                (coin_balance, amount - coin_balance)
            else
                abort error::invalid_argument(EINSUFFICIENT_BALANCE)
        }
    }

    fun maybe_convert_to_fungible_store<CoinType>(account: address) acquires CoinStore, CoinConversionMap, CoinInfo {
        if (!features::coin_to_fungible_asset_migration_feature_enabled()) {
            abort error::unavailable(ECOIN_TO_FUNGIBLE_ASSET_FEATURE_NOT_ENABLED)
        };
        assert!(is_coin_initialized<CoinType>(), error::invalid_argument(ECOIN_INFO_NOT_PUBLISHED));

        let metadata = ensure_paired_metadata<CoinType>();
        let store = primary_fungible_store::ensure_primary_store_exists(account, metadata);
        let store_address = object::object_address(&store);
        if (exists<CoinStore<CoinType>>(account)) {
            let CoinStore<CoinType> { coin, frozen, deposit_events, withdraw_events } = move_from<CoinStore<CoinType>>(
                account
            );
            event::emit(
                CoinEventHandleDeletion {
                    event_handle_creation_address: guid::creator_address(
                        event::guid(&deposit_events)
                    ),
                    deleted_deposit_event_handle_creation_number: guid::creation_num(event::guid(&deposit_events)),
                    deleted_withdraw_event_handle_creation_number: guid::creation_num(event::guid(&withdraw_events))
                }
            );
            event::destroy_handle(deposit_events);
            event::destroy_handle(withdraw_events);
            if (coin.value == 0) {
                destroy_zero(coin);
            } else {
                fungible_asset::deposit(store, coin_to_fungible_asset(coin));
            };
            // Note:
            // It is possible the primary fungible store may already exist before this function call.
            // In this case, if the account owns a frozen CoinStore and an unfrozen primary fungible store, this
            // function would convert and deposit the rest coin into the primary store and freeze it to make the
            // `frozen` semantic as consistent as possible.
            if (frozen != fungible_asset::is_frozen(store)) {
                fungible_asset::set_frozen_flag_internal(store, frozen);
            }
        };
        if (!exists<MigrationFlag>(store_address)) {
            move_to(&create_signer::create_signer(store_address), MigrationFlag {});
        }
    }

    /// Voluntarily migrate to fungible store for `CoinType` if not yet.
    public entry fun migrate_to_fungible_store<CoinType>(
        account: &signer
    ) acquires CoinStore, CoinConversionMap, CoinInfo {
        maybe_convert_to_fungible_store<CoinType>(signer::address_of(account));
    }

    //
    // Getter functions
    //

    /// A helper function that returns the address of CoinType.
    fun coin_address<CoinType>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }

    #[view]
    /// Returns the balance of `owner` for provided `CoinType` and its paired FA if exists.
    public fun balance<CoinType>(owner: address): u64 acquires CoinConversionMap, CoinStore {
        let paired_metadata = paired_metadata<CoinType>();
        coin_balance<CoinType>(owner) + if (option::is_some(&paired_metadata)) {
            primary_fungible_store::balance(
                owner,
                option::extract(&mut paired_metadata)
            )
        } else { 0 }
    }

    #[view]
    /// Returns whether the balance of `owner` for provided `CoinType` and its paired FA is >= `amount`.
    public fun is_balance_at_least<CoinType>(owner: address, amount: u64): bool acquires CoinConversionMap, CoinStore {
        let coin_balance = coin_balance<CoinType>(owner);
        if (coin_balance >= amount) {
            return true
        };

        let paired_metadata = paired_metadata<CoinType>();
        let left_amount = amount - coin_balance;
        if (option::is_some(&paired_metadata)) {
            primary_fungible_store::is_balance_at_least(
                owner,
                option::extract(&mut paired_metadata),
                left_amount
            )
        } else { false }
    }

    inline fun coin_balance<CoinType>(owner: address): u64 {
        if (exists<CoinStore<CoinType>>(owner)) {
            borrow_global<CoinStore<CoinType>>(owner).coin.value
        } else {
            0
        }
    }

    #[view]
    /// Returns `true` if the type `CoinType` is an initialized coin.
    public fun is_coin_initialized<CoinType>(): bool {
        exists<CoinInfo<CoinType>>(coin_address<CoinType>())
    }

    #[view]
    /// Returns `true` is account_addr has frozen the CoinStore or if it's not registered at all
    public fun is_coin_store_frozen<CoinType>(
        account_addr: address
    ): bool acquires CoinStore, CoinConversionMap {
        if (!is_account_registered<CoinType>(account_addr)) {
            return true
        };

        let coin_store = borrow_global<CoinStore<CoinType>>(account_addr);
        coin_store.frozen
    }

    #[view]
    /// Returns `true` if `account_addr` is registered to receive `CoinType`.
    public fun is_account_registered<CoinType>(account_addr: address): bool acquires CoinConversionMap {
        assert!(is_coin_initialized<CoinType>(), error::invalid_argument(ECOIN_INFO_NOT_PUBLISHED));
        if (exists<CoinStore<CoinType>>(account_addr)) {
            true
        } else {
            let paired_metadata_opt = paired_metadata<CoinType>();
            (option::is_some(
                &paired_metadata_opt
            ) && migrated_primary_fungible_store_exists(account_addr, option::destroy_some(paired_metadata_opt)))
        }
    }

    #[view]
    /// Returns the name of the coin.
    public fun name<CoinType>(): string::String acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).name
    }

    #[view]
    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType>(): string::String acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).symbol
    }

    #[view]
    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType>(): u8 acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).decimals
    }

    #[view]
    /// Returns the amount of coin in existence.
    public fun supply<CoinType>(): Option<u128> acquires CoinInfo, CoinConversionMap {
        let coin_supply = coin_supply<CoinType>();
        let metadata = paired_metadata<CoinType>();
        if (option::is_some(&metadata)) {
            let fungible_asset_supply = fungible_asset::supply(option::extract(&mut metadata));
            if (option::is_some(&coin_supply)) {
                let supply = option::borrow_mut(&mut coin_supply);
                *supply = *supply + option::destroy_some(fungible_asset_supply);
            };
        };
        coin_supply
    }

    #[view]
    /// Returns the amount of coin in existence.
    public fun coin_supply<CoinType>(): Option<u128> acquires CoinInfo {
        let maybe_supply = &borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).supply;
        if (option::is_some(maybe_supply)) {
            // We do track supply, in this case read from optional aggregator.
            let supply = option::borrow(maybe_supply);
            let value = optional_aggregator::read(supply);
            option::some(value)
        } else {
            option::none()
        }
    }
    //
    // Public functions
    //

    /// Burn `coin` with capability.
    /// The capability `_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    public fun burn<CoinType>(coin: Coin<CoinType>, _cap: &BurnCapability<CoinType>) acquires CoinInfo {
        burn_internal(coin);
    }

    /// Burn `coin` from the specified `account` with capability.
    /// The capability `burn_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    /// This function shouldn't fail as it's called as part of transaction fee burning.
    ///
    /// Note: This bypasses CoinStore::frozen -- coins within a frozen CoinStore can be burned.
    public fun burn_from<CoinType>(
        account_addr: address,
        amount: u64,
        burn_cap: &BurnCapability<CoinType>,
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedFungibleAssetRefs {
        // Skip burning if amount is zero. This shouldn't error out as it's called as part of transaction fee burning.
        if (amount == 0) {
            return
        };

        let (coin_amount_to_burn, fa_amount_to_burn) = calculate_amount_to_withdraw<CoinType>(
            account_addr,
            amount
        );
        if (coin_amount_to_burn > 0) {
            let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
            let coin_to_burn = extract(&mut coin_store.coin, coin_amount_to_burn);
            burn(coin_to_burn, burn_cap);
        };
        if (fa_amount_to_burn > 0) {
            fungible_asset::burn_from(
                borrow_paired_burn_ref(burn_cap),
                primary_fungible_store::primary_store(account_addr, option::destroy_some(paired_metadata<CoinType>())),
                fa_amount_to_burn
            );
        };
    }

    /// Deposit the coin balance into the recipient's account and emit an event.
    public fun deposit<CoinType>(
        account_addr: address,
        coin: Coin<CoinType>
    ) acquires CoinStore, CoinConversionMap, CoinInfo {
        if (exists<CoinStore<CoinType>>(account_addr)) {
            let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
            assert!(
                !coin_store.frozen,
                error::permission_denied(EFROZEN),
            );
            if (std::features::module_event_migration_enabled()) {
                event::emit(
                    CoinDeposit { coin_type: type_name<CoinType>(), account: account_addr, amount: coin.value }
                );
            };
            event::emit_event<DepositEvent>(
                &mut coin_store.deposit_events,
                DepositEvent { amount: coin.value },
            );
            merge(&mut coin_store.coin, coin);
        } else {
            let metadata = paired_metadata<CoinType>();
            if (option::is_some(&metadata) && migrated_primary_fungible_store_exists(
                account_addr,
                option::destroy_some(metadata)
            )) {
                primary_fungible_store::deposit(account_addr, coin_to_fungible_asset(coin));
            } else {
                abort error::not_found(ECOIN_STORE_NOT_PUBLISHED)
            };
        }
    }

    inline fun migrated_primary_fungible_store_exists(
        account_address: address,
        metadata: Object<Metadata>
    ): bool {
        let primary_store_address = primary_fungible_store::primary_store_address<Metadata>(account_address, metadata);
        fungible_asset::store_exists(primary_store_address) && (
            // migration flag is needed, until we start defaulting new accounts to APT PFS
            features::new_accounts_default_to_fa_apt_store_enabled() || exists<MigrationFlag>(primary_store_address)
        )
    }

    /// Deposit the coin balance into the recipient's account without checking if the account is frozen.
    /// This is for internal use only and doesn't emit an DepositEvent.
    public(friend) fun force_deposit<CoinType>(
        account_addr: address,
        coin: Coin<CoinType>
    ) acquires CoinStore, CoinConversionMap, CoinInfo {
        if (exists<CoinStore<CoinType>>(account_addr)) {
            let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
            merge(&mut coin_store.coin, coin);
        } else {
            let metadata = paired_metadata<CoinType>();
            if (option::is_some(&metadata) && migrated_primary_fungible_store_exists(
                account_addr,
                option::destroy_some(metadata)
            )) {
                let fa = coin_to_fungible_asset(coin);
                let metadata = fungible_asset::asset_metadata(&fa);
                let store = primary_fungible_store::primary_store(account_addr, metadata);
                fungible_asset::deposit_internal(object::object_address(&store), fa);
            } else {
                abort error::not_found(ECOIN_STORE_NOT_PUBLISHED)
            }
        }
    }

    /// Destroys a zero-value coin. Calls will fail if the `value` in the passed-in `token` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Coin` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(zero_coin: Coin<CoinType>) {
        spec {
            update supply<CoinType> = supply<CoinType> - zero_coin.value;
        };
        let Coin { value } = zero_coin;
        assert!(value == 0, error::invalid_argument(EDESTRUCTION_OF_NONZERO_TOKEN))
    }

    /// Extracts `amount` from the passed-in `coin`, where the original token is modified in place.
    public fun extract<CoinType>(coin: &mut Coin<CoinType>, amount: u64): Coin<CoinType> {
        assert!(coin.value >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));
        spec {
            update supply<CoinType> = supply<CoinType> - amount;
        };
        coin.value = coin.value - amount;
        spec {
            update supply<CoinType> = supply<CoinType> + amount;
        };
        Coin { value: amount }
    }

    /// Extracts the entire amount from the passed-in `coin`, where the original token is modified in place.
    public fun extract_all<CoinType>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let total_value = coin.value;
        spec {
            update supply<CoinType> = supply<CoinType> - coin.value;
        };
        coin.value = 0;
        spec {
            update supply<CoinType> = supply<CoinType> + total_value;
        };
        Coin { value: total_value }
    }

    #[legacy_entry_fun]
    /// Freeze a CoinStore to prevent transfers
    public entry fun freeze_coin_store<CoinType>(
        account_addr: address,
        _freeze_cap: &FreezeCapability<CoinType>,
    ) acquires CoinStore {
        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        coin_store.frozen = true;
    }

    #[legacy_entry_fun]
    /// Unfreeze a CoinStore to allow transfers
    public entry fun unfreeze_coin_store<CoinType>(
        account_addr: address,
        _freeze_cap: &FreezeCapability<CoinType>,
    ) acquires CoinStore {
        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        coin_store.frozen = false;
    }

    /// Upgrade total supply to use a parallelizable implementation if it is
    /// available.
    public entry fun upgrade_supply<CoinType>(account: &signer) acquires CoinInfo, SupplyConfig {
        let account_addr = signer::address_of(account);

        // Only coin creators can upgrade total supply.
        assert!(
            coin_address<CoinType>() == account_addr,
            error::invalid_argument(ECOIN_INFO_ADDRESS_MISMATCH),
        );

        // Can only succeed once on-chain governance agreed on the upgrade.
        assert!(
            borrow_global_mut<SupplyConfig>(@aptos_framework).allow_upgrades,
            error::permission_denied(ECOIN_SUPPLY_UPGRADE_NOT_SUPPORTED)
        );

        let maybe_supply = &mut borrow_global_mut<CoinInfo<CoinType>>(account_addr).supply;
        if (option::is_some(maybe_supply)) {
            let supply = option::borrow_mut(maybe_supply);

            // If supply is tracked and the current implementation uses an integer - upgrade.
            if (!optional_aggregator::is_parallelizable(supply)) {
                optional_aggregator::switch(supply);
            }
        }
    }

    /// Creates a new Coin with given `CoinType` and returns minting/freezing/burning capabilities.
    /// The given signer also becomes the account hosting the information  about the coin
    /// (name, supply, etc.). Supply is initialized as non-parallelizable integer.
    public fun initialize<CoinType>(
        account: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
        monitor_supply: bool,
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        initialize_internal(account, name, symbol, decimals, monitor_supply, false)
    }

    /// Same as `initialize` but supply can be initialized to parallelizable aggregator.
    public(friend) fun initialize_with_parallelizable_supply<CoinType>(
        account: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
        monitor_supply: bool,
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        system_addresses::assert_aptos_framework(account);
        initialize_internal(account, name, symbol, decimals, monitor_supply, true)
    }

    fun initialize_internal<CoinType>(
        account: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
        monitor_supply: bool,
        parallelizable: bool,
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        let account_addr = signer::address_of(account);

        assert!(
            coin_address<CoinType>() == account_addr,
            error::invalid_argument(ECOIN_INFO_ADDRESS_MISMATCH),
        );

        assert!(
            !exists<CoinInfo<CoinType>>(account_addr),
            error::already_exists(ECOIN_INFO_ALREADY_PUBLISHED),
        );

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ECOIN_NAME_TOO_LONG));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ECOIN_SYMBOL_TOO_LONG));

        let coin_info = CoinInfo<CoinType> {
            name,
            symbol,
            decimals,
            supply: if (monitor_supply) {
                option::some(
                    optional_aggregator::new(MAX_U128, parallelizable)
                )
            } else { option::none() },
        };
        move_to(account, coin_info);

        (BurnCapability<CoinType> {}, FreezeCapability<CoinType> {}, MintCapability<CoinType> {})
    }

    /// "Merges" the two given coins.  The coin passed in as `dst_coin` will have a value equal
    /// to the sum of the two tokens (`dst_coin` and `source_coin`).
    public fun merge<CoinType>(dst_coin: &mut Coin<CoinType>, source_coin: Coin<CoinType>) {
        spec {
            assume dst_coin.value + source_coin.value <= MAX_U64;
        };
        spec {
            update supply<CoinType> = supply<CoinType> - source_coin.value;
        };
        let Coin { value } = source_coin;
        spec {
            update supply<CoinType> = supply<CoinType> + value;
        };
        dst_coin.value = dst_coin.value + value;
    }

    /// Mint new `Coin` with capability.
    /// The capability `_cap` should be passed as reference to `MintCapability<CoinType>`.
    /// Returns minted `Coin`.
    public fun mint<CoinType>(
        amount: u64,
        _cap: &MintCapability<CoinType>,
    ): Coin<CoinType> acquires CoinInfo {
        mint_internal<CoinType>(amount)
    }

    public fun register<CoinType>(account: &signer) acquires CoinConversionMap {
        let account_addr = signer::address_of(account);
        // Short-circuit and do nothing if account is already registered for CoinType.
        if (is_account_registered<CoinType>(account_addr)) {
            return
        };

        account::register_coin<CoinType>(account_addr);
        let coin_store = CoinStore<CoinType> {
            coin: Coin { value: 0 },
            frozen: false,
            deposit_events: account::new_event_handle<DepositEvent>(account),
            withdraw_events: account::new_event_handle<WithdrawEvent>(account),
        };
        move_to(account, coin_store);
    }

    /// Transfers `amount` of coins `CoinType` from `from` to `to`.
    public entry fun transfer<CoinType>(
        from: &signer,
        to: address,
        amount: u64,
    ) acquires CoinStore, CoinConversionMap, CoinInfo, PairedCoinType {
        let coin = withdraw<CoinType>(from, amount);
        deposit(to, coin);
    }

    /// Returns the `value` passed in `coin`.
    public fun value<CoinType>(coin: &Coin<CoinType>): u64 {
        coin.value
    }

    /// Withdraw specified `amount` of coin `CoinType` from the signing account.
    public fun withdraw<CoinType>(
        account: &signer,
        amount: u64,
    ): Coin<CoinType> acquires CoinStore, CoinConversionMap, CoinInfo, PairedCoinType {
        let account_addr = signer::address_of(account);

        let (coin_amount_to_withdraw, fa_amount_to_withdraw) = calculate_amount_to_withdraw<CoinType>(
            account_addr,
            amount
        );
        let withdrawn_coin = if (coin_amount_to_withdraw > 0) {
            let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
            assert!(
                !coin_store.frozen,
                error::permission_denied(EFROZEN),
            );
            if (std::features::module_event_migration_enabled()) {
                event::emit(
                    CoinWithdraw {
                        coin_type: type_name<CoinType>(), account: account_addr, amount: coin_amount_to_withdraw
                    }
                );
            };
            event::emit_event<WithdrawEvent>(
                &mut coin_store.withdraw_events,
                WithdrawEvent { amount: coin_amount_to_withdraw },
            );
            extract(&mut coin_store.coin, coin_amount_to_withdraw)
        } else {
            zero()
        };
        if (fa_amount_to_withdraw > 0) {
            let fa = primary_fungible_store::withdraw(
                account,
                option::destroy_some(paired_metadata<CoinType>()),
                fa_amount_to_withdraw
            );
            merge(&mut withdrawn_coin, fungible_asset_to_coin(fa));
        };
        withdrawn_coin
    }

    /// Create a new `Coin<CoinType>` with a value of `0`.
    public fun zero<CoinType>(): Coin<CoinType> {
        spec {
            update supply<CoinType> = supply<CoinType> + 0;
        };
        Coin<CoinType> {
            value: 0
        }
    }

    /// Destroy a freeze capability. Freeze capability is dangerous and therefore should be destroyed if not used.
    public fun destroy_freeze_cap<CoinType>(freeze_cap: FreezeCapability<CoinType>) {
        let FreezeCapability<CoinType> {} = freeze_cap;
    }

    /// Destroy a mint capability.
    public fun destroy_mint_cap<CoinType>(mint_cap: MintCapability<CoinType>) {
        let MintCapability<CoinType> {} = mint_cap;
    }

    /// Destroy a burn capability.
    public fun destroy_burn_cap<CoinType>(burn_cap: BurnCapability<CoinType>) {
        let BurnCapability<CoinType> {} = burn_cap;
    }

    fun mint_internal<CoinType>(amount: u64): Coin<CoinType> acquires CoinInfo {
        if (amount == 0) {
            return Coin<CoinType> {
                value: 0
            }
        };

        let maybe_supply = &mut borrow_global_mut<CoinInfo<CoinType>>(coin_address<CoinType>()).supply;
        if (option::is_some(maybe_supply)) {
            let supply = option::borrow_mut(maybe_supply);
            spec {
                use aptos_framework::optional_aggregator;
                use aptos_framework::aggregator;
                assume optional_aggregator::is_parallelizable(supply) ==> (aggregator::spec_aggregator_get_val(
                    option::borrow(supply.aggregator)
                )
                    + amount <= aggregator::spec_get_limit(option::borrow(supply.aggregator)));
                assume !optional_aggregator::is_parallelizable(supply) ==>
                    (option::borrow(supply.integer).value + amount <= option::borrow(supply.integer).limit);
            };
            optional_aggregator::add(supply, (amount as u128));
        };
        spec {
            update supply<CoinType> = supply<CoinType> + amount;
        };
        Coin<CoinType> { value: amount }
    }

    fun burn_internal<CoinType>(coin: Coin<CoinType>): u64 acquires CoinInfo {
        spec {
            update supply<CoinType> = supply<CoinType> - coin.value;
        };
        let Coin { value: amount } = coin;
        if (amount != 0) {
            let maybe_supply = &mut borrow_global_mut<CoinInfo<CoinType>>(coin_address<CoinType>()).supply;
            if (option::is_some(maybe_supply)) {
                let supply = option::borrow_mut(maybe_supply);
                optional_aggregator::sub(supply, (amount as u128));
            };
        };
        amount
    }

    #[test_only]
    struct FakeMoney {}

    #[test_only]
    struct FakeMoneyCapabilities has key {
        burn_cap: BurnCapability<FakeMoney>,
        freeze_cap: FreezeCapability<FakeMoney>,
        mint_cap: MintCapability<FakeMoney>,
    }

    #[test_only]
    struct FakeMoneyRefs has key {
        mint_ref: MintRef,
        transfer_ref: TransferRef,
        burn_ref: BurnRef,
    }

    #[test_only]
    fun create_coin_store<CoinType>(account: &signer) {
        assert!(is_coin_initialized<CoinType>(), error::invalid_argument(ECOIN_INFO_NOT_PUBLISHED));
        if (!exists<CoinStore<CoinType>>(signer::address_of(account))) {
            let coin_store = CoinStore<CoinType> {
                coin: Coin { value: 0 },
                frozen: false,
                deposit_events: account::new_event_handle<DepositEvent>(account),
                withdraw_events: account::new_event_handle<WithdrawEvent>(account),
            };
            move_to(account, coin_store);
        }
    }

    #[test_only]
    fun coin_store_exists<CoinType>(account: address): bool {
        exists<CoinStore<CoinType>>(account)
    }

    #[test_only]
    fun initialize_fake_money(
        account: &signer,
        decimals: u8,
        monitor_supply: bool,
    ): (BurnCapability<FakeMoney>, FreezeCapability<FakeMoney>, MintCapability<FakeMoney>) {
        aggregator_factory::initialize_aggregator_factory_for_test(account);
        initialize<FakeMoney>(
            account,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            decimals,
            monitor_supply
        )
    }

    #[test_only]
    public fun initialize_and_register_fake_money(
        account: &signer,
        decimals: u8,
        monitor_supply: bool,
    ): (BurnCapability<FakeMoney>, FreezeCapability<FakeMoney>, MintCapability<FakeMoney>) {
        let (burn_cap, freeze_cap, mint_cap) = initialize_fake_money(
            account,
            decimals,
            monitor_supply
        );
        create_coin_store<FakeMoney>(account);
        create_coin_conversion_map(account);
        (burn_cap, freeze_cap, mint_cap)
    }

    #[test_only]
    public entry fun create_fake_money(
        source: &signer,
        destination: &signer,
        amount: u64
    ) acquires CoinInfo, CoinStore, CoinConversionMap {
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(source, 18, true);

        create_coin_store<FakeMoney>(destination);
        let coins_minted = mint<FakeMoney>(amount, &mint_cap);
        deposit(signer::address_of(source), coins_minted);
        move_to(source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    public entry fun end_to_end(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedCoinType {
        let source_addr = signer::address_of(&source);
        account::create_account_for_test(source_addr);
        let destination_addr = signer::address_of(&destination);
        account::create_account_for_test(destination_addr);

        let name = string::utf8(b"Fake money");
        let symbol = string::utf8(b"FMD");

        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(
            &source,
            18,
            true
        );
        register<FakeMoney>(&source);
        register<FakeMoney>(&destination);
        assert!(*option::borrow(&supply<FakeMoney>()) == 0, 0);

        assert!(name<FakeMoney>() == name, 1);
        assert!(symbol<FakeMoney>() == symbol, 2);
        assert!(decimals<FakeMoney>() == 18, 3);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(source_addr, coins_minted);
        maybe_convert_to_fungible_store<FakeMoney>(source_addr);
        assert!(!coin_store_exists<FakeMoney>(source_addr), 0);
        assert!(coin_store_exists<FakeMoney>(destination_addr), 0);

        transfer<FakeMoney>(&source, destination_addr, 50);
        maybe_convert_to_fungible_store<FakeMoney>(destination_addr);
        assert!(!coin_store_exists<FakeMoney>(destination_addr), 0);

        assert!(balance<FakeMoney>(source_addr) == 50, 4);
        assert!(balance<FakeMoney>(destination_addr) == 50, 5);
        assert!(*option::borrow(&supply<FakeMoney>()) == 100, 6);

        let coin = withdraw<FakeMoney>(&source, 10);
        assert!(value(&coin) == 10, 7);
        burn(coin, &burn_cap);
        assert!(*option::borrow(&supply<FakeMoney>()) == 90, 8);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    public entry fun end_to_end_no_supply(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedCoinType {
        let source_addr = signer::address_of(&source);
        account::create_account_for_test(source_addr);
        let destination_addr = signer::address_of(&destination);
        account::create_account_for_test(destination_addr);

        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1, false);

        register<FakeMoney>(&destination);
        assert!(option::is_none(&supply<FakeMoney>()), 0);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit<FakeMoney>(source_addr, coins_minted);
        transfer<FakeMoney>(&source, destination_addr, 50);

        assert!(balance<FakeMoney>(source_addr) == 50, 1);
        assert!(balance<FakeMoney>(destination_addr) == 50, 2);
        assert!(option::is_none(&supply<FakeMoney>()), 3);

        let coin = withdraw<FakeMoney>(&source, 10);
        burn(coin, &burn_cap);
        assert!(option::is_none(&supply<FakeMoney>()), 4);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x2, framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10001, location = Self)]
    public fun fail_initialize(source: signer, framework: signer) {
        aggregator_factory::initialize_aggregator_factory_for_test(&framework);
        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            &source,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            1,
            true,
        );

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    public entry fun transfer_to_migrated_destination(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedCoinType {
        let source_addr = signer::address_of(&source);
        account::create_account_for_test(source_addr);
        let destination_addr = signer::address_of(&destination);
        account::create_account_for_test(destination_addr);

        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1, true);
        assert!(*option::borrow(&supply<FakeMoney>()) == 0, 0);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(source_addr, coins_minted);
        maybe_convert_to_fungible_store<FakeMoney>(source_addr);
        assert!(!coin_store_exists<FakeMoney>(source_addr), 0);
        maybe_convert_to_fungible_store<FakeMoney>(destination_addr);
        transfer<FakeMoney>(&source, destination_addr, 50);
        assert!(balance<FakeMoney>(destination_addr) == 50, 2);
        assert!(!coin_store_exists<FakeMoney>(destination_addr), 0);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    public entry fun test_burn_from_with_capability(
        source: signer,
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedFungibleAssetRefs {
        let source_addr = signer::address_of(&source);
        account::create_account_for_test(source_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1, true);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(source_addr, coins_minted);
        let fa_minted = coin_to_fungible_asset(mint<FakeMoney>(200, &mint_cap));
        primary_fungible_store::deposit(source_addr, fa_minted);

        // Burn coin only with both stores
        burn_from<FakeMoney>(source_addr, 50, &burn_cap);
        assert!(balance<FakeMoney>(source_addr) == 250, 0);
        assert!(coin_balance<FakeMoney>(source_addr) == 50, 0);

        // Burn coin and fa with both stores
        burn_from<FakeMoney>(source_addr, 100, &burn_cap);
        assert!(balance<FakeMoney>(source_addr) == 150, 0);
        assert!(primary_fungible_store::balance(source_addr, ensure_paired_metadata<FakeMoney>()) == 150, 0);

        // Burn fa only with both stores
        burn_from<FakeMoney>(source_addr, 100, &burn_cap);
        assert!(balance<FakeMoney>(source_addr) == 50, 0);
        assert!(primary_fungible_store::balance(source_addr, ensure_paired_metadata<FakeMoney>()) == 50, 0);

        // Burn fa only with only fungible store
        let coins_minted = mint<FakeMoney>(50, &mint_cap);
        deposit(source_addr, coins_minted);
        maybe_convert_to_fungible_store<FakeMoney>(source_addr);
        assert!(!coin_store_exists<FakeMoney>(source_addr), 0);
        assert!(balance<FakeMoney>(source_addr) == 100, 0);
        assert!(*option::borrow(&supply<FakeMoney>()) == 100, 1);

        burn_from<FakeMoney>(source_addr, 10, &burn_cap);
        assert!(balance<FakeMoney>(source_addr) == 90, 2);
        assert!(*option::borrow(&supply<FakeMoney>()) == 90, 3);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x10007, location = Self)]
    public fun test_destroy_non_zero(
        source: signer,
    ) acquires CoinInfo {
        account::create_account_for_test(signer::address_of(&source));
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1, true);
        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        destroy_zero(coins_minted);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    public entry fun test_extract(
        source: signer,
    ) acquires CoinInfo, CoinStore, CoinConversionMap {
        let source_addr = signer::address_of(&source);
        account::create_account_for_test(source_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1, true);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);

        let extracted = extract(&mut coins_minted, 25);
        assert!(value(&coins_minted) == 75, 0);
        assert!(value(&extracted) == 25, 1);

        deposit(source_addr, coins_minted);
        deposit(source_addr, extracted);

        assert!(balance<FakeMoney>(source_addr) == 100, 2);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    public fun test_is_coin_initialized(source: signer) {
        assert!(!is_coin_initialized<FakeMoney>(), 0);

        let (burn_cap, freeze_cap, mint_cap) = initialize_fake_money(&source, 1, true);
        assert!(is_coin_initialized<FakeMoney>(), 1);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @0x1)]
    public fun test_is_coin_store_frozen(account: signer) acquires CoinStore, CoinConversionMap, CoinInfo {
        let account_addr = signer::address_of(&account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18, true);
        assert!(coin_store_exists<FakeMoney>(account_addr), 1);
        assert!(!is_coin_store_frozen<FakeMoney>(account_addr), 1);
        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        assert!(!coin_store_exists<FakeMoney>(account_addr), 1);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test]
    fun test_zero() {
        let zero = zero<FakeMoney>();
        assert!(value(&zero) == 0, 1);
        destroy_zero(zero);
    }

    #[test(account = @0x1)]
    public entry fun burn_frozen(
        account: signer
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedFungibleAssetRefs {
        let account_addr = signer::address_of(&account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18, true);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(account_addr, coins_minted);

        freeze_coin_store(account_addr, &freeze_cap);
        burn_from(account_addr, 90, &burn_cap);
        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        assert!(primary_fungible_store::is_frozen(account_addr, ensure_paired_metadata<FakeMoney>()), 1);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 10, 1);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @0x1)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::fungible_asset)]
    public entry fun withdraw_frozen(account: signer) acquires CoinInfo, CoinStore, CoinConversionMap, PairedCoinType {
        let account_addr = signer::address_of(&account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18, true);
        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(account_addr, coins_minted);

        freeze_coin_store(account_addr, &freeze_cap);
        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        let coin = withdraw<FakeMoney>(&account, 90);
        burn(coin, &burn_cap);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @0x1)]
    #[expected_failure(abort_code = 0x5000A, location = Self)]
    public entry fun deposit_frozen(account: signer) acquires CoinInfo, CoinStore, CoinConversionMap {
        let account_addr = signer::address_of(&account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18, true);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        freeze_coin_store(account_addr, &freeze_cap);
        deposit(account_addr, coins_minted);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @0x1)]
    public entry fun deposit_withdraw_unfrozen(
        account: signer
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedCoinType {
        let account_addr = signer::address_of(&account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18, true);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        freeze_coin_store(account_addr, &freeze_cap);
        unfreeze_coin_store(account_addr, &freeze_cap);
        deposit(account_addr, coins_minted);

        freeze_coin_store(account_addr, &freeze_cap);
        unfreeze_coin_store(account_addr, &freeze_cap);
        let coin = withdraw<FakeMoney>(&account, 10);
        burn(coin, &burn_cap);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test_only]
    fun initialize_with_aggregator(account: &signer) {
        let (burn_cap, freeze_cap, mint_cap) = initialize_with_parallelizable_supply<FakeMoney>(
            account,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            1,
            true
        );
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test_only]
    fun initialize_with_integer(account: &signer) {
        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            account,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            1,
            true
        );
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }


    #[test(framework = @aptos_framework, other = @0x123)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::system_addresses)]
    fun test_supply_initialize_fails(framework: signer, other: signer) {
        aggregator_factory::initialize_aggregator_factory_for_test(&framework);
        initialize_with_aggregator(&other);
    }

    #[test(other = @0x123)]
    #[expected_failure(abort_code = 0x10003, location = Self)]
    fun test_create_coin_store_with_non_coin_type(other: signer) acquires CoinConversionMap {
        register<String>(&other);
    }

    #[test(other = @0x123)]
    #[expected_failure(abort_code = 0x10003, location = Self)]
    fun test_migration_coin_store_with_non_coin_type(other: signer) acquires CoinConversionMap, CoinStore, CoinInfo {
        migrate_to_fungible_store<String>(&other);
    }

    #[test(framework = @aptos_framework)]
    fun test_supply_initialize(framework: signer) acquires CoinInfo {
        aggregator_factory::initialize_aggregator_factory_for_test(&framework);
        initialize_with_aggregator(&framework);

        let maybe_supply = &mut borrow_global_mut<CoinInfo<FakeMoney>>(coin_address<FakeMoney>()).supply;
        let supply = option::borrow_mut(maybe_supply);

        // Supply should be parallelizable.
        assert!(optional_aggregator::is_parallelizable(supply), 0);

        optional_aggregator::add(supply, 100);
        optional_aggregator::sub(supply, 50);
        optional_aggregator::add(supply, 950);
        assert!(optional_aggregator::read(supply) == 1000, 0);
    }

    #[test(framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x20001, location = aptos_framework::aggregator)]
    fun test_supply_overflow(framework: signer) acquires CoinInfo {
        aggregator_factory::initialize_aggregator_factory_for_test(&framework);
        initialize_with_aggregator(&framework);

        let maybe_supply = &mut borrow_global_mut<CoinInfo<FakeMoney>>(coin_address<FakeMoney>()).supply;
        let supply = option::borrow_mut(maybe_supply);

        optional_aggregator::add(supply, MAX_U128);
        optional_aggregator::add(supply, 1);
        optional_aggregator::sub(supply, 1);
    }

    #[test(framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x5000B, location = aptos_framework::coin)]
    fun test_supply_upgrade_fails(framework: signer) acquires CoinInfo, SupplyConfig {
        initialize_supply_config(&framework);
        aggregator_factory::initialize_aggregator_factory_for_test(&framework);
        initialize_with_integer(&framework);

        let maybe_supply = &mut borrow_global_mut<CoinInfo<FakeMoney>>(coin_address<FakeMoney>()).supply;
        let supply = option::borrow_mut(maybe_supply);

        // Supply should be non-parallelizable.
        assert!(!optional_aggregator::is_parallelizable(supply), 0);

        optional_aggregator::add(supply, 100);
        optional_aggregator::sub(supply, 50);
        optional_aggregator::add(supply, 950);
        assert!(optional_aggregator::read(supply) == 1000, 0);

        upgrade_supply<FakeMoney>(&framework);
    }

    #[test(framework = @aptos_framework)]
    fun test_supply_upgrade(framework: signer) acquires CoinInfo, SupplyConfig {
        initialize_supply_config(&framework);
        aggregator_factory::initialize_aggregator_factory_for_test(&framework);
        initialize_with_integer(&framework);

        // Ensure we have a non-parellelizable non-zero supply.
        let maybe_supply = &mut borrow_global_mut<CoinInfo<FakeMoney>>(coin_address<FakeMoney>()).supply;
        let supply = option::borrow_mut(maybe_supply);
        assert!(!optional_aggregator::is_parallelizable(supply), 0);
        optional_aggregator::add(supply, 100);

        // Upgrade.
        allow_supply_upgrades(&framework, true);
        upgrade_supply<FakeMoney>(&framework);

        // Check supply again.
        let maybe_supply = &mut borrow_global_mut<CoinInfo<FakeMoney>>(coin_address<FakeMoney>()).supply;
        let supply = option::borrow_mut(maybe_supply);
        assert!(optional_aggregator::is_parallelizable(supply), 0);
        assert!(optional_aggregator::read(supply) == 100, 0);
    }

    #[test_only]
    fun destroy_aggregatable_coin_for_test<CoinType>(aggregatable_coin: AggregatableCoin<CoinType>) {
        let AggregatableCoin { value } = aggregatable_coin;
        aggregator::destroy(value);
    }

    #[test(framework = @aptos_framework)]
    public entry fun test_collect_from_and_drain(
        framework: signer,
    ) acquires CoinInfo, CoinStore, CoinConversionMap, PairedCoinType {
        let framework_addr = signer::address_of(&framework);
        account::create_account_for_test(framework_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&framework, 1, true);

        // Collect from coin store only.
        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(framework_addr, coins_minted);
        let aggregatable_coin = initialize_aggregatable_coin<FakeMoney>(&framework);
        collect_into_aggregatable_coin<FakeMoney>(framework_addr, 50, &mut aggregatable_coin);

        let fa_minted = coin_to_fungible_asset(mint<FakeMoney>(100, &mint_cap));
        primary_fungible_store::deposit(framework_addr, fa_minted);
        assert!(balance<FakeMoney>(framework_addr) == 150, 0);
        assert!(*option::borrow(&supply<FakeMoney>()) == 200, 0);

        // Collect from coin store and fungible store.
        collect_into_aggregatable_coin<FakeMoney>(framework_addr, 100, &mut aggregatable_coin);

        assert!(balance<FakeMoney>(framework_addr) == 50, 0);
        maybe_convert_to_fungible_store<FakeMoney>(framework_addr);
        // Collect from fungible store only.
        collect_into_aggregatable_coin<FakeMoney>(framework_addr, 30, &mut aggregatable_coin);

        // Check that aggregatable coin has the right amount.
        let collected_coin = drain_aggregatable_coin(&mut aggregatable_coin);
        assert!(is_aggregatable_coin_zero(&aggregatable_coin), 0);
        assert!(value(&collected_coin) == 180, 0);

        // Supply of coins should be unchanged, but the balance on the account should decrease.
        assert!(balance<FakeMoney>(framework_addr) == 20, 0);
        assert!(*option::borrow(&supply<FakeMoney>()) == 200, 0);

        burn(collected_coin, &burn_cap);
        destroy_aggregatable_coin_for_test(aggregatable_coin);
        move_to(&framework, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test_only]
    fun deposit_to_coin_store<CoinType>(account_addr: address, coin: Coin<CoinType>) acquires CoinStore {
        assert!(
            coin_store_exists<CoinType>(account_addr),
            error::not_found(ECOIN_STORE_NOT_PUBLISHED),
        );

        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        assert!(
            !coin_store.frozen,
            error::permission_denied(EFROZEN),
        );

        event::emit_event<DepositEvent>(
            &mut coin_store.deposit_events,
            DepositEvent { amount: coin.value },
        );

        merge(&mut coin_store.coin, coin);
    }

    #[test(account = @aptos_framework)]
    fun test_conversion_basic(
        account: &signer
    ) acquires CoinConversionMap, CoinInfo, CoinStore, PairedCoinType, PairedFungibleAssetRefs {
        let account_addr = signer::address_of(account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);

        assert!(fungible_asset::name(ensure_paired_metadata<FakeMoney>()) == name<FakeMoney>(), 0);
        assert!(fungible_asset::symbol(ensure_paired_metadata<FakeMoney>()) == symbol<FakeMoney>(), 0);
        assert!(fungible_asset::decimals(ensure_paired_metadata<FakeMoney>()) == decimals<FakeMoney>(), 0);

        let minted_coin = mint(100, &mint_cap);
        let converted_fa = coin_to_fungible_asset(minted_coin);

        // check and get refs
        assert!(paired_mint_ref_exists<FakeMoney>(), 0);
        assert!(paired_transfer_ref_exists<FakeMoney>(), 0);
        assert!(paired_burn_ref_exists<FakeMoney>(), 0);
        let (mint_ref, mint_ref_receipt) = get_paired_mint_ref(&mint_cap);
        let (transfer_ref, transfer_ref_receipt) = get_paired_transfer_ref(&freeze_cap);
        let (burn_ref, burn_ref_receipt) = get_paired_burn_ref(&burn_cap);
        assert!(!paired_mint_ref_exists<FakeMoney>(), 0);
        assert!(!paired_transfer_ref_exists<FakeMoney>(), 0);
        assert!(!paired_burn_ref_exists<FakeMoney>(), 0);

        let minted_fa = fungible_asset::mint(&mint_ref, 100);
        assert!(&converted_fa == &minted_fa, 0);

        let coin = fungible_asset_to_coin<FakeMoney>(converted_fa);
        assert!(value(&coin) == 100, 0);

        deposit_to_coin_store(account_addr, coin);
        assert!(coin_store_exists<FakeMoney>(account_addr), 0);
        primary_fungible_store::deposit(account_addr, minted_fa);
        assert!(balance<FakeMoney>(account_addr) == 200, 0);

        let withdrawn_coin = withdraw<FakeMoney>(account, 1);
        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        assert!(!coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(balance<FakeMoney>(account_addr) == 199, 0);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 199, 0);

        let fa = coin_to_fungible_asset(withdrawn_coin);
        fungible_asset::burn(&burn_ref, fa);

        // Return and check the refs
        return_paired_mint_ref(mint_ref, mint_ref_receipt);
        return_paired_transfer_ref(transfer_ref, transfer_ref_receipt);
        return_paired_burn_ref(burn_ref, burn_ref_receipt);
        assert!(paired_mint_ref_exists<FakeMoney>(), 0);
        assert!(paired_transfer_ref_exists<FakeMoney>(), 0);
        assert!(paired_burn_ref_exists<FakeMoney>(), 0);

        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework, aaron = @0xcafe)]
    fun test_balance_with_both_stores(
        account: &signer,
        aaron: &signer
    ) acquires CoinConversionMap, CoinInfo, CoinStore {
        let account_addr = signer::address_of(account);
        let aaron_addr = signer::address_of(aaron);
        account::create_account_for_test(account_addr);
        account::create_account_for_test(aaron_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);
        create_coin_store<FakeMoney>(aaron);
        let coin = mint(100, &mint_cap);
        let fa = coin_to_fungible_asset(mint(100, &mint_cap));
        primary_fungible_store::deposit(aaron_addr, fa);
        deposit_to_coin_store(aaron_addr, coin);
        assert!(coin_balance<FakeMoney>(aaron_addr) == 100, 0);
        assert!(balance<FakeMoney>(aaron_addr) == 200, 0);
        maybe_convert_to_fungible_store<FakeMoney>(aaron_addr);
        assert!(balance<FakeMoney>(aaron_addr) == 200, 0);
        assert!(coin_balance<FakeMoney>(aaron_addr) == 0, 0);

        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework)]
    fun test_deposit(
        account: &signer,
    ) acquires CoinConversionMap, CoinInfo, CoinStore {
        let account_addr = signer::address_of(account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);
        let coin = mint<FakeMoney>(100, &mint_cap);
        deposit(account_addr, coin);
        assert!(coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 0, 0);
        assert!(balance<FakeMoney>(account_addr) == 100, 0);

        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        let coin = mint<FakeMoney>(100, &mint_cap);
        deposit(account_addr, coin);
        assert!(!coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(balance<FakeMoney>(account_addr) == 200, 0);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 200, 0);

        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework)]
    fun test_withdraw(
        account: &signer,
    ) acquires CoinConversionMap, CoinInfo, CoinStore, PairedCoinType {
        let account_addr = signer::address_of(account);
        account::create_account_for_test(account_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);
        let coin = mint<FakeMoney>(200, &mint_cap);
        deposit_to_coin_store(account_addr, coin);
        assert!(coin_balance<FakeMoney>(account_addr) == 200, 0);
        assert!(balance<FakeMoney>(account_addr) == 200, 0);

        // Withdraw from coin store only.
        let coin = withdraw<FakeMoney>(account, 100);
        assert!(coin_balance<FakeMoney>(account_addr) == 100, 0);
        assert!(balance<FakeMoney>(account_addr) == 100, 0);

        let fa = coin_to_fungible_asset(coin);
        primary_fungible_store::deposit(account_addr, fa);
        assert!(coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 100, 0);
        assert!(balance<FakeMoney>(account_addr) == 200, 0);

        // Withdraw from both coin store and fungible store.
        let coin = withdraw<FakeMoney>(account, 150);
        assert!(coin_balance<FakeMoney>(account_addr) == 0, 0);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 50, 0);

        deposit_to_coin_store(account_addr, coin);
        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        assert!(!coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(balance<FakeMoney>(account_addr) == 200, 0);
        assert!(primary_fungible_store::balance(account_addr, ensure_paired_metadata<FakeMoney>()) == 200, 0);

        // Withdraw from fungible store only.
        let coin = withdraw<FakeMoney>(account, 150);
        assert!(balance<FakeMoney>(account_addr) == 50, 0);
        burn(coin, &burn_cap);

        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework)]
    fun test_supply(
        account: &signer,
    ) acquires CoinConversionMap, CoinInfo, PairedCoinType, PairedFungibleAssetRefs {
        account::create_account_for_test(signer::address_of(account));
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);
        let coin = mint<FakeMoney>(100, &mint_cap);
        ensure_paired_metadata<FakeMoney>();
        let (mint_ref, mint_ref_receipt) = get_paired_mint_ref(&mint_cap);
        let (burn_ref, burn_ref_receipt) = get_paired_burn_ref(&burn_cap);
        let fungible_asset = fungible_asset::mint(&mint_ref, 50);
        assert!(option::is_none(&fungible_asset::maximum(ensure_paired_metadata<FakeMoney>())), 0);
        assert!(supply<FakeMoney>() == option::some(150), 0);
        assert!(coin_supply<FakeMoney>() == option::some(100), 0);
        assert!(fungible_asset::supply(ensure_paired_metadata<FakeMoney>()) == option::some(50), 0);
        let fa_from_coin = coin_to_fungible_asset(coin);
        assert!(supply<FakeMoney>() == option::some(150), 0);
        assert!(coin_supply<FakeMoney>() == option::some(0), 0);
        assert!(fungible_asset::supply(ensure_paired_metadata<FakeMoney>()) == option::some(150), 0);

        let coin_from_fa = fungible_asset_to_coin<FakeMoney>(fungible_asset);
        assert!(supply<FakeMoney>() == option::some(150), 0);
        assert!(coin_supply<FakeMoney>() == option::some(50), 0);
        assert!(fungible_asset::supply(ensure_paired_metadata<FakeMoney>()) == option::some(100), 0);
        burn(coin_from_fa, &burn_cap);
        fungible_asset::burn(&burn_ref, fa_from_coin);
        assert!(supply<FakeMoney>() == option::some(0), 0);
        return_paired_mint_ref(mint_ref, mint_ref_receipt);
        return_paired_burn_ref(burn_ref, burn_ref_receipt);
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework, aaron = @0xaa10, bob = @0xb0b)]
    #[expected_failure(abort_code = 0x60005, location = Self)]
    fun test_force_deposit(
        account: &signer,
        aaron: &signer,
        bob: &signer
    ) acquires CoinConversionMap, CoinInfo, CoinStore {
        let account_addr = signer::address_of(account);
        let aaron_addr = signer::address_of(aaron);
        let bob_addr = signer::address_of(bob);
        account::create_account_for_test(account_addr);
        account::create_account_for_test(aaron_addr);
        account::create_account_for_test(bob_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);
        maybe_convert_to_fungible_store<FakeMoney>(aaron_addr);
        deposit(aaron_addr, mint<FakeMoney>(1, &mint_cap));

        force_deposit(account_addr, mint<FakeMoney>(100, &mint_cap));
        force_deposit(aaron_addr, mint<FakeMoney>(50, &mint_cap));
        assert!(
            primary_fungible_store::balance(aaron_addr, option::extract(&mut paired_metadata<FakeMoney>())) == 51,
            0
        );
        assert!(coin_balance<FakeMoney>(account_addr) == 100, 0);
        force_deposit(bob_addr, mint<FakeMoney>(1, &mint_cap));
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework, aaron = @0xaa10, bob = @0xb0b)]
    fun test_is_account_registered(
        account: &signer,
        aaron: &signer,
        bob: &signer,
    ) acquires CoinConversionMap, CoinInfo, CoinStore {
        let account_addr = signer::address_of(account);
        let aaron_addr = signer::address_of(aaron);
        let bob_addr = signer::address_of(bob);
        account::create_account_for_test(account_addr);
        account::create_account_for_test(aaron_addr);
        account::create_account_for_test(bob_addr);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);

        assert!(coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(is_account_registered<FakeMoney>(account_addr), 0);

        assert!(!coin_store_exists<FakeMoney>(aaron_addr), 0);
        assert!(!is_account_registered<FakeMoney>(aaron_addr), 0);

        maybe_convert_to_fungible_store<FakeMoney>(aaron_addr);
        let coin = mint<FakeMoney>(100, &mint_cap);
        deposit(aaron_addr, coin);

        assert!(!coin_store_exists<FakeMoney>(aaron_addr), 0);
        assert!(is_account_registered<FakeMoney>(aaron_addr), 0);

        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        assert!(!coin_store_exists<FakeMoney>(account_addr), 0);
        assert!(is_account_registered<FakeMoney>(account_addr), 0);

        // Deposit FA to bob to created primary fungible store without `MigrationFlag`.
        primary_fungible_store::deposit(bob_addr, coin_to_fungible_asset(mint<FakeMoney>(100, &mint_cap)));
        assert!(!coin_store_exists<FakeMoney>(bob_addr), 0);
        register<FakeMoney>(bob);
        assert!(coin_store_exists<FakeMoney>(bob_addr), 0);
        maybe_convert_to_fungible_store<FakeMoney>(bob_addr);
        assert!(!coin_store_exists<FakeMoney>(bob_addr), 0);
        register<FakeMoney>(bob);
        assert!(!coin_store_exists<FakeMoney>(bob_addr), 0);

        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @aptos_framework, aaron = @0xaa10)]
    fun test_migration_with_existing_primary_fungible_store(
        account: &signer,
    ) acquires CoinConversionMap, CoinInfo, CoinStore, PairedCoinType {
        account::create_account_for_test(signer::address_of(account));
        let account_addr = signer::address_of(account);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(account, 1, true);

        let coin = mint<FakeMoney>(100, &mint_cap);
        primary_fungible_store::deposit(account_addr, coin_to_fungible_asset(coin));
        assert!(coin_balance<FakeMoney>(account_addr) == 0, 0);
        assert!(balance<FakeMoney>(account_addr) == 100, 0);
        let coin = withdraw<FakeMoney>(account, 50);
        assert!(!migrated_primary_fungible_store_exists(account_addr, ensure_paired_metadata<FakeMoney>()), 0);
        maybe_convert_to_fungible_store<FakeMoney>(account_addr);
        assert!(migrated_primary_fungible_store_exists(account_addr, ensure_paired_metadata<FakeMoney>()), 0);
        deposit(account_addr, coin);
        assert!(coin_balance<FakeMoney>(account_addr) == 0, 0);
        assert!(balance<FakeMoney>(account_addr) == 100, 0);

        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }
}
