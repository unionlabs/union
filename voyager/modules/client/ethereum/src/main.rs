use std::{borrow::Cow, collections::VecDeque};

use chain_utils::ethereum::ETHEREUM_REVISION_NUMBER;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::Op;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{instrument, warn};
use unionlabs::{
    self,
    encoding::{DecodeAs, EncodeAs, EthAbi, Proto},
    google::protobuf::any::Any,
    ibc::{
        core::client::height::Height,
        lightclients::{
            cometbls,
            ethereum::{self, header::UnboundedHeader, storage_proof::StorageProof},
            wasm,
        },
    },
    ErrorReporter,
};
use voyager_message::{
    data::Data,
    plugin::{
        ClientModuleServer, ClientStateMeta, ConsensusStateMeta, IbcGo08WasmClientMetadata,
        PluginInfo, PluginKind, PluginModuleServer, SupportedInterface,
    },
    run_module_server, ChainId, ClientType, IbcInterface, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

pub mod call;
pub mod callback;
pub mod data;

// TODO: Thread this value through the config (or something similar)
const SUPPORTED_CLIENT_TYPE: ClientType<'static> =
    ClientType::new_static(ClientType::ETHEREUM_MINIMAL);
const SUPPORTED_IBC_INTERFACE: IbcInterface<'static> =
    IbcInterface::new_static(IbcInterface::IBC_GO_V8_08_WASM);

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(
        Module::new,
        ClientModuleServer::into_rpc,
        voyager_message::default_subcommand_handler,
    )
    .await
}

#[derive(Debug, Clone)]
pub struct Module {
    // TODO: Make configurable?
    // pub client_type: ClientType<'static>,
    // pub ibc_interface: IbcInterface<'static>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

type SelfConsensusState =
    Any<wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>>;
type SelfClientState = Any<wasm::client_state::ClientState<ethereum::client_state::ClientState>>;

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{SUPPORTED_CLIENT_TYPE}/{SUPPORTED_IBC_INTERFACE}")
    }

    pub async fn new(_config: Config, _voyager_config: String) -> Result<Self, ModuleInitError> {
        Ok(Self {})
    }

    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<SelfConsensusState> {
        SelfConsensusState::decode_as::<Proto>(consensus_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode consensus state: {}", ErrorReporter(err)),
                None::<()>,
            )
        })
    }

    pub fn decode_client_state(client_state: &[u8]) -> RpcResult<SelfClientState> {
        <SelfClientState>::decode_as::<Proto>(client_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode client state: {}", ErrorReporter(err)),
                None::<()>,
            )
        })
    }

    pub fn make_height(revision_height: u64) -> Height {
        Height {
            revision_number: ETHEREUM_REVISION_NUMBER,
            revision_height,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Client),
            interest_filter: None,
        })
    }

    #[instrument]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {}
    }

    #[instrument]
    fn callback(
        &self,
        callback: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match callback {}
    }
}

#[async_trait]
impl ClientModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument]
    async fn supported_interface(&self) -> RpcResult<SupportedInterface> {
        Ok(SupportedInterface {
            client_type: SUPPORTED_CLIENT_TYPE,
            ibc_interface: SUPPORTED_IBC_INTERFACE,
        })
    }

    #[instrument]
    async fn decode_client_state_meta(
        &self,
        client_state: Cow<'static, [u8]>,
    ) -> RpcResult<ClientStateMeta> {
        let cs = Self::decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            chain_id: ChainId::new(cs.0.data.chain_id.to_string()),
            height: Self::make_height(cs.0.data.latest_slot),
        })
    }

    #[instrument]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Cow<'static, [u8]>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = Self::decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.0.data.timestamp,
        })
    }

    #[instrument]
    async fn decode_client_state(&self, client_state: Cow<'static, [u8]>) -> RpcResult<Value> {
        Ok(serde_json::to_value(Self::decode_client_state(&client_state)?).unwrap())
    }

    #[instrument]
    async fn decode_consensus_state(
        &self,
        consensus_state: Cow<'static, [u8]>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(Self::decode_consensus_state(&consensus_state)?).unwrap())
    }

    #[instrument]
    async fn encode_client_state(
        &self,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Vec<u8>> {
        let IbcGo08WasmClientMetadata { checksum } =
            serde_json::from_value(metadata).map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize metadata: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?;

        serde_json::from_value::<ethereum::client_state::ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| {
                Any(wasm::client_state::ClientState {
                    latest_height: Self::make_height(cs.latest_slot),
                    data: cs,
                    checksum,
                })
                .encode_as::<Proto>()
            })
    }

    #[instrument]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Vec<u8>> {
        serde_json::from_value::<ethereum::consensus_state::ConsensusState>(consensus_state)
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
            .map(|cs| Any(wasm::consensus_state::ConsensusState { data: cs }).encode_as::<Proto>())
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Cow<'static, [u8]>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Vec<u8>> {
        match client_type.as_str() {
            ClientType::COMETBLS => Ok(Any(
                cometbls::client_state::ClientState::decode_as::<EthAbi>(&client_state).map_err(
                    |err| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("unable to decode client state: {}", ErrorReporter(err)),
                            Some(json!({
                                "client_type": client_type,
                            })),
                        )
                    },
                )?,
            )
            .encode_as::<Proto>()),
            _ => Ok(client_state.into()),
        }
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Cow<'static, [u8]>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Vec<u8>> {
        match client_type.as_str() {
            ClientType::COMETBLS => Ok(Any(wasm::consensus_state::ConsensusState {
                data: cometbls::consensus_state::ConsensusState::decode_as::<EthAbi>(
                    &consensus_state,
                )
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        Some(json!({
                            "client_type": client_type,
                        })),
                    )
                })?,
            })
            .encode_as::<Proto>()),
            _ => Ok(consensus_state.into()),
        }
    }

    #[instrument]
    async fn encode_header(&self, header: Value) -> RpcResult<Vec<u8>> {
        serde_json::from_value::<UnboundedHeader>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|header| {
                Any(wasm::client_message::ClientMessage { data: header }).encode_as::<Proto>()
            })
    }

    #[instrument]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Vec<u8>> {
        serde_json::from_value::<StorageProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| cs.encode_as::<Proto>())
    }
}
