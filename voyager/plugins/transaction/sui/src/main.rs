use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fmt::Debug,
    panic::AssertUnwindSafe,
    str::FromStr,
    sync::Arc,
};

use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use ibc_union_spec::{IbcUnion, datagram::Datagram, path::ChannelPath};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use sui_sdk::{
    SuiClient, SuiClientBuilder,
    rpc_types::SuiObjectDataOptions,
    types::{
        base_types::{ObjectID, SequenceNumber, SuiAddress},
        crypto::SuiKeyPair,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, Command, ProgrammableTransaction},
    },
};
use tracing::instrument;
use unionlabs::{
    ErrorReporter,
    primitives::{Bytes, encoding::HexPrefixed},
};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient,
    anyhow::{self},
    hook::SubmitTxHook,
    message::{PluginMessage, VoyagerMessage, data::Data},
    plugin::Plugin,
    primitives::{ChainId, QueryHeight},
    rpc::{FATAL_JSONRPC_ERROR_CODE, PluginServer, types::PluginInfo},
    serde_json::{self, json},
    vm::{Op, Visit, call, noop, pass::PassResult},
};
use voyager_transaction_plugin_sui::{ModuleInfo, TransactionPluginClient, send_transactions};

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;
pub mod move_api;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_contract: SuiAddress,

    pub ibc_store: SuiAddress,

    pub graphql_url: String,

    pub sui_client: SuiClient,

    pub keyring: ConcurrentKeyring<SuiAddress, Arc<SuiKeyPair>>,

    pub ibc_store_initial_seq: SequenceNumber,

    pub channel_version_to_plugin: BTreeMap<String, String>,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        let ibc_store_initial_seq = sui_client
            .read_api()
            .get_object_with_options(
                ObjectID::new(config.ibc_store.to_inner()),
                SuiObjectDataOptions::default().with_owner(),
            )
            .await
            .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?
            .data
            .expect("ibc store object exists on chain")
            .owner
            .expect("owner will be present")
            .start_version()
            .expect("ibc store is shared, hence it has a start version");

        let query = json!({
            "query": "query ($address: SuiAddress) { packageVersions(address: $address, last: 1) { nodes { address } } }",
            "variables": { "address": config.ibc_contract }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&config.graphql_url)
            .header("Content-Type", "application/json")
            .body(query.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let v: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
        let ibc_contract = SuiAddress::from_str(
            v["data"]["packageVersions"]["nodes"][0]["address"]
                .as_str()
                .unwrap(),
        )
        .unwrap();

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_contract,
            sui_client,
            graphql_url: config.graphql_url,
            ibc_store_initial_seq,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    println!("{}", Bytes::<HexPrefixed>::new(config.value()));
                    let pk = SuiKeyPair::decode(
                        &String::from_utf8(config.value()).expect("priv keys are utf8 strings"),
                    )
                    .expect("private key is valid");

                    let address = SuiAddress::from(&pk.public());

                    KeyringEntry {
                        address,
                        signer: Arc::new(pk),
                    }
                }),
            ),
            ibc_store: config.ibc_store,
            channel_version_to_plugin: config.channel_version_to_plugin,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: SubmitTxHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub graphql_url: String,
    pub ibc_contract: SuiAddress,
    pub ibc_store: SuiAddress,
    pub keyring: KeyringConfig,
    pub channel_version_to_plugin: BTreeMap<String, String>,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    #[allow(clippy::type_complexity)]
    async fn process_msgs(
        &self,
        voyager_client: &VoyagerClient,
        pk: &Arc<SuiKeyPair>,
        msgs: Vec<Datagram>,
        fee_recipient: SuiAddress,
    ) -> anyhow::Result<ProgrammableTransaction> {
        let mut ptb_builder = ProgrammableTransactionBuilder::new();
        let mut ptb = ProgrammableTransactionBuilder::new().finish();
        for msg in msgs {
            match msg {
                Datagram::CreateClient(data) => {
                    move_api::create_client(&mut ptb_builder, self, data)?
                }
                Datagram::UpdateClient(data) => {
                    move_api::update_client(&mut ptb_builder, self, data)?
                }
                Datagram::ConnectionOpenInit(data) => {
                    move_api::connection_open_init(&mut ptb_builder, self, data)?
                }
                Datagram::ConnectionOpenTry(data) => {
                    move_api::connection_open_try(&mut ptb_builder, self, data)?
                }
                Datagram::ConnectionOpenAck(data) => {
                    move_api::connection_open_ack(&mut ptb_builder, self, data)?
                }
                Datagram::ConnectionOpenConfirm(data) => {
                    move_api::connection_open_confirm(&mut ptb_builder, self, data)?
                }
                Datagram::ChannelOpenInit(data) => move_api::channel_open_init(
                    &mut ptb_builder,
                    self,
                    self.module_info_from_port(&data.port_id).await?,
                    data,
                )?,
                Datagram::ChannelOpenTry(data) => move_api::channel_open_try(
                    &mut ptb_builder,
                    self,
                    self.module_info_from_port(&data.port_id).await?,
                    data,
                )?,
                Datagram::ChannelOpenAck(data) => move_api::channel_open_ack(
                    &mut ptb_builder,
                    self,
                    self.module_info_from_port(
                        move_api::get_port_id(self, data.channel_id).await?.as_ref(),
                    )
                    .await?,
                    data,
                )?,
                Datagram::ChannelOpenConfirm(data) => move_api::channel_open_confirm_call(
                    &mut ptb_builder,
                    self,
                    self.module_info_from_port(
                        move_api::get_port_id(self, data.channel_id).await?.as_ref(),
                    )
                    .await?,
                    data,
                )?,
                Datagram::PacketRecv(data) => {
                    let port_id =
                        move_api::get_port_id(self, data.packets[0].destination_channel_id).await?;

                    let module_info = self.module_info_from_port(port_id.as_ref()).await?;

                    let channel_version = voyager_client
                        .query_ibc_state(
                            self.chain_id.clone(),
                            QueryHeight::Latest,
                            ChannelPath {
                                channel_id: data.packets[0].destination_channel_id,
                            },
                        )
                        .await?
                        .version;

                    if let Some(plugin_client) =
                        self.channel_version_to_plugin.get(&channel_version)
                    {
                        let p = voyager_client
                            .plugin_client(plugin_client)
                            .on_recv_packet(
                                pk.copy(),
                                module_info.clone(),
                                fee_recipient,
                                data.clone(),
                            )
                            .await?;

                        merge_ptbs(&mut ptb, p);
                    } else {
                        // TODO(aeryz): Regular recv_packet here. Note that this is not used by us, therefore I don't want to rush
                        // it until we have the use case.
                        unimplemented!()
                    }
                }
                Datagram::PacketAcknowledgement(data) => {
                    // using the source channel id since the send happened on sui
                    let port_id =
                        move_api::get_port_id(self, data.packets[0].source_channel_id).await?;

                    let module_info = self.module_info_from_port(port_id.as_ref()).await?;

                    let channel_version = voyager_client
                        .query_ibc_state(
                            self.chain_id.clone(),
                            QueryHeight::Latest,
                            ChannelPath {
                                channel_id: data.packets[0].source_channel_id,
                            },
                        )
                        .await?
                        .version;

                    if let Some(plugin_client) =
                        self.channel_version_to_plugin.get(&channel_version)
                    {
                        let p = voyager_client
                            .plugin_client(plugin_client)
                            .on_acknowledge_packet(
                                pk.copy(),
                                module_info.clone(),
                                fee_recipient,
                                data.clone(),
                            )
                            .await?;
                        merge_ptbs(&mut ptb, p);
                    } else {
                        // TODO(aeryz): Regular ack_packet here. Note that this is not used by us, therefore I don't want to rush
                        // it until we have the use case.
                        unimplemented!()
                    }
                }
                Datagram::PacketTimeout(data) => {
                    // using the source channel id since the send happened on sui
                    let port_id =
                        move_api::get_port_id(self, data.packet.source_channel_id).await?;

                    let module_info = self.module_info_from_port(port_id.as_ref()).await?;

                    let channel_version = voyager_client
                        .query_ibc_state(
                            self.chain_id.clone(),
                            QueryHeight::Latest,
                            ChannelPath {
                                channel_id: data.packet.destination_channel_id,
                            },
                        )
                        .await?
                        .version;

                    if let Some(plugin_client) =
                        self.channel_version_to_plugin.get(&channel_version)
                    {
                        let p = voyager_client
                            .plugin_client(plugin_client)
                            .on_timeout_packet(pk.copy(), module_info.clone(), data.clone())
                            .await?;
                        merge_ptbs(&mut ptb, p);
                    } else {
                        // TODO(aeryz): Regular timeout_packet here. Note that this is not used by us, therefore I don't want to rush
                        // it until we have the use case.
                        unimplemented!()
                    }
                }
                Datagram::CommitPacketTimeout(data) => {
                    move_api::packet_timeout_commitment_call(&mut ptb_builder, self, data)?
                }
                _ => todo!(),
            }
        }

        // TODO(aeryz): this is messing with the tx order, instead merge on each loop
        let mut final_ptb = ptb_builder.finish();
        merge_ptbs(&mut final_ptb, ptb);

        Ok(final_ptb)
    }

    // original_address::module_name::store_address
    // TODO(aeryz): we can also choose to include store_name here
    pub async fn module_info_from_port(&self, port_id: &[u8]) -> RpcResult<ModuleInfo> {
        let port_id = ObjectID::try_from(port_id).map_err(|_| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "port parsing: port expected to be a object ID",
                Some(json!({
                    "port": port_id
                })),
            )
        })?;

        let bcs_port = self
            .sui_client
            .read_api()
            .get_move_object_bcs(port_id)
            .await
            .map_err(|_| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "port parsing: the object id must exist on chain",
                    Some(json!({
                        "port": port_id
                    })),
                )
            })?;

        let port: Port = bcs::from_bytes(&bcs_port).map_err(|_| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "port parsing: the port object is not in the correct format",
                Some(json!({
                    "port": port_id
                })),
            )
        })?;

        let query = json!({
            "query": "query ($address: SuiAddress) { packageVersions(address: $address, last: 1) { nodes { address } } }",
            "variables": { "address": port.module_address }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&self.graphql_url)
            .header("Content-Type", "application/json")
            .body(query.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let v: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
        let latest_address = SuiAddress::from_str(
            v["data"]["packageVersions"]["nodes"][0]["address"]
                .as_str()
                .unwrap(),
        )
        .unwrap();

        let initial_seq = self
            .sui_client
            .read_api()
            .get_object_with_options(
                port.store_address.into(),
                SuiObjectDataOptions::new().with_owner(),
            )
            .await
            .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?
            .data
            .expect("store must exist on chain")
            .owner
            .expect("owner will be present")
            .start_version()
            .expect("store is shared, hence it has a start version");

        Ok(ModuleInfo {
            original_address: port.module_address,
            latest_address,
            stores: vec![(port.store_address, initial_seq)],
        })
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, mut op)| {
                    SubmitTxHook::new(&self.chain_id, |submit_tx| {
                        PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::SubmitTransaction(
                                submit_tx
                                    .datagrams
                                    .iter()
                                    .map(|message| {
                                        message.decode_datagram::<IbcUnion>().unwrap().unwrap()
                                    })
                                    .collect(),
                            ),
                        )
                        .into()
                    })
                    .visit_op(&mut op);

                    (vec![idx], op)
                })
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => self
                .keyring
                .with(|pk| {
                    let sender = SuiAddress::from(&pk.public());
                    let msgs = msgs.clone();
                    AssertUnwindSafe(async move {
                        let ptb = self
                            .process_msgs(e.voyager_client()?, pk, msgs, sender)
                            .await
                            .unwrap();

                        let _ = send_transactions(&self.sui_client, pk, ptb).await?;
                        Ok(noop())
                    })
                })
                .await
                .unwrap_or_else(|| {
                    Ok(call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::SubmitTransaction(msgs),
                    )))
                }),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

