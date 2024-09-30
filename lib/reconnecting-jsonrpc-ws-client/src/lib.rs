use core::fmt;
use std::{fmt::Debug, future::Future, sync::Arc, time::Duration};

use arc_swap::ArcSwapOption;
use jsonrpsee::core::{
    async_trait,
    client::{BatchResponse, ClientT},
    params::BatchRequestBuilder,
    traits::ToRpcParams,
    DeserializeOwned,
};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{debug, debug_span, error, instrument, trace, Instrument};

#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

#[derive(Debug)] // NOTE: Does NOT impl Clone, otherwise you have a bad time with cancellation tokens being cloned and dropped
pub struct ClientInner {
    client: Arc<ArcSwapOption<jsonrpsee::core::client::Client>>,
    cancellation_token: CancellationToken,
}

impl Client {
    pub fn new<
        B: (Fn() -> Fut) + Clone + Send + 'static,
        Fut: Future<Output = Result<jsonrpsee::core::client::Client, E>> + Send + 'static,
        E: Debug + Send,
    >(
        builder: B,
    ) -> Self {
        let client = Arc::new(ArcSwapOption::from(None));

        let cancellation_token = CancellationToken::new();

        tokio::spawn({
            let maybe_client = client.clone();
            let cancellation_token = cancellation_token.child_token();

            async move {
                cancellation_token
                    .run_until_cancelled(async move {
                        let mut total_reconnects = 0;

                        loop {
                            // if !healthy {
                            //     continue;
                            // }

                            let loaded_maybe_client = maybe_client.load();

                            match loaded_maybe_client.as_deref() {
                                Some(client) => {
                                    let client: &jsonrpsee::core::client::Client = client;

                                    tokio::select! {
                                        _ = client.on_disconnect() => {
                                            debug!("client disconnected");

                                            maybe_client.store(None);

                                            let reason = tokio::time::timeout(
                                                Duration::from_secs(1),
                                                client.disconnect_reason()
                                            )
                                            .await
                                            .ok();

                                            match reason {
                                                Some(reason) => {
                                                    debug!("disconnect reason: {reason:?}");
                                                }
                                                None => {
                                                    debug!("unable to retrieve disconnect reason");
                                                }
                                            }

                                            // reconnect(
                                            //     &maybe_client,
                                            //     builder.clone(),
                                            //     &mut total_reconnects,
                                            // )
                                            // .await;

                                            // healthy = true;
                                            // healthy.store(true, Ordering::SeqCst);
                                        }
                                        _ = sleep(Duration::from_secs(3)) => {
                                            trace!("still connected");
                                       }
                                    }

                                    continue;
                                }
                                None => {
                                    debug!("client not connected yet, attempting to connect");

                                    reconnect(
                                        &maybe_client,
                                        builder.clone(),
                                        &mut total_reconnects,
                                    )
                                    .await;
                                }
                            }
                        }
                    })
                    .instrument(debug_span!("ws client reconnect task"))
                    .await
            }
        });

        Self {
            inner: Arc::new(ClientInner {
                client,
                // handle: Arc::new(handle),
                cancellation_token,
            }),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.inner.client.load().is_some()
        // && self.healthy.load(Ordering::SeqCst)
    }

    pub async fn wait_until_connected(
        &self,
        timeout: Duration,
    ) -> Result<(), ConnectionTimeoutError> {
        tokio::time::timeout(timeout, async {
            loop {
                if self.is_healthy() {
                    return;
                } else {
                    trace!("unhealthy");
                    sleep(Duration::from_millis(100)).await
                }
            }
        })
        .await
        .map_err(|_| ConnectionTimeoutError { timeout })
    }

    pub fn shutdown(&self) {
        self.inner.cancellation_token.cancel();
        // self.handle.abort()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("websocket connection timed out after {}.{}s", timeout.as_secs(), timeout.subsec_nanos())]
pub struct ConnectionTimeoutError {
    pub timeout: Duration,
}

// impl Drop for Client {
//     fn drop(&mut self) {
//         // panic!("WHO IS DROPPING ME");
//         self.inner.cancellation_token.cancel();
//         // self.handle.abort();
//     }
// }

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
        self.inner
            .client
            .load_full()
            .as_deref()
            .ok_or_else(|| {
                jsonrpsee::core::client::Error::Custom(format!(
                    "not yet connected (request: {method})",
                ))
            })?
            .notification(method, params)
            .await
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
        self.inner
            .client
            .load_full()
            .as_deref()
            .ok_or_else(|| {
                jsonrpsee::core::client::Error::Custom(format!(
                    "not yet connected (request: {method})",
                ))
            })?
            .request(method, params)
            .await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned + fmt::Debug + 'a,
    {
        self.inner
            .client
            .load_full()
            .as_deref()
            .ok_or(jsonrpsee::core::client::Error::Custom(
                "not yet connected (batch request)".to_owned(),
            ))?
            .batch_request(batch)
            .await
    }
}

#[instrument(name = "reconnect", skip_all)]
async fn reconnect<
    B: (Fn() -> Fut) + Send + 'static,
    Fut: Future<Output = Result<jsonrpsee::core::client::Client, E>> + Send + 'static,
    E: Debug + Send,
>(
    maybe_client: &ArcSwapOption<jsonrpsee::core::client::Client>,
    builder: B,
    total_reconnects: &mut u64,
) {
    let mut retry_ms = 5;

    const MAX_RETRY_MS: u64 = 8_000;

    let mut attempt = 0;

    let new_client = loop {
        match builder().await {
            Ok(client) => break client,
            Err(error) => {
                attempt += 1;

                debug!(
                    ?error,
                    %attempt,
                    "error while reconnecting client, \
                    trying again in {retry_ms} ms"
                );

                sleep(Duration::from_millis(retry_ms)).await;

                retry_ms = std::cmp::min((retry_ms * 3) / 2, MAX_RETRY_MS);
            }
        }
    };

    *total_reconnects += 1;

    debug!(%total_reconnects, "client reconnected");

    maybe_client.store(Some(Arc::new(new_client)));
}
