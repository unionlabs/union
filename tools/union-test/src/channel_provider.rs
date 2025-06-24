use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use jsonrpsee::{
    core::{async_trait}};
use voyager_sdk::{primitives::ChainId, anyhow};
use unionlabs::primitives::Bytes;
use std::future::Future;

#[derive(Debug, Clone, Copy)]
pub struct ChannelPair {
    pub src: u32,
    pub dest: u32,
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

pub struct ChannelPool {
    inner: Mutex<HashMap<(ChainId, ChainId), Vec<ChannelPair>>>,
}

impl ChannelPool {
    pub fn new() -> Arc<Self> {
        Arc::new(ChannelPool {
            inner: Mutex::new(HashMap::new()),
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
    ) -> anyhow::Result<usize>
    where
        F: Fn(ChainId, Bytes, Bytes, u32, String) -> anyhow::Result<()> + Send + Sync,
        C: Fn(Duration) -> Fut + Send + Sync,
        Fut: Future<Output = anyhow::Result<ChannelConfirm>> + Send,
    {
        let mut map = self.inner.lock().await;
        let mut success_count = 0;

        for _ in 0..count {
            opener(
                src_chain.clone(),
                src_port.clone(),
                dst_port.clone(),
                connection_id,
                version.clone(),
            )?;

            match confirmer(Duration::from_secs(240))
                .await{
                    Ok(confirm) => {
                        let pair = ChannelPair {
                            src: confirm.counterparty_channel_id,
                            dest: confirm.channel_id,
                        };
                        println!(
                            "✅  channel-open-confirm: src_chain={}, src_channel={}, dst_chain={}, dst_channel={}",
                            src_chain, pair.src, dst_chain, pair.dest
                        );

                        // Store in forward direction
                        map.entry((src_chain.clone(), dst_chain.clone()))
                            .or_default()
                            .push(pair);
                        // Store in reverse direction
                        map.entry((dst_chain.clone(), src_chain.clone()))
                            .or_default()
                            .push(ChannelPair { src: pair.dest, dest: pair.src });

                        success_count += 1;
                    }
                    Err(err) => {
                        eprintln!("⚠️  error waiting for channel-open-confirm: {}", err);
                    }
                }

        }

        Ok(success_count)
    }

    pub async fn get_channel(
        &self,
        src_chain: &ChainId,
        dst_chain: &ChainId,
    ) -> Option<ChannelPair> {
        let mut map = self.inner.lock().await;
        map.get_mut(&(src_chain.clone(), dst_chain.clone()))
            .and_then(|v| v.pop())
    }
}
