use std::{fmt::Debug, num::NonZeroU64, sync::Arc};

use jsonrpsee::{
    core::client::ClientT,
    rpc_params,
    ws_client::{WsClient, WsClientBuilder},
};
use serde::{de, Deserialize, Deserializer, Serialize};
use tracing::debug;
use unionlabs::{
    bounded::BoundedU8,
    google::protobuf::timestamp::Timestamp,
    hash::{H160, H256},
    option_unwrap, result_unwrap,
    tendermint::{
        abci::response_query::ResponseQuery,
        crypto::public_key::PublicKey,
        p2p::default_node_info::DefaultNodeInfo,
        types::{
            block::Block, block_id::BlockId, signed_header::SignedHeader, validator::Validator,
        },
    },
};

#[derive(Debug, Clone)]
pub struct Client {
    client: Arc<WsClient>,
}

pub type JsonRpcError = jsonrpsee::core::client::Error;

impl Client {
    pub async fn new(url: impl AsRef<str>) -> Result<Self, JsonRpcError> {
        Ok(Self {
            client: Arc::new(WsClientBuilder::default().build(url).await?),
        })
    }

    pub async fn commit(&self, height: Option<NonZeroU64>) -> Result<CommitResponse, JsonRpcError> {
        self.client
            .request("commit", (height.map(|x| x.to_string()),))
            .await
    }

    pub async fn validators(
        &self,
        height: Option<NonZeroU64>,
        pagination: Option<ValidatorsPagination>,
    ) -> Result<ValidatorsResponse, JsonRpcError> {
        self.client
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
    pub async fn abci_query(
        &self,
        path: impl AsRef<str>,
        data: impl AsRef<[u8]>,
        height: Option<NonZeroU64>,
        prove: bool,
    ) -> Result<AbciQueryResponse, JsonRpcError> {
        let path = path.as_ref();
        let height = height.map(|x| x.to_string());

        debug!(
            %path,
            data = %::serde_utils::to_hex(data.as_ref()),
            height = %height.as_deref().unwrap_or(""),
            %prove,
            "fetching abci query"
        );

        let res: AbciQueryResponse = self
            .client
            // the rpc needs an un-prefixed hex string
            .request("abci_query", (path, hex::encode(data), height, prove))
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

    // would be cool to somehow have this be generic and do decoding automatically
    pub async fn status(&self) -> Result<StatusResponse, JsonRpcError> {
        self.client.request("status", rpc_params!()).await
    }

    pub async fn block(&self, height: Option<NonZeroU64>) -> Result<BlockResponse, JsonRpcError> {
        self.client
            .request("block", (height.map(|x| x.to_string()),))
            .await
    }
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
    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    earliest_app_hash: H256,
    #[serde(with = "::serde_utils::hex_allow_unprefixed")]
    earliest_block_hash: H256,
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

//     use super::*;

//     const UNION_TESTNET: &'static str = "wss://rpc.testnet.bonlulu.uno/websocket";
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

//         dbg!(result);
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
// }
