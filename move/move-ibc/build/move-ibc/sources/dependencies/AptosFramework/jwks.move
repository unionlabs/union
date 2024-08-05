/// JWK functions and structs.
///
/// Note: An important design constraint for this module is that the JWK consensus Rust code is unable to
/// spawn a VM and make a Move function call. Instead, the JWK consensus Rust code will have to directly
/// write some of the resources in this file. As a result, the structs in this file are declared so as to
/// have a simple layout which is easily accessible in Rust.
module aptos_framework::jwks {
    use std::error;
    use std::option;
    use std::option::Option;
    use std::string;
    use std::string::{String, utf8};
    use std::vector;
    use aptos_std::comparator::{compare_u8_vector, is_greater_than, is_equal};
    use aptos_std::copyable_any;
    use aptos_std::copyable_any::Any;
    use aptos_framework::chain_status;
    use aptos_framework::config_buffer;
    use aptos_framework::event::emit;
    use aptos_framework::reconfiguration;
    use aptos_framework::system_addresses;
    #[test_only]
    use aptos_framework::account::create_account_for_test;

    friend aptos_framework::genesis;
    friend aptos_framework::reconfiguration_with_dkg;

    const EUNEXPECTED_EPOCH: u64 = 1;
    const EUNEXPECTED_VERSION: u64 = 2;
    const EUNKNOWN_PATCH_VARIANT: u64 = 3;
    const EUNKNOWN_JWK_VARIANT: u64 = 4;
    const EISSUER_NOT_FOUND: u64 = 5;
    const EJWK_ID_NOT_FOUND: u64 = 6;

    const ENATIVE_MISSING_RESOURCE_VALIDATOR_SET: u64 = 0x0101;
    const ENATIVE_MISSING_RESOURCE_OBSERVED_JWKS: u64 = 0x0102;
    const ENATIVE_INCORRECT_VERSION: u64 = 0x0103;
    const ENATIVE_MULTISIG_VERIFICATION_FAILED: u64 = 0x0104;
    const ENATIVE_NOT_ENOUGH_VOTING_POWER: u64 = 0x0105;

    /// An OIDC provider.
    struct OIDCProvider has copy, drop, store {
        /// The utf-8 encoded issuer string. E.g., b"https://www.facebook.com".
        name: vector<u8>,

        /// The ut8-8 encoded OpenID configuration URL of the provider.
        /// E.g., b"https://www.facebook.com/.well-known/openid-configuration/".
        config_url: vector<u8>,
    }

    /// A list of OIDC providers whose JWKs should be watched by validators. Maintained by governance proposals.
    struct SupportedOIDCProviders has copy, drop, key, store {
        providers: vector<OIDCProvider>,
    }

    /// An JWK variant that represents the JWKs which were observed but not yet supported by Aptos.
    /// Observing `UnsupportedJWK`s means the providers adopted a new key type/format, and the system should be updated.
    struct UnsupportedJWK has copy, drop, store {
        id: vector<u8>,
        payload: vector<u8>,
    }

    /// A JWK variant where `kty` is `RSA`.
    struct RSA_JWK has copy, drop, store {
        kid: String,
        kty: String,
        alg: String,
        e: String,
        n: String,
    }

    /// A JSON web key.
    struct JWK has copy, drop, store {
        /// A `JWK` variant packed as an `Any`.
        /// Currently the variant type is one of the following.
        /// - `RSA_JWK`
        /// - `UnsupportedJWK`
        variant: Any,
    }

    /// A provider and its `JWK`s.
    struct ProviderJWKs has copy, drop, store {
        /// The utf-8 encoding of the issuer string (e.g., "https://www.facebook.com").
        issuer: vector<u8>,

        /// A version number is needed by JWK consensus to dedup the updates.
        /// e.g, when on chain version = 5, multiple nodes can propose an update with version = 6.
        /// Bumped every time the JWKs for the current issuer is updated.
        /// The Rust authenticator only uses the latest version.
        version: u64,

        /// Vector of `JWK`'s sorted by their unique ID (from `get_jwk_id`) in dictionary order.
        jwks: vector<JWK>,
    }

    /// Multiple `ProviderJWKs` objects, indexed by issuer and key ID.
    struct AllProvidersJWKs has copy, drop, store {
        /// Vector of `ProviderJWKs` sorted by `ProviderJWKs::issuer` in dictionary order.
        entries: vector<ProviderJWKs>,
    }

