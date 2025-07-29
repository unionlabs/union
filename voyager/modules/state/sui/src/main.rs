use std::fmt::Debug;

use ibc_union_spec::{
    path::StorePath,
    query::{PacketByHash, Query},
    Channel, ChannelId, ChannelState, ClientId, Connection, ConnectionState, IbcUnion, Packet,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_sdk::{
    rpc_types::{SuiMoveValue, SuiObjectDataOptions, SuiParsedData, SuiTypeTag},
    types::{
        base_types::{ObjectID, SuiAddress},
        dynamic_field::DynamicFieldName,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, CallArg, Command, ObjectArg, TransactionKind},
        Identifier, TypeTag,
    },
    SuiClient, SuiClientBuilder,
};
use tracing::instrument;
use unionlabs::{
    encoding::{Bcs, DecodeAs as _},
    ibc::core::client::height::Height,
    primitives::{Bytes, H256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::StateModule,
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface},
    rpc::{types::StateModuleInfo, StateModuleServer, FATAL_JSONRPC_ERROR_CODE},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    VaultAddress,
    SubmitTx,
    FetchAbi,
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub rpc_url: String,

    pub sui_client: sui_sdk::SuiClient,

    pub ibc_store: ObjectID,

    pub ibc_contract: ObjectID,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PacketSend {
    pub channel_id: u32,
    pub packet_hash: Vec<u8>,

    pub packet: Packet,
}

impl Module {
    pub async fn query_packet_by_hash(
        &self,
        _channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<Packet> {
        let SuiParsedData::MoveObject(object) = self
            .sui_client
            .read_api()
            .get_dynamic_field_object(
                self.ibc_store,
                DynamicFieldName {
                    type_: TypeTag::Vector(Box::new(TypeTag::U8)),
                    value: serde_json::to_value(&packet_hash).expect("serde will work"),
                },
            )
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to fetch the packet hash: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            .data
            .expect("data exists")
            .content
            .expect("content exists")
        else {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "unexpected data found when trying to fetch the packet hash to digest",
                None::<()>,
            ));
        };

        let SuiMoveValue::Vector(v) = object
            .fields
            .field_value("value")
            .expect("table has a value")
        else {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "invalid value type when parsing the packet to digest",
                None::<()>,
            ));
        };

        let digest: Vec<u8> = v
            .into_iter()
            .map(|n| {
                let SuiMoveValue::Number(n) = n else {
                    panic!("this will always be u8");
                };

                n as u8
            })
            .collect();

        let packet = self
            .sui_client
            .event_api()
            .get_events(digest.try_into().expect("ibc saves tx digest"))
            .await
            .expect("there must be some events exist")
            .into_iter()
            .find_map(|e| {
                if e.type_.address == self.ibc_contract.into()
                    && e.type_.module.as_str() == "ibc"
                    && e.type_.name.as_str() == "PacketSend"
                {
                    let send: PacketSend = serde_json::from_value(e.parsed_json).unwrap();
                    if &send.packet_hash == packet_hash.as_ref() {
                        Some(send.packet)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "couldn't find the send event with packet hash: {}",
                    packet_hash
                ),
                None::<()>,
            ))?;

        Ok(packet)
    }
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(&chain_id)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            sui_client,
            rpc_url: config.rpc_url,
            ibc_store: config.ibc_store,
            ibc_contract: config.ibc_contract,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_store: ObjectID,
    pub ibc_contract: ObjectID,
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, _: &Extensions, query: Query) -> RpcResult<Value> {
        match query {
            Query::PacketByHash(PacketByHash {
                channel_id,
                packet_hash,
            }) => self
                .query_packet_by_hash(channel_id, packet_hash)
                .await
                .map(into_value),
            Query::PacketsByBatchHash(_packets_by_batch_hash) => todo!(),
            Query::ClientStatus(_client_status) => todo!(),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, _: ClientId) -> RpcResult<ClientInfo> {
        Ok(ClientInfo {
            // TODO(aeryz): make this queryable
            client_type: ClientType::new("cometbls"),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_MOVE_SUI),
            metadata: Default::default(),
        })
    }

    async fn query_ibc_state(
        &self,
        _: &Extensions,
        _: Height,
        path: StorePath,
    ) -> RpcResult<Value> {
        let query = SuiQuery::new(&self.sui_client, self.ibc_store).await;

        Ok(match path {
            StorePath::Connection(path) => {
                let res = query
                    .add_param(path.connection_id.raw())
                    .call(self.ibc_contract, "get_connection")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                into_value(convert_connection(
                    SuiConnection::decode_as::<Bcs>(&res[0].0).unwrap(),
                ))
            }
            StorePath::Channel(path) => {
                let res = query
                    .add_param(path.channel_id.raw())
                    .call(self.ibc_contract, "get_channel")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                into_value(convert_channel(
                    SuiChannel::decode_as::<Bcs>(&res[0].0).unwrap(),
                ))
            }
            StorePath::ClientState(path) => {
                let res = query
                    .add_param(path.client_id.raw())
                    .call(self.ibc_contract, "get_client_state")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("was expecting a single encoded client state");
                }

                // Doing 1.. here since the return data is bcs encoded vector<u8> which is
                // just `prefix + vector<u8>`
                let client_state_bytes: Bytes = res[0].clone().0[1..].into();

                into_value(Some(client_state_bytes))
            }
            StorePath::ConsensusState(path) => {
                let res = query
                    .add_param(path.client_id.raw())
                    .add_param(path.height)
                    .call(self.ibc_contract, "get_consensus_state")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("was expecting a single encoded consensus state");
                }

                // Doing 1.. here since the return data is bcs encoded vector<u8> which is
                // just `prefix + vector<u8>`
                let consensus_state_bytes: Bytes = res[0].clone().0[1..].into();

                into_value(consensus_state_bytes)
            }
            _ => todo!(),
        })
    }
}

