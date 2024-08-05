/// Gas parameters for global storage.
///
/// # General overview sections
///
/// [Definitions](#definitions)
///
/// * [Utilization dimensions](#utilization-dimensions)
/// * [Utilization ratios](#utilization-ratios)
/// * [Gas curve lookup](#gas-curve-lookup)
/// * [Item-wise operations](#item-wise-operations)
/// * [Byte-wise operations](#byte-wise-operations)
///
/// [Function dependencies](#function-dependencies)
///
/// * [Initialization](#initialization)
/// * [Reconfiguration](#reconfiguration)
/// * [Setting configurations](#setting-configurations)
///
/// # Definitions
///
/// ## Utilization dimensions
///
/// Global storage gas fluctuates each epoch based on total utilization,
/// which is defined across two dimensions:
///
/// 1. The number of "items" in global storage.
/// 2. The number of bytes in global storage.
///
/// "Items" include:
///
/// 1. Resources having the `key` attribute, which have been moved into
///    global storage via a `move_to()` operation.
/// 2.  Table entries.
///
/// ## Utilization ratios
///
/// `initialize()` sets an arbitrary "target" utilization for both
/// item-wise and byte-wise storage, then each epoch, gas parameters are
/// reconfigured based on the "utilization ratio" for each of the two
/// utilization dimensions. The utilization ratio for a given dimension,
/// either item-wise or byte-wise, is taken as the quotient of actual
/// utilization and target utilization. For example, given a 500 GB
/// target and 250 GB actual utilization, the byte-wise utilization
/// ratio is 50%.
///
/// See `base_8192_exponential_curve()` for mathematical definitions.
///
/// ## Gas curve lookup
///
/// The utilization ratio in a given epoch is used as a lookup value in
/// a Eulerian approximation to an exponential curve, known as a
/// `GasCurve`, which is defined in `base_8192_exponential_curve()`,
/// based on a minimum gas charge and a maximum gas charge.
///
/// The minimum gas charge and maximum gas charge at the endpoints of
/// the curve are set in `initialize()`, and correspond to the following
/// operations defined in `StorageGas`:
///
/// 1. Per-item read
/// 2. Per-item create
/// 3. Per-item write
/// 4. Per-byte read
/// 5. Per-byte create
/// 6. Per-byte write
///
/// For example, if the byte-wise utilization ratio is 50%, then
/// per-byte reads will charge the minimum per-byte gas cost, plus
/// 1.09% of the difference between the maximum and the minimum cost.
/// See `base_8192_exponential_curve()` for a supporting calculation.
///
/// ## Item-wise operations
///
/// 1. Per-item read gas is assessed whenever an item is read from
///    global storage via `borrow_global<T>()` or via a table entry read
///    operation.
/// 2. Per-item create gas is assessed whenever an item is created in
///    global storage via `move_to<T>()` or via a table entry creation
///    operation.
/// 3. Per-item write gas is assessed whenever an item is overwritten in
///    global storage via `borrow_global_mut<T>` or via a table entry
///    mutation operation.
///
/// ## Byte-wise operations
///
/// Byte-wise operations are assessed in a manner similar to per-item
/// operations, but account for the number of bytes affected by the
/// given operation. Notably, this number denotes the total number of
/// bytes in an *entire item*.
///
/// For example, if an operation mutates a `u8` field in a resource that
/// has 5 other `u128` fields, the per-byte gas write cost will account
/// for $(5 * 128) / 8 + 1 = 81$ bytes. Vectors are similarly treated
/// as fields.
///
/// # Function dependencies
///
/// The below dependency chart uses `mermaid.js` syntax, which can be
/// automatically rendered into a diagram (depending on the browser)
/// when viewing the documentation file generated from source code. If
/// a browser renders the diagrams with coloring that makes it difficult
/// to read, try a different browser.
///
/// ## Initialization
///
/// ```mermaid
///
/// flowchart LR
///
/// initialize --> base_8192_exponential_curve
/// base_8192_exponential_curve --> new_gas_curve
/// base_8192_exponential_curve --> new_point
/// new_gas_curve --> validate_points
///
/// ```
///
/// ## Reconfiguration
///
/// ```mermaid
///
/// flowchart LR
///
/// calculate_gas --> Interpolate %% capitalized
/// calculate_read_gas --> calculate_gas
/// calculate_create_gas --> calculate_gas
/// calculate_write_gas --> calculate_gas
/// on_reconfig --> calculate_read_gas
/// on_reconfig --> calculate_create_gas
/// on_reconfig --> calculate_write_gas
/// reconfiguration::reconfigure --> on_reconfig
///
/// ```
///
/// Here, the function `interpolate()` is spelled `Interpolate` because
/// `interpolate` is a reserved word in `mermaid.js`.
///
/// ## Setting configurations
///
/// ```mermaid
///
/// flowchart LR
///
/// gas_schedule::set_storage_gas_config --> set_config
///
/// ```
///
/// # Complete docgen index
///
/// The below index is automatically generated from source code:
module aptos_framework::storage_gas {

