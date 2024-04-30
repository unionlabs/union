use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::hash::H256;

#[derive(Debug, Clone)]
pub struct ScrollClient {
    client: reqwest::Client,
    base_url: String,
}

impl ScrollClient {
    #[allow(clippy::new_without_default)]
    pub fn new(base_url: impl AsRef<str>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.as_ref().trim_end_matches('/').to_string(),
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn batch(&self, batch: u64) -> BatchResponse {
        self.client
            .get(format!("{}/api/batch?index={batch}", self.base_url))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    // #[instrument(level = "debug", skip(self))]
    // pub async fn batches(&self, page: u64, per_page: u64) -> BatchResponse {
    //     self.client
    //         .get(format!(
    //             "{}/api/batches?page={page}&per_page={per_page}",
    //             self.base_url
    //         ))
    //         .send()
    //         .await
    //         .unwrap()
    //         .json()
    //         .await
    //         .unwrap()
    // }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct BatchResponseData {
    pub commit_tx_hash: H256,
    #[serde(with = "::serde_utils::string")]
    pub committed_at: u64,
    #[serde(with = "::serde_utils::string")]
    pub created_at: u64,
    pub end_block_number: u64,
    pub end_chunk_hash: H256,
    pub end_chunk_index: u64,
    pub finalize_tx_hash: Option<H256>,
    #[serde(with = "::serde_utils::string_opt")]
    pub finalized_at: Option<u64>,
    pub hash: H256,
    pub index: u64,
    pub rollup_status: Option<RollupStatus>,
    pub start_block_number: u64,
    pub start_chunk_hash: H256,
    pub start_chunk_index: u64,
    pub total_tx_num: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct BatchResponse {
    pub batch: BatchResponseData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupStatus {
    Finalized,
    Committed,
}