struct SuiQuery<'a> {
    client: &'a SuiClient,
    params: Vec<CallArg>,
}

impl<'a> SuiQuery<'a> {
    async fn new(client: &'a SuiClient, ibc_store_id: ObjectID) -> Self {
        let object_ref = client
            .read_api()
            .get_object_with_options(ibc_store_id, SuiObjectDataOptions::new())
            .await
            .unwrap()
            .object_ref_if_exists()
            .unwrap();
        Self {
            client,
            params: vec![CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref))],
        }
    }

    fn add_param<T>(mut self, param: T) -> Self
    where
        T: serde::Serialize,
    {
        self.params
            .push(CallArg::Pure(bcs::to_bytes(&param).unwrap()));
        self
    }

    async fn call(
        self,
        package: ObjectID,
        function: &str,
    ) -> Result<Vec<(Vec<u8>, SuiTypeTag)>, String> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        ptb.command(Command::move_call(
            package,
            Identifier::new("ibc").unwrap(),
            Identifier::new(function).unwrap(),
            vec![],
            self.params
                .iter()
                .enumerate()
                .map(|(i, _)| Argument::Input(i as u16))
                .collect(),
        ));

        for arg in self.params {
            ptb.input(arg).unwrap();
        }

        let res = self
            .client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::ZERO,
                TransactionKind::ProgrammableTransaction(ptb.finish()),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        match (res.results, res.error) {
            (Some(res), _) => Ok(res[0].clone().return_values),
            (_, Some(err)) => Err(err),
            _ => panic!("invalid"),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SuiConnection {
    pub state: u8,
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
}

fn convert_connection(connection: SuiConnection) -> Connection {
    Connection {
        state: match connection.state {
            1 => ConnectionState::Init,
            2 => ConnectionState::TryOpen,
            3 => ConnectionState::Open,
            _ => panic!("connection state must be 1..=3"),
        },
        client_id: connection.client_id.try_into().unwrap(),
        counterparty_client_id: connection.counterparty_client_id.try_into().unwrap(),
        counterparty_connection_id: connection.counterparty_connection_id.try_into().ok(),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SuiChannel {
    pub state: u8,
    pub connection_id: u32,
    pub counterparty_channel_id: u32,
    pub counterparty_port_id: Vec<u8>,
    pub version: String,
}

fn convert_channel(channel: SuiChannel) -> Channel {
    Channel {
        state: match channel.state {
            1 => ChannelState::Init,
            2 => ChannelState::TryOpen,
            3 => ChannelState::Open,
            4 => ChannelState::Closed,
            _ => panic!("channel state must be 1..=4"),
        },
        connection_id: channel.connection_id.try_into().unwrap(),
        counterparty_channel_id: if channel.counterparty_channel_id == 0 {
            None
        } else {
            Some(channel.counterparty_channel_id.try_into().unwrap())
        },
        counterparty_port_id: channel.counterparty_port_id.into(),
        version: channel.version,
    }
}
