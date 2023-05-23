use crate::test_utils::{load_snappy_ssz, load_yaml, Config};
use ethereum_consensus::state_transition::{Context, Result};
use serde::Deserialize;
use std::fmt;

pub struct AttestationTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: T,
    config: Config,
}

impl<S, T> AttestationTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/attestation.ssz_snappy";
        let operation: T = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &T, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        let result = f(&mut self.pre, &self.operation, &context);

        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct AttesterSlashingTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: T,
    config: Config,
}

impl<S, T> AttesterSlashingTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/attester_slashing.ssz_snappy";
        let operation: T = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &mut T, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        let result = f(&mut self.pre, &mut self.operation, &context);

        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct BlockHeaderTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: T,
    config: Config,
}

impl<S, T> BlockHeaderTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/block.ssz_snappy";
        let operation: T = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &mut T, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        let result = f(&mut self.pre, &mut self.operation, &context);

        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct DepositTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: Option<T>,
    config: Config,
}

impl<S, T> DepositTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/deposit.ssz_snappy";
        let operation = load_snappy_ssz::<T>(&path);

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &mut T, &Context) -> Result<()>,
    {
        if let Some(operation) = self.operation.as_mut() {
            let context = match self.config {
                Config::Minimal => Context::for_minimal(),
                Config::Mainnet => Context::for_mainnet(),
            };

            let result = f(&mut self.pre, operation, &context);

            if let Some(post) = self.post.as_ref() {
                assert_eq!(&self.pre, post);
            } else {
                assert!(result.is_err())
            }
        } else {
            assert!(self.post.is_none())
        }
    }
}

pub struct ProposerSlashingTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: Option<T>,
    config: Config,
}

impl<S, T> ProposerSlashingTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/proposer_slashing.ssz_snappy";
        let operation = load_snappy_ssz::<T>(&path);

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &mut T, &Context) -> Result<()>,
    {
        if let Some(operation) = self.operation.as_mut() {
            let context = match self.config {
                Config::Minimal => Context::for_minimal(),
                Config::Mainnet => Context::for_mainnet(),
            };

            let result = f(&mut self.pre, operation, &context);

            if let Some(post) = self.post.as_ref() {
                assert_eq!(&self.pre, post);
            } else {
                assert!(result.is_err())
            }
        } else {
            assert!(self.post.is_none())
        }
    }
}

pub struct VoluntaryExitTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: T,
    config: Config,
}

impl<S, T> VoluntaryExitTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/voluntary_exit.ssz_snappy";
        let operation: T = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &mut T, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        let result = f(&mut self.pre, &mut self.operation, &context);

        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct SyncAggregateTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: T,
    config: Config,
}

impl<S, T> SyncAggregateTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/sync_aggregate.ssz_snappy";
        let operation: T = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &T, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        let result = f(&mut self.pre, &self.operation, &context);

        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

#[derive(Deserialize)]
struct ExecutionValidity {
    execution_valid: bool,
}

pub struct ExecutionPayloadTestCase<S, T> {
    pre: S,
    post: Option<S>,
    operation: T,
    execution_validity: ExecutionValidity,
    config: Config,
}

impl<S, T> ExecutionPayloadTestCase<S, T>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    T: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post = load_snappy_ssz::<S>(&path);

        let path = test_case_path.to_string() + "/execution_payload.ssz_snappy";
        let operation: T = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/execution.yaml";
        let execution_validity: ExecutionValidity = load_yaml(&path);

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            post,
            operation,
            execution_validity,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &mut T, &Context, bool) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        let result = f(
            &mut self.pre,
            &mut self.operation,
            &context,
            self.execution_validity.execution_valid,
        );

        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}
