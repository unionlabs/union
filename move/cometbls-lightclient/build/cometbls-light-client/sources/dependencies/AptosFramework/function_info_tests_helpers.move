#[test_only]
module 0xcafe::function_info_tests_helpers {
    use aptos_framework::function_info::FunctionInfo;

    public fun lhs(_s: &FunctionInfo) {}

    public fun rhs() {}

    public fun rhs2(_u: u8) {}

    public fun lhs_generic<T>(_s: &FunctionInfo) {}

    public fun rhs_generic<T>() {}
}
