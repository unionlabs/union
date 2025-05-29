use std::{future::Future, time::Duration};

use futures::TryFutureExt;
use jsonrpsee::core::RpcResult;
use moka::policy::EvictionPolicy;
use opentelemetry::KeyValue;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tracing::trace;
use unionlabs::ibc::core::client::height::Height;
use voyager_primitives::{ChainId, ClientInfo, IbcSpec, IbcSpecId, IbcStorePathKey};
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
