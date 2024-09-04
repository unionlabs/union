use std::{collections::VecDeque, sync::Arc};

use aptos_crypto::PrivateKey;
use aptos_rest_client::{
    aptos_api_types::{Address, MoveType},
    Transaction,
};
use aptos_types::transaction::RawTransaction;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{call, noop, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use sha3::Digest;
use tracing::{debug, instrument, warn};
use unionlabs::{
    events::{CreateClient, IbcEvent, UpdateClient},
    hash::H256,
    ibc::core::client::height::Height,
    ics24::{ClientStatePath, Path},
    id::ClientId,
    ErrorReporter,
};
use voyager_message::{
    call::Call,
    data::{ClientInfo, Data},
    plugin::{ChainModuleServer, PluginInfo, PluginKind, PluginModuleServer, RawClientState},
    reth_ipc::client::IpcClientBuilder,
    run_module_server, ChainId, ClientType, IbcInterface, VoyagerMessage,
};

use crate::{
    call::{FetchBlock, FetchBlocks, ModuleCall},
    callback::ModuleCallback,
    client::Core::ClientExt,
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

pub mod client;
pub mod events;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server(
        Module::new,
        ChainModuleServer::into_rpc,
        |config, cmd| async move { Module::new(config, String::new()).await?.cmd(cmd).await },
    )
    .await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
    VaultAddress,
    SubmitTx,
    FetchAbi,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub client: Arc<jsonrpsee::ws_client::WsClient>,

    pub chain_id: ChainId<'static>,

    pub aptos_client: aptos_rest_client::Client,

    pub ibc_handler_address: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_handler_address: Address,
}

impl client::Core::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }

    fn module_address(&self) -> aptos_types::account_address::AccountAddress {
        self.ibc_handler_address.into()
    }
}

