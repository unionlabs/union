/// This module provides an interface for aggregators (version 2). Aggregators are
/// similar to unsigned integers and support addition and subtraction (aborting on
/// underflow or on overflowing a custom upper limit). The difference from integers
/// is that aggregators allow to perform both additions and subtractions in parallel
/// across multiple transactions, enabling parallel execution. For example, if the
/// first transaction is doing `try_add(X, 1)` for aggregator `X`, and the second is
/// doing `try_sub(X,3)`, they can be executed in parallel avoiding a read-modify-write
/// dependency.
/// However, reading the aggregator value (i.e. calling `read(X)`) is a resource-intensive
/// operation that also reduced parallelism, and should be avoided as much as possible.
/// If you need to capture the value, without revealing it, use snapshot function instead,
/// which has no parallelism impact.
///
/// From parallelism considerations, there are three different levels of effects:
/// * enable full parallelism (cannot create conflicts):
///     max_value, create_*, snapshot, derive_string_concat
/// * enable speculative parallelism (generally parallel via branch prediction)
///     try_add, add, try_sub, sub, is_at_least
/// * create read/write conflicts, as if you were using a regular field
///     read, read_snapshot, read_derived_string
module aptos_framework::aggregator_v2 {
    use std::error;
    use std::features;
    use std::string::String;

    /// The value of aggregator overflows. Raised by uncoditional add() call
    const EAGGREGATOR_OVERFLOW: u64 = 1;

    /// The value of aggregator underflows (goes below zero). Raised by uncoditional sub() call
    const EAGGREGATOR_UNDERFLOW: u64 = 2;

    /// The generic type supplied to the aggregator snapshot is not supported.
    const EUNSUPPORTED_AGGREGATOR_SNAPSHOT_TYPE: u64 = 5;

    /// The aggregator api v2 feature flag is not enabled.
    const EAGGREGATOR_API_V2_NOT_ENABLED: u64 = 6;

    /// The generic type supplied to the aggregator is not supported.
    const EUNSUPPORTED_AGGREGATOR_TYPE: u64 = 7;

    /// Arguments passed to concat exceed max limit of 256 bytes (for prefix and suffix together).
    const ECONCAT_STRING_LENGTH_TOO_LARGE: u64 = 8;

    /// The native aggregator function, that is in the move file, is not yet supported.
    /// and any calls will raise this error.
    const EAGGREGATOR_FUNCTION_NOT_YET_SUPPORTED: u64 = 9;

    /// Represents an integer which supports parallel additions and subtractions
    /// across multiple transactions. See the module description for more details.
    ///
    /// Currently supported types for IntElement are u64 and u128.
    struct Aggregator<IntElement> has store, drop {
        value: IntElement,
        max_value: IntElement,
    }

    /// Represents a constant value, that was derived from an aggregator at given instant in time.
    /// Unlike read() and storing the value directly, this enables parallel execution of transactions,
    /// while storing snapshot of aggregator state elsewhere.
    struct AggregatorSnapshot<IntElement> has store, drop {
        value: IntElement,
    }

    struct DerivedStringSnapshot has store, drop {
        value: String,
        padding: vector<u8>,
    }

    /// Returns `max_value` exceeding which aggregator overflows.
    public fun max_value<IntElement: copy + drop>(aggregator: &Aggregator<IntElement>): IntElement {
        aggregator.max_value
    }

    /// Creates new aggregator, with given 'max_value'.
    ///
    /// Currently supported types for IntElement are u64 and u128.
    /// EAGGREGATOR_ELEMENT_TYPE_NOT_SUPPORTED raised if called with a different type.
    public native fun create_aggregator<IntElement: copy + drop>(max_value: IntElement): Aggregator<IntElement>;

    public fun create_aggregator_with_value<IntElement: copy + drop>(start_value: IntElement, max_value: IntElement): Aggregator<IntElement> {
        let aggregator = create_aggregator(max_value);
        add(&mut aggregator, start_value);
        aggregator
    }

    /// Creates new aggregator, without any 'max_value' on top of the implicit bound restriction
    /// due to the width of the type (i.e. MAX_U64 for u64, MAX_U128 for u128).
    ///
    /// Currently supported types for IntElement are u64 and u128.
    /// EAGGREGATOR_ELEMENT_TYPE_NOT_SUPPORTED raised if called with a different type.
    public native fun create_unbounded_aggregator<IntElement: copy + drop>(): Aggregator<IntElement>;

