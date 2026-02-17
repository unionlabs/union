use core::fmt;
use std::{
    fmt::Debug,
    num::{NonZeroU8, NonZeroU32, NonZeroU64},
    time::Duration,
};

use ::serde::de::DeserializeOwned;
use cometbft_types::CometbftHeight;
use jsonrpsee::{
    core::{
        client::{BatchResponse, ClientT},
        params::BatchRequestBuilder,
        traits::ToRpcParams,
    },
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
    ws_client::{PingConfig, WsClientBuilder},
};
use tracing::{Instrument, debug, debug_span, instrument, trace};
use unionlabs::{ErrorReporter, bounded::BoundedU8, primitives::H256, result_unwrap};

use crate::rpc_types::{
    AbciInfoResponse, AbciQueryResponse, AllValidatorsResponse, BlockResponse,
    BlockResultsResponse, BlockSearchResponse, BlockchainResponse, BroadcastTxSyncResponse,
    CommitResponse, GenesisResponse, GrpcAbciQueryResponse, HeaderResponse, Order, StatusResponse,
    TxResponse, TxSearchResponse, ValidatorsPagination, ValidatorsResponse,
};

#[cfg(test)]
mod tests;

pub mod rpc_types;
pub mod serde;
pub use cometbft_types as types;

pub type JsonRpcError = jsonrpsee::core::client::Error;

#[derive(Debug, Clone)]
pub struct Client {
    inner: ClientInner,
}

impl Client {
    pub async fn new(url: impl AsRef<str>) -> Result<Self, JsonRpcError> {
        let url = url.as_ref().to_owned();

        let inner = match url.split_once("://") {
            Some(("ws" | "wss", _)) => {
                let client = reconnecting_jsonrpc_ws_client::Client::new(move || {
                    WsClientBuilder::default()
                        .enable_ws_ping(PingConfig::new())
                        .build(url.clone())
                        .instrument(debug_span!("cometbft_rpc_client", %url))
                });

                // TODO: Config
                client
                    .wait_until_connected(Duration::from_secs(5))
                    .await
                    .map_err(|e| JsonRpcError::Custom(e.to_string()))?;

                ClientInner::Ws(client)
            }
            Some(("http" | "https", _)) => {
                return Self::on_http(
                    HttpClientBuilder::default()
                        .max_response_size(100 * 1024 * 1024)
                        .build(url)?,
                );
            }
            _ => return Err(JsonRpcError::Custom(format!("invalid url {url}"))),
        };

        Ok(Self { inner })
    }

    pub fn on_http(client: HttpClient) -> Result<Self, JsonRpcError> {
        Ok(Self {
            inner: ClientInner::Http(Box::new(client)),
        })
    }

    // TODO: This should be bounded correctly
    pub async fn commit(
        &self,
        height: Option<CometbftHeight>,
    ) -> Result<CommitResponse, JsonRpcError> {
        self.inner
            .request("commit", (height.map(|x| x.to_string()),))
            .await
    }

    pub async fn header(
        &self,
        height: Option<CometbftHeight>,
    ) -> Result<HeaderResponse, JsonRpcError> {
        self.inner
            .request("header", (height.map(|x| x.to_string()),))
            .await
    }

    pub async fn validators(
        &self,
        height: Option<CometbftHeight>,
        pagination: Option<ValidatorsPagination>,
    ) -> Result<ValidatorsResponse, JsonRpcError> {
        self.inner
            .request(
                "validators",
                (
                    height.map(|x| x.to_string()),
                    pagination.map(|x| x.page).map(|x| x.to_string()),
                    pagination.and_then(|x| x.per_page).map(|x| x.to_string()),
                ),
            )
            .await
    }

