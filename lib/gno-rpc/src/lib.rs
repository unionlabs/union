use core::fmt;
use std::{
    fmt::{Debug, Display},
    time::Duration,
};

use ::serde::de::DeserializeOwned;
use base64::{Engine, prelude::BASE64_STANDARD};
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
use unionlabs::{
    ErrorReporter,
    bounded::BoundedI64,
    primitives::{Bech32, Bytes, H160, H256, encoding::Base64},
};

use crate::rpc_types::{
    AbciAccount, AbciInfoResponse, AbciQueryResponse, BlockResponse, BlockResultsResponse,
    BroadcastTxCommitResult, CommitResponse, StatusResponse, TxResponse, ValidatorsResponse,
};

#[cfg(test)]
mod tests;

pub mod rpc_types;
pub use gno_types as types;

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
                        .instrument(debug_span!("gno_rpc_client", %url))
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
    // NOTE: For some reason, the jsonrpc doesn't work the same as the rest api
    //       height is required when using jsonrpc, but if omitted when using the rest api, the latest commit is returned
    pub async fn commit(&self, height: BoundedI64<0>) -> Result<CommitResponse, JsonRpcError> {
        self.inner.request("commit", (height.to_string(),)).await
    }

    pub async fn validators(
        &self,
        height: BoundedI64<0>,
    ) -> Result<ValidatorsResponse, JsonRpcError> {
        self.inner
            .request("validators", (height.to_string(),))
            .await
    }

    #[instrument(skip_all, fields())]
    pub async fn abci_info(&self) -> Result<AbciInfoResponse, JsonRpcError> {
        debug!("fetching abci info");

        let res: AbciInfoResponse = self.inner.request("abci_info", rpc_params!()).await?;

        debug!(
            data = ?res.response.response_base.data,
            abci_version = %res.response.abci_version,
            app_version = %res.response.app_version,
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
            data = %String::from_utf8_lossy(data.as_ref()),
            height = %height.map(|x| x.to_string()).as_deref().unwrap_or(""),
            %prove,
        )
    )]
    pub async fn abci_query(
        &self,
        path: impl AsRef<str>,
        data: impl AsRef<[u8]>,
        height: Option<BoundedI64<1>>,
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
                    BASE64_STANDARD.encode(data),
                    height.map(|x| x.to_string()),
                    prove,
                ),
            )
            .await?;

        debug!(
            log = %res.response.response_base.log,
            info = %res.response.response_base.info,
            key = ?&res.response.key,
            has_value = res.response.value.is_some(),
            height = %res.response.height,
            error = ?res.response.response_base.error,
            "fetched abci query"
        );
        trace!(value = ?&res.response.value, "value");

        Ok(res)
    }

    #[instrument(
        skip_all,
        fields(
            realm = %realm.as_ref(),
            %query,
            height = %height.map(|x| x.to_string()).as_deref().unwrap_or(""),
        )
    )]
    pub async fn render_query(
        &self,
        realm: impl AsRef<str>,
        query: impl Display,
        height: Option<BoundedI64<1>>,
    ) -> Result<Option<Bytes<Base64>>, JsonRpcError> {
        debug!("fetching render query");

        let res = self
            .abci_query(
                "vm/qrender",
                format!("{}:{query}", realm.as_ref()),
                height,
                false,
            )
            .await?
            .response;

        Ok(res.value)
    }

    #[instrument(
        skip_all,
        fields(
            realm = %realm.as_ref(),
            %query,
            height = %height.map(|x| x.to_string()).as_deref().unwrap_or(""),
        )
    )]
    pub async fn eval_query(
        &self,
        realm: impl AsRef<str>,
        query: impl Display,
        height: Option<BoundedI64<1>>,
    ) -> Result<Option<Bytes<Base64>>, JsonRpcError> {
        debug!("fetching eval query");

        let res = self
            .abci_query(
                "vm/qeval",
                format!("{}.{query}", realm.as_ref()),
                height,
                false,
            )
            .await?
            .response;

        if let Some(error) = res.response_base.error {
            // TODO: Don't reuse the jsonrpc error type here
            return Err(JsonRpcError::Custom(ErrorReporter(error).to_string()));
        }

        Ok(res.response_base.data)
    }

    pub async fn status(
        &self,
        height_gte: Option<BoundedI64<0>>,
    ) -> Result<StatusResponse, JsonRpcError> {
        self.inner
            .request("status", (height_gte.map(|x| x.to_string()),))
            .await
    }

    pub async fn block(&self, height: BoundedI64<1>) -> Result<BlockResponse, JsonRpcError> {
        self.inner.request("block", (height.to_string(),)).await
    }

    // pub async fn blockchain(
    //     &self,
    //     min_height: NonZeroU64,
    //     max_height: NonZeroU64,
    // ) -> Result<BlockchainResponse, JsonRpcError> {
    //     self.inner
    //         .request(
    //             "blockchain",
    //             (min_height.to_string(), max_height.to_string()),
    //         )
    //         .await
    // }

    pub async fn tx(&self, hash: H256<Base64>) -> Result<TxResponse, JsonRpcError> {
        use base64::prelude::*;

        self.inner
            .request("tx", rpc_params![BASE64_STANDARD.encode(hash)])
            .await
    }

    pub async fn broadcast_tx_commit(
        &self,
        tx: &[u8],
    ) -> Result<BroadcastTxCommitResult, JsonRpcError> {
        use base64::prelude::*;

        self.inner
            .request(
                "broadcast_tx_commit",
                rpc_params![BASE64_STANDARD.encode(tx)],
            )
            .await
    }

    pub async fn block_results(
        &self,
        height: BoundedI64<0>,
    ) -> Result<BlockResultsResponse, JsonRpcError> {
        self.inner
            .request("block_results", (height.to_string(),))
            .await
    }

    pub async fn account_info(
        &self,
        account: &Bech32<H160>,
    ) -> Result<Option<AbciAccount>, JsonRpcError> {
        debug!(%account, "fetching account");

        let Some(data) = self
            .abci_query(format!("auth/accounts/{account}"), &[], None, false)
            .await?
            .response
            .response_base
            .data
        else {
            return Ok(None);
        };

        let account = serde_json::from_slice::<AbciAccount>(&data)?;

        debug!(?account, "fetched account");

        Ok(Some(account))
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
