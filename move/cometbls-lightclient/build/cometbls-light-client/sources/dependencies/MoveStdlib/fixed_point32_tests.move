#[test_only]
module std::fixed_point32_tests {
    use std::fixed_point32;

    #[test]
    #[expected_failure(abort_code = fixed_point32::EDENOMINATOR)]
    fun create_div_zero() {
        // A denominator of zero should cause an arithmetic error.
        fixed_point32::create_from_rational(2, 0);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::ERATIO_OUT_OF_RANGE)]
    fun create_overflow() {
        // The maximum value is 2^32 - 1. Check that anything larger aborts
        // with an overflow.
        fixed_point32::create_from_rational(4294967296, 1); // 2^32
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::ERATIO_OUT_OF_RANGE)]
    fun create_underflow() {
        // The minimum non-zero value is 2^-32. Check that anything smaller
        // aborts.
        fixed_point32::create_from_rational(1, 8589934592); // 2^-33
    }

    #[test]
    fun create_zero() {
        let x = fixed_point32::create_from_rational(0, 1);
        assert!(fixed_point32::is_zero(x), 0);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::EDIVISION_BY_ZERO)]
    fun divide_by_zero() {
        // Dividing by zero should cause an arithmetic error.
        let f = fixed_point32::create_from_raw_value(0);
        fixed_point32::divide_u64(1, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::EDIVISION)]
    fun divide_overflow_small_divisore() {
        let f = fixed_point32::create_from_raw_value(1); // 0x0.00000001
        // Divide 2^32 by the minimum fractional value. This should overflow.
        fixed_point32::divide_u64(4294967296, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::EDIVISION)]
    fun divide_overflow_large_numerator() {
        let f = fixed_point32::create_from_rational(1, 2); // 0.5
        // Divide the maximum u64 value by 0.5. This should overflow.
        fixed_point32::divide_u64(18446744073709551615, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::EMULTIPLICATION)]
    fun multiply_overflow_small_multiplier() {
        let f = fixed_point32::create_from_rational(3, 2); // 1.5
        // Multiply the maximum u64 value by 1.5. This should overflow.
        fixed_point32::multiply_u64(18446744073709551615, f);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::EMULTIPLICATION)]
    fun multiply_overflow_large_multiplier() {
        let f = fixed_point32::create_from_raw_value(18446744073709551615);
        // Multiply 2^33 by the maximum fixed-point value. This should overflow.
        fixed_point32::multiply_u64(8589934592, f);
    }

    #[test]
    fun exact_multiply() {
        let f = fixed_point32::create_from_rational(3, 4); // 0.75
        let nine = fixed_point32::multiply_u64(12, f); // 12 * 0.75
        assert!(nine == 9, 0);
    }

    #[test]
    fun exact_divide() {
        let f = fixed_point32::create_from_rational(3, 4); // 0.75
        let twelve = fixed_point32::divide_u64(9, f); // 9 / 0.75
        assert!(twelve == 12, 0);
    }

    #[test]
    fun multiply_truncates() {
        let f = fixed_point32::create_from_rational(1, 3); // 0.333...
        let not_three = fixed_point32::multiply_u64(9, copy f); // 9 * 0.333...
        // multiply_u64 does NOT round -- it truncates -- so values that
        // are not perfectly representable in binary may be off by one.
        assert!(not_three == 2, 0);

        // Try again with a fraction slightly larger than 1/3.
        let f = fixed_point32::create_from_raw_value(fixed_point32::get_raw_value(f) + 1);
        let three = fixed_point32::multiply_u64(9, f);
        assert!(three == 3, 1);
    }

    #[test]
    fun create_from_rational_max_numerator_denominator() {
        // Test creating a 1.0 fraction from the maximum u64 value.
        let f = fixed_point32::create_from_rational(18446744073709551615, 18446744073709551615);
        let one = fixed_point32::get_raw_value(f);
        assert!(one == 4294967296, 0); // 0x1.00000000
    }

    #[test]
    fun min_can_return_smaller_fixed_point_number() {
        let one = fixed_point32::create_from_rational(1, 1);
        let two = fixed_point32::create_from_rational(2, 1);
        let smaller_number1 = fixed_point32::min(one, two);
        let val1 = fixed_point32::get_raw_value(smaller_number1);
        assert!(val1 == 4294967296, 0);  // 0x1.00000000
        let smaller_number2 = fixed_point32::min(two, one);
        let val2 = fixed_point32::get_raw_value(smaller_number2);
        assert!(val2 == 4294967296, 0);  // 0x1.00000000
    }

    #[test]
    fun max_can_return_larger_fixed_point_number() {
        let one = fixed_point32::create_from_rational(1, 1);
        let two = fixed_point32::create_from_rational(2, 1);
        let larger_number1 = fixed_point32::max(one, two);
        let larger_number2 = fixed_point32::max(two, one);
        let val1 = fixed_point32::get_raw_value(larger_number1);
        assert!(val1 == 8589934592, 0);  // 0x2.00000000
        let val2 = fixed_point32::get_raw_value(larger_number2);
        assert!(val2 == 8589934592, 0);  // 0x2.00000000
    }

    #[test]
    fun floor_can_return_the_correct_number_zero() {
        let point_five = fixed_point32::create_from_rational(1, 2);
        let val = fixed_point32::floor(point_five);
        assert!(val == 0, 0);
    }

    #[test]
    fun create_from_u64_create_correct_fixed_point_number() {
        let one = fixed_point32::create_from_u64(1);
        let val = fixed_point32::get_raw_value(one);
        assert!(val == 4294967296, 0);
    }

    #[test]
    #[expected_failure(abort_code = fixed_point32::ERATIO_OUT_OF_RANGE)]
    fun create_from_u64_throw_error_when_number_too_large() {
        fixed_point32::create_from_u64(4294967296); // (u64 >> 32) + 1
    }

    #[test]
    fun floor_can_return_the_correct_number_one() {
        let three_point_five = fixed_point32::create_from_rational(7, 2); // 3.5
        let val = fixed_point32::floor(three_point_five);
        assert!(val == 3, 0);
    }

    #[test]
    fun ceil_can_round_up_correctly() {
        let point_five = fixed_point32::create_from_rational(1, 2); // 0.5
        let val = fixed_point32::ceil(point_five);
        assert!(val == 1, 0);
    }

    #[test]
    fun ceil_will_not_change_if_number_already_integer() {
        let one = fixed_point32::create_from_rational(1, 1); // 0.5
        let val = fixed_point32::ceil(one);
        assert!(val == 1, 0);
    }

    #[test]
    fun round_can_round_up_correctly() {
        let point_five = fixed_point32::create_from_rational(1, 2); // 0.5
        let val = fixed_point32::round(point_five);
        assert!(val == 1, 0);
    }

    #[test]
    fun round_can_round_down_correctly() {
        let num = fixed_point32::create_from_rational(499, 1000); // 0.499
        let val = fixed_point32::round(num);
        assert!(val == 0, 0);
    }
}
