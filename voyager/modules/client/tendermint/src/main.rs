use jsonrpsee::{Extensions, core::async_trait};
use macros::model;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tendermint_light_client_types::{ClientState, ConsensusState, Header};
use tracing::{debug, instrument};
use unionlabs::{
    encoding::{Bincode, DecodeAs, EncodeAs, EthAbi},
    primitives::Bytes,
};
use voyager_sdk::{
    anyhow::{self, anyhow},
    ensure_null, into_value,
    plugin::ClientModule,
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface,
        Timestamp,
    },
    rpc::{ClientModuleServer, RpcError, RpcResult, types::ClientModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[model(no_serde)]
#[derive(Copy, Serialize, Deserialize)]
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

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> anyhow::Result<Self> {
        info.ensure_client_type(ClientType::TENDERMINT)?;
        info.ensure_consensus_type(ConsensusType::TENDERMINT)?;

        // TODO: Verify the rest of the fields
        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())
                .map_err(|e| anyhow!("{e}"))?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(&self, consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcCosmwasm => {
                ConsensusState::decode_as::<EthAbi>(consensus_state)
                    .map_err(RpcError::fatal("unable to decode consensus state"))
            }
        }
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcCosmwasm => ClientState::decode_as::<Bincode>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
        }
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    #[instrument(skip_all)]
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        let client_state = self.decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            counterparty_chain_id: ChainId::new(client_state.chain_id),
            counterparty_height: client_state.latest_height,
        })
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta> {
        let consensus_state = self.decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp: Timestamp::from_nanos(consensus_state.timestamp.as_unix_nanos()),
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state(&self, _: &Extensions, client_state: Bytes) -> RpcResult<Value> {
        Ok(into_value(self.decode_client_state(&client_state)?))
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        Ok(into_value(self.decode_consensus_state(&consensus_state)?))
    }

    #[instrument(skip_all)]
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

    #[instrument(skip_all)]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(RpcError::fatal("unable to deserialize consensus state"))
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<EthAbi>().into(),
            })
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(RpcError::fatal("unable to deserialize header"))
            .map(|header| match self.ibc_interface {
                SupportedIbcInterface::IbcCosmwasm => header.encode_as::<Bincode>().into(),
            })
    }

    #[instrument]
    async fn decode_header(&self, _: &Extensions, header: Bytes) -> RpcResult<Value> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcCosmwasm => Header::decode_as::<Bincode>(&header)
                .map_err(RpcError::fatal("unable to decode header")),
        }
        .map(into_value)
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<unionlabs::ibc::core::commitment::merkle_proof::MerkleProof>(proof)
            .map_err(RpcError::fatal("unable to deserialize proof"))
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<Bincode>().into(),
            })
    }

    #[instrument]
    async fn decode_proof(&self, _: &Extensions, proof: Bytes) -> RpcResult<Value> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcCosmwasm => {
                unionlabs::ibc::core::commitment::merkle_proof::MerkleProof::decode_as::<Bincode>(
                    &proof,
                )
                .map(into_value)
                .map_err(RpcError::fatal("unable to decode proof"))
            }
        }
    }
}