    /// The `AllProvidersJWKs` that validators observed and agreed on.
    struct ObservedJWKs has copy, drop, key, store {
        jwks: AllProvidersJWKs,
    }

    #[event]
    /// When `ObservedJWKs` is updated, this event is sent to resync the JWK consensus state in all validators.
    struct ObservedJWKsUpdated has drop, store {
        epoch: u64,
        jwks: AllProvidersJWKs,
    }

    /// A small edit or patch that is applied to a `AllProvidersJWKs` to obtain `PatchedJWKs`.
    struct Patch has copy, drop, store {
        /// A `Patch` variant packed as an `Any`.
        /// Currently the variant type is one of the following.
        /// - `PatchRemoveAll`
        /// - `PatchRemoveIssuer`
        /// - `PatchRemoveJWK`
        /// - `PatchUpsertJWK`
        variant: Any,
    }

    /// A `Patch` variant to remove all JWKs.
    struct PatchRemoveAll has copy, drop, store {}

    /// A `Patch` variant to remove an issuer and all its JWKs.
    struct PatchRemoveIssuer has copy, drop, store {
        issuer: vector<u8>,
    }

    /// A `Patch` variant to remove a specific JWK of an issuer.
    struct PatchRemoveJWK has copy, drop, store {
        issuer: vector<u8>,
        jwk_id: vector<u8>,
    }

    /// A `Patch` variant to upsert a JWK for an issuer.
    struct PatchUpsertJWK has copy, drop, store {
        issuer: vector<u8>,
        jwk: JWK,
    }

    /// A sequence of `Patch` objects that are applied *one by one* to the `ObservedJWKs`.
    ///
    /// Maintained by governance proposals.
    struct Patches has key {
        patches: vector<Patch>,
    }

    /// The result of applying the `Patches` to the `ObservedJWKs`.
    /// This is what applications should consume.
    struct PatchedJWKs has drop, key {
        jwks: AllProvidersJWKs,
    }

    //
    // Structs end.
    // Functions begin.
    //

    /// Get a JWK by issuer and key ID from the `PatchedJWKs`.
    /// Abort if such a JWK does not exist.
    /// More convenient to call from Rust, since it does not wrap the JWK in an `Option`.
    public fun get_patched_jwk(issuer: vector<u8>, jwk_id: vector<u8>): JWK acquires PatchedJWKs {
        option::extract(&mut try_get_patched_jwk(issuer, jwk_id))
    }

    /// Get a JWK by issuer and key ID from the `PatchedJWKs`, if it exists.
    /// More convenient to call from Move, since it does not abort.
    public fun try_get_patched_jwk(issuer: vector<u8>, jwk_id: vector<u8>): Option<JWK> acquires PatchedJWKs {
        let jwks = &borrow_global<PatchedJWKs>(@aptos_framework).jwks;
        try_get_jwk_by_issuer(jwks, issuer, jwk_id)
    }

    /// Deprecated by `upsert_oidc_provider_for_next_epoch()`.
    ///
    /// TODO: update all the tests that reference this function, then disable this function.
    public fun upsert_oidc_provider(fx: &signer, name: vector<u8>, config_url: vector<u8>): Option<vector<u8>> acquires SupportedOIDCProviders {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();

        let provider_set = borrow_global_mut<SupportedOIDCProviders>(@aptos_framework);

        let old_config_url= remove_oidc_provider_internal(provider_set, name);
        vector::push_back(&mut provider_set.providers, OIDCProvider { name, config_url });
        old_config_url
    }

    /// Used in on-chain governances to update the supported OIDC providers, effective starting next epoch.
    /// Example usage:
    /// ```
    /// aptos_framework::jwks::upsert_oidc_provider_for_next_epoch(
    ///     &framework_signer,
    ///     b"https://accounts.google.com",
    ///     b"https://accounts.google.com/.well-known/openid-configuration"
    /// );
    /// aptos_framework::aptos_governance::reconfigure(&framework_signer);
    /// ```
    public fun upsert_oidc_provider_for_next_epoch(fx: &signer, name: vector<u8>, config_url: vector<u8>): Option<vector<u8>> acquires SupportedOIDCProviders {
        system_addresses::assert_aptos_framework(fx);

        let provider_set = if (config_buffer::does_exist<SupportedOIDCProviders>()) {
            config_buffer::extract<SupportedOIDCProviders>()
        } else {
            *borrow_global_mut<SupportedOIDCProviders>(@aptos_framework)
        };

        let old_config_url = remove_oidc_provider_internal(&mut provider_set, name);
        vector::push_back(&mut provider_set.providers, OIDCProvider { name, config_url });
        config_buffer::upsert(provider_set);
        old_config_url
    }