    public fun create_unbounded_aggregator_with_value<IntElement: copy + drop>(start_value: IntElement): Aggregator<IntElement> {
        let aggregator = create_unbounded_aggregator();
        add(&mut aggregator, start_value);
        aggregator
    }

    /// Adds `value` to aggregator.
    /// If addition would exceed the max_value, `false` is returned, and aggregator value is left unchanged.
    ///
    /// Parallelism info: This operation enables speculative parallelism.
    public native fun try_add<IntElement>(aggregator: &mut Aggregator<IntElement>, value: IntElement): bool;

    /// Adds `value` to aggregator, unconditionally.
    /// If addition would exceed the max_value, EAGGREGATOR_OVERFLOW exception will be thrown.
    ///
    /// Parallelism info: This operation enables speculative parallelism.
    public fun add<IntElement>(aggregator: &mut Aggregator<IntElement>, value: IntElement) {
        assert!(try_add(aggregator, value), error::out_of_range(EAGGREGATOR_OVERFLOW));
    }

    /// Subtracts `value` from aggregator.
    /// If subtraction would result in a negative value, `false` is returned, and aggregator value is left unchanged.
    ///
    /// Parallelism info: This operation enables speculative parallelism.
    public native fun try_sub<IntElement>(aggregator: &mut Aggregator<IntElement>, value: IntElement): bool;

    // Subtracts `value` to aggregator, unconditionally.
    // If subtraction would result in a negative value, EAGGREGATOR_UNDERFLOW exception will be thrown.
    ///
    /// Parallelism info: This operation enables speculative parallelism.
    public fun sub<IntElement>(aggregator: &mut Aggregator<IntElement>, value: IntElement) {
        assert!(try_sub(aggregator, value), error::out_of_range(EAGGREGATOR_UNDERFLOW));
    }

    native fun is_at_least_impl<IntElement>(aggregator: &Aggregator<IntElement>, min_amount: IntElement): bool;

    /// Returns true if aggregator value is larger than or equal to the given `min_amount`, false otherwise.
    ///
    /// This operation is more efficient and much more parallelization friendly than calling `read(agg) > min_amount`.
    /// Until traits are deployed, `is_at_most`/`is_equal` utility methods can be derived from this one (assuming +1 doesn't overflow):
    /// - for `is_at_most(agg, max_amount)`, you can do `!is_at_least(max_amount + 1)`
    /// - for `is_equal(agg, value)`, you can do `is_at_least(value) && !is_at_least(value + 1)`
    ///
    /// Parallelism info: This operation enables speculative parallelism.
    public fun is_at_least<IntElement>(aggregator: &Aggregator<IntElement>, min_amount: IntElement): bool {
        assert!(features::aggregator_v2_is_at_least_api_enabled(), EAGGREGATOR_API_V2_NOT_ENABLED);
        is_at_least_impl(aggregator, min_amount)
    }

    // TODO waiting for integer traits
    // public fun is_at_most<IntElement>(aggregator: &Aggregator<IntElement>, max_amount: IntElement): bool {
    //     !is_at_least(max_amount + 1)
    // }

    // TODO waiting for integer traits
    // public fun is_equal<IntElement>(aggregator: &Aggregator<IntElement>, value: IntElement): bool {
    //     is_at_least(value) && !is_at_least(value + 1)
    // }

    /// Returns a value stored in this aggregator.
    /// Note: This operation is resource-intensive, and reduces parallelism.
    /// If you need to capture the value, without revealing it, use snapshot function instead,
    /// which has no parallelism impact.
    /// If called in a transaction that also modifies the aggregator, or has other read/write conflicts,
    /// it will sequentialize that transaction. (i.e. up to concurrency_level times slower)
    /// If called in a separate transaction (i.e. after transaction that modifies aggregator), it might be
    /// up to two times slower.
    ///
    /// Parallelism info: This operation *prevents* speculative parallelism.
    public native fun read<IntElement>(aggregator: &Aggregator<IntElement>): IntElement;

    /// Returns a wrapper of a current value of an aggregator
    /// Unlike read(), it is fast and avoids sequential dependencies.
    ///
    /// Parallelism info: This operation enables parallelism.
    public native fun snapshot<IntElement>(aggregator: &Aggregator<IntElement>): AggregatorSnapshot<IntElement>;

    /// Creates a snapshot of a given value.
    /// Useful for when object is sometimes created via snapshot() or string_concat(), and sometimes directly.
    public native fun create_snapshot<IntElement: copy + drop>(value: IntElement): AggregatorSnapshot<IntElement>;

