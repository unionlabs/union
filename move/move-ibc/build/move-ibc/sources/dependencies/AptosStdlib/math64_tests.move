#[test_only]
module aptos_std::math64_tests {
    use aptos_std::math64;

    #[test]
    fun test_nested_mul_div() {
       let a = math64::mul_div(1, 1, 1);
       assert!(math64::mul_div(1, a, 1) == 1, 0);
    }

    #[test]
    fun test_nested_mul_div2() {
	assert!(math64::mul_div(1, math64::mul_div(1, 1, 1),1) == 1, 0);
    }

    #[test]
    fun test_nested_mul_div3() {
        let a = math64::mul_div(1, math64::mul_div(1, 1, 1),1);
        assert!(a == 1, 0);
    }
}
