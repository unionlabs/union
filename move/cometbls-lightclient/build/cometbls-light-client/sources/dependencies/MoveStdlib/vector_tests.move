#[test_only]
module std::vector_tests {
    use std::vector as V;
    use std::vector;

    struct R has store { }
    struct Droppable has drop {}
    struct NotDroppable {}

    #[test]
    fun test_singleton_contains() {
        assert!(*V::borrow(&V::singleton(0), 0) == 0, 0);
        assert!(*V::borrow(&V::singleton(true), 0) == true, 0);
        assert!(*V::borrow(&V::singleton(@0x1), 0) == @0x1, 0);
    }

    #[test]
    fun test_singleton_len() {
        assert!(V::length(&V::singleton(0)) == 1, 0);
        assert!(V::length(&V::singleton(true)) == 1, 0);
        assert!(V::length(&V::singleton(@0x1)) == 1, 0);
    }

    #[test]
    fun test_empty_is_empty() {
        assert!(V::is_empty(&V::empty<u64>()), 0);
    }

    #[test]
    fun append_empties_is_empty() {
        let v1 = V::empty<u64>();
        let v2 = V::empty<u64>();
        V::append(&mut v1, v2);
        assert!(V::is_empty(&v1), 0);
    }

    #[test]
    fun append_respects_order_empty_lhs() {
        let v1 = V::empty();
        let v2 = V::empty();
        V::push_back(&mut v2, 0);
        V::push_back(&mut v2, 1);
        V::push_back(&mut v2, 2);
        V::push_back(&mut v2, 3);
        V::append(&mut v1, v2);
        assert!(!V::is_empty(&v1), 0);
        assert!(V::length(&v1) == 4, 1);
        assert!(*V::borrow(&v1, 0) == 0, 2);
        assert!(*V::borrow(&v1, 1) == 1, 3);
        assert!(*V::borrow(&v1, 2) == 2, 4);
        assert!(*V::borrow(&v1, 3) == 3, 5);
    }

    #[test]
    fun append_respects_order_empty_rhs() {
        let v1 = V::empty();
        let v2 = V::empty();
        V::push_back(&mut v1, 0);
        V::push_back(&mut v1, 1);
        V::push_back(&mut v1, 2);
        V::push_back(&mut v1, 3);
        V::append(&mut v1, v2);
        assert!(!V::is_empty(&v1), 0);
        assert!(V::length(&v1) == 4, 1);
        assert!(*V::borrow(&v1, 0) == 0, 2);
        assert!(*V::borrow(&v1, 1) == 1, 3);
        assert!(*V::borrow(&v1, 2) == 2, 4);
        assert!(*V::borrow(&v1, 3) == 3, 5);
    }

    #[test]
    fun append_respects_order_nonempty_rhs_lhs() {
        let v1 = V::empty();
        let v2 = V::empty();
        V::push_back(&mut v1, 0);
        V::push_back(&mut v1, 1);
        V::push_back(&mut v1, 2);
        V::push_back(&mut v1, 3);
        V::push_back(&mut v2, 4);
        V::push_back(&mut v2, 5);
        V::push_back(&mut v2, 6);
        V::push_back(&mut v2, 7);
        V::append(&mut v1, v2);
        assert!(!V::is_empty(&v1), 0);
        assert!(V::length(&v1) == 8, 1);
        let i = 0;
        while (i < 8) {
            assert!(*V::borrow(&v1, i) == i, i);
            i = i + 1;
        }
    }

    #[test]
    fun test_trim() {
        {
            let v = V::empty<u64>();
            assert!(&V::trim(&mut v, 0) == &vector[], 0);
        };
        {
            let v = vector[1];
            assert!(&V::trim(&mut v, 1) == &vector[], 1);
            assert!(&V::trim(&mut v, 0) == &vector[1], 2);
        };
        {
            let v = vector[1, 2];
            assert!(&V::trim(&mut v, 0) == &vector[1, 2], 3);
        };
    }
    #[test]
    #[expected_failure(abort_code = V::EINDEX_OUT_OF_BOUNDS)]
    fun test_trim_fail() {
        let v = vector[1];
        V::trim(&mut v, 2);
    }

    #[test]
    #[expected_failure(vector_error, minor_status = 1, location = Self)]
    fun borrow_out_of_range() {
        let v = V::empty();
        V::push_back(&mut v, 7);
        V::borrow(&v, 1);
    }

