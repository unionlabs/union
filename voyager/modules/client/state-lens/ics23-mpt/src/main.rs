use ethereum_light_client_types::StorageProof;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use state_lens_ics23_mpt_light_client_types::{ClientState, ConsensusState};
use state_lens_light_client_types::Header;
use tracing::instrument;
use unionlabs::{
    self,
    encoding::{Bcs, Bincode, DecodeAs, EncodeAs, EthAbi},
    ibc::core::client::height::Height,
    primitives::Bytes,
};
use voyager_message::{
    into_value,
    module::{ClientModuleInfo, ClientModuleServer},
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface,
    },
    ClientModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SupportedIbcInterface {
    IbcSolidity,
    IbcCosmwasm,
    IbcMoveAptos,
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_SOLIDITY => Ok(SupportedIbcInterface::IbcSolidity),
            IbcInterface::IBC_COSMWASM => Ok(SupportedIbcInterface::IbcCosmwasm),
            IbcInterface::IBC_MOVE_APTOS => Ok(SupportedIbcInterface::IbcMoveAptos),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcSolidity => IbcInterface::IBC_SOLIDITY,
            SupportedIbcInterface::IbcCosmwasm => IbcInterface::IBC_COSMWASM,
            SupportedIbcInterface::IbcMoveAptos => IbcInterface::IBC_MOVE_APTOS,
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
#[serde(deny_unknown_fields)]
pub struct Config {}

impl ClientModule for Module {
    type Config = Config;

    async fn new(_: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ClientType::STATE_LENS_ICS23_MPT)?;
        info.ensure_consensus_type(ConsensusType::ETHEREUM)?;
        info.ensure_ibc_interface(IbcInterface::IBC_SOLIDITY)
            .or(info.ensure_ibc_interface(IbcInterface::IBC_COSMWASM))
            .or(info.ensure_ibc_interface(IbcInterface::IBC_MOVE_APTOS))?;

        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        ConsensusState::decode_as::<EthAbi>(consensus_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode consensus state: {err}"),
                None::<()>,
            )
        })
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => ClientState::decode_as::<EthAbi>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {err}"),
                        None::<()>,
                    )
                }),
            SupportedIbcInterface::IbcCosmwasm => ClientState::decode_as::<Bincode>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {err}"),
                        None::<()>,
                    )
                }),
            SupportedIbcInterface::IbcMoveAptos => ClientState::decode_as::<Bcs>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {err}"),
                        None::<()>,
                    )
                }),
        }
    }

    pub fn make_height(revision_height: u64) -> Height {
        Height::new(revision_height)
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    #[instrument]
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        let cs = self.decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            counterparty_chain_id: ChainId::new(cs.l2_chain_id.to_string()),
            counterparty_height: Module::make_height(cs.l2_latest_height),
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
            timestamp: cs.timestamp,
        })
    }

    #[instrument]
    async fn decode_client_state(&self, _: &Extensions, client_state: Bytes) -> RpcResult<Value> {
        Ok(into_value(self.decode_client_state(&client_state)?))
    }

    #[instrument]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        Ok(into_value(Module::decode_consensus_state(
            &consensus_state,
        )?))
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
                    format!("unable to deserialize client state: {err}"),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => cs.encode_as::<EthAbi>(),
                SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<Bincode>(),
                SupportedIbcInterface::IbcMoveAptos => cs.encode_as::<Bcs>(),
            })
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
                    format!("unable to deserialize consensus state: {err}"),
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
                    format!("unable to deserialize header: {err}"),
                    None::<()>,
                )
            })
            .map(|header| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => header.encode_as::<EthAbi>(),
                SupportedIbcInterface::IbcCosmwasm => header.encode_as::<Bincode>(),
                SupportedIbcInterface::IbcMoveAptos => header.encode_as::<Bcs>(),
            })
            .map(Into::into)
    }

    #[instrument]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        let proof = serde_json::from_value::<StorageProof>(proof).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to deserialize proof: {err}"),
                None::<()>,
            )
        })?;
        match self.ibc_interface {
            // TODO: extract to unionlabs? this is MPT proofs encoding for EVM
            // the solidity MPT verifier expects the proof RLP nodes to be serialized in sequence
            SupportedIbcInterface::IbcSolidity => Ok(proof.proof.into_iter().flatten().collect()),
            SupportedIbcInterface::IbcCosmwasm => Ok(proof.encode_as::<Bincode>().into()),
            SupportedIbcInterface::IbcMoveAptos => Ok(proof.encode_as::<Bcs>().into()),
        }
    }
}
