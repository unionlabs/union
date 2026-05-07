use std::{fmt::Display, num::NonZeroU64, str::FromStr};

use gno_types::{
    Block, BlockMeta, Event, InfoResponse, NodeInfo, PublicKey, QueryResponse, ResponseBase,
    SignedHeader, Validator,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    google::protobuf::timestamp::Timestamp,
    primitives::{Bech32, Bytes, H160, H256, encoding::Base64},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Order {
    Asc,
    Desc,
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        })
    }
}

impl FromStr for Order {
    type Err = InvalidOrder;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Order::Asc),
            "desc" => Ok(Order::Desc),
            _ => Err(InvalidOrder {}),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid order, must be either `asc` or `desc`")]
pub struct InvalidOrder {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResponse {
    pub block_meta: BlockMeta,
    pub block: Block,
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct BlockchainResponse {
//     #[serde(with = "::serde_utils::string")]
//     pub last_height: u64,
//     pub block_metas: Vec<BlockMeta>,
// }

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct BlockMeta {
//     pub block_id: BlockId,
//     #[serde(with = "::serde_utils::string")]
//     pub block_size: u64,
//     pub header: Header,
//     #[serde(with = "::serde_utils::string")]
//     pub num_txs: u64,
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusResponse {
    pub node_info: NodeInfo,
    pub sync_info: SyncInfo,
    pub validator_info: ValidatorInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncInfo {
    pub latest_block_hash: Option<H256<Base64>>,
    pub latest_app_hash: Option<H256<Base64>>,
    #[serde(with = "::serde_utils::string")]
    pub latest_block_height: u64,
    pub latest_block_time: Timestamp,
    pub catching_up: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorInfo {
    pub address: Bech32<H160>,
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
    pub validators: Vec<Validator>,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct AllValidatorsResponse {
//     pub block_height: NonZeroU64,
//     pub validators: Vec<Validator>,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct ValidatorsPagination {
//     pub page: NonZeroU64,
//     // :]
//     pub per_page: Option<BoundedU8<1, 100>>,
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciQueryResponse {
    pub response: QueryResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciInfoResponse {
    pub response: InfoResponse,
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct GrpcAbciQueryResponse<T> {
//     pub code: Code,
//     /// nondeterministic
//     pub log: String,
//     /// nondeterministic
//     pub info: String,
//     pub index: i64,
//     pub key: Option<Bytes<Base64>>,
//     pub value: Option<T>,
//     pub proof_ops: Option<ProofOps>,
//     pub height: BoundedI64<0, { i64::MAX }>,
//     pub codespace: String,
// }

// impl<R> GrpcAbciQueryResponse<R> {
//     pub fn into_result(self) -> Result<Option<R>, GrpcAbciQueryError> {
//         match self.code {
//             Code::Err(error_code) => Err(GrpcAbciQueryError {
//                 error_code,
//                 codespace: self.codespace,
//                 log: self.log,
//             }),
//             Code::Ok => Ok(self.value),
//         }
//     }
// }

// #[derive(Debug, Clone, thiserror::Error)]
// #[error("grpc abci query error: {error_code}, {codespace}: {log}")]
// pub struct GrpcAbciQueryError {
//     pub error_code: NonZeroU32,
//     pub codespace: String,
//     pub log: String,
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitResponse {
    pub signed_header: SignedHeader,
    pub canonical: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResultsResponse {
    #[serde(with = "::serde_utils::string")]
    pub height: u64,
    pub results: AbciResponses,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciResponses {
    pub deliver_tx: Option<Vec<DeliverTxResponse>>,
    pub end_block: EndBlockResponse,
    pub begin_block: BeginBlockResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeginBlockResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseCheckTx {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    /// nondeterministic
    #[serde(rename = "GasWanted", with = "::serde_utils::string")]
    pub gas_wanted: BoundedI64<0>,
    #[serde(rename = "GasUsed", with = "::serde_utils::string")]
    pub gas_used: BoundedI64<0>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliverTxResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    #[serde(rename = "GasWanted", with = "::serde_utils::string")]
    pub gas_wanted: BoundedI64<0>,
    #[serde(rename = "GasUsed", with = "::serde_utils::string")]
    pub gas_used: BoundedI64<0>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndBlockResponse {
    pub response_base: ResponseBase,
    #[serde(rename = "ValidatorUpdates")]
    pub validator_updates: Option<Vec<ValidatorUpdate>>,
    #[serde(rename = "ConsensusParams")]
    pub consensus_params: Option<ConsensusParams>,
    #[serde(rename = "Events")]
    pub events: Option<Vec<Event>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorUpdate {
    #[serde(rename = "Address")]
    pub address: Bech32<H160>,
    #[serde(rename = "PubKey")]
    pub pub_key: Bytes<Base64>,
    #[serde(rename = "Power", with = "::serde_utils::string")]
    pub power: i64,
}

// #[derive(macros::Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct BroadcastTxSyncResponse {
//     pub codespace: String,
//     pub code: Code,
//     pub data: Bytes<Base64>,
//     pub log: String,
//     pub hash: H256<HexUnprefixed>,
// }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusParams {
    #[serde(rename = "Block")]
    pub block: Option<BlockParams>,
    #[serde(rename = "Validator")]
    pub validator: Option<ValidatorParams>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockParams {
    #[serde(rename = "MaxTxBytes", with = "::serde_utils::string")]
    pub max_tx_bytes: BoundedI64<1>,
    #[serde(rename = "MaxDataBytes", with = "::serde_utils::string")]
    pub max_data_bytes: BoundedI64<1>,
    #[serde(rename = "MaxBlockBytes", with = "::serde_utils::string")]
    pub max_block_bytes: BoundedI64<1>,
    #[serde(rename = "MaxGas", with = "::serde_utils::string")]
    pub max_gas: BoundedI64<{ -1 }>,
    #[serde(rename = "TimeIotaMS", with = "::serde_utils::string")]
    pub time_iota_ms: BoundedI64<1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorParams {
    #[serde(rename = "PubKeyTypeURLs")]
    pub pub_key_type_urls: Vec<String>,
}
