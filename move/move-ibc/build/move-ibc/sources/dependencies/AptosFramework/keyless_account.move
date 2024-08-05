/// This module is responsible for configuring keyless blockchain accounts which were introduced in
/// [AIP-61](https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-61.md).
module aptos_framework::keyless_account {
    use std::bn254_algebra;
    use std::config_buffer;
    use std::option;
    use std::option::Option;
    use std::signer;
    use std::string::String;
    use std::vector;
    use aptos_std::crypto_algebra;
    use aptos_std::ed25519;
    use aptos_framework::chain_status;
    use aptos_framework::system_addresses;

    // The `aptos_framework::reconfiguration_with_dkg` module needs to be able to call `on_new_epoch`.
    friend aptos_framework::reconfiguration_with_dkg;

    /// The training wheels PK needs to be 32 bytes long.
    const E_TRAINING_WHEELS_PK_WRONG_SIZE : u64 = 1;

    /// A serialized BN254 G1 point is invalid.
    const E_INVALID_BN254_G1_SERIALIZATION: u64 = 2;

    /// A serialized BN254 G2 point is invalid.
    const E_INVALID_BN254_G2_SERIALIZATION: u64 = 3;

    #[resource_group(scope = global)]
    struct Group {}

    #[resource_group_member(group = aptos_framework::keyless_account::Group)]
    /// The 288-byte Groth16 verification key (VK) for the ZK relation that implements keyless accounts
    struct Groth16VerificationKey has key, store, drop {
        /// 32-byte serialization of `alpha * G`, where `G` is the generator of `G1`.
        alpha_g1: vector<u8>,
        /// 64-byte serialization of `alpha * H`, where `H` is the generator of `G2`.
        beta_g2: vector<u8>,
        /// 64-byte serialization of `gamma * H`, where `H` is the generator of `G2`.
        gamma_g2: vector<u8>,
        /// 64-byte serialization of `delta * H`, where `H` is the generator of `G2`.
        delta_g2: vector<u8>,
        /// `\forall i \in {0, ..., \ell}, 64-byte serialization of gamma^{-1} * (beta * a_i + alpha * b_i + c_i) * H`, where
        /// `H` is the generator of `G1` and `\ell` is 1 for the ZK relation.
        gamma_abc_g1: vector<vector<u8>>,
    }

    #[resource_group_member(group = aptos_framework::keyless_account::Group)]
    struct Configuration has key, store, drop, copy {
        /// An override `aud` for the identity of a recovery service, which will help users recover their keyless accounts
        /// associated with dapps or wallets that have disappeared.
        /// IMPORTANT: This recovery service **cannot** on its own take over user accounts; a user must first sign in
        /// via OAuth in the recovery service in order to allow it to rotate any of that user's keyless accounts.
        override_aud_vals: vector<String>,
        /// No transaction can have more than this many keyless signatures.
        max_signatures_per_txn: u16,
        /// How far in the future from the JWT issued at time the EPK expiry can be set.
        max_exp_horizon_secs: u64,
        /// The training wheels PK, if training wheels are on
        training_wheels_pubkey: Option<vector<u8>>,
        /// The max length of an ephemeral public key supported in our circuit (93 bytes)
        max_commited_epk_bytes: u16,
        /// The max length of the value of the JWT's `iss` field supported in our circuit (e.g., `"https://accounts.google.com"`)
        max_iss_val_bytes: u16,
        /// The max length of the JWT field name and value (e.g., `"max_age":"18"`) supported in our circuit
        max_extra_field_bytes: u16,
        /// The max length of the base64url-encoded JWT header in bytes supported in our circuit
        max_jwt_header_b64_bytes: u32,
    }

    #[test_only]
    public fun initialize_for_test(fx: &signer, vk: Groth16VerificationKey, constants: Configuration) {
        system_addresses::assert_aptos_framework(fx);

        move_to(fx, vk);
        move_to(fx, constants);
    }

    public fun new_groth16_verification_key(alpha_g1: vector<u8>,
                                            beta_g2: vector<u8>,
                                            gamma_g2: vector<u8>,
                                            delta_g2: vector<u8>,
                                            gamma_abc_g1: vector<vector<u8>>
    ): Groth16VerificationKey {
        Groth16VerificationKey {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1,
        }
    }

