#[test_only]
module std::bit_vector_tests {
    use std::bit_vector;

    #[test_only]
    fun test_bitvector_set_unset_of_size(k: u64) {
        let bitvector = bit_vector::new(k);
        let index = 0;
        while (index < k) {
            bit_vector::set(&mut bitvector, index);
            assert!(bit_vector::is_index_set(&bitvector, index), 0);
            index = index + 1;
            let index_to_right = index;
            while (index_to_right < k) {
                assert!(!bit_vector::is_index_set(&bitvector, index_to_right), 1);
                index_to_right = index_to_right + 1;
            };
        };
        // now go back down unsetting
        index = 0;

        while (index < k) {
            bit_vector::unset(&mut bitvector, index);
            assert!(!bit_vector::is_index_set(&bitvector, index), 0);
            index = index + 1;
            let index_to_right = index;
            while (index_to_right < k) {
                assert!(bit_vector::is_index_set(&bitvector, index_to_right), 1);
                index_to_right = index_to_right + 1;
            };
        };
    }

    #[test]
    #[expected_failure(abort_code = bit_vector::EINDEX)]
    fun set_bit_out_of_bounds() {
        let bitvector = bit_vector::new(bit_vector::word_size());
        bit_vector::set(&mut bitvector, bit_vector::word_size());
    }

    #[test]
    #[expected_failure(abort_code = bit_vector::EINDEX)]
    fun unset_bit_out_of_bounds() {
        let bitvector = bit_vector::new(bit_vector::word_size());
        bit_vector::unset(&mut bitvector, bit_vector::word_size());
    }

    #[test]
    #[expected_failure(abort_code = bit_vector::EINDEX)]
    fun index_bit_out_of_bounds() {
        let bitvector = bit_vector::new(bit_vector::word_size());
        bit_vector::is_index_set(&mut bitvector, bit_vector::word_size());
    }

    #[test]
    fun test_set_bit_and_index_basic() {
        test_bitvector_set_unset_of_size(8)
    }

    #[test]
    fun test_set_bit_and_index_odd_size() {
        test_bitvector_set_unset_of_size(300)
    }

    #[test]
    fun longest_sequence_no_set_zero_index() {
        let bitvector = bit_vector::new(100);
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 0) == 0, 0);
    }

    #[test]
    fun longest_sequence_one_set_zero_index() {
        let bitvector = bit_vector::new(100);
        bit_vector::set(&mut bitvector, 1);
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 0) == 0, 0);
    }

    #[test]
    fun longest_sequence_no_set_nonzero_index() {
        let bitvector = bit_vector::new(100);
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 51) == 0, 0);
    }

    #[test]
    fun longest_sequence_two_set_nonzero_index() {
        let bitvector = bit_vector::new(100);
        bit_vector::set(&mut bitvector, 50);
        bit_vector::set(&mut bitvector, 52);
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 51) == 0, 0);
    }

    #[test]
    fun longest_sequence_with_break() {
        let bitvector = bit_vector::new(100);
        let i = 0;
        while (i < 20) {
            bit_vector::set(&mut bitvector, i);
            i = i + 1;
        };
        // create a break in the run
        i = i + 1;
        while (i < 100) {
            bit_vector::set(&mut bitvector, i);
            i = i + 1;
        };
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 0) == 20, 0);
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 20) == 0, 0);
        assert!(bit_vector::longest_set_sequence_starting_at(&bitvector, 21) == 100 - 21, 0);
    }

    #[test]
    fun test_shift_left() {
        let bitlen = 133;
        let bitvector = bit_vector::new(bitlen);

        let i = 0;
        while (i < bitlen) {
            bit_vector::set(&mut bitvector, i);
            i = i + 1;
        };

        i = bitlen - 1;
        while (i > 0) {
            assert!(bit_vector::is_index_set(&bitvector, i), 0);
            bit_vector::shift_left(&mut bitvector, 1);
            assert!(!bit_vector::is_index_set(&bitvector,  i), 1);
            i = i - 1;
        };
    }

    #[test]
    fun test_shift_left_specific_amount() {
        let bitlen = 300;
        let shift_amount = 133;
        let bitvector = bit_vector::new(bitlen);

        bit_vector::set(&mut bitvector, 201);
        assert!(bit_vector::is_index_set(&bitvector, 201), 0);

        bit_vector::shift_left(&mut bitvector, shift_amount);
        assert!(bit_vector::is_index_set(&bitvector, 201 - shift_amount), 1);
        assert!(!bit_vector::is_index_set(&bitvector, 201), 2);

        // Make sure this shift clears all the bits
        bit_vector::shift_left(&mut bitvector, bitlen  - 1);

        let i = 0;
        while (i < bitlen) {
            assert!(!bit_vector::is_index_set(&bitvector, i), 3);
            i = i + 1;
        }
    }

    #[test]
    fun test_shift_left_specific_amount_to_unset_bit() {
        let bitlen = 50;
        let chosen_index = 24;
        let shift_amount = 3;
        let bitvector = bit_vector::new(bitlen);

        let i = 0;

        while (i < bitlen) {
            bit_vector::set(&mut bitvector, i);
            i = i + 1;
        };

        bit_vector::unset(&mut bitvector, chosen_index);
        assert!(!bit_vector::is_index_set(&bitvector, chosen_index), 0);

        bit_vector::shift_left(&mut bitvector, shift_amount);

        i = 0;

        while (i < bitlen) {
            // only chosen_index - shift_amount and the remaining bits should be BitVector::unset
            if ((i == chosen_index - shift_amount) || (i >= bitlen - shift_amount)) {
                assert!(!bit_vector::is_index_set(&bitvector, i), 1);
            } else {
                assert!(bit_vector::is_index_set(&bitvector, i), 2);
            };
            i = i + 1;
        }
    }

    #[test]
    fun shift_left_at_size() {
        let bitlen = 133;
        let bitvector = bit_vector::new(bitlen);

        let i = 0;
        while (i < bitlen) {
            bit_vector::set(&mut bitvector, i);
            i = i + 1;
        };

        bit_vector::shift_left(&mut bitvector, bitlen - 1);
        i = bitlen - 1;
        while (i > 0) {
            assert!(!bit_vector::is_index_set(&bitvector,  i), 1);
            i = i - 1;
        };
    }

    #[test]
    fun shift_left_more_than_size() {
        let bitlen = 133;
        let bitvector = bit_vector::new(bitlen);
        bit_vector::shift_left(&mut bitvector, bitlen);
    }

    #[test]
    #[expected_failure(abort_code = bit_vector::ELENGTH)]
    fun empty_bitvector() {
        bit_vector::new(0);
    }

    #[test]
    fun single_bit_bitvector() {
        let bitvector = bit_vector::new(1);
        assert!(bit_vector::length(&bitvector) == 1, 0);
    }
}
