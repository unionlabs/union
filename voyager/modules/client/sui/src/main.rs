use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sui_light_client_types::{
    client_state::ClientState, consensus_state::ConsensusState, header::Header,
};
use tracing::instrument;
use unionlabs::{
    self,
    encoding::{Bincode, DecodeAs, EncodeAs, EthAbi},
    ibc::core::client::height::Height,
    primitives::Bytes,
    ErrorReporter,
};
use voyager_message::{
    module::{ClientModuleInfo, ClientModuleServer},
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface,
        Timestamp,
    },
    vm::BoxDynError,
    ClientModule, FATAL_JSONRPC_ERROR_CODE,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl ClientModule for Module {
    type Config = Config;

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ClientType::SUI)?;
        info.ensure_consensus_type(ConsensusType::SUI)?;
        info.ensure_ibc_interface(IbcInterface::IBC_COSMWASM)?;

        Ok(Module {})
    }
}

impl Module {
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
        <ClientState>::decode_as::<Bincode>(client_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode client state: {err}"),
                None::<()>,
            )
        })
    }

    pub fn make_height(revision_height: u64) -> Height {
        Height::new(revision_height)
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
        let ClientState::V1(cs) = Module::decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            counterparty_chain_id: ChainId::new(cs.chain_id.to_string()),
            counterparty_height: Module::make_height(cs.latest_checkpoint),
        })
    }

    #[instrument]
    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = Module::decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp: Timestamp::from_nanos(cs.timestamp),
        })
    }

    #[instrument]
    async fn decode_client_state(&self, _: &Extensions, client_state: Bytes) -> RpcResult<Value> {
        Ok(serde_json::to_value(Module::decode_client_state(&client_state)?).unwrap())
    }

    #[instrument]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(Module::decode_consensus_state(&consensus_state)?).unwrap())
    }

    #[instrument]
    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
        if !metadata.is_null() {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "metadata was provided, but this client type does not require \
                metadata for client state encoding",
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
            .map(|cs| cs.encode_as::<Bincode>())
            .map(Into::into)
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
            .map(|cs| cs.encode_as::<EthAbi>())
            .map(Into::into)
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
            .map(|header| header.encode_as::<Bincode>())
            .map(Into::into)
    }

    #[instrument]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| cs.encode_as::<Bincode>())
            .map(Into::into)
    }
}