    public fun new_configuration(
        override_aud_val: vector<String>,
        max_signatures_per_txn: u16,
        max_exp_horizon_secs: u64,
        training_wheels_pubkey: Option<vector<u8>>,
        max_commited_epk_bytes: u16,
        max_iss_val_bytes: u16,
        max_extra_field_bytes: u16,
        max_jwt_header_b64_bytes: u32
    ): Configuration {
        Configuration {
            override_aud_vals: override_aud_val,
            max_signatures_per_txn,
            max_exp_horizon_secs,
            training_wheels_pubkey,
            max_commited_epk_bytes,
            max_iss_val_bytes,
            max_extra_field_bytes,
            max_jwt_header_b64_bytes,
        }
    }

    /// Pre-validate the VK to actively-prevent incorrect VKs from being set on-chain.
    fun validate_groth16_vk(vk: &Groth16VerificationKey) {
        // Could be leveraged to speed up the VM deserialization of the VK by 2x, since it can assume the points are valid.
        assert!(option::is_some(&crypto_algebra::deserialize<bn254_algebra::G1, bn254_algebra::FormatG1Compr>(&vk.alpha_g1)), E_INVALID_BN254_G1_SERIALIZATION);
        assert!(option::is_some(&crypto_algebra::deserialize<bn254_algebra::G2, bn254_algebra::FormatG2Compr>(&vk.beta_g2)), E_INVALID_BN254_G2_SERIALIZATION);
        assert!(option::is_some(&crypto_algebra::deserialize<bn254_algebra::G2, bn254_algebra::FormatG2Compr>(&vk.gamma_g2)), E_INVALID_BN254_G2_SERIALIZATION);
        assert!(option::is_some(&crypto_algebra::deserialize<bn254_algebra::G2, bn254_algebra::FormatG2Compr>(&vk.delta_g2)), E_INVALID_BN254_G2_SERIALIZATION);
        for (i in 0..vector::length(&vk.gamma_abc_g1)) {
            assert!(option::is_some(&crypto_algebra::deserialize<bn254_algebra::G1, bn254_algebra::FormatG1Compr>(vector::borrow(&vk.gamma_abc_g1, i))), E_INVALID_BN254_G1_SERIALIZATION);
        };
    }

    /// Sets the Groth16 verification key, only callable during genesis. To call during governance proposals, use
    /// `set_groth16_verification_key_for_next_epoch`.
    ///
    /// WARNING: See `set_groth16_verification_key_for_next_epoch` for caveats.
    public fun update_groth16_verification_key(fx: &signer, vk: Groth16VerificationKey) {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();
        // There should not be a previous resource set here.
        move_to(fx, vk);
    }

    /// Sets the keyless configuration, only callable during genesis. To call during governance proposals, use
    /// `set_configuration_for_next_epoch`.
    ///
    /// WARNING: See `set_configuration_for_next_epoch` for caveats.
    public fun update_configuration(fx: &signer, config: Configuration) {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();
        // There should not be a previous resource set here.
        move_to(fx, config);
    }

    #[deprecated]
    public fun update_training_wheels(fx: &signer, pk: Option<vector<u8>>) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();

        if (option::is_some(&pk)) {
            assert!(vector::length(option::borrow(&pk)) == 32, E_TRAINING_WHEELS_PK_WRONG_SIZE)
        };

