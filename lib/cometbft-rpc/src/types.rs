use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedU8,
    cometbft::{
        abci::{exec_tx_result::ExecTxResult, query_response::QueryResponse},
        crypto::public_key::PublicKey,
        p2p::default_node_info::DefaultNodeInfo,
        types::{
            block::Block, block_id::BlockId, signed_header::SignedHeader, tx_proof::TxProof,
            validator::Validator,
        },
    },
    google::protobuf::timestamp::Timestamp,
    hash::{hash_v2::HexUnprefixed, H160, H256},
};

use crate::serde::{serde_as, serde_as_list, serde_as_opt};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::types::v1::BlockId, _>")]
    pub block_id: BlockId,
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::types::v1::Block, _>")]
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::p2p::v1::DefaultNodeInfo, _>")]
    pub node_info: DefaultNodeInfo,
    pub sync_info: SyncInfo,
    pub validator_info: ValidatorInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncInfo {
    pub catching_up: bool,
    pub earliest_app_hash: Option<H256<HexUnprefixed>>,
    pub earliest_block_hash: Option<H256<HexUnprefixed>>,
    #[serde(with = "::serde_utils::string")]
    pub earliest_block_height: u64,
    pub earliest_block_time: Timestamp,
    pub latest_app_hash: H256<HexUnprefixed>,
    pub latest_block_hash: H256<HexUnprefixed>,
    #[serde(with = "::serde_utils::string")]
    pub latest_block_height: u64,
    pub latest_block_time: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorInfo {
    pub address: H160<HexUnprefixed>,
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::crypto::v1::PublicKey, _>")]
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
    #[serde(deserialize_with = "serde_as_list::<_, protos::cometbft::types::v1::Validator, _>")]
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
    pub page: NonZeroU64,
    // :]
    pub per_page: Option<BoundedU8<1, 100>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciQueryResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::abci::v1::QueryResponse, _>")]
    pub response: QueryResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitResponse {
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::types::v1::SignedHeader, _>")]
    pub signed_header: SignedHeader,
    pub canonical: bool,
}

#[derive(macros::Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxResponse {
    pub hash: H256<HexUnprefixed>,
    // review: is this really optional?
    #[serde(with = "::serde_utils::string_opt")]
    pub height: Option<NonZeroU64>,
    pub index: u32,
    #[serde(deserialize_with = "serde_as::<_, protos::cometbft::abci::v1::ExecTxResult, _>")]
    pub tx_result: ExecTxResult,
    #[serde(with = "::serde_utils::base64")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub tx: Vec<u8>,
    #[serde(
        default,
        deserialize_with = "serde_as_opt::<_, protos::cometbft::types::v1::TxProof, _>"
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
pub struct BroadcastTxSyncResponse {
    pub codespace: String,

    pub code: u32,

    #[serde(with = "::serde_utils::base64")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,

    pub log: String,

    pub hash: H256<HexUnprefixed>,
}
