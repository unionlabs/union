use std::{borrow::Cow, collections::VecDeque};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::Op;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, instrument, warn};
use unionlabs::{
    self,
    encoding::{DecodeAs, EncodeAs, EthAbi},
    ibc::lightclients::cometbls::{
        client_state::ClientState, consensus_state::ConsensusState, header::Header,
    },
    ErrorReporter,
};
use voyager_message::{
    data::Data,
    plugin::{
        ClientModuleServer, ClientStateMeta, ConsensusStateMeta, PluginInfo, PluginKind,
        PluginModuleServer, SupportedInterface,
    },
    run_module_server, ClientType, IbcInterface, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{aggregate::ModuleAggregate, data::ModuleData, fetch::ModuleFetch};

pub mod aggregate;
pub mod data;
pub mod fetch;

const SUPPORTED_CLIENT_TYPE: ClientType<'static> = ClientType::new_static(ClientType::COMETBLS);
const SUPPORTED_IBC_INTERFACE: IbcInterface<'static> =
    IbcInterface::new_static(IbcInterface::IBC_SOLIDITY);

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(Module::new, ClientModuleServer::into_rpc).await
}

#[derive(Debug, Clone)]
pub struct Module {
    // REVIEW: Make configurable?
    // pub client_type: ClientType<'static>,
    // pub ibc_interface: IbcInterface<'static>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // REVIEW: Make configurable?
    // pub client_type: ClientType<'static>,
    // pub ibc_interface: IbcInterface<'static>,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{SUPPORTED_CLIENT_TYPE}/{SUPPORTED_IBC_INTERFACE}")
    }

    pub async fn new(_config: Config) -> Result<Self, ModuleInitError> {
        Ok(Self {})
    }

    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        ConsensusState::decode_as::<EthAbi>(consensus_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode consensus state: {}", ErrorReporter(err)),
                None::<()>,
            )
        })
    }

    pub fn decode_client_state(client_state: &[u8]) -> RpcResult<ClientState> {
        ClientState::decode_as::<EthAbi>(client_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode client state: {}", ErrorReporter(err)),
                None::<()>,
            )
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleFetch, ModuleAggregate> for Module {
    #[instrument(skip_all)]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Client),
            interest_filter: None,
        })
    }

    #[instrument(skip_all)]
    async fn handle_fetch(
        &self,
        msg: ModuleFetch,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>> {
        match msg {}
    }

    #[instrument(skip_all)]
    fn handle_aggregate(
        &self,
        aggregate: ModuleAggregate,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>> {
        match aggregate {}
    }
}

#[async_trait]
impl ClientModuleServer<ModuleData, ModuleFetch, ModuleAggregate> for Module {
    #[instrument(skip_all)]
    async fn supported_interface(&self) -> RpcResult<SupportedInterface> {
        Ok(SupportedInterface {
            client_type: SUPPORTED_CLIENT_TYPE,
            ibc_interface: SUPPORTED_IBC_INTERFACE,
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state_meta(
        &self,
        client_state: Cow<'static, [u8]>,
    ) -> RpcResult<ClientStateMeta> {
        let cs = Self::decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            chain_id: cs.chain_id,
            height: cs.latest_height,
        })
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Cow<'static, [u8]>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = Self::decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.timestamp,
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state(&self, client_state: Cow<'static, [u8]>) -> RpcResult<Value> {
        Ok(serde_json::to_value(Self::decode_client_state(&client_state)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state(
        &self,
        consensus_state: Cow<'static, [u8]>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(Self::decode_consensus_state(&consensus_state)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn encode_client_state(
        &self,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Vec<u8>> {
        if !metadata.is_null() {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "metadata was provided, but this client type does not require metadata for client \
                state encoding",
                Some(json!({
                    "provided_metadata": metadata,
                })),
            ));
        }

        serde_json::from_value::<ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| cs.encode_as::<EthAbi>())
    }

    #[instrument(skip_all)]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Vec<u8>> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!(
                        "unable to deserialize consensus state: {}",
                        ErrorReporter(err)
                    ),
                    None::<()>,
                )
            })
            .map(|cs| cs.encode_as::<EthAbi>())
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Cow<'static, [u8]>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Vec<u8>> {
        Ok(client_state.into())
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Cow<'static, [u8]>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Vec<u8>> {
        Ok(consensus_state.into())
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, header: Value) -> RpcResult<Vec<u8>> {
        serde_json::from_value::<Header>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| cs.encode_as::<EthAbi>())
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Vec<u8>> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<unionlabs::ibc::core::commitment::merkle_proof::MerkleProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| {
                unionlabs::union::ics23::merkle_proof::MerkleProof::try_from(
                    protos::ibc::core::commitment::v1::MerkleProof::from(cs),
                )
                .unwrap()
                .encode_as::<EthAbi>()
            })
    }
}
