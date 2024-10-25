//! Beacon API client, implemented as per <https://ethereum.github.io/beacon-APIs/releases/v2.4.1/beacon-node-oapi.json>

use std::fmt::Display;

use beacon_api_types::{
    GenesisData, LightClientBootstrap, LightClientFinalityUpdate, SignedBeaconBlock,
};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, info, trace};
use unionlabs::hash::H256;

use crate::{
    errors::{Error, InternalServerError, NotFoundError},
    types::{BeaconHeaderData, LightClientUpdatesResponse, Spec},
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct BeaconApiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum NewError {
    #[error("incorrect chain spec")]
    IncorrectChainSpec,
    #[error(transparent)]
    Error(#[from] Error),
}

impl BeaconApiClient {
    pub async fn new(base_url: String) -> core::result::Result<Self, NewError> {
        let this = Self {
            client: reqwest::Client::new(),
            base_url,
        };

        // TODO: Do checks against a spec?
        let _spec = this.spec().await?;

        // if spec.data.seconds_per_slot != C::SECONDS_PER_SLOT::U64 {
        //     return Err(NewError::IncorrectChainSpec);
        // }

        // if spec.data.slots_per_epoch != C::SLOTS_PER_EPOCH::U64 {
        //     return Err(NewError::IncorrectChainSpec);
        // }

        Ok(this)
    }

    pub async fn spec(&self) -> Result<Response<Spec>> {
        self.get_json("/eth/v1/config/spec").await
    }

    pub async fn finality_update(&self) -> Result<Response<LightClientFinalityUpdate, Version>> {
        self.get_json("/eth/v1/beacon/light_client/finality_update")
            .await
    }

    pub async fn header(
        &self,
        block_id: BlockId,
    ) -> Result<Response<BeaconHeaderData, BeaconHeaderExtra>> {
        self.get_json(format!("/eth/v1/beacon/headers/{block_id}"))
            .await
    }

    pub async fn block(
        &self,
        block_id: BlockId,
    ) -> Result<Response<SignedBeaconBlock, BeaconBlockExtra>> {
        self.get_json(format!("/eth/v2/beacon/blocks/{block_id}"))
            .await
    }

    pub async fn bootstrap(&self, finalized_root: H256) -> Result<Response<LightClientBootstrap>> {
        self.get_json(format!(
            "/eth/v1/beacon/light_client/bootstrap/{finalized_root}"
        ))
        .await
    }

    // Light Client API

    pub async fn genesis(&self) -> Result<Response<GenesisData>> {
        self.get_json("/eth/v1/beacon/genesis").await
    }

    // TODO: Just return the inner type directly (Vec<_>)
    pub async fn light_client_updates(
        &self,
        start_period: u64,
        count: u64,
    ) -> Result<LightClientUpdatesResponse> {
        self.get_json(format!(
            "/eth/v1/beacon/light_client/updates?start_period={start_period}&count={count}"
        ))
        .await
    }

    /// Convenience method to fetch the execution height of a beacon height.
    pub async fn execution_height(&self, block_id: BlockId) -> Result<u64> {
        let height = self
            .block(block_id.clone())
            .await?
            .data
            .message
            .body
            .execution_payload
            .block_number;

        debug!("beacon height {block_id} is execution height {height}");

        Ok(height)
    }

    pub async fn bootstrap_for_slot(&self, slot: u64) -> Result<Response<LightClientBootstrap>> {
        // NOTE(benluelo): While this is technically two actions, I consider it to be one
        // action - if the beacon chain doesn't have the header, it won't have the bootstrap
        // either. It would be nice if the beacon chain exposed "fetch bootstrap by slot"
        // functionality; I'm surprised it doesn't.

        let mut amount_of_slots_back: u64 = 0;

        let spec = self.spec().await?.data;

        let floored_slot = slot / (spec.slots_per_epoch * spec.epochs_per_sync_committee_period)
            * (spec.slots_per_epoch * spec.epochs_per_sync_committee_period);

        info!("fetching bootstrap at {}", floored_slot);

        loop {
            let header_response = self
                .header(BlockId::Slot(floored_slot - amount_of_slots_back))
                .await;

            let header = match header_response {
                Ok(header) => header,
                Err(Error::NotFound(NotFoundError {
                    status_code: _,
                    error: _,
                    message,
                })) if message.starts_with("No block found for id") => {
                    amount_of_slots_back += 1;
                    continue;
                }

                Err(err) => return Err(err),
            };

            let bootstrap_response = self.bootstrap(header.data.root).await;

            match bootstrap_response {
                Ok(ok) => break Ok(ok),
                Err(err) => match err {
                    Error::Internal(InternalServerError {
                        status_code: _,
                        error: _,
                        message,
                    }) if message.starts_with("syncCommitteeWitness not available") => {
                        amount_of_slots_back += 1;
                    }
                    _ => return Err(err),
                },
            };
        }
    }

    // Helper functions

    async fn get_json<T: DeserializeOwned>(&self, path: impl Into<String>) -> Result<T> {
        let url = format!("{}{}", self.base_url, path.into());

        debug!(%url, "get_json");

        let res = self.client.get(url).send().await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = res.bytes().await?;

                trace!(response = %String::from_utf8_lossy(&bytes), "get_json");

                Ok(serde_json::from_slice(&bytes).map_err(Error::Json)?)
            }
            StatusCode::NOT_FOUND => Err(Error::NotFound(res.json::<NotFoundError>().await?)),
            StatusCode::INTERNAL_SERVER_ERROR => {
                Err(Error::Internal(res.json::<InternalServerError>().await?))
            }
            code => Err(Error::Other {
                code,
                text: res.text().await?,
            }),
        }
    }
}

pub enum Encoding {
    Json,
    Ssz,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EthConsensusVersion {
    #[serde(rename = "phase0")]
    Phase0,
    #[serde(rename = "altair")]
    Altair,
    #[serde(rename = "bellatrix")]
    Bellatrix,
    #[serde(rename = "capella")]
    Capella,
    #[serde(rename = "deneb")]
    Deneb,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<Data, Extra = Nil> {
    pub data: Data,
    #[serde(flatten)]
    pub extra: Extra,
}

impl<Data, Extra> Response<Data, Extra> {
    pub fn map_data<T>(self, f: impl FnOnce(Data) -> T) -> Response<T, Extra> {
        Response {
            data: f(self.data),
            extra: self.extra,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nil {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: EthConsensusVersion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconHeaderExtra {
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconBlockExtra {
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
    pub version: EthConsensusVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Slot(u64),
    Hash(H256),
}

impl From<u64> for BlockId {
    fn from(slot: u64) -> Self {
        BlockId::Slot(slot)
    }
}

impl From<H256> for BlockId {
    fn from(root: H256) -> Self {
        BlockId::Hash(root)
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockId::Head => write!(f, "head"),
            BlockId::Genesis => write!(f, "genesis"),
            BlockId::Finalized => write!(f, "finalized"),
            BlockId::Slot(slot) => write!(f, "{slot}"),
            BlockId::Hash(hash) => write!(f, "{hash}"),
        }
    }
}