    /// Returns a value stored in this snapshot.
    /// Note: This operation is resource-intensive, and reduces parallelism.
    /// (Especially if called in a transaction that also modifies the aggregator,
    /// or has other read/write conflicts)
    ///
    /// Parallelism info: This operation *prevents* speculative parallelism.
    public native fun read_snapshot<IntElement>(snapshot: &AggregatorSnapshot<IntElement>): IntElement;

    /// Returns a value stored in this DerivedStringSnapshot.
    /// Note: This operation is resource-intensive, and reduces parallelism.
    /// (Especially if called in a transaction that also modifies the aggregator,
    /// or has other read/write conflicts)
    ///
    /// Parallelism info: This operation *prevents* speculative parallelism.
    public native fun read_derived_string(snapshot: &DerivedStringSnapshot): String;

    /// Creates a DerivedStringSnapshot of a given value.
    /// Useful for when object is sometimes created via string_concat(), and sometimes directly.
    public native fun create_derived_string(value: String): DerivedStringSnapshot;

    /// Concatenates `before`, `snapshot` and `after` into a single string.
    /// snapshot passed needs to have integer type - currently supported types are u64 and u128.
    /// Raises EUNSUPPORTED_AGGREGATOR_SNAPSHOT_TYPE if called with another type.
    /// If length of prefix and suffix together exceed 256 bytes, ECONCAT_STRING_LENGTH_TOO_LARGE is raised.
    ///
    /// Parallelism info: This operation enables parallelism.
    public native fun derive_string_concat<IntElement>(before: String, snapshot: &AggregatorSnapshot<IntElement>, after: String): DerivedStringSnapshot;

    // ===== DEPRECATE/NOT YET IMPLEMENTED ====

    #[deprecated]
    /// NOT YET IMPLEMENTED, always raises EAGGREGATOR_FUNCTION_NOT_YET_SUPPORTED.
    public native fun copy_snapshot<IntElement: copy + drop>(snapshot: &AggregatorSnapshot<IntElement>): AggregatorSnapshot<IntElement>;

    #[deprecated]
    /// DEPRECATED, use derive_string_concat() instead. always raises EAGGREGATOR_FUNCTION_NOT_YET_SUPPORTED.
    public native fun string_concat<IntElement>(before: String, snapshot: &AggregatorSnapshot<IntElement>, after: String): AggregatorSnapshot<String>;

    // ========================================

    #[test]
    fun test_aggregator() {
        let agg = create_aggregator(10);
        assert!(try_add(&mut agg, 5), 1);
        assert!(try_add(&mut agg, 5), 2);
        assert!(read(&agg) == 10, 3);
        assert!(!try_add(&mut agg, 5), 4);
        assert!(read(&agg) == 10, 5);
        assert!(try_sub(&mut agg, 5), 6);
        assert!(read(&agg) == 5, 7);

        let snap = snapshot(&agg);
        assert!(try_add(&mut agg, 2), 8);
        assert!(read(&agg) == 7, 9);
        assert!(read_snapshot(&snap) == 5, 10);
    }

    #[test]
    fun test_correct_read() {
        let snapshot = create_snapshot(42);
        assert!(read_snapshot(&snapshot) == 42, 0);

        let derived = create_derived_string(std::string::utf8(b"42"));
        assert!(read_derived_string(&derived) == std::string::utf8(b"42"), 0);
    }

    #[test]
    #[expected_failure(abort_code = 0x030009, location = Self)]
    fun test_copy_not_yet_supported() {
        let snapshot = create_snapshot(42);
        copy_snapshot(&snapshot);
    }

    #[test]
    fun test_string_concat1() {
        let snapshot = create_snapshot(42);
        let derived = derive_string_concat(std::string::utf8(b"before"), &snapshot, std::string::utf8(b"after"));
        assert!(read_derived_string(&derived) == std::string::utf8(b"before42after"), 0);
    }

    #[test]
    #[expected_failure(abort_code = 0x030007, location = Self)]
    fun test_aggregator_invalid_type1() {
        create_unbounded_aggregator<u8>();
    }

    #[test]
    fun test_aggregator_valid_type() {
        create_unbounded_aggregator<u64>();
        create_unbounded_aggregator<u128>();
        create_aggregator<u64>(5);
        create_aggregator<u128>(5);
    }

    #[test]
    #[expected_failure(abort_code = 0x030005, location = Self)]
    fun test_snpashot_invalid_type1() {
        use std::option;
        create_snapshot(option::some(42));
    }

    #[test]
    #[expected_failure(abort_code = 0x030005, location = Self)]
    fun test_snpashot_invalid_type2() {
        create_snapshot(vector[42]);
    }
}
