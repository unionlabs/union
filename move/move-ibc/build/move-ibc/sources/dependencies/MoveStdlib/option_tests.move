#[test_only]
module std::option_tests {
    use std::option;
    use std::vector;

    #[test]
    fun option_none_is_none() {
        let none = option::none<u64>();
        assert!(option::is_none(&none), 0);
        assert!(!option::is_some(&none), 1);
    }

    #[test]
    fun option_some_is_some() {
        let some = option::some(5);
        assert!(!option::is_none(&some), 0);
        assert!(option::is_some(&some), 1);
    }

    #[test]
    fun option_contains() {
        let none = option::none<u64>();
        let some = option::some(5);
        let some_other = option::some(6);
        assert!(option::contains(&some, &5), 0);
        assert!(option::contains(&some_other, &6), 1);
        assert!(!option::contains(&none, &5), 2);
        assert!(!option::contains(&some_other, &5), 3);
    }

    #[test]
    fun option_borrow_some() {
        let some = option::some(5);
        let some_other = option::some(6);
        assert!(*option::borrow(&some) == 5, 3);
        assert!(*option::borrow(&some_other) == 6, 4);
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_NOT_SET)]
    fun option_borrow_none() {
        option::borrow(&option::none<u64>());
    }

    #[test]
    fun borrow_mut_some() {
        let some = option::some(1);
        let ref = option::borrow_mut(&mut some);
        *ref = 10;
        assert!(*option::borrow(&some) == 10, 0);
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_NOT_SET)]
    fun borrow_mut_none() {
        option::borrow_mut(&mut option::none<u64>());
    }

    #[test]
    fun borrow_with_default() {
        let none = option::none<u64>();
        let some = option::some(5);
        assert!(*option::borrow_with_default(&some, &7) == 5, 0);
        assert!(*option::borrow_with_default(&none, &7) == 7, 1);
    }

    #[test]
    fun get_with_default() {
        let none = option::none<u64>();
        let some = option::some(5);
        assert!(option::get_with_default(&some, 7) == 5, 0);
        assert!(option::get_with_default(&none, 7) == 7, 1);
    }

    #[test]
    fun extract_some() {
        let opt = option::some(1);
        assert!(option::extract(&mut opt) == 1, 0);
        assert!(option::is_none(&opt), 1);
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_NOT_SET)]
    fun extract_none() {
        option::extract(&mut option::none<u64>());
    }

    #[test]
    fun swap_some() {
        let some = option::some(5);
        assert!(option::swap(&mut some, 1) == 5, 0);
        assert!(*option::borrow(&some) == 1, 1);
    }

    #[test]
    fun swap_or_fill_some() {
        let some = option::some(5);
        assert!(option::swap_or_fill(&mut some, 1) == option::some(5), 0);
        assert!(*option::borrow(&some) == 1, 1);
    }

    #[test]
    fun swap_or_fill_none() {
        let none = option::none();
        assert!(option::swap_or_fill(&mut none, 1) == option::none(), 0);
        assert!(*option::borrow(&none) == 1, 1);
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_NOT_SET)]
    fun swap_none() {
        option::swap(&mut option::none<u64>(), 1);
    }

    #[test]
    fun fill_none() {
        let none = option::none<u64>();
        option::fill(&mut none, 3);
        assert!(option::is_some(&none), 0);
        assert!(*option::borrow(&none) == 3, 1);
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_IS_SET)]
    fun fill_some() {
        option::fill(&mut option::some(3), 0);
    }

    #[test]
    fun destroy_with_default() {
        assert!(option::destroy_with_default(option::none<u64>(), 4) == 4, 0);
        assert!(option::destroy_with_default(option::some(4), 5) == 4, 1);
    }

    #[test]
    fun destroy_some() {
        assert!(option::destroy_some(option::some(4)) == 4, 0);
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_NOT_SET)]
    fun destroy_some_none() {
        option::destroy_some(option::none<u64>());
    }

    #[test]
    fun destroy_none() {
        option::destroy_none(option::none<u64>());
    }

    #[test]
    #[expected_failure(abort_code = option::EOPTION_IS_SET)]
    fun destroy_none_some() {
        option::destroy_none(option::some<u64>(0));
    }

    #[test]
    fun into_vec_some() {
        let v = option::to_vec(option::some<u64>(0));
        assert!(vector::length(&v) == 1, 0);
        let x = vector::pop_back(&mut v);
        assert!(x == 0, 1);
    }

    #[test]
    fun into_vec_none() {
        let v: vector<u64> = option::to_vec(option::none());
        assert!(vector::is_empty(&v), 0);
    }

    #[test]
    fun test_for_each() {
        let r = 0;
        option::for_each(option::some(1), |x| r = x);
        assert!(r == 1, 0);
        r = 0;
        option::for_each(option::none<u64>(), |x| r = x);
        assert!(r == 0, 1);
    }

    #[test]
    fun test_for_each_ref() {
        let r = 0;
        option::for_each_ref(&option::some(1), |x| r = *x);
        assert!(r == 1, 0);
        r = 0;
        option::for_each_ref(&option::none<u64>(), |x| r = *x);
        assert!(r == 0, 1);
    }

    #[test]
    fun test_for_each_mut() {
        let o = option::some(0);
        option::for_each_mut(&mut o, |x| *x = 1);
        assert!(o == option::some(1), 0);
    }

    #[test]
    fun test_fold() {
        let r = option::fold(option::some(1), 1, |a, b| a + b);
        assert!(r == 2, 0);
        let r = option::fold(option::none<u64>(), 1, |a, b| a + b);
        assert!(r == 1, 0);
    }

    #[test]
    fun test_map() {
        let x = option::map(option::some(1), |e| e + 1);
        assert!(option::extract(&mut x) == 2, 0);
    }

    #[test]
    fun test_map_ref() {
        let x = option::map_ref(&option::some(1), |e| *e + 1);
        assert!(option::extract(&mut x) == 2, 0);
    }

    #[test]
    fun test_filter() {
        let x = option::filter(option::some(1), |e| *e != 1);
        assert!(option::is_none(&x), 0);
    }

    #[test]
    fun test_any() {
        let r = option::any(&option::some(1), |e| *e == 1);
        assert!(r, 0);
    }


}