    /// Deprecated by `remove_oidc_provider_for_next_epoch()`.
    ///
    /// TODO: update all the tests that reference this function, then disable this function.
    public fun remove_oidc_provider(fx: &signer, name: vector<u8>): Option<vector<u8>> acquires SupportedOIDCProviders {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();

        let provider_set = borrow_global_mut<SupportedOIDCProviders>(@aptos_framework);
        remove_oidc_provider_internal(provider_set, name)
    }

    /// Used in on-chain governances to update the supported OIDC providers, effective starting next epoch.
    /// Example usage:
    /// ```
    /// aptos_framework::jwks::remove_oidc_provider_for_next_epoch(
    ///     &framework_signer,
    ///     b"https://accounts.google.com",
    /// );
    /// aptos_framework::aptos_governance::reconfigure(&framework_signer);
    /// ```
    public fun remove_oidc_provider_for_next_epoch(fx: &signer, name: vector<u8>): Option<vector<u8>> acquires SupportedOIDCProviders {
        system_addresses::assert_aptos_framework(fx);

        let provider_set = if (config_buffer::does_exist<SupportedOIDCProviders>()) {
            config_buffer::extract<SupportedOIDCProviders>()
        } else {
            *borrow_global_mut<SupportedOIDCProviders>(@aptos_framework)
        };
        let ret = remove_oidc_provider_internal(&mut provider_set, name);
        config_buffer::upsert(provider_set);
        ret
    }

    /// Only used in reconfigurations to apply the pending `SupportedOIDCProviders`, if there is any.
    public(friend) fun on_new_epoch(framework: &signer) acquires SupportedOIDCProviders {
        system_addresses::assert_aptos_framework(framework);
        if (config_buffer::does_exist<SupportedOIDCProviders>()) {
            let new_config = config_buffer::extract<SupportedOIDCProviders>();
            if (exists<SupportedOIDCProviders>(@aptos_framework)) {
                *borrow_global_mut<SupportedOIDCProviders>(@aptos_framework) = new_config;
            } else {
                move_to(framework, new_config);
            }
        }
    }

    /// Set the `Patches`. Only called in governance proposals.
    public fun set_patches(fx: &signer, patches: vector<Patch>) acquires Patches, PatchedJWKs, ObservedJWKs {
        system_addresses::assert_aptos_framework(fx);
        borrow_global_mut<Patches>(@aptos_framework).patches = patches;
        regenerate_patched_jwks();
    }

    /// Create a `Patch` that removes all entries.
    public fun new_patch_remove_all(): Patch {
        Patch {
            variant: copyable_any::pack(PatchRemoveAll {}),
        }
    }

    /// Create a `Patch` that removes the entry of a given issuer, if exists.
    public fun new_patch_remove_issuer(issuer: vector<u8>): Patch {
        Patch {
            variant: copyable_any::pack(PatchRemoveIssuer { issuer }),
        }
    }

    /// Create a `Patch` that removes the entry of a given issuer, if exists.
    public fun new_patch_remove_jwk(issuer: vector<u8>, jwk_id: vector<u8>): Patch {
        Patch {
            variant: copyable_any::pack(PatchRemoveJWK { issuer, jwk_id })
        }
    }

    /// Create a `Patch` that upserts a JWK into an issuer's JWK set.
    public fun new_patch_upsert_jwk(issuer: vector<u8>, jwk: JWK): Patch {
        Patch {
            variant: copyable_any::pack(PatchUpsertJWK { issuer, jwk })
        }
    }

    /// Create a `JWK` of variant `RSA_JWK`.
    public fun new_rsa_jwk(kid: String, alg: String, e: String, n: String): JWK {
        JWK {
            variant: copyable_any::pack(RSA_JWK {
                kid,
                kty: utf8(b"RSA"),
                e,
                n,
                alg,
            }),
        }
    }

    /// Create a `JWK` of variant `UnsupportedJWK`.
    public fun new_unsupported_jwk(id: vector<u8>, payload: vector<u8>): JWK {
        JWK {
            variant: copyable_any::pack(UnsupportedJWK { id, payload })
        }
    }

