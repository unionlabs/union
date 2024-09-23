use std::{cmp, collections::VecDeque, fmt::Debug};

use aptos_move_ibc::ibc::{self, ClientExt as _};
use aptos_rest_client::{
    aptos_api_types::{Address, MoveType},
    error::RestError,
    Transaction,
};
use aptos_types::state_store::state_value::PersistedStateValueMetadata;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use queue_msg::{call, conc, data, noop, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use sha2::{Digest as _, Sha256};
use tracing::{debug, error, instrument, warn};
use unionlabs::{
    aptos::{
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        storage_proof::{StateValue, StateValueMetadata, StorageProof},
    },
    hash::H256,
    ibc::core::{
        channel::{self, channel::Channel, order::Order},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath, Path},
    id::{ChannelId, ClientId, PortId},
    uint::U256,
    ErrorReporter, QueryHeight,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    data::{
        AcknowledgePacket, ChainEvent, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm,
        ChannelOpenInit, ChannelOpenTry, ConnectionMetadata, ConnectionOpenAck,
        ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, CreateClient, Data,
        FullIbcEvent, PacketMetadata, RecvPacket, SendPacket, UpdateClient, WriteAcknowledgement,
    },
    module::{
        ChainModuleInfo, ChainModuleServer, ModuleInfo, QueueInteractionsServer, RawClientState,
    },
    reconnecting_jsonrpc_ws_client,
    rpc::{json_rpc_error_to_rpc_error, missing_state, VoyagerRpcClient, VoyagerRpcClientExt as _},
    run_module_server, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
};

use crate::{
    call::{FetchBlock, FetchBlocks, MakeEvent, ModuleCall},
    callback::ModuleCallback,
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

pub mod events;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module, _, _, _>().await
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
    pub chain_id: ChainId<'static>,

    pub aptos_client: aptos_rest_client::Client,
    pub movement_rpc_url: String,

    pub ibc_handler_address: Address,
}

impl ModuleContext for Module {
    type Config = Config;
    // TODO: Use Cmd here
    type Cmd = DefaultCmd;
    type Info = ChainModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let aptos_client = aptos_rest_client::Client::new(config.rpc_url.parse()?);

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            aptos_client,
            movement_rpc_url: config.movement_rpc_url,
            ibc_handler_address: config.ibc_handler_address,
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: ChainModuleInfo {
                chain_id: config.chain_id,
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub rpc_url: String,
    pub movement_rpc_url: String,
    pub ibc_handler_address: Address,
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    pub async fn cmd(&self, cmd: Cmd) -> Result<(), BoxDynError> {
        // match cmd {
        //     Cmd::ChainId => println!("{}", self.chain_id),
        //     // Cmd::LatestHeight => println!("{}", self.query_latest_height().await?),
        //     Cmd::VaultAddress => {
        //         let addr = self
        //             .get_vault_addr(self.ibc_handler_address.into(), None)
        //             .await?;

        //         println!("{addr}");
        //     }
        //     Cmd::SubmitTx => {
        //         // let pk = aptos_crypto::ed25519::Ed25519PrivateKey::try_from(
        //         //     hex_literal::hex!(
        //         //         "f90391c81027f03cdea491ed8b36ffaced26b6df208a9b569e5baf2590eb9b16"
        //         //     )
        //         //     .as_slice(),
        //         // )
        //         // .unwrap();

        //         // let sender = H256::from(
        //         //     sha3::Sha3_256::new()
        //         //         .chain_update(pk.public_key().to_bytes())
        //         //         .chain_update([0])
        //         //         .finalize(),
        //         // )
        //         // .0
        //         // .into();

        //         // dbg!(&sender);

        //         // let account = self
        //         //     .aptos_client
        //         //     .get_account(sender)
        //         //     .await
        //         //     .unwrap()
        //         //     .into_inner();

        //         // dbg!(&account);

        //         // // let raw = RawTransaction::new_entry_function(
        //         // //     sender,
        //         // //     account.sequence_number,
        //         // //     self.hackerman(
        //         // //         // client::height::Height {
        //         // //         //     revision_number: 1.into(),
        //         // //         //     revision_height: 1.into(),
        //         // //         // }
        //         // //         // .with_address(self.ibc_handler_address.into()),
        //         // //         // ("hi".to_owned(),),
        //         // //         (69_420_u64,),
        //         // //     ),
        //         // //     400000,
        //         // //     100,
        //         // //     queue_msg::now() + 10,
        //         // //     aptos_types::chain_id::ChainId::new(27),
        //         // // );

        //         // let sig = raw.sign(&pk, pk.public_key()).unwrap();

        //         // dbg!(&sig);

        //         // let res = self.aptos_client.submit_and_wait(&sig).await.unwrap();

        //         // dbg!(&res);

        //         // let tx_events = match res.into_inner() {
        //         //     Transaction::UserTransaction(tx) => tx.events,
        //         //     e => panic!("{e:?}"),
        //         // };

        //         // let (typ, data) = tx_events
        //         //     .into_iter()
        //         //     .find_map(|e| match e.typ {
        //         //         MoveType::Struct(s) => {
        //         //             (s.address == self.ibc_handler_address).then_some((s, e.data))
        //         //         }
        //         //         _ => None,
        //         //     })
        //         //     .unwrap();

        //         // dbg!(&typ, &data);

        //         // // let event = serde_json::from_value::<crate::events::IbcEvent>(data).unwrap();
        //         // // let event: unionlabs::events::IbcEvent = match typ.name.as_str() {
        //         // //     "ClientCreatedEvent" => {
        //         // //         serde_json::from_value::<unionlabs::events::CreateClient>(data)
        //         // //             .unwrap()
        //         // //             .into()
        //         // //     }
        //         // //     unknown => panic!("unknown event {unknown}"),
        //         // // };

        //         // // println!("{:?}", event);
        //     }
        //     Cmd::FetchAbi => {
        //         let abis = self
        //             .aptos_client
        //             .get_account_modules(self.ibc_handler_address.into())
        //             .await?
        //             .into_inner()
        //             .into_iter()
        //             .flat_map(|x| x.try_parse_abi().unwrap().abi)
        //             .collect::<Vec<_>>();

        //         dbg!(abis);
        //     }
        // }

        Ok(())
    }

    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            // TODO: Make this a constant
            revision_number: 0,
            revision_height: height,
        }
    }

    pub async fn ledger_version_of_height(&self, height: u64) -> u64 {
        let ledger_version = self
            .aptos_client
            .get_block_by_height(height, false)
            .await
            // .map_err(rest_error_to_rpc_error)?
            .unwrap()
            .into_inner()
            .last_version
            .0;

        debug!("height {height} is ledger version {ledger_version}");

        ledger_version
    }

    async fn make_packet_metadata(
        &self,
        event_height: Height,
        self_port_id: PortId,
        self_channel_id: ChannelId,
        voyager_rpc_client: &reconnecting_jsonrpc_ws_client::Client,
    ) -> RpcResult<(
        ChainId<'static>,
        ClientInfo,
        ChannelMetadata,
        ChannelMetadata,
        channel::order::Order,
    )> {
        let self_channel = voyager_rpc_client
            .query_ibc_state_typed(
                self.chain_id.clone(),
                event_height.into(),
                ChannelEndPath {
                    port_id: self_port_id.clone(),
                    channel_id: self_channel_id.clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let self_connection = voyager_rpc_client
            .query_ibc_state_typed(
                self.chain_id.clone(),
                event_height.into(),
                ConnectionPath {
                    connection_id: self_channel.connection_hops[0].clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let self_connection_state = self_connection
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let client_info = voyager_rpc_client
            .client_info(
                self.chain_id.clone(),
                self_connection_state.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let client_meta = voyager_rpc_client
            .client_meta(
                self.chain_id.clone(),
                event_height.into(),
                self_connection_state.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let other_channel = voyager_rpc_client
            .query_ibc_state_typed(
                client_meta.chain_id.clone(),
                QueryHeight::Latest,
                ChannelEndPath {
                    port_id: self_channel.counterparty.port_id.clone(),
                    channel_id: self_channel.counterparty.channel_id.parse().unwrap(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let other_channel_state = other_channel
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

        let source_channel = ChannelMetadata {
            port_id: self_port_id.clone(),
            channel_id: self_channel_id.clone(),
            version: self_channel.version,
            connection: ConnectionMetadata {
                client_id: self_connection_state.client_id,
                connection_id: self_connection.path.connection_id.clone(),
            },
        };
        let destination_channel = ChannelMetadata {
            port_id: other_channel.path.port_id.clone(),
            channel_id: other_channel.path.channel_id.clone(),
            version: other_channel_state.version,
            connection: ConnectionMetadata {
                client_id: self_connection_state.counterparty.client_id,
                connection_id: self_connection_state.counterparty.connection_id.unwrap(),
            },
        };

        Ok((
            client_meta.chain_id,
            client_info,
            source_channel,
            destination_channel,
            self_channel.ordering,
        ))
    }
}

#[async_trait]
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::FetchBlock(FetchBlock { height }) => {
                let events = self
                    .ctx.aptos_client
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
                        MoveType::Struct(s) => (dbg!(&s).address == self.ctx.ibc_handler_address)
                            .then_some((s, e.data, hash)),
                        _ => None,
                    })
                    .map(|(typ, data, hash)| {
                        let event = match typ.name.0.as_str() {
    "ClientCreatedEvent" => serde_json::from_value::<ibc::ClientCreatedEvent>(data).unwrap().into(),
    "ClientUpdated" => serde_json::from_value::<ibc::ClientUpdated>(data).unwrap().into(),
    "ConnectionOpenInit" => serde_json::from_value::<ibc::ConnectionOpenInit>(data).unwrap().into(),
    "ConnectionOpenTry" => serde_json::from_value::<ibc::ConnectionOpenTry>(data).unwrap().into(),
    "ConnectionOpenAck" => serde_json::from_value::<ibc::ConnectionOpenAck>(data).unwrap().into(),
    "ConnectionOpenConfirm" => serde_json::from_value::<ibc::ConnectionOpenConfirm>(data).unwrap().into(),
    "ChannelOpenInit" => serde_json::from_value::<ibc::ChannelOpenInit>(data).unwrap().into(),
    "ChannelOpenTry" => serde_json::from_value::<ibc::ChannelOpenTry>(data).unwrap().into(),
    "ChannelOpenAck" => serde_json::from_value::<ibc::ChannelOpenAck>(data).unwrap().into(),
    "ChannelOpenConfirm" => serde_json::from_value::<ibc::ChannelOpenConfirm>(data).unwrap().into(),
    "WriteAcknowledgement" => serde_json::from_value::<ibc::WriteAcknowledgement>(data).unwrap().into(),
    "RecvPacket" => serde_json::from_value::<ibc::RecvPacket>(data).unwrap().into(),
    "SendPacket" => serde_json::from_value::<ibc::SendPacket>(data).unwrap().into(),
    "AcknowledgePacket" => serde_json::from_value::<ibc::AcknowledgePacket>(data).unwrap().into(),
    "TimeoutPacket" => serde_json::from_value::<ibc::TimeoutPacket>(data).unwrap().into(),
    unknown => panic!("unknown event `{unknown}`")
                        };
                        // TODO: Check the type before deserializing
                        call(Call::plugin(
                            self.ctx.plugin_name(),
                            MakeEvent {
                                event,
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
                        self.ctx.plugin_name(),
                        FetchBlock {
                            height: from_height,
                        },
                    )),
                    call(Call::plugin(
                        self.ctx.plugin_name(),
                        FetchBlocks {
                            from_height: from_height + 1,
                            to_height,
                        },
                    )),
                ]),
                // from_height + 1 == to_height, range is finished
                cmp::Ordering::Equal => call(Call::plugin(
                    self.ctx.plugin_name(),
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
                fn ibc_height(h: aptos_move_ibc::height::Height) -> Height {
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
                    events::IbcEvent::ChannelOpenInit(e) => {
                        let ledger_version = self.ctx.ledger_version_of_height(height).await;

                        let connection = self
                            .ctx
                            .get_connection(
                                self.ctx.ibc_handler_address.into(),
                                (e.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenInit {
                                port_id: e.port_id.parse().unwrap(),
                                channel_id: e.channel_id.parse().unwrap(),
                                counterparty_port_id: e.counterparty_port_id.parse().unwrap(),
                                connection,
                                version: e.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::ChannelOpenTry(e) => {
                        let ledger_version = self.ctx.ledger_version_of_height(height).await;

                        let connection = self
                            .ctx
                            .get_connection(
                                self.ctx.ibc_handler_address.into(),
                                (e.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenTry {
                                port_id: e.port_id.parse().unwrap(),
                                channel_id: e.channel_id.parse().unwrap(),
                                counterparty_port_id: e.counterparty_port_id.parse().unwrap(),
                                counterparty_channel_id: e.counterparty_port_id.parse().unwrap(),
                                connection,
                                version: e.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::ChannelOpenAck(e) => {
                        let ledger_version = self.ctx.ledger_version_of_height(height).await;

                        let connection = self
                            .ctx
                            .get_connection(
                                self.ctx.ibc_handler_address.into(),
                                (e.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let channel = self
                            .ctx
                            .get_channel(
                                self.ctx.ibc_handler_address.into(),
                                (e.port_id.clone(), e.channel_id.clone()),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let channel = convert_channel(channel);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenAck {
                                port_id: e.port_id.parse().unwrap(),
                                channel_id: e.channel_id.parse().unwrap(),
                                counterparty_port_id: e.counterparty_port_id.parse().unwrap(),
                                counterparty_channel_id: e.counterparty_channel_id.parse().unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::ChannelOpenConfirm(e) => {
                        let ledger_version = self.ctx.ledger_version_of_height(height).await;

                        let connection = self
                            .ctx
                            .get_connection(
                                self.ctx.ibc_handler_address.into(),
                                (e.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let channel = self
                            .ctx
                            .get_channel(
                                self.ctx.ibc_handler_address.into(),
                                (e.port_id.clone(), e.channel_id.clone()),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let channel = convert_channel(channel);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenConfirm {
                                port_id: e.port_id.parse().unwrap(),
                                channel_id: e.channel_id.parse().unwrap(),
                                counterparty_port_id: e.counterparty_port_id.parse().unwrap(),
                                counterparty_channel_id: e.counterparty_channel_id.parse().unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::WriteAcknowledgement(e) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                self.ctx.make_height(height),
                                e.packet.destination_port.parse().unwrap(),
                                e.packet.destination_channel.parse().unwrap(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        let client_id = destination_channel.connection.client_id.clone();

                        (
                            WriteAcknowledgement {
                                packet_data: e.packet.data.into(),
                                packet_ack: e.acknowledgement.into(),
                                packet: PacketMetadata {
                                    sequence: (*e.packet.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(e.packet.timeout_height),
                                    timeout_timestamp: *e.packet.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::RecvPacket(e) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                self.ctx.make_height(height),
                                e.packet.destination_port.parse().unwrap(),
                                e.packet.destination_channel.parse().unwrap(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        let client_id = destination_channel.connection.client_id.clone();

                        (
                            RecvPacket {
                                packet_data: e.packet.data.into(),
                                packet: PacketMetadata {
                                    sequence: (*e.packet.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(e.packet.timeout_height),
                                    timeout_timestamp: *e.packet.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::SendPacket(e) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                self.ctx.make_height(height),
                                e.source_port.parse().unwrap(),
                                e.source_channel.parse().unwrap(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        let client_id = source_channel.connection.client_id.clone();

                        (
                            SendPacket {
                                packet_data: e.data.into(),
                                packet: PacketMetadata {
                                    sequence: (*e.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(e.timeout_height),
                                    timeout_timestamp: *e.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::AcknowledgePacket(e) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                self.ctx.make_height(height),
                                e.packet.source_port.parse().unwrap(),
                                e.packet.source_channel.parse().unwrap(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        let client_id = source_channel.connection.client_id.clone();

                        (
                            AcknowledgePacket {
                                packet: PacketMetadata {
                                    sequence: (*e.packet.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(e.packet.timeout_height),
                                    timeout_timestamp: *e.packet.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::TimeoutPacket(_) => todo!(),
                };

                let client_info = self
                    .voyager_rpc_client
                    .client_info(self.ctx.chain_id.clone(), client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_rpc_error)?;

                let client_meta = self
                    .voyager_rpc_client
                    .client_meta(
                        self.ctx.chain_id.clone(),
                        self.ctx.make_height(height).into(),
                        client_id.clone(),
                    )
                    .await
                    .map_err(json_rpc_error_to_rpc_error)?;

                Ok(data(ChainEvent {
                    chain_id: self.ctx.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_meta.chain_id,
                    tx_hash,
                    // TODO: Review this, does it need to be +1?
                    provable_height: self.ctx.make_height(height),
                    event: full_event,
                }))
            }
        }
    }
}

#[async_trait]
impl ChainModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    fn chain_id(&self) -> RpcResult<ChainId<'static>> {
        Ok(self.ctx.chain_id.clone())
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_height(&self) -> RpcResult<Height> {
        match self.ctx.aptos_client.get_index().await {
            Ok(ledger_info) => {
                let height = ledger_info.inner().block_height.0;

                debug!(height, "latest height");

                Ok(self.ctx.make_height(height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height> {
        self.query_latest_height().await
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_timestamp(&self) -> RpcResult<i64> {
        let latest_height = self.query_latest_height().await?;

        match self
            .ctx
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

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.ctx.plugin_name(),
            FetchBlocks {
                from_height: from_height.revision_height,
                to_height: to_height.revision_height,
            },
        )))
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
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

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id, %at, %path))]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        let ledger_version = self.ctx.ledger_version_of_height(at.revision_height).await;

        Ok(match path {
            Path::ClientState(path) => {
                let client_state_bytes = self
                    .ctx
                    .client_state(
                        self.ctx.ibc_handler_address.into(),
                        (path.client_id.to_string(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(client_state_bytes)
            }
            Path::ClientConsensusState(path) => {
                let consensus_state_bytes = self
                    .ctx
                    .consensus_state(
                        self.ctx.ibc_handler_address.into(),
                        (
                            path.client_id.to_string(),
                            path.height.revision_number,
                            path.height.revision_height,
                        ),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(consensus_state_bytes)
            }
            Path::Connection(path) => into_value(
                self.ctx
                    .get_connection(
                        self.ctx.ibc_handler_address.into(),
                        (path.connection_id.to_string(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                    .into_option()
                    .map(convert_connection),
            ),
            Path::ChannelEnd(path) => into_value(
                self.ctx
                    .get_channel(
                        self.ctx.ibc_handler_address.into(),
                        (path.port_id.to_string(), path.channel_id.to_string()),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                    .into_option()
                    .map(convert_channel),
            ),
            Path::Commitment(path) => {
                let commitment = self
                    .ctx
                    .get_commitment(
                        self.ctx.ibc_handler_address.into(),
                        (Sha256::new()
                            .chain_update(path.to_string().into_bytes())
                            .finalize()
                            .to_vec()
                            .into(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(H256::try_from(Into::<Vec<_>>::into(commitment)).unwrap())
            }
            Path::Acknowledgement(_) => todo!(),
            Path::Receipt(path) => {
                let commitment = self
                    .ctx
                    .get_commitment(
                        self.ctx.ibc_handler_address.into(),
                        (Sha256::new()
                            .chain_update(path.to_string().into_bytes())
                            .finalize()
                            .to_vec()
                            .into(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(match &commitment.0[..] {
                    [] => false,
                    [1] => true,
                    _ => panic!("not a bool??? {commitment}"),
                })
            }
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

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        let client = reqwest::Client::new();

        let ledger_version = self.ctx.ledger_version_of_height(at.revision_height).await;

        let vault_addr = self
            .ctx
            .get_vault_addr(self.ctx.ibc_handler_address.into(), Some(ledger_version))
            .await
            .unwrap();

        let address_str = self
            .ctx
            .aptos_client
            .get_account_resource(
                vault_addr.into(),
                &format!("{}::ibc::IBCStore", self.ctx.ibc_handler_address),
            )
            .await
            .unwrap()
            .into_inner()
            .unwrap()
            .data["commitments"]["handle"]
            .clone()
            .as_str()
            .unwrap()
            .to_owned();

        let address = H256(U256::from_be_hex(address_str).unwrap().to_be_bytes());

        let (state_value, proof): (
            Option<aptos_types::state_store::state_value::StateValue>,
            aptos_types::proof::SparseMerkleProof,
        ) = client
            .get(format!(
                "{base_url}/movement/v1/resource-proof/{key}/{address}/{height}",
                base_url = self.ctx.movement_rpc_url,
                key = hex::encode(path.to_string()),
                address = address,
                height = at.revision_height,
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        Ok(into_value(StorageProof {
            state_value: state_value.map(|s| {
                let (metadata, data) = s.unpack();
                match metadata.into_persistable() {
                    None => StateValue::V0(data.to_vec()),
                    Some(PersistedStateValueMetadata::V0 {
                        deposit,
                        creation_time_usecs,
                    }) => StateValue::WithMetadata {
                        data: data.to_vec(),
                        metadata: StateValueMetadata::V0 {
                            deposit,
                            creation_time_usecs,
                        },
                    },
                    Some(PersistedStateValueMetadata::V1 {
                        slot_deposit,
                        bytes_deposit,
                        creation_time_usecs,
                    }) => StateValue::WithMetadata {
                        data: data.to_vec(),
                        metadata: StateValueMetadata::V1 {
                            slot_deposit,
                            bytes_deposit,
                            creation_time_usecs,
                        },
                    },
                }
            }),
            proof: SparseMerkleProof {
                leaf: proof.leaf().map(|leaf| SparseMerkleLeafNode {
                    key: (*leaf.key().as_ref()).into(),
                    value_hash: (*leaf.value_hash().as_ref()).into(),
                }),
                siblings: proof
                    .siblings()
                    .iter()
                    .map(AsRef::as_ref)
                    .copied()
                    .map(Into::into)
                    .collect(),
            },
        }))
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
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

pub fn convert_connection(
    connection: aptos_move_ibc::connection_end::ConnectionEnd,
) -> ConnectionEnd {
    ConnectionEnd {
        client_id: connection.client_id.parse().unwrap(),
        versions: connection
            .versions
            .into_iter()
            .map(|version| connection::version::Version {
                identifier: version.identifier,
                features: version
                    .features
                    .into_iter()
                    .map(|feature| Order::from_proto_str(&feature).expect("unknown feature"))
                    .collect(),
            })
            .collect(),
        state: connection::state::State::try_from(u8::try_from(connection.state.0).unwrap())
            .unwrap(),
        counterparty: connection::counterparty::Counterparty {
            client_id: connection.counterparty.client_id.parse().unwrap(),
            connection_id: if connection.counterparty.connection_id.is_empty() {
                None
            } else {
                Some(connection.counterparty.connection_id.parse().unwrap())
            },
            prefix: MerklePrefix {
                key_prefix: connection.counterparty.prefix.key_prefix.into(),
            },
        },
        delay_period: connection.delay_period.0,
    }
}

pub fn convert_channel(channel: aptos_move_ibc::channel::Channel) -> Channel {
    Channel {
        state: channel.state.try_into().unwrap(),
        ordering: channel.ordering.try_into().unwrap(),
        counterparty: channel::counterparty::Counterparty {
            port_id: channel.counterparty.port_id.parse().unwrap(),
            channel_id: channel.counterparty.channel_id.parse().unwrap(),
        },
        connection_hops: channel
            .connection_hops
            .into_iter()
            .map(|hop| hop.parse().unwrap())
            .collect(),
        version: channel.version,
    }
}
