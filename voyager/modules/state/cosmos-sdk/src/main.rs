// #![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU64, ParseIntError},
};

use cometbft_rpc::types::abci::response_query::QueryResponse;
use ibc_classic_spec::{
    AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
    ConnectionPath, IbcClassic, NextClientSequencePath, NextConnectionSequencePath,
    NextSequenceAckPath, NextSequenceRecvPath, NextSequenceSendPath, ReceiptPath, StorePath,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, instrument};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    never::Never,
    primitives::{Bytes, H256, H64},
    ErrorReporter,
};
use voyager_message::{
    into_value,
    module::{StateModuleInfo, StateModuleServer},
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface},
    StateModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

const IBC_STORE_PATH: &str = "store/ibc/key";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as StateModule<IbcClassic>>::run().await;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    #[serde(default = "default_max_drift")]
    pub max_drift: u64,
}

fn default_max_drift() -> u64 {
    10
}

impl StateModule<IbcClassic> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        let chain_revision = chain_id
            .split('-')
            .next_back()
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
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }

    async fn abci_query(&self, path_string: &str, height: Height) -> RpcResult<QueryResponse> {
        self.tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                Some(
                    i64::try_from(height.height())
                        .expect("should be fine")
                        .try_into()
                        .expect("invalid height"),
                ),
                false,
            )
            .await
            .map_err(rpc_error(
                format_args!("error fetching abci query"),
                Some(json!({ "height": height, "path": path_string })),
            ))
            .map(|response| response.response)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(&self, height: Height, client_id: ClientId) -> RpcResult<Bytes> {
        let path_string = ClientStatePath { client_id }.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(query_result.value.unwrap().into_encoding())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_client_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Bytes> {
        let path_string = ClientConsensusStatePath {
            client_id,
            height: trusted_height,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(query_result.value.unwrap().into_encoding())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionEnd>> {
        let path_string = ConnectionPath { connection_id }.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(
                ConnectionEnd::decode_as::<Proto>(&value)
                    .map_err(fatal_rpc_error("error decoding connection end", None))?,
            ),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_channel(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
        let path_string = ChannelEndPath {
            channel_id,
            port_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(
                Channel::decode_as::<Proto>(&value)
                    .map_err(fatal_rpc_error("error decoding channel end", None))?,
            ),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_commitment(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let path_string = CommitmentPath {
            port_id,
            channel_id,
            sequence,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(
                H256::try_from(value)
                    .map_err(fatal_rpc_error("error decoding commitment", None))?,
            ),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_acknowledgement(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let path_string = AcknowledgementPath {
            port_id,
            channel_id,
            sequence,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(H256::try_from(value).map_err(fatal_rpc_error(
                "error decoding acknowledgement commitment",
                None,
            ))?),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_receipt(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        let path_string = ReceiptPath {
            port_id,
            channel_id,
            sequence,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            None => false,
            Some(b) if b == [1] => true,
            Some(invalid) => {
                return Err(fatal_rpc_error("error decoding receipt", None)(format!(
                    "value is neither empty nor the single byte 0x01, found {}",
                    serde_utils::to_hex(invalid)
                )))
            }
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_send(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let path_string = NextSequenceSendPath {
            port_id,
            channel_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_sequence_send", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_recv(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let path_string = NextSequenceRecvPath {
            port_id,
            channel_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_sequence_recv", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_ack(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let path_string = NextSequenceAckPath {
            port_id,
            channel_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_sequence_ack", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_connection_sequence(&self, height: Height) -> RpcResult<u64> {
        let path_string = NextConnectionSequencePath {}.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error(
                    "error decoding next_connection_sequence",
                    None,
                ))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_client_sequence(&self, height: Height) -> RpcResult<u64> {
        let path_string = NextClientSequencePath {}.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_client_sequence", None))?
                .get(),
        ))
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
impl StateModuleServer<IbcClassic> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, _: &Extensions, query: Never) -> RpcResult<Value> {
        match query {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        match client_id.to_string().rsplit_once('-') {
            Some(("07-tendermint", _)) => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::TENDERMINT),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_NATIVE),
                metadata: Default::default(),
            }),
            Some(("08-wasm", _)) => {
                // let checksum = self.checksum_of_client_id(client_id.clone()).await?;

                // Ok(ClientInfo {
                //     client_type: match self.client_type_of_checksum(checksum).await? {
                //         Some(ty) => match ty {
                //             WasmClientType::Cometbls => {
                //                 ClientType::new(ClientType::COMETBLS_GROTH16)
                //             }
                //             WasmClientType::Tendermint => ClientType::new(ClientType::TENDERMINT),
                //         },
                //         None => {
                //             warn!(%client_id, "unknown client type for 08-wasm client");
                //             // this early return is kind of dirty but it works
                //             return Err(ErrorObject::owned(
                //                 FATAL_JSONRPC_ERROR_CODE,
                //                 "unknown client type for 08-wasm client",
                //                 Some(json!({
                //                     "client_id": client_id.to_string()
                //                 })),
                //             ));
                //         }
                //     },
                //     ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_08_WASM),
                //     metadata: into_value(IbcGo08WasmClientMetadata { checksum }),
                // })
                todo!()
            }
            _ => Err(ErrorObject::owned(
                -1,
                format!("unknown client type (client id `{client_id}`)"),
                Some(json!({
                    "client_id": client_id.to_string()
                })),
            )),
        }
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
            StorePath::ClientConsensusState(path) => self
                .query_client_consensus_state(at, path.client_id, path.height)
                .await
                .map(into_value),
            StorePath::Connection(path) => self
                .query_connection(at, path.connection_id)
                .await
                .map(into_value),
            StorePath::ChannelEnd(path) => self
                .query_channel(at, path.port_id, path.channel_id)
                .await
                .map(into_value),
            StorePath::Commitment(path) => self
                .query_commitment(at, path.port_id, path.channel_id, path.sequence)
                .await
                .map(into_value),
            StorePath::Acknowledgement(path) => self
                .query_acknowledgement(at, path.port_id, path.channel_id, path.sequence)
                .await
                .map(into_value),
            StorePath::Receipt(path) => self
                .query_receipt(at, path.port_id, path.channel_id, path.sequence)
                .await
                .map(into_value),
            StorePath::NextSequenceSend(path) => self
                .query_next_sequence_send(at, path.port_id, path.channel_id)
                .await
                .map(into_value),
            StorePath::NextSequenceRecv(path) => self
                .query_next_sequence_recv(at, path.port_id, path.channel_id)
                .await
                .map(into_value),
            StorePath::NextSequenceAck(path) => self
                .query_next_sequence_ack(at, path.port_id, path.channel_id)
                .await
                .map(into_value),
            StorePath::NextConnectionSequence(_path) => self
                .query_next_connection_sequence(at)
                .await
                .map(into_value),
            StorePath::NextClientSequence(_path) => {
                self.query_next_client_sequence(at).await.map(into_value)
            }
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

fn fatal_rpc_error<E: Into<Box<dyn Error>>>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| {
        let e = e.into();
        let message = format!("{message}: {}", ErrorReporter(&*e));
        error!(%message, data = %data.as_ref().unwrap_or(&serde_json::Value::Null));
        ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, message, data)
    }
}
