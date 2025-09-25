use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    convert,
    fmt::Debug,
    future::Future,
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use either::Either;
use futures::{stream::FuturesOrdered, StreamExt};
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, error, info, instrument, trace, warn};
use unionlabs::{ibc::core::client::height::Height, id::ClientId, ErrorReporter};
use voyager_sdk::{
    anyhow,
    hook::simple_take_filter,
    message::{
        call::WaitForHeight,
        data::{ChainEvent, Data, EventProvableHeight},
        PluginMessage, VoyagerMessage,
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec, QueryHeight},
    rpc::{types::PluginInfo, PluginServer, FATAL_JSONRPC_ERROR_CODE},
    types::RawClientId,
    vm::{call, conc, data, noop, pass::PassResult, seq, Op},
    DefaultCmd, ExtensionsExt, VoyagerClient,
};

use crate::{
    call::{MakeTransactionBatchesWithUpdate, ModuleCall},
    callback::ModuleCallback,
    data::{BatchableEvent, EventBatch, EventClassic, EventUnion, ModuleData},
};

pub mod call;
pub mod callback;
pub mod data;

#[derive(Debug, Clone)]
pub struct Module {
    // The destination chain (i.e. where the messages will be sent to)
    pub chain_id: ChainId,
    pub client_configs: ClientConfigs,
}