#[derive(Deserialize)]
pub struct Port {
    _id: ObjectID,
    module_address: SuiAddress,
    store_address: SuiAddress,
}

pub fn merge_ptbs(lhs: &mut ProgrammableTransaction, rhs: ProgrammableTransaction) {
    let mut call_arg_indices = HashMap::new();
    let mut cursor: u16 = 0;

    let mut unique_inputs = Vec::new();

    lhs.inputs.iter().chain(rhs.inputs.iter()).for_each(|i| {
        if !call_arg_indices.contains_key(i) {
            unique_inputs.push(i.clone());
            call_arg_indices.insert(i.clone(), cursor);
            cursor += 1;
        }
    });

    lhs.inputs = unique_inputs;

    let result_offset = lhs.commands.len() as u16;

    let adjust_indices = |arg: &mut Argument| match arg {
        Argument::Input(i) => *i = *call_arg_indices.get(&rhs.inputs[*i as usize]).unwrap(),
        Argument::Result(i) => *i += result_offset,
        Argument::NestedResult(i, _) => *i += result_offset,
        _ => {}
    };

    for mut command in rhs.commands {
        match command {
            Command::MoveCall(ref mut call) => call.arguments.iter_mut().for_each(adjust_indices),
            Command::TransferObjects(ref mut args, ref mut arg) => {
                args.iter_mut().for_each(adjust_indices);
                adjust_indices(arg);
            }
            Command::SplitCoins(ref mut arg, ref mut args) => {
                adjust_indices(arg);
                args.iter_mut().for_each(adjust_indices);
            }
            Command::MergeCoins(ref mut arg, ref mut args) => {
                adjust_indices(arg);
                args.iter_mut().for_each(adjust_indices);
            }
            Command::MakeMoveVec(_, ref mut args) => args.iter_mut().for_each(adjust_indices),
            Command::Upgrade(_, _, _, ref mut arg) => adjust_indices(arg),
            _ => {}
        };

        lhs.commands.push(command);
    }
}
