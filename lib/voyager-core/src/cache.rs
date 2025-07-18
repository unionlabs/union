use std::{future::Future, time::Duration};

use futures::TryFutureExt;
use jsonrpsee::core::RpcResult;
use moka::policy::EvictionPolicy;
use opentelemetry::KeyValue;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, trace, warn};
use unionlabs::ibc::core::client::height::Height;
use voyager_primitives::{ChainId, ClientInfo, IbcSpec, IbcSpecId, IbcStorePathKey, Timestamp};
use voyager_types::RawClientId;

#[derive(Debug, Clone)]
pub struct Cache {
    state_cache: moka::future::Cache<StateRequest, Value>,
    state_cache_size_metric: opentelemetry::metrics::Gauge<u64>,
    state_cache_hit_counter_metric: opentelemetry::metrics::Counter<u64>,
    state_cache_miss_counter_metric: opentelemetry::metrics::Counter<u64>,

    client_info_cache: moka::future::Cache<ClientInfoRequest, ClientInfo>,
    client_info_cache_size_metric: opentelemetry::metrics::Gauge<u64>,
    client_info_cache_hit_counter_metric: opentelemetry::metrics::Counter<u64>,
    client_info_cache_miss_counter_metric: opentelemetry::metrics::Counter<u64>,

    latest_height_cache: moka::future::Cache<(ChainId, bool), Height>,
    latest_height_metric: opentelemetry::metrics::Gauge<u64>,

    latest_timestamp_cache: moka::future::Cache<(ChainId, bool), Timestamp>,
    latest_timestamp_metric: opentelemetry::metrics::Gauge<u64>,
    // proof_cache: moka::future::Cache,
}

impl Cache {
    #[allow(clippy::new_without_default)]
    pub fn new(config: Config) -> Self {
        Self {
            state_cache: moka::future::CacheBuilder::new(config.state.capacity)
                // .expire_after()
                .time_to_live(Duration::from_secs(config.state.time_to_live))
                .time_to_idle(Duration::from_secs(config.state.time_to_idle))
                .eviction_policy(EvictionPolicy::lru())
                .build(),
            state_cache_size_metric: opentelemetry::global::meter("voyager")
                .u64_gauge("cache.state.size")
                .build(),
            state_cache_hit_counter_metric: opentelemetry::global::meter("voyager")
                .u64_counter("cache.state.hit")
                .build(),
            state_cache_miss_counter_metric: opentelemetry::global::meter("voyager")
                .u64_counter("cache.state.miss")
                .build(),
            client_info_cache: moka::future::CacheBuilder::new(config.state.capacity)
                // never expire, this state is assumed to be immutable
                .time_to_live(Duration::from_secs(60 * 60 * 24 * 365 * 1000))
                .time_to_idle(Duration::from_secs(60 * 60 * 24 * 365 * 1000))
                .eviction_policy(EvictionPolicy::lru())
                .build(),
            client_info_cache_size_metric: opentelemetry::global::meter("voyager")
                .u64_gauge("cache.client_info.size")
                .build(),
            client_info_cache_hit_counter_metric: opentelemetry::global::meter("voyager")
                .u64_counter("cache.client_info.hit")
                .build(),
            client_info_cache_miss_counter_metric: opentelemetry::global::meter("voyager")
                .u64_counter("cache.client_info.miss")
                .build(),

            // 100 should be enough but would ideally be a passed in value
            latest_height_cache: moka::future::CacheBuilder::new(100)
                .time_to_live(Duration::from_secs(1000 * 365 * 24 * 60 * 60))
                .time_to_idle(Duration::from_secs(1000 * 365 * 24 * 60 * 60))
                .eviction_policy(EvictionPolicy::lru())
                .build(),
            latest_height_metric: opentelemetry::global::meter("voyager")
                .u64_gauge("chain.latest_height")
                .build(),

            // 100 should be enough but would ideally be a passed in value
            latest_timestamp_cache: moka::future::CacheBuilder::new(100)
                .time_to_live(Duration::from_secs(1000 * 365 * 24 * 60 * 60))
                .time_to_idle(Duration::from_secs(1000 * 365 * 24 * 60 * 60))
                .eviction_policy(EvictionPolicy::lru())
                .build(),
            latest_timestamp_metric: opentelemetry::global::meter("voyager")
                .u64_gauge("chain.latest_timestamp")
                .build(),
        }
    }