    /// Initialize some JWK resources. Should only be invoked by genesis.
    public fun initialize(fx: &signer) {
        system_addresses::assert_aptos_framework(fx);
        move_to(fx, SupportedOIDCProviders { providers: vector[] });
        move_to(fx, ObservedJWKs { jwks: AllProvidersJWKs { entries: vector[] } });
        move_to(fx, Patches { patches: vector[] });
        move_to(fx, PatchedJWKs { jwks: AllProvidersJWKs { entries: vector[] } });
    }

    /// Helper function that removes an OIDC provider from the `SupportedOIDCProviders`.
    /// Returns the old config URL of the provider, if any, as an `Option`.
    fun remove_oidc_provider_internal(provider_set: &mut SupportedOIDCProviders, name: vector<u8>): Option<vector<u8>> {
        let (name_exists, idx) = vector::find(&provider_set.providers, |obj| {
            let provider: &OIDCProvider = obj;
            provider.name == name
        });

        if (name_exists) {
            let old_provider = vector::swap_remove(&mut provider_set.providers, idx);
            option::some(old_provider.config_url)
        } else {
            option::none()
        }
    }

    /// Only used by validators to publish their observed JWK update.
    ///
    /// NOTE: It is assumed verification has been done to ensure each update is quorum-certified,
    /// and its `version` equals to the on-chain version + 1.
    public fun upsert_into_observed_jwks(fx: &signer, provider_jwks_vec: vector<ProviderJWKs>) acquires ObservedJWKs, PatchedJWKs, Patches {
        system_addresses::assert_aptos_framework(fx);
        let observed_jwks = borrow_global_mut<ObservedJWKs>(@aptos_framework);
        vector::for_each(provider_jwks_vec, |obj| {
            let provider_jwks: ProviderJWKs = obj;
            upsert_provider_jwks(&mut observed_jwks.jwks, provider_jwks);
        });

        let epoch = reconfiguration::current_epoch();
        emit(ObservedJWKsUpdated { epoch, jwks: observed_jwks.jwks });
        regenerate_patched_jwks();
    }

    /// Only used by governance to delete an issuer from `ObservedJWKs`, if it exists.
    ///
    /// Return the potentially existing `ProviderJWKs` of the given issuer.
    public fun remove_issuer_from_observed_jwks(fx: &signer, issuer: vector<u8>): Option<ProviderJWKs> acquires ObservedJWKs, PatchedJWKs, Patches {
        system_addresses::assert_aptos_framework(fx);
        let observed_jwks = borrow_global_mut<ObservedJWKs>(@aptos_framework);
        let old_value = remove_issuer(&mut observed_jwks.jwks, issuer);

        let epoch = reconfiguration::current_epoch();
        emit(ObservedJWKsUpdated { epoch, jwks: observed_jwks.jwks });
        regenerate_patched_jwks();

        old_value
    }

    /// Regenerate `PatchedJWKs` from `ObservedJWKs` and `Patches` and save the result.
    fun regenerate_patched_jwks() acquires PatchedJWKs, Patches, ObservedJWKs {
        let jwks = borrow_global<ObservedJWKs>(@aptos_framework).jwks;
        let patches = borrow_global<Patches>(@aptos_framework);
        vector::for_each_ref(&patches.patches, |obj|{
            let patch: &Patch = obj;
            apply_patch(&mut jwks, *patch);
        });
        *borrow_global_mut<PatchedJWKs>(@aptos_framework) = PatchedJWKs { jwks };
    }

    /// Get a JWK by issuer and key ID from a `AllProvidersJWKs`, if it exists.
    fun try_get_jwk_by_issuer(jwks: &AllProvidersJWKs, issuer: vector<u8>, jwk_id: vector<u8>): Option<JWK> {
        let (issuer_found, index) = vector::find(&jwks.entries, |obj| {
            let provider_jwks: &ProviderJWKs = obj;
            issuer == provider_jwks.issuer
        });

        if (issuer_found) {
            try_get_jwk_by_id(vector::borrow(&jwks.entries, index), jwk_id)
        } else {
            option::none()
        }
    }

    /// Get a JWK by key ID from a `ProviderJWKs`, if it exists.
    fun try_get_jwk_by_id(provider_jwks: &ProviderJWKs, jwk_id: vector<u8>): Option<JWK> {
        let (jwk_id_found, index) = vector::find(&provider_jwks.jwks, |obj|{
            let jwk: &JWK = obj;
            jwk_id == get_jwk_id(jwk)
        });

        if (jwk_id_found) {
            option::some(*vector::borrow(&provider_jwks.jwks, index))
        } else {
            option::none()
        }
    }

