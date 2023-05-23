use crate::test_utils::{load_snappy_ssz, load_yaml, Config};
use ethereum_consensus::primitives::{Bytes32, Hash32};
use ethereum_consensus::state_transition::{Context, Result};
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize)]
struct Eth1 {
    eth1_block_hash: Bytes32,
    eth1_timestamp: u64,
}

#[derive(Deserialize)]
struct Meta {
    deposits_count: usize,
    execution_payload_header: Option<bool>,
}

pub struct InitializationTestCase<S, D, H> {
    eth1: Eth1,
    deposits: Vec<D>,
    execution_payload_header: Option<H>,
    state: S,
    config: Config,
}

impl<S, D, H> InitializationTestCase<S, D, H>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
    D: ssz_rs::Deserialize,
    H: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/eth1.yaml";
        let eth1: Eth1 = load_yaml(&path);

        let path = test_case_path.to_string() + "/meta.yaml";
        let meta: Meta = load_yaml(&path);

        let mut deposits = vec![];
        for i in 0..meta.deposits_count {
            let path = format!("{test_case_path}/deposits_{i}.ssz_snappy");
            let deposit: D = load_snappy_ssz(&path).unwrap();
            deposits.push(deposit);
        }

        let execution_payload_header = if meta.execution_payload_header.unwrap_or(false) {
            let path = test_case_path.to_string() + "/execution_payload_header.ssz_snappy";
            let header: H = load_snappy_ssz(&path).unwrap();
            Some(header)
        } else {
            None
        };

        let path = test_case_path.to_string() + "/state.ssz_snappy";
        let state: S = load_snappy_ssz(&path).unwrap();

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            eth1,
            deposits,
            execution_payload_header,
            state,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(Hash32, u64, &mut [D], Option<&H>, &Context) -> Result<S>,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let state = f(
            self.eth1.eth1_block_hash.clone(),
            self.eth1.eth1_timestamp,
            &mut self.deposits,
            self.execution_payload_header.as_ref(),
            &context,
        )
        .unwrap();
        assert_eq!(state, self.state);
    }
}

pub struct ValidityTestCase<S> {
    state: S,
    is_valid: bool,
    config: Config,
}

impl<S> ValidityTestCase<S>
where
    S: ssz_rs::Deserialize,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/genesis.ssz_snappy";
        let state: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/is_valid.yaml";
        let is_valid: bool = load_yaml(&path);

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            todo!("support preset");
        };

        Self {
            state,
            is_valid,
            config,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&S, &Context) -> bool,
    {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        let is_valid = f(&self.state, &context);
        assert_eq!(is_valid, self.is_valid);
    }
}
