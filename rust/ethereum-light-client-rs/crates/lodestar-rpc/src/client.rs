use crate::errors::Error;
use crate::types::{
    BeaconBlockRootResponse, BeaconHeaderResponse, FinalityCheckpointsResponse,
    GenesisDataResponse, LightClientBootstrapResponse, LightClientFinalityUpdateResponse,
    LightClientUpdatesResponse,
};
use ethereum_consensus::beacon::Slot;
use ethereum_consensus::sync_protocol::SyncCommitteePeriod;
use ethereum_consensus::types::H256;
use log::debug;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;

type Result<T> = core::result::Result<T, Error>;

pub struct RPCClient {
    http_client: Client,
    endpoint: String,
}

impl RPCClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            endpoint: endpoint.into(),
        }
    }

    // Beacon API

    pub async fn get_genesis(&self) -> Result<GenesisDataResponse> {
        self.request_get("/eth/v1/beacon/genesis").await
    }

    pub async fn get_beacon_block_root(&self, slot: Slot) -> Result<BeaconBlockRootResponse> {
        self.request_get(format!("/eth/v1/beacon/blocks/{}/root", slot))
            .await
    }

    pub async fn get_beacon_header_by_slot(&self, slot: Slot) -> Result<BeaconHeaderResponse> {
        self.request_get(format!("/eth/v1/beacon/headers/{}", slot))
            .await
    }

    pub async fn get_finality_checkpoints(&self) -> Result<FinalityCheckpointsResponse> {
        self.request_get("/eth/v1/beacon/states/head/finality_checkpoints")
            .await
    }

    // Light Client API

    pub async fn get_finality_update<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
    ) -> Result<
        LightClientFinalityUpdateResponse<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    > {
        self.request_get("/eth/v1/beacon/light_client/finality_update")
            .await
    }

    pub async fn get_bootstrap<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        finalized_root: H256,
    ) -> Result<
        LightClientBootstrapResponse<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    > {
        self.request_get(format!(
            "/eth/v1/beacon/light_client/bootstrap/0x{}",
            finalized_root
        ))
        .await
    }

    pub async fn get_light_client_updates<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        start_period: SyncCommitteePeriod,
        count: u64,
    ) -> Result<
        LightClientUpdatesResponse<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    > {
        let count = if count < 1 { 1 } else { count };
        for c in (1..=count).rev() {
            let res = self
                .request_get(format!(
                    "/eth/v1/beacon/light_client/updates?start_period={}&count={}",
                    start_period, c
                ))
                .await;
            if res.is_ok()
                || !res
                    .as_ref()
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("No partialUpdate available for period")
            {
                return res;
            }
        }
        unreachable!()
    }

    pub async fn get_light_client_updates_simple<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        start_period: SyncCommitteePeriod,
        count: u64,
    ) -> Result<
        LightClientUpdatesResponse<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    > {
        let count = if count < 1 { 1 } else { count };
        self.request_get(format!(
            "/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            start_period, count
        ))
        .await
    }

    // Helper functions

    async fn request_get<T: DeserializeOwned>(&self, path: impl Into<String>) -> Result<T> {
        let url = format!("{}{}", self.endpoint, path.into());
        debug!("request_get: url={}", url);
        let res = self.http_client.get(url).send().await?;
        match res.status() {
            StatusCode::OK => {
                let bytes = res.bytes().await?;
                debug!("request_get: response={}", String::from_utf8_lossy(&bytes));
                Ok(serde_json::from_slice(&bytes).map_err(Error::JSONDecodeError)?)
            }
            StatusCode::INTERNAL_SERVER_ERROR => Err(Error::RPCInternalServerError(
                res.json::<InternalServerError>().await?.message,
            )),
            _ => Err(Error::Other {
                description: res.text().await?,
            }),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InternalServerError {
    #[serde(rename = "statusCode")]
    status_code: u64,
    error: String,
    message: String,
}
