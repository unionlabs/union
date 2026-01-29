use aptos_types::{sparse_merkle_proof::SparseMerkleProof, storage_proof::StorageProof};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state_lens_ics23_smt_light_client_types::{ClientState, ConsensusState};
use state_lens_light_client_types::Header;
use tracing::instrument;
use unionlabs::{
    encoding::{Bincode, DecodeAs, EncodeAs, EthAbi},
    ibc::core::client::height::Height,
    primitives::Bytes,
};
use voyager_sdk::{
    anyhow::{self, anyhow},
    ensure_null, into_value,
    plugin::ClientModule,
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface,
    },
    rpc::{ClientModuleServer, RpcError, RpcResult, types::ClientModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SupportedIbcInterface {
    IbcCosmwasm,
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_COSMWASM => Ok(SupportedIbcInterface::IbcCosmwasm),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcCosmwasm => IbcInterface::IBC_COSMWASM,
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

    async fn new(_: Self::Config, info: ClientModuleInfo) -> anyhow::Result<Self> {
        info.ensure_client_type(ClientType::STATE_LENS_ICS23_SMT)?;
        info.ensure_consensus_type(ConsensusType::MOVEMENT)?;
        info.ensure_ibc_interface(IbcInterface::IBC_SOLIDITY)
            .or(info.ensure_ibc_interface(IbcInterface::IBC_COSMWASM))?;

        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())
                .map_err(|e| anyhow!(e))?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        ConsensusState::decode_as::<EthAbi>(consensus_state)
            .map_err(RpcError::fatal("unable to decode consensus state"))
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcCosmwasm => <ClientState>::decode_as::<Bincode>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
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
        let client_state = self.decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            counterparty_chain_id: ChainId::new(client_state.l2_chain_id),
            counterparty_height: Module::make_height(client_state.l2_latest_height),
        })
    }

    #[instrument]
    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta> {
        let consensus_state = Module::decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp: consensus_state.timestamp,
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
        ensure_null(metadata)?;

        serde_json::from_value::<ClientState>(client_state)
            .map_err(RpcError::fatal("unable to deserialize client state"))
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<Bincode>().into(),
            })
    }

    #[instrument]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(RpcError::fatal("unable to deserialize consensus state"))
            .map(|cs| cs.encode_as::<EthAbi>().into())
    }

    #[instrument]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(RpcError::fatal("unable to deserialize header"))
            .map(|header| match self.ibc_interface {
                SupportedIbcInterface::IbcCosmwasm => header.encode_as::<Bincode>().into(),
            })
    }

    #[instrument]
    async fn encode_proof(&self, _: &Extensions, _proof: Value) -> RpcResult<Bytes> {
        // TODO(aeryz): we cannot verify the proofs on evm yet, this will be mock until we have cw
        Ok(StorageProof {
            state_value: None,
            proof: SparseMerkleProof {
                leaf: None,
                siblings: vec![],
            },
        }
        .encode_as::<Bincode>()
        .into())
        // let proof = serde_json::from_value::<StorageProof>(proof).map_err(|err| {
        //     ErrorObject::owned(
        //         FATAL_JSONRPC_ERROR_CODE,
        //         format!("unable to deserialize proof: {err}"),
        //         None::<()>,
        //     )
        // })?;
        // match self.ibc_interface {
        //     SupportedIbcInterface::IbcCosmwasm => Ok(proof.encode_as::<Bincode>().into()),
        // }
    }

    #[instrument]
    async fn decode_header(&self, _: &Extensions, header: Bytes) -> RpcResult<Value> {
        Header::decode_as::<Bincode>(&header)
            .map(into_value)
            .map_err(RpcError::fatal("unable to decode header"))
    }

    #[instrument]
    async fn decode_proof(&self, _: &Extensions, proof: Bytes) -> RpcResult<Value> {
        StorageProof::decode_as::<Bincode>(&proof)
            .map(into_value)
            .map_err(RpcError::fatal("unable to proof"))
    }
}
