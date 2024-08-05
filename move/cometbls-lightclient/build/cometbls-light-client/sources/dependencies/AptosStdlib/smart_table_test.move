#[test_only]
module aptos_std::smart_table_test {
    use aptos_std::smart_table::{Self, SmartTable};

    #[test_only]
    public fun make_smart_table(): SmartTable<u64, u64> {
        let table = smart_table::new_with_config<u64, u64>(0, 50, 10);
        let i = 0u64;
        while (i < 100) {
            smart_table::add(&mut table, i, i);
            i = i + 1;
        };
        table
    }

    #[test]
    public fun smart_table_for_each_ref_test() {
        let t = make_smart_table();
        let s = 0;
        smart_table::for_each_ref(&t, |x, y| {
            s = s + *x + *y;
        });
        assert!(s == 9900, 0);
        smart_table::destroy(t);
    }

    #[test]
    public fun smart_table_for_each_mut_test() {
        let t = make_smart_table();
        smart_table::for_each_mut(&mut t, |_key, val| {
            let val: &mut u64 = val;
            *val = *val + 1
        });
        smart_table::for_each_ref(&t, |key, val| {
            assert!(*key + 1 == *val, *key);
        });
        smart_table::destroy(t);
    }

    #[test]
    public fun smart_table_test_map_ref_test() {
        let t = make_smart_table();
        let r = smart_table::map_ref(&t, |val| *val + 1);
        smart_table::for_each_ref(&r, |key, val| {
            assert!(*key + 1 == *val, *key);
        });
        smart_table::destroy(t);
        smart_table::destroy(r);
    }

    #[test]
    public fun smart_table_any_test() {
        let t = make_smart_table();
        let r = smart_table::any(&t, |_k, v| *v >= 99);
        assert!(r, 0);
        smart_table::destroy(t);
    }
}
