use crate::test_utils::{load_snappy_ssz, load_yaml, Config};
use ethereum_consensus::primitives::Epoch;
use ethereum_consensus::state_transition::{Context, Result};
use serde::Deserialize;
use std::fmt;

pub struct CoreTestCase<S> {
    test_case_path: String,
    post: S,
    meta: Meta,
    config: Config,
}

#[derive(Deserialize)]
struct Meta {
    post_fork: String,
    fork_epoch: Epoch,
    fork_block: Option<usize>,
    blocks_count: usize,
}

impl<S> CoreTestCase<S>
where
    S: fmt::Debug + ssz_rs::Deserialize + PartialEq<S>,
{
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/post.ssz_snappy";
        let post: S = load_snappy_ssz(&path).unwrap();

        let path = test_case_path.to_string() + "/meta.yaml";
        let meta: Meta = load_yaml(&path);

        let config = if test_case_path.contains("minimal") {
            Config::Minimal
        } else {
            Config::Mainnet
        };

        Self {
            test_case_path: test_case_path.to_string(),
            post,
            meta,
            config,
        }
    }

    pub fn execute<F, B, P, A>(&mut self, f: F)
    where
        F: FnOnce(P, Vec<A>, Vec<B>, Context) -> Result<S>,
        B: fmt::Debug + ssz_rs::Deserialize + PartialEq<B>,
        P: ssz_rs::Deserialize,
        A: ssz_rs::Deserialize,
    {
        let path = self.test_case_path.to_string() + "/pre.ssz_snappy";
        let pre: P = load_snappy_ssz(&path).unwrap();

        let blocks_count = self.meta.blocks_count;
        let mut pre_blocks = vec![];
        let mut post_blocks = vec![];
        for i in 0..blocks_count {
            let path = format!("{}/blocks_{}.ssz_snappy", self.test_case_path, i);
            if let Some(fork_index) = self.meta.fork_block {
                if i <= fork_index {
                    let block: A = load_snappy_ssz(&path).unwrap();
                    pre_blocks.push(block);
                } else {
                    let block: B = load_snappy_ssz(&path).unwrap();
                    post_blocks.push(block);
                }
            } else {
                let block: B = load_snappy_ssz(&path).unwrap();
                post_blocks.push(block);
            }
        }

        let mut context = match self.config {
            Config::Minimal => Context::for_minimal(),
            Config::Mainnet => Context::for_mainnet(),
        };
        match self.meta.post_fork.as_ref() {
            "altair" => {
                context.altair_fork_epoch = self.meta.fork_epoch;
                context.bellatrix_fork_epoch = Epoch::MAX;
            }
            "bellatrix" => {
                context.altair_fork_epoch = 0;
                context.bellatrix_fork_epoch = self.meta.fork_epoch;
            }
            _ => todo!("support other forks"),
        }

        let post = f(pre, pre_blocks, post_blocks, context).unwrap();
        assert_eq!(&post, &self.post);
    }
}
