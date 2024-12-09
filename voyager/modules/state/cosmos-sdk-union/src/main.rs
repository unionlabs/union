// #![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::ParseIntError,
    sync::Arc,
};

use cometbft_rpc::types::abci::response_query::QueryResponse;
use dashmap::DashMap;
use ibc_solidity::{Channel, Connection};
use ibc_union_spec::{IbcUnion, StorePath};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use prost::Message;
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, instrument};
use unionlabs::{
    bech32::Bech32,
    bytes::Bytes,
    hash::{hash_v2::Base64, H256},
    ibc::core::client::height::Height,
    ErrorReporter, WasmClientType,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    into_value,
    module::{StateModuleInfo, StateModuleServer},
    StateModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as StateModule<IbcUnion>>::run().await;
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub chain_revision: u64,

    pub tm_client: cometbft_rpc::Client,
    pub grpc_url: String,

    pub ibc_host_contract_address: Bech32<H256>,

    pub checksum_cache: Arc<DashMap<H256, WasmClientType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub ws_url: String,
    pub grpc_url: String,
    pub ibc_host_contract_address: Bech32<H256>,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| ChainIdParseError {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| ChainIdParseError {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            grpc_url: config.grpc_url,
            ibc_host_contract_address: config.ibc_host_contract_address,
            checksum_cache: Arc::new(DashMap::default()),
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }

    pub async fn query_smart<Q: Serialize, R: DeserializeOwned>(
        &self,
        query: &Q,
        height: Option<Height>,
    ) -> RpcResult<Option<R>> {
        let response = self
            .abci_query(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                QuerySmartContractStateRequest {
                    address: self.ibc_host_contract_address.to_string(),
                    query_data: serde_json::to_vec(query).unwrap(),
                }
                .encode_to_vec()
                .into(),
                height,
            )
            .await?;

        Ok(response.value.map(|value| {
            serde_json::from_slice(
                &QuerySmartContractStateResponse::decode(&*value)
                    .unwrap()
                    .data,
            )
            .unwrap()
        }))
    }

    async fn abci_query(
        &self,
        path: &str,
        data: Bytes,
        height: Option<Height>,
    ) -> RpcResult<QueryResponse> {
        self.tm_client
            .abci_query(
                &path,
                &data,
                height.map(|height| {
                    i64::try_from(height.height())
                        .expect("should be fine")
                        .try_into()
                        .expect("invalid height")
                }),
                false,
            )
            .await
            .map_err(rpc_error(
                format_args!("error fetching abci query"),
                Some(json!({ "height": height, "path": data })),
            ))
            .map(|response| response.response)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(&self, height: Height, client_id: u32) -> RpcResult<Option<Bytes>> {
        let client_state = self
            .query_smart::<_, Bytes<Base64>>(
                &union_ibc_msg::query::QueryMsg::GetClientState { client_id },
                Some(height),
            )
            .await?;

        Ok(client_state.map(Bytes::into_encoding))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_consensus_state(
        &self,
        height: Height,
        client_id: u32,
        trusted_height: u64,
    ) -> RpcResult<Option<Bytes>> {
        let client_state = self
            .query_smart::<_, Bytes<Base64>>(
                &union_ibc_msg::query::QueryMsg::GetConsensusState {
                    client_id,
                    height: trusted_height,
                },
                Some(height),
            )
            .await?;

        Ok(client_state.map(Bytes::into_encoding))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: u32,
    ) -> RpcResult<Option<Connection>> {
        let client_state = self
            .query_smart::<_, Connection>(
                &union_ibc_msg::query::QueryMsg::GetConnection { connection_id },
                Some(height),
            )
            .await?;

        Ok(client_state)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_channel(&self, height: Height, channel_id: u32) -> RpcResult<Option<Channel>> {
        let channel = self
            .query_smart::<_, Channel>(
                &union_ibc_msg::query::QueryMsg::GetChannel { channel_id },
                Some(height),
            )
            .await?;

        Ok(channel)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id, %batch_hash))]
    async fn query_batch_packets(
        &self,
        height: Height,
        channel_id: u32,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let commitment = self
            .query_smart::<_, Option<H256>>(
                &union_ibc_msg::query::QueryMsg::GetBatchPackets {
                    channel_id,
                    batch_hash,
                },
                Some(height),
            )
            .await?;

        Ok(commitment.flatten())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id, %batch_hash))]
    async fn query_batch_receipts(
        &self,
        height: Height,
        channel_id: u32,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let commitment = self
            .query_smart::<_, Option<H256>>(
                &union_ibc_msg::query::QueryMsg::GetBatchReceipts {
                    channel_id,
                    batch_hash,
                },
                Some(height),
            )
            .await?;

        Ok(commitment.flatten())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: u32) -> RpcResult<ClientInfo> {
        let client_type = self
            .query_smart::<_, String>(
                &union_ibc_msg::query::QueryMsg::GetClientType { client_id },
                None,
            )
            .await?
            .ok_or(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("client `{client_id}` not found"),
                None::<()>,
            ))?;

        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_COSMWASM),
            metadata: Value::Null,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_state(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Value> {
        match path {
            StorePath::ClientState(path) => self
                .query_client_state(at, path.client_id)
                .await
                .map(into_value),
            StorePath::ConsensusState(path) => self
                .query_consensus_state(at, path.client_id, path.height)
                .await
                .map(into_value),
            StorePath::Connection(path) => self
                .query_connection(at, path.connection_id)
                .await
                .map(into_value),
            StorePath::Channel(path) => self
                .query_channel(at, path.channel_id)
                .await
                .map(into_value),
            StorePath::BatchPackets(path) => self
                .query_batch_packets(at, path.channel_id, path.batch_hash)
                .await
                .map(into_value),
            StorePath::BatchReceipts(path) => self
                .query_batch_receipts(at, path.channel_id, path.batch_hash)
                .await
                .map(into_value),
        }
    }
}

// NOTE: For both of the below functions, `message` as a field will override any actual message put in (i.e. `error!("foo", message = "bar")` will print as "bar", not "foo" with an extra field `message = "bar"`.

fn rpc_error<E: Error>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| {
        let message = format!("{message}: {}", ErrorReporter(e));
        error!(%message, data = %data.as_ref().unwrap_or(&serde_json::Value::Null));
        ErrorObject::owned(-1, message, data)
    }
}
