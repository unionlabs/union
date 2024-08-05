/// Provides the configuration for staking and rewards
module aptos_framework::staking_config {
    use std::error;
    use std::features;

    use aptos_framework::system_addresses;
    use aptos_framework::timestamp;

    use aptos_std::fixed_point64::{Self, FixedPoint64, less_or_equal};
    use aptos_std::math_fixed64;

    friend aptos_framework::genesis;
    friend aptos_framework::stake;

    /// Stake lockup duration cannot be zero.
    const EZERO_LOCKUP_DURATION: u64 = 1;
    /// Reward rate denominator cannot be zero.
    const EZERO_REWARDS_RATE_DENOMINATOR: u64 = 2;
    /// Specified stake range is invalid. Max must be greater than min.
    const EINVALID_STAKE_RANGE: u64 = 3;
    /// The voting power increase limit percentage must be within (0, 50].
    const EINVALID_VOTING_POWER_INCREASE_LIMIT: u64 = 4;
    /// Specified rewards rate is invalid, which must be within [0, MAX_REWARDS_RATE].
    const EINVALID_REWARDS_RATE: u64 = 5;
    /// Specified min rewards rate is invalid, which must be within [0, rewards_rate].
    const EINVALID_MIN_REWARDS_RATE: u64 = 6;
    /// Specified start time of last rewards rate period is invalid, which must be not late than the current timestamp.
    const EINVALID_LAST_REWARDS_RATE_PERIOD_START: u64 = 7;
    /// Specified rewards rate decrease rate is invalid, which must be not greater than BPS_DENOMINATOR.
    const EINVALID_REWARDS_RATE_DECREASE_RATE: u64 = 8;
    /// Specified rewards rate period is invalid. It must be larger than 0 and cannot be changed if configured.
    const EINVALID_REWARDS_RATE_PERIOD: u64 = 9;
    /// The function has been deprecated.
    const EDEPRECATED_FUNCTION: u64 = 10;
    /// The function is disabled or hasn't been enabled.
    const EDISABLED_FUNCTION: u64 = 11;

    /// Limit the maximum value of `rewards_rate` in order to avoid any arithmetic overflow.
    const MAX_REWARDS_RATE: u64 = 1000000;
    /// Denominator of number in basis points. 1 bps(basis points) = 0.01%.
    const BPS_DENOMINATOR: u64 = 10000;
    /// 1 year => 365 * 24 * 60 * 60
    const ONE_YEAR_IN_SECS: u64 = 31536000;

    const MAX_U64: u128 = 18446744073709551615;


    /// Validator set configurations that will be stored with the @aptos_framework account.
    struct StakingConfig has copy, drop, key {
        // A validator needs to stake at least this amount to be able to join the validator set.
        // If after joining the validator set and at the start of any epoch, a validator's stake drops below this amount
        // they will be removed from the set.
        minimum_stake: u64,
        // A validator can only stake at most this amount. Any larger stake will be rejected.
        // If after joining the validator set and at the start of any epoch, a validator's stake exceeds this amount,
        // their voting power and rewards would only be issued for the max stake amount.
        maximum_stake: u64,
        recurring_lockup_duration_secs: u64,
        // Whether validators are allow to join/leave post genesis.
        allow_validator_set_change: bool,
        // DEPRECATING: staking reward configurations will be in StakingRewardsConfig once REWARD_RATE_DECREASE flag is enabled.
        // The maximum rewards given out every epoch. This will be divided by the rewards rate denominator.
        // For example, 0.001% (0.00001) can be represented as 10 / 1000000.
        rewards_rate: u64,
        // DEPRECATING: staking reward configurations will be in StakingRewardsConfig once REWARD_RATE_DECREASE flag is enabled.
        rewards_rate_denominator: u64,
        // Only this % of current total voting power is allowed to join the validator set in each epoch.
        // This is necessary to prevent a massive amount of new stake from joining that can potentially take down the
        // network if corresponding validators are not ready to participate in consensus in time.
        // This value is within (0, 50%), not inclusive.
        voting_power_increase_limit: u64,
    }