    use aptos_framework::system_addresses;
    use std::error;
    use aptos_framework::state_storage;
    use std::vector;

    friend aptos_framework::gas_schedule;
    friend aptos_framework::genesis;
    friend aptos_framework::reconfiguration;

    const ESTORAGE_GAS_CONFIG: u64 = 0;
    const ESTORAGE_GAS: u64 = 1;
    const EINVALID_GAS_RANGE: u64 = 2;
    const EZERO_TARGET_USAGE: u64 = 3;
    const ETARGET_USAGE_TOO_BIG: u64 = 4;
    const EINVALID_MONOTONICALLY_NON_DECREASING_CURVE: u64 = 5;
    const EINVALID_POINT_RANGE: u64 = 6;

    const BASIS_POINT_DENOMINATION: u64 = 10000;

    const MAX_U64: u64 = 18446744073709551615;

    /// Storage parameters, reconfigured each epoch.
    ///
    /// Parameters are updated during reconfiguration via
    /// `on_reconfig()`, based on storage utilization at the beginning
    /// of the epoch in which the reconfiguration transaction is
    /// executed. The gas schedule derived from these parameters will
    /// then be used to calculate gas for the entirety of the
    /// following epoch, such that the data is one epoch older than
    /// ideal. Notably, however, per this approach, the virtual machine
    /// does not need to reload gas parameters after the
    /// first transaction of an epoch.
    struct StorageGas has key {
        /// Cost to read an item from global storage.
        per_item_read: u64,
        /// Cost to create an item in global storage.
        per_item_create: u64,
        /// Cost to overwrite an item in global storage.
        per_item_write: u64,
        /// Cost to read a byte from global storage.
        per_byte_read: u64,
        /// Cost to create a byte in global storage.
        per_byte_create: u64,
        /// Cost to overwrite a byte in global storage.
        per_byte_write: u64,
    }

    /// A point in a Eulerian curve approximation, with each coordinate
    /// given in basis points:
    ///
    /// | Field value | Percentage |
    /// |-------------|------------|
    /// | `1`         | 00.01 %    |
    /// | `10`        | 00.10 %    |
    /// | `100`       | 01.00 %    |
    /// | `1000`      | 10.00 %    |
    struct Point has copy, drop, store {
        /// x-coordinate basis points, corresponding to utilization
        /// ratio in `base_8192_exponential_curve()`.
        x: u64,
        /// y-coordinate basis points, corresponding to utilization
        /// multiplier in `base_8192_exponential_curve()`.
        y: u64
    }

    /// A gas configuration for either per-item or per-byte costs.
    ///
    /// Contains a target usage amount, as well as a Eulerian
    /// approximation of an exponential curve for reads, creations, and
    /// overwrites. See `StorageGasConfig`.
    struct UsageGasConfig has copy, drop, store {
        target_usage: u64,
        read_curve: GasCurve,
        create_curve: GasCurve,
        write_curve: GasCurve,
    }

