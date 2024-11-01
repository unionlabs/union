use chain_utils::BoxDynError;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use movement_light_client_types::{ClientState, ConsensusState, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_utils::Hex;
use tracing::instrument;
use unionlabs::{
    self,
    aptos::storage_proof::StorageProof,
    encoding::{DecodeAs, EncodeAs, Proto},
    google::protobuf::any::Any,
    ibc::{core::client::height::Height, lightclients::wasm},
    ErrorReporter,
};
use voyager_message::{
    core::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType,
        IbcGo08WasmClientMetadata, IbcInterface,
    },
    module::{ClientModuleInfo, ClientModuleServer},
    run_client_module_server, ClientModule, FATAL_JSONRPC_ERROR_CODE,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_client_module_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

type SelfConsensusState = Any<
    wasm::consensus_state::ConsensusState<
        movement_light_client_types::consensus_state::ConsensusState,
    >,
>;
type SelfClientState =
    Any<wasm::client_state::ClientState<movement_light_client_types::client_state::ClientState>>;

impl ClientModule for Module {
    type Config = Config;

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ClientType::MOVEMENT)?;
        info.ensure_consensus_type(ConsensusType::MOVEMENT)?;
        info.ensure_ibc_interface(IbcInterface::IBC_GO_V8_08_WASM)?;

        Ok(Module {})
    }
}

impl Module {
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
impl ClientModuleServer for Module {
    #[instrument]
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Bytes,
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
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = Module::decode_consensus_state(&consensus_state.0)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.0.data.timestamp,
        })
    }

    #[instrument]
    async fn decode_client_state(&self, _: &Extensions, client_state: Bytes) -> RpcResult<Value> {
        Ok(serde_json::to_value(Module::decode_client_state(&client_state.0)?).unwrap())
    }

    #[instrument]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(Module::decode_consensus_state(&consensus_state.0)?).unwrap())
    }

    #[instrument]
    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
        let IbcGo08WasmClientMetadata { checksum } =
            serde_json::from_value(metadata).map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize metadata: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?;

        serde_json::from_value::<ClientState>(client_state)
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
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
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
            .map(|cs| Any(wasm::consensus_state::ConsensusState { data: cs }).encode_as::<Proto>())
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        _: &Extensions,
        _client_state: Bytes,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Bytes> {
        // match client_type.as_str() {
        //     ClientType::COMETBLS_GROTH16 => {
        //         Ok(Hex(Any(cometbls::client_state::ClientState::decode_as::<
        //             Bcs,
        //         >(&client_state.0)
        //         .map_err(|err| {
        //             ErrorObject::owned(
        //                 FATAL_JSONRPC_ERROR_CODE,
        //                 format!("unable to decode client state: {}", ErrorReporter(err)),
        //                 Some(json!({
        //                     "client_type": client_type,
        //                 })),
        //             )
        //         })?)
        //         .encode_as::<Proto>()))
        //     }
        //     _ => Ok(client_state),
        // }

        todo!()
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Bytes> {
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
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
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
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<StorageProof>(proof)
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