    /// Auto-paginated version of [`Self::validators`].
    pub async fn all_validators(
        &self,
        mut height: Option<CometbftHeight>,
    ) -> Result<AllValidatorsResponse, JsonRpcError> {
        const PER_PAGE: BoundedU8<1, 100> =
            const { result_unwrap!(BoundedU8::<1, 100>::new_const(100)) };

        let mut page = const { NonZeroU64::new(1).unwrap() };

        let mut out = vec![];

        loop {
            let ValidatorsResponse {
                block_height,
                validators,
                count: _,
                total,
            } = self
                .validators(
                    height,
                    Some(rpc_types::ValidatorsPagination {
                        page,
                        per_page: Some(PER_PAGE),
                    }),
                )
                .await?;

            out.extend(validators);

            height.get_or_insert(block_height);

            if out.len() as u64 >= total {
                return Ok(AllValidatorsResponse {
                    block_height: height.expect("height was just inserted into; qed;"),
                    validators: out,
                });
            }

            page = page
                .checked_add(1)
                .expect("validator count will always be < u64 max");
        }
    }

    #[instrument(skip_all, fields())]
    pub async fn abci_info(&self) -> Result<AbciInfoResponse, JsonRpcError> {
        debug!("fetching abci info");

        let res: AbciInfoResponse = self.inner.request("abci_info", rpc_params!()).await?;

        debug!(
            data = %res.response.data,
            version = %res.response.version,
            last_block_height = %res.response.last_block_height,
            last_block_app_hash = %res.response.last_block_app_hash,
            "fetched abci info"
        );

        Ok(res)
    }

