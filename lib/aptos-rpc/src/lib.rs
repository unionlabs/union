use macros::model;
use reqwest::{Client, Result};
use serde::Deserialize;
use serde_json::Value;
use unionlabs::hash::H256;

#[derive(Debug, Clone)]
pub struct AptosRpcClient {
    client: Client,
    base_url: String,
}

impl AptosRpcClient {
    pub fn new(base_url: impl AsRef<str>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.as_ref().trim_end_matches('/').to_owned(),
        }
    }

    /// `url` MUST start with a `/`.
    async fn do_request<T: for<'de> Deserialize<'de>>(&self, url: impl AsRef<str>) -> Result<T> {
        self.client
            .get(self.base_url.clone() + url.as_ref())
            .send()
            .await?
            .json()
            .await
    }

    pub async fn ledger_info(&self) -> Result<LedgerInfo> {
        self.do_request("/v1").await
    }

    pub async fn block_by_height(&self, height: u64, with_transactions: bool) -> Result<Block> {
        self.do_request(format!(
            "/v1/blocks/by_height/{height}?with_transactions={with_transactions}"
        ))
        .await
    }

    pub async fn view(&self, body: ViewBody) -> Result<Value> {
        todo!()
    }
}

#[model]
pub struct LedgerInfo {
    // TODO: Chain id has to be unique across all chains for use in voyager, figure out a way to make this unique (maybe prefix with `aptos-` downstream or something similar?)
    pub chain_id: u8,
    #[serde(with = "::serde_utils::string")]
    pub block_height: u64,
}

#[model]
pub struct Block {
    #[serde(with = "::serde_utils::string")]
    pub block_height: u64,
    pub block_hash: H256,
    #[serde(with = "::serde_utils::string")]
    pub block_timestamp: u64,
}

// curl http://localhost:30731/v1/view -H 'Content-Type: application/json' -d '{"function": "0xdc08d8da6e3de03f62fdab618dc36f2e4e9cd70b03722c1d46df767370602771::Core::get_vault_addr","type_arguments":[],"arguments":[]}'
#[model]
pub struct ViewBody {}
