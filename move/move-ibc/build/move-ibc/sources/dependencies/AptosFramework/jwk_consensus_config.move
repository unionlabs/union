/// Structs and functions related to JWK consensus configurations.
module aptos_framework::jwk_consensus_config {
    use std::error;
    use std::option;
    use std::string::String;
    use std::vector;
    use aptos_std::copyable_any;
    use aptos_std::copyable_any::Any;
    use aptos_std::simple_map;
    use aptos_framework::config_buffer;
    use aptos_framework::system_addresses;
    #[test_only]
    use std::string;
    #[test_only]
    use std::string::utf8;

    friend aptos_framework::reconfiguration_with_dkg;

    /// `ConfigV1` creation failed with duplicated providers given.
    const EDUPLICATE_PROVIDERS: u64 = 1;

    /// The configuration of the JWK consensus feature.
    struct JWKConsensusConfig has drop, key, store {
        /// A config variant packed as an `Any`.
        /// Currently the variant type is one of the following.
        /// - `ConfigOff`
        /// - `ConfigV1`
        variant: Any,
    }

    /// A JWK consensus config variant indicating JWK consensus should not run.
    struct ConfigOff has copy, drop, store {}

    struct OIDCProvider has copy, drop, store {
        name: String,
        config_url: String,
    }

    /// A JWK consensus config variant indicating JWK consensus should run to watch a given list of OIDC providers.
    struct ConfigV1 has copy, drop, store {
        oidc_providers: vector<OIDCProvider>,
    }

    /// Initialize the configuration. Used in genesis or governance.
    public fun initialize(framework: &signer, config: JWKConsensusConfig) {
        system_addresses::assert_aptos_framework(framework);
        if (!exists<JWKConsensusConfig>(@aptos_framework)) {
            move_to(framework, config);
        }
    }

    /// This can be called by on-chain governance to update JWK consensus configs for the next epoch.
    /// Example usage:
    /// ```
    /// use aptos_framework::jwk_consensus_config;
    /// use aptos_framework::aptos_governance;
    /// // ...
    /// let config = jwk_consensus_config::new_v1(vector[]);
    /// jwk_consensus_config::set_for_next_epoch(&framework_signer, config);
    /// aptos_governance::reconfigure(&framework_signer);
    /// ```
    public fun set_for_next_epoch(framework: &signer, config: JWKConsensusConfig) {
        system_addresses::assert_aptos_framework(framework);
        config_buffer::upsert(config);
    }

    /// Only used in reconfigurations to apply the pending `JWKConsensusConfig`, if there is any.
    public(friend) fun on_new_epoch(framework: &signer) acquires JWKConsensusConfig {
        system_addresses::assert_aptos_framework(framework);
        if (config_buffer::does_exist<JWKConsensusConfig>()) {
            let new_config = config_buffer::extract<JWKConsensusConfig>();
            if (exists<JWKConsensusConfig>(@aptos_framework)) {
                *borrow_global_mut<JWKConsensusConfig>(@aptos_framework) = new_config;
            } else {
                move_to(framework, new_config);
            };
        }
    }

    /// Construct a `JWKConsensusConfig` of variant `ConfigOff`.
    public fun new_off(): JWKConsensusConfig {
        JWKConsensusConfig {
            variant: copyable_any::pack( ConfigOff {} )
        }
    }

    /// Construct a `JWKConsensusConfig` of variant `ConfigV1`.
    ///
    /// Abort if the given provider list contains duplicated provider names.
    public fun new_v1(oidc_providers: vector<OIDCProvider>): JWKConsensusConfig {
        let name_set = simple_map::new<String, u64>();
        vector::for_each_ref(&oidc_providers, |provider| {
            let provider: &OIDCProvider = provider;
            let (_, old_value) = simple_map::upsert(&mut name_set, provider.name, 0);
            if (option::is_some(&old_value)) {
                abort(error::invalid_argument(EDUPLICATE_PROVIDERS))
            }
        });
        JWKConsensusConfig {
            variant: copyable_any::pack( ConfigV1 { oidc_providers } )
        }
    }

    /// Construct an `OIDCProvider` object.
    public fun new_oidc_provider(name: String, config_url: String): OIDCProvider {
        OIDCProvider { name, config_url }
    }

    #[test_only]
    fun enabled(): bool acquires JWKConsensusConfig {
        let variant= borrow_global<JWKConsensusConfig>(@aptos_framework).variant;
        let variant_type_name = *string::bytes(copyable_any::type_name(&variant));
        variant_type_name != b"0x1::jwk_consensus_config::ConfigOff"
    }

    #[test_only]
    fun initialize_for_testing(framework: &signer) {
        config_buffer::initialize(framework);
        initialize(framework, new_off());
    }

    #[test(framework = @0x1)]
    fun init_buffer_apply(framework: signer) acquires JWKConsensusConfig {
        initialize_for_testing(&framework);
        let config = new_v1(vector[
            new_oidc_provider(utf8(b"Bob"), utf8(b"https://bob.dev")),
            new_oidc_provider(utf8(b"Alice"), utf8(b"https://alice.io")),
        ]);
        set_for_next_epoch(&framework, config);
        on_new_epoch(&framework);
        assert!(enabled(), 1);

        set_for_next_epoch(&framework, new_off());
        on_new_epoch(&framework);
        assert!(!enabled(), 2)
    }

    #[test]
    #[expected_failure(abort_code = 0x010001, location = Self)]
    fun name_uniqueness_in_config_v1() {
        new_v1(vector[
            new_oidc_provider(utf8(b"Alice"), utf8(b"https://alice.info")),
            new_oidc_provider(utf8(b"Bob"), utf8(b"https://bob.dev")),
            new_oidc_provider(utf8(b"Alice"), utf8(b"https://alice.io")),
        ]);

    }
}