    /// Staking reward configurations that will be stored with the @aptos_framework account.
    struct StakingRewardsConfig has copy, drop, key {
        // The target rewards rate given out every epoch. This will be divided by the rewards rate denominator.
        // For example, 0.001% (0.00001) can be represented as 10 / 1000000.
        rewards_rate: FixedPoint64,
        // The minimum threshold for rewards_rate. rewards_rate won't be lower than this.
        // This will be divided by the rewards rate denominator.
        min_rewards_rate: FixedPoint64,
        // Reward rate decreases every rewards_rate_period_in_secs seconds.
        // Currently it can only equal to 1 year. Keep this field as a plceholder so we can change the reward period
        // without adding new struct.
        rewards_rate_period_in_secs: u64,
        // Timestamp of start of last rewards period.
        last_rewards_rate_period_start_in_secs: u64,
        // Rate of reward rate decrease in BPS. 1 bps(basis points) = 0.01%.
        rewards_rate_decrease_rate: FixedPoint64,
    }

    /// Only called during genesis.
    public(friend) fun initialize(
        aptos_framework: &signer,
        minimum_stake: u64,
        maximum_stake: u64,
        recurring_lockup_duration_secs: u64,
        allow_validator_set_change: bool,
        rewards_rate: u64,
        rewards_rate_denominator: u64,
        voting_power_increase_limit: u64,
    ) {
        system_addresses::assert_aptos_framework(aptos_framework);

        // This can fail genesis but is necessary so that any misconfigurations can be corrected before genesis succeeds
        validate_required_stake(minimum_stake, maximum_stake);

        assert!(recurring_lockup_duration_secs > 0, error::invalid_argument(EZERO_LOCKUP_DURATION));
        assert!(
            rewards_rate_denominator > 0,
            error::invalid_argument(EZERO_REWARDS_RATE_DENOMINATOR),
        );
        assert!(
            voting_power_increase_limit > 0 && voting_power_increase_limit <= 50,
            error::invalid_argument(EINVALID_VOTING_POWER_INCREASE_LIMIT),
        );

        // `rewards_rate` which is the numerator is limited to be `<= MAX_REWARDS_RATE` in order to avoid the arithmetic
        // overflow in the rewards calculation. `rewards_rate_denominator` can be adjusted to get the desired rewards
        // rate (i.e., rewards_rate / rewards_rate_denominator).
        assert!(rewards_rate <= MAX_REWARDS_RATE, error::invalid_argument(EINVALID_REWARDS_RATE));

        // We assert that (rewards_rate / rewards_rate_denominator <= 1).
        assert!(rewards_rate <= rewards_rate_denominator, error::invalid_argument(EINVALID_REWARDS_RATE));

        move_to(aptos_framework, StakingConfig {
            minimum_stake,
            maximum_stake,
            recurring_lockup_duration_secs,
            allow_validator_set_change,
            rewards_rate,
            rewards_rate_denominator,
            voting_power_increase_limit,
        });
    }

    #[view]
    /// Return the reward rate of this epoch as a tuple (numerator, denominator).
    public fun reward_rate(): (u64, u64) acquires StakingRewardsConfig, StakingConfig {
        get_reward_rate(borrow_global<StakingConfig>(@aptos_framework))
    }

    /// Initialize rewards configurations.
    /// Can only be called as part of the Aptos governance proposal process established by the AptosGovernance module.
    public fun initialize_rewards(
        aptos_framework: &signer,
        rewards_rate: FixedPoint64,
        min_rewards_rate: FixedPoint64,
        rewards_rate_period_in_secs: u64,
        last_rewards_rate_period_start_in_secs: u64,
        rewards_rate_decrease_rate: FixedPoint64,
    ) {
        system_addresses::assert_aptos_framework(aptos_framework);

        validate_rewards_config(
            rewards_rate,
            min_rewards_rate,
            rewards_rate_period_in_secs,
            rewards_rate_decrease_rate,
        );
        assert!(
            timestamp::now_seconds() >= last_rewards_rate_period_start_in_secs,
            error::invalid_argument(EINVALID_LAST_REWARDS_RATE_PERIOD_START)
        );

        move_to(aptos_framework, StakingRewardsConfig {
            rewards_rate,
            min_rewards_rate,
            rewards_rate_period_in_secs,
            last_rewards_rate_period_start_in_secs,
            rewards_rate_decrease_rate,
        });
    }

    public fun get(): StakingConfig acquires StakingConfig {
        *borrow_global<StakingConfig>(@aptos_framework)
    }

    /// Return whether validator set changes are allowed
    public fun get_allow_validator_set_change(config: &StakingConfig): bool {
        config.allow_validator_set_change
    }