    /// Eulerian approximation of an exponential curve.
    ///
    /// Assumes the following endpoints:
    ///
    /// * $(x_0, y_0) = (0, 0)$
    /// * $(x_f, y_f) = (10000, 10000)$
    ///
    /// Intermediate points must satisfy:
    ///
    /// 1. $x_i > x_{i - 1}$ ( $x$ is strictly increasing).
    /// 2. $0 \leq x_i \leq 10000$ ( $x$ is between 0 and 10000).
    /// 3. $y_i \geq y_{i - 1}$ ( $y$ is non-decreasing).
    /// 4. $0 \leq y_i \leq 10000$ ( $y$ is between 0 and 10000).
    ///
    /// Lookup between two successive points is calculated via linear
    /// interpolation, e.g., as if there were a straight line between
    /// them.
    ///
    /// See `base_8192_exponential_curve()`.
    struct GasCurve has copy, drop, store {
        min_gas: u64,
        max_gas: u64,
        points: vector<Point>,
    }

    /// Default exponential curve having base 8192.
    ///
    /// # Function definition
    ///
    /// Gas price as a function of utilization ratio is defined as:
    ///
    /// $$g(u_r) = g_{min} + \frac{(b^{u_r} - 1)}{b - 1} \Delta_g$$
    ///
    /// $$g(u_r) = g_{min} + u_m \Delta_g$$
    ///
    /// | Variable                            | Description            |
    /// |-------------------------------------|------------------------|
    /// | $g_{min}$                           | `min_gas`              |
    /// | $g_{max}$                           | `max_gas`              |
    /// | $\Delta_{g} = g_{max} - g_{min}$    | Gas delta              |
    /// | $u$                                 | Utilization            |
    /// | $u_t$                               | Target utilization     |
    /// | $u_r = u / u_t$                     | Utilization ratio      |
    /// | $u_m = \frac{(b^{u_r} - 1)}{b - 1}$ | Utilization multiplier |
    /// | $b = 8192$                          | Exponent base          |
    ///
    /// # Example
    ///
    /// Hence for a utilization ratio of 50% ( $u_r = 0.5$ ):
    ///
    /// $$g(0.5) = g_{min} + \frac{8192^{0.5} - 1}{8192 - 1} \Delta_g$$
    ///
    /// $$g(0.5) \approx g_{min} + 0.0109 \Delta_g$$
    ///
    /// Which means that the price above `min_gas` is approximately
    /// 1.09% of the difference between `max_gas` and `min_gas`.
    ///
    /// # Utilization multipliers
    ///
    /// | $u_r$ | $u_m$ (approximate) |
    /// |-------|---------------------|
    /// | 10%   | 0.02%               |
    /// | 20%   | 0.06%               |
    /// | 30%   | 0.17%               |
    /// | 40%   | 0.44%               |
    /// | 50%   | 1.09%               |
    /// | 60%   | 2.71%               |
    /// | 70%   | 6.69%               |
    /// | 80%   | 16.48%              |
    /// | 90%   | 40.61%              |
    /// | 95%   | 63.72%              |
    /// | 99%   | 91.38%              |
    public fun base_8192_exponential_curve(min_gas: u64, max_gas: u64): GasCurve {
        new_gas_curve(min_gas, max_gas,
            vector[
                new_point(1000, 2),
                new_point(2000, 6),
                new_point(3000, 17),
                new_point(4000, 44),
                new_point(5000, 109),
                new_point(6000, 271),
                new_point(7000, 669),
                new_point(8000, 1648),
                new_point(9000, 4061),
                new_point(9500, 6372),
                new_point(9900, 9138),
            ]
        )
    }

    /// Gas configurations for per-item and per-byte prices.
    struct StorageGasConfig has copy, drop, key {
        /// Per-item gas configuration.
        item_config: UsageGasConfig,
        /// Per-byte gas configuration.
        byte_config: UsageGasConfig,
    }

    public fun new_point(x: u64, y: u64): Point {
        assert!(
            x <= BASIS_POINT_DENOMINATION && y <= BASIS_POINT_DENOMINATION,
            error::invalid_argument(EINVALID_POINT_RANGE)
        );
        Point { x, y }
    }

    public fun new_gas_curve(min_gas: u64, max_gas: u64, points: vector<Point>): GasCurve {
        assert!(max_gas >= min_gas, error::invalid_argument(EINVALID_GAS_RANGE));
        assert!(max_gas <= MAX_U64 / BASIS_POINT_DENOMINATION, error::invalid_argument(EINVALID_GAS_RANGE));
        validate_points(&points);
        GasCurve {
            min_gas,
            max_gas,
            points
        }
    }

