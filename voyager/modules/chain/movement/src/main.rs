use std::{
    collections::VecDeque,
    num::{NonZeroU32, NonZeroU8},
};

use aptos_rpc::AptosRpcClient;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{call, conc, promise, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, info, instrument, warn};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height, commitment::merkle_proof::MerkleProof,
        connection::connection_end::ConnectionEnd,
    },
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath, Path},
    id::ClientId,
    option_unwrap, ErrorReporter, QueryHeight, WasmClientType,
};
use voyager_message::{
    call::{compound::fetch_client_state_meta, Call, FetchClientInfo, FetchState},
    callback::{
        AggregateDecodeClientStateMetaFromConnection, AggregateFetchClientFromConnection,
        AggregateFetchCounterpartyChannelAndConnection, Callback, InfoOrMeta,
    },
    data::{ClientInfo, Data, IbcState},
    plugin::{
        ChainModuleServer, IbcGo08WasmClientMetadata, PluginInfo, PluginKind, PluginModuleServer,
        RawClientState,
    },
    run_module_server, ChainId, ClientType, IbcInterface, VoyagerMessage,
};

use crate::{
    call::{FetchBlocks, FetchClientFromConnectionId, FetchTransactions, ModuleCall},
    callback::{MakeFullEvent, ModuleCallback},
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

const PER_PAGE_LIMIT: NonZeroU8 = option_unwrap!(NonZeroU8::new(10));

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(
        Module::new,
        ChainModuleServer::into_rpc,
        |config, cmd| async move { Module::new(config).await?.cmd(cmd).await },
    )
    .await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub aptos_client: AptosRpcClient,

    pub ibc_handler_address: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_handler_address: H256,
}

impl Module {
    pub async fn cmd(&self, cmd: Cmd) -> Result<(), BoxDynError> {
        match cmd {
            Cmd::ChainId => println!("{}", self.chain_id),
            Cmd::LatestHeight => println!("{}", self.query_latest_height().await?),
        }

        Ok(())
    }

    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, BoxDynError> {
        let aptos_client = AptosRpcClient::new(config.rpc_url);

        let chain_id = aptos_client.ledger_info().await?.chain_id;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            aptos_client,
            ibc_handler_address: config.ibc_handler_address,
        })
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            // TODO: Make this a constant
            revision_number: 0,
            revision_height: height,
        }
    }
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Chain),
            interest_filter: None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn callback(
        &self,
        cb: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(match cb {
            ModuleCallback::MakeFullEvent(aggregate) => aggregate.do_aggregate(data),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        todo!()
    }
}

#[async_trait]
impl ChainModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn chain_id(&self) -> RpcResult<ChainId<'static>> {
        Ok(self.chain_id.clone())
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self) -> RpcResult<Height> {
        match self.aptos_client.ledger_info().await {
            Ok(ledger_info) => {
                let height = ledger_info.block_height;

                debug!(height, "latest height");

                Ok(self.make_height(height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height> {
        self.query_latest_height().await
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self) -> RpcResult<i64> {
        let latest_height = self.query_latest_height().await?;

        match self
            .aptos_client
            .block_by_height(latest_height.revision_height, false)
            .await
        {
            Ok(block) => {
                let timestamp = block.block_timestamp;

                debug!(%timestamp, %latest_height, "latest timestamp");

                Ok(timestamp.try_into().unwrap())
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.plugin_name(),
            FetchBlocks {
                from_height,
                to_height,
            },
        )))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo> {
        match client_id.to_string().rsplit_once('-') {
            Some(("cometbls", _)) => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::COMETBLS),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_MOVE_APTOS),
                metadata: Default::default(),
            }),
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
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let query_result = self
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                Some(
                    i64::try_from(at.revision_height)
                        .unwrap()
                        .try_into()
                        .expect("invalid height"),
                ),
                false,
            )
            .await
            .unwrap()
            .response;

        Ok(match path {
            Path::ClientState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
            Path::ClientConsensusState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
            Path::Connection(_) => serde_json::to_value(
                ConnectionEnd::decode_as::<Proto>(&query_result.value).unwrap(),
            )
            .unwrap(),
            Path::ChannelEnd(_) => {
                serde_json::to_value(Channel::decode_as::<Proto>(&query_result.value).unwrap())
                    .unwrap()
            }
            Path::Commitment(_) => {
                serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
            }
            Path::Acknowledgement(_) => {
                serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
            }
            Path::Receipt(_) => serde_json::to_value(match query_result.value[..] {
                [] => false,
                [1] => true,
                ref invalid => panic!("not a bool??? {invalid:?}"),
            })
            .unwrap(),
            Path::NextSequenceSend(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextSequenceRecv(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextSequenceAck(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextConnectionSequence(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextClientSequence(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let query_result = self
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(
                    (i64::try_from(at.revision_height).unwrap() - 1)
                        .try_into()
                        .expect("invalid height"),
                ),
                true,
            )
            .await
            .unwrap();

        Ok(serde_json::to_value(
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
                proofs: query_result
                    .response
                    .proof_ops
                    .unwrap()
                    .ops
                    .into_iter()
                    .map(|op| {
                        <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(
                            op.data.as_slice(),
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>(),
            })
            .unwrap(),
        )
        .unwrap())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        let height = self.query_latest_height().await?;

        let client_state = serde_json::from_value::<Hex<Vec<u8>>>(
            self.query_ibc_state(
                height,
                ClientStatePath {
                    client_id: client_id.clone(),
                }
                .into(),
            )
            .await?,
        )
        .unwrap();

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(client_id.clone()).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: client_state.0.into(),
        })
    }
}