    pub async fn state<T: Serialize + DeserializeOwned>(
        &self,
        state_request: StateRequest,
        fut: impl Future<Output = RpcResult<Option<T>>>,
    ) -> RpcResult<Option<T>> {
        let attributes = &[KeyValue::new(
            "chain_id",
            state_request.chain_id.to_string(),
        )];

        self.state_cache_size_metric
            .record(self.state_cache.entry_count(), attributes);

        if let Some(state) = self.state_cache.get(&state_request).await {
            self.state_cache_hit_counter_metric.add(1, attributes);

            return Ok(Some(serde_json::from_value(state).expect(
                "infallible; only valid values are inserted into the cache; qed;",
            )));
        };

        self.state_cache_miss_counter_metric.add(1, attributes);

        let init = fut
            .map_ok(|state| serde_json::to_value(state).expect("serialization is infallible; qed;"))
            .await?;

        if init.is_null() {
            Ok(None)
        } else {
            let entry = self.state_cache.entry(state_request).or_insert(init).await;

            let value = entry.into_value();

            trace!(%value, "cached value");

            Ok(serde_json::from_value(value)
                .expect("infallible; only valid values are inserted into the cache; qed;"))
        }
    }

    pub async fn client_info(
        &self,
        client_info_request: ClientInfoRequest,
        fut: impl Future<Output = RpcResult<Option<ClientInfo>>>,
    ) -> RpcResult<Option<ClientInfo>> {
        let attributes = &[KeyValue::new(
            "chain_id",
            client_info_request.chain_id.to_string(),
        )];

        self.client_info_cache_size_metric
            .record(self.client_info_cache.entry_count(), attributes);

        if let Some(client_info) = self.client_info_cache.get(&client_info_request).await {
            self.client_info_cache_hit_counter_metric.add(1, attributes);

            return Ok(Some(client_info));
        };

        self.client_info_cache_miss_counter_metric
            .add(1, attributes);

        match fut.await? {
            Some(init) => {
                let entry = self
                    .client_info_cache
                    .entry(client_info_request)
                    .or_insert(init)
                    .await;

                let client_info = entry.into_value();

                trace!(
                    %client_info.client_type,
                    %client_info.ibc_interface,
                    %client_info.metadata,
                    "cached value"
                );

                Ok(Some(client_info))
            }
            None => Ok(None),
        }
    }

    // TODO: Perhaps ensure that unfinalized is never > finalized?
    pub async fn latest_height(
        &self,
        chain_id: ChainId,
        finalized: bool,
        fut: impl Future<Output = RpcResult<Height>>,
    ) -> RpcResult<Height> {
        let attributes = &[
            KeyValue::new("chain_id", chain_id.to_string()),
            KeyValue::new("finalized", finalized.to_string()),
        ];

        let height_response = match fut.await {
            Err(err) => {
                let latest_known_height = self
                    .latest_height_cache
                    .get(&(chain_id.clone(), finalized))
                    .await;
                if let Some(latest_known_height) = latest_known_height {
                    warn!(
                        %latest_known_height,
                        %err,
                        "unable to query latest height, returning latest known value"
                    );
                    return Ok(latest_known_height);
                } else {
                    return Err(err);
                }
            }
            Ok(height_response) => height_response,
        };

        let new_height = self
            .latest_height_cache
            .entry((chain_id.clone(), finalized))
            .and_upsert_with(async |maybe_entry| match maybe_entry {
                Some(cached_height) => {
                    let cached_height = cached_height.into_value();
                    if height_response < cached_height {
                        debug!(
                            %cached_height,
                            %height_response,
                            %chain_id,
                            %finalized,
                            "inconsistent latest height"
                        );
                        cached_height
                    } else {
                        height_response
                    }
                }
                None => height_response,
            })
            .await
            .into_value();

        self.latest_height_metric
            .record(new_height.height(), attributes);

        trace!(
            %new_height,
            %finalized,
            "latest height"
        );

        Ok(new_height)
    }

