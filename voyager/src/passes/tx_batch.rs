use std::{collections::HashMap, num::NonZeroUsize};

use itertools::Itertools;
use queue_msg::{
    effect,
    optimize::{OptimizationResult, PurePass},
    Op,
};
use relay_message::{
    effect::{AnyEffect, BatchMsg, Effect},
    id, identified, AnyLightClientIdentified, ChainExt, RelayMessage,
};
use unionlabs::traits::ChainIdOf;
use voyager_message::{FromOp, VoyagerEffect, VoyagerMessage};

#[derive(Debug, Clone)]
pub struct TxBatch {
    pub size_limit: NonZeroUsize,
}

impl PurePass<VoyagerMessage> for TxBatch {
    fn run_pass_pure(&self, msgs: Vec<Op<VoyagerMessage>>) -> OptimizationResult<VoyagerMessage> {
        let mut opt_res = OptimizationResult {
            optimize_further: vec![],
            ready: vec![],
        };

        let mut ethereum_mainnet_on_union_batch = Batcher::new(self.size_limit);
        let mut ethereum_minimal_on_union_batch = Batcher::new(self.size_limit);
        let mut scroll_on_union_batch = Batcher::new(self.size_limit);
        let mut arbitrum_on_union_batch = Batcher::new(self.size_limit);
        let mut berachain_on_union_batch = Batcher::new(self.size_limit);
        let mut wasm_cosmos_on_union_batch = Batcher::new(self.size_limit);
        let mut cosmos_on_union_batch = Batcher::new(self.size_limit);

        for (idx, msg) in msgs.into_iter().enumerate() {
            match msg {
                Op::Effect(VoyagerEffect::Relay(effect)) => match effect {
                    AnyLightClientIdentified::EthereumMainnetOnUnion(effect) => {
                        ethereum_mainnet_on_union_batch.push(idx, effect)
                    }
                    AnyLightClientIdentified::EthereumMinimalOnUnion(effect) => {
                        ethereum_minimal_on_union_batch.push(idx, effect)
                    }
                    AnyLightClientIdentified::ScrollOnUnion(effect) => {
                        scroll_on_union_batch.push(idx, effect)
                    }
                    AnyLightClientIdentified::ArbitrumOnUnion(effect) => {
                        arbitrum_on_union_batch.push(idx, effect)
                    }
                    AnyLightClientIdentified::BerachainOnUnion(effect) => {
                        berachain_on_union_batch.push(idx, effect)
                    }
                    AnyLightClientIdentified::WasmCosmosOnUnion(effect) => {
                        wasm_cosmos_on_union_batch.push(idx, effect)
                    }
                    AnyLightClientIdentified::CosmosOnUnion(effect) => {
                        cosmos_on_union_batch.push(idx, effect)
                    }
                    e => {
                        opt_res
                            .ready
                            .push((vec![idx], Op::Effect(VoyagerEffect::Relay(e))));
                    }
                },
                msg => opt_res.ready.push((vec![idx], msg)),
            }
        }

        opt_res
            .ready
            .extend(ethereum_mainnet_on_union_batch.into_batch());
        opt_res
            .ready
            .extend(ethereum_minimal_on_union_batch.into_batch());
        opt_res.ready.extend(scroll_on_union_batch.into_batch());
        opt_res.ready.extend(arbitrum_on_union_batch.into_batch());
        opt_res.ready.extend(berachain_on_union_batch.into_batch());
        opt_res
            .ready
            .extend(wasm_cosmos_on_union_batch.into_batch());
        opt_res.ready.extend(cosmos_on_union_batch.into_batch());

        opt_res
    }
}

struct Batcher<Hc: ChainExt, Tr: ChainExt> {
    // TODO: Make this a per-chain-id mapping?
    max_size: NonZeroUsize,
    #[allow(clippy::type_complexity)] // leave me alone
    // bucket by chain id and then again by batch size
    batches: HashMap<ChainIdOf<Hc>, Vec<(usize, Effect<Hc, Tr>)>>,
}

impl<Hc: ChainExt, Tr: ChainExt> Batcher<Hc, Tr> {
    fn new(max_size: NonZeroUsize) -> Self {
        Self {
            max_size,
            batches: Default::default(),
        }
    }

    fn push(&mut self, idx: usize, effect: identified!(Effect<Hc, Tr>)) {
        match effect.t {
            Effect::Batch(b) => {
                let entry = self.batches.entry(effect.chain_id.clone()).or_default();
                for e in b.0 {
                    entry.push((idx, e))
                }
            }
            e => {
                let entry = self.batches.entry(effect.chain_id.clone()).or_default();
                entry.push((idx, e))
            }
        }
    }

    fn into_batch(self) -> Vec<(Vec<usize>, Op<VoyagerMessage>)>
    where
        AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Hc, Tr>)>,
    {
        self.batches
            .into_iter()
            .flat_map(|(chain_id, x): (ChainIdOf<Hc>, _)| {
                x.into_iter()
                    .chunks(self.max_size.get())
                    .into_iter()
                    .map(|x| {
                        let (ids, batch) = x.into_iter().collect::<(Vec<_>, Vec<_>)>();

                        (
                            ids,
                            VoyagerMessage::from_op(effect::<RelayMessage>(id(
                                chain_id.clone(),
                                BatchMsg(batch),
                            ))),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<(Vec<usize>, Op<VoyagerMessage>)>>()
    }
}
