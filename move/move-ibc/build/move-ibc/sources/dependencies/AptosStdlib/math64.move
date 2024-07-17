/// Standard math utilities missing in the Move Language.
module aptos_std::math64 {

    use std::fixed_point32::FixedPoint32;
    use std::fixed_point32;

    /// Cannot log2 the value 0
    const EINVALID_ARG_FLOOR_LOG2: u64 = 1;

    /// Return the largest of two numbers.
    public fun max(a: u64, b: u64): u64 {
        if (a >= b) a else b
    }

    /// Return the smallest of two numbers.
    public fun min(a: u64, b: u64): u64 {
        if (a < b) a else b
    }

    /// Return the average of two.
    public fun average(a: u64, b: u64): u64 {
        if (a < b) {
            a + (b - a) / 2
        } else {
            b + (a - b) / 2
        }
    }

    /// Return greatest common divisor of `a` & `b`, via the Euclidean algorithm.
    public inline fun gcd(a: u64, b: u64): u64 {
        let (large, small) = if (a > b) (a, b) else (b, a);
        while (small != 0) {
            let tmp = small;
            small = large % small;
            large = tmp;
        };
        large
    }

    /// Returns a * b / c going through u128 to prevent intermediate overflow
    public inline fun mul_div(a: u64, b: u64, c: u64): u64 {
        // Inline functions cannot take constants, as then every module using it needs the constant
        assert!(c != 0, std::error::invalid_argument(4));
        (((a as u128) * (b as u128) / (c as u128)) as u64)
    }

    /// Return x clamped to the interval [lower, upper].
    public fun clamp(x: u64, lower: u64, upper: u64): u64 {
        min(upper, max(lower, x))
    }

    /// Return the value of n raised to power e
    public fun pow(n: u64, e: u64): u64 {
        if (e == 0) {
            1
        } else {
            let p = 1;
            while (e > 1) {
                if (e % 2 == 1) {
                    p = p * n;
                };
                e = e / 2;
                n = n * n;
            };
            p * n
        }
    }

    /// Returns floor(lg2(x))
    public fun floor_log2(x: u64): u8 {
        let res = 0;
        assert!(x != 0, std::error::invalid_argument(EINVALID_ARG_FLOOR_LOG2));
        // Effectively the position of the most significant set bit
        let n = 32;
        while (n > 0) {
            if (x >= (1 << n)) {
                x = x >> n;
                res = res + n;
            };
            n = n >> 1;
        };
        res
    }

    // Returns log2(x)
    public fun log2(x: u64): FixedPoint32 {
        let integer_part = floor_log2(x);
        // Normalize x to [1, 2) in fixed point 32.
        let y = (if (x >= 1 << 32) {
            x >> (integer_part - 32)
        } else {
            x << (32 - integer_part)
        } as u128);
        let frac = 0;
        let delta = 1 << 31;
        while (delta != 0) {
            // log x = 1/2 log x^2
            // x in [1, 2)
            y = (y * y) >> 32;
            // x is now in [1, 4)
            // if x in [2, 4) then log x = 1 + log (x / 2)
            if (y >= (2 << 32)) { frac = frac + delta; y = y >> 1; };
            delta = delta >> 1;
        };
        fixed_point32::create_from_raw_value (((integer_part as u64) << 32) + frac)
    }

    /// Returns square root of x, precisely floor(sqrt(x))
    public fun sqrt(x: u64): u64 {
        if (x == 0) return 0;
        // Note the plus 1 in the expression. Let n = floor_lg2(x) we have x in [2^n, 2^(n+1)> and thus the answer in
        // the half-open interval [2^(n/2), 2^((n+1)/2)>. For even n we can write this as [2^(n/2), sqrt(2) 2^(n/2)>
        // for odd n [2^((n+1)/2)/sqrt(2), 2^((n+1)/2>. For even n the left end point is integer for odd the right
        // end point is integer. If we choose as our first approximation the integer end point we have as maximum
        // relative error either (sqrt(2) - 1) or (1 - 1/sqrt(2)) both are smaller then 1/2.
        let res = 1 << ((floor_log2(x) + 1) >> 1);
        // We use standard newton-rhapson iteration to improve the initial approximation.
        // The error term evolves as delta_i+1 = delta_i^2 / 2 (quadratic convergence).
        // It turns out that after 4 iterations the delta is smaller than 2^-32 and thus below the treshold.
        res = (res + x / res) >> 1;
        res = (res + x / res) >> 1;
        res = (res + x / res) >> 1;
        res = (res + x / res) >> 1;
        min(res, x / res)
    }

    public inline fun ceil_div(x: u64, y: u64): u64 {
        // ceil_div(x, y) = floor((x + y - 1) / y) = floor((x - 1) / y) + 1
        // (x + y - 1) could spuriously overflow. so we use the later version
        if (x == 0) {
            // Inline functions cannot take constants, as then every module using it needs the constant
            assert!(y != 0, std::error::invalid_argument(4));
            0
        }
        else (x - 1) / y + 1
    }

    #[test]
    public entry fun test_ceil_div() {
        assert!(ceil_div(9, 3) == 3, 0);
        assert!(ceil_div(10, 3) == 4, 0);
        assert!(ceil_div(11, 3) == 4, 0);
        assert!(ceil_div(12, 3) == 4, 0);
        assert!(ceil_div(13, 3) == 5, 0);

        // No overflow
        assert!(ceil_div((((1u128<<64) - 9) as u64), 11) == 1676976733973595601, 0);
    }

