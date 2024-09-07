use core::fmt;
use std::{fmt::Debug, future::Future, sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use jsonrpsee::core::{
    async_trait,
    client::{BatchResponse, ClientT},
    params::BatchRequestBuilder,
    traits::ToRpcParams,
    DeserializeOwned,
};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{debug, debug_span, error, Instrument};

#[derive(Debug, Clone)]
pub struct Client {
    client: Arc<ArcSwap<jsonrpsee::core::client::Client>>,
    handle: Arc<JoinHandle<()>>,
}

impl Client {
    pub async fn new<
        B: (Fn() -> Fut) + Send + 'static,
        Fut: Future<Output = Result<jsonrpsee::core::client::Client, E>> + Send + 'static,
        E: Debug + Send,
    >(
        builder: B,
    ) -> Result<Self, E> {
        let client = Arc::new(ArcSwap::from_pointee(builder().await?));

        let handle = tokio::spawn({
            let client = client.clone();

            async move {
                let mut total_reconnects = 0;

                loop {
                    client.load().on_disconnect().await;

                    debug!("client disconnected");

                    let reason = client.load().disconnect_reason().await;

                    debug!("disconnect reason: {reason:?}");

                    let mut retry_ms = 500;

                    const MAX_RETRY_MS: u64 = 8_000;

                    let mut attempt = 0;

                    let new_client = loop {
                        match builder().await {
                            Ok(client) => break client,
                            Err(error) => {
                                attempt += 1;

                                error!(
                                    ?error,
                                    %attempt,
                                    "error while reconnecting client, \
                                    trying again in {retry_ms} ms"
                                );

                                sleep(Duration::from_millis(retry_ms)).await;

                                retry_ms = std::cmp::min(retry_ms * 2, MAX_RETRY_MS);
                            }
                        }
                    };

                    total_reconnects += 1;

                    debug!(%total_reconnects, "client reconnected");

                    client.store(Arc::new(new_client));
                }
            }
            .instrument(debug_span!("ws client reconnect task",))
        });

        Ok(Self {
            client,
            handle: Arc::new(handle),
        })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[async_trait]
impl ClientT for Client {
    async fn notification<Params>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<(), jsonrpsee::core::client::Error>
    where
        Params: ToRpcParams + Send,
    {
        self.client.load().notification(method, params).await
    }

    async fn request<R, Params>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<R, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        self.client.load().request(method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned + fmt::Debug + 'a,
    {
        self.client.load().batch_request(batch).await
    }
}