    public fun new_usage_gas_config(target_usage: u64, read_curve: GasCurve, create_curve: GasCurve, write_curve: GasCurve): UsageGasConfig {
        assert!(target_usage > 0, error::invalid_argument(EZERO_TARGET_USAGE));
        assert!(target_usage <= MAX_U64 / BASIS_POINT_DENOMINATION, error::invalid_argument(ETARGET_USAGE_TOO_BIG));
        UsageGasConfig {
            target_usage,
            read_curve,
            create_curve,
            write_curve,
        }
    }

    public fun new_storage_gas_config(item_config: UsageGasConfig, byte_config: UsageGasConfig): StorageGasConfig {
        StorageGasConfig {
            item_config,
            byte_config
        }
    }

    public(friend) fun set_config(aptos_framework: &signer, config: StorageGasConfig) acquires StorageGasConfig {
        system_addresses::assert_aptos_framework(aptos_framework);
        *borrow_global_mut<StorageGasConfig>(@aptos_framework) = config;
    }

    /// Initialize per-item and per-byte gas prices.
    ///
    /// Target utilization is set to 2 billion items and 1 TB.
    ///
    /// `GasCurve` endpoints are initialized as follows:
    ///
    /// | Data style | Operation | Minimum gas | Maximum gas |
    /// |------------|-----------|-------------|-------------|
    /// | Per item   | Read      | 300K        | 300K * 100  |
    /// | Per item   | Create    | 300k        | 300k * 100    |
    /// | Per item   | Write     | 300K        | 300K * 100  |
    /// | Per byte   | Read      | 300         | 300 * 100   |
    /// | Per byte   | Create    | 5K          | 5K * 100    |
    /// | Per byte   | Write     | 5K          | 5K * 100    |
    ///
    /// `StorageGas` values are additionally initialized, but per
    /// `on_reconfig()`, they will be reconfigured for each subsequent
    /// epoch after initialization.
    ///
    /// See `base_8192_exponential_curve()` fore more information on
    /// target utilization.
    public fun initialize(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(
            !exists<StorageGasConfig>(@aptos_framework),
            error::already_exists(ESTORAGE_GAS_CONFIG)
        );

        let k: u64 = 1000;
        let m: u64 = 1000 * 1000;

        let item_config = UsageGasConfig {
            target_usage: 2 * k * m, // 2 billion
            read_curve: base_8192_exponential_curve(300 * k, 300 * k * 100),
            create_curve: base_8192_exponential_curve(300 * k, 300 * k * 100),
            write_curve: base_8192_exponential_curve(300 * k, 300 * k * 100),
        };
        let byte_config = UsageGasConfig {
            target_usage: 1 * m * m, // 1TB
            read_curve: base_8192_exponential_curve(300, 300 * 100),
            create_curve: base_8192_exponential_curve(5 * k,  5 * k * 100),
            write_curve: base_8192_exponential_curve(5 * k,  5 * k * 100),
        };
        move_to(aptos_framework, StorageGasConfig {
            item_config,
            byte_config,
        });

        assert!(
            !exists<StorageGas>(@aptos_framework),
            error::already_exists(ESTORAGE_GAS)
        );
        move_to(aptos_framework, StorageGas {
            per_item_read: 300 * k,
            per_item_create: 5 * m,
            per_item_write: 300 * k,
            per_byte_read: 300,
            per_byte_create: 5 * k,
            per_byte_write: 5 * k,
        });
    }

    fun validate_points(points: &vector<Point>) {
        let len = vector::length(points);
        spec {
            assume len < MAX_U64;
        };
        let i = 0;
        while ({
            spec {
                invariant forall j in 0..i: {
                    let cur = if (j == 0) { Point { x: 0, y: 0 } } else { points[j - 1] };
                    let next = if (j == len) { Point { x: BASIS_POINT_DENOMINATION, y: BASIS_POINT_DENOMINATION } } else { points[j] };
                    cur.x < next.x && cur.y <= next.y
                };
            };
            i <= len
        }) {
            let cur = if (i == 0) { &Point { x: 0, y: 0 } } else { vector::borrow(points, i - 1) };
            let next = if (i == len) { &Point { x: BASIS_POINT_DENOMINATION, y: BASIS_POINT_DENOMINATION } } else { vector::borrow(points, i) };
            assert!(cur.x < next.x && cur.y <= next.y, error::invalid_argument(EINVALID_MONOTONICALLY_NON_DECREASING_CURVE));
            i = i + 1;
        }
    }

