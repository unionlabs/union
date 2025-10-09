// #![warn(clippy::unwrap_used)]

use std::sync::{Arc, OnceLock};

use anyhow::{Context as _, anyhow};
use futures::TryFutureExt;
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::{ErrorObject, ErrorObjectOwned},
};
use serde_json::Value;
use tracing::{debug, info_span, instrument, trace};
use unionlabs::{ErrorReporter, ibc::core::client::height::Height, primitives::Bytes};
use voyager_plugin_protocol::WithId;
use voyager_primitives::{
    ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface, IbcSpec,
    IbcSpecId, IbcStorePathKey, QueryHeight, Timestamp,
};
use voyager_rpc::{
    ClientBootstrapModuleClient, ClientModuleClient, FATAL_JSONRPC_ERROR_CODE,
    FinalityModuleClient, PluginClient, RawProofModuleClient, RawStateModuleClient,
    VoyagerRpcServer, json_rpc_error_to_error_object,
    types::{
        IbcProofResponse, IbcStateResponse, InfoResponse, SelfClientStateResponse,
        SelfConsensusStateResponse,
    },
};
use voyager_types::{IbcProof, RawClientId};
use voyager_vm::ItemId;

use crate::{
    cache::{ClientInfoRequest, StateRequest},
    context::Context,
};

#[derive(Clone)]
pub struct Server {
    context: Arc<OnceLock<Context>>,
    cache: crate::cache::Cache,
    item_id: Option<ItemId>,
    server_metrics: ServerMetrics,
}

#[derive(Clone)]
pub struct ServerMetrics {
    // latest_height_gauge: Gauge<u64>,
    // latest_timestamp_gauge: Gauge<u64>,
}

impl ServerMetrics {
    fn new() -> Self {
        Self {
            // latest_height_gauge: opentelemetry::global::meter("voyager")
            //     .u64_gauge("chain.latest_height")
            //     .build(),
            // latest_timestamp_gauge: opentelemetry::global::meter("voyager")
            //     .u64_gauge("chain.latest_timestamp")
            //     .build(),
        }
    }
}

impl Server {
    pub fn new(cache: crate::cache::Cache, context: Arc<OnceLock<Context>>) -> Self {
        Server {
            context,
            cache,
            item_id: None,
            server_metrics: ServerMetrics::new(),
        }
    }

    pub fn id(&self) -> Option<ItemId> {
        self.item_id
    }

    // REVIEW: Don't clone here?
    pub fn with_id(&self, item_id: Option<ItemId>) -> Server {
        Server {
            context: self.context.clone(),
            cache: self.cache.clone(),
            item_id,
            server_metrics: self.server_metrics.clone(),
        }
    }

    fn span(&self) -> tracing::Span {
        match self.item_id {
            Some(item_id) => info_span!("processing_request", item_id = item_id.raw()),
            None => info_span!("processing_request"),
        }
    }

    /// Returns the contained context, if it has been loaded.
    pub fn context(&self) -> RpcResult<&Context> {
        self.context
            .get()
            .ok_or_else(|| ErrorObject::owned(-2, "server has not started", None::<()>))
    }

    #[instrument(skip_all, fields(%chain_id, %finalized))]
    pub async fn latest_height(&self, chain_id: &ChainId, finalized: bool) -> RpcResult<Height> {
        trace!("querying latest height");

        let latest_height = self
            .cache
            .latest_height(
                chain_id.clone(),
                finalized,
                self.context()?
                    .finality_module(chain_id)?
                    .with_id(self.item_id)
                    .query_latest_height(finalized)
                    .map_err(json_rpc_error_to_error_object),
            )
            .await?;

        trace!("queried latest height");

        // self.server_metrics.latest_height_gauge.record(
        //     latest_height.height(),
        //     &[
        //         KeyValue::new("chain_id", chain_id.to_string()),
        //         KeyValue::new("finalized", false),
        //     ],
        // );

        Ok(latest_height)
    }