    /// Return the required min/max stake.
    public fun get_required_stake(config: &StakingConfig): (u64, u64) {
        (config.minimum_stake, config.maximum_stake)
    }

    /// Return the recurring lockup duration that every validator is automatically renewed for (unless they unlock and
    /// withdraw all funds).
    public fun get_recurring_lockup_duration(config: &StakingConfig): u64 {
        config.recurring_lockup_duration_secs
    }

    /// Return the reward rate of this epoch.
    public fun get_reward_rate(config: &StakingConfig): (u64, u64) acquires StakingRewardsConfig {
        if (features::periodical_reward_rate_decrease_enabled()) {
            let epoch_rewards_rate = borrow_global<StakingRewardsConfig>(@aptos_framework).rewards_rate;
            if (fixed_point64::is_zero(epoch_rewards_rate)) {
                (0u64, 1u64)
            } else {
                // Maximize denominator for higher precision.
                // Restriction: nominator <= MAX_REWARDS_RATE && denominator <= MAX_U64
                let denominator = fixed_point64::divide_u128((MAX_REWARDS_RATE as u128), epoch_rewards_rate);
                if (denominator > MAX_U64) {
                    denominator = MAX_U64
                };
                let nominator = (fixed_point64::multiply_u128(denominator, epoch_rewards_rate) as u64);
                (nominator, (denominator as u64))
            }
        } else {
            (config.rewards_rate, config.rewards_rate_denominator)
        }
    }

    /// Return the joining limit %.
    public fun get_voting_power_increase_limit(config: &StakingConfig): u64 {
        config.voting_power_increase_limit
    }

    /// Calculate and save the latest rewards rate.
    public(friend) fun calculate_and_save_latest_epoch_rewards_rate(): FixedPoint64 acquires StakingRewardsConfig {
        assert!(features::periodical_reward_rate_decrease_enabled(), error::invalid_state(EDISABLED_FUNCTION));
        let staking_rewards_config = calculate_and_save_latest_rewards_config();
        staking_rewards_config.rewards_rate
    }

    /// Calculate and return the up-to-date StakingRewardsConfig.
    fun calculate_and_save_latest_rewards_config(): StakingRewardsConfig acquires StakingRewardsConfig {
        let staking_rewards_config = borrow_global_mut<StakingRewardsConfig>(@aptos_framework);
        let current_time_in_secs = timestamp::now_seconds();
        assert!(
            current_time_in_secs >= staking_rewards_config.last_rewards_rate_period_start_in_secs,
            error::invalid_argument(EINVALID_LAST_REWARDS_RATE_PERIOD_START)
        );
        if (current_time_in_secs - staking_rewards_config.last_rewards_rate_period_start_in_secs < staking_rewards_config.rewards_rate_period_in_secs) {
            return *staking_rewards_config
        };
        // Rewards rate decrease rate cannot be greater than 100%. Otherwise rewards rate will be negative.
        assert!(
            fixed_point64::ceil(staking_rewards_config.rewards_rate_decrease_rate) <= 1,
            error::invalid_argument(EINVALID_REWARDS_RATE_DECREASE_RATE)
        );
        let new_rate = math_fixed64::mul_div(
            staking_rewards_config.rewards_rate,
            fixed_point64::sub(
                fixed_point64::create_from_u128(1),
                staking_rewards_config.rewards_rate_decrease_rate,
            ),
            fixed_point64::create_from_u128(1),
        );
        new_rate = fixed_point64::max(new_rate, staking_rewards_config.min_rewards_rate);

        staking_rewards_config.rewards_rate = new_rate;
        staking_rewards_config.last_rewards_rate_period_start_in_secs =
            staking_rewards_config.last_rewards_rate_period_start_in_secs +
            staking_rewards_config.rewards_rate_period_in_secs;
        return *staking_rewards_config
    }

    /// Update the min and max stake amounts.
    /// Can only be called as part of the Aptos governance proposal process established by the AptosGovernance module.
    public fun update_required_stake(
        aptos_framework: &signer,
        minimum_stake: u64,
        maximum_stake: u64,
    ) acquires StakingConfig {
        system_addresses::assert_aptos_framework(aptos_framework);
        validate_required_stake(minimum_stake, maximum_stake);

        let staking_config = borrow_global_mut<StakingConfig>(@aptos_framework);
        staking_config.minimum_stake = minimum_stake;
        staking_config.maximum_stake = maximum_stake;
    }

