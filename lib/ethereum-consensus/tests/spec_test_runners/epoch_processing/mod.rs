use crate::test_utils::{load_snappy_ssz, Config};
use ethereum_consensus::state_transition::{Context, Result};
use std::fmt;

fn load_epoch_test<S: ssz_rs::Deserialize>(test_case_path: &str) -> (S, Option<S>, Config) {
    let path = test_case_path.to_string() + "/pre.ssz_snappy";
    let pre: S = load_snappy_ssz(&path).unwrap();

    let path = test_case_path.to_string() + "/post.ssz_snappy";
    let post: Option<S> = load_snappy_ssz(&path);

    let config = if test_case_path.contains("minimal") {
        Config::Minimal
    } else {
        Config::Mainnet
    };

    (pre, post, config)
}

pub struct EffectiveBalanceUpdatesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> EffectiveBalanceUpdatesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct Eth1DataResetTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> Eth1DataResetTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct HistoricalRootsUpdateTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> HistoricalRootsUpdateTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct InactivityUpdatesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> InactivityUpdatesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct JustificationAndFinalizationTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> JustificationAndFinalizationTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct ParticipationRecordUpdatesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> ParticipationRecordUpdatesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct ParticipationFlagUpdatesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> ParticipationFlagUpdatesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct RandaoMixesResetTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> RandaoMixesResetTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct RegistryUpdatesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> RegistryUpdatesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct RewardsAndPenaltiesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> RewardsAndPenaltiesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct SlashingsTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> SlashingsTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct SlashingsResetTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> SlashingsResetTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}

pub struct SyncCommitteeUpdatesTestCase<S> {
    pre: S,
    post: Option<S>,
    config: Config,
}

impl<S> SyncCommitteeUpdatesTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let (pre, post, config) = load_epoch_test::<S>(test_case_path);
        Self { pre, post, config }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut S, &Context) -> Result<()>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let result = f(&mut self.pre, &context);
        if let Some(post) = self.post.as_ref() {
            assert_eq!(&self.pre, post);
        } else {
            assert!(result.is_err())
        }
    }
}
