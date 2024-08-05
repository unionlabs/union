#[test_only]
module aptos_std::fixed_point64_tests {
    use aptos_std::fixed_point64;

    const POW2_64: u128 = 1 << 64;
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    #[test]
    #[expected_failure(abort_code = fixed_point64::EDENOMINATOR)]
    fun create_div_zero() {
        // A denominator of zero should cause an arithmetic error.
        fixed_point64::create_from_rational(2, 0);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::ERATIO_OUT_OF_RANGE)]
    fun create_overflow() {
        // The maximum value is 2^32 - 1. Check that anything larger aborts
        // with an overflow.
        fixed_point64::create_from_rational(POW2_64, 1); // 2^64
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::ERATIO_OUT_OF_RANGE)]
    fun create_underflow() {
        // The minimum non-zero value is 2^-32. Check that anything smaller
        // aborts.
        fixed_point64::create_from_rational(1, 2 * POW2_64); // 2^-65
    }

    #[test]
    fun create_zero() {
        let x = fixed_point64::create_from_rational(0, 1);
        assert!(fixed_point64::is_zero(x), 0);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::EDIVISION_BY_ZERO)]
    fun divide_by_zero() {
        // Dividing by zero should cause an arithmetic error.
        let f = fixed_point64::create_from_raw_value(0);
        fixed_point64::divide_u128(1, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::EDIVISION)]
    fun divide_overflow_small_divisore() {
        let f = fixed_point64::create_from_raw_value(1); // 2^-64
        // Divide 2^64 by the minimum fractional value. This should overflow.
        fixed_point64::divide_u128(POW2_64, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::EDIVISION)]
    fun divide_overflow_large_numerator() {
        let f = fixed_point64::create_from_rational(1, 2); // 0.5
        // Divide the maximum u128 value by 0.5. This should overflow.
        fixed_point64::divide_u128(MAX_U128, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::EMULTIPLICATION)]
    fun multiply_overflow_small_multiplier() {
        let f = fixed_point64::create_from_rational(3, 2); // 1.5
        // Multiply the maximum u64 value by 1.5. This should overflow.
        fixed_point64::multiply_u128(MAX_U128, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::EMULTIPLICATION)]
    fun multiply_overflow_large_multiplier() {
        let f = fixed_point64::create_from_raw_value(MAX_U128);
        // Multiply 2^65 by the maximum fixed-point value. This should overflow.
        fixed_point64::multiply_u128(2 * POW2_64, f);
    }

    #[test]
    fun exact_multiply() {
        let f = fixed_point64::create_from_rational(3, 4); // 0.75
        let nine = fixed_point64::multiply_u128(12, f); // 12 * 0.75
        assert!(nine == 9, 0);
    }

    #[test]
    fun exact_divide() {
        let f = fixed_point64::create_from_rational(3, 4); // 0.75
        let twelve = fixed_point64::divide_u128(9, f); // 9 / 0.75
        assert!(twelve == 12, 0);
    }

    #[test]
    fun multiply_truncates() {
        let f = fixed_point64::create_from_rational(1, 3); // 0.333...
        let not_three = fixed_point64::multiply_u128(9, copy f); // 9 * 0.333...
        // multiply_u128 does NOT round -- it truncates -- so values that
        // are not perfectly representable in binary may be off by one.
        assert!(not_three == 2, 0);

        // Try again with a fraction slightly larger than 1/3.
        let f = fixed_point64::create_from_raw_value(fixed_point64::get_raw_value(f) + 1);
        let three = fixed_point64::multiply_u128(9, f);
        assert!(three == 3, 1);
    }

    #[test]
    fun create_from_rational_max_numerator_denominator() {
        // Test creating a 1.0 fraction from the maximum u64 value.
        let f = fixed_point64::create_from_rational(MAX_U128, MAX_U128);
        let one = fixed_point64::get_raw_value(f);
        assert!(one == POW2_64, 0); // 0x1.00000000
    }

    #[test]
    fun min_can_return_smaller_fixed_point_number() {
        let one = fixed_point64::create_from_rational(1, 1);
        let two = fixed_point64::create_from_rational(2, 1);
        let smaller_number1 = fixed_point64::min(one, two);
        let val1 = fixed_point64::get_raw_value(smaller_number1);
        assert!(val1 == POW2_64, 0);  // 0x1.00000000
        let smaller_number2 = fixed_point64::min(two, one);
        let val2 = fixed_point64::get_raw_value(smaller_number2);
        assert!(val2 == POW2_64, 0);  // 0x1.00000000
    }

    #[test]
    fun max_can_return_larger_fixed_point_number() {
        let one = fixed_point64::create_from_rational(1, 1);
        let two = fixed_point64::create_from_rational(2, 1);
        let larger_number1 = fixed_point64::max(one, two);
        let larger_number2 = fixed_point64::max(two, one);
        let val1 = fixed_point64::get_raw_value(larger_number1);
        assert!(val1 == 2 * POW2_64, 0);  // 0x2.00000000
        let val2 = fixed_point64::get_raw_value(larger_number2);
        assert!(val2 == 2 * POW2_64, 0);  // 0x2.00000000
    }

    #[test]
    fun floor_can_return_the_correct_number_zero() {
        let point_five = fixed_point64::create_from_rational(1, 2);
        let val = fixed_point64::floor(point_five);
        assert!(val == 0, 0);
    }

    #[test]
    fun create_from_u128_create_correct_fixed_point_number() {
        let one = fixed_point64::create_from_u128(1);
        let val = fixed_point64::get_raw_value(one);
        assert!(val == POW2_64, 0);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point64::ERATIO_OUT_OF_RANGE)]
    fun create_from_u128_throw_error_when_number_too_large() {
        fixed_point64::create_from_u128(POW2_64);
    }

    #[test]
    fun floor_can_return_the_correct_number_one() {
        let three_point_five = fixed_point64::create_from_rational(7, 2); // 3.5
        let val = fixed_point64::floor(three_point_five);
        assert!(val == 3, 0);
    }

    #[test]
    fun ceil_can_round_up_correctly() {
        let point_five = fixed_point64::create_from_rational(1, 2); // 0.5
        let val = fixed_point64::ceil(point_five);
        assert!(val == 1, 0);
    }

    #[test]
    fun ceil_will_not_change_if_number_already_integer() {
        let one = fixed_point64::create_from_rational(1, 1); // 0.5
        let val = fixed_point64::ceil(one);
        assert!(val == 1, 0);
    }

    #[test]
    fun round_can_round_up_correctly() {
        let point_five = fixed_point64::create_from_rational(1, 2); // 0.5
        let val = fixed_point64::round(point_five);
        assert!(val == 1, 0);
    }

    #[test]
    fun round_can_round_down_correctly() {
        let num = fixed_point64::create_from_rational(499, 1000); // 0.499
        let val = fixed_point64::round(num);
        assert!(val == 0, 0);
    }
}