    /// Update the recurring lockup duration.
    /// Can only be called as part of the Aptos governance proposal process established by the AptosGovernance module.
    public fun update_recurring_lockup_duration_secs(
        aptos_framework: &signer,
        new_recurring_lockup_duration_secs: u64,
    ) acquires StakingConfig {
        assert!(new_recurring_lockup_duration_secs > 0, error::invalid_argument(EZERO_LOCKUP_DURATION));
        system_addresses::assert_aptos_framework(aptos_framework);

        let staking_config = borrow_global_mut<StakingConfig>(@aptos_framework);
        staking_config.recurring_lockup_duration_secs = new_recurring_lockup_duration_secs;
    }

    /// DEPRECATING
    /// Update the rewards rate.
    /// Can only be called as part of the Aptos governance proposal process established by the AptosGovernance module.
    public fun update_rewards_rate(
        aptos_framework: &signer,
        new_rewards_rate: u64,
        new_rewards_rate_denominator: u64,
    ) acquires StakingConfig {
        assert!(!features::periodical_reward_rate_decrease_enabled(), error::invalid_state(EDEPRECATED_FUNCTION));
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(
            new_rewards_rate_denominator > 0,
            error::invalid_argument(EZERO_REWARDS_RATE_DENOMINATOR),
        );
        // `rewards_rate` which is the numerator is limited to be `<= MAX_REWARDS_RATE` in order to avoid the arithmetic
        // overflow in the rewards calculation. `rewards_rate_denominator` can be adjusted to get the desired rewards
        // rate (i.e., rewards_rate / rewards_rate_denominator).
        assert!(new_rewards_rate <= MAX_REWARDS_RATE, error::invalid_argument(EINVALID_REWARDS_RATE));

        // We assert that (rewards_rate / rewards_rate_denominator <= 1).
        assert!(new_rewards_rate <= new_rewards_rate_denominator, error::invalid_argument(EINVALID_REWARDS_RATE));

        let staking_config = borrow_global_mut<StakingConfig>(@aptos_framework);
        staking_config.rewards_rate = new_rewards_rate;
        staking_config.rewards_rate_denominator = new_rewards_rate_denominator;
    }

    public fun update_rewards_config(
        aptos_framework: &signer,
        rewards_rate: FixedPoint64,
        min_rewards_rate: FixedPoint64,
        rewards_rate_period_in_secs: u64,
        rewards_rate_decrease_rate: FixedPoint64,
    ) acquires StakingRewardsConfig {
        system_addresses::assert_aptos_framework(aptos_framework);

        validate_rewards_config(
            rewards_rate,
            min_rewards_rate,
            rewards_rate_period_in_secs,
            rewards_rate_decrease_rate,
        );

        let staking_rewards_config = borrow_global_mut<StakingRewardsConfig>(@aptos_framework);
        // Currently rewards_rate_period_in_secs is not allowed to be changed because this could bring complicated
        // logics. At the moment the argument is just a placeholder for future use.
        assert!(
            rewards_rate_period_in_secs == staking_rewards_config.rewards_rate_period_in_secs,
            error::invalid_argument(EINVALID_REWARDS_RATE_PERIOD),
        );
        staking_rewards_config.rewards_rate = rewards_rate;
        staking_rewards_config.min_rewards_rate = min_rewards_rate;
        staking_rewards_config.rewards_rate_period_in_secs = rewards_rate_period_in_secs;
        staking_rewards_config.rewards_rate_decrease_rate = rewards_rate_decrease_rate;
    }

    /// Update the joining limit %.
    /// Can only be called as part of the Aptos governance proposal process established by the AptosGovernance module.
    public fun update_voting_power_increase_limit(
        aptos_framework: &signer,
        new_voting_power_increase_limit: u64,
    ) acquires StakingConfig {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(
            new_voting_power_increase_limit > 0 && new_voting_power_increase_limit <= 50,
            error::invalid_argument(EINVALID_VOTING_POWER_INCREASE_LIMIT),
        );

        let staking_config = borrow_global_mut<StakingConfig>(@aptos_framework);
        staking_config.voting_power_increase_limit = new_voting_power_increase_limit;
    }

    fun validate_required_stake(minimum_stake: u64, maximum_stake: u64) {
        assert!(minimum_stake <= maximum_stake && maximum_stake > 0, error::invalid_argument(EINVALID_STAKE_RANGE));
    }

