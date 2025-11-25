use base_light_client_types::{ClientState, ConsensusState, Header};
use ethereum_light_client_types::StorageProof;
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::{
    encoding::{Bincode, DecodeAs, EncodeAs, EthAbi},
    ibc::core::client::height::Height,
    primitives::Bytes,
};
use voyager_sdk::{
    anyhow, ensure_null, into_value,
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

#[derive(Debug, Clone)]
pub struct Module {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl ClientModule for Module {
    type Config = Config;

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> anyhow::Result<Self> {
        info.ensure_client_type(ClientType::BASE)
            .or_else(|_| info.ensure_client_type(ClientType::OPTIMISM))?;
        info.ensure_consensus_type(ConsensusType::BASE)
            .or_else(|_| info.ensure_consensus_type(ConsensusType::OPTIMISM))?;
        info.ensure_ibc_interface(IbcInterface::IBC_COSMWASM)?;

        Ok(Self {})
    }
}

impl Module {
    pub fn decode_client_state(client_state: &[u8]) -> RpcResult<ClientState> {
        ClientState::decode_as::<Bincode>(client_state)
            .map_err(RpcError::fatal("unable to decode client state"))
    }

    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        ConsensusState::decode_as::<EthAbi>(consensus_state)
            .map_err(RpcError::fatal("unable to decode consensus state"))
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
        match Module::decode_client_state(&client_state)? {
            ClientState::V1(v1) => Ok(ClientStateMeta {
                counterparty_chain_id: ChainId::new(v1.chain_id.to_string()),
                counterparty_height: Height::new(v1.latest_height),
            }),
        }
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
        Ok(into_value(Module::decode_client_state(&client_state)?))
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
            .map(|client_state| client_state.encode_as::<Bincode>().into())
    }

    #[instrument]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(RpcError::fatal("unable to deserialize consensus state"))
            .map(|consensus_state| consensus_state.encode_as::<EthAbi>().into())
    }

    #[instrument]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(RpcError::fatal("unable to deserialize header"))
            .map(|header| header.encode_as::<Bincode>().into())
    }

    #[instrument]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<StorageProof>(proof)
            .map_err(RpcError::fatal("unable to deserialize storage proof"))
            .map(|storage_proof| storage_proof.encode_as::<Bincode>().into())
    }
}