    #[test]
    fun vector_contains() {
        let vec = V::empty();
        assert!(!V::contains(&vec, &0), 1);

        V::push_back(&mut vec, 0);
        assert!(V::contains(&vec, &0), 2);
        assert!(!V::contains(&vec, &1), 3);

        V::push_back(&mut vec, 1);
        assert!(V::contains(&vec, &0), 4);
        assert!(V::contains(&vec, &1), 5);
        assert!(!V::contains(&vec, &2), 6);

        V::push_back(&mut vec, 2);
        assert!(V::contains(&vec, &0), 7);
        assert!(V::contains(&vec, &1), 8);
        assert!(V::contains(&vec, &2), 9);
        assert!(!V::contains(&vec, &3), 10);
    }

    #[test]
    fun destroy_empty() {
        V::destroy_empty(V::empty<u64>());
        V::destroy_empty(V::empty<R>());
    }

    #[test]
    fun destroy_empty_with_pops() {
        let v = V::empty();
        V::push_back(&mut v, 42);
        V::pop_back(&mut v);
        V::destroy_empty(v);
    }

    #[test]
    #[expected_failure(vector_error, minor_status = 3, location = Self)]
    fun destroy_non_empty() {
        let v = V::empty();
        V::push_back(&mut v, 42);
        V::destroy_empty(v);
    }

    #[test]
    fun get_set_work() {
        let vec = V::empty();
        V::push_back(&mut vec, 0);
        V::push_back(&mut vec, 1);
        assert!(*V::borrow(&vec, 1) == 1, 0);
        assert!(*V::borrow(&vec, 0) == 0, 1);

        *V::borrow_mut(&mut vec, 0) = 17;
        assert!(*V::borrow(&vec, 1) == 1, 0);
        assert!(*V::borrow(&vec, 0) == 17, 0);
    }

    #[test]
    #[expected_failure(vector_error, minor_status = 2, location = Self)]
    fun pop_out_of_range() {
        let v = V::empty<u64>();
        V::pop_back(&mut v);
    }

    #[test]
    fun swap_different_indices() {
        let vec = V::empty();
        V::push_back(&mut vec, 0);
        V::push_back(&mut vec, 1);
        V::push_back(&mut vec, 2);
        V::push_back(&mut vec, 3);
        V::swap(&mut vec, 0, 3);
        V::swap(&mut vec, 1, 2);
        assert!(*V::borrow(&vec, 0) == 3, 0);
        assert!(*V::borrow(&vec, 1) == 2, 0);
        assert!(*V::borrow(&vec, 2) == 1, 0);
        assert!(*V::borrow(&vec, 3) == 0, 0);
    }

    #[test]
    fun swap_same_index() {
        let vec = V::empty();
        V::push_back(&mut vec, 0);
        V::push_back(&mut vec, 1);
        V::push_back(&mut vec, 2);
        V::push_back(&mut vec, 3);
        V::swap(&mut vec, 1, 1);
        assert!(*V::borrow(&vec, 0) == 0, 0);
        assert!(*V::borrow(&vec, 1) == 1, 0);
        assert!(*V::borrow(&vec, 2) == 2, 0);
        assert!(*V::borrow(&vec, 3) == 3, 0);
    }