    fun validate_rewards_config(
        rewards_rate: FixedPoint64,
        min_rewards_rate: FixedPoint64,
        rewards_rate_period_in_secs: u64,
        rewards_rate_decrease_rate: FixedPoint64,
    ) {
        // Bound rewards rate to avoid arithmetic overflow.
        assert!(
            less_or_equal(rewards_rate, fixed_point64::create_from_u128((1u128))),
            error::invalid_argument(EINVALID_REWARDS_RATE)
        );
        assert!(
            less_or_equal(min_rewards_rate, rewards_rate),
            error::invalid_argument(EINVALID_MIN_REWARDS_RATE)
        );
        // Rewards rate decrease rate cannot be greater than 100%. Otherwise rewards rate will be negative.
        assert!(
            fixed_point64::ceil(rewards_rate_decrease_rate) <= 1,
            error::invalid_argument(EINVALID_REWARDS_RATE_DECREASE_RATE)
        );
        // This field, rewards_rate_period_in_secs must be greater than 0.
        // TODO: rewards_rate_period_in_secs should be longer than the epoch duration but reading epoch duration causes a circular dependency.
        assert!(
            rewards_rate_period_in_secs > 0,
            error::invalid_argument(EINVALID_REWARDS_RATE_PERIOD),
        );
    }

    #[test_only]
    use aptos_std::fixed_point64::{equal, create_from_rational};

    #[test(aptos_framework = @aptos_framework)]
    public entry fun test_change_staking_configs(aptos_framework: signer) acquires StakingConfig {
        initialize(&aptos_framework, 0, 1, 1, false, 1, 1, 1);

        update_required_stake(&aptos_framework, 100, 1000);
        update_recurring_lockup_duration_secs(&aptos_framework, 10000);
        update_rewards_rate(&aptos_framework, 10, 100);
        update_voting_power_increase_limit(&aptos_framework, 10);

        let config = borrow_global<StakingConfig>(@aptos_framework);
        assert!(config.minimum_stake == 100, 0);
        assert!(config.maximum_stake == 1000, 1);
        assert!(config.recurring_lockup_duration_secs == 10000, 3);
        assert!(config.rewards_rate == 10, 4);
        assert!(config.rewards_rate_denominator == 100, 4);
        assert!(config.voting_power_increase_limit == 10, 5);
    }

    #[test(aptos_framework = @aptos_framework)]
    public entry fun test_staking_rewards_rate_decrease_over_time(aptos_framework: signer) acquires StakingRewardsConfig {
        let start_time_in_secs: u64 = 100001000000;
        initialize_rewards_for_test(
            &aptos_framework,
            create_from_rational(1, 100),
            create_from_rational(3, 1000),
            ONE_YEAR_IN_SECS,
            start_time_in_secs,
            create_from_rational(50, 100)
        );

        let epoch_reward_rate = calculate_and_save_latest_epoch_rewards_rate();
        assert!(equal(epoch_reward_rate, create_from_rational(1, 100)), 0);
        // Rewards rate should not change until the current reward rate period ends.
        timestamp::fast_forward_seconds(ONE_YEAR_IN_SECS / 2);
        epoch_reward_rate = calculate_and_save_latest_epoch_rewards_rate();
        assert!(equal(epoch_reward_rate, create_from_rational(1, 100)), 1);

        // Rewards rate decreases to 1 / 100 * 5000 / 10000 = 5 / 1000.
        timestamp::fast_forward_seconds(ONE_YEAR_IN_SECS / 2);
        epoch_reward_rate = calculate_and_save_latest_epoch_rewards_rate();
        assert!(equal(epoch_reward_rate, create_from_rational(5, 1000)), 2);

        // Rewards rate decreases to 5 / 1000 * 5000 / 10000 = 2.5 / 1000.
        // But rewards_rate cannot be lower than min_rewards_rate = 3 / 1000.
        timestamp::fast_forward_seconds(ONE_YEAR_IN_SECS);
        epoch_reward_rate = calculate_and_save_latest_epoch_rewards_rate();
        assert!(equal(epoch_reward_rate, create_from_rational(3, 1000)), 3);

        // Test when rewards_rate_decrease_rate is very small
        update_rewards_config(
            &aptos_framework,
            epoch_reward_rate,
            create_from_rational(0, 1000),
            ONE_YEAR_IN_SECS,
            create_from_rational(15, 1000),
        );
        // Rewards rate decreases to 3 / 1000 * 985 / 1000 = 2955 / 1000000.
        timestamp::fast_forward_seconds(ONE_YEAR_IN_SECS);
        epoch_reward_rate = calculate_and_save_latest_epoch_rewards_rate();
        assert!(
            fixed_point64::almost_equal(
                epoch_reward_rate,
                create_from_rational(2955, 1000000),
                create_from_rational(1, 100000000)
            ),
            4);
    }