#[derive(Debug, Clone)]
pub enum ClientConfigs {
    Any(ClientConfig),
    Many(HashMap<RawClientId, ClientConfig>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub client_configs: ClientConfigsSerde,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClientConfigsSerde {
    Any(ClientConfig),
    Many(Vec<SpecificClientConfig>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificClientConfig {
    pub client_id: RawClientId,
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
}

impl SpecificClientConfig {
    fn into_config(self) -> (RawClientId, ClientConfig) {
        (
            self.client_id,
            ClientConfig {
                min_batch_size: self.min_batch_size,
                max_batch_size: self.max_batch_size,
                max_wait_time: self.max_wait_time,
            },
        )
    }
}

pub trait IbcSpecExt: IbcSpec + Clone {
    type BatchableEvent: TryFrom<Self::Event, Error = ()>
        + Eq
        + Serialize
        + DeserializeOwned
        + Debug
        + Clone;

    fn proof_height(msg: &Self::Datagram) -> Height;

    fn event_name(msg: &Self::BatchableEvent) -> &'static str;
}

impl IbcSpecExt for IbcClassic {
    type BatchableEvent = crate::data::EventClassic;

    fn proof_height(msg: &Self::Datagram) -> Height {
        msg.proof_height()
            .expect("all batchable messages have a proof")
    }

    fn event_name(msg: &Self::BatchableEvent) -> &'static str {
        match msg {
            EventClassic::ConnectionOpenInit(_) => "connection_open_init",
            EventClassic::ConnectionOpenTry(_) => "connection_open_try",
            EventClassic::ConnectionOpenAck(_) => "connection_open_ack",
            EventClassic::ChannelOpenInit(_) => "channel_open_init",
            EventClassic::ChannelOpenTry(_) => "channel_open_try",
            EventClassic::ChannelOpenAck(_) => "channel_open_ack",
            EventClassic::SendPacket(_) => "send_packet",
            EventClassic::WriteAcknowledgement(_) => "write_ack",
        }
    }
}

impl IbcSpecExt for IbcUnion {
    type BatchableEvent = crate::data::EventUnion;

    fn proof_height(msg: &Self::Datagram) -> Height {
        msg.proof_height()
            .expect("all batchable messages have a proof")
    }

    fn event_name(msg: &Self::BatchableEvent) -> &'static str {
        match msg {
            EventUnion::ConnectionOpenInit(_) => "connection_open_init",
            EventUnion::ConnectionOpenTry(_) => "connection_open_try",
            EventUnion::ConnectionOpenAck(_) => "connection_open_ack",
            EventUnion::ChannelOpenInit(_) => "channel_open_init",
            EventUnion::ChannelOpenTry(_) => "channel_open_try",
            EventUnion::ChannelOpenAck(_) => "channel_open_ack",
            EventUnion::PacketSend(_) => "packet_send",
            EventUnion::BatchSend(_) => "batch_send",
            EventUnion::WriteAck(_) => "write_ack",
        }
    }
}

impl ClientConfigs {
    fn new(configs: ClientConfigsSerde) -> Self {
        match configs {
            ClientConfigsSerde::Any(client_config) => ClientConfigs::Any(client_config),
            ClientConfigsSerde::Many(vec) => {
                ClientConfigs::Many(vec.into_iter().map(|s| s.into_config()).collect())
            }
        }
    }

    fn config_for_client<V: IbcSpec>(&self, client_id: &V::ClientId) -> &ClientConfig {
        match &self {
            ClientConfigs::Any(any) => any,
            ClientConfigs::Many(many) => &many[&RawClientId::new(client_id)],
        }
    }

    fn jaq_filter(&self) -> String {
        match self {
            ClientConfigs::Any(_) => "true".to_owned(),
            ClientConfigs::Many(many) => {
                let clients_json =
                    serde_json::to_string(&many.keys().map(|k| (k, ())).collect::<HashMap<_, _>>())
                        .unwrap();

                format!("{clients_json} | has($client_id)")
            }
        }
    }
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: simple_take_filter(format!(
                r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events that cause an action on this chain (i.e. where we are the destination)
    # the counterparty of the event origin is the destination

    ($data."@type" == "plugin"
        and $data."@value".plugin == "{plugin_name}"
        and (
            $data."@value".message."@type" == "batch_events_union"
            or $data."@value".message."@type" == "batch_events_v1"
    )) or

    # ibc v1
    if $data."@type" == "ibc_event" and $data."@value".counterparty_chain_id == "{chain_id}" and $data."@value".ibc_spec_id == "{ibc_v1_id}" then
        $data."@value".event."@type" as $event_type |
        $data."@value".event."@value" as $event_data |

        (
            $event_type == "connection_open_init"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "connection_open_try"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "connection_open_ack"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_init"
            and ($event_data.connection.counterparty.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_try"
            and ($event_data.connection.counterparty.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_ack"
            and ($event_data.connection.counterparty.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "send_packet"
            and ($event_data.packet.destination_channel.connection.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "write_ack"
            and ($event_data.packet.source_channel.connection.client_id as $client_id | {clients_filter})
        )
    # ibc union
    elif $data."@type" == "ibc_event" and $data."@value".counterparty_chain_id == "{chain_id}" and $data."@value".ibc_spec_id == "{ibc_union_id}" then
        $data."@value".event."@type" as $event_type |
        $data."@value".event."@value" as $event_data |

        (
            $event_type == "connection_open_init"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "connection_open_try"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "connection_open_ack"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_init"
            and ($event_data.connection.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_try"
            and ($event_data.connection.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_ack"
            and ($event_data.connection.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "packet_send"
            and ($event_data.packet.destination_channel.connection.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "batch_send"
            and ($event_data.destination_channel.connection.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "write_ack"
            and ($event_data.packet.source_channel.connection.client_id as $client_id | {clients_filter})
        )
    else
        false
    end
else
    false
end
"#,
                chain_id = module.chain_id,
                plugin_name = module.plugin_name(),
                clients_filter = module.client_configs.jaq_filter(),
                ibc_v1_id = IbcClassic::ID,
                ibc_union_id = IbcUnion::ID,
            )),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub fn new(config: Config) -> Self {
        // // TODO: Make this a better error
        // assert!(config.min_batch_size <= config.max_batch_size);

        Self {
            chain_id: config.chain_id,
            client_configs: ClientConfigs::new(config.client_configs),
        }
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        e: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        self.run_pass_internal(e, msgs).await
    }

    #[instrument(skip_all)]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        let voyager_client = e.voyager_client()?;

        match msg {
            ModuleCall::MakeTransactionBatchesWithUpdateClassic(mk) => {
                mk.call(self, e.voyager_client()?).await
            }
            ModuleCall::MakeTransactionBatchesWithUpdateUnion(mk) => {
                mk.call(self, e.voyager_client()?).await
            }
            ModuleCall::MakeMsgClassic(mk) => mk.call(voyager_client).await,
            ModuleCall::MakeMsgUnion(mk) => mk.call(voyager_client).await,
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        e: &Extensions,
        cb: ModuleCallback,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {
            ModuleCallback::MakeIbcMessagesFromUpdateClassic(cb) => {
                cb.call(e.voyager_client()?, self, datas).await
            }
            ModuleCallback::MakeIbcMessagesFromUpdateUnion(cb) => {
                cb.call(e.voyager_client()?, self, datas).await
            }
            ModuleCallback::MakeBatchTransactionV1(cb) => {
                cb.call(self, e.voyager_client()?, self.chain_id.clone(), datas)
                    .await
            }
            ModuleCallback::MakeBatchTransactionUnion(cb) => {
                cb.call(self, e.voyager_client()?, self.chain_id.clone(), datas)
                    .await
            }
        }
    }
}

impl Module {
    #[allow(clippy::type_complexity)] // if you knew why this was here you'd leave me alone
    fn run_pass_internal<'a>(
        &'a self,
        e: &'a Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> Pin<Box<dyn Future<Output = RpcResult<PassResult<VoyagerMessage>>> + Send + 'a>> {
        Box::pin(async move {
            let mut batchers_classic =
                HashMap::<ClientId, Vec<(usize, BatchableEvent<IbcClassic>)>>::new();
            let mut batchers_union =
                HashMap::<ibc_union_spec::ClientId, Vec<(usize, BatchableEvent<IbcUnion>)>>::new();

            for (idx, msg) in msgs.into_iter().enumerate() {
                let Op::Data(msg) = msg else {
                    error!("unexpected message: {msg:?}");
                    continue;
                };

                match ChainEvent::try_from(msg) {
                    Ok(chain_event) => {
                        let first_seen_at: u64 = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                            .try_into()
                            .expect("how many milliseconds can there be man");

                        // client_id is the client id of the client on this chain (we are the counterparty from the perspective of the chain where the event was emitted)
                        // this is the client that will need to be updated before this ibc message can be sent

                        if let Some(full_ibc_event) = chain_event.decode_event::<IbcClassic>() {
                            let full_ibc_event = full_ibc_event.unwrap();

                            let client_id = full_ibc_event
                                .counterparty_client_id()
                                .expect("all batchable messages have a counterparty");

                            trace!(%client_id, "batching event");

                            batchers_classic
                                .entry(client_id.clone())
                                .or_default()
                                .push((
                                    idx,
                                    BatchableEvent {
                                        first_seen_at,
                                        provable_height: chain_event.provable_height,
                                        // TODO: Handle this more gracefully
                                        event: full_ibc_event.try_into().unwrap(),
                                    },
                                ));
                        }

                        if let Some(full_ibc_event) = chain_event.decode_event::<IbcUnion>() {
                            let full_ibc_event = full_ibc_event.unwrap();

                            let client_id = full_ibc_event
                                .counterparty_client_id()
                                .expect("all batchable messages have a counterparty");

                            trace!(%client_id, "batching event");

                            batchers_union.entry(client_id).or_default().push((
                                idx,
                                BatchableEvent {
                                    first_seen_at,
                                    provable_height: chain_event.provable_height,
                                    // TODO: Handle this more gracefully
                                    event: full_ibc_event.try_into().unwrap(),
                                },
                            ));
                        }
                    }
                    Err(msg) => {
                        match msg.as_plugin::<ModuleData>(self.plugin_name()) {
                            Ok(ModuleData::BatchEventsClassic(message)) => {
                                trace!(
                                    client_id = %message.client_id,
                                    events.len = %message.events.len(),
                                    "batching event"
                                );

                                batchers_classic
                                    .entry(message.client_id)
                                    .or_default()
                                    .extend(message.events.into_iter().map(|event| (idx, event)));
                            }
                            Ok(ModuleData::BatchEventsUnion(message)) => {
                                trace!(
                                    client_id = %message.client_id,
                                    events.len = %message.events.len(),
                                    "batching event"
                                );

                                batchers_union
                                    .entry(message.client_id)
                                    .or_default()
                                    .extend(message.events.into_iter().map(|event| (idx, event)));
                            }

                            Ok(msg) => {
                                error!("unexpected message: {msg:?}");
                            }

                            Err(msg) => {
                                error!("unexpected message: {msg:?}");
                            }
                        };
                    }
                };
            }

            let (ready_v1, optimize_further_v1) = batchers_classic
                .into_iter()
                .flat_map(|(client_id, events)| split_ready(client_id, events, self))
                .partition_map::<Vec<_>, Vec<_>, _, _, _>(convert::identity);

            let (ready_union, optimize_further_union) = batchers_union
                .into_iter()
                .flat_map(|(client_id, events)| split_ready(client_id, events, self))
                .partition_map::<Vec<_>, Vec<_>, _, _, _>(convert::identity);

            let voyager_client = e.voyager_client()?;

            let (ready_v1_errored, ready_v1) = ready_v1
                .into_iter()
                .into_group_map()
                .into_iter()
                .map(|(client_id, events)| mk_ready_ops(client_id, events, self, voyager_client))
                .collect::<FuturesOrdered<_>>()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .partition_map::<Vec<_>, Vec<_>, _, _, _>(Either::from);

            let (ready_union_errored, ready_union) = ready_union
                .into_iter()
                .into_group_map()
                .into_iter()
                .map(|(client_id, events)| mk_ready_ops(client_id, events, self, voyager_client))
                .collect::<FuturesOrdered<_>>()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .partition_map::<Vec<_>, Vec<_>, _, _, _>(Either::from);

            Ok(PassResult {
                optimize_further: optimize_further_v1
                    .into_iter()
                    .chain(optimize_further_union)
                    .chain(ready_v1_errored.into_iter().flatten())
                    .chain(ready_union_errored.into_iter().flatten())
                    .collect(),
                ready: ready_v1.into_iter().chain(ready_union).collect(),
            })
        })
    }
}

#[allow(clippy::type_complexity)] // skill issue
fn split_ready<V: IbcSpecExt>(
    client_id: V::ClientId,
    mut events: Vec<(usize, BatchableEvent<V>)>,
    this: &Module,
) -> Vec<
    Either<
        // ready
        (V::ClientId, (Vec<usize>, Vec<BatchableEvent<V>>)),
        // optimize further
        (Vec<usize>, Op<VoyagerMessage>, String),
    >,
>
where
    ModuleData: From<EventBatch<V>>,
{
    let client_config = &this.client_configs.config_for_client::<V>(&client_id);

    events.sort_by_key(|e| e.1.first_seen_at);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let is_overdue =
        |first_seen_at| Duration::from_millis(first_seen_at) + client_config.max_wait_time < now;

    let (mut overdue_events, mut events): (Vec<_>, Vec<_>) =
        events.into_iter().partition_map(|e| {
            if is_overdue(e.1.first_seen_at) {
                Either::Left(e)
            } else {
                Either::Right(e)
            }
        });

    events.sort_by_key(|e| *e.1.provable_height.height());
    overdue_events.sort_by_key(|e| *e.1.provable_height.height());

    if !overdue_events.is_empty()
        && overdue_events.len() + events.len() < client_config.min_batch_size
    {
        warn!(
            "found {} overdue events and {} non-overdue events, but the min batch \
            size for this client ({client_id}) is {}",
            overdue_events.len(),
            events.len(),
            client_config.min_batch_size
        );
    }

    // [...overdue_events_sorted_by_provable_height, ...events_sorted_by_provable_height]
    overdue_events
        .into_iter()
        .chain(events)
        .chunks(client_config.max_batch_size)
        .into_iter()
        .map(move |chunk| {
            let (idxs, events): (Vec<_>, Vec<_>) = chunk.into_iter().unzip();

            if events.len() == client_config.max_batch_size
                || events.iter().any(|e| is_overdue(e.first_seen_at))
            {
                // this batch is ready to send out, we need to fetch an update for the client on our chain and turn the events into `IbcMessage`s.
                //
                // in order to do this, we first need to figure out what height the client is at, and request an update from that height to a height >= the highest height of all of the messages in this batch.
                // note that we can't request a *specific* height to update to, since not all chains provide this functionality (ethereum being a notable one) - we instead need to wait for the update to be constructed, and then use the new trusted height of the update to fetch our proofs from.
                //
                // this will be done in a multi-step aggregation, where first we fetch the update, then construct the messages, and then turn that into a batch transaction.
                Either::Left((client_id.clone(), (idxs, events)))
            } else {
                Either::Right((
                    idxs,
                    data(PluginMessage::new(
                        this.plugin_name(),
                        ModuleData::from(EventBatch {
                            client_id: client_id.clone(),
                            events,
                        }),
                    )),
                    this.plugin_name(),
                ))
            }
        })
        .collect::<Vec<_>>()
}

#[allow(unstable_name_collisions)] // for Itertools::intersperse
async fn mk_ready_ops<V: IbcSpecExt>(
    client_id: V::ClientId,
    events: Vec<(Vec<usize>, Vec<BatchableEvent<V>>)>,
    module: &Module,
    voyager_client: &VoyagerClient,
) -> Result<(Vec<usize>, Op<VoyagerMessage>), Vec<(Vec<usize>, Op<VoyagerMessage>, String)>>
where
    ModuleCall: From<MakeTransactionBatchesWithUpdate<V>>,
    ModuleData: From<EventBatch<V>>,
{
    // the height on the counterparty chain that all of the events in these batches are provable at
    // we only want to generate one update for all of these batches
    let (min_target_height, exact_target_heights) = events
        .iter()
        .flat_map(|x| &x.1)
        .map(|e| e.provable_height)
        .fold((None, BTreeSet::new()), |mut acc, elem| match elem {
            EventProvableHeight::Min(height) => (
                Some(acc.0.map(|h: Height| h.min(height)).unwrap_or(height)),
                acc.1,
            ),
            EventProvableHeight::Exactly(height) => {
                acc.1.insert(height);
                acc
            }
        });

    info!(
        "target height of updates for batch is min {}, exact [{}]",
        min_target_height.map_or("<none>".to_string(), |h: Height| h.to_string()),
        exact_target_heights
            .iter()
            .map(|h| h.to_string())
            .intersperse(",".to_string())
            .collect::<String>(),
    );

    debug!(%client_id, "querying client state meta for client");

    let client_state_meta = match voyager_client
        .client_state_meta::<V>(
            module.chain_id.clone(),
            QueryHeight::Latest,
            client_id.clone(),
        )
        .await
    {
        Ok(ok) => ok,
        Err(err) => {
            if err.code() == FATAL_JSONRPC_ERROR_CODE {
                error!(
                    error = %ErrorReporter(err),
                    "fatal error fetching client state meta for client {client_id} on chain {}", module.chain_id
                );

                return Ok((vec![], noop()));
            } else {
                error!(
                    error = %ErrorReporter(err),
                    "error fetching client state meta for client {client_id} on chain {}", module.chain_id
                );
            }

            return Err(events
                .into_iter()
                .map(|(idxs, events)| {
                    (
                        idxs,
                        data(PluginMessage::new(
                            module.plugin_name(),
                            ModuleData::from(EventBatch {
                                client_id: client_id.clone(),
                                events,
                            }),
                        )),
                        module.plugin_name(),
                    )
                })
                .collect());
        }
    };

    let (idxs, events): (Vec<_>, Vec<_>) = events.into_iter().unzip();

    Ok((
        idxs.into_iter().flatten().collect::<Vec<_>>(),
        // REVIEW: This might need to be a seq depending on what the impl of the client update plugin is
        conc(
            exact_target_heights
                .into_iter()
                .chain(min_target_height)
                .map(|height| {
                    seq([
                        call(WaitForHeight {
                            chain_id: client_state_meta.counterparty_chain_id.clone(),
                            height,
                            finalized: true,
                        }),
                        call(PluginMessage::new(
                            module.plugin_name(),
                            ModuleCall::from(MakeTransactionBatchesWithUpdate {
                                client_id: client_id.clone(),
                                batches: events.clone(),
                            }),
                        )),
                    ])
                }),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn config_serde() {
        let config_json = json!({
          "chain_id": "union-devnet-1",
          "client_configs": {
            "min_batch_size": 1,
            "max_batch_size": 3,
            "max_wait_time": {
              "secs": 10,
              "nanos": 0
            }
          }
        });

        let config = serde_json::from_value::<Config>(config_json).unwrap();

        assert_eq!(
            config,
            Config {
                chain_id: ChainId::new("union-devnet-1"),
                client_configs: ClientConfigsSerde::Any(ClientConfig {
                    min_batch_size: 1,
                    max_batch_size: 3,
                    max_wait_time: Duration::from_secs(10)
                })
            }
        );
    }
}
