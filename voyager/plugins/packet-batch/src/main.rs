use std::{
    collections::VecDeque,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use either::Either;
use ibc_union_spec::{
    datagram::{MsgBatchAcks, MsgBatchSend},
    event::FullEvent,
    ChannelId, IbcUnion, Packet,
};
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use unionlabs::never::Never;
use voyager_sdk::{
    anyhow,
    hook::simple_take_filter,
    message::{
        call::SubmitTx,
        data::{Data, IbcDatagram},
        PluginMessage, VoyagerMessage,
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec},
    rpc::{types::PluginInfo, PluginServer},
    vm::{call, data, pass::PassResult, Op},
    DefaultCmd,
};

use crate::data::{BatchAck, BatchSend, ModuleData};

pub mod data;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub channel_id: ChannelId,
    pub max_batch_count: usize,
    pub max_batch_size_bytes: usize,
    pub max_wait_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub channel_id: ChannelId,
    pub max_batch_count: usize,
    pub max_batch_size_bytes: usize,
    pub max_wait_time: Duration,
}

impl Plugin for Module {
    type Call = Never;
    type Callback = Never;

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

    if
        $data."@type" == "ibc_event"
        and $data."@value".chain_id == "{chain_id}"
        and $data."@value".ibc_spec_id == "{ibc_union_id}"
    then
        $data."@value".event."@type" as $event_type |
        $data."@value".event."@value" as $event_data |

        (
            $event_type == "packet_send"
            and ($event_data.packet.source_channel.channel_id == {channel_id})
        ) or (
            $event_type == "write_ack"
            and ($event_data.packet.destination_channel.channel_id == {channel_id})
        )
    else
        false
    end
else
    false
end
"#,
                chain_id = module.chain_id,
                channel_id = module.channel_id,
                ibc_union_id = IbcUnion::ID,
            )),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub fn plugin_name(chain_id: &ChainId, channel_id: ChannelId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}/{}", chain_id, channel_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id, self.channel_id)
    }

    pub fn new(config: Config) -> Self {
        Self {
            chain_id: config.chain_id,
            channel_id: config.channel_id,
            max_batch_count: config.max_batch_count,
            max_batch_size_bytes: config.max_batch_size_bytes,
            max_wait_time: config.max_wait_time,
        }
    }
}