    #[test(aptos_framework = @aptos_framework)]
    public entry fun test_change_staking_rewards_configs(aptos_framework: signer) acquires StakingRewardsConfig {
        let start_time_in_secs: u64 = 100001000000;
        initialize_rewards_for_test(
            &aptos_framework,
            create_from_rational(1, 100),
            create_from_rational(3, 1000),
            ONE_YEAR_IN_SECS,
            start_time_in_secs,
            create_from_rational(50, 100),
        );

        update_rewards_config(
            &aptos_framework,
            create_from_rational(2, 100),
            create_from_rational(6, 1000),
            ONE_YEAR_IN_SECS,
            create_from_rational(25, 100),
        );

        let config = borrow_global<StakingRewardsConfig>(@aptos_framework);
        assert!(equal(config.rewards_rate, create_from_rational(2, 100)), 0);
        assert!(equal(config.min_rewards_rate, create_from_rational(6, 1000)), 1);
        assert!(config.rewards_rate_period_in_secs == ONE_YEAR_IN_SECS, 4);
        assert!(config.last_rewards_rate_period_start_in_secs == start_time_in_secs, 4);
        assert!(equal(config.rewards_rate_decrease_rate, create_from_rational(25, 100)), 5);
    }

    #[test(account = @0x123)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::system_addresses)]
    public entry fun test_update_required_stake_unauthorized_should_fail(account: signer) acquires StakingConfig {
        update_required_stake(&account, 1, 2);
    }

    #[test(account = @0x123)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::system_addresses)]
    public entry fun test_update_required_lockup_unauthorized_should_fail(account: signer) acquires StakingConfig {
        update_recurring_lockup_duration_secs(&account, 1);
    }

    #[test(account = @0x123)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::system_addresses)]
    public entry fun test_update_rewards_unauthorized_should_fail(account: signer) acquires StakingConfig {
        update_rewards_rate(&account, 1, 10);
    }

    #[test(account = @0x123)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::system_addresses)]
    public entry fun test_update_voting_power_increase_limit_unauthorized_should_fail(account: signer) acquires StakingConfig {
        update_voting_power_increase_limit(&account, 10);
    }

