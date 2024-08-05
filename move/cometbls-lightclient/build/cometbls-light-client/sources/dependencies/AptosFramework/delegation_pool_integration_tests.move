#[test_only]
module aptos_framework::delegation_pool_integration_tests {
    use std::features;
    use std::signer;

    use aptos_std::bls12381;
    use aptos_std::stake;
    use aptos_std::vector;

    use aptos_framework::account;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::coin;
    use aptos_framework::reconfiguration;
    use aptos_framework::delegation_pool as dp;
    use aptos_framework::timestamp;

    #[test_only]
    const EPOCH_DURATION: u64 = 60;

    #[test_only]
    const LOCKUP_CYCLE_SECONDS: u64 = 3600;

    #[test_only]
    const ONE_APT: u64 = 100000000;

    #[test_only]
    const VALIDATOR_STATUS_PENDING_ACTIVE: u64 = 1;
    const VALIDATOR_STATUS_ACTIVE: u64 = 2;
    const VALIDATOR_STATUS_PENDING_INACTIVE: u64 = 3;
    const VALIDATOR_STATUS_INACTIVE: u64 = 4;

    #[test_only]
    const DELEGATION_POOLS: u64 = 11;

    #[test_only]
    const MODULE_EVENT: u64 = 26;

    #[test_only]
    public fun initialize_for_test(aptos_framework: &signer) {
        initialize_for_test_custom(
            aptos_framework,
            100 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            100,
            1000000
        );
    }

    #[test_only]
    public fun end_epoch() {
        stake::end_epoch();
        reconfiguration::reconfigure_for_test_custom();
    }

    // Convenient function for setting up all required stake initializations.
    #[test_only]
    public fun initialize_for_test_custom(
        aptos_framework: &signer,
        minimum_stake: u64,
        maximum_stake: u64,
        recurring_lockup_secs: u64,
        allow_validator_set_change: bool,
        rewards_rate_numerator: u64,
        rewards_rate_denominator: u64,
        voting_power_increase_limit: u64,
    ) {
        account::create_account_for_test(signer::address_of(aptos_framework));
        stake::initialize_for_test_custom(
            aptos_framework,
            minimum_stake,
            maximum_stake,
            recurring_lockup_secs,
            allow_validator_set_change,
            rewards_rate_numerator,
            rewards_rate_denominator,
            voting_power_increase_limit
        );
        reconfiguration::initialize_for_test(aptos_framework);
        features::change_feature_flags_for_testing(aptos_framework, vector[DELEGATION_POOLS, MODULE_EVENT], vector[]);
    }

    #[test_only]
    public fun mint_and_add_stake(account: &signer, amount: u64) {
        stake::mint(account, amount);
        dp::add_stake(account, dp::get_owned_pool_address(signer::address_of(account)), amount);
    }

    #[test_only]
    public fun initialize_test_validator(
        public_key: &bls12381::PublicKey,
        proof_of_possession: &bls12381::ProofOfPossession,
        validator: &signer,
        amount: u64,
        should_join_validator_set: bool,
        should_end_epoch: bool,
    ) {
        let validator_address = signer::address_of(validator);
        if (!account::exists_at(signer::address_of(validator))) {
            account::create_account_for_test(validator_address);
        };

        dp::initialize_delegation_pool(validator, 0, vector::empty<u8>());
        validator_address = dp::get_owned_pool_address(validator_address);

        let pk_bytes = bls12381::public_key_to_bytes(public_key);
        let pop_bytes = bls12381::proof_of_possession_to_bytes(proof_of_possession);
        stake::rotate_consensus_key(validator, validator_address, pk_bytes, pop_bytes);

        if (amount > 0) {
            mint_and_add_stake(validator, amount);
        };

        if (should_join_validator_set) {
            stake::join_validator_set(validator, validator_address);
        };
        if (should_end_epoch) {
            end_epoch();
        };
    }

