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
use tracing::{debug, info};
use unionlabs::traits::ChainIdOf;
use voyager_message::{FromOp, VoyagerEffect, VoyagerMessage};

#[derive(Debug, Clone)]
pub struct TxBatch {
    pub min_batch_size: NonZeroUsize,
    pub max_batch_size: NonZeroUsize,
}

impl PurePass<VoyagerMessage> for TxBatch {
    fn run_pass_pure(&self, msgs: Vec<Op<VoyagerMessage>>) -> OptimizationResult<VoyagerMessage> {
        let mut opt_res = OptimizationResult {
            optimize_further: vec![],
            ready: vec![],
        };

        let mut ethereum_mainnet_on_union_batch =
            Batcher::new(self.min_batch_size, self.max_batch_size);
        let mut ethereum_minimal_on_union_batch =
            Batcher::new(self.min_batch_size, self.max_batch_size);
        let mut scroll_on_union_batch = Batcher::new(self.min_batch_size, self.max_batch_size);
        let mut arbitrum_on_union_batch = Batcher::new(self.min_batch_size, self.max_batch_size);
        let mut berachain_on_union_batch = Batcher::new(self.min_batch_size, self.max_batch_size);
        let mut wasm_cosmos_on_union_batch = Batcher::new(self.min_batch_size, self.max_batch_size);
        let mut cosmos_on_union_batch = Batcher::new(self.min_batch_size, self.max_batch_size);

        debug!(count = msgs.len(), "optimizing messages");

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
                            .optimize_further
                            .push((vec![idx], Op::Effect(VoyagerEffect::Relay(e))));
                    }
                },
                msg => opt_res.optimize_further.push((vec![idx], msg)),
            }
        }

        opt_res
            .ready
            .extend(ethereum_mainnet_on_union_batch.into_batch().ready);
        opt_res
            .ready
            .extend(ethereum_minimal_on_union_batch.into_batch().ready);
        opt_res
            .ready
            .extend(scroll_on_union_batch.into_batch().ready);
        opt_res
            .ready
            .extend(arbitrum_on_union_batch.into_batch().ready);
        opt_res
            .ready
            .extend(berachain_on_union_batch.into_batch().ready);
        opt_res
            .ready
            .extend(wasm_cosmos_on_union_batch.into_batch().ready);
        opt_res
            .ready
            .extend(cosmos_on_union_batch.into_batch().ready);

        opt_res
    }
}

struct Batcher<Hc: ChainExt, Tr: ChainExt> {
    // TODO: Make these per-chain-id mappings?
    min_batch_size: NonZeroUsize,
    max_batch_size: NonZeroUsize,
    #[allow(clippy::type_complexity)] // leave me alone
    // bucket by chain id and then again by batch size
    batches: HashMap<ChainIdOf<Hc>, Vec<(usize, Effect<Hc, Tr>)>>,
}

impl<Hc: ChainExt, Tr: ChainExt> Batcher<Hc, Tr> {
    fn new(min_batch_size: NonZeroUsize, max_batch_size: NonZeroUsize) -> Self {
        Self {
            max_batch_size,
            min_batch_size,
            batches: Default::default(),
        }
    }

    fn push(&mut self, idx: usize, effect: identified!(Effect<Hc, Tr>)) {
        let entry = self.batches.entry(effect.chain_id.clone()).or_default();
        match effect.t {
            Effect::Batch(b) => {
                for e in b.0 {
                    entry.push((idx, e))
                }
            }
            e => {
                entry.push((idx, e));
            }
        }
    }

    fn into_batch(self) -> OptimizationResult<VoyagerMessage>
    where
        AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Hc, Tr>)>,
    {
        let ready = self
            .batches
            .into_iter()
            .flat_map(|(chain_id, msgs): (ChainIdOf<Hc>, _)| {
                msgs.into_iter()
                    .chunks(self.max_batch_size.get())
                    .into_iter()
                    .map(|chunk| {
                        let (ids, mut batch) = chunk.into_iter().collect::<(Vec<_>, Vec<_>)>();

                        if batch.len() == 1 {
                            (
                                ids,
                                VoyagerMessage::from_op(effect::<RelayMessage>(id(
                                    chain_id.clone(),
                                    batch.pop().expect("length is 1; qed;"),
                                ))),
                            )
                        } else {
                            info!(batch_size = batch.len(), %chain_id, "batched messages");

                            (
                                ids,
                                VoyagerMessage::from_op(effect::<RelayMessage>(id(
                                    chain_id.clone(),
                                    BatchMsg(batch),
                                ))),
                            )
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<(Vec<usize>, Op<VoyagerMessage>)>>();

        OptimizationResult {
            optimize_further: vec![],
            ready,
        }
    }
}

#[cfg(test)]
mod tests {
    use chain_utils::{ethereum::Ethereum, union::Union, wasm::Wasm};
    use relay_message::effect::MsgConnectionOpenInitData;
    use unionlabs::{
        ethereum::config::Mainnet,
        ibc::core::{
            commitment::merkle_prefix::MerklePrefix,
            connection::{
                counterparty::Counterparty, msg_connection_open_init::MsgConnectionOpenInit,
            },
        },
    };

    use super::*;

    #[test]
    fn test_batch() {
        let msg = VoyagerMessage::from_op(effect::<RelayMessage>(id(
            "union".parse().unwrap(),
            MsgConnectionOpenInitData::<Wasm<Union>, Ethereum<Mainnet>>(MsgConnectionOpenInit {
                client_id: "client_id".parse().unwrap(),
                counterparty: Counterparty {
                    client_id: "counterparty".parse().unwrap(),
                    connection_id: "".parse().unwrap(),
                    prefix: MerklePrefix {
                        key_prefix: b"".to_vec(),
                    },
                },
                version: unionlabs::ibc::core::connection::version::Version {
                    identifier: "1".to_owned(),
                    features: [].to_vec(),
                },
                delay_period: 0,
            }),
        )));

        let msgs = [msg.clone(), msg].to_vec();

        let batcher = TxBatch {
            min_batch_size: 1.try_into().unwrap(),
            max_batch_size: 10.try_into().unwrap(),
        };

        dbg!(batcher.run_pass_pure(msgs));
    }
}
