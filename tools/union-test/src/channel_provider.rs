use std::{collections::HashMap, future::Future, sync::Arc, time::Duration};

use jsonrpsee::core::async_trait;
use tokio::sync::Mutex;
use unionlabs::primitives::Bytes;
use voyager_sdk::{anyhow, primitives::ChainId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelPair {
    pub src: u32,
    pub dest: u32,
}
struct PoolInner {
    available: HashMap<(ChainId, ChainId), Vec<ChannelPair>>,
    borrowed: HashMap<(ChainId, ChainId), Vec<ChannelPair>>,
}

pub struct ChannelPool {
    inner: Mutex<PoolInner>,
}

#[derive(Debug)]
pub struct ChannelConfirm {
    pub channel_id: u32,
    pub counterparty_channel_id: u32,
}

pub trait ChannelOpener {
    fn open_channel(
        &self,
        chain_id: ChainId,
        port_id: Bytes,
        counterparty_port_id: Bytes,
        connection_id: u32,
        version: String,
    ) -> anyhow::Result<usize>;
}

#[async_trait]
pub trait ChannelConfirmer {
    async fn wait_for_channel_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<ChannelConfirm>;
}

impl ChannelPool {
    pub fn new() -> Arc<Self> {
        Arc::new(ChannelPool {
            inner: Mutex::new(PoolInner {
                available: HashMap::new(),
                borrowed: HashMap::new(),
            }),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn open_channels<F, C, Fut>(
        &self,
        opener: F,
        confirmer: C,
        src_chain: ChainId,
        src_port: Bytes,
        dst_chain: ChainId,
        dst_port: Bytes,
        connection_id: u32,
        version: String,
        count: usize,
        duration: Duration,
    ) -> anyhow::Result<usize>
    where
        F: Fn(ChainId, Bytes, Bytes, u32, String) -> anyhow::Result<()> + Send + Sync,
        C: Fn(Duration) -> Fut + Send + Sync,
        Fut: Future<Output = anyhow::Result<Vec<ChannelConfirm>>> + Send,
    {
        // let deadline = Instant::now() + duration;

        let mut success_count = 0;

        for _attempt in 0..count {
            opener(
                src_chain.clone(),
                src_port.clone(),
                dst_port.clone(),
                connection_id,
                version.clone(),
            )?;
        }
        // loop {
        //     if Instant::now() >= deadline || success_count == count {
        //         break;
        //     }

        match confirmer(duration).await {
            Ok(confirms) => {
                let mut map = self.inner.lock().await;
                let key = (src_chain.clone(), dst_chain.clone());
                let rev_key = (dst_chain.clone(), src_chain.clone());
                println!("length of confirms: {}", confirms.len());
                for confirm in confirms {
                    println!("confirm in channel_provider: {:?}", confirm);
                    let pair = ChannelPair {
                        src: confirm.counterparty_channel_id,
                        dest: confirm.channel_id,
                    };

                    let dup_forward = map.available.get(&key).is_some_and(|v| v.contains(&pair));
                    let dup_backward = map.available.get(&rev_key).is_some_and(|v| {
                        v.contains(&ChannelPair {
                            src: pair.dest,
                            dest: pair.src,
                        })
                    });

                    if dup_forward || dup_backward {
                        println!(
                            "⚠️  duplicate channel pair detected: \
                            src_chain={}, src_channel={}, dst_chain={}, dst_channel={}",
                            src_chain, pair.src, dst_chain, pair.dest
                        );
                        continue;
                    }

                    // Store in forward direction
                    map.available.entry(key.clone()).or_default().push(pair);
                    // Store in reverse direction
                    map.available
                        .entry(rev_key.clone())
                        .or_default()
                        .push(ChannelPair {
                            src: pair.dest,
                            dest: pair.src,
                        });

                    success_count += 1;
                    println!(
                        "✅  channel-open-confirm: src_chain={}, src_channel={}, dst_chain={}, dst_channel={}",
                        src_chain, pair.src, dst_chain, pair.dest
                    );
                }
            }
            Err(err) => {
                println!("⚠️  error waiting for channel-open-confirm: {}", err);
            }
        }

        Ok(success_count)
    }

    pub async fn get_channel(
        &self,
        src_chain: &ChainId,
        dst_chain: &ChainId,
    ) -> Option<ChannelPair> {
        let mut inner = self.inner.lock().await;
        let key = (src_chain.clone(), dst_chain.clone());
        let pair = inner.available.get_mut(&key).and_then(|v| v.pop());
        if let Some(p) = pair {
            inner.borrowed.entry(key.clone()).or_default().push(p);
            inner
                .borrowed
                .entry((key.1.clone(), key.0.clone()))
                .or_default()
                .push(ChannelPair {
                    src: p.dest,
                    dest: p.src,
                });
        }
        pair
    }

    pub async fn get_available_channel_count(
        &self,
        src_chain: &ChainId,
        dst_chain: &ChainId,
    ) -> usize {
        let inner = self.inner.lock().await;
        let key = (src_chain.clone(), dst_chain.clone());
        inner.available.get(&key).map_or(0, |v| v.len())
    }

    pub async fn release_channel(
        &self,
        src_chain: &ChainId,
        dst_chain: &ChainId,
        pair: ChannelPair,
    ) {
        let mut inner = self.inner.lock().await;
        let key = (src_chain.clone(), dst_chain.clone());
        if let Some(vec) = inner.borrowed.get_mut(&key)
            && let Some(i) = vec.iter().position(|x| *x == pair)
        {
            vec.swap_remove(i);
        }
        let rev = ChannelPair {
            src: pair.dest,
            dest: pair.src,
        };
        let rev_key = (key.1.clone(), key.0.clone());
        if let Some(vec) = inner.borrowed.get_mut(&rev_key)
            && let Some(i) = vec.iter().position(|x| *x == rev)
        {
            vec.swap_remove(i);
        }
        inner.available.entry(key.clone()).or_default().push(pair);
        inner.available.entry(rev_key).or_default().push(rev);
    }
}
