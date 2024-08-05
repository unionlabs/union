/// This module defines structs and methods to initialize the gas schedule, which dictates how much
/// it costs to execute Move on the network.
module aptos_framework::gas_schedule {
    use std::bcs;
    use std::error;
    use std::string::String;
    use std::vector;
    use aptos_std::aptos_hash;
    use aptos_framework::chain_status;
    use aptos_framework::config_buffer;

    use aptos_framework::reconfiguration;
    use aptos_framework::system_addresses;
    use aptos_framework::util::from_bytes;
    use aptos_framework::storage_gas::StorageGasConfig;
    use aptos_framework::storage_gas;
    #[test_only]
    use std::bcs::to_bytes;

    friend aptos_framework::genesis;
    friend aptos_framework::reconfiguration_with_dkg;

    /// The provided gas schedule bytes are empty or invalid
    const EINVALID_GAS_SCHEDULE: u64 = 1;
    const EINVALID_GAS_FEATURE_VERSION: u64 = 2;
    const EINVALID_GAS_SCHEDULE_HASH: u64 = 3;

    struct GasEntry has store, copy, drop {
        key: String,
        val: u64,
    }

    struct GasSchedule has key, copy, drop {
        entries: vector<GasEntry>
    }

    struct GasScheduleV2 has key, copy, drop, store {
        feature_version: u64,
        entries: vector<GasEntry>,
    }

    /// Only called during genesis.
    public(friend) fun initialize(aptos_framework: &signer, gas_schedule_blob: vector<u8>) {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(!vector::is_empty(&gas_schedule_blob), error::invalid_argument(EINVALID_GAS_SCHEDULE));

        // TODO(Gas): check if gas schedule is consistent
        let gas_schedule: GasScheduleV2 = from_bytes(gas_schedule_blob);
        move_to<GasScheduleV2>(aptos_framework, gas_schedule);
    }

    /// Deprecated by `set_for_next_epoch()`.
    ///
    /// WARNING: calling this while randomness is enabled will trigger a new epoch without randomness!
    ///
    /// TODO: update all the tests that reference this function, then disable this function.
    public fun set_gas_schedule(aptos_framework: &signer, gas_schedule_blob: vector<u8>) acquires GasSchedule, GasScheduleV2 {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(!vector::is_empty(&gas_schedule_blob), error::invalid_argument(EINVALID_GAS_SCHEDULE));
        chain_status::assert_genesis();

        if (exists<GasScheduleV2>(@aptos_framework)) {
            let gas_schedule = borrow_global_mut<GasScheduleV2>(@aptos_framework);
            let new_gas_schedule: GasScheduleV2 = from_bytes(gas_schedule_blob);
            assert!(new_gas_schedule.feature_version >= gas_schedule.feature_version,
                error::invalid_argument(EINVALID_GAS_FEATURE_VERSION));
            // TODO(Gas): check if gas schedule is consistent
            *gas_schedule = new_gas_schedule;
        }
        else {
            if (exists<GasSchedule>(@aptos_framework)) {
                _ = move_from<GasSchedule>(@aptos_framework);
            };
            let new_gas_schedule: GasScheduleV2 = from_bytes(gas_schedule_blob);
            // TODO(Gas): check if gas schedule is consistent
            move_to<GasScheduleV2>(aptos_framework, new_gas_schedule);
        };

        // Need to trigger reconfiguration so validator nodes can sync on the updated gas schedule.
        reconfiguration::reconfigure();
    }