    #[instrument(skip_all, fields(%chain_id, %finalized))]
    pub async fn latest_timestamp(
        &self,
        chain_id: &ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        trace!("querying latest timestamp");

        let latest_timestamp = self
            .cache
            .latest_timestamp(
                chain_id.clone(),
                finalized,
                self.context()?
                    .finality_module(chain_id)?
                    .with_id(self.item_id)
                    .query_latest_timestamp(finalized)
                    .map_err(json_rpc_error_to_error_object),
            )
            .await?;

        trace!("queried latest timestamp");

        // self.server_metrics.latest_timestamp_gauge.record(
        //     latest_timestamp.timestamp(),
        //     &[
        //         KeyValue::new("chain_id", chain_id.to_string()),
        //         KeyValue::new("finalized", false),
        //     ],
        // );

        Ok(latest_timestamp)
    }

    #[instrument(skip_all, fields(%height, %chain_id))]
    pub async fn query_height(&self, chain_id: &ChainId, height: QueryHeight) -> RpcResult<Height> {
        match height {
            QueryHeight::Latest => self.latest_height(chain_id, false).await,
            QueryHeight::Finalized => self.latest_height(chain_id, true).await,
            // TODO: Check if this is <= the latest height of the chain?
            QueryHeight::Specific(height) => Ok(height),
        }
    }
}

