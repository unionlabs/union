//! Beacon API client, implemented as per <https://ethereum.github.io/beacon-APIs/releases/v2.4.1/beacon-node-oapi.json>

use std::{
    fmt::{Debug, Display},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use beacon_api_types::custom_types::Slot;
use moka::{future::Cache, ops::compute::Op};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, trace};
use unionlabs::primitives::H256;

use crate::{
    errors::Error,
    routes::{
        block::BeaconBlockResponse,
        genesis::{GenesisData, GenesisResponse},
        header::BeaconBlockHeaderResponse,
        light_client_bootstrap::LightClientBootstrapResponseTypes,
        light_client_finality_update::LightClientFinalityUpdateResponseTypes,
        light_client_updates::LightClientUpdateResponseTypes,
        spec::{Spec, SpecResponse},
    },
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct BeaconApiClient {
    client: Client,
    base_url: String,
    spec: Cache<(), Spec>,
    genesis: Cache<(), GenesisData>,
    finality_update: Cache<(), VersionedResponse<LightClientFinalityUpdateResponseTypes>>,
}

impl BeaconApiClient {
    pub fn new(base_url: impl AsRef<str>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.as_ref().trim_end_matches('/').into(),

            // refresh these caches every 12 hours
            spec: moka::future::CacheBuilder::new(1)
                .name("spec")
                .initial_capacity(1)
                .time_to_live(Duration::from_secs(12 * 60 * 60))
                .time_to_idle(Duration::from_secs(12 * 60 * 60))
                .build(),
            genesis: moka::future::CacheBuilder::new(1)
                .name("genesis")
                .initial_capacity(1)
                .time_to_live(Duration::from_secs(12 * 60 * 60))
                .time_to_idle(Duration::from_secs(12 * 60 * 60))
                .build(),
            finality_update: moka::future::CacheBuilder::new(1)
                .name("finality_update")
                .initial_capacity(1)
                .time_to_live(Duration::from_secs(12 * 60 * 60))
                .time_to_idle(Duration::from_secs(12 * 60 * 60))
                .build(),
        }
    }

    pub async fn spec(&self) -> Result<Spec> {
        Ok(self
            .spec
            .entry(())
            .and_try_compute_with(async |spec| match spec {
                Some(_) => Ok(Op::Nop),
                None => self
                    .get_json::<SpecResponse>("/eth/v1/config/spec")
                    .await
                    .map(|res| Op::Put(res.data)),
            })
            .await?
            .unwrap()
            .into_value())
    }

    pub async fn finality_update(
        &self,
    ) -> Result<VersionedResponse<LightClientFinalityUpdateResponseTypes>> {
        Ok(self
            .finality_update
            .entry(())
            .and_try_compute_with(async |spec| {
                let put = async || {
                    self.get_json("/eth/v1/beacon/light_client/finality_update")
                        .await
                        .map(Op::Put)
                };

                match spec {
                    Some(finality_update) => {
                        let Some(timestamp) = finality_update.value().fold_ref(
                            |f| match *f {},
                            // altair can't be trivially cached as it doesn't contain the execution payload
                            |_| None,
                            |f| Some(f.finalized_header.execution.timestamp),
                            |f| Some(f.finalized_header.execution.timestamp),
                            |f| Some(f.finalized_header.execution.timestamp),
                            |f| Some(f.finalized_header.execution.timestamp),
                        ) else {
                            return put().await;
                        };

                        let spec = self.spec().await?;
                        // 3 epochs is finalized
                        let max_age = 3 * spec.seconds_per_slot * spec.slots_per_epoch.get();

                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        let age = now.saturating_sub(timestamp);

                        info!(%age, %max_age, %timestamp, %now, "finality update cache info");

                        if age >= max_age {
                            info!("expired");
                            put().await
                        } else {
                            info!("not yet expired");
                            Ok(Op::Nop)
                        }
                    }
                    None => put().await,
                }
            })
            .await?
            .unwrap()
            .into_value())
    }

    pub async fn header(&self, block_id: BlockId) -> Result<BeaconBlockHeaderResponse> {
        self.get_json(format!("/eth/v1/beacon/headers/{block_id}"))
            .await
    }

    pub async fn block(&self, block_id: BlockId) -> Result<BeaconBlockResponse> {
        self.get_json(format!("/eth/v2/beacon/blocks/{block_id}"))
            .await
    }

    pub async fn bootstrap(
        &self,
        finalized_root: H256,
    ) -> Result<VersionedResponse<LightClientBootstrapResponseTypes>> {
        self.get_json(format!(
            "/eth/v1/beacon/light_client/bootstrap/{finalized_root}"
        ))
        .await
    }

    // Light Client API

    pub async fn genesis(&self) -> Result<GenesisData> {
        Ok(self
            .genesis
            .entry(())
            .and_try_compute_with(async |spec| match spec {
                Some(_) => Ok(Op::Nop),
                None => self
                    .get_json::<GenesisResponse>("/eth/v1/beacon/genesis")
                    .await
                    .map(|res| Op::Put(res.data)),
            })
            .await?
            .unwrap()
            .into_value())
    }

    /// NOTE: Lodestar will return an empty array if count is 0, however other implementations return an error response.
    pub async fn light_client_updates(
        &self,
        start_period: u64,
        count: u64,
    ) -> Result<Vec<VersionedResponse<LightClientUpdateResponseTypes>>> {
        self.get_json(format!(
            "/eth/v1/beacon/light_client/updates?start_period={start_period}&count={count}"
        ))
        .await
    }

    /// Convenience method to fetch the execution height of a beacon height.
    pub async fn execution_height(&self, block_id: BlockId) -> Result<u64> {
        let height = match self.block(block_id.clone()).await?.response {
            VersionedResponse::Phase0(_block) => {
                // block.message.body.execution_payload.block_number
                todo!("phase0 has no execution_payload")
            }
            VersionedResponse::Altair(_block) => {
                // block.message.body.execution_payload.block_number
                todo!("altair has no execution_payload")
            }
            VersionedResponse::Bellatrix(block) => {
                block.message.body.execution_payload.block_number
            }
            VersionedResponse::Capella(block) => block.message.body.execution_payload.block_number,
            VersionedResponse::Deneb(block) => block.message.body.execution_payload.block_number,
            VersionedResponse::Electra(block) => block.message.body.execution_payload.block_number,
        };

        debug!("beacon height {block_id} is execution height {height}");

        Ok(height)
    }

    pub async fn bootstrap_for_slot(
        &self,
        slot: Slot,
    ) -> Result<VersionedResponse<LightClientBootstrapResponseTypes>> {
        // NOTE(benluelo): While this is technically two actions, I consider it to be one
        // action - if the beacon chain doesn't have the header, it won't have the bootstrap
        // either. It would be nice if the beacon chain exposed "fetch bootstrap by slot"
        // functionality; I'm surprised it doesn't.

        let mut amount_of_slots_back = Slot::new(0);

        let spec = self.spec().await?;

        let floored_slot = Slot::new(
            slot.get() / (spec.slots_per_epoch.get() * spec.epochs_per_sync_committee_period)
                * (spec.slots_per_epoch.get() * spec.epochs_per_sync_committee_period),
        );

        info!("fetching bootstrap at {}", floored_slot);

        loop {
            let header_response = self
                .header(BlockId::Slot(floored_slot - amount_of_slots_back))
                .await;

            let header = match header_response {
                Ok(header) => header,
                Err(Error::NotFound(_)) => {
                    amount_of_slots_back = Slot::new(amount_of_slots_back.get() + 1);
                    continue;
                }

                Err(err) => return Err(err),
            };

            let bootstrap_response = self.bootstrap(header.data.root).await;

            match bootstrap_response {
                Ok(ok) => break Ok(ok),
                Err(err) => match err {
                    Error::Internal(_) => {
                        amount_of_slots_back = Slot::new(amount_of_slots_back.get() + 1);
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
            StatusCode::NOT_FOUND => {
                let raw = res.json::<Value>().await?;
                trace!(%raw, "not found");
                Err(Error::NotFound(raw))
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                let raw = res.json::<Value>().await?;
                trace!(%raw, "internal server error");
                Err(Error::Internal(raw))
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "version",
    content = "data",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = "")
)]
pub enum VersionedResponse<T: VersionedResponseTypes> {
    Phase0(T::Phase0),
    Altair(T::Altair),
    Bellatrix(T::Bellatrix),
    Capella(T::Capella),
    Deneb(T::Deneb),
    Electra(T::Electra),
}

impl<T: VersionedResponseTypes> VersionedResponse<T>
where
    T: VersionedResponseTypes<
        // fun fact: if these bounds are just T::Phase0, this fails with a circular resolution error
        Altair = <T as VersionedResponseTypes>::Phase0,
        Bellatrix = <T as VersionedResponseTypes>::Phase0,
        Capella = <T as VersionedResponseTypes>::Phase0,
        Deneb = <T as VersionedResponseTypes>::Phase0,
        Electra = <T as VersionedResponseTypes>::Phase0,
    >,
{
    /// "Unwrap" the inner type. This is only possible if all of the inner types are all the same.
    pub fn into_inner(self) -> T::Phase0 {
        match self {
            VersionedResponse::Phase0(t) => t,
            VersionedResponse::Altair(t) => t,
            VersionedResponse::Bellatrix(t) => t,
            VersionedResponse::Capella(t) => t,
            VersionedResponse::Deneb(t) => t,
            VersionedResponse::Electra(t) => t,
        }
    }
}

impl<T: VersionedResponseTypes> VersionedResponse<T> {
    pub fn fold<U>(
        self,
        phase0: impl FnOnce(T::Phase0) -> U,
        altair: impl FnOnce(T::Altair) -> U,
        bellatrix: impl FnOnce(T::Bellatrix) -> U,
        capella: impl FnOnce(T::Capella) -> U,
        deneb: impl FnOnce(T::Deneb) -> U,
        electra: impl FnOnce(T::Electra) -> U,
    ) -> U {
        match self {
            VersionedResponse::Phase0(t) => phase0(t),
            VersionedResponse::Altair(t) => altair(t),
            VersionedResponse::Bellatrix(t) => bellatrix(t),
            VersionedResponse::Capella(t) => capella(t),
            VersionedResponse::Deneb(t) => deneb(t),
            VersionedResponse::Electra(t) => electra(t),
        }
    }

    pub fn fold_ref<U>(
        &self,
        phase0: impl FnOnce(&T::Phase0) -> U,
        altair: impl FnOnce(&T::Altair) -> U,
        bellatrix: impl FnOnce(&T::Bellatrix) -> U,
        capella: impl FnOnce(&T::Capella) -> U,
        deneb: impl FnOnce(&T::Deneb) -> U,
        electra: impl FnOnce(&T::Electra) -> U,
    ) -> U {
        match self {
            VersionedResponse::Phase0(t) => phase0(t),
            VersionedResponse::Altair(t) => altair(t),
            VersionedResponse::Bellatrix(t) => bellatrix(t),
            VersionedResponse::Capella(t) => capella(t),
            VersionedResponse::Deneb(t) => deneb(t),
            VersionedResponse::Electra(t) => electra(t),
        }
    }
}

pub trait VersionedResponseTypes {
    type Phase0: Debug + Serialize + DeserializeOwned;
    type Altair: Debug + Serialize + DeserializeOwned;
    type Bellatrix: Debug + Serialize + DeserializeOwned;
    type Capella: Debug + Serialize + DeserializeOwned;
    type Deneb: Debug + Serialize + DeserializeOwned;
    type Electra: Debug + Serialize + DeserializeOwned;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Slot(Slot),
    Hash(H256),
}

impl From<Slot> for BlockId {
    fn from(slot: Slot) -> Self {
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