        let config = borrow_global_mut<Configuration>(signer::address_of(fx));
        config.training_wheels_pubkey = pk;
    }

    #[deprecated]
    public fun update_max_exp_horizon(fx: &signer, max_exp_horizon_secs: u64) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();

        let config = borrow_global_mut<Configuration>(signer::address_of(fx));
        config.max_exp_horizon_secs = max_exp_horizon_secs;
    }

    #[deprecated]
    public fun remove_all_override_auds(fx: &signer) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();

        let config = borrow_global_mut<Configuration>(signer::address_of(fx));
        config.override_aud_vals = vector[];
    }

    #[deprecated]
    public fun add_override_aud(fx: &signer, aud: String) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);
        chain_status::assert_genesis();

        let config = borrow_global_mut<Configuration>(signer::address_of(fx));
        vector::push_back(&mut config.override_aud_vals, aud);
    }

    /// Queues up a change to the Groth16 verification key. The change will only be effective after reconfiguration.
    /// Only callable via governance proposal.
    ///
    /// WARNING: To mitigate against DoS attacks, a VK change should be done together with a training wheels PK change,
    /// so that old ZKPs for the old VK cannot be replayed as potentially-valid ZKPs.
    ///
    /// WARNING: If a malicious key is set, this would lead to stolen funds.
    public fun set_groth16_verification_key_for_next_epoch(fx: &signer, vk: Groth16VerificationKey) {
        system_addresses::assert_aptos_framework(fx);
        config_buffer::upsert<Groth16VerificationKey>(vk);
    }


    /// Queues up a change to the keyless configuration. The change will only be effective after reconfiguration. Only
    /// callable via governance proposal.
    ///
    /// WARNING: A malicious `Configuration` could lead to DoS attacks, create liveness issues, or enable a malicious
    /// recovery service provider to phish users' accounts.
    public fun set_configuration_for_next_epoch(fx: &signer, config: Configuration) {
        system_addresses::assert_aptos_framework(fx);
        config_buffer::upsert<Configuration>(config);
    }

    /// Convenience method to queue up a change to the training wheels PK. The change will only be effective after
    /// reconfiguration. Only callable via governance proposal.
    ///
    /// WARNING: If a malicious key is set, this *could* lead to stolen funds.
    public fun update_training_wheels_for_next_epoch(fx: &signer, pk: Option<vector<u8>>) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);

        // If a PK is being set, validate it first.
        if (option::is_some(&pk)) {
            let bytes = *option::borrow(&pk);
            let vpk = ed25519::new_validated_public_key_from_bytes(bytes);
            assert!(option::is_some(&vpk), E_TRAINING_WHEELS_PK_WRONG_SIZE)
        };

        let config = if (config_buffer::does_exist<Configuration>()) {
            config_buffer::extract<Configuration>()
        } else {
            *borrow_global<Configuration>(signer::address_of(fx))
        };

        config.training_wheels_pubkey = pk;

        set_configuration_for_next_epoch(fx, config);
    }

    /// Convenience method to queues up a change to the max expiration horizon. The change will only be effective after
    /// reconfiguration. Only callable via governance proposal.
    public fun update_max_exp_horizon_for_next_epoch(fx: &signer, max_exp_horizon_secs: u64) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);

        let config = if (config_buffer::does_exist<Configuration>()) {
            config_buffer::extract<Configuration>()
        } else {
            *borrow_global<Configuration>(signer::address_of(fx))
        };

        config.max_exp_horizon_secs = max_exp_horizon_secs;

        set_configuration_for_next_epoch(fx, config);
    }

    /// Convenience method to queue up clearing the set of override `aud`'s. The change will only be effective after
    /// reconfiguration. Only callable via governance proposal.
    ///
    /// WARNING: When no override `aud` is set, recovery of keyless accounts associated with applications that disappeared
    /// is no longer possible.
    public fun remove_all_override_auds_for_next_epoch(fx: &signer) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);

        let config = if (config_buffer::does_exist<Configuration>()) {
            config_buffer::extract<Configuration>()
        } else {
            *borrow_global<Configuration>(signer::address_of(fx))
        };

        config.override_aud_vals = vector[];

        set_configuration_for_next_epoch(fx, config);
    }

    /// Convenience method to queue up an append to to the set of override `aud`'s. The change will only be effective
    /// after reconfiguration. Only callable via governance proposal.
    ///
    /// WARNING: If a malicious override `aud` is set, this *could* lead to stolen funds.
    public fun add_override_aud_for_next_epoch(fx: &signer, aud: String) acquires Configuration {
        system_addresses::assert_aptos_framework(fx);

        let config = if (config_buffer::does_exist<Configuration>()) {
            config_buffer::extract<Configuration>()
        } else {
            *borrow_global<Configuration>(signer::address_of(fx))
        };

        vector::push_back(&mut config.override_aud_vals, aud);

        set_configuration_for_next_epoch(fx, config);
    }

    /// Only used in reconfigurations to apply the queued up configuration changes, if there are any.
    public(friend) fun on_new_epoch(fx: &signer) acquires Groth16VerificationKey, Configuration {
        system_addresses::assert_aptos_framework(fx);

        if (config_buffer::does_exist<Groth16VerificationKey>()) {
            let vk = config_buffer::extract();
            if (exists<Groth16VerificationKey>(@aptos_framework)) {
                *borrow_global_mut<Groth16VerificationKey>(@aptos_framework) = vk;
            } else {
                move_to(fx, vk);
            }
        };

        if (config_buffer::does_exist<Configuration>()) {
            let config = config_buffer::extract();
            if (exists<Configuration>(@aptos_framework)) {
                *borrow_global_mut<Configuration>(@aptos_framework) = config;
            } else {
                move_to(fx, config);
            }
        };
    }
}