impl Server {
    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_height(
        &self,
        chain_id: &ChainId,
        finalized: bool,
    ) -> RpcResult<Height> {
        self.span()
            .in_scope(|| self.latest_height(chain_id, finalized))
            .await
    }

    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_timestamp(
        &self,
        chain_id: &ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        self.span()
            .in_scope(|| self.latest_timestamp(chain_id, finalized))
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, client_id = %client_id.as_raw()))]
    pub async fn client_info(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.span()
            .in_scope(|| {
                self.cache.client_info(
                    ClientInfoRequest::new_raw(
                        chain_id.clone(),
                        ibc_spec_id.clone(),
                        client_id.clone(),
                    ),
                    async {
                        trace!("fetching client info");

                        let client_info = self
                            .context()?
                            .state_module(chain_id, ibc_spec_id)?
                            .with_id(self.item_id)
                            .client_info_raw(client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        match client_info {
                            Some(ref client_info) => {
                                trace!(
                                    %client_info.ibc_interface,
                                    %client_info.client_type,
                                    "fetched client info"
                                );
                            }
                            None => {
                                trace!("client not found");
                            }
                        }

                        Ok(client_info)
                    },
                )
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, height = %at, client_id = %client_id.as_raw()))]
    pub async fn client_state_meta(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching client state meta");

                let height = self.query_height(chain_id, at).await?;

                let context = self.context()?;

                let client_info = self
                    .client_info(chain_id, ibc_spec_id, client_id.clone())
                    .await?;

                let Some(client_info) = client_info else {
                    trace!(
                        "client info for client {client_id} not \
                        found at height {height} on chain {chain_id}"
                    );
                    return Ok(None);
                };

                let ibc_spec_handler = self
                    .context()?
                    .ibc_spec_handlers
                    .handlers
                    .get(ibc_spec_id)
                    .ok_or_else(|| {
                        fatal_error(&*anyhow!(
                            "ibc spec {ibc_spec_id} is not \
                            supported in this build of voyager"
                        ))
                    })?;

                let raw_client_state = self
                    .query_ibc_state_raw(
                        chain_id.clone(),
                        ibc_spec_id.clone(),
                        QueryHeight::Specific(height),
                        (ibc_spec_handler.client_state_path)(client_id.clone())
                            .map_err(|err| fatal_error(&*err))?,
                    )
                    .await?
                    .state;

                let Some(raw_client_state) = raw_client_state else {
                    trace!(
                        "client state for client {client_id} not \
                        found at height {height} on chain {chain_id}"
                    );
                    return Ok(None);
                };

                trace!(?raw_client_state);

                let client_state = serde_json::from_value::<Bytes>(raw_client_state)
                    .with_context(|| {
                        format!(
                            "querying client state for client \
                            {client_id} at {height} on {chain_id}"
                        )
                    })
                    .map_err(|e| fatal_error(&*e))?;

                let meta = context
                    .client_module(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        ibc_spec_id,
                    )?
                    .with_id(self.item_id)
                    .decode_client_state_meta(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    client_state_meta.height = %meta.counterparty_height,
                    client_state_meta.chain_id = %meta.counterparty_chain_id,
                    %client_info.ibc_interface,
                    %client_info.client_type,
                    "fetched client state meta"
                );

                Ok(Some(meta))
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, height = %at, client_id = %client_id.as_raw()))]
    pub async fn consensus_state_meta(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching consensus state meta");

                let height = self.query_height(chain_id, at).await?;

                let context = self.context()?;

                let Some(client_info) = self
                    .client_info(chain_id, ibc_spec_id, client_id.clone())
                    .await?
                else {
                    trace!(
                        "client info for client {client_id} not \
                        found at height {height} on chain {chain_id}"
                    );
                    return Ok(None);
                };

                let ibc_spec_handler = self
                    .context()?
                    .ibc_spec_handlers
                    .handlers
                    .get(ibc_spec_id)
                    .ok_or_else(|| {
                        fatal_error(&*anyhow!(
                            "ibc spec {ibc_spec_id} is \
                            not supported in this build of voyager"
                        ))
                    })?;

                let raw_consensus_state = self
                    .query_ibc_state_raw(
                        chain_id.clone(),
                        ibc_spec_id.clone(),
                        QueryHeight::Specific(height),
                        (ibc_spec_handler.consensus_state_path)(
                            client_id.clone(),
                            counterparty_height.to_string(),
                        )
                        .map_err(|err| fatal_error(&*err))?,
                    )
                    .await?
                    .state;

                let Some(raw_consensus_state) = raw_consensus_state else {
                    trace!(
                        "consensus state for client {client_id}, \
                        counterparty height {counterparty_height} not \
                        found at height {height} on chain {chain_id}"
                    );
                    return Ok(None);
                };

                let consensus_state = serde_json::from_value::<Option<Bytes>>(raw_consensus_state)
                    .with_context(|| {
                        format!(
                            "querying consensus state for client {client_id}, \
                            counterparty height {counterparty_height} at \
                            {height} on {chain_id}"
                        )
                    })
                    .map_err(|e| fatal_error(&*e))?;

                let Some(consensus_state) = consensus_state else {
                    trace!(
                        "consensus state for client {client_id}, counterparty \
                        height {counterparty_height} not found at height \
                        {height} on chain {chain_id}"
                    );
                    return Ok(None);
                };

                let meta = context
                    .client_module(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        ibc_spec_id,
                    )?
                    .with_id(self.item_id)
                    .decode_consensus_state_meta(consensus_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    consensus_state_meta.timestamp = %meta.timestamp,
                    %client_info.ibc_interface,
                    %client_info.client_type,
                    "fetched consensus state meta"
                );

                Ok(Some(meta))
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %query))]
    async fn query_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        query: Value,
    ) -> RpcResult<Value> {
        self.span()
            .in_scope(|| async {
                trace!("query");

                let state_module = self
                    .context()?
                    .state_module(&chain_id, &ibc_spec_id)?
                    .with_id(self.item_id);

                let value = state_module
                    .query_raw(serde_json::to_value(query.clone()).unwrap())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%value, "queried");

                Ok(value)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_state_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcStateResponse<Value>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching ibc state");

                let height = self.query_height(&chain_id, height).await?;

                let state_module = self
                    .context()?
                    .state_module(&chain_id, &ibc_spec_id)?
                    .with_id(self.item_id);

                let state = self
                    .cache
                    .state::<Value>(
                        StateRequest::new_raw(
                            chain_id.clone(),
                            ibc_spec_id.clone(),
                            height,
                            path.clone(),
                        ),
                        state_module
                            .query_ibc_state_raw(
                                height,
                                serde_json::to_value(path.clone()).unwrap(),
                            )
                            .map_ok(|state| {
                                // TODO: Use valuable here
                                trace!(%state, "fetched ibc state");

                                if state.is_null() { None } else { Some(state) }
                            })
                            .map_err(|e| json_rpc_error_to_error_object(e)),
                    )
                    .await?;

                Ok(IbcStateResponse { height, state })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_proof_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProofResponse> {
        self.span()
            .in_scope(|| async {
                let height = self.query_height(&chain_id, height).await?;

                debug!("fetching ibc proof");

                let proof_module = self
                    .context()?
                    .proof_module(&chain_id, &ibc_spec_id)?
                    .with_id(self.item_id);

                let res = proof_module
                    .query_ibc_proof_raw(height, path)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                debug!(result = %serde_json::to_value(&res).unwrap(), "fetched ibc proof");

                Ok(
                    res.map_or(IbcProofResponse::NotAvailable, |(proof, proof_type)| {
                        IbcProofResponse::Proof(IbcProof {
                            height,
                            proof,
                            proof_type,
                        })
                    }),
                )
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height, ?path))]
    pub async fn query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: &ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> RpcResult<IbcStateResponse<P::Value>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching ibc state");

                let state_module = self
                    .context()?
                    .state_module(chain_id, &P::Spec::ID)?
                    .with_id(self.item_id);

                let state = self
                    .cache
                    .state::<P::Value>(
                        StateRequest::new::<P>(chain_id.clone(), height, path.clone()),
                        state_module
                            .query_ibc_state_raw(
                                height,
                                serde_json::to_value(path.clone()).unwrap(),
                            )
                            .map_ok(|state| {
                                // TODO: Use valuable here
                                trace!(%state, "fetched ibc state");

                                serde_json::from_value(state).unwrap()
                            })
                            .map_err(|e| json_rpc_error_to_error_object(e)),
                    )
                    .await?;

                Ok(IbcStateResponse { height, state })
            })
            .await
    }

    #[instrument(
        skip_all,
        fields(
            %chain_id,
            ibc_spec_id = %P::Spec::ID,
            %height,
            path = %serde_json::to_value(path.clone()).unwrap()
        )
    )]
    pub async fn query_ibc_proof<P: IbcStorePathKey>(
        &self,
        chain_id: &ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> RpcResult<IbcProofResponse> {
        self.span()
            .in_scope(|| async {
                trace!("fetching ibc state");

                let proof_module = self
                    .context()?
                    .proof_module(chain_id, &P::Spec::ID)?
                    .with_id(self.item_id);

                let res = proof_module
                    .query_ibc_proof_raw(height, serde_json::to_value(path.clone()).unwrap())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                debug!(result = %serde_json::to_value(&res).unwrap(), "fetched ibc proof");

                Ok(
                    res.map_or(IbcProofResponse::NotAvailable, |(proof, proof_type)| {
                        IbcProofResponse::Proof(IbcProof {
                            height,
                            proof,
                            proof_type,
                        })
                    }),
                )
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %client_type, %height))]
    pub async fn self_client_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: Height,
        config: Value,
    ) -> RpcResult<SelfClientStateResponse> {
        self.span()
            .in_scope(|| async {
                trace!("querying self client state");

                let client_bootstrap_module = self
                    .context()?
                    .client_bootstrap_module(&chain_id, &client_type)?
                    .with_id(self.item_id);

                let state = client_bootstrap_module
                    .self_client_state(height, config)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%state, "fetched self client state");

                Ok(SelfClientStateResponse { height, state })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %client_type, %height))]
    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusStateResponse> {
        self.span()
            .in_scope(|| async {
                trace!("querying self consensus state");

                let client_bootstrap_module = self
                    .context()?
                    .client_bootstrap_module(&chain_id, &client_type)?
                    .with_id(self.item_id);

                let height = self.query_height(&chain_id, height).await?;

                let state = client_bootstrap_module
                    .self_consensus_state(height, config)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%state, "fetched self consensus state");

                Ok(SelfConsensusStateResponse { height, state })
            })
            .await
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id, %proof))]
    pub async fn encode_proof(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes> {
        self.span()
            .in_scope(|| async {
                trace!("encoding proof");

                let client_module = self
                    .context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id);

                let proof = client_module
                    .encode_proof(proof)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(%proof, "encoded proof");

                Ok(proof)
            })
            .await
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id, %header))]
    pub async fn encode_header(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        header: Value,
    ) -> RpcResult<Bytes> {
        self.span()
            .in_scope(|| async {
                trace!("encoding header");

                let client_module = self
                    .context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id);

                let header = client_module
                    .encode_header(header)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(%header, "encoded header");

                Ok(header)
            })
            .await
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_client_state_meta(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        self.span()
            .in_scope(|| async {
                trace!("decoding client state meta");

                let client_module = self
                    .context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id);

                let meta = client_module
                    .decode_client_state_meta(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    height = %meta.counterparty_height,
                    chain_id = %meta.counterparty_chain_id,
                    "decoded client state meta"
                );

                Ok(meta)
            })
            .await
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_client_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.span()
            .in_scope(|| async {
                self.context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id)
                    .decode_client_state(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)
            })
            .await
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_consensus_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.span()
            .in_scope(|| async {
                self.context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id)
                    .decode_consensus_state(consensus_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)
            })
            .await
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn encode_client_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
        self.span()
            .in_scope(|| async {
                self.context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id)
                    .encode_client_state(client_state, metadata)
                    .await
                    .map_err(json_rpc_error_to_error_object)
            })
            .await
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn encode_consensus_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        self.span()
            .in_scope(|| async {
                self.context()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id)
                    .encode_consensus_state(consensus_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)
            })
            .await
    }
}

