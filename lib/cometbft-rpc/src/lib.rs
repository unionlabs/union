use core::fmt;
use std::{
    fmt::Debug,
    num::{NonZeroU32, NonZeroU64, NonZeroU8},
    sync::Arc,
};

use jsonrpsee::{
    core::{
        async_trait,
        client::{BatchResponse, ClientT},
        params::BatchRequestBuilder,
        traits::ToRpcParams,
    },
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
    ws_client::{PingConfig, WsClientBuilder},
};
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};
use tracing::{debug, instrument};
use unionlabs::{
    bounded::{BoundedI64, BoundedU8},
    google::protobuf::timestamp::Timestamp,
    hash::{H160, H256},
    option_unwrap, result_unwrap,
    tendermint::{
        abci::{exec_tx_result::ExecTxResult, response_query::ResponseQuery},
        crypto::public_key::PublicKey,
        p2p::default_node_info::DefaultNodeInfo,
        types::{
            block::Block, block_id::BlockId, signed_header::SignedHeader, tx_proof::TxProof,
            validator::Validator,
        },
    },
};

pub type JsonRpcError = jsonrpsee::core::client::Error;

#[derive(Debug, Clone)]
pub struct Client {
    inner: ClientInner,
}

impl Client {
    pub async fn new(url: impl AsRef<str>) -> Result<Self, JsonRpcError> {
        let inner = match url.as_ref().split_once("://") {
            Some(("ws" | "wss", _)) => ClientInner::Ws(Arc::new(
                WsClientBuilder::default()
                    .enable_ws_ping(PingConfig::new())
                    .build(url)
                    .await?,
            )),
            Some(("http" | "https", _)) => {
                ClientInner::Http(HttpClientBuilder::default().build(url)?)
            }
            _ => {
                return Err(JsonRpcError::Custom(format!(
                    "invalid url {}",
                    url.as_ref()
                )))
            }
        };

        Ok(Self { inner })
    }

    pub async fn commit(&self, height: Option<NonZeroU64>) -> Result<CommitResponse, JsonRpcError> {
        self.inner
            .request("commit", (height.map(|x| x.to_string()),))
            .await
    }