    #[test(account = @0x123, aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x50003, location = aptos_framework::system_addresses)]
    public entry fun test_update_rewards_config_unauthorized_should_fail(account: signer, aptos_framework: signer) acquires StakingRewardsConfig {
        features::change_feature_flags_for_testing(&aptos_framework, vector[features::get_periodical_reward_rate_decrease_feature()], vector[]);
        update_rewards_config(
            &account,
            create_from_rational(1, 100),
            create_from_rational(1, 100),
            ONE_YEAR_IN_SECS,
            create_from_rational(1, 100),
        );
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10003, location = Self)]
    public entry fun test_update_required_stake_invalid_range_should_fail(aptos_framework: signer) acquires StakingConfig {
        update_required_stake(&aptos_framework, 10, 5);
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10003, location = Self)]
    public entry fun test_update_required_stake_zero_max_stake_should_fail(aptos_framework: signer) acquires StakingConfig {
        update_required_stake(&aptos_framework, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10001, location = Self)]
    public entry fun test_update_required_lockup_to_zero_should_fail(aptos_framework: signer) acquires StakingConfig {
        update_recurring_lockup_duration_secs(&aptos_framework, 0);
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10002, location = Self)]
    public entry fun test_update_rewards_invalid_denominator_should_fail(aptos_framework: signer) acquires StakingConfig {
        update_rewards_rate(&aptos_framework, 1, 0);
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10005, location = Self)]
    public entry fun test_update_rewards_config_rewards_rate_greater_than_1_should_fail(aptos_framework: signer) acquires StakingRewardsConfig {
        let start_time_in_secs: u64 = 100001000000;
        initialize_rewards_for_test(
            &aptos_framework,
            create_from_rational(15981, 1000000000),
            create_from_rational(7991, 1000000000),
            ONE_YEAR_IN_SECS,
            start_time_in_secs,
            create_from_rational(15, 1000),
        );
        update_rewards_config(
            &aptos_framework,
            create_from_rational(101, 100),
            create_from_rational(1, 100),
            ONE_YEAR_IN_SECS,
            create_from_rational(1, 100),
        );
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10008, location = Self)]
    public entry fun test_update_rewards_config_invalid_rewards_rate_decrease_rate_should_fail(aptos_framework: signer) acquires StakingRewardsConfig {
        let start_time_in_secs: u64 = 100001000000;
        initialize_rewards_for_test(
            &aptos_framework,
            create_from_rational(15981, 1000000000),
            create_from_rational(7991, 1000000000),
            ONE_YEAR_IN_SECS,
            start_time_in_secs,
            create_from_rational(15, 1000),
        );
        update_rewards_config(
            &aptos_framework,
            create_from_rational(1, 100),
            create_from_rational(1, 100),
            ONE_YEAR_IN_SECS,
            create_from_rational(101, 100),
        );
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10009, location = Self)]
    public entry fun test_update_rewards_config_cannot_change_rewards_rate_period(aptos_framework: signer) acquires StakingRewardsConfig {
        let start_time_in_secs: u64 = 100001000000;
        initialize_rewards_for_test(
            &aptos_framework,
            create_from_rational(15981, 1000000000),
            create_from_rational(7991, 1000000000),
            ONE_YEAR_IN_SECS,
            start_time_in_secs,
            create_from_rational(15, 1000),
        );
        update_rewards_config(
            &aptos_framework,
            create_from_rational(15981, 1000000000),
            create_from_rational(7991, 1000000000),
            ONE_YEAR_IN_SECS - 1,
            create_from_rational(15, 1000),
        );
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x3000B, location = Self)]
    public entry fun test_feature_flag_disabled_get_epoch_rewards_rate_should_fail(aptos_framework: signer) acquires StakingRewardsConfig {
        features::change_feature_flags_for_testing(&aptos_framework, vector[], vector[features::get_periodical_reward_rate_decrease_feature()]);
        calculate_and_save_latest_epoch_rewards_rate();
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10004, location = Self)]
    public entry fun test_update_voting_power_increase_limit_to_zero_should_fail(
        aptos_framework: signer
    ) acquires StakingConfig {
        update_voting_power_increase_limit(&aptos_framework, 0);
    }

    #[test(aptos_framework = @aptos_framework)]
    #[expected_failure(abort_code = 0x10004, location = aptos_framework::staking_config)]
    public entry fun test_update_voting_power_increase_limit_to_more_than_upper_bound_should_fail(
        aptos_framework: signer
    ) acquires StakingConfig {
        update_voting_power_increase_limit(&aptos_framework, 51);
    }

    // For tests to bypass all validations.
    #[test_only]
    public fun initialize_for_test(
        aptos_framework: &signer,
        minimum_stake: u64,
        maximum_stake: u64,
        recurring_lockup_duration_secs: u64,
        allow_validator_set_change: bool,
        rewards_rate: u64,
        rewards_rate_denominator: u64,
        voting_power_increase_limit: u64,
    ) {
        if (!exists<StakingConfig>(@aptos_framework)) {
            move_to(aptos_framework, StakingConfig {
                minimum_stake,
                maximum_stake,
                recurring_lockup_duration_secs,
                allow_validator_set_change,
                rewards_rate,
                rewards_rate_denominator,
                voting_power_increase_limit,
            });
        };
    }

    // For tests to bypass all validations.
    #[test_only]
    public fun initialize_rewards_for_test(
        aptos_framework: &signer,
        rewards_rate: FixedPoint64,
        min_rewards_rate: FixedPoint64,
        rewards_rate_period_in_micros: u64,
        last_rewards_rate_period_start_in_secs: u64,
        rewards_rate_decrease_rate: FixedPoint64,
    ) {
        features::change_feature_flags_for_testing(aptos_framework, vector[features::get_periodical_reward_rate_decrease_feature()], vector[]);
        timestamp::set_time_has_started_for_testing(aptos_framework);
        timestamp::update_global_time_for_test_secs(last_rewards_rate_period_start_in_secs);
        initialize_rewards(
            aptos_framework,
            rewards_rate,
            min_rewards_rate,
            rewards_rate_period_in_micros,
            last_rewards_rate_period_start_in_secs,
            rewards_rate_decrease_rate,
        );
    }
}