    /// Get the ID of a JWK.
    fun get_jwk_id(jwk: &JWK): vector<u8> {
        let variant_type_name = *string::bytes(copyable_any::type_name(&jwk.variant));
        if (variant_type_name == b"0x1::jwks::RSA_JWK") {
            let rsa = copyable_any::unpack<RSA_JWK>(jwk.variant);
            *string::bytes(&rsa.kid)
        } else if (variant_type_name == b"0x1::jwks::UnsupportedJWK") {
            let unsupported = copyable_any::unpack<UnsupportedJWK>(jwk.variant);
            unsupported.id
        } else {
            abort(error::invalid_argument(EUNKNOWN_JWK_VARIANT))
        }
    }

    /// Upsert a `ProviderJWKs` into an `AllProvidersJWKs`. If this upsert replaced an existing entry, return it.
    /// Maintains the sorted-by-issuer invariant in `AllProvidersJWKs`.
    fun upsert_provider_jwks(jwks: &mut AllProvidersJWKs, provider_jwks: ProviderJWKs): Option<ProviderJWKs> {
        // NOTE: Using a linear-time search here because we do not expect too many providers.
        let found = false;
        let index = 0;
        let num_entries = vector::length(&jwks.entries);
        while (index < num_entries) {
            let cur_entry = vector::borrow(&jwks.entries, index);
            let comparison = compare_u8_vector(provider_jwks.issuer, cur_entry.issuer);
            if (is_greater_than(&comparison)) {
                index = index + 1;
            } else {
                found = is_equal(&comparison);
                break
            }
        };

        // Now if `found == true`, `index` points to the JWK we want to update/remove; otherwise, `index` points to
        // where we want to insert.
        let ret = if (found) {
            let entry = vector::borrow_mut(&mut jwks.entries, index);
            let old_entry = option::some(*entry);
            *entry = provider_jwks;
            old_entry
        } else {
            vector::insert(&mut jwks.entries, index, provider_jwks);
            option::none()
        };

        ret
    }

    /// Remove the entry of an issuer from a `AllProvidersJWKs` and return the entry, if exists.
    /// Maintains the sorted-by-issuer invariant in `AllProvidersJWKs`.
    fun remove_issuer(jwks: &mut AllProvidersJWKs, issuer: vector<u8>): Option<ProviderJWKs> {
        let (found, index) = vector::find(&jwks.entries, |obj| {
            let provider_jwk_set: &ProviderJWKs = obj;
            provider_jwk_set.issuer == issuer
        });

        let ret = if (found) {
            option::some(vector::remove(&mut jwks.entries, index))
        } else {
            option::none()
        };

        ret
    }

    /// Upsert a `JWK` into a `ProviderJWKs`. If this upsert replaced an existing entry, return it.
    fun upsert_jwk(set: &mut ProviderJWKs, jwk: JWK): Option<JWK> {
        let found = false;
        let index = 0;
        let num_entries = vector::length(&set.jwks);
        while (index < num_entries) {
            let cur_entry = vector::borrow(&set.jwks, index);
            let comparison = compare_u8_vector(get_jwk_id(&jwk), get_jwk_id(cur_entry));
            if (is_greater_than(&comparison)) {
                index = index + 1;
            } else {
                found = is_equal(&comparison);
                break
            }
        };

        // Now if `found == true`, `index` points to the JWK we want to update/remove; otherwise, `index` points to
        // where we want to insert.
        let ret = if (found) {
            let entry = vector::borrow_mut(&mut set.jwks, index);
            let old_entry = option::some(*entry);
            *entry = jwk;
            old_entry
        } else {
            vector::insert(&mut set.jwks, index, jwk);
            option::none()
        };

        ret
    }

    /// Remove the entry of a key ID from a `ProviderJWKs` and return the entry, if exists.
    fun remove_jwk(jwks: &mut ProviderJWKs, jwk_id: vector<u8>): Option<JWK> {
        let (found, index) = vector::find(&jwks.jwks, |obj| {
            let jwk: &JWK = obj;
            jwk_id == get_jwk_id(jwk)
        });

        let ret = if (found) {
            option::some(vector::remove(&mut jwks.jwks, index))
        } else {
            option::none()
        };

        ret
    }

