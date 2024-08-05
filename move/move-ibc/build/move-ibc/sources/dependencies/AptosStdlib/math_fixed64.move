/// Standard math utilities missing in the Move Language.

module aptos_std::math_fixed64 {
    use aptos_std::fixed_point64;
    use aptos_std::fixed_point64::FixedPoint64;
    use aptos_std::math128;

    /// Abort code on overflow
    const EOVERFLOW_EXP: u64 = 1;

    /// Natural log 2 in 32 bit fixed point
    const LN2: u256 = 12786308645202655660;  // ln(2) in fixed 64 representation

    /// Square root of fixed point number
    public fun sqrt(x: FixedPoint64): FixedPoint64 {
        let y = fixed_point64::get_raw_value(x);
        let z = (math128::sqrt(y) << 32 as u256);
        z = (z + ((y as u256) << 64) / z) >> 1;
        fixed_point64::create_from_raw_value((z as u128))
    }

    /// Exponent function with a precission of 9 digits.
    public fun exp(x: FixedPoint64): FixedPoint64 {
        let raw_value = (fixed_point64::get_raw_value(x) as u256);
        fixed_point64::create_from_raw_value((exp_raw(raw_value) as u128))
    }

    /// Because log2 is negative for values < 1 we instead return log2(x) + 64 which
    /// is positive for all values of x.
    public fun log2_plus_64(x: FixedPoint64): FixedPoint64 {
        let raw_value = (fixed_point64::get_raw_value(x) as u128);
        math128::log2_64(raw_value)
    }

    public fun ln_plus_32ln2(x: FixedPoint64): FixedPoint64 {
        let raw_value = fixed_point64::get_raw_value(x);
        let x = (fixed_point64::get_raw_value(math128::log2_64(raw_value)) as u256);
        fixed_point64::create_from_raw_value(((x * LN2) >> 64 as u128))
    }

    /// Integer power of a fixed point number
    public fun pow(x: FixedPoint64, n: u64): FixedPoint64 {
        let raw_value = (fixed_point64::get_raw_value(x) as u256);
        fixed_point64::create_from_raw_value((pow_raw(raw_value, (n as u128)) as u128))
    }

    /// Specialized function for x * y / z that omits intermediate shifting
    public fun mul_div(x: FixedPoint64, y: FixedPoint64, z: FixedPoint64): FixedPoint64 {
        let a = fixed_point64::get_raw_value(x);
        let b = fixed_point64::get_raw_value(y);
        let c = fixed_point64::get_raw_value(z);
        fixed_point64::create_from_raw_value (math128::mul_div(a, b, c))
    }

    // Calculate e^x where x and the result are fixed point numbers
    fun exp_raw(x: u256): u256 {
        // exp(x / 2^64) = 2^(x / (2^64 * ln(2))) = 2^(floor(x / (2^64 * ln(2))) + frac(x / (2^64 * ln(2))))
        let shift_long = x / LN2;
        assert!(shift_long <= 63, std::error::invalid_state(EOVERFLOW_EXP));
        let shift = (shift_long as u8);
        let remainder = x % LN2;
        // At this point we want to calculate 2^(remainder / ln2) << shift
        // ln2 = 580 * 22045359733108027
        let bigfactor = 22045359733108027;
        let exponent = remainder / bigfactor;
        let x = remainder % bigfactor;
        // 2^(remainder / ln2) = (2^(1/580))^exponent * exp(x / 2^64)
        let roottwo = 18468802611690918839;  // fixed point representation of 2^(1/580)
        // 2^(1/580) = roottwo(1 - eps), so the number we seek is roottwo^exponent (1 - eps * exponent)
        let power = pow_raw(roottwo, (exponent as u128));
        let eps_correction = 219071715585908898;
        power = power - ((power * eps_correction * exponent) >> 128);
        // x is fixed point number smaller than bigfactor/2^64 < 0.0011 so we need only 5 tayler steps
        // to get the 15 digits of precission
        let taylor1 = (power * x) >> (64 - shift);
        let taylor2 = (taylor1 * x) >> 64;
        let taylor3 = (taylor2 * x) >> 64;
        let taylor4 = (taylor3 * x) >> 64;
        let taylor5 = (taylor4 * x) >> 64;
        let taylor6 = (taylor5 * x) >> 64;
        (power << shift) + taylor1 + taylor2 / 2 + taylor3 / 6 + taylor4 / 24 + taylor5 / 120 + taylor6 / 720
    }

    // Calculate x to the power of n, where x and the result are fixed point numbers.
    fun pow_raw(x: u256, n: u128): u256 {
        let res: u256 = 1 << 64;
        while (n != 0) {
            if (n & 1 != 0) {
                res = (res * x) >> 64;
            };
            n = n >> 1;
            x = (x * x) >> 64;
        };
        res
    }

    #[test]
    public entry fun test_sqrt() {
        // Sqrt is based on math128::sqrt and thus most of the testing is done there.
        let fixed_base = 1 << 64;
        let result = sqrt(fixed_point64::create_from_u128(1));
        assert!(fixed_point64::get_raw_value(result) == fixed_base, 0);

        let result = sqrt(fixed_point64::create_from_u128(2));
        assert_approx_the_same((fixed_point64::get_raw_value(result) as u256), 26087635650665564424, 16);
    }

    #[test]
    public entry fun test_exp() {
        let fixed_base = 1 << 64;
        let result = exp_raw(0);
        assert!(result == fixed_base, 0);

        let result = exp_raw(fixed_base);
        let e = 50143449209799256682;  // e in 32 bit fixed point
        assert_approx_the_same(result, e, 16);

        let result = exp_raw(10 * fixed_base);
        let exp10 = 406316577365116946489258;  // e^10 in 32 bit fixed point
        assert_approx_the_same(result, exp10, 16);
    }

    #[test]
    public entry fun test_pow() {
        // We use the case of exp
        let result = pow_raw(18468802611690918839, 580);
        assert_approx_the_same(result,  1 << 65, 16);
    }

    #[test_only]
    /// For functions that approximate a value it's useful to test a value is close
    /// to the most correct value up to last digit
    fun assert_approx_the_same(x: u256, y: u256, precission: u128) {
        if (x < y) {
            let tmp = x;
            x = y;
            y = tmp;
        };
        let mult = (math128::pow(10, precission) as u256);
        assert!((x - y) * mult < x, 0);
    }
}
