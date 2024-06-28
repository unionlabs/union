use std::{
    collections::HashMap,
    num::{NonZeroU8, NonZeroUsize},
};

use itertools::Itertools;
use queue_msg::{
    effect,
    optimize::{OptimizationResult, PurePass},
    retry, Op,
};
use relay_message::{
    effect::{AnyEffect, BatchMsg, Effect},
    id, identified, AnyLightClientIdentified, ChainExt, RelayMessage,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use unionlabs::traits::ChainIdOf;
use voyager_message::{FromOp, VoyagerEffect, VoyagerMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxBatch {
    // TODO: Make these per-chain-id mappings?
    pub retry_count: NonZeroU8,
    pub min_batch_size: NonZeroUsize,
    pub max_batch_size: NonZeroUsize,
}

impl PurePass<VoyagerMessage> for TxBatch {
    fn run_pass_pure(&self, msgs: Vec<Op<VoyagerMessage>>) -> OptimizationResult<VoyagerMessage> {
        let mut opt_res = OptimizationResult {
            optimize_further: vec![],
            ready: vec![],
        };

        let mut ethereum_mainnet_on_union_batch = Batcher::new(self);
        let mut ethereum_minimal_on_union_batch = Batcher::new(self);
        let mut scroll_on_union_batch = Batcher::new(self);
        let mut arbitrum_on_union_batch = Batcher::new(self);
        let mut berachain_on_union_batch = Batcher::new(self);
        let mut wasm_cosmos_on_union_batch = Batcher::new(self);
        let mut cosmos_on_union_batch = Batcher::new(self);

        debug!(count = msgs.len(), "optimizing messages");

        let mut do_batch = |idx, effect, opt_res: &mut OptimizationResult<_>| match effect {
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
        };

        for (idx, msg) in msgs.into_iter().enumerate() {
            match msg {
                Op::Retry { remaining: _, msg } => match *msg {
                    Op::Effect(VoyagerEffect::Relay(effect)) => do_batch(idx, effect, &mut opt_res),
                    msg => opt_res.optimize_further.push((vec![idx], msg)),
                },
                Op::Effect(VoyagerEffect::Relay(effect)) => do_batch(idx, effect, &mut opt_res),
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

struct Batcher<'a, Hc: ChainExt, Tr: ChainExt> {
    config: &'a TxBatch,
    #[allow(clippy::type_complexity)] // leave me alone
    // bucket by chain id and then again by batch size
    batches: HashMap<ChainIdOf<Hc>, Vec<(usize, Effect<Hc, Tr>)>>,
}

impl<'a, Hc: ChainExt, Tr: ChainExt> Batcher<'a, Hc, Tr> {
    fn new(config: &'a TxBatch) -> Self {
        Self {
            config,
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
                    .chunks(self.config.max_batch_size.get())
                    .into_iter()
                    .map(|chunk| {
                        let (ids, mut batch) = chunk.into_iter().collect::<(Vec<_>, Vec<_>)>();

                        if batch.len() == 1 {
                            (
                                ids,
                                VoyagerMessage::from_op(retry(
                                    self.config.retry_count,
                                    effect::<RelayMessage>(id(
                                        chain_id.clone(),
                                        batch.pop().expect("length is 1; qed;"),
                                    )),
                                )),
                            )
                        } else {
                            info!(batch_size = batch.len(), %chain_id, "batched messages");

                            (
                                ids,
                                VoyagerMessage::from_op(retry(
                                    self.config.retry_count,
                                    effect::<RelayMessage>(id(chain_id.clone(), BatchMsg(batch))),
                                )),
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
            retry_count: 3.try_into().unwrap(),
            min_batch_size: 1.try_into().unwrap(),
            max_batch_size: 10.try_into().unwrap(),
        };

        dbg!(batcher.run_pass_pure(msgs));
    }
}