    /// Modify an `AllProvidersJWKs` object with a `Patch`.
    /// Maintains the sorted-by-issuer invariant in `AllProvidersJWKs`.
    fun apply_patch(jwks: &mut AllProvidersJWKs, patch: Patch) {
        let variant_type_name = *string::bytes(copyable_any::type_name(&patch.variant));
        if (variant_type_name == b"0x1::jwks::PatchRemoveAll") {
            jwks.entries = vector[];
        } else if (variant_type_name == b"0x1::jwks::PatchRemoveIssuer") {
            let cmd = copyable_any::unpack<PatchRemoveIssuer>(patch.variant);
            remove_issuer(jwks, cmd.issuer);
        } else if (variant_type_name == b"0x1::jwks::PatchRemoveJWK") {
            let cmd = copyable_any::unpack<PatchRemoveJWK>(patch.variant);
            // TODO: This is inefficient: we remove the issuer, modify its JWKs & and reinsert the updated issuer. Why
            // not just update it in place?
            let existing_jwk_set = remove_issuer(jwks, cmd.issuer);
            if (option::is_some(&existing_jwk_set)) {
                let jwk_set = option::extract(&mut existing_jwk_set);
                remove_jwk(&mut jwk_set, cmd.jwk_id);
                upsert_provider_jwks(jwks, jwk_set);
            };
        } else if (variant_type_name == b"0x1::jwks::PatchUpsertJWK") {
            let cmd = copyable_any::unpack<PatchUpsertJWK>(patch.variant);
            // TODO: This is inefficient: we remove the issuer, modify its JWKs & and reinsert the updated issuer. Why
            // not just update it in place?
            let existing_jwk_set = remove_issuer(jwks, cmd.issuer);
            let jwk_set = if (option::is_some(&existing_jwk_set)) {
                option::extract(&mut existing_jwk_set)
            } else {
                ProviderJWKs {
                    version: 0,
                    issuer: cmd.issuer,
                    jwks: vector[],
                }
            };
            upsert_jwk(&mut jwk_set, cmd.jwk);
            upsert_provider_jwks(jwks, jwk_set);
        } else {
            abort(std::error::invalid_argument(EUNKNOWN_PATCH_VARIANT))
        }
    }

    //
    // Functions end.
    // Tests begin.
    //

    #[test_only]
    fun initialize_for_test(aptos_framework: &signer) {
        create_account_for_test(@aptos_framework);
        reconfiguration::initialize_for_test(aptos_framework);
        initialize(aptos_framework);
    }

    #[test(fx = @aptos_framework)]
    fun test_observed_jwks_operations(fx: &signer) acquires ObservedJWKs, PatchedJWKs, Patches {
        initialize_for_test(fx);
        let jwk_0 = new_unsupported_jwk(b"key_id_0", b"key_payload_0");
        let jwk_1 = new_unsupported_jwk(b"key_id_1", b"key_payload_1");
        let jwk_2 = new_unsupported_jwk(b"key_id_2", b"key_payload_2");
        let jwk_3 = new_unsupported_jwk(b"key_id_3", b"key_payload_3");
        let jwk_4 = new_unsupported_jwk(b"key_id_4", b"key_payload_4");
        let expected = AllProvidersJWKs { entries: vector[] };
        assert!(expected == borrow_global<ObservedJWKs>(@aptos_framework).jwks, 1);

        let alice_jwks_v1 = ProviderJWKs {
            issuer: b"alice",
            version: 1,
            jwks: vector[jwk_0, jwk_1],
        };
        let bob_jwks_v1 = ProviderJWKs{
            issuer: b"bob",
            version: 1,
            jwks: vector[jwk_2, jwk_3],
        };
        upsert_into_observed_jwks(fx, vector[bob_jwks_v1]);
        upsert_into_observed_jwks(fx, vector[alice_jwks_v1]);
        let expected = AllProvidersJWKs { entries: vector[
            alice_jwks_v1,
            bob_jwks_v1,
        ] };
        assert!(expected == borrow_global<ObservedJWKs>(@aptos_framework).jwks, 2);

        let alice_jwks_v2 = ProviderJWKs {
            issuer: b"alice",
            version: 2,
            jwks: vector[jwk_1, jwk_4],
        };
        upsert_into_observed_jwks(fx, vector[alice_jwks_v2]);
        let expected = AllProvidersJWKs { entries: vector[
            alice_jwks_v2,
            bob_jwks_v1,
        ] };
        assert!(expected == borrow_global<ObservedJWKs>(@aptos_framework).jwks, 3);

        remove_issuer_from_observed_jwks(fx, b"alice");
        let expected = AllProvidersJWKs { entries: vector[bob_jwks_v1] };
        assert!(expected == borrow_global<ObservedJWKs>(@aptos_framework).jwks, 4);
    }

