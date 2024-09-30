use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use macros::model;
use queue_msg::BoxDynError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, instrument};
use unionlabs::{
    self,
    encoding::{DecodeAs, EncodeAs, Proto},
    google::protobuf::any::Any,
    ibc::lightclients::tendermint::{
        client_state::ClientState, consensus_state::ConsensusState, header::Header,
    },
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface},
    module::{ClientModuleInfo, ClientModuleServer, ModuleInfo},
    run_module_server, DefaultCmd, ModuleContext, FATAL_JSONRPC_ERROR_CODE,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
}

#[model(no_serde)]
#[derive(Copy, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SupportedIbcInterface {
    IbcGoV8Native,
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_GO_V8_NATIVE => Ok(SupportedIbcInterface::IbcGoV8Native),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcGoV8Native => IbcInterface::IBC_GO_V8_NATIVE,
        }
    }
}

impl From<SupportedIbcInterface> for String {
    fn from(value: SupportedIbcInterface) -> Self {
        value.as_str().to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub ibc_interface: SupportedIbcInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ibc_interface: SupportedIbcInterface,
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ClientModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Self {
            ibc_interface: config.ibc_interface,
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            kind: ClientModuleInfo {
                client_type: ClientType::new(ClientType::TENDERMINT),
                consensus_type: ConsensusType::new(ConsensusType::TENDERMINT),
                ibc_interface: IbcInterface::new(config.ibc_interface.as_str()),
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

impl Module {
    pub fn decode_consensus_state(&self, consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcGoV8Native => {
                <Any<ConsensusState>>::decode_as::<Proto>(consensus_state)
                    .map_err(|err| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("unable to decode consensus state: {}", ErrorReporter(err)),
                            None::<()>,
                        )
                    })
                    .map(|any| any.0)
            }
        }
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcGoV8Native => {
                <Any<ClientState>>::decode_as::<Proto>(client_state)
                    .map_err(|err| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("unable to decode client state: {}", ErrorReporter(err)),
                            None::<()>,
                        )
                    })
                    .map(|any| any.0)
            }
        }
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    #[instrument(skip_all)]
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta> {
        let cs = self.decode_client_state(&client_state.0)?;

        Ok(ClientStateMeta {
            chain_id: ChainId::new(cs.chain_id),
            height: cs.latest_height,
        })
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = self.decode_consensus_state(&consensus_state.0)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.timestamp.as_unix_nanos(),
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_client_state(&client_state.0)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_consensus_state(&consensus_state.0)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
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
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcGoV8Native => Any(cs).encode_as::<Proto>(),
            })
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
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
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcGoV8Native => Any(cs).encode_as::<Proto>(),
            })
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(client_state)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(consensus_state)
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<Header>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|header| match self.ibc_interface {
                SupportedIbcInterface::IbcGoV8Native => Ok(Any(header).encode_as::<Proto>()),
            })?
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Hex<Vec<u8>>> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<unionlabs::ibc::core::commitment::merkle_proof::MerkleProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcGoV8Native => cs.encode_as::<Proto>(),
            })
            .map(Hex)
    }
}
