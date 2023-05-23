use crate::test_utils::{load_snappy_ssz, Config};
use ethereum_consensus::primitives::Gwei;
use ethereum_consensus::state_transition::Context;
use ssz_rs::prelude::*;
use std::fmt;
// NOTE: constant across presets
use ethereum_consensus::phase0::mainnet::VALIDATOR_REGISTRY_LIMIT;

#[derive(Debug, Default, SimpleSerialize)]
struct Deltas {
    rewards: List<Gwei, VALIDATOR_REGISTRY_LIMIT>,
    penalties: List<Gwei, VALIDATOR_REGISTRY_LIMIT>,
}

type Pair = (Vec<Gwei>, Vec<Gwei>);

fn assert_deltas(expected: &Deltas, provided: Pair) {
    let rewards = provided.0;
    let penalties = provided.1;

    assert_eq!(rewards, expected.rewards.as_ref());
    assert_eq!(penalties, expected.penalties.as_ref());
}

pub struct BasicTestCase<S> {
    pre: S,
    source_deltas: Deltas,
    target_deltas: Deltas,
    head_deltas: Deltas,
    inclusion_delay_deltas: Option<Deltas>,
    inactivity_penalty_deltas: Deltas,
    config: Config,
}

impl<S> BasicTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/source_deltas.ssz_snappy";
        let source_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/target_deltas.ssz_snappy";
        let target_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/head_deltas.ssz_snappy";
        let head_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/inclusion_delay_deltas.ssz_snappy";
        let inclusion_delay_deltas: Option<Deltas> = load_snappy_ssz(&path);

        let path = test_case_path.to_string() + "/inactivity_penalty_deltas.ssz_snappy";
        let inactivity_penalty_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            source_deltas,
            target_deltas,
            head_deltas,
            inclusion_delay_deltas,
            inactivity_penalty_deltas,
            config,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&S, &Context) -> (Pair, Pair, Pair, Option<Pair>, Pair),
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let (
            source_deltas,
            target_deltas,
            head_deltas,
            inclusion_delay_deltas,
            inactivity_penalty_deltas,
        ) = f(&self.pre, &context);

        assert_deltas(&self.source_deltas, source_deltas);
        assert_deltas(&self.target_deltas, target_deltas);
        assert_deltas(&self.head_deltas, head_deltas);
        if let Some(expected) = self.inclusion_delay_deltas.as_ref() {
            assert_deltas(expected, inclusion_delay_deltas.unwrap());
        } else {
            assert!(inclusion_delay_deltas.is_none());
        }
        assert_deltas(&self.inactivity_penalty_deltas, inactivity_penalty_deltas);
    }
}

pub struct LeakTestCase<S> {
    pre: S,
    source_deltas: Deltas,
    target_deltas: Deltas,
    head_deltas: Deltas,
    inclusion_delay_deltas: Option<Deltas>,
    inactivity_penalty_deltas: Deltas,
    config: Config,
}

impl<S> LeakTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/source_deltas.ssz_snappy";
        let source_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/target_deltas.ssz_snappy";
        let target_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/head_deltas.ssz_snappy";
        let head_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/inclusion_delay_deltas.ssz_snappy";
        let inclusion_delay_deltas: Option<Deltas> = load_snappy_ssz(&path);

        let path = test_case_path.to_string() + "/inactivity_penalty_deltas.ssz_snappy";
        let inactivity_penalty_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            source_deltas,
            target_deltas,
            head_deltas,
            inclusion_delay_deltas,
            inactivity_penalty_deltas,
            config,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&S, &Context) -> (Pair, Pair, Pair, Option<Pair>, Pair),
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let (
            source_deltas,
            target_deltas,
            head_deltas,
            inclusion_delay_deltas,
            inactivity_penalty_deltas,
        ) = f(&self.pre, &context);

        assert_deltas(&self.source_deltas, source_deltas);
        assert_deltas(&self.target_deltas, target_deltas);
        assert_deltas(&self.head_deltas, head_deltas);
        if let Some(expected) = self.inclusion_delay_deltas.as_ref() {
            assert_deltas(expected, inclusion_delay_deltas.unwrap());
        } else {
            assert!(inclusion_delay_deltas.is_none());
        }
        assert_deltas(&self.inactivity_penalty_deltas, inactivity_penalty_deltas);
    }
}

pub struct RandomTestCase<S> {
    pre: S,
    source_deltas: Deltas,
    target_deltas: Deltas,
    head_deltas: Deltas,
    inclusion_delay_deltas: Option<Deltas>,
    inactivity_penalty_deltas: Deltas,
    config: Config,
}

impl<S> RandomTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/source_deltas.ssz_snappy";
        let source_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/target_deltas.ssz_snappy";
        let target_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/head_deltas.ssz_snappy";
        let head_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/inclusion_delay_deltas.ssz_snappy";
        let inclusion_delay_deltas: Option<Deltas> = load_snappy_ssz(&path);

        let path = test_case_path.to_string() + "/inactivity_penalty_deltas.ssz_snappy";
        let inactivity_penalty_deltas: Deltas = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            pre,
            source_deltas,
            target_deltas,
            head_deltas,
            inclusion_delay_deltas,
            inactivity_penalty_deltas,
            config,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&S, &Context) -> (Pair, Pair, Pair, Option<Pair>, Pair),
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let (
            source_deltas,
            target_deltas,
            head_deltas,
            inclusion_delay_deltas,
            inactivity_penalty_deltas,
        ) = f(&self.pre, &context);

        assert_deltas(&self.source_deltas, source_deltas);
        assert_deltas(&self.target_deltas, target_deltas);
        assert_deltas(&self.head_deltas, head_deltas);
        if let Some(expected) = self.inclusion_delay_deltas.as_ref() {
            assert_deltas(expected, inclusion_delay_deltas.unwrap());
        } else {
            assert!(inclusion_delay_deltas.is_none());
        }
        assert_deltas(&self.inactivity_penalty_deltas, inactivity_penalty_deltas);
    }
}