    #[test]
    fun test_apply_patch() {
        let jwks = AllProvidersJWKs {
            entries: vector[
                ProviderJWKs {
                    issuer: b"alice",
                    version: 111,
                    jwks: vector[
                        new_rsa_jwk(
                            utf8(b"e4adfb436b9e197e2e1106af2c842284e4986aff"), // kid
                            utf8(b"RS256"), // alg
                            utf8(b"AQAB"), // e
                            utf8(b"psply8S991RswM0JQJwv51fooFFvZUtYdL8avyKObshyzj7oJuJD8vkf5DKJJF1XOGi6Wv2D-U4b3htgrVXeOjAvaKTYtrQVUG_Txwjebdm2EvBJ4R6UaOULjavcSkb8VzW4l4AmP_yWoidkHq8n6vfHt9alDAONILi7jPDzRC7NvnHQ_x0hkRVh_OAmOJCpkgb0gx9-U8zSBSmowQmvw15AZ1I0buYZSSugY7jwNS2U716oujAiqtRkC7kg4gPouW_SxMleeo8PyRsHpYCfBME66m-P8Zr9Fh1Qgmqg4cWdy_6wUuNc1cbVY_7w1BpHZtZCNeQ56AHUgUFmo2LAQQ"), // n
                        ),
                        new_unsupported_jwk(b"key_id_0", b"key_content_0"),
                    ],
                },
                ProviderJWKs {
                    issuer: b"bob",
                    version: 222,
                    jwks: vector[
                        new_unsupported_jwk(b"key_id_1", b"key_content_1"),
                        new_unsupported_jwk(b"key_id_2", b"key_content_2"),
                    ],
                },
            ],
        };

        let patch = new_patch_remove_issuer(b"alice");
        apply_patch(&mut jwks, patch);
        assert!(jwks == AllProvidersJWKs {
            entries: vector[
                ProviderJWKs {
                    issuer: b"bob",
                    version: 222,
                    jwks: vector[
                        new_unsupported_jwk(b"key_id_1", b"key_content_1"),
                        new_unsupported_jwk(b"key_id_2", b"key_content_2"),
                    ],
                },
            ],
        }, 1);

        let patch = new_patch_remove_jwk(b"bob", b"key_id_1");
        apply_patch(&mut jwks, patch);
        assert!(jwks == AllProvidersJWKs {
            entries: vector[
                ProviderJWKs {
                    issuer: b"bob",
                    version: 222,
                    jwks: vector[
                        new_unsupported_jwk(b"key_id_2", b"key_content_2"),
                    ],
                },
            ],
        }, 1);

        let patch = new_patch_upsert_jwk(b"carl", new_rsa_jwk(
            utf8(b"0ad1fec78504f447bae65bcf5afaedb65eec9e81"), // kid
            utf8(b"RS256"), // alg
            utf8(b"AQAB"), // e
            utf8(b"sm72oBH-R2Rqt4hkjp66tz5qCtq42TMnVgZg2Pdm_zs7_-EoFyNs9sD1MKsZAFaBPXBHDiWywyaHhLgwETLN9hlJIZPzGCEtV3mXJFSYG-8L6t3kyKi9X1lUTZzbmNpE0tf-eMW-3gs3VQSBJQOcQnuiANxbSXwS3PFmi173C_5fDSuC1RoYGT6X3JqLc3DWUmBGucuQjPaUF0w6LMqEIy0W_WYbW7HImwANT6dT52T72md0JWZuAKsRRnRr_bvaUX8_e3K8Pb1K_t3dD6WSLvtmEfUnGQgLynVl3aV5sRYC0Hy_IkRgoxl2fd8AaZT1X_rdPexYpx152Pl_CHJ79Q"), // n
        ));
        apply_patch(&mut jwks, patch);
        let edit = new_patch_upsert_jwk(b"bob", new_unsupported_jwk(b"key_id_2", b"key_content_2b"));
        apply_patch(&mut jwks, edit);
        let edit = new_patch_upsert_jwk(b"alice", new_unsupported_jwk(b"key_id_3", b"key_content_3"));
        apply_patch(&mut jwks, edit);
        let edit = new_patch_upsert_jwk(b"alice", new_unsupported_jwk(b"key_id_0", b"key_content_0b"));
        apply_patch(&mut jwks, edit);
        assert!(jwks == AllProvidersJWKs {
            entries: vector[
                ProviderJWKs {
                    issuer: b"alice",
                    version: 0,
                    jwks: vector[
                        new_unsupported_jwk(b"key_id_0", b"key_content_0b"),
                        new_unsupported_jwk(b"key_id_3", b"key_content_3"),
                    ],
                },
                ProviderJWKs {
                    issuer: b"bob",
                    version: 222,
                    jwks: vector[
                        new_unsupported_jwk(b"key_id_2", b"key_content_2b"),
                    ],
                },
                ProviderJWKs {
                    issuer: b"carl",
                    version: 0,
                    jwks: vector[
                        new_rsa_jwk(
                            utf8(b"0ad1fec78504f447bae65bcf5afaedb65eec9e81"), // kid
                            utf8(b"RS256"), // alg
                            utf8(b"AQAB"), // e
                            utf8(b"sm72oBH-R2Rqt4hkjp66tz5qCtq42TMnVgZg2Pdm_zs7_-EoFyNs9sD1MKsZAFaBPXBHDiWywyaHhLgwETLN9hlJIZPzGCEtV3mXJFSYG-8L6t3kyKi9X1lUTZzbmNpE0tf-eMW-3gs3VQSBJQOcQnuiANxbSXwS3PFmi173C_5fDSuC1RoYGT6X3JqLc3DWUmBGucuQjPaUF0w6LMqEIy0W_WYbW7HImwANT6dT52T72md0JWZuAKsRRnRr_bvaUX8_e3K8Pb1K_t3dD6WSLvtmEfUnGQgLynVl3aV5sRYC0Hy_IkRgoxl2fd8AaZT1X_rdPexYpx152Pl_CHJ79Q"), // n
                        )
                    ],
                },
            ],
        }, 1);

        let patch = new_patch_remove_all();
        apply_patch(&mut jwks, patch);
        assert!(jwks == AllProvidersJWKs { entries: vector[] }, 1);
    }