    // would be cool to somehow have this be generic and do decoding automatically
    #[instrument(
        skip_all,
        fields(
            path = %path.as_ref(),
            height = %height.map(|x| x.to_string()).as_deref().unwrap_or(""),
            %prove,
        )
    )]
    pub async fn abci_query(
        &self,
        path: impl AsRef<str>,
        data: impl AsRef<[u8]>,
        height: Option<CometbftHeight>,
        prove: bool,
    ) -> Result<AbciQueryResponse, JsonRpcError> {
        trace!(data = %::serde_utils::to_hex(data.as_ref()), "data");
        debug!("fetching abci query");

        let res: AbciQueryResponse = self
            .inner
            // the rpc needs an un-prefixed hex string
            .request(
                "abci_query",
                (
                    path.as_ref(),
                    hex::encode(data),
                    height.map(|x| x.to_string()),
                    prove,
                ),
            )
            .await?;

        debug!(
            code = %res.response.code,
            log = %res.response.log,
            info = %res.response.info,
            index = %res.response.index,
            key = ?&res.response.key,
            has_value = res.response.value.is_some(),
            height = %res.response.height,
            codespace = %res.response.codespace,
            proof_ops = ?res.response.proof_ops,
            "fetched abci query"
        );
        trace!(value = ?&res.response.value, "value");

        Ok(res)
    }

    // would be cool to somehow have this be generic and do decoding automatically
    #[instrument(
        skip_all,
        fields(
            path = %path.as_ref(),
            // ?data,
            height = %height.map(|x| x.to_string()).as_deref().unwrap_or(""),
            %prove,
        )
    )]
    pub async fn grpc_abci_query<
        Q: unionlabs::prost::Message,
        R: unionlabs::prost::Message + Default,
    >(
        &self,
        path: impl AsRef<str>,
        data: &Q,
        height: Option<CometbftHeight>,
        prove: bool,
    ) -> Result<GrpcAbciQueryResponse<R>, JsonRpcError> {
        debug!("fetching grpc abci query");

        let res = self
            .abci_query(path, data.encode_to_vec(), height, prove)
            .await?
            .response;

        Ok(GrpcAbciQueryResponse {
            code: res.code,
            log: res.log,
            info: res.info,
            index: res.index,
            key: res.key,
            value: res
                .value
                .map(|value| R::decode(&*value))
                .transpose()
                .map_err(|e| JsonRpcError::Custom(ErrorReporter(e).to_string()))?,
            proof_ops: res.proof_ops,
            height: res.height,
            codespace: res.codespace,
        })
    }

    pub async fn status(&self) -> Result<StatusResponse, JsonRpcError> {
        self.inner.request("status", rpc_params!()).await
    }

    pub async fn block(
        &self,
        height: Option<CometbftHeight>,
    ) -> Result<BlockResponse, JsonRpcError> {
        self.inner
            .request("block", (height.map(|x| x.to_string()),))
            .await
    }

    pub async fn block_by_hash(&self, hash: H256) -> Result<BlockResponse, JsonRpcError> {
        self.inner
            .request("block_by_hash", (hash.to_string(),))
            .await
    }

    pub async fn blockchain(
        &self,
        min_height: NonZeroU64,
        max_height: NonZeroU64,
    ) -> Result<BlockchainResponse, JsonRpcError> {
        self.inner
            .request(
                "blockchain",
                (min_height.to_string(), max_height.to_string()),
            )
            .await
    }

    #[instrument(
        skip_all,
        fields(
            query = query.as_ref(),
            prove,
            page,
            per_page,
            ?order_by
        )
    )]
    pub async fn tx_search(
        &self,
        query: impl AsRef<str>,
        prove: bool,
        page: NonZeroU32,
        // REVIEW: Is this bounded in the same way as the validators pagination?
        per_page: NonZeroU8,
        // REVIEW: There is the enum `cosmos.tx.v1beta.OrderBy`, is that related to this?
        order_by: Order,
    ) -> Result<TxSearchResponse, JsonRpcError> {
        let response = self
            .inner
            .request::<TxSearchResponse, _>(
                "tx_search",
                rpc_params![
                    query.as_ref(),
                    prove,
                    page.to_string(),
                    per_page.to_string(),
                    order_by
                ],
            )
            .await?;

        debug!(total_count = response.total_count, "tx_search");

        Ok(response)
    }

    #[instrument(
        skip_all,
        fields(
            query = query.as_ref(),
            prove,
            page,
            per_page,
            ?order_by
        )
    )]
    pub async fn block_search(
        &self,
        query: impl AsRef<str>,
        page: NonZeroU32,
        // REVIEW: Is this bounded in the same way as the validators pagination?
        per_page: NonZeroU8,
        // REVIEW: There is the enum `cosmos.tx.v1beta.OrderBy`, is that related to this?
        order_by: Order,
    ) -> Result<BlockSearchResponse, JsonRpcError> {
        let response = self
            .inner
            .request::<BlockSearchResponse, _>(
                "block_search",
                rpc_params![
                    query.as_ref(),
                    page.to_string(),
                    per_page.to_string(),
                    order_by
                ],
            )
            .await?;

        debug!(total_count = response.total_count, "block_search");

        Ok(response)
    }

    // TODO: support order_by
    pub async fn tx(&self, hash: H256, prove: bool) -> Result<TxResponse, JsonRpcError> {
        use base64::prelude::*;

        self.inner
            .request("tx", rpc_params![BASE64_STANDARD.encode(hash), prove])
            .await
    }

    pub async fn broadcast_tx_sync(
        &self,
        tx: &[u8],
    ) -> Result<BroadcastTxSyncResponse, JsonRpcError> {
        use base64::prelude::*;

        self.inner
            .request("broadcast_tx_sync", rpc_params![BASE64_STANDARD.encode(tx)])
            .await
    }

    pub async fn block_results(
        &self,
        height: Option<CometbftHeight>,
    ) -> Result<BlockResultsResponse, JsonRpcError> {
        self.inner
            .request("block_results", rpc_params![height.map(|x| x.to_string())])
            .await
    }

    pub async fn genesis<AppState: DeserializeOwned>(
        &self,
    ) -> Result<GenesisResponse<AppState>, JsonRpcError> {
        self.inner.request("genesis", rpc_params![]).await
    }
}

#[derive(Debug, Clone)]
enum ClientInner {
    Http(Box<HttpClient>),
    Ws(reconnecting_jsonrpc_ws_client::Client),
}