#[async_trait]
impl PluginServer<Never, Never> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let (send, ack) =
            msgs.into_iter()
                .enumerate()
                .filter_map(|(idx, op)| {
                    op.into_data()
                        .and_then(|d| match d {
                            // merge in new events
                            Data::IbcEvent(chain_event) => {
                                match chain_event.decode_event::<IbcUnion>().unwrap().unwrap() {
                                    FullEvent::PacketSend(e) => Some(ModuleData::BatchSendPacket(
                                        vec![BatchSend::new(e.packet())],
                                    )),
                                    FullEvent::WriteAck(e) => Some(ModuleData::BatchAckPacket(
                                        vec![BatchAck::new(e.packet(), e.acknowledgement)],
                                    )),
                                    event => panic!("{event:?}"),
                                }
                            }
                            d => d.as_plugin::<ModuleData>(self.plugin_name()).ok(),
                        })
                        .map(|d| match d {
                            ModuleData::BatchSendPacket(batch) => Either::Left((idx, batch)),
                            ModuleData::BatchAckPacket(batch) => Either::Right((idx, batch)),
                        })
                })
                .partition_map::<Vec<_>, Vec<_>, _, _, _>(std::convert::identity);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .expect("how many milliseconds can there be man");

        fn f<T>(
            msgs: Vec<(usize, Vec<T>)>,
            now: u64,
            get_first_seen_at: fn(&T) -> u64,
            get_packet: fn(&T) -> &Packet,
            max_batch_count: &usize,
            max_batch_size_bytes: &usize,
            max_wait_time: &Duration,
        ) -> (Vec<(Vec<usize>, Vec<T>)>, Vec<(Vec<usize>, Vec<T>)>) {
            let (ready, wait) = msgs
                .into_iter()
                .flat_map(|(idx, ts)| ts.into_iter().map(move |t| (idx, t)))
                .sorted_by_key(|(_, t)| get_first_seen_at(t))
                .fold(vec![(false, vec![], vec![])], |mut acc, (idx, t)| {
                    let (full, idxs, current_batch) = acc.last_mut().unwrap();

                    let size = current_batch
                        .iter()
                        .map(|t| get_packet(t).data.len())
                        .sum::<usize>();

                    // first check size, then if the batch would not be too large, check if it would have too many items
                    if size + get_packet(&t).data.len() > *max_batch_size_bytes {
                        // batch is full (size), start a new one
                        info!(
                            max_size = %max_batch_size_bytes,
                            batch_size = %size,
                            "filled batch (size)"
                        );
                        *full = true;

                        acc.push((false, vec![idx], vec![t]));
                    } else if current_batch.len() + 1 > *max_batch_count {
                        // batch is full (count), start a new one
                        info!(count = %max_batch_count, "filled batch (count)");
                        *full = true;

                        acc.push((false, vec![idx], vec![t]));
                    } else {
                        // batch has room
                        idxs.push(idx);
                        current_batch.push(t);
                    }

                    acc
                })
                .into_iter()
                .partition::<Vec<_>, _>(|(full, _idxs, ts)| {
                    *full
                        || ts.iter().any(|t| {
                            (get_first_seen_at(t) + max_wait_time.as_millis() as u64) < now
                        })
                });

            (
                ready.into_iter().map(|(_, idxs, ts)| (idxs, ts)).collect(),
                wait.into_iter().map(|(_, idxs, ts)| (idxs, ts)).collect(),
            )
        }

        let (send_ready, send_wait) = f(
            send,
            now,
            |t| t.first_seen_at,
            |t| &t.packet,
            &self.max_batch_count,
            &self.max_batch_size_bytes,
            &self.max_wait_time,
        );
        let (ack_ready, ack_wait) = f(
            ack,
            now,
            |t| t.first_seen_at,
            |t| &t.packet,
            &self.max_batch_count,
            &self.max_batch_size_bytes,
            &self.max_wait_time,
        );

        Ok(PassResult {
            optimize_further: send_wait
                .into_iter()
                .map(|(idxs, events)| {
                    (
                        idxs,
                        data(PluginMessage::new(
                            self.plugin_name(),
                            ModuleData::BatchSendPacket(events),
                        )),
                        self.plugin_name(),
                    )
                })
                .chain(ack_wait.into_iter().map(|(idxs, events)| {
                    (
                        idxs,
                        data(PluginMessage::new(
                            self.plugin_name(),
                            ModuleData::BatchAckPacket(events),
                        )),
                        self.plugin_name(),
                    )
                }))
                .collect(),
            ready: send_ready
                .into_iter()
                .map(|(idxs, d)| {
                    let mut packets = d.into_iter().map(|d| d.packet).collect::<Vec<_>>();
                    packets.sort_by_cached_key(|packet| packet.hash());

                    (
                        idxs,
                        call(SubmitTx {
                            chain_id: self.chain_id.clone(),
                            datagrams: vec![IbcDatagram::new::<IbcUnion>(MsgBatchSend { packets })],
                        }),
                    )
                })
                .chain(ack_ready.into_iter().map(|(idxs, d)| {
                    let (packets, acks) = d
                        .into_iter()
                        .map(|d| (d.packet, d.ack))
                        .sorted_by_cached_key(|(packet, _ack)| packet.hash())
                        .unzip();

                    (
                        idxs,
                        call(SubmitTx {
                            chain_id: self.chain_id.clone(),
                            datagrams: vec![IbcDatagram::new::<IbcUnion>(MsgBatchAcks {
                                packets,
                                acks,
                            })],
                        }),
                    )
                }))
                .collect(),
        })
    }

    #[instrument(skip_all)]
    async fn call(&self, _: &Extensions, msg: Never) -> RpcResult<Op<VoyagerMessage>> {
        match msg {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}