    pub async fn validators(
        &self,
        height: Option<NonZeroU64>,
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
        mut height: Option<NonZeroU64>,
    ) -> Result<AllValidatorsResponse, JsonRpcError> {
        const PER_PAGE: BoundedU8<1, 100> = const { result_unwrap!(BoundedU8::<1, 100>::new(100)) };

        let mut page = const { option_unwrap!(NonZeroU64::new(1)) };

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
                    Some(ValidatorsPagination {
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

    // would be cool to somehow have this be generic and do decoding automatically
    #[instrument(
        skip_all,
        fields(
            path = %path.as_ref(),
            data = %::serde_utils::to_hex(data.as_ref()),
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
            key = %::serde_utils::to_hex(&res.response.key),
            value = %::serde_utils::to_hex(&res.response.value),
            height = %res.response.height,
            codespace = %res.response.codespace,
            "fetched abci query"
        );

        Ok(res)
    }

    pub async fn status(&self) -> Result<StatusResponse, JsonRpcError> {
        self.inner.request("status", rpc_params!()).await
    }

    pub async fn block(&self, height: Option<NonZeroU64>) -> Result<BlockResponse, JsonRpcError> {
        self.inner
            .request("block", (height.map(|x| x.to_string()),))
            .await
    }

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
        self.inner
            .request(
                "tx_search",
                rpc_params![
                    query.as_ref(),
                    prove,
                    page.to_string(),
                    per_page.to_string(),
                    order_by
                ],
            )
            .await
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
    ) -> Result<TxBroadcastSyncResponse, JsonRpcError> {
        use base64::prelude::*;

        self.inner
            .request("broadcast_tx_sync", rpc_params![BASE64_STANDARD.encode(tx)])
            .await
    }

    // pub async fn block_results(
    //     &self,
    //     height: Option<BoundedI64<1>>,
    // ) -> Result<TxSearchResponse, JsonRpcError> {
    //     self.client
    //         .request("block_results", rpc_params![height.map(|x| x.to_string())])
    //         .await
    // }
}

#[derive(Debug, Clone)]
enum ClientInner {
    Http(HttpClient),
    Ws(Arc<jsonrpsee::core::client::Client>),
}

#[async_trait]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::types::BlockId, _>")]
    pub block_id: BlockId,
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::types::Block, _>")]
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::p2p::DefaultNodeInfo, _>")]
    pub node_info: DefaultNodeInfo,
    pub sync_info: SyncInfo,
    pub validator_info: ValidatorInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncInfo {
    catching_up: bool,
    #[serde(with = "::serde_utils::hex_allow_unprefixed_maybe_empty")]
    earliest_app_hash: Option<H256>,
    #[serde(with = "::serde_utils::hex_allow_unprefixed_maybe_empty")]
    earliest_block_hash: Option<H256>,
    #[serde(with = "::serde_utils::string")]
    earliest_block_height: u64,
    earliest_block_time: Timestamp,
    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    latest_app_hash: H256,
    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    latest_block_hash: H256,
    #[serde(with = "::serde_utils::string")]
    latest_block_height: u64,
    latest_block_time: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorInfo {
    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    pub address: H160,
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::crypto::PublicKey, _>")]
    pub pub_key: PublicKey,
    // REVIEW: is this bounded the same way as Validator?
    #[serde(with = "::serde_utils::string")]
    pub voting_power: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorsResponse {
    #[serde(with = "::serde_utils::string")]
    pub block_height: NonZeroU64,
    #[serde(deserialize_with = "serde_as_list::<_, protos::tendermint::types::Validator, _>")]
    pub validators: Vec<Validator>,
    #[serde(with = "::serde_utils::string")]
    pub count: u64,
    #[serde(with = "::serde_utils::string")]
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllValidatorsResponse {
    pub block_height: NonZeroU64,
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatorsPagination {
    page: NonZeroU64,
    // :]
    per_page: Option<BoundedU8<1, 100>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciQueryResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::abci::ResponseQuery, _>")]
    pub response: ResponseQuery,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::types::SignedHeader, _>")]
    pub signed_header: SignedHeader,
    pub canonical: bool,
}

#[derive(macros::Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxResponse {
    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    pub hash: H256,
    // review: is this really optional?
    #[serde(with = "::serde_utils::string_opt")]
    pub height: Option<NonZeroU64>,
    pub index: u32,
    #[serde(deserialize_with = "serde_as::<_, protos::tendermint::abci::ExecTxResult, _>")]
    pub tx_result: ExecTxResult,
    #[serde(with = "::serde_utils::base64")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub tx: Vec<u8>,
    #[serde(
        default,
        deserialize_with = "serde_as_opt::<_, protos::tendermint::types::TxProof, _>"
    )]
    pub proof: Option<TxProof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxSearchResponse {
    pub txs: Vec<TxResponse>,
    #[serde(with = "::serde_utils::string")]
    pub total_count: u32,
}

#[derive(macros::Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxBroadcastSyncResponse {
    pub codespace: String,

    pub code: u32,

    #[serde(with = "::serde_utils::base64")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,

    pub log: String,

    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    pub hash: H256,
}

pub fn serde_as<'de, D, Src, Dst>(deserializer: D) -> Result<Dst, D::Error>
where
    D: Deserializer<'de>,
    Src: Deserialize<'de>,
    Src: TryInto<Dst, Error: Debug>,
{
    Src::deserialize(deserializer)?
        .try_into()
        .map_err(|e| de::Error::custom(format!("{e:?}")))
}

pub fn serde_as_opt<'de, D, Src, Dst>(deserializer: D) -> Result<Option<Dst>, D::Error>
where
    D: Deserializer<'de>,
    Src: Deserialize<'de>,
    Src: TryInto<Dst, Error: Debug>,
{
    <Option<Src>>::deserialize(deserializer)?
        .map(|src| {
            src.try_into()
                .map_err(|e| de::Error::custom(format!("{e:?}")))
        })
        .transpose()
}

pub fn serde_as_list<'de, D, Src, Dst>(deserializer: D) -> Result<Vec<Dst>, D::Error>
where
    D: Deserializer<'de>,
    Src: Deserialize<'de>,
    Src: TryInto<Dst, Error: Debug>,
{
    <Vec<Src>>::deserialize(deserializer)?
        .into_iter()
        .map(|x| {
            x.try_into()
                .map_err(|e| de::Error::custom(format!("{e:?}")))
        })
        .collect()
}

// These tests are useful in testing and debugging, but should not be run in CI
// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use hex_literal::hex;

//     use super::*;

//     const UNION_TESTNET: &'static str = "wss://rpc.testnet-8.union.build/websocket";
//     const BERACHAIN_DEVNET: &'static str = "ws://localhost:26657/websocket";
//     const BERACHAIN_TESTNET: &'static str = "wss://bartio-cosmos.berachain-devnet.com/websocket";
//     const OSMOSIS_TESTNET: &'static str = "wss://osmosis-rpc.publicnode.com/websocket";

//     #[tokio::test]
//     async fn commit() {
//         let client = Client::new(BERACHAIN_DEVNET).await.unwrap();

//         let result = client.commit(Some(1.try_into().unwrap())).await;

//         dbg!(result);
//     }

//     #[tokio::test]
//     async fn abci_query() {
//         let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(BERACHAIN_DEVNET).await.unwrap();

//         let result = client
//             .abci_query(
//                 "store/beacon/key",
//                 &[0x11],
//                 Some(3358.try_into().unwrap()),
//                 true,
//             )
//             .await;

//         bg!(result);
//     }

//     #[tokio::test]
//     async fn validators() {
//         let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(BERACHAIN_DEVNET).await.unwrap();

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
//         let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(BERACHAIN_DEVNET).await.unwrap();

//         let result = client.status().await.unwrap();

//         dbg!(result);
//     }

//     #[tokio::test]
//     async fn block() {
//         let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(BERACHAIN_TESTNET).await.unwrap();

//         let mut i = 1376377;

//         loop {
//             dbg!(i);

//             let result = client
//                 .block(Some(i.try_into().unwrap()))
//                 // .block(None)
//                 .await
//                 .unwrap();

//             dbg!(result.block.evidence);

//             i += 1;

//             tokio::time::sleep(Duration::from_millis(100)).await;
//         }
//     }

//     #[tokio::test]
//     async fn tx() {
//         let _ = tracing_subscriber::fmt().try_init();

//         let client = Client::new(UNION_TESTNET).await.unwrap();

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
