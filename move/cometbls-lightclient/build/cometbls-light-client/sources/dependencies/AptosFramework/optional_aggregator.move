/// This module provides an interface to aggregate integers either via
/// aggregator (parallelizable) or via normal integers.
module aptos_framework::optional_aggregator {
    use std::error;
    use std::option::{Self, Option};

    use aptos_framework::aggregator_factory;
    use aptos_framework::aggregator::{Self, Aggregator};

    friend aptos_framework::coin;
    friend aptos_framework::fungible_asset;

    /// The value of aggregator underflows (goes below zero). Raised by native code.
    const EAGGREGATOR_OVERFLOW: u64 = 1;

    /// Aggregator feature is not supported. Raised by native code.
    const EAGGREGATOR_UNDERFLOW: u64 = 2;

    /// Wrapper around integer with a custom overflow limit. Supports add, subtract and read just like `Aggregator`.
    struct Integer has store {
        value: u128,
        limit: u128,
    }

    /// Creates a new integer which overflows on exceeding a `limit`.
    fun new_integer(limit: u128): Integer {
        Integer {
            value: 0,
            limit,
        }
    }

    /// Adds `value` to integer. Aborts on overflowing the limit.
    fun add_integer(integer: &mut Integer, value: u128) {
        assert!(
            value <= (integer.limit - integer.value),
            error::out_of_range(EAGGREGATOR_OVERFLOW)
        );
        integer.value = integer.value + value;
    }

    /// Subtracts `value` from integer. Aborts on going below zero.
    fun sub_integer(integer: &mut Integer, value: u128) {
        assert!(value <= integer.value, error::out_of_range(EAGGREGATOR_UNDERFLOW));
        integer.value = integer.value - value;
    }

    /// Returns an overflow limit of integer.
    fun limit(integer: &Integer): u128 {
        integer.limit
    }

    /// Returns a value stored in this integer.
    fun read_integer(integer: &Integer): u128 {
        integer.value
    }

    /// Destroys an integer.
    fun destroy_integer(integer: Integer) {
        let Integer { value: _, limit: _ } = integer;
    }

    /// Contains either an aggregator or a normal integer, both overflowing on limit.
    struct OptionalAggregator has store {
        // Parallelizable.
        aggregator: Option<Aggregator>,
        // Non-parallelizable.
        integer: Option<Integer>,
    }

    /// Creates a new optional aggregator.
    public(friend) fun new(limit: u128, parallelizable: bool): OptionalAggregator {
        if (parallelizable) {
            OptionalAggregator {
                aggregator: option::some(aggregator_factory::create_aggregator_internal(limit)),
                integer: option::none(),
            }
        } else {
            OptionalAggregator {
                aggregator: option::none(),
                integer: option::some(new_integer(limit)),
            }
        }
    }

    /// Switches between parallelizable and non-parallelizable implementations.
    public fun switch(optional_aggregator: &mut OptionalAggregator) {
        let value = read(optional_aggregator);
        switch_and_zero_out(optional_aggregator);
        add(optional_aggregator, value);
    }

    /// Switches between parallelizable and non-parallelizable implementations, setting
    /// the value of the new optional aggregator to zero.
    fun switch_and_zero_out(optional_aggregator: &mut OptionalAggregator) {
        if (is_parallelizable(optional_aggregator)) {
            switch_to_integer_and_zero_out(optional_aggregator);
        } else {
            switch_to_aggregator_and_zero_out(optional_aggregator);
        }
    }

    /// Switches from parallelizable to non-parallelizable implementation, zero-initializing
    /// the value.
    fun switch_to_integer_and_zero_out(
        optional_aggregator: &mut OptionalAggregator
    ): u128 {
        let aggregator = option::extract(&mut optional_aggregator.aggregator);
        let limit = aggregator::limit(&aggregator);
        aggregator::destroy(aggregator);
        let integer = new_integer(limit);
        option::fill(&mut optional_aggregator.integer, integer);
        limit
    }

    /// Switches from non-parallelizable to parallelizable implementation, zero-initializing
    /// the value.
    fun switch_to_aggregator_and_zero_out(
        optional_aggregator: &mut OptionalAggregator
    ): u128 {
        let integer = option::extract(&mut optional_aggregator.integer);
        let limit = limit(&integer);
        destroy_integer(integer);
        let aggregator = aggregator_factory::create_aggregator_internal(limit);
        option::fill(&mut optional_aggregator.aggregator, aggregator);
        limit
    }

    /// Destroys optional aggregator.
    public fun destroy(optional_aggregator: OptionalAggregator) {
        if (is_parallelizable(&optional_aggregator)) {
            destroy_optional_aggregator(optional_aggregator);
        } else {
            destroy_optional_integer(optional_aggregator);
        }
    }

    /// Destroys parallelizable optional aggregator and returns its limit.
    fun destroy_optional_aggregator(optional_aggregator: OptionalAggregator): u128 {
        let OptionalAggregator { aggregator, integer } = optional_aggregator;
        let limit = aggregator::limit(option::borrow(&aggregator));
        aggregator::destroy(option::destroy_some(aggregator));
        option::destroy_none(integer);
        limit
    }

