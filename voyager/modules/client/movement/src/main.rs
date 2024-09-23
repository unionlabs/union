use std::collections::VecDeque;

use chain_utils::BoxDynError;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::Op;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::instrument;
use unionlabs::{
    self,
    aptos::sparse_merkle_proof::SparseMerkleProof,
    encoding::{Bcs, DecodeAs, EncodeAs, Proto},
    google::protobuf::any::Any,
    ibc::{
        core::client::height::Height,
        lightclients::{
            cometbls,
            movement::{self, header::Header},
            wasm,
        },
    },
    ErrorReporter,
};
use voyager_message::{
    core::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, IbcGo08WasmClientMetadata,
        IbcInterface,
    },
    data::Data,
    module::{ClientModuleInfo, ClientModuleServer, ModuleInfo, QueueInteractionsServer},
    run_module_server, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

pub mod call;
pub mod callback;
pub mod data;

const SUPPORTED_IBC_INTERFACE: IbcInterface<'static> =
    IbcInterface::new_static(IbcInterface::IBC_GO_V8_08_WASM);

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module, _, _, _>().await
}

#[derive(Debug, Clone)]
pub struct Module {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

type SelfConsensusState =
    Any<wasm::consensus_state::ConsensusState<movement::consensus_state::ConsensusState>>;
type SelfClientState = Any<wasm::client_state::ClientState<movement::client_state::ClientState>>;

impl ModuleContext for Module {
    type Config = Config;

    type Cmd = DefaultCmd;

    type Info = ClientModuleInfo;

    async fn new(_config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Module {})
    }

    fn info(_config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(),
            kind: ClientModuleInfo {
                client_type: ClientType::new(ClientType::MOVEMENT),
                ibc_interface: SUPPORTED_IBC_INTERFACE,
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name() -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{SUPPORTED_IBC_INTERFACE}",)
}

impl Module {
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
            revision_number: 0, // TODO(aeryz): use chain_utils
            revision_height,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {}

#[async_trait]
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {}
    }

    #[instrument]
    async fn callback(
        &self,
        callback: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match callback {}
    }
}

#[async_trait]
impl ClientModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument]
    async fn supported_interface(&self) -> RpcResult<ClientModuleInfo> {
        Ok(ClientModuleInfo {
            client_type: ClientType::new_static(ClientType::MOVEMENT),
            ibc_interface: SUPPORTED_IBC_INTERFACE,
        })
    }

    #[instrument]
    async fn decode_client_state_meta(
        &self,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta> {
        let cs = Module::decode_client_state(&client_state.0)?;

        Ok(ClientStateMeta {
            chain_id: ChainId::new(cs.0.data.chain_id.to_string()),
            height: Module::make_height(cs.0.data.latest_block_num),
        })
    }

    #[instrument]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = Module::decode_consensus_state(&consensus_state.0)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.0.data.timestamp,
        })
    }

    #[instrument]
    async fn decode_client_state(&self, client_state: Hex<Vec<u8>>) -> RpcResult<Value> {
        Ok(serde_json::to_value(Module::decode_client_state(&client_state.0)?).unwrap())
    }

    #[instrument]
    async fn decode_consensus_state(&self, consensus_state: Hex<Vec<u8>>) -> RpcResult<Value> {
        Ok(serde_json::to_value(Module::decode_consensus_state(&consensus_state.0)?).unwrap())
    }

    #[instrument]
    async fn encode_client_state(
        &self,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        let IbcGo08WasmClientMetadata { checksum } =
            serde_json::from_value(metadata).map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize metadata: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?;

        serde_json::from_value::<movement::client_state::ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| {
                Any(wasm::client_state::ClientState {
                    latest_height: Module::make_height(cs.latest_block_num),
                    data: cs,
                    checksum,
                })
                .encode_as::<Proto>()
            })
            .map(Hex)
    }

    #[instrument]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<movement::consensus_state::ConsensusState>(consensus_state)
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
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Hex<Vec<u8>>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        match client_type.as_str() {
            ClientType::COMETBLS => Ok(Hex(Any(cometbls::client_state::ClientState::decode_as::<
                Bcs,
            >(&client_state.0)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to decode client state: {}", ErrorReporter(err)),
                    Some(json!({
                        "client_type": client_type,
                    })),
                )
            })?)
            .encode_as::<Proto>())),
            _ => Ok(client_state),
        }
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        // match client_type.as_str() {
        //     ClientType::COMETBLS => Ok(Hex(Any(wasm::consensus_state::ConsensusState {
        //         data: cometbls::consensus_state::ConsensusState::decode_as::<EthAbi>(
        //             &consensus_state.0,
        //         )
        //         .map_err(|err| {
        //             ErrorObject::owned(
        //                 FATAL_JSONRPC_ERROR_CODE,
        //                 format!("unable to decode client state: {}", ErrorReporter(err)),
        //                 Some(json!({
        //                     "client_type": client_type,
        //                 })),
        //             )
        //         })?,
        //     })
        //     .encode_as::<Proto>())),
        //     _ => Ok(consensus_state),
        // }
        Ok(consensus_state)
    }

    #[instrument]
    async fn encode_header(&self, header: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<Header>(header)
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
            .map(Hex)
    }

    #[instrument]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<SparseMerkleProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| cs.encode_as::<Proto>())
            .map(Hex)
    }
}
