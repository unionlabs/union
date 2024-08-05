/// Provides a framework for comparing two elements
module aptos_std::comparator {
    use std::bcs;
    use std::vector;

    const EQUAL: u8 = 0;
    const SMALLER: u8 = 1;
    const GREATER: u8 = 2;

    struct Result has drop {
        inner: u8,
    }

    public fun is_equal(result: &Result): bool {
        result.inner == EQUAL
    }

    public fun is_smaller_than(result: &Result): bool {
        result.inner == SMALLER
    }

    public fun is_greater_than(result: &Result): bool {
        result.inner == GREATER
    }

    // Performs a comparison of two types after BCS serialization.
    // BCS uses little endian encoding for all integer types,
    // so comparison between primitive integer types will not behave as expected.
    // For example, 1(0x1) will be larger than 256(0x100) after BCS serialization.
    public fun compare<T>(left: &T, right: &T): Result {
        let left_bytes = bcs::to_bytes(left);
        let right_bytes = bcs::to_bytes(right);

        compare_u8_vector(left_bytes, right_bytes)
    }

    // Performs a comparison of two vector<u8>s or byte vectors
    public fun compare_u8_vector(left: vector<u8>, right: vector<u8>): Result {
        let left_length = vector::length(&left);
        let right_length = vector::length(&right);

        let idx = 0;

        while (idx < left_length && idx < right_length) {
            let left_byte = *vector::borrow(&left, idx);
            let right_byte = *vector::borrow(&right, idx);

            if (left_byte < right_byte) {
                return Result { inner: SMALLER }
            } else if (left_byte > right_byte) {
                return Result { inner: GREATER }
            };
            idx = idx + 1;
        };

        if (left_length < right_length) {
            Result { inner: SMALLER }
        } else if (left_length > right_length) {
            Result { inner: GREATER }
        } else {
            Result { inner: EQUAL }
        }
    }

    #[test]
    public fun test_strings() {
        use std::string;

        let value0 = string::utf8(b"alpha");
        let value1 = string::utf8(b"beta");
        let value2 = string::utf8(b"betaa");

        assert!(is_equal(&compare(&value0, &value0)), 0);
        assert!(is_equal(&compare(&value1, &value1)), 1);
        assert!(is_equal(&compare(&value2, &value2)), 2);

        assert!(is_greater_than(&compare(&value0, &value1)), 3);
        assert!(is_smaller_than(&compare(&value1, &value0)), 4);

        assert!(is_smaller_than(&compare(&value0, &value2)), 5);
        assert!(is_greater_than(&compare(&value2, &value0)), 6);

        assert!(is_smaller_than(&compare(&value1, &value2)), 7);
        assert!(is_greater_than(&compare(&value2, &value1)), 8);
    }

    #[test]
    #[expected_failure]
    public fun test_integer() {
        // 1(0x1) will be larger than 256(0x100) after BCS serialization.
        let value0: u128 = 1;
        let value1: u128 = 256;

        assert!(is_equal(&compare(&value0, &value0)), 0);
        assert!(is_equal(&compare(&value1, &value1)), 1);

        assert!(is_smaller_than(&compare(&value0, &value1)), 2);
        assert!(is_greater_than(&compare(&value1, &value0)), 3);
    }

    #[test]
    public fun test_u128() {
        let value0: u128 = 5;
        let value1: u128 = 152;
        let value2: u128 = 511; // 0x1ff

        assert!(is_equal(&compare(&value0, &value0)), 0);
        assert!(is_equal(&compare(&value1, &value1)), 1);
        assert!(is_equal(&compare(&value2, &value2)), 2);

        assert!(is_smaller_than(&compare(&value0, &value1)), 2);
        assert!(is_greater_than(&compare(&value1, &value0)), 3);

        assert!(is_smaller_than(&compare(&value0, &value2)), 3);
        assert!(is_greater_than(&compare(&value2, &value0)), 4);

        assert!(is_smaller_than(&compare(&value1, &value2)), 5);
        assert!(is_greater_than(&compare(&value2, &value1)), 6);
    }

    #[test_only]
    struct Complex has drop {
        value0: vector<u128>,
        value1: u8,
        value2: u64,
    }

    #[test]
    public fun test_complex() {
        let value0_0 = vector::empty();
        vector::push_back(&mut value0_0, 10);
        vector::push_back(&mut value0_0, 9);
        vector::push_back(&mut value0_0, 5);

        let value0_1 = vector::empty();
        vector::push_back(&mut value0_1, 10);
        vector::push_back(&mut value0_1, 9);
        vector::push_back(&mut value0_1, 5);
        vector::push_back(&mut value0_1, 1);

        let base = Complex {
            value0: value0_0,
            value1: 13,
            value2: 41,
        };

        let other_0 = Complex {
            value0: value0_1,
            value1: 13,
            value2: 41,
        };

        let other_1 = Complex {
            value0: copy value0_0,
            value1: 14,
            value2: 41,
        };

        let other_2 = Complex {
            value0: value0_0,
            value1: 13,
            value2: 42,
        };

        assert!(is_equal(&compare(&base, &base)), 0);
        assert!(is_smaller_than(&compare(&base, &other_0)), 1);
        assert!(is_greater_than(&compare(&other_0, &base)), 2);
        assert!(is_smaller_than(&compare(&base, &other_1)), 3);
        assert!(is_greater_than(&compare(&other_1, &base)), 4);
        assert!(is_smaller_than(&compare(&base, &other_2)), 5);
        assert!(is_greater_than(&compare(&other_2, &base)), 6);
    }
}