impl ClientT for ClientInner {
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), JsonRpcError>
    where
        Params: ToRpcParams + Send,
    {
        match self {
            ClientInner::Http(client) => client.notification(method, params).await,
            ClientInner::Ws(client) => client.notification(method, params).await,
        }
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, JsonRpcError>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        match self {
            ClientInner::Http(client) => client.request(method, params).await,
            ClientInner::Ws(client) => client.request(method, params).await,
        }
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, JsonRpcError>
    where
        R: DeserializeOwned + fmt::Debug + 'a,
    {
        match self {
            ClientInner::Http(client) => client.batch_request(batch).await,
            ClientInner::Ws(client) => client.batch_request(batch).await,
        }
    }
}

// These tests are useful in testing and debugging, but should not be run in CI
// #[cfg(test)]
// mod live_tests {
//     use hex_literal::hex;

//     use super::*;

//     const UNION_TESTNET: &str = "https://rpc.testnet-9.union.build";
//     const BERACHAIN_DEVNET: &str = "ws://localhost:26657/websocket";
//     const BERACHAIN_TESTNET: &str = "wss://bartio-cosmos.berachain.com/websocket";
//     const OSMOSIS_TESTNET: &str = "wss://osmosis-rpc.publicnode.com/websocket";
//     const BABYLON_TESTNET: &str = "https://rpc.bbn-test-5.babylon.chain.kitchen";

//     const TEST_URL: &str = UNION_TESTNET;

//     #[tokio::test]
//     async fn commit() {
//         let client = Client::new(TEST_URL).await.unwrap();

//         let result = client.commit(Some(1.try_into().unwrap())).await;

//         dbg!(result);
//     }

//     #[tokio::test]
//     async fn abci_query() {
//         // let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new("https://rpc.pacific-1.sei.io").await.unwrap();

//         let result = client
//             .abci_query(
//                 "store/evm/key",
//                 &[
//                     [0x03].as_slice(),
//                     &hex!("4a4d9abD36F923cBA0Af62A39C01dEC2944fb638"),
//                     &hex!("0000000000000000000000000000000000000000000000000000000000000000"),
//                 ]
//                 .into_iter()
//                 .flatten()
//                 .copied()
//                 .collect::<Vec<_>>(),
//                 Some(142070066.try_into().unwrap()),
//                 true,
//             )
//             .await;

//         dbg!(result);
//     }

//     #[tokio::test]
//     async fn validators() {
//         // let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(TEST_URL).await.unwrap();

//         // let result = client
//         //     .validators(
//         //         Some(100.try_into().unwrap()),
//         //         Some(ValidatorsPagination {
//         //             page: 1.try_into().unwrap(),
//         //             per_page: None,
//         //         }),
//         //     )
//         //     .await;

//         let result = client.all_validators(None).await.unwrap();

//         dbg!(result.validators.len(),);

//         println!(
//             "{}",
//             serde_json::to_string_pretty(&result.validators).unwrap()
//         );
//     }

//     #[tokio::test]
//     async fn status() {
//         // let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(TEST_URL).await.unwrap();

//         let result = client.status().await.unwrap();

//         dbg!(result);
//     }

//     #[tokio::test]
//     async fn block() {
//         // let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(TEST_URL).await.unwrap();

//         // let mut i = 1376377;

//         // loop {
//         //     dbg!(i);

//         let result = client
//             .block(Some(1.try_into().unwrap()))
//             // .block(None)
//             .await
//             .unwrap();

//         dbg!(result.block);

//         //     i += 1;

//         //     tokio::time::sleep(Duration::from_millis(100)).await;
//         // }
//     }

//     #[tokio::test]
//     async fn tx() {
//         // let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(TEST_URL).await.unwrap();

//         let result = client
//             .tx(
//                 hex!("32DAD1842DF0441870B168D0C177F8EEC156B18B32D88C3658349BE07F352CCA").into(),
//                 true,
//             )
//             .await
//             .unwrap();

//         dbg!(result);
//     }
// }
