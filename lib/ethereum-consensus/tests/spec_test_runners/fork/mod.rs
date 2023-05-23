use crate::test_utils::{load_snappy_ssz, Config};
use ethereum_consensus::state_transition::Context;
use std::fmt;

pub struct ForkTestCase<S, T> {
    pre: S,
    post: T,
    config: Config,
}

impl<S, T> ForkTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: fmt::Debug + ssz_rs::Deserialize + PartialEq<T>,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post: T = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self { pre, post, config }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&S, &Context) -> T,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let post = f(&self.pre, &context);
        assert_eq!(post, self.post);
    }
}