    // TODO: Perhaps ensure that unfinalized is never > finalized?
    pub async fn latest_timestamp(
        &self,
        chain_id: ChainId,
        finalized: bool,
        fut: impl Future<Output = RpcResult<Timestamp>>,
    ) -> RpcResult<Timestamp> {
        let attributes = &[
            KeyValue::new("chain_id", chain_id.to_string()),
            KeyValue::new("finalized", finalized.to_string()),
        ];

        let timestamp_response = match fut.await {
            Err(why) => {
                let latest_known_timestamp = self
                    .latest_timestamp_cache
                    .get(&(chain_id.clone(), finalized))
                    .await;
                if let Some(latest_known_timestamp) = latest_known_timestamp {
                    warn!(
                        %latest_known_timestamp,
                        "unable to query latest timestamp, returning latest known value"
                    );
                    return Ok(latest_known_timestamp);
                } else {
                    return Err(why);
                }
            }
            Ok(timestamp_response) => timestamp_response,
        };

        let new_timestamp = self
            .latest_timestamp_cache
            .entry((chain_id.clone(), finalized))
            .and_upsert_with(async |maybe_entry| match maybe_entry {
                Some(cached_timestamp) => {
                    let cached_timestamp = cached_timestamp.into_value();
                    if timestamp_response < cached_timestamp {
                        debug!(
                            %cached_timestamp,
                            %timestamp_response,
                            %chain_id,
                            %finalized,
                            "inconsistent latest timestamp"
                        );
                        cached_timestamp
                    } else {
                        timestamp_response
                    }
                }
                None => timestamp_response,
            })
            .await
            .into_value();

        self.latest_timestamp_metric
            .record(new_timestamp.as_nanos(), attributes);

        trace!(
            %new_timestamp,
            %finalized,
            "latest Timestamp"
        );

        Ok(new_timestamp)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub state: CacheConfig,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct CacheConfig {
    pub capacity: u64,
    pub time_to_live: u64,
    pub time_to_idle: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateRequest {
    chain_id: ChainId,
    ibc_spec_id: IbcSpecId,
    height: u64,
    path: Value,
}

impl StateRequest {
    pub fn new<P: IbcStorePathKey>(
        chain_id: ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> Self {
        Self {
            chain_id,
            ibc_spec_id: P::Spec::ID,
            height: height.height(),
            path: serde_json::to_value(path).expect("serialization is infallible; qed;"),
        }
    }

    pub fn new_raw(chain_id: ChainId, ibc_spec_id: IbcSpecId, height: Height, path: Value) -> Self {
        Self {
            chain_id,
            ibc_spec_id,
            height: height.height(),
            path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientInfoRequest {
    chain_id: ChainId,
    ibc_spec_id: IbcSpecId,
    client_id: RawClientId,
}

impl ClientInfoRequest {
    pub fn new<V: IbcSpec>(chain_id: ChainId, client_id: V::ClientId) -> Self {
        Self {
            chain_id,
            ibc_spec_id: V::ID,
            client_id: RawClientId::new(client_id),
        }
    }

    pub fn new_raw(chain_id: ChainId, ibc_spec_id: IbcSpecId, client_id: RawClientId) -> Self {
        Self {
            chain_id,
            ibc_spec_id,
            client_id,
        }
    }
}