/// rpc impl
#[async_trait]
impl VoyagerRpcServer for Server {
    async fn info(&self, _: &Extensions) -> RpcResult<InfoResponse> {
        Ok(self.context()?.info())
    }

    async fn equivalent_chain_ids(
        &self,
        _: &Extensions,
        chain_id: ChainId,
    ) -> RpcResult<Vec<ChainId>> {
        Ok(self
            .context()?
            .equivalent_chain_ids()
            .equivalents(&chain_id)
            .cloned()
            .collect())
    }

    // =========
    // CONSENSUS
    // =========

    async fn query_latest_height(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Height> {
        self.with_id(e.try_get().ok().cloned())
            .query_latest_height(&chain_id, finalized)
            .await
    }

    async fn query_latest_timestamp(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        self.with_id(e.try_get().ok().cloned())
            .query_latest_timestamp(&chain_id, finalized)
            .await
    }

    // =====
    // STATE
    // =====

    async fn client_info(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.with_id(e.try_get().ok().cloned())
            .client_info(&chain_id, &ibc_spec_id, client_id)
            .await
    }

    async fn client_state_meta(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.with_id(e.try_get().ok().cloned())
            .client_state_meta(&chain_id, &ibc_spec_id, at, client_id)
            .await
    }

    async fn consensus_state_meta(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.with_id(e.try_get().ok().cloned())
            .consensus_state_meta(&chain_id, &ibc_spec_id, at, client_id, counterparty_height)
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %query))]
    async fn query(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        query: Value,
    ) -> RpcResult<Value> {
        self.with_id(e.try_get().ok().cloned())
            .query_raw(chain_id, ibc_spec_id, query)
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_state(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcStateResponse<Value>> {
        self.with_id(e.try_get().ok().cloned())
            .query_ibc_state_raw(chain_id, ibc_spec_id, height, path)
            .await
    }

    // =====
    // PROOF
    // =====

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_proof(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProofResponse> {
        self.with_id(e.try_get().ok().cloned())
            .query_ibc_proof_raw(chain_id, ibc_spec_id, height, path)
            .await
    }

    // ==========
    // SELF STATE
    // ==========

    async fn self_client_state(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfClientStateResponse> {
        let item_id = e.try_get().ok().cloned();
        let this = self.with_id(item_id);

        let height = this.query_height(&chain_id, height).await?;

        this.self_client_state(chain_id, client_type, height, config)
            .await
    }

    async fn self_consensus_state(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusStateResponse> {
        self.with_id(e.try_get().ok().cloned())
            .self_consensus_state(chain_id, client_type, height, config)
            .await
    }

    // =====
    // CODEC
    // =====

    // TODO: Use valuable here
    async fn encode_proof(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes> {
        self.with_id(e.try_get().ok().cloned())
            .encode_proof(&client_type, &ibc_interface, &ibc_spec_id, proof)
            .await
    }

    async fn encode_header(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        header: Value,
    ) -> RpcResult<Bytes> {
        self.with_id(e.try_get().ok().cloned())
            .encode_header(&client_type, &ibc_interface, &ibc_spec_id, header)
            .await
    }

    // TODO: Use valuable here
    async fn decode_client_state_meta(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        self.with_id(e.try_get().ok().cloned())
            .decode_client_state_meta(&client_type, &ibc_interface, &ibc_spec_id, client_state)
            .await
    }

    async fn decode_client_state(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.with_id(e.try_get().ok().cloned())
            .decode_client_state(&client_type, &ibc_interface, &ibc_spec_id, client_state)
            .await
    }

    async fn decode_consensus_state(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.with_id(e.try_get().ok().cloned())
            .decode_consensus_state(&client_type, &ibc_interface, &ibc_spec_id, consensus_state)
            .await
    }

    async fn encode_client_state(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
        self.with_id(e.try_get().ok().cloned())
            .encode_client_state(
                &client_type,
                &ibc_interface,
                &ibc_spec_id,
                client_state,
                metadata,
            )
            .await
    }

    async fn encode_consensus_state(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        self.with_id(e.try_get().ok().cloned())
            .encode_consensus_state(&client_type, &ibc_interface, &ibc_spec_id, consensus_state)
            .await
    }

    #[instrument(skip_all, fields(%plugin, %method))]
    async fn plugin_custom(
        &self,
        e: &Extensions,
        plugin: String,
        method: String,
        params: Vec<Value>,
    ) -> RpcResult<Value> {
        debug!(?params);

        PluginClient::<Value, Value>::custom(
            self.with_id(e.try_get().ok().cloned())
                .context()?
                .plugin(&plugin)?,
            method,
            params,
        )
        .await
        .map_err(json_rpc_error_to_error_object)
    }
}

pub(crate) fn fatal_error(t: impl core::error::Error) -> ErrorObjectOwned {
    ErrorObject::owned(
        FATAL_JSONRPC_ERROR_CODE,
        ErrorReporter(t).to_string(),
        None::<()>,
    )
}

trait ExtensionsExt {
    /// Retrieve a value from this [`Extensions`], returning an [`RpcResult`] for more
    /// convenient handling in rpc server implementations.
    fn try_get<T: Send + Sync + 'static>(&self) -> RpcResult<&T>;
}

impl ExtensionsExt for Extensions {
    fn try_get<T: Send + Sync + 'static>(&self) -> RpcResult<&T> {
        match self.get() {
            Some(t) => Ok(t),
            None => Err(ErrorObject::owned(
                -1,
                format!(
                    "failed to retrieve value of type {} from extensions",
                    std::any::type_name::<T>(),
                ),
                None::<()>,
            )),
        }
    }
}