    #[test(aptos_framework = @aptos_framework)]
    fun test_patched_jwks(aptos_framework: signer) acquires ObservedJWKs, PatchedJWKs, Patches {
        initialize_for_test(&aptos_framework);
        let jwk_0 = new_unsupported_jwk(b"key_id_0", b"key_payload_0");
        let jwk_1 = new_unsupported_jwk(b"key_id_1", b"key_payload_1");
        let jwk_2 = new_unsupported_jwk(b"key_id_2", b"key_payload_2");
        let jwk_3 = new_unsupported_jwk(b"key_id_3", b"key_payload_3");
        let jwk_3b = new_unsupported_jwk(b"key_id_3", b"key_payload_3b");

        // Fake observation from validators.
        upsert_into_observed_jwks(&aptos_framework, vector [
            ProviderJWKs {
                issuer: b"alice",
                version: 111,
                jwks: vector[jwk_0, jwk_1],
            },
            ProviderJWKs{
                issuer: b"bob",
                version: 222,
                jwks: vector[jwk_2, jwk_3],
            },
        ]);
        assert!(jwk_3 == get_patched_jwk(b"bob", b"key_id_3"), 1);
        assert!(option::some(jwk_3) == try_get_patched_jwk(b"bob", b"key_id_3"), 1);

        // Ignore all Bob's keys.
        set_patches(&aptos_framework, vector[
            new_patch_remove_issuer(b"bob"),
        ]);
        assert!(option::none() == try_get_patched_jwk(b"bob", b"key_id_3"), 1);

        // Update one of Bob's key..
        set_patches(&aptos_framework, vector[
            new_patch_upsert_jwk(b"bob", jwk_3b),
        ]);
        assert!(jwk_3b == get_patched_jwk(b"bob", b"key_id_3"), 1);
        assert!(option::some(jwk_3b) == try_get_patched_jwk(b"bob", b"key_id_3"), 1);

        // Wipe everything, then add some keys back.
        set_patches(&aptos_framework, vector[
            new_patch_remove_all(),
            new_patch_upsert_jwk(b"alice", jwk_1),
            new_patch_upsert_jwk(b"bob", jwk_3),
        ]);
        assert!(jwk_3 == get_patched_jwk(b"bob", b"key_id_3"), 1);
        assert!(option::some(jwk_3) == try_get_patched_jwk(b"bob", b"key_id_3"), 1);
    }
}