    #[test]
    fun remove_singleton_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        assert!(V::remove(&mut v, 0) == 0, 0);
        assert!(V::length(&v) == 0, 0);
    }

    #[test]
    fun remove_nonsingleton_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(V::remove(&mut v, 1) == 1, 0);
        assert!(V::length(&v) == 3, 0);
        assert!(*V::borrow(&v, 0) == 0, 0);
        assert!(*V::borrow(&v, 1) == 2, 0);
        assert!(*V::borrow(&v, 2) == 3, 0);
    }

    #[test]
    fun remove_nonsingleton_vector_last_elem() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(V::remove(&mut v, 3) == 3, 0);
        assert!(V::length(&v) == 3, 0);
        assert!(*V::borrow(&v, 0) == 0, 0);
        assert!(*V::borrow(&v, 1) == 1, 0);
        assert!(*V::borrow(&v, 2) == 2, 0);
    }

    #[test]
    #[expected_failure(abort_code = V::EINDEX_OUT_OF_BOUNDS)]
    fun remove_empty_vector() {
        let v = V::empty<u64>();
        V::remove(&mut v, 0);
    }

    #[test]
    #[expected_failure(abort_code = V::EINDEX_OUT_OF_BOUNDS)]
    fun remove_out_of_bound_index() {
        let v = V::empty<u64>();
        V::push_back(&mut v, 0);
        V::remove(&mut v, 1);
    }

    #[test]
    fun remove_value_singleton_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        assert!(V::borrow(&V::remove_value(&mut v, &0), 0) == &0, 0);
        assert!(V::length(&v) == 0, 0);
    }

    #[test]
    fun remove_value_nonsingleton_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(V::borrow(&V::remove_value(&mut v, &2), 0) == &2, 0);
        assert!(V::length(&v) == 3, 0);
        assert!(*V::borrow(&v, 0) == 0, 0);
        assert!(*V::borrow(&v, 1) == 1, 0);
        assert!(*V::borrow(&v, 2) == 3, 0);
    }

    #[test]
    fun remove_value_nonsingleton_vector_last_elem() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(V::borrow(&V::remove_value(&mut v, &3), 0) == &3, 0);
        assert!(V::length(&v) == 3, 0);
        assert!(*V::borrow(&v, 0) == 0, 0);
        assert!(*V::borrow(&v, 1) == 1, 0);
        assert!(*V::borrow(&v, 2) == 2, 0);
    }

    #[test]
    fun remove_value_empty_vector() {
        let v = V::empty<u64>();
        assert!(V::length(&V::remove_value(&mut v, &1)) == 0, 0);
        assert!(V::length(&v) == 0, 1);
    }

    #[test]
    fun remove_value_nonexistent() {
        let v = V::empty<u64>();
        V::push_back(&mut v, 0);
        assert!(V::length(&V::remove_value(&mut v, &1)) == 0, 0);
        assert!(V::length(&v) == 1, 1);
    }

    #[test]
    fun reverse_vector_empty() {
        let v = V::empty<u64>();
        let is_empty = V::is_empty(&v);
        V::reverse(&mut v);
        assert!(is_empty == V::is_empty(&v), 0);
    }

    #[test]
    fun reverse_singleton_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        assert!(*V::borrow(&v, 0) == 0, 1);
        V::reverse(&mut v);
        assert!(*V::borrow(&v, 0) == 0, 2);
    }

    #[test]
    fun reverse_vector_nonempty_even_length() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(*V::borrow(&v, 0) == 0, 1);
        assert!(*V::borrow(&v, 1) == 1, 2);
        assert!(*V::borrow(&v, 2) == 2, 3);
        assert!(*V::borrow(&v, 3) == 3, 4);

        V::reverse(&mut v);

        assert!(*V::borrow(&v, 3) == 0, 5);
        assert!(*V::borrow(&v, 2) == 1, 6);
        assert!(*V::borrow(&v, 1) == 2, 7);
        assert!(*V::borrow(&v, 0) == 3, 8);
    }

    #[test]
    fun reverse_vector_nonempty_odd_length_non_singleton() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);

        assert!(*V::borrow(&v, 0) == 0, 1);
        assert!(*V::borrow(&v, 1) == 1, 2);
        assert!(*V::borrow(&v, 2) == 2, 3);

        V::reverse(&mut v);

        assert!(*V::borrow(&v, 2) == 0, 4);
        assert!(*V::borrow(&v, 1) == 1, 5);
        assert!(*V::borrow(&v, 0) == 2, 6);
    }

    #[test]
    #[expected_failure(vector_error, minor_status = 1, location = Self)]
    fun swap_empty() {
        let v = V::empty<u64>();
        V::swap(&mut v, 0, 0);
    }

    #[test]
    #[expected_failure(vector_error, minor_status = 1, location = Self)]
    fun swap_out_of_range() {
        let v = V::empty<u64>();

        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        V::swap(&mut v, 1, 10);
    }

    #[test]
    #[expected_failure(abort_code = V::EINDEX_OUT_OF_BOUNDS)]
    fun swap_remove_empty() {
        let v = V::empty<u64>();
        V::swap_remove(&mut v, 0);
    }

    #[test]
    fun swap_remove_singleton() {
        let v = V::empty<u64>();
        V::push_back(&mut v, 0);
        assert!(V::swap_remove(&mut v, 0) == 0, 0);
        assert!(V::is_empty(&v), 1);
    }

    #[test]
    fun swap_remove_inside_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(*V::borrow(&v, 0) == 0, 1);
        assert!(*V::borrow(&v, 1) == 1, 2);
        assert!(*V::borrow(&v, 2) == 2, 3);
        assert!(*V::borrow(&v, 3) == 3, 4);

        assert!(V::swap_remove(&mut v, 1) == 1, 5);
        assert!(V::length(&v) == 3, 6);

        assert!(*V::borrow(&v, 0) == 0, 7);
        assert!(*V::borrow(&v, 1) == 3, 8);
        assert!(*V::borrow(&v, 2) == 2, 9);

    }

    #[test]
    fun swap_remove_end_of_vector() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);

        assert!(*V::borrow(&v, 0) == 0, 1);
        assert!(*V::borrow(&v, 1) == 1, 2);
        assert!(*V::borrow(&v, 2) == 2, 3);
        assert!(*V::borrow(&v, 3) == 3, 4);

        assert!(V::swap_remove(&mut v, 3) == 3, 5);
        assert!(V::length(&v) == 3, 6);

        assert!(*V::borrow(&v, 0) == 0, 7);
        assert!(*V::borrow(&v, 1) == 1, 8);
        assert!(*V::borrow(&v, 2) == 2, 9);
    }

    #[test]
    #[expected_failure(vector_error, minor_status = 1, location = std::vector)]
    fun swap_remove_out_of_range() {
        let v = V::empty();
        V::push_back(&mut v, 0);
        V::swap_remove(&mut v, 1);
    }

    #[test]
    fun push_back_and_borrow() {
        let v = V::empty();
        V::push_back(&mut v, 7);
        assert!(!V::is_empty(&v), 0);
        assert!(V::length(&v) == 1, 1);
        assert!(*V::borrow(&v, 0) == 7, 2);

        V::push_back(&mut v, 8);
        assert!(V::length(&v) == 2, 3);
        assert!(*V::borrow(&v, 0) == 7, 4);
        assert!(*V::borrow(&v, 1) == 8, 5);
    }

    #[test]
    fun index_of_empty_not_has() {
        let v = V::empty();
        let (has, index) = V::index_of(&v, &true);
        assert!(!has, 0);
        assert!(index == 0, 1);
    }

    #[test]
    fun index_of_nonempty_not_has() {
        let v = V::empty();
        V::push_back(&mut v, false);
        let (has, index) = V::index_of(&v, &true);
        assert!(!has, 0);
        assert!(index == 0, 1);
    }

    #[test]
    fun index_of_nonempty_has() {
        let v = V::empty();
        V::push_back(&mut v, false);
        V::push_back(&mut v, true);
        let (has, index) = V::index_of(&v, &true);
        assert!(has, 0);
        assert!(index == 1, 1);
    }

    // index_of will return the index first occurence that is equal
    #[test]
    fun index_of_nonempty_has_multiple_occurences() {
        let v = V::empty();
        V::push_back(&mut v, false);
        V::push_back(&mut v, true);
        V::push_back(&mut v, true);
        let (has, index) = V::index_of(&v, &true);
        assert!(has, 0);
        assert!(index == 1, 1);
    }

    #[test]
    fun find_empty_not_has() {
        let v = V::empty<u64>();
        let (has, index) = V::find(&v, |_x| true);
        assert!(!has, 0);
        assert!(index == 0, 1);
    }

    #[test]
    fun find_nonempty_not_has() {
        let v = V::empty();
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        let (has, index) = V::find(&v, |x| *x == 3);
        assert!(!has, 0);
        assert!(index == 0, 1);
    }

    #[test]
    fun find_nonempty_has() {
        let v = V::empty();
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);
        let (has, index) = V::find(&v, |x| *x == 2);
        assert!(has, 0);
        assert!(index == 1, 1);
    }

    #[test]
    fun find_nonempty_has_multiple_occurences() {
        let v = V::empty();
        V::push_back(&mut v, 1);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 2);
        V::push_back(&mut v, 3);
        let (has, index) = V::find(&v, |x| *x == 2);
        assert!(has, 0);
        assert!(index == 1, 1);
    }

    #[test]
    fun length() {
        let empty = V::empty();
        assert!(V::length(&empty) == 0, 0);
        let i = 0;
        let max_len = 42;
        while (i < max_len) {
            V::push_back(&mut empty, i);
            assert!(V::length(&empty) == i + 1, i);
            i = i + 1;
        }
    }

    #[test]
    fun pop_push_back() {
        let v = V::empty();
        let i = 0;
        let max_len = 42;

        while (i < max_len) {
            V::push_back(&mut v, i);
            i = i + 1;
        };

        while (i > 0) {
            assert!(V::pop_back(&mut v) == i - 1, i);
            i = i - 1;
        };
    }

    #[test_only]
    fun test_natives_with_type<T>(x1: T, x2: T): (T, T) {
        let v = V::empty();
        assert!(V::length(&v) == 0, 0);
        V::push_back(&mut v, x1);
        assert!(V::length(&v) == 1, 1);
        V::push_back(&mut v, x2);
        assert!(V::length(&v) == 2, 2);
        V::swap(&mut v, 0, 1);
        x1 = V::pop_back(&mut v);
        assert!(V::length(&v) == 1, 3);
        x2 = V::pop_back(&mut v);
        assert!(V::length(&v) == 0, 4);
        V::destroy_empty(v);
        (x1, x2)
    }

    #[test]
    fun test_natives_with_different_instantiations() {
        test_natives_with_type<u8>(1u8, 2u8);
        test_natives_with_type<u64>(1u64, 2u64);
        test_natives_with_type<u128>(1u128, 2u128);
        test_natives_with_type<bool>(true, false);
        test_natives_with_type<address>(@0x1, @0x2);

        test_natives_with_type<vector<u8>>(V::empty(), V::empty());

        test_natives_with_type<Droppable>(Droppable{}, Droppable{});
        (NotDroppable {}, NotDroppable {}) = test_natives_with_type<NotDroppable>(
            NotDroppable {},
            NotDroppable {}
        );
    }

    #[test]
    fun test_for_each() {
        let v = vector[1, 2, 3];
        let s = 0;
        V::for_each(v, |e| {
            s = s + e;
        });
        assert!(s == 6, 0)
    }

    #[test]
    fun test_zip() {
        let v1 = vector[1, 2, 3];
        let v2 = vector[10, 20, 30];
        let s = 0;
        V::zip(v1, v2, |e1, e2| s = s + e1 * e2);
        assert!(s == 140, 0);
    }

    #[test]
    // zip is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::EVECTORS_LENGTH_MISMATCH, location = Self)]
    fun test_zip_mismatching_lengths_should_fail() {
        let v1 = vector[1];
        let v2 = vector[10, 20];
        let s = 0;
        V::zip(v1, v2, |e1, e2| s = s + e1 * e2);
    }

    #[test]
    fun test_enumerate_ref() {
        let v = vector[1, 2, 3];
        let i_s = 0;
        let s = 0;
        V::enumerate_ref(&v, |i, e| {
            i_s = i_s + i;
            s = s + *e;
        });
        assert!(i_s == 3, 0);
        assert!(s == 6, 0);
    }

    #[test]
    fun test_for_each_ref() {
        let v = vector[1, 2, 3];
        let s = 0;
        V::for_each_ref(&v, |e| s = s + *e);
        assert!(s == 6, 0)
    }

    #[test]
    fun test_for_each_mut() {
        let v = vector[1, 2, 3];
        let s = 2;
        V::for_each_mut(&mut v, |e| { *e = s; s = s + 1 });
        assert!(v == vector[2, 3, 4], 0)
    }

    #[test]
    fun test_zip_ref() {
        let v1 = vector[1, 2, 3];
        let v2 = vector[10, 20, 30];
        let s = 0;
        V::zip_ref(&v1, &v2, |e1, e2| s = s + *e1 * *e2);
        assert!(s == 140, 0);
    }

    #[test]
    // zip_ref is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::EVECTORS_LENGTH_MISMATCH, location = Self)]
    fun test_zip_ref_mismatching_lengths_should_fail() {
        let v1 = vector[1];
        let v2 = vector[10, 20];
        let s = 0;
        V::zip_ref(&v1, &v2, |e1, e2| s = s + *e1 * *e2);
    }

    #[test]
    fun test_zip_mut() {
        let v1 = vector[1, 2, 3];
        let v2 = vector[10, 20, 30];
        V::zip_mut(&mut v1, &mut v2, |e1, e2| {
            let e1: &mut u64 = e1;
            let e2: &mut u64 = e2;
            *e1 = *e1 + 1;
            *e2 = *e2 + 10;
        });
        assert!(v1 == vector[2, 3, 4], 0);
        assert!(v2 == vector[20, 30, 40], 0);
    }

    #[test]
    fun test_zip_map() {
        let v1 = vector[1, 2, 3];
        let v2 = vector[10, 20, 30];
        let result = V::zip_map(v1, v2, |e1, e2| e1 + e2);
        assert!(result == vector[11, 22, 33], 0);
    }

    #[test]
    fun test_zip_map_ref() {
        let v1 = vector[1, 2, 3];
        let v2 = vector[10, 20, 30];
        let result = V::zip_map_ref(&v1, &v2, |e1, e2| *e1 + *e2);
        assert!(result == vector[11, 22, 33], 0);
    }

    #[test]
    // zip_mut is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::EVECTORS_LENGTH_MISMATCH, location = Self)]
    fun test_zip_mut_mismatching_lengths_should_fail() {
        let v1 = vector[1];
        let v2 = vector[10, 20];
        let s = 0;
        V::zip_mut(&mut v1, &mut v2, |e1, e2| s = s + *e1 * *e2);
    }

    #[test]
    // zip_map is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::EVECTORS_LENGTH_MISMATCH, location = Self)]
    fun test_zip_map_mismatching_lengths_should_fail() {
        let v1 = vector[1];
        let v2 = vector[10, 20];
        V::zip_map(v1, v2, |e1, e2| e1 * e2);
    }

    #[test]
    // zip_map_ref is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::EVECTORS_LENGTH_MISMATCH, location = Self)]
    fun test_zip_map_ref_mismatching_lengths_should_fail() {
        let v1 = vector[1];
        let v2 = vector[10, 20];
        V::zip_map_ref(&v1, &v2, |e1, e2| *e1 * *e2);
    }

    #[test]
    fun test_enumerate_mut() {
        let v = vector[1, 2, 3];
        let i_s = 0;
        let s = 2;
        V::enumerate_mut(&mut v, |i, e| {
            i_s = i_s + i;
            *e = s;
            s = s + 1
        });
        assert!(i_s == 3, 0);
        assert!(v == vector[2, 3, 4], 0);
    }

    #[test]
    fun test_fold() {
        let v = vector[1, 2, 3];
        let s = V::fold(v, 0, |r, e| r + e);
        assert!(s == 6 , 0)
    }

    #[test]
    fun test_foldr() {
        // use non-commutative minus operation to test the difference between fold and foldr
        {
            let v = vector[3, 2, 1];
            // ((100 - 3) - 2) - 1 = 94
            let s = V::fold(v, 100, |l, r| l - r);
            assert!(s == 94, 0)
        };
        {
            let v = vector[3, 2, 1];
            // 3 - (2 - (1 - 0)) = 2
            let s = V::foldr(v, 0, |l, r| l - r);
            assert!(s == 2, 1)
        }
    }

    #[test]
    fun test_map() {
        let v = vector[1, 2, 3];
        let s = V::map(v, |x| x + 1);
        assert!(s == vector[2, 3, 4] , 0)
    }

    #[test]
    fun test_map_ref() {
        let v = vector[1, 2, 3];
        let s = V::map_ref(&v, |x| *x + 1);
        assert!(s == vector[2, 3, 4] , 0)
    }

    #[test]
    fun test_filter() {
        let v = vector[1, 2, 3];
        let s = V::filter(v, |x| *x % 2 == 0);
        assert!(s == vector[2] , 0)
    }

    #[test]
    fun test_any() {
        let v = vector[1, 2, 3];
        let r = V::any(&v, |x| *x > 2);
        assert!(r, 0)
    }

    #[test]
    fun test_all() {
        let v = vector[1, 2, 3];
        let r = V::all(&v, |x| *x >= 1);
        assert!(r, 0)
    }

    #[test]
    fun test_rotate() {
        let v = vector[1, 2, 3, 4, 5];
        assert!(vector::rotate(&mut v, 2) == 3, 0);
        assert!(&v == &vector[3, 4, 5, 1, 2], 1);

        assert!(vector::rotate_slice(&mut v, 1, 2, 5) == 4, 2);
        assert!(&v == &vector[3, 5, 1, 2, 4], 3);

        assert!(vector::rotate_slice(&mut v, 0, 0, 5) == 5, 2);
        assert!(&v == &vector[3, 5, 1, 2, 4], 3);
        assert!(vector::rotate_slice(&mut v, 0, 5, 5) == 0, 2);
        assert!(&v == &vector[3, 5, 1, 2, 4], 3);
    }

    #[test]
    fun test_partition() {
        let v = vector[1, 2, 3, 4, 5];
        assert!(vector::partition(&mut v, |n| *n % 2 == 0) == 2, 0);
        assert!(&v == &vector[2, 4, 3, 1, 5], 1);

        assert!(vector::partition(&mut v, |_n| false) == 0, 0);
        assert!(&v == &vector[2, 4, 3, 1, 5], 1);

        assert!(vector::partition(&mut v, |_n| true) == 5, 0);
        assert!(&v == &vector[2, 4, 3, 1, 5], 1);
    }

    #[test]
    fun test_stable_partition() {
        let v:vector<u64> = vector[1, 2, 3, 4, 5];

        assert!(vector::stable_partition(&mut v, |n| *n % 2 == 0) == 2, 0);
        assert!(&v == &vector[2, 4, 1, 3, 5], 1);

        assert!(vector::partition(&mut v, |_n| false) == 0, 0);
        assert!(&v == &vector[2, 4, 1, 3, 5], 1);

        assert!(vector::partition(&mut v, |_n| true) == 5, 0);
        assert!(&v == &vector[2, 4, 1, 3, 5], 1);
    }

    #[test]
    fun test_insert() {
        let v:vector<u64> = vector[1, 2, 3, 4, 5];

        vector::insert(&mut v,2, 6);
        assert!(&v == &vector[1, 2, 6, 3, 4, 5], 1);

        vector::insert(&mut v,6, 7);
        assert!(&v == &vector[1, 2, 6, 3, 4, 5, 7], 1);
    }

    #[test]
    #[expected_failure(abort_code = V::EINDEX_OUT_OF_BOUNDS)]
    fun test_insert_out_of_bounds() {
        let v:vector<u64> = vector[1, 2, 3, 4, 5];

        vector::insert(&mut v,6, 6);
    }

    #[test]
    fun test_range() {
        let result = vector::range(5, 10);
        assert!(result == vector[5, 6, 7, 8, 9], 1);
    }

    #[test]
    fun test_range_with_step() {
        let result = vector::range_with_step(0, 10, 2);
        assert!(result == vector[0, 2, 4, 6, 8], 1);

        let empty_result = vector::range_with_step(10, 10, 2);
        assert!(empty_result == vector[], 1);

        // Test with `start` greater than `end`
        let reverse_result = vector::range_with_step(10, 0, 2);
        assert!(reverse_result == vector[], 1);
    }

    #[test]
    #[expected_failure(abort_code = V::EINVALID_STEP)]
    fun test_range_with_invalid_step() {
        vector::range_with_step(0, 10, 0);
    }

    #[test]
    fun test_slice() {
        let v = &vector[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let slice_beginning = vector::slice(v, 0, 3);
        assert!(slice_beginning == vector[0, 1, 2], 1);

        let slice_end = vector::slice(v, 7, 10);
        assert!(slice_end == vector[7, 8, 9], 1);

        let empty_slice = vector::slice(v, 5, 5);
        assert!(empty_slice == vector[], 1);
        let empty_slice = vector::slice(v, 0, 0);
        assert!(empty_slice == vector[], 1);

        let full_slice = &vector::slice(v, 0, 10);
        assert!(full_slice == v, 1);
    }

    #[test]
    #[expected_failure(abort_code = V::EINVALID_SLICE_RANGE)]
    fun test_slice_invalid_range() {
        let v = &vector[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        vector::slice(v, 7, 6); // start is greater than end
    }

    #[test]
    #[expected_failure(abort_code = V::EINVALID_SLICE_RANGE)]
    fun test_slice_out_of_bounds() {
        let v = &vector[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        vector::slice(v, 0, 11); // end is out of bounds
    }

    #[test_only]
    struct MoveOnly {}

    #[test]
    fun test_destroy() {
        let v = vector[MoveOnly {}];
        vector::destroy(v, |m| { let MoveOnly {} = m; })
    }
}