    fun calculate_gas(max_usage: u64, current_usage: u64, curve: &GasCurve): u64 {
        let capped_current_usage = if (current_usage > max_usage) max_usage else current_usage;
        let points = &curve.points;
        let num_points = vector::length(points);
        let current_usage_bps = capped_current_usage * BASIS_POINT_DENOMINATION / max_usage;

        // Check the corner case that current_usage_bps drops before the first point.
        let (left, right) = if (num_points == 0) {
            (&Point { x: 0, y: 0 }, &Point { x: BASIS_POINT_DENOMINATION, y: BASIS_POINT_DENOMINATION })
        } else if (current_usage_bps < vector::borrow(points, 0).x) {
            (&Point { x: 0, y: 0 }, vector::borrow(points, 0))
        } else if (vector::borrow(points, num_points - 1).x <= current_usage_bps) {
            (vector::borrow(points, num_points - 1), &Point { x: BASIS_POINT_DENOMINATION, y: BASIS_POINT_DENOMINATION })
        } else {
            let (i, j) = (0, num_points - 2);
            while ({
                spec {
                    invariant i <= j;
                    invariant j < num_points - 1;
                    invariant points[i].x <= current_usage_bps;
                    invariant current_usage_bps < points[j + 1].x;
                };
                i < j
            }) {
                let mid = j - (j - i) / 2;
                if (current_usage_bps < vector::borrow(points, mid).x) {
                    spec {
                        // j is strictly decreasing.
                        assert mid - 1 < j;
                    };
                    j = mid - 1;
                } else {
                    spec {
                        // i is strictly increasing.
                        assert i < mid;
                    };
                    i = mid;
                };
            };
            (vector::borrow(points, i), vector::borrow(points, i + 1))
        };
        let y_interpolated = interpolate(left.x, right.x, left.y, right.y, current_usage_bps);
        interpolate(0, BASIS_POINT_DENOMINATION, curve.min_gas, curve.max_gas, y_interpolated)
    }

    // Interpolates y for x on the line between (x0, y0) and (x1, y1).
    fun interpolate(x0: u64, x1: u64, y0: u64, y1: u64, x: u64): u64 {
        y0 + (x - x0) * (y1 - y0) / (x1 - x0)
    }

    fun calculate_read_gas(config: &UsageGasConfig, usage: u64): u64 {
        calculate_gas(config.target_usage, usage, &config.read_curve)
    }

    fun calculate_create_gas(config: &UsageGasConfig, usage: u64): u64 {
        calculate_gas(config.target_usage, usage, &config.create_curve)
    }

    fun calculate_write_gas(config: &UsageGasConfig, usage: u64): u64 {
        calculate_gas(config.target_usage, usage, &config.write_curve)
    }

    public(friend) fun on_reconfig() acquires StorageGas, StorageGasConfig {
        assert!(
            exists<StorageGasConfig>(@aptos_framework),
            error::not_found(ESTORAGE_GAS_CONFIG)
        );
        assert!(
            exists<StorageGas>(@aptos_framework),
            error::not_found(ESTORAGE_GAS)
        );
        let (items, bytes) = state_storage::current_items_and_bytes();
        let gas_config = borrow_global<StorageGasConfig>(@aptos_framework);
        let gas = borrow_global_mut<StorageGas>(@aptos_framework);
        gas.per_item_read = calculate_read_gas(&gas_config.item_config, items);
        gas.per_item_create = calculate_create_gas(&gas_config.item_config, items);
        gas.per_item_write = calculate_write_gas(&gas_config.item_config, items);
        gas.per_byte_read = calculate_read_gas(&gas_config.byte_config, bytes);
        gas.per_byte_create = calculate_create_gas(&gas_config.byte_config, bytes);
        gas.per_byte_write = calculate_write_gas(&gas_config.byte_config, bytes);
    }