    #[test_only]
    public fun generate_identity(): (bls12381::SecretKey, bls12381::PublicKey, bls12381::ProofOfPossession) {
        let (sk, pkpop) = bls12381::generate_keys();
        let pop = bls12381::generate_proof_of_possession(&sk);
        let unvalidated_pk = bls12381::public_key_with_pop_to_normal(&pkpop);
        (sk, unvalidated_pk, pop)
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10007, location = aptos_framework::stake)]
    public entry fun test_inactive_validator_can_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, false, false);

        // Add more stake to exceed max. This should fail.
        mint_and_add_stake(validator, 9900 * ONE_APT + 1);
    }

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x10007, location = aptos_framework::stake)]
    public entry fun test_pending_active_validator_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            100000
        );
        // Have one validator join the set to ensure the validator set is not empty when main validator joins.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, true, true);

        // Validator 2 joins validator set but epoch has not ended so validator is in pending_active state.
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, false);

        // Add more stake to exceed max. This should fail.
        mint_and_add_stake(validator_2, 9900 * ONE_APT + 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10007, location = aptos_framework::stake)]
    public entry fun test_active_validator_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        // Validator joins validator set and waits for epoch end so it's in the validator set.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Add more stake to exceed max. This should fail.
        mint_and_add_stake(validator, 9900 * ONE_APT + 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10007, location = aptos_framework::stake)]
    public entry fun test_active_validator_with_pending_inactive_stake_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        // Validator joins validator set and waits for epoch end so it's in the validator set.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Request to unlock 50 coins, which go to pending_inactive. Validator has 50 remaining in active.
        let pool_address = dp::get_owned_pool_address(signer::address_of(validator));
        dp::unlock(validator, pool_address, 50 * ONE_APT);
        stake::assert_validator_state(pool_address, 50 * ONE_APT, 0, 0, 50 * ONE_APT, 0);

        // Add 9900 APT + 1 more. Total stake is 50 (active) + 50 (pending_inactive) + 9900 APT + 1 > 10000 so still exceeding max.
        mint_and_add_stake(validator, 9900 * ONE_APT + 1);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x10007, location = aptos_framework::stake)]
    public entry fun test_pending_inactive_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, true);

        // Leave validator set so validator is in pending_inactive state.
        stake::leave_validator_set(validator_1, dp::get_owned_pool_address(signer::address_of(validator_1)));

        // Add 9900 APT + 1 more. Total stake is 50 (active) + 50 (pending_inactive) + 9900 APT + 1 > 10000 so still exceeding max.
        mint_and_add_stake(validator_1, 9900 * ONE_APT + 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_end_to_end(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Validator has a lockup now that they've joined the validator set.
        let validator_address = signer::address_of(validator);
        let pool_address = dp::get_owned_pool_address(validator_address);
        assert!(stake::get_remaining_lockup_secs(pool_address) == LOCKUP_CYCLE_SECONDS, 1);

        // Validator adds more stake while already being active.
        // The added stake should go to pending_active to wait for activation when next epoch starts.
        stake::mint(validator, 900 * ONE_APT);
        dp::add_stake(validator, pool_address, 100 * ONE_APT);
        assert!(coin::balance<AptosCoin>(validator_address) == 800 * ONE_APT, 2);
        stake::assert_validator_state(pool_address, 100 * ONE_APT, 0, 100 * ONE_APT, 0, 0);

        // Pending_active stake is activated in the new epoch.
        // Rewards of 1 coin are also distributed for the existing active stake of 100 coins.
        end_epoch();
        assert!(stake::get_validator_state(pool_address) == VALIDATOR_STATUS_ACTIVE, 3);
        stake::assert_validator_state(pool_address, 201 * ONE_APT, 0, 0, 0, 0);

        // Request unlock of 100 coins. These 100 coins are moved to pending_inactive and will be unlocked when the
        // current lockup expires.
        dp::unlock(validator, pool_address, 100 * ONE_APT);
        stake::assert_validator_state(pool_address, 10100000001, 0, 0, 9999999999, 0);

        // Enough time has passed so the current lockup cycle should have ended.
        // The first epoch after the lockup cycle ended should automatically move unlocked (pending_inactive) stake
        // to inactive.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        // Rewards were also minted to pending_inactive, which got all moved to inactive.
        stake::assert_validator_state(pool_address, 10201000001, 10099999998, 0, 0, 0);
        // Lockup is renewed and validator is still active.
        assert!(stake::get_validator_state(pool_address) == VALIDATOR_STATUS_ACTIVE, 4);
        assert!(stake::get_remaining_lockup_secs(pool_address) == LOCKUP_CYCLE_SECONDS, 5);

        // Validator withdraws from inactive stake multiple times.
        dp::withdraw(validator, pool_address, 50 * ONE_APT);
        assert!(coin::balance<AptosCoin>(validator_address) == 84999999999, 6);
        stake::assert_validator_state(pool_address, 10201000001, 5099999999, 0, 0, 0);
        dp::withdraw(validator, pool_address, 51 * ONE_APT);
        assert!(coin::balance<AptosCoin>(validator_address) == 90099999998, 7);
        stake::assert_validator_state(pool_address, 10201000001, 0, 0, 0, 0);

        // Enough time has passed again and the validator's lockup is renewed once more. Validator is still active.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();

        assert!(stake::get_validator_state(pool_address) == VALIDATOR_STATUS_ACTIVE, 8);
        assert!(stake::get_remaining_lockup_secs(pool_address) == LOCKUP_CYCLE_SECONDS, 9);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x1000D, location = aptos_framework::stake)]
    public entry fun test_inactive_validator_cannot_join_if_exceed_increase_limit(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            50
        );
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, false, false);

        // Validator 1 needs to be in the set so validator 2's added stake counts against the limit.
        stake::join_validator_set(validator_1, dp::get_owned_pool_address(signer::address_of(validator_1)));
        end_epoch();

        // Validator 2 joins the validator set but their stake would lead to exceeding the voting power increase limit.
        // Therefore, this should fail.
        stake::join_validator_set(validator_2, dp::get_owned_pool_address(signer::address_of(validator_2)));
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_pending_active_validator_can_add_more_stake(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            10000
        );
        // Need 1 validator to be in the active validator set so joining limit works.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, false, true);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, false, false);

        // Add more stake while still pending_active.
        let validator_2_address = dp::get_owned_pool_address(signer::address_of(validator_2));
        stake::join_validator_set(validator_2, validator_2_address);
        assert!(stake::get_validator_state(validator_2_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 0);
        mint_and_add_stake(validator_2, 100 * ONE_APT);
        stake::assert_validator_state(validator_2_address, 200 * ONE_APT, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x1000D, location = aptos_framework::stake)]
    public entry fun test_pending_active_validator_cannot_add_more_stake_than_limit(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        // 100% voting power increase is allowed in each epoch.
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            100
        );
        // Need 1 validator to be in the active validator set so joining limit works.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, true, true);

        // Validator 2 joins the validator set but epoch has not ended so they're still pending_active.
        // Current voting power increase is already 100%. This is not failing yet.
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, false);

        // Add more stake, which now exceeds the 100% limit. This should fail.
        mint_and_add_stake(validator_2, ONE_APT);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_pending_active_validator_leaves_validator_set(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        // Validator joins but epoch hasn't ended, so the validator is still pending_active.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, false);
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 0);

        // Leave the validator set immediately.
        stake::leave_validator_set(validator, validator_address);
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000D, location = aptos_framework::stake)]
    public entry fun test_active_validator_cannot_add_more_stake_than_limit_in_multiple_epochs(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            50
        );
        // Add initial stake and join the validator set.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        stake::assert_validator_state(validator_address, 100 * ONE_APT, 0, 0, 0, 0);
        end_epoch();
        stake::assert_validator_state(validator_address, 110 * ONE_APT, 0, 0, 0, 0);
        end_epoch();
        stake::assert_validator_state(validator_address, 121 * ONE_APT, 0, 0, 0, 0);
        // Add more than 50% limit. The following line should fail.
        mint_and_add_stake(validator, 99 * ONE_APT);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000D, location = aptos_framework::stake)]
    public entry fun test_active_validator_cannot_add_more_stake_than_limit(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            50
        );
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Add more than 50% limit. This should fail.
        mint_and_add_stake(validator, 50 * ONE_APT + 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_unlock_partial_stake(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        // Reward rate = 10%.
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            100
        );
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Unlock half of the coins.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 1);
        dp::unlock(validator, validator_address, 50 * ONE_APT);
        stake::assert_validator_state(validator_address, 50 * ONE_APT, 0, 0, 50 * ONE_APT, 0);

        // Enough time has passed so the current lockup cycle should have ended.
        // 50 coins should have unlocked while the remaining 51 (50 + rewards) should stay locked for another cycle.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        // Validator received rewards in both active and pending inactive.
        stake::assert_validator_state(validator_address, 55 * ONE_APT, 55 * ONE_APT, 0, 0, 0);
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 3);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_can_withdraw_all_stake_and_rewards_at_once(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 0);

        // One more epoch passes to generate rewards.
        end_epoch();
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 1);
        stake::assert_validator_state(validator_address, 101 * ONE_APT, 0, 0, 0, 0);

        // Unlock all coins while still having a lockup.
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS - EPOCH_DURATION, 2);
        dp::unlock(validator, validator_address, 101 * ONE_APT);
        stake::assert_validator_state(validator_address, 0, 0, 0, 101 * ONE_APT, 0);

        // One more epoch passes while the current lockup cycle (3600 secs) has not ended.
        timestamp::fast_forward_seconds(1000);
        end_epoch();
        // Validator should not be removed from the validator set since their 100 coins in pending_inactive state should
        // still count toward voting power.
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 3);
        stake::assert_validator_state(validator_address, 0, 0, 0, 10201000000, 0);

        // Enough time has passed so the current lockup cycle should have ended. Funds are now fully unlocked.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        stake::assert_validator_state(validator_address, 0, 10303010000, 0, 0, 0);
        // Validator has been kicked out of the validator set as their stake is 0 now.
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 4);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10006, location = aptos_framework::delegation_pool)]
    public entry fun test_active_validator_unlocking_more_than_available_stake_should_cap(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, false, false);

        // Validator unlocks more stake than they have active. This should limit the unlock to 100.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        dp::unlock(validator, validator_address, 200 * ONE_APT);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_withdraw_should_cap_by_inactive_stake(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        // Initial balance = 900 (idle) + 100 (staked) = 1000.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);
        stake::mint(validator, 900 * ONE_APT);

        // Validator unlocks stake.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        dp::unlock(validator, validator_address, 100 * ONE_APT);
        // Enough time has passed so the stake is fully unlocked.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();

        // Validator can only withdraw a max of 100 unlocked coins even if they request to withdraw more than 100.
        dp::withdraw(validator, validator_address, 200 * ONE_APT);

        // Receive back all coins with an extra 1 for rewards.
        assert!(coin::balance<AptosCoin>(signer::address_of(validator)) == 100100000000, 2);
        stake::assert_validator_state(validator_address, 0, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_can_reactivate_pending_inactive_stake(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Validator unlocks stake, which gets moved into pending_inactive.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        dp::unlock(validator, validator_address, 50 * ONE_APT);
        stake::assert_validator_state(validator_address, 50 * ONE_APT, 0, 0, 50 * ONE_APT, 0);

        // Validator can reactivate pending_inactive stake.
        dp::reactivate_stake(validator, validator_address, 50 * ONE_APT);
        stake::assert_validator_state(validator_address, 100 * ONE_APT, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_reactivate_more_than_available_pending_inactive_stake_should_cap(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Validator tries to reactivate more than available pending_inactive stake, which should limit to 50.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        dp::unlock(validator, validator_address, 50 * ONE_APT);
        stake::assert_validator_state(validator_address, 50 * ONE_APT, 0, 0, 50 * ONE_APT, 0);
        dp::reactivate_stake(validator, validator_address, 51 * ONE_APT);
        stake::assert_validator_state(validator_address, 100 * ONE_APT, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_having_insufficient_remaining_stake_after_withdrawal_gets_kicked(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);

        // Unlock enough coins that the remaining is not enough to meet the min required.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 1);
        dp::unlock(validator, validator_address, 50 * ONE_APT);
        stake::assert_validator_state(validator_address, 50 * ONE_APT, 0, 0, 50 * ONE_APT, 0);

        // Enough time has passed so the current lockup cycle should have ended.
        // 50 coins should have unlocked while the remaining 51 (50 + rewards) is not enough so the validator is kicked
        // from the validator set.
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 2);
        stake::assert_validator_state(validator_address, 5050000000, 5050000000, 0, 0, 0);
        // Lockup is no longer renewed since the validator is no longer a part of the validator set.
        assert!(stake::get_remaining_lockup_secs(validator_address) == 0, 3);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123, validator_2 = @0x234)]
    public entry fun test_active_validator_leaves_staking_but_still_has_a_lockup(
        aptos_framework: &signer,
        validator: &signer,
        validator_2: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator, 100 * ONE_APT, true, false);
        // We need a second validator here just so the first validator can leave.
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, true);

        // Leave the validator set while still having a lockup.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 0);
        stake::leave_validator_set(validator, validator_address);
        // Validator is in pending_inactive state but is technically still part of the validator set.
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_PENDING_INACTIVE, 2);
        stake::assert_validator_state(validator_address, 100 * ONE_APT, 0, 0, 0, 1);
        end_epoch();

        // Epoch has ended so validator is no longer part of the validator set.
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 3);
        // However, their stake, including rewards, should still subject to the existing lockup.
        stake::assert_validator_state(validator_address, 101 * ONE_APT, 0, 0, 0, 1);
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS - EPOCH_DURATION, 4);

        // If they try to unlock, their stake is moved to pending_inactive and would only be withdrawable after the
        // lockup has expired.
        dp::unlock(validator, validator_address, 50 * ONE_APT);
        stake::assert_validator_state(validator_address, 5100000001, 0, 0, 4999999999, 1);
        // A couple of epochs passed but lockup has not expired so the validator's stake remains the same.
        end_epoch();
        end_epoch();
        end_epoch();
        stake::assert_validator_state(validator_address, 5100000001, 0, 0, 4999999999, 1);
        // Fast forward enough so the lockup expires. Now the validator can just call withdraw directly to withdraw
        // pending_inactive stakes.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);

        // an epoch passes but validator is inactive and its pending_inactive stake is not explicitly inactivated
        end_epoch();
        // pending_inactive stake should not be inactivated
        stake::assert_validator_state(validator_address, 5100000001, 0, 0, 4999999999, 1);
        // delegator's stake should be in sync with states reported by stake pool
        dp::assert_delegation(signer::address_of(validator), validator_address, 5100000001, 0, 4999999999);

        dp::withdraw(validator, validator_address, 4999999999);
        stake::assert_validator_state(validator_address, 5100000001, 0, 0, 0, 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123, validator_2 = @0x234)]
    public entry fun test_active_validator_leaves_staking_and_rejoins_with_expired_lockup_should_be_renewed(
        aptos_framework: &signer,
        validator: &signer,
        validator_2: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator, 100 * ONE_APT, true, false);
        // We need a second validator here just so the first validator can leave.
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, true);

        // Leave the validator set while still having a lockup.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 0);
        stake::leave_validator_set(validator, validator_address);
        end_epoch();

        // Fast forward enough so the lockup expires.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        assert!(stake::get_remaining_lockup_secs(validator_address) == 0, 1);

        // Validator rejoins the validator set. Once the current epoch ends, their lockup should be automatically
        // renewed.
        stake::join_validator_set(validator, validator_address);
        end_epoch();
        assert!(stake::get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        assert!(stake::get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 2);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_pending_inactive_validator_does_not_count_in_increase_limit(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            10,
            50
        );
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, true, false);
        // We need a second validator here just so the first validator can leave.
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, true);

        // Validator 1 leaves the validator set. Epoch has not ended so they're still pending_inactive.
        stake::leave_validator_set(validator_1, dp::get_owned_pool_address(signer::address_of(validator_1)));
        // Validator 1 adds more stake. This should not succeed as it should not count as a voting power increase.
        mint_and_add_stake(validator_1, 51 * ONE_APT);
    }

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234, validator_3 = @0x345)]
    public entry fun test_multiple_validators_join_and_leave(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
        validator_3: &signer
    ) {
        initialize_for_test_custom(
            aptos_framework,
            100 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            100,
            100
        );
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        let (_sk_3, pk_3, pop_3) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_3, &pop_3, validator_3, 100 * ONE_APT, false, false);

        let validator_1_address = dp::get_owned_pool_address(signer::address_of(validator_1));
        let validator_2_address = dp::get_owned_pool_address(signer::address_of(validator_2));
        let validator_3_address = dp::get_owned_pool_address(signer::address_of(validator_3));

        // Validator 1 and 2 join the validator set.
        stake::join_validator_set(validator_2, validator_2_address);
        stake::join_validator_set(validator_1, validator_1_address);
        end_epoch();
        assert!(stake::get_validator_state(validator_1_address) == VALIDATOR_STATUS_ACTIVE, 0);
        assert!(stake::get_validator_state(validator_2_address) == VALIDATOR_STATUS_ACTIVE, 1);

        // Validator indices is the reverse order of the joining order.
        stake::assert_validator_state(validator_1_address, 100 * ONE_APT, 0, 0, 0, 0);
        stake::assert_validator_state(validator_2_address, 100 * ONE_APT, 0, 0, 0, 1);

        // Validator 1 rotates consensus key. Validator 2 leaves. Validator 3 joins.
        let (_sk_1b, pk_1b, pop_1b) = generate_identity();
        let pk_1b_bytes = bls12381::public_key_to_bytes(&pk_1b);
        let pop_1b_bytes = bls12381::proof_of_possession_to_bytes(&pop_1b);
        stake::rotate_consensus_key(validator_1, validator_1_address, pk_1b_bytes, pop_1b_bytes);
        stake::leave_validator_set(validator_2, validator_2_address);
        stake::join_validator_set(validator_3, validator_3_address);
        // Validator 2 is not effectively removed until next epoch.
        assert!(stake::get_validator_state(validator_2_address) == VALIDATOR_STATUS_PENDING_INACTIVE, 6);

        // Validator 3 is not effectively added until next epoch.
        assert!(stake::get_validator_state(validator_3_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 7);

        // Changes applied after new epoch
        end_epoch();
        assert!(stake::get_validator_state(validator_1_address) == VALIDATOR_STATUS_ACTIVE, 8);
        stake::assert_validator_state(validator_1_address, 101 * ONE_APT, 0, 0, 0, 0);
        assert!(stake::get_validator_state(validator_2_address) == VALIDATOR_STATUS_INACTIVE, 9);
        // The validator index of validator 2 stays the same but this doesn't matter as the next time they rejoin the
        // validator set, their index will get set correctly.
        stake::assert_validator_state(validator_2_address, 101 * ONE_APT, 0, 0, 0, 1);
        assert!(stake::get_validator_state(validator_3_address) == VALIDATOR_STATUS_ACTIVE, 10);
        stake::assert_validator_state(validator_3_address, 100 * ONE_APT, 0, 0, 0, 1);

        // Validators without enough stake will be removed.
        dp::unlock(validator_1, validator_1_address, 50 * ONE_APT);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(stake::get_validator_state(validator_1_address) == VALIDATOR_STATUS_INACTIVE, 11);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_delegated_staking_with_owner_cap(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            100 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            100,
            100
        );
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 0, false, false);

        // Add stake when the validator is not yet activated.
        mint_and_add_stake(validator, 100 * ONE_APT);
        let pool_address = dp::get_owned_pool_address(signer::address_of(validator));
        stake::assert_validator_state(pool_address, 100 * ONE_APT, 0, 0, 0, 0);

        // Join the validator set with enough stake.
        stake::join_validator_set(validator, pool_address);
        end_epoch();
        assert!(stake::get_validator_state(pool_address) == VALIDATOR_STATUS_ACTIVE, 0);

        // Unlock the entire stake.
        dp::unlock(validator, pool_address, 100 * ONE_APT);
        stake::assert_validator_state(pool_address, 0, 0, 0, 100 * ONE_APT, 0);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();

        // Withdraw stake + rewards.
        stake::assert_validator_state(pool_address, 0, 101 * ONE_APT, 0, 0, 0);
        dp::withdraw(validator, pool_address, 101 * ONE_APT);
        assert!(coin::balance<AptosCoin>(signer::address_of(validator)) == 101 * ONE_APT, 1);
        stake::assert_validator_state(pool_address, 0, 0, 0, 0, 0);

        // Operator can separately rotate consensus key.
        let (_sk_new, pk_new, pop_new) = generate_identity();
        let pk_new_bytes = bls12381::public_key_to_bytes(&pk_new);
        let pop_new_bytes = bls12381::proof_of_possession_to_bytes(&pop_new);
        stake::rotate_consensus_key(validator, pool_address, pk_new_bytes, pop_new_bytes);
        let (consensus_pubkey, _, _) = stake::get_validator_config(pool_address);
        assert!(consensus_pubkey == pk_new_bytes, 2);

        // Operator can update network and fullnode addresses.
        stake::update_network_and_fullnode_addresses(validator, pool_address, b"1", b"2");
        let (_, network_addresses, fullnode_addresses) = stake::get_validator_config(pool_address);
        assert!(network_addresses == b"1", 3);
        assert!(fullnode_addresses == b"2", 4);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000A, location = aptos_framework::stake)]
    public entry fun test_validator_cannot_join_post_genesis(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            100 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            false,
            1,
            100,
            100
        );

        // Joining the validator set should fail as post genesis validator set change is not allowed.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000E, location = aptos_framework::stake)]
    public entry fun test_invalid_pool_address(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, true, true);
        stake::join_validator_set(validator, @0x234);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000A, location = aptos_framework::stake)]
    public entry fun test_validator_cannot_leave_post_genesis(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            100 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            false,
            1,
            100,
            100
        );
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100 * ONE_APT, false, false);

        // Bypass the check to join. This is the same function called during Genesis.
        let validator_address = dp::get_owned_pool_address(signer::address_of(validator));
        stake::join_validator_set(validator, validator_address);
        end_epoch();

        // Leaving the validator set should fail as post genesis validator set change is not allowed.
        stake::leave_validator_set(validator, validator_address);
    }

    #[test(
        aptos_framework = @aptos_framework,
        validator_1 = @aptos_framework,
        validator_2 = @0x2,
        validator_3 = @0x3,
        validator_4 = @0x4,
        validator_5 = @0x5
    )]
    public entry fun test_staking_validator_index(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
        validator_3: &signer,
        validator_4: &signer,
        validator_5: &signer,
    ) {
        initialize_for_test(aptos_framework);

        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        let (_sk_3, pk_3, pop_3) = generate_identity();
        let (_sk_4, pk_4, pop_4) = generate_identity();
        let (_sk_5, pk_5, pop_5) = generate_identity();

        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_3, &pop_3, validator_3, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_4, &pop_4, validator_4, 100 * ONE_APT, false, false);
        initialize_test_validator(&pk_5, &pop_5, validator_5, 100 * ONE_APT, false, false);

        let v1_addr = dp::get_owned_pool_address(signer::address_of(validator_1));
        let v2_addr = dp::get_owned_pool_address(signer::address_of(validator_2));
        let v3_addr = dp::get_owned_pool_address(signer::address_of(validator_3));
        let v4_addr = dp::get_owned_pool_address(signer::address_of(validator_4));
        let v5_addr = dp::get_owned_pool_address(signer::address_of(validator_5));

        stake::join_validator_set(validator_3, v3_addr);
        end_epoch();
        assert!(stake::get_validator_index(v3_addr) == 0, 0);

        stake::join_validator_set(validator_4, v4_addr);
        end_epoch();
        assert!(stake::get_validator_index(v3_addr) == 0, 1);
        assert!(stake::get_validator_index(v4_addr) == 1, 2);

        stake::join_validator_set(validator_1, v1_addr);
        stake::join_validator_set(validator_2, v2_addr);
        // pending_inactive is appended in reverse order
        end_epoch();
        assert!(stake::get_validator_index(v3_addr) == 0, 6);
        assert!(stake::get_validator_index(v4_addr) == 1, 7);
        assert!(stake::get_validator_index(v2_addr) == 2, 8);
        assert!(stake::get_validator_index(v1_addr) == 3, 9);

        stake::join_validator_set(validator_5, v5_addr);
        end_epoch();
        assert!(stake::get_validator_index(v3_addr) == 0, 10);
        assert!(stake::get_validator_index(v4_addr) == 1, 11);
        assert!(stake::get_validator_index(v2_addr) == 2, 12);
        assert!(stake::get_validator_index(v1_addr) == 3, 13);
        assert!(stake::get_validator_index(v5_addr) == 4, 14);

        // after swap remove, it's 3,4,2,5
        stake::leave_validator_set(validator_1, v1_addr);
        // after swap remove, it's 5,4,2
        stake::leave_validator_set(validator_3, v3_addr);
        end_epoch();

        assert!(stake::get_validator_index(v5_addr) == 0, 15);
        assert!(stake::get_validator_index(v4_addr) == 1, 16);
        assert!(stake::get_validator_index(v2_addr) == 2, 17);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000B, location = aptos_framework::stake)]
    public entry fun test_invalid_config(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            100,
            100
        );

        // Call initialize_stake_owner, which only initializes the stake pool but not validator config.
        let validator_address = signer::address_of(validator);
        account::create_account_for_test(validator_address);
        dp::initialize_delegation_pool(validator, 0, vector::empty<u8>());
        validator_address = dp::get_owned_pool_address(validator_address);
        mint_and_add_stake(validator, 100 * ONE_APT);

        // Join the validator set with enough stake. This should fail because the validator didn't initialize validator
        // config.
        stake::join_validator_set(validator, validator_address);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_valid_config(
        aptos_framework: &signer,
        validator: &signer,
    ) {
        initialize_for_test_custom(
            aptos_framework,
            50 * ONE_APT,
            10000 * ONE_APT,
            LOCKUP_CYCLE_SECONDS,
            true,
            1,
            100,
            100
        );

        // Call initialize_stake_owner, which only initializes the stake pool but not validator config.
        let validator_address = signer::address_of(validator);
        account::create_account_for_test(validator_address);
        dp::initialize_delegation_pool(validator, 0, vector::empty<u8>());
        validator_address = dp::get_owned_pool_address(validator_address);
        mint_and_add_stake(validator, 100 * ONE_APT);

        // Initialize validator config.
        let (_sk_new, pk_new, pop_new) = generate_identity();
        let pk_new_bytes = bls12381::public_key_to_bytes(&pk_new);
        let pop_new_bytes = bls12381::proof_of_possession_to_bytes(&pop_new);
        stake::rotate_consensus_key(validator, validator_address, pk_new_bytes, pop_new_bytes);

        // Join the validator set with enough stake. This now wouldn't fail since the validator config already exists.
        stake::join_validator_set(validator, validator_address);
    }

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_removing_validator_from_active_set(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100 * ONE_APT, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100 * ONE_APT, true, true);

        // Remove validator 1 from the active validator set. Only validator 2 remains.
        let validator_to_remove = dp::get_owned_pool_address(signer::address_of(validator_1));
        stake::remove_validators(aptos_framework, &vector[validator_to_remove]);
        assert!(stake::get_validator_state(validator_to_remove) == VALIDATOR_STATUS_PENDING_INACTIVE, 1);
    }
}
