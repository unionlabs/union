use std::{cmp, collections::VecDeque, fmt::Debug, sync::Arc};

use aptos_rest_client::{
    aptos_api_types::{Address, MoveType},
    error::RestError,
    Transaction,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use queue_msg::{call, conc, data, noop, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, error, instrument, warn};
use unionlabs::{
    hash::H256,
    ibc::core::{
        channel::order::Order,
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{ClientStatePath, Path},
    id::ClientId,
    ErrorReporter,
};
use voyager_message::{
    call::Call,
    data::{
        ChainEvent, ClientInfo, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, Data, FullIbcEvent, UpdateClient,
    },
    plugin::{ChainModuleServer, PluginInfo, PluginKind, PluginModuleServer, RawClientState},
    reth_ipc::client::IpcClientBuilder,
    rpc::{json_rpc_error_to_rpc_error, VoyagerRpcClient},
    run_module_server, ChainId, ClientType, IbcInterface, VoyagerMessage,
};

use crate::{
    call::{FetchBlock, FetchBlocks, MakeEvent, ModuleCall},
    callback::ModuleCallback,
    client::ibc::ClientExt,
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

impl client::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

impl Module {
    pub async fn cmd(&self, cmd: Cmd) -> Result<(), BoxDynError> {
        match cmd {
            Cmd::ChainId => println!("{}", self.chain_id),
            Cmd::LatestHeight => println!("{}", self.query_latest_height().await?),
            Cmd::VaultAddress => {
                let addr = self
                    .get_vault_addr(self.ibc_handler_address.into(), None)
                    .await?;

                println!("{addr}");
            }
            Cmd::SubmitTx => {
                // let pk = aptos_crypto::ed25519::Ed25519PrivateKey::try_from(
                //     hex_literal::hex!(
                //         "f90391c81027f03cdea491ed8b36ffaced26b6df208a9b569e5baf2590eb9b16"
                //     )
                //     .as_slice(),
                // )
                // .unwrap();

                // let sender = H256::from(
                //     sha3::Sha3_256::new()
                //         .chain_update(pk.public_key().to_bytes())
                //         .chain_update([0])
                //         .finalize(),
                // )
                // .0
                // .into();

                // dbg!(&sender);

                // let account = self
                //     .aptos_client
                //     .get_account(sender)
                //     .await
                //     .unwrap()
                //     .into_inner();

                // dbg!(&account);

                // // let raw = RawTransaction::new_entry_function(
                // //     sender,
                // //     account.sequence_number,
                // //     self.hackerman(
                // //         // client::height::Height {
                // //         //     revision_number: 1.into(),
                // //         //     revision_height: 1.into(),
                // //         // }
                // //         // .with_address(self.ibc_handler_address.into()),
                // //         // ("hi".to_owned(),),
                // //         (69_420_u64,),
                // //     ),
                // //     400000,
                // //     100,
                // //     queue_msg::now() + 10,
                // //     aptos_types::chain_id::ChainId::new(27),
                // // );

                // let sig = raw.sign(&pk, pk.public_key()).unwrap();

                // dbg!(&sig);

                // let res = self.aptos_client.submit_and_wait(&sig).await.unwrap();

                // dbg!(&res);

                // let tx_events = match res.into_inner() {
                //     Transaction::UserTransaction(tx) => tx.events,
                //     e => panic!("{e:?}"),
                // };

                // let (typ, data) = tx_events
                //     .into_iter()
                //     .find_map(|e| match e.typ {
                //         MoveType::Struct(s) => {
                //             (s.address == self.ibc_handler_address).then_some((s, e.data))
                //         }
                //         _ => None,
                //     })
                //     .unwrap();

                // dbg!(&typ, &data);

                // // let event = serde_json::from_value::<crate::events::IbcEvent>(data).unwrap();
                // // let event: unionlabs::events::IbcEvent = match typ.name.as_str() {
                // //     "ClientCreatedEvent" => {
                // //         serde_json::from_value::<unionlabs::events::CreateClient>(data)
                // //             .unwrap()
                // //             .into()
                // //     }
                // //     unknown => panic!("unknown event {unknown}"),
                // // };

                // // println!("{:?}", event);
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
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
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
                    .flat_map(|tx| tx.events.into_iter().map(move |events| (events, tx.info.hash)))
                    .filter_map(|(e, hash)| match e.typ {
                        MoveType::Struct(s) => (dbg!(&s).address == self.ibc_handler_address)
                            .then_some((s, e.data, hash)),
                        _ => None,
                    })
                    .map(|(_typ, data, hash)| {
                        // TODO: Check the type before deserializing
                        call(Call::plugin(
                            self.plugin_name(),
                            MakeEvent {
                                event: serde_json::from_value::<crate::events::IbcEvent>(dbg!(
                                    data
                                ))
                                .unwrap(),
                                tx_hash: H256(*hash.0),
                                height,
                            },
                        ))
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
                    // .map(|e| match e {
                    //     events::IbcEvent::CreateClient(client::ibc::ClientCreatedEvent {
                    //         client_id,
                    //         client_type,
                    //         consensus_height,
                    //     }) => CreateClient {
                    //         client_id,
                    //         client_type,
                    //         consensus_height: consensus_height.into(),
                    //     }
                    //     .into(),
                    //     events::IbcEvent::UpdateClient {
                    //         client_id,
                    //         client_type,
                    //         consensus_heights,
                    //     } => UpdateClient {
                    //         client_id,
                    //         client_type,
                    //         consensus_heights: consensus_heights
                    //             .into_iter()
                    //             .map(Into::into)
                    //             .collect(),
                    //     }
                    //     .into(),
                    // })
                    ;

                // dbg!(events);

                Ok(conc(events))
            }
            ModuleCall::FetchBlocks(FetchBlocks {
                from_height,
                to_height,
            }) => Ok(match (from_height + 1).cmp(&to_height) {
                // still blocks left to fetch between from_height and to_height
                cmp::Ordering::Less => conc([
                    call(Call::plugin(
                        self.plugin_name(),
                        FetchBlock {
                            height: from_height,
                        },
                    )),
                    call(Call::plugin(
                        self.plugin_name(),
                        FetchBlocks {
                            from_height: from_height + 1,
                            to_height,
                        },
                    )),
                ]),
                // from_height + 1 == to_height, range is finished
                cmp::Ordering::Equal => call(Call::plugin(
                    self.plugin_name(),
                    FetchBlock {
                        height: from_height,
                    },
                )),
                // inverted range, this is either a bug or a bad input
                cmp::Ordering::Greater => {
                    error!("attempted to fetch blocks in range {from_height} to {to_height}");
                    // REVIEW: Should this return an error instead?
                    noop()
                }
            }),
            ModuleCall::MakeEvent(MakeEvent {
                event,
                tx_hash,
                height,
            }) => {
                fn ibc_height(h: crate::client::height::Height) -> Height {
                    Height {
                        revision_number: h.revision_number.0,
                        revision_height: h.revision_height.0,
                    }
                }

                let (full_event, client_id): (FullIbcEvent, ClientId) = match event {
                    events::IbcEvent::CreateClient(e) => (
                        CreateClient {
                            client_id: e.client_id.parse().unwrap(),
                            client_type: ClientType::new(e.client_type),
                            consensus_height: ibc_height(e.consensus_height),
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::UpdateClient(e) => (
                        UpdateClient {
                            client_id: e.client_id.parse().unwrap(),
                            client_type: ClientType::new(e.client_type),
                            consensus_heights: vec![ibc_height(e.height)],
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenInit(e) => (
                        ConnectionOpenInit {
                            client_id: e.client_id.parse().unwrap(),
                            connection_id: e.connection_id.parse().unwrap(),
                            counterparty_client_id: e.counterparty_client_id.parse().unwrap(),
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenTry(e) => (
                        ConnectionOpenTry {
                            client_id: e.client_id.parse().unwrap(),
                            connection_id: e.connection_id.parse().unwrap(),
                            counterparty_client_id: e.counterparty_client_id.parse().unwrap(),
                            counterparty_connection_id: e
                                .counterparty_connection_id
                                .parse()
                                .unwrap(),
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenAck(e) => (
                        ConnectionOpenAck {
                            client_id: e.client_id.parse().unwrap(),
                            connection_id: e.connection_id.parse().unwrap(),
                            counterparty_client_id: e.counterparty_client_id.parse().unwrap(),
                            counterparty_connection_id: e
                                .counterparty_connection_id
                                .parse()
                                .unwrap(),
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenConfirm(e) => (
                        ConnectionOpenConfirm {
                            client_id: e.client_id.parse().unwrap(),
                            connection_id: e.connection_id.parse().unwrap(),
                            counterparty_client_id: e.counterparty_client_id.parse().unwrap(),
                            counterparty_connection_id: e
                                .counterparty_connection_id
                                .parse()
                                .unwrap(),
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ChannelOpenInit(_) => todo!(),
                    events::IbcEvent::ChannelOpenTry(_) => todo!(),
                    events::IbcEvent::ChannelOpenAck(_) => todo!(),
                    events::IbcEvent::ChannelOpenConfirm(_) => todo!(),
                    events::IbcEvent::WriteAcknowledgement(_) => todo!(),
                    events::IbcEvent::RecvPacket(_) => todo!(),
                    events::IbcEvent::SendPacket(_) => todo!(),
                    events::IbcEvent::AcknowledgePacket(_) => todo!(),
                    events::IbcEvent::TimeoutPacket(_) => todo!(),
                };

                let client_info = self
                    .client
                    .client_info(self.chain_id.clone(), client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_rpc_error)?;

                let client_meta = self
                    .client
                    .client_meta(
                        self.chain_id.clone(),
                        self.make_height(height).into(),
                        client_id.clone(),
                    )
                    .await
                    .map_err(json_rpc_error_to_rpc_error)?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_meta.chain_id,
                    tx_hash,
                    // TODO: Review this, does it need to be +1?
                    provable_height: self.make_height(height),
                    event: full_event,
                }))
            }
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %at, %path))]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        let ledger_version = self
            .aptos_client
            .get_block_by_height(at.revision_height, false)
            .await
            .map_err(rest_error_to_rpc_error)?
            .into_inner()
            .last_version
            .0;

        debug!("height {at} is ledger version {ledger_version}");

        Ok(match path {
            Path::ClientState(path) => {
                let client_state_bytes = self
                    .client_state(
                        self.ibc_handler_address.into(),
                        (path.client_id.to_string(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(Hex(client_state_bytes))
            }
            Path::ClientConsensusState(path) => {
                let consensus_state_bytes = self
                    .consensus_state(
                        self.ibc_handler_address.into(),
                        (
                            path.client_id.to_string(),
                            path.height.revision_number,
                            path.height.revision_height,
                        ),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(Hex(consensus_state_bytes))
            }
            Path::Connection(path) => into_value(
                match self
                    .get_connection(
                        self.ibc_handler_address.into(),
                        (path.connection_id.to_string(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                {
                    Some(connection) => Some(ConnectionEnd {
                        client_id: connection.client_id.parse().unwrap(),
                        versions: connection
                            .versions
                            .into_iter()
                            .map(|version| connection::version::Version {
                                identifier: version.identifier,
                                features: version
                                    .features
                                    .into_iter()
                                    .map(|feature| {
                                        Order::from_proto_str(&feature).expect("unknown feature")
                                    })
                                    .collect(),
                            })
                            .collect(),
                        state: connection::state::State::try_from(
                            u8::try_from(connection.state.0).unwrap(),
                        )
                        .unwrap(),
                        counterparty: connection::counterparty::Counterparty {
                            client_id: connection.counterparty.client_id.parse().unwrap(),
                            connection_id: if connection.counterparty.connection_id.is_empty() {
                                None
                            } else {
                                Some(connection.counterparty.connection_id.parse().unwrap())
                            },
                            prefix: MerklePrefix {
                                key_prefix: connection.counterparty.prefix.key_prefix,
                            },
                        },
                        delay_period: connection.delay_period.0,
                    }),
                    None => None,
                },
            ),
            Path::ChannelEnd(_) => todo!(),
            Path::Commitment(_) => todo!(),
            Path::Acknowledgement(_) => todo!(),
            Path::Receipt(_) => todo!(),
            Path::NextSequenceSend(_) => todo!(),
            Path::NextSequenceRecv(_) => todo!(),
            Path::NextSequenceAck(_) => todo!(),
            Path::NextConnectionSequence(_) => todo!(),
            Path::NextClientSequence(_) => todo!(),
        })

        // self.get_connection(, , )

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

pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
    ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
}

// TODO: Deduplicate this (it's also in the cosmos-sdk chain module), probably put it in voyager-message
#[track_caller]
fn into_value<T: Debug + Serialize>(t: T) -> Value {
    match serde_json::to_value(t) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                error = %ErrorReporter(err),
                "error serializing value of type {}",
                std::any::type_name::<T>()
            );

            panic!(
                "error serializing value of type {}",
                std::any::type_name::<T>()
            );
        }
    }
}
