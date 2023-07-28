//! Beacon API client, implemented as per https://ethereum.github.io/beacon-APIs/releases/v2.4.1/beacon-node-oapi.json

use std::{fmt::Display, marker::PhantomData};

use ibc_types::{
    ethereum::{
        beacon::{GenesisData, LightClientBootstrap, LightClientFinalityUpdate},
        SignedBeaconBlock, H256,
    },
    ethereum_consts_traits::ChainSpec,
};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    errors::{Error, InternalServerError},
    types::{BeaconHeaderData, LightClientUpdatesResponse, Spec},
};

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct BeaconApiClient<C: ChainSpec> {
    client: Client,
    base_url: String,
    _marker: PhantomData<C>,
}

impl<C: ChainSpec> BeaconApiClient<C> {
    pub async fn new(base_url: String) -> Self {
        let this = Self {
            client: reqwest::Client::new(),
            base_url,
            _marker: PhantomData,
        };

        let spec = this.spec().await.unwrap();

        assert_eq!(spec.data.preset_base, C::PRESET_BASE_KIND);

        this
    }

    pub async fn spec(&self) -> Result<Response<Spec>> {
        self.get_json("/eth/v1/config/spec").await
    }

    pub async fn finality_update(&self) -> Result<Response<LightClientFinalityUpdate<C>, Version>> {
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
    ) -> Result<Response<SignedBeaconBlock<C>, BeaconBlockExtra>> {
        self.get_json(format!("/eth/v2/beacon/blocks/{block_id}"))
            .await
    }

    pub async fn bootstrap(
        &self,
        finalized_root: H256,
    ) -> Result<Response<LightClientBootstrap<C>>> {
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
    ) -> Result<LightClientUpdatesResponse<C>> {
        self.get_json(format!(
            "/eth/v1/beacon/light_client/updates?start_period={start_period}&count={count}"
        ))
        .await
    }

    // pub async fn get_light_client_updates_simple<
    //     const SYNC_COMMITTEE_SIZE: usize,
    //     const BYTES_PER_LOGS_BLOOM: usize,
    //     const MAX_EXTRA_DATA_BYTES: usize,
    // >(
    //     &self,
    //     start_period: SyncCommitteePeriod,
    //     count: u64,
    // ) -> Result<
    //     LightClientUpdatesResponse<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    // > {
    //     let count = if count < 1 { 1 } else { count };
    //     self.get_json(format!(
    //         "/eth/v1/beacon/light_client/updates?start_period={}&count={}",
    //         start_period, count
    //     ))
    //     .await
    // }

    // Helper functions

    async fn get_json<T: DeserializeOwned>(&self, path: impl Into<String>) -> Result<T> {
        let url = format!("{}{}", self.base_url, path.into());

        tracing::debug!(%url, "get_json");

        let res = self.client.get(url).send().await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = res.bytes().await?;

                tracing::trace!(response = %String::from_utf8_lossy(&bytes), "get_json");

                Ok(serde_json::from_slice(&bytes).map_err(Error::Json)?)
            }
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

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Slot(u64),
    Hash(H256),
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
