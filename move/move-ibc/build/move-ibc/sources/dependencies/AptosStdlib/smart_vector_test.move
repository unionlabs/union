#[test_only]
module aptos_std::smart_vector_test {
    use aptos_std::smart_vector as V;
    use aptos_std::smart_vector::SmartVector;

    #[test_only]
    fun make_smart_vector(k: u64): SmartVector<u64> {
        let v = V::new<u64>();
        let i = 1u64;
        while (i <= k) {
            V::push_back(&mut v, i);
            i = i + 1;
        };
        v
    }

    #[test]
    fun smart_vector_for_each_test() {
        let v = make_smart_vector(100);
        let i = 0;
        V::for_each(v, |x| {
            assert!(i + 1 == x, 0);
            i = i + 1;
        });
    }

    #[test]
    fun smart_vector_for_each_reverse_test() {
        let v = make_smart_vector(100);
        let i = 0;
        V::for_each_reverse(v, |x| {
            assert!(i == 100 - x, 0);
            i = i + 1;
        });
    }

    #[test]
    fun smart_vector_for_each_ref_test() {
        let v = make_smart_vector(100);
        let s = 0;
        V::for_each_ref(&v, |x| {
            s = s + *x;
        });
        assert!(s == 5050, 0);
        V::destroy(v);
    }

    #[test]
    fun smart_vector_for_each_mut_test() {
        let v = make_smart_vector(100);
        V::for_each_mut(&mut v, |x| {
            let x: &mut u64 = x;
            *x = *x + 1;
        });
        assert!(V::fold(v, 0, |s, x| {
            s + x
        }) == 5150, 0);
    }

    #[test]
    fun smart_vector_enumerate_ref_test() {
        let v = make_smart_vector(100);
        V::enumerate_ref(&v, |i, x| {
            assert!(i + 1 == *x, 0);
        });
        V::destroy(v);
    }

    #[test]
    fun smart_vector_enumerate_mut_test() {
        let v = make_smart_vector(100);
        V::enumerate_mut(&mut v, |i, x| {
            let x: &mut u64 = x;
            assert!(i + 1 == *x, 0);
            *x = *x + 1;
        });
        assert!(V::fold(v, 0, |s, x| {
            s + x
        }) == 5150, 0);
    }

    #[test]
    fun smart_vector_fold_test() {
        let v = make_smart_vector(100);
        let i = 0;
        let sum = V::fold(v, 0, |s, x| {
            assert!(i + 1 == x, 0);
            i = i + 1;
            s + x
        });
        assert!(sum == 5050, 0);
    }

    #[test]
    fun smart_vector_for_foldr_test() {
        let v = make_smart_vector(100);
        let i = 0;
        let sum = V::foldr(v, 0, |x, s| {
            assert!(i == 100 - x, i);
            i = i + 1;
            s + x
        });
        assert!(sum == 5050, 0);
    }

    #[test]
    fun smart_vector_map_test() {
        let v = make_smart_vector(100);
        let mapped_v = V::map(v, |x| { x * 2 });
        let sum = V::fold(mapped_v, 0, |s, x| {
            s + x
        });
        assert!(sum == 10100, 0);
    }

    #[test]
    fun smart_vector_map_ref_test() {
        let v = make_smart_vector(100);
        let mapped_v = V::map_ref(&v, |x|  *x * 2);
        assert!(V::fold(v, 0, |s, x| {
            s + x
        }) == 5050, 0);
        assert!(V::fold(mapped_v, 0, |s, x| {
            s + x
        }) == 10100, 0);
    }

    #[test]
    fun smart_vector_filter_test() {
        let v = make_smart_vector(100);
        let filtered_v = V::filter(v, |x| *x % 10 == 0);
        V::enumerate_ref(&filtered_v, |i, x| {
            assert!((i + 1) * 10 == *x, 0);
        });
        V::destroy(filtered_v);
    }

    #[test]
    fun smart_vector_test_zip() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(100);
        let s = 0;
        V::zip(v1, v2, |e1, e2| {
            let e1: u64 = e1;
            let e2: u64 = e2;
            s = s + e1 / e2
        });
        assert!(s == 100, 0);
    }

    #[test]
    // zip is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::ESMART_VECTORS_LENGTH_MISMATCH, location = Self)]
    fun smart_vector_test_zip_mismatching_lengths_should_fail() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(99);
        let s = 0;
        V::zip(v1, v2, |e1, e2| {
            let e1: u64 = e1;
            let e2: u64 = e2;
            s = s + e1 / e2
        });
    }

    #[test]
    fun smart_vector_test_zip_ref() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(100);
        let s = 0;
        V::zip_ref(&v1, &v2, |e1, e2| s = s + *e1 / *e2);
        assert!(s == 100, 0);
        V::destroy(v1);
        V::destroy(v2);
    }

    #[test]
    // zip_ref is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::ESMART_VECTORS_LENGTH_MISMATCH, location = Self)]
    fun smart_vector_test_zip_ref_mismatching_lengths_should_fail() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(99);
        let s = 0;
        V::zip_ref(&v1, &v2, |e1, e2| s = s + *e1 / *e2);
        V::destroy(v1);
        V::destroy(v2);
    }

    #[test]
    fun smart_vector_test_zip_mut() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(100);
        V::zip_mut(&mut v1, &mut v2, |e1, e2| {
            let e1: &mut u64 = e1;
            let e2: &mut u64 = e2;
            *e1 = *e1 + 1;
            *e2 = *e2 - 1;
        });
        V::zip_ref(&v1, &v2, |e1, e2| assert!(*e1 == *e2 + 2, 0));
        V::destroy(v1);
        V::destroy(v2);
    }

    #[test]
    fun smart_vector_test_zip_map() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(100);
        let result = V::zip_map(v1, v2, |e1, e2| e1 / e2);
        V::for_each(result, |v| assert!(v == 1, 0));
    }

    #[test]
    fun smart_vector_test_zip_map_ref() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(100);
        let result = V::zip_map_ref(&v1, &v2, |e1, e2| *e1 / *e2);
        V::for_each(result, |v| assert!(v == 1, 0));
        V::destroy(v1);
        V::destroy(v2);
    }

    #[test]
    // zip_mut is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::ESMART_VECTORS_LENGTH_MISMATCH, location = Self)]
    fun smart_vector_test_zip_mut_mismatching_lengths_should_fail() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(99);
        let s = 0;
        V::zip_mut(&mut v1, &mut v2, |e1, e2| s = s + *e1 / *e2);
        V::destroy(v1);
        V::destroy(v2);
    }

    #[test]
    // zip_map is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::ESMART_VECTORS_LENGTH_MISMATCH, location = Self)]
    fun smart_vector_test_zip_map_mismatching_lengths_should_fail() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(99);
        V::destroy(V::zip_map(v1, v2, |e1, e2| e1 / e2));
    }

    #[test]
    // zip_map_ref is an inline function so any error code will be reported at the call site.
    #[expected_failure(abort_code = V::ESMART_VECTORS_LENGTH_MISMATCH, location = Self)]
    fun smart_vector_test_zip_map_ref_mismatching_lengths_should_fail() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(99);
        V::destroy(V::zip_map_ref(&v1, &v2, |e1, e2| *e1 / *e2));
        V::destroy(v1);
        V::destroy(v2);
    }
}
