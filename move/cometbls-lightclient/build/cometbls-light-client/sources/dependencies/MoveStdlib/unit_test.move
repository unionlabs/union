#[test_only]
/// Module providing testing functionality. Only included for tests.
module std::unit_test {
    /// Return a `num_signers` number of unique signer values. No ordering or
    /// starting value guarantees are made, only that the order and values of
    /// the signers in the returned vector is deterministic.
    ///
    /// This function is also used to poison modules compiled in `test` mode.
    /// This will cause a linking failure if an attempt is made to publish a
    /// test module in a VM that isn't in unit test mode.
    native public fun create_signers_for_testing(num_signers: u64): vector<signer>;
}