    // TODO: reactivate this test after fixing assertions
    //#[test(framework = @aptos_framework)]
    #[test_only]
    fun test_initialize_and_reconfig(framework: signer) acquires StorageGas, StorageGasConfig {
        state_storage::initialize(&framework);
        initialize(&framework);
        on_reconfig();
        let gas_parameter = borrow_global<StorageGas>(@aptos_framework);
        assert!(gas_parameter.per_item_read == 10, 0);
        assert!(gas_parameter.per_item_create == 10, 0);
        assert!(gas_parameter.per_item_write == 10, 0);
        assert!(gas_parameter.per_byte_read == 1, 0);
        assert!(gas_parameter.per_byte_create == 1, 0);
        assert!(gas_parameter.per_byte_write == 1, 0);
    }

    #[test]
    fun test_curve() {
        let constant_curve = new_gas_curve(5, 5, vector[]);
        let linear_curve = new_gas_curve(1, 1000, vector[]);
        let standard_curve = base_8192_exponential_curve(1, 1000);
        let target = BASIS_POINT_DENOMINATION / 2;
        while (target < 2 * BASIS_POINT_DENOMINATION) {
            let i = 0;
            let old_standard_curve_gas = 1;
            while (i <= target + 7) {
                assert!(calculate_gas(target, i, &constant_curve) == 5, 0);
                assert!(calculate_gas(target, i, &linear_curve) == (if (i < target) { 1 + 999 * (i * BASIS_POINT_DENOMINATION / target) / BASIS_POINT_DENOMINATION } else { 1000 }), 0);
                let new_standard_curve_gas = calculate_gas(target, i, &standard_curve);
                assert!(new_standard_curve_gas >= old_standard_curve_gas, 0);
                old_standard_curve_gas = new_standard_curve_gas;
                i = i + 3;
            };
            assert!(old_standard_curve_gas == 1000, 0);
            target = target + BASIS_POINT_DENOMINATION;
        }
    }

    #[test(framework = @aptos_framework)]
    fun test_set_storage_gas_config(framework: signer) acquires StorageGas, StorageGasConfig {
        state_storage::initialize(&framework);
        initialize(&framework);
        let item_curve = new_gas_curve(1000, 2000,
            vector[new_point(3000, 0), new_point(5000, 5000), new_point(8000, 5000)]
        );
        let byte_curve = new_gas_curve(0, 1000, vector::singleton<Point>(new_point(5000, 3000)));
        let item_usage_config = new_usage_gas_config(100, copy item_curve, copy item_curve, copy item_curve);
        let byte_usage_config = new_usage_gas_config(2000, copy byte_curve, copy byte_curve, copy byte_curve);
        let storage_gas_config = new_storage_gas_config(item_usage_config, byte_usage_config);
        set_config(&framework, storage_gas_config);
        {
            state_storage::set_for_test(0, 20, 100);
            on_reconfig();
            let gas_parameter = borrow_global<StorageGas>(@aptos_framework);
            assert!(gas_parameter.per_item_read == 1000, 0);
            assert!(gas_parameter.per_byte_read == 30, 0);
        };
        {
            state_storage::set_for_test(0, 40, 800);
            on_reconfig();
            let gas_parameter = borrow_global<StorageGas>(@aptos_framework);
            assert!(gas_parameter.per_item_create == 1250, 0);
            assert!(gas_parameter.per_byte_create == 240, 0);
        };
        {
            state_storage::set_for_test(0, 60, 1200);
            on_reconfig();
            let gas_parameter = borrow_global<StorageGas>(@aptos_framework);
            assert!(gas_parameter.per_item_write == 1500, 0);
            assert!(gas_parameter.per_byte_write == 440, 0);
        };
        {
            state_storage::set_for_test(0, 90, 1800);
            on_reconfig();
            let gas_parameter = borrow_global<StorageGas>(@aptos_framework);
            assert!(gas_parameter.per_item_create == 1750, 0);
            assert!(gas_parameter.per_byte_create == 860, 0);
        };
        {
            // usage overflow case
            state_storage::set_for_test(0, 110, 2200);
            on_reconfig();
            let gas_parameter = borrow_global<StorageGas>(@aptos_framework);
            assert!(gas_parameter.per_item_read == 2000, 0);
            assert!(gas_parameter.per_byte_read == 1000, 0);
        };
    }
}
