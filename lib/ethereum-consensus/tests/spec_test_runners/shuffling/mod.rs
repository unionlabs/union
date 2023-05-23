use crate::test_utils::{load_yaml, Config};
use ethereum_consensus::phase0::compute_shuffled_index;
use ethereum_consensus::primitives::Bytes32;
use ethereum_consensus::state_transition::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ShufflingTestData {
    seed: Bytes32,
    count: usize,
    mapping: Vec<usize>,
}

#[derive(Debug)]
pub struct CoreTestCase {
    data: ShufflingTestData,
    config: Config,
}

impl CoreTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/mapping.yaml";

        let data: ShufflingTestData = load_yaml(&path);

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self { data, config }
    }

    pub fn execute(&self) {
        self.test_unoptimized_forward_shuffle();
    }

    fn test_unoptimized_forward_shuffle(&self) {
        let context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };

        for index in 0..self.data.count {
            let result =
                compute_shuffled_index(index, self.data.count, &self.data.seed, &context).unwrap();
            assert_eq!(result, self.data.mapping[index]);
        }
    }
}