impl Module {
    pub async fn cmd(&self, cmd: Cmd) -> Result<(), BoxDynError> {
        match cmd {
            Cmd::ChainId => println!("{}", self.chain_id),
            Cmd::LatestHeight => println!("{}", self.query_latest_height().await?),
            Cmd::VaultAddress => {
                let addr = self.get_vault_addr(None).await?;

                println!("{addr}");
            }
            Cmd::SubmitTx => {
                let pk = aptos_crypto::ed25519::Ed25519PrivateKey::try_from(
                    hex_literal::hex!(
                        "f90391c81027f03cdea491ed8b36ffaced26b6df208a9b569e5baf2590eb9b16"
                    )
                    .as_slice(),
                )
                .unwrap();

                let sender = H256::from(
                    sha3::Sha3_256::new()
                        .chain_update(pk.public_key().to_bytes())
                        .chain_update([0])
                        .finalize(),
                )
                .0
                .into();

                dbg!(&sender);

                let account = self
                    .aptos_client
                    .get_account(sender)
                    .await
                    .unwrap()
                    .into_inner();

                dbg!(&account);

                let raw = RawTransaction::new_entry_function(
                    sender,
                    account.sequence_number,
                    self.hackerman(
                        // client::height::Height {
                        //     revision_number: 1.into(),
                        //     revision_height: 1.into(),
                        // }
                        // .with_address(self.ibc_handler_address.into()),
                        // ("hi".to_owned(),),
                        (69_420_u64,),
                    ),
                    400000,
                    100,
                    queue_msg::now() + 10,
                    aptos_types::chain_id::ChainId::new(27),
                );

                let sig = raw.sign(&pk, pk.public_key()).unwrap();

                dbg!(&sig);

                let res = self.aptos_client.submit_and_wait(&sig).await.unwrap();

                dbg!(&res);

                let tx_events = match res.into_inner() {
                    Transaction::UserTransaction(tx) => tx.events,
                    e => panic!("{e:?}"),
                };

                let (typ, data) = tx_events
                    .into_iter()
                    .find_map(|e| match e.typ {
                        MoveType::Struct(s) => {
                            (s.address == self.ibc_handler_address).then_some((s, e.data))
                        }
                        _ => None,
                    })
                    .unwrap();

                dbg!(&typ, &data);

                // let event = serde_json::from_value::<crate::events::IbcEvent>(data).unwrap();
                // let event: unionlabs::events::IbcEvent = match typ.name.as_str() {
                //     "ClientCreatedEvent" => {
                //         serde_json::from_value::<unionlabs::events::CreateClient>(data)
                //             .unwrap()
                //             .into()
                //     }
                //     unknown => panic!("unknown event {unknown}"),
                // };

                // println!("{:?}", event);
            }
            Cmd::FetchAbi => {
                let abis = self
                    .aptos_client
                    .get_account_modules(self.ibc_handler_address.into())
                    .await?
                    .into_inner()
                    .into_iter()
                    .flat_map(|x| x.try_parse_abi().unwrap().abi)
                    .collect::<Vec<_>>();

                dbg!(abis);
            }
        }

        Ok(())
    }

    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config, voyager_socket: String) -> Result<Self, BoxDynError> {
        let client = Arc::new(IpcClientBuilder::default().build(&voyager_socket).await?);

        let aptos_client = aptos_rest_client::Client::new(config.rpc_url.parse()?);

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        Ok(Self {
            client,
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
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::FetchBlock(FetchBlock { height }) => {
                let events = self
                    .aptos_client
                    .get_block_by_height(height, true)
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            format!("error fetching height: {}", ErrorReporter(e)),
                            None::<()>,
                        )
                    })?
                    .into_inner()
                    .transactions
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|tx| match tx {
                        Transaction::UserTransaction(tx) => Some(tx),
                        _ => None,
                    })
                    .flat_map(|tx| tx.events)
                    .filter_map(|e| match e.typ {
                        MoveType::Struct(s) => {
                            (s.address == self.ibc_handler_address).then_some((s, e.data))
                        }
                        _ => None,
                    })
                    .map(|(_typ, data)| {
                        // TODO: Check the type before deserializing
                        serde_json::from_value::<crate::events::IbcEvent>(data).unwrap()
                    })
                    // .map(|e| ChainEvent {
                    //     chain_id: todo!(),
                    //     client_info: todo!(),
                    //     counterparty_chain_id: todo!(),
                    //     tx_hash: todo!(),
                    //     provable_height: todo!(),
                    //     event: match e {
                    //         events::IbcEvent::CreateClient {
                    //             client_id,
                    //             client_type,
                    //             consensus_height,
                    //         } => CreateClient {
                    //             client_id,
                    //             client_type,
                    //             consensus_height: consensus_height.into(),
                    //         }
                    //         .into(),
                    //         events::IbcEvent::UpdateClient {
                    //             client_id,
                    //             client_type,
                    //             consensus_heights,
                    //         } => UpdateClient {
                    //             client_id,
                    //             client_type,
                    //             consensus_heights: consensus_heights
                    //                 .into_iter()
                    //                 .map(Into::into)
                    //                 .collect(),
                    //         }
                    //         .into(),
                    //     },
                    // });
                    .map(|e| match e {
                        events::IbcEvent::CreateClient {
                            client_id,
                            client_type,
                            consensus_height,
                        } => CreateClient {
                            client_id,
                            client_type,
                            consensus_height: consensus_height.into(),
                        }
                        .into(),
                        events::IbcEvent::UpdateClient {
                            client_id,
                            client_type,
                            consensus_heights,
                        } => UpdateClient {
                            client_id,
                            client_type,
                            consensus_heights: consensus_heights
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                        }
                        .into(),
                    })
                    .collect::<Vec<IbcEvent>>();

                dbg!(events);

                Ok(noop())
            }
            ModuleCall::FetchBlocks(_) => todo!(),
        }
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
        match self.aptos_client.get_index().await {
            Ok(ledger_info) => {
                let height = ledger_info.inner().block_height.0;

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
            .get_block_by_height(latest_height.revision_height, false)
            .await
        {
            Ok(block) => {
                let timestamp = block.inner().block_timestamp.0;

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
                from_height: from_height.revision_height,
                to_height: to_height.revision_height,
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
        let _ = (at, path);

        // const IBC_STORE_PATH: &str = "store/ibc/key";

        // let path_string = path.to_string();

        // let query_result = self
        //     .tm_client
        //     .abci_query(
        //         IBC_STORE_PATH,
        //         &path_string,
        //         Some(
        //             i64::try_from(at.revision_height)
        //                 .unwrap()
        //                 .try_into()
        //                 .expect("invalid height"),
        //         ),
        //         false,
        //     )
        //     .await
        //     .unwrap()
        //     .response;

        // Ok(match path {
        //     Path::ClientState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
        //     Path::ClientConsensusState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
        //     Path::Connection(_) => serde_json::to_value(
        //         ConnectionEnd::decode_as::<Proto>(&query_result.value).unwrap(),
        //     )
        //     .unwrap(),
        //     Path::ChannelEnd(_) => {
        //         serde_json::to_value(Channel::decode_as::<Proto>(&query_result.value).unwrap())
        //             .unwrap()
        //     }
        //     Path::Commitment(_) => {
        //         serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
        //     }
        //     Path::Acknowledgement(_) => {
        //         serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
        //     }
        //     Path::Receipt(_) => serde_json::to_value(match query_result.value[..] {
        //         [] => false,
        //         [1] => true,
        //         ref invalid => panic!("not a bool??? {invalid:?}"),
        //     })
        //     .unwrap(),
        //     Path::NextSequenceSend(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextSequenceRecv(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextSequenceAck(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextConnectionSequence(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextClientSequence(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        // })

        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        let _ = (at, path);

        // const IBC_STORE_PATH: &str = "store/ibc/key";

        // let path_string = path.to_string();

        // let query_result = self
        //     .tm_client
        //     .abci_query(
        //         IBC_STORE_PATH,
        //         &path_string,
        //         // a proof at height H is provable at height H + 1
        //         // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
        //         Some(
        //             (i64::try_from(at.revision_height).unwrap() - 1)
        //                 .try_into()
        //                 .expect("invalid height"),
        //         ),
        //         true,
        //     )
        //     .await
        //     .unwrap();

        // Ok(serde_json::to_value(
        //     MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
        //         proofs: query_result
        //             .response
        //             .proof_ops
        //             .unwrap()
        //             .ops
        //             .into_iter()
        //             .map(|op| {
        //                 <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(
        //                     op.data.as_slice(),
        //                 )
        //                 .unwrap()
        //             })
        //             .collect::<Vec<_>>(),
        //     })
        //     .unwrap(),
        // )
        // .unwrap())

        todo!()
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
