use jsonrpsee::{self, core::RpcResult, proc_macros::rpc};
use macros::model;
use serde_json::Value;
use unionlabs::{ibc::core::client::height::Height, ics24::Path, id::ClientId, QueryHeight};

use crate::{data::ClientInfo, ChainId, ClientType, IbcInterface};

#[rpc(
    client,
    server,
    client_bounds(Self: Send + Sync),
    server_bounds(Self:),
    namespace = "voyager",
)]
pub trait VoyagerRpc {
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<Info>;

    #[method(name = "queryLatestHeight")]
    async fn query_latest_height(&self, chain_id: ChainId<'static>) -> RpcResult<Height>;

    #[method(name = "clientInfo")]
    async fn client_info(
        &self,
        chain_id: ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo>;

    #[method(name = "queryibcState")]
    async fn query_ibc_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcState>;

    #[method(name = "queryibcProof")]
    async fn query_ibc_proof(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcProof>;

    #[method(name = "selfClientState")]
    async fn self_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfClientState>;

    #[method(name = "selfConsensusState")]
    async fn self_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState>;
}

#[model]
pub struct Info {
    chain: Vec<ChainId<'static>>,
    consensus: Vec<ChainId<'static>>,
    client: Vec<(ClientType<'static>, IbcInterface<'static>)>,
}

#[model]
pub struct IbcState {
    pub chain_id: ChainId<'static>,
    pub path: Path,
    /// The height that the state was read at.
    pub height: Height,
    pub state: Value,
}

#[model]
pub struct IbcProof {
    pub chain_id: ChainId<'static>,
    pub path: Path,
    /// The height that the proof was read at.
    pub height: Height,
    pub proof: Value,
}

#[model]
pub struct SelfClientState {
    pub height: Height,
    pub state: Value,
}

#[model]
pub struct SelfConsensusState {
    pub height: Height,
    pub state: Value,
}

// State(FetchState),
// RawProof(FetchRawProof),

// LatestHeight(FetchLatestHeight),

// ClientInfo(FetchClientInfo),

// UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState),

// SelfClientState(FetchSelfClientState),
// SelfConsensusState(FetchSelfConsensusState),

// DecodeClientStateMeta(DecodeClientStateMeta),
// DecodeConsensusStateMeta(DecodeConsensusStateMeta),

// EncodeClientState(EncodeClientState),
// EncodeConsensusState(EncodeConsensusState),
// EncodeHeader(EncodeHeader),

// EncodeProof(EncodeProof),

// UpdateHeaders(FetchUpdateHeaders),

// MakeMsgCreateClient(MakeMsgCreateClient),

// MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry),
// MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck),
// MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm),

// MakeMsgChannelOpenTry(MakeMsgChannelOpenTry),
// MakeMsgChannelOpenAck(MakeMsgChannelOpenAck),
// MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm),

// MakeMsgAcknowledgement(MakeMsgAcknowledgement),
// MakeMsgRecvPacket(MakeMsgRecvPacket),

// Height(WaitForHeight),
// HeightRelative(WaitForHeightRelative),
// Timestamp(WaitForTimestamp),
// TrustedHeight(WaitForTrustedHeight),

// Plugin(PluginMessage<C>),

pub mod server {
    use std::sync::{Arc, OnceLock};

    use jsonrpsee::{
        core::RpcResult,
        types::{ErrorObject, ErrorObjectOwned},
    };
    use serde_json::Value;
    use tonic::async_trait;
    use tracing::{debug, instrument};
    use unionlabs::{
        ibc::core::client::height::Height, ics24::Path, id::ClientId, ErrorReporter, QueryHeight,
    };

    use super::{IbcProof, IbcState, SelfClientState, SelfConsensusState, VoyagerRpcServer};
    use crate::{
        context::Modules,
        data::ClientInfo,
        plugin::{ChainModuleClient, ConsensusModuleClient},
        rpc::Info,
        ChainId, FATAL_JSONRPC_ERROR_CODE,
    };

    #[derive(Clone)]
    pub struct Server(Arc<ServerInner>);

    #[derive(Clone)]
    pub struct ServerInner {
        modules: OnceLock<Arc<Modules>>,
    }

    impl Server {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Server(Arc::new(ServerInner {
                modules: OnceLock::new(),
            }))
        }

        pub fn start(&self, modules: Arc<Modules>) {
            assert!(
                self.0.modules.set(modules).is_ok(),
                "server has already been started"
            );
        }

        fn modules(&self) -> RpcResult<&Modules> {
            self.0
                .modules
                .get()
                .map(|x| &**x)
                .ok_or_else(|| ErrorObject::owned(-2, "server has not been started", None::<()>))
        }
    }

    impl Server {
        #[instrument(skip_all, fields(%height, %chain_id))]
        async fn fetch_query_height(
            &self,
            chain_id: &ChainId<'_>,
            height: QueryHeight,
        ) -> RpcResult<Height> {
            match height {
                QueryHeight::Latest => {
                    let latest_height = self
                        .modules()?
                        .chain_module::<Value, Value, Value>(chain_id)
                        .map_err(fatal_error)?
                        .query_latest_height()
                        .await
                        .map_err(fatal_error)?;

                    debug!(%latest_height, "queried latest height");

                    Ok(latest_height)
                }
                QueryHeight::Specific(height) => Ok(height),
            }
        }
    }

    #[async_trait]
    impl VoyagerRpcServer for Server {
        #[instrument(skip_all)]
        async fn info(&self) -> RpcResult<Info> {
            let chain = self.modules()?.loaded_chain_modules().cloned().collect();
            let consensus = self
                .modules()?
                .loaded_consensus_modules()
                .cloned()
                .collect();
            let client = self
                .modules()?
                .loaded_client_modules()
                .flat_map(|(c, is)| is.map(|i| (c.clone(), i.clone())))
                .collect();

            Ok(Info {
                chain,
                consensus,
                client,
            })
        }

        #[instrument(skip_all, fields(%chain_id))]
        async fn query_latest_height(&self, chain_id: ChainId<'static>) -> RpcResult<Height> {
            debug!("querying latest height");

            let latest_height = self
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?
                .query_latest_height()
                .await
                .map_err(fatal_error)?;

            debug!(%latest_height, "queried latest height");

            Ok(latest_height)
        }

        #[instrument(skip_all, fields(%chain_id, %client_id))]
        async fn client_info(
            &self,
            chain_id: ChainId<'static>,
            client_id: ClientId,
        ) -> RpcResult<ClientInfo> {
            debug!("fetching client info");

            let client_info = <_ as ChainModuleClient<Value, Value, Value>>::client_info(
                self.modules()?
                    .chain_module(&chain_id)
                    .map_err(fatal_error)?,
                client_id,
            )
            .await
            .map_err(fatal_error)?;

            debug!(%client_info.ibc_interface, %client_info.client_type, "fetched client info");

            Ok(client_info)
        }

        #[instrument(skip_all, fields(%chain_id, %path, %height))]
        async fn query_ibc_state(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
            path: Path,
        ) -> RpcResult<IbcState> {
            debug!("fetching ibc state");

            let chain_module = self
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.fetch_query_height(&chain_id, height).await?;

            let state = chain_module
                .query_ibc_state(height, path.clone())
                .await
                .map_err(fatal_error)?;

            // TODO: Use valuable here
            debug!(%state, "fetched ibc state");

            Ok(IbcState {
                chain_id,
                path,
                height,
                state,
            })
        }

        #[instrument(skip_all, fields(%chain_id, %path, %height))]
        async fn query_ibc_proof(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
            path: Path,
        ) -> RpcResult<IbcProof> {
            debug!("fetching ibc state");

            let chain_module = self
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.fetch_query_height(&chain_id, height).await?;

            let proof = chain_module
                .query_ibc_proof(height, path.clone())
                .await
                .map_err(fatal_error)?;

            // TODO: Use valuable here
            debug!(%proof, "fetched ibc proof");

            Ok(IbcProof {
                chain_id,
                path,
                height,
                proof,
            })
        }

        #[instrument(skip_all, fields(%chain_id, %height))]
        async fn self_client_state(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
        ) -> RpcResult<SelfClientState> {
            debug!("querying self client state");

            let chain_module = self
                .modules()?
                .consensus_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.fetch_query_height(&chain_id, height).await?;

            let state = chain_module
                .self_client_state(height)
                .await
                .map_err(fatal_error)?;

            // TODO: Use valuable here
            debug!(%state, "fetched self client state");

            Ok(SelfClientState { height, state })
        }

        #[instrument(skip_all, fields(%chain_id, %height))]
        async fn self_consensus_state(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
        ) -> RpcResult<SelfConsensusState> {
            debug!("querying self consensus state");

            let chain_module = self
                .modules()?
                .consensus_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.fetch_query_height(&chain_id, height).await?;

            let state = chain_module
                .self_consensus_state(height)
                .await
                .map_err(fatal_error)?;

            // TODO: Use valuable here
            debug!(%state, "fetched self consensus state");

            Ok(SelfConsensusState { height, state })
        }
    }

    pub(crate) fn fatal_error(t: impl std::error::Error) -> ErrorObjectOwned {
        ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            ErrorReporter(t).to_string(),
            None::<()>,
        )
    }
}