    #[test]
    fun test_gcd() {
        assert!(gcd(20, 8) == 4, 0);
        assert!(gcd(8, 20) == 4, 0);
        assert!(gcd(1, 100) == 1, 0);
        assert!(gcd(100, 1) == 1, 0);
        assert!(gcd(210, 45) == 15, 0);
        assert!(gcd(45, 210) == 15, 0);
        assert!(gcd(0, 0) == 0, 0);
        assert!(gcd(1, 0) == 1, 0);
        assert!(gcd(50, 0) == 50, 0);
        assert!(gcd(0, 1) == 1, 0);
        assert!(gcd(0, 50) == 50, 0);
        assert!(gcd(54, 24) == 6, 0);
        assert!(gcd(24, 54) == 6, 0);
        assert!(gcd(10, 10) == 10, 0);
        assert!(gcd(1071, 462) == 21, 0);
        assert!(gcd(462, 1071) == 21, 0);
    }

    #[test]
    public entry fun test_max_64() {
        let result = max(3u64, 6u64);
        assert!(result == 6, 0);

        let result = max(15u64, 12u64);
        assert!(result == 15, 1);
    }

    #[test]
    public entry fun test_min() {
        let result = min(3u64, 6u64);
        assert!(result == 3, 0);

        let result = min(15u64, 12u64);
        assert!(result == 12, 1);
    }

    #[test]
    public entry fun test_average() {
        let result = average(3u64, 6u64);
        assert!(result == 4, 0);

        let result = average(15u64, 12u64);
        assert!(result == 13, 0);
    }

    #[test]
    public entry fun test_average_does_not_overflow() {
        let result = average(18446744073709551615, 18446744073709551615);
        assert!(result == 18446744073709551615, 0);
    }

    #[test]
    public entry fun test_pow() {
        let result = pow(10u64, 18u64);
        assert!(result == 1000000000000000000, 0);

        let result = pow(10u64, 1u64);
        assert!(result == 10, 0);

        let result = pow(10u64, 0u64);
        assert!(result == 1, 0);
    }

    #[test]
    public entry fun test_mul_div() {
        let tmp: u64 = 1<<63;
        assert!(mul_div(tmp,tmp,tmp) == tmp, 0);

        assert!(mul_div(tmp,5,5) == tmp, 0);
        // Note that ordering other way is imprecise.
        assert!((tmp / 5) * 5 != tmp, 0);
    }

    #[test]
    #[expected_failure(abort_code = 0x10004, location = aptos_std::math64)]
    public entry fun test_mul_div_by_zero() {
        mul_div(1, 1, 0);
    }

    #[test]
    public entry fun test_floor_lg2() {
        let idx: u8 = 0;
        while (idx < 64) {
            assert!(floor_log2(1<<idx) == idx, 0);
            idx = idx + 1;
        };
        idx = 1;
        while (idx <= 64) {
            assert!(floor_log2((((1u128<<idx) - 1) as u64)) == idx - 1, 0);
            idx = idx + 1;
        };
    }

    #[test]
    public entry fun test_log2() {
        let idx: u8 = 0;
        while (idx < 64) {
            let res = log2(1<<idx);
            assert!(fixed_point32::get_raw_value(res) == (idx as u64) << 32, 0);
            idx = idx + 1;
        };
        idx = 10;
        while (idx <= 64) {
            let res = log2((((1u128<<idx) - 1) as u64));
            // idx + log2 (1 - 1/2^idx) = idx + ln (1-1/2^idx)/ln2
            // Use 3rd order taylor to approximate expected result
            let expected = (idx as u128) << 32;
            let taylor1 = ((1 << 32) / ((1u256<<idx)) as u128);
            let taylor2 = (taylor1 * taylor1) >> 32;
            let taylor3 = (taylor2 * taylor1) >> 32;
            let expected = expected - ((taylor1 + taylor2 / 2 + taylor3 / 3) << 32) / 2977044472;
            // verify it matches to 8 significant digits
            assert_approx_the_same((fixed_point32::get_raw_value(res) as u128), expected, 8);
            idx = idx + 1;
        };
    }

    #[test]
    public entry fun test_sqrt() {
        let result = sqrt(0);
        assert!(result == 0, 0);

        let result = sqrt(1);
        assert!(result == 1, 0);

        let result = sqrt(256);
        assert!(result == 16, 0);

        let result = sqrt(1<<62);
        assert!(result == 1<<31, 0);

        let result = sqrt((((1u128 << 64) - 1) as u64));
        assert!(result == (1u64 << 32) - 1, 0);

        let result = sqrt((1u64 << 63));
        assert!(result == 3037000499, 0);

        let result = sqrt((1u64 << 63) - 1);
        assert!(result == 3037000499, 0);
    }

    #[test_only]
    /// For functions that approximate a value it's useful to test a value is close
    /// to the most correct value up to last digit
    fun assert_approx_the_same(x: u128, y: u128, precission: u64) {
        if (x < y) {
            let tmp = x;
            x = y;
            y = tmp;
        };
        let mult = (pow(10, precission) as u128);
        assert!((x - y) * mult < x, 0);
    }
}
