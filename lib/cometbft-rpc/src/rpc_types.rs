use std::num::{NonZeroU32, NonZeroU64};

use cometbft_types::{
    abci::{event::Event, exec_tx_result::ExecTxResult, response_query::QueryResponse},
    code::Code,
    crypto::{proof_ops::ProofOps, public_key::PublicKey},
    p2p::default_node_info::DefaultNodeInfo,
    types::{
        block::Block, block_id::BlockId, header::Header, signed_header::SignedHeader,
        tx_proof::TxProof, validator::Validator,
    },
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::{BoundedI64, BoundedU8},
    google::protobuf::timestamp::Timestamp,
    primitives::{
        encoding::{Base64, HexUnprefixed},
        Bytes, H160, H256,
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResponse {
    pub block_id: BlockId,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockchainResponse {
    #[serde(with = "::serde_utils::string")]
    pub last_height: u64,
    pub block_metas: Vec<BlockMeta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockMeta {
    pub block_id: BlockId,
    #[serde(with = "::serde_utils::string")]
    pub block_size: u64,
    pub header: Header,
    #[serde(with = "::serde_utils::string")]
    pub num_txs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusResponse {
    pub node_info: DefaultNodeInfo,
    pub sync_info: SyncInfo,
    pub validator_info: ValidatorInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncInfo {
    pub catching_up: bool,
    #[serde(with = "::cometbft_types::serde::maybe_empty_h256")]
    pub earliest_app_hash: Option<H256<HexUnprefixed>>,
    #[serde(with = "::cometbft_types::serde::maybe_empty_h256")]
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
    pub response: QueryResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcAbciQueryResponse<T> {
    pub code: Code,
    /// nondeterministic
    pub log: String,
    /// nondeterministic
    pub info: String,
    pub index: i64,
    pub key: Option<Bytes<Base64>>,
    pub value: Option<T>,
    pub proof_ops: Option<ProofOps>,
    pub height: BoundedI64<0, { i64::MAX }>,
    pub codespace: String,
}

impl<R> GrpcAbciQueryResponse<R> {
    pub fn into_result(self) -> Result<Option<R>, GrpcAbciQueryError> {
        match self.code {
            Code::Err(error_code) => Err(GrpcAbciQueryError {
                error_code,
                codespace: self.codespace,
                log: self.log,
            }),
            Code::Ok => Ok(self.value),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("grpc abci query error: {error_code}, {codespace}: {log}")]
pub struct GrpcAbciQueryError {
    pub error_code: NonZeroU32,
    pub codespace: String,
    pub log: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitResponse {
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
    pub tx_result: ExecTxResult,
    pub tx: Bytes<Base64>,
    #[serde(default)]
    pub proof: Option<TxProof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxSearchResponse {
    pub txs: Vec<TxResponse>,
    #[serde(with = "::serde_utils::string")]
    pub total_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockResultsResponse {
    #[serde(with = "::serde_utils::string")]
    pub height: u64,
    pub txs_results: Option<Vec<ExecTxResult>>,
    pub finalize_block_events: Option<Vec<Event>>,
}

#[derive(macros::Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BroadcastTxSyncResponse {
    pub codespace: String,
    pub code: Code,
    pub data: Bytes<Base64>,
    pub log: String,
    pub hash: H256<HexUnprefixed>,
}