    /// Destroys non-parallelizable optional aggregator and returns its limit.
    fun destroy_optional_integer(optional_aggregator: OptionalAggregator): u128 {
        let OptionalAggregator { aggregator, integer } = optional_aggregator;
        let limit = limit(option::borrow(&integer));
        destroy_integer(option::destroy_some(integer));
        option::destroy_none(aggregator);
        limit
    }

    /// Adds `value` to optional aggregator, aborting on exceeding the `limit`.
    public fun add(optional_aggregator: &mut OptionalAggregator, value: u128) {
        if (option::is_some(&optional_aggregator.aggregator)) {
            let aggregator = option::borrow_mut(&mut optional_aggregator.aggregator);
            aggregator::add(aggregator, value);
        } else {
            let integer = option::borrow_mut(&mut optional_aggregator.integer);
            add_integer(integer, value);
        }
    }

    /// Subtracts `value` from optional aggregator, aborting on going below zero.
    public fun sub(optional_aggregator: &mut OptionalAggregator, value: u128) {
        if (option::is_some(&optional_aggregator.aggregator)) {
            let aggregator = option::borrow_mut(&mut optional_aggregator.aggregator);
            aggregator::sub(aggregator, value);
        } else {
            let integer = option::borrow_mut(&mut optional_aggregator.integer);
            sub_integer(integer, value);
        }
    }

    /// Returns the value stored in optional aggregator.
    public fun read(optional_aggregator: &OptionalAggregator): u128 {
        if (option::is_some(&optional_aggregator.aggregator)) {
            let aggregator = option::borrow(&optional_aggregator.aggregator);
            aggregator::read(aggregator)
        } else {
            let integer = option::borrow(&optional_aggregator.integer);
            read_integer(integer)
        }
    }

    /// Returns true if optional aggregator uses parallelizable implementation.
    public fun is_parallelizable(optional_aggregator: &OptionalAggregator): bool {
        option::is_some(&optional_aggregator.aggregator)
    }

    #[test(account = @aptos_framework)]
    fun optional_aggregator_test(account: signer) {
        aggregator_factory::initialize_aggregator_factory(&account);

        let aggregator = new(30, false);
        assert!(!is_parallelizable(&aggregator), 0);

        add(&mut aggregator, 12);
        add(&mut aggregator, 3);
        assert!(read(&aggregator) == 15, 0);

        sub(&mut aggregator, 10);
        assert!(read(&aggregator) == 5, 0);

        // Switch to parallelizable aggregator and check the value is preserved.
        switch(&mut aggregator);
        assert!(is_parallelizable(&aggregator), 0);
        assert!(read(&aggregator) == 5, 0);

        add(&mut aggregator, 12);
        add(&mut aggregator, 3);
        assert!(read(&aggregator) == 20, 0);

        sub(&mut aggregator, 10);
        assert!(read(&aggregator) == 10, 0);

        // Switch back!
        switch(&mut aggregator);
        assert!(!is_parallelizable(&aggregator), 0);
        assert!(read(&aggregator) == 10, 0);

        destroy(aggregator);
    }

    #[test(account = @aptos_framework)]
    fun optional_aggregator_destroy_test(account: signer) {
        aggregator_factory::initialize_aggregator_factory(&account);

        let aggregator = new(30, false);
        destroy(aggregator);

        let aggregator = new(30, true);
        destroy(aggregator);

        let aggregator = new(12, false);
        assert!(destroy_optional_integer(aggregator) == 12, 0);

        let aggregator = new(21, true);
        assert!(destroy_optional_aggregator(aggregator) == 21, 0);
    }

    #[test(account = @aptos_framework)]
    #[expected_failure(abort_code = 0x020001, location = Self)]
    fun non_parallelizable_aggregator_overflow_test(account: signer) {
        aggregator_factory::initialize_aggregator_factory(&account);
        let aggregator = new(15, false);

        // Overflow!
        add(&mut aggregator, 16);

        destroy(aggregator);
    }

    #[test(account = @aptos_framework)]
    #[expected_failure(abort_code = 0x020002, location = Self)]
    fun non_parallelizable_aggregator_underflow_test(account: signer) {
        aggregator_factory::initialize_aggregator_factory(&account);
        let aggregator = new(100, false);

        // Underflow!
        sub(&mut aggregator, 100);
        add(&mut aggregator, 100);

        destroy(aggregator);
    }

    #[test(account = @aptos_framework)]
    #[expected_failure(abort_code = 0x020001, location = aptos_framework::aggregator)]
    fun parallelizable_aggregator_overflow_test(account: signer) {
        aggregator_factory::initialize_aggregator_factory(&account);
        let aggregator = new(15, true);

        // Overflow!
        add(&mut aggregator, 16);

        destroy(aggregator);
    }

    #[test(account = @aptos_framework)]
    #[expected_failure(abort_code = 0x020002, location = aptos_framework::aggregator)]
    fun parallelizable_aggregator_underflow_test(account: signer) {
        aggregator_factory::initialize_aggregator_factory(&account);
        let aggregator = new(100, true);

        // Underflow!
        add(&mut aggregator, 99);
        sub(&mut aggregator, 100);
        add(&mut aggregator, 100);

        destroy(aggregator);
    }
}