    /// Set the gas schedule for the next epoch, typically called by on-chain governance.
    /// Abort if the version of the given schedule is lower than the current version.
    ///
    /// Example usage:
    /// ```
    /// aptos_framework::gas_schedule::set_for_next_epoch(&framework_signer, some_gas_schedule_blob);
    /// aptos_framework::aptos_governance::reconfigure(&framework_signer);
    /// ```
    public fun set_for_next_epoch(aptos_framework: &signer, gas_schedule_blob: vector<u8>) acquires GasScheduleV2 {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(!vector::is_empty(&gas_schedule_blob), error::invalid_argument(EINVALID_GAS_SCHEDULE));
        let new_gas_schedule: GasScheduleV2 = from_bytes(gas_schedule_blob);
        if (exists<GasScheduleV2>(@aptos_framework)) {
            let cur_gas_schedule = borrow_global<GasScheduleV2>(@aptos_framework);
            assert!(
                new_gas_schedule.feature_version >= cur_gas_schedule.feature_version,
                error::invalid_argument(EINVALID_GAS_FEATURE_VERSION)
            );
        };
        config_buffer::upsert(new_gas_schedule);
    }

    /// Set the gas schedule for the next epoch, typically called by on-chain governance.
    /// Abort if the version of the given schedule is lower than the current version.
    /// Require a hash of the old gas schedule to be provided and will abort if the hashes mismatch.
    public fun set_for_next_epoch_check_hash(
        aptos_framework: &signer,
        old_gas_schedule_hash: vector<u8>,
        new_gas_schedule_blob: vector<u8>
    ) acquires GasScheduleV2 {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(!vector::is_empty(&new_gas_schedule_blob), error::invalid_argument(EINVALID_GAS_SCHEDULE));

        let new_gas_schedule: GasScheduleV2 = from_bytes(new_gas_schedule_blob);
        if (exists<GasScheduleV2>(@aptos_framework)) {
            let cur_gas_schedule = borrow_global<GasScheduleV2>(@aptos_framework);
            assert!(
                new_gas_schedule.feature_version >= cur_gas_schedule.feature_version,
                error::invalid_argument(EINVALID_GAS_FEATURE_VERSION)
            );
            let cur_gas_schedule_bytes = bcs::to_bytes(cur_gas_schedule);
            let cur_gas_schedule_hash = aptos_hash::sha3_512(cur_gas_schedule_bytes);
            assert!(
                cur_gas_schedule_hash == old_gas_schedule_hash,
                error::invalid_argument(EINVALID_GAS_SCHEDULE_HASH)
            );
        };

        config_buffer::upsert(new_gas_schedule);
    }

    /// Only used in reconfigurations to apply the pending `GasScheduleV2`, if there is any.
    public(friend) fun on_new_epoch(framework: &signer) acquires GasScheduleV2 {
        system_addresses::assert_aptos_framework(framework);
        if (config_buffer::does_exist<GasScheduleV2>()) {
            let new_gas_schedule = config_buffer::extract<GasScheduleV2>();
            if (exists<GasScheduleV2>(@aptos_framework)) {
                *borrow_global_mut<GasScheduleV2>(@aptos_framework) = new_gas_schedule;
            } else {
                move_to(framework, new_gas_schedule);
            }
        }
    }

    public fun set_storage_gas_config(aptos_framework: &signer, config: StorageGasConfig) {
        storage_gas::set_config(aptos_framework, config);
        // Need to trigger reconfiguration so the VM is guaranteed to load the new gas fee starting from the next
        // transaction.
        reconfiguration::reconfigure();
    }

    public fun set_storage_gas_config_for_next_epoch(aptos_framework: &signer, config: StorageGasConfig) {
        storage_gas::set_config(aptos_framework, config);
    }

    #[test(fx = @0x1)]
    #[expected_failure(abort_code=0x010002, location = Self)]
    fun set_for_next_epoch_should_abort_if_gas_version_is_too_old(fx: signer) acquires GasScheduleV2 {
        // Setup.
        let old_gas_schedule = GasScheduleV2 {
            feature_version: 1000,
            entries: vector[],
        };
        move_to(&fx, old_gas_schedule);

        // Setting an older version should not work.
        let new_gas_schedule = GasScheduleV2 {
            feature_version: 999,
            entries: vector[],
        };
        let new_bytes = to_bytes(&new_gas_schedule);
        set_for_next_epoch(&fx, new_bytes);
    }
}
