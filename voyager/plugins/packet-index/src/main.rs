use std::collections::VecDeque;

use ibc_union_spec::{
    event::{ChannelMetadata, ConnectionMetadata, FullEvent, PacketMetadata, PacketSend, WriteAck},
    path::{BatchPacketsPath, BatchReceiptsPath, ChannelPath, ConnectionPath, COMMITMENT_MAGIC},
    query::{PacketAckByHash, PacketByHash},
    ChannelId, IbcUnion,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use unionlabs::{ibc::core::client::height::Height, never::Never, primitives::H256};
use voyager_sdk::{
    anyhow, into_value,
    message::{
        data::{ChainEvent, Data, EventProvableHeight},
        VoyagerMessage,
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec, QueryHeight},
    rpc::{types::PluginInfo, PluginServer},
    vm::{data, noop, pass::PassResult, Op},
    DefaultCmd, ExtensionsExt, VoyagerClient,
};

use crate::call::{MakePacketEvent, ModuleCall};

pub mod call {
    use enumorph::Enumorph;
    use ibc_union_spec::ChannelId;
    use serde::{Deserialize, Serialize};
    use unionlabs::primitives::H256;
    use voyager_sdk::primitives::ChainId;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Enumorph)]
    #[serde(
        tag = "@type",
        content = "@value",
        rename_all = "snake_case",
        deny_unknown_fields
    )]
    pub enum ModuleCall {
        MakePacketEvent(MakePacketEvent),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case", deny_unknown_fields)]
    pub struct MakePacketEvent {
        pub chain_id: ChainId,
        pub channel_id: ChannelId,
        pub packet_hash: H256,
    }
}

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(Config {}: Self::Config) -> anyhow::Result<Self> {
        Ok(Module {})
    }

    fn info(Config {}: Self::Config) -> PluginInfo {
        let module = Module {};

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: "null".to_owned(),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        PLUGIN_NAME.to_owned()
    }

    #[instrument(skip_all, fields(%chain_id, %channel_id, %packet_hash))]
    async fn make_packet_event(
        &self,
        voyager_client: &VoyagerClient,
        chain_id: ChainId,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let source_commitment = voyager_client
            .maybe_query_ibc_state(
                chain_id.clone(),
                QueryHeight::Latest,
                BatchPacketsPath {
                    batch_hash: packet_hash,
                },
            )
            .await?;

        match source_commitment.state {
            Some(source_commitment) => {
                info!(%source_commitment);

                if source_commitment == COMMITMENT_MAGIC {
                    info!("packet has not been acknowledged yet");
                } else {
                    info!("packet has already been acknowledged");
                    return Ok(noop());
                }
            }
            None => {
                info!("packet not committed on source chain");
                return Ok(noop());
            }
        }

        let channel = voyager_client
            .query_ibc_state(
                chain_id.clone(),
                QueryHeight::Latest,
                ChannelPath { channel_id },
            )
            .await?;

        info!(
            state = %channel.state,
            connection_id = %channel.connection_id,
            counterparty_channel_id = channel.counterparty_channel_id.map(|id| id.raw()),
            counterparty_port_id = %channel.counterparty_port_id,
            version = %channel.version,
            "channel",
        );

        let connection = voyager_client
            .query_ibc_state(
                chain_id.clone(),
                QueryHeight::Latest,
                ConnectionPath {
                    connection_id: channel.connection_id,
                },
            )
            .await?;

        info!(
            state = %connection.state,
            client_id = %connection.client_id,
            counterparty_client_id = %connection.counterparty_client_id,
            counterparty_connection_id = connection.counterparty_connection_id.map(|id| id.raw()),
            "connection",
        );

        let client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                chain_id.clone(),
                QueryHeight::Latest,
                connection.client_id,
            )
            .await?;

        info!(
            counterparty_chain_id = %client_meta.counterparty_chain_id,
            counterparty_height = %client_meta.counterparty_height,
            "client meta",
        );

        let counterparty_chain_id = client_meta.counterparty_chain_id;

        let destination_commitment = voyager_client
            .maybe_query_ibc_state(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                BatchReceiptsPath {
                    batch_hash: packet_hash,
                },
            )
            .await?;

        let already_received = match destination_commitment.state {
            Some(destination_commitment) => {
                info!(%destination_commitment, "packet has already been received");
                true
            }
            None => {
                info!("packet has not been received");
                false
            }
        };

        let packet_response = voyager_client
            .query(
                chain_id.clone(),
                PacketByHash {
                    channel_id,
                    packet_hash,
                },
            )
            .await?;

        let event = if already_received {
            let ack_response = voyager_client
                .query(
                    counterparty_chain_id.clone(),
                    PacketAckByHash {
                        channel_id: channel.counterparty_channel_id.unwrap(),
                        packet_hash,
                    },
                )
                .await?;

            info!(
                ack = %ack_response.ack,
                "ack response",
            );

            let client_info = voyager_client
                .client_info::<IbcUnion>(
                    counterparty_chain_id.clone(),
                    connection.counterparty_client_id,
                )
                .await?;

            info!(
                client_type = %client_info.client_type,
                ibc_interface = %client_info.ibc_interface,
                "client info",
            );

            ChainEvent {
                chain_id: counterparty_chain_id,
                client_info,
                counterparty_chain_id: chain_id,
                tx_hash: ack_response.tx_hash,
                provable_height: EventProvableHeight::Min(Height::new(
                    ack_response.provable_height,
                )),
                ibc_spec_id: IbcUnion::ID,
                event: into_value(FullEvent::WriteAck(WriteAck {
                    acknowledgement: ack_response.ack,
                    packet_data: packet_response.packet.data,
                    packet: PacketMetadata {
                        destination_channel: ChannelMetadata {
                            channel_id: channel.counterparty_channel_id.unwrap(),
                            version: channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: connection.counterparty_client_id,
                                connection_id: connection.counterparty_connection_id.unwrap(),
                            },
                        },
                        source_channel: ChannelMetadata {
                            channel_id,
                            version: channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: connection.client_id,
                                connection_id: channel.connection_id,
                            },
                        },
                        timeout_timestamp: packet_response.packet.timeout_timestamp,
                    },
                })),
            }
        } else {
            let client_info = voyager_client
                .client_info::<IbcUnion>(chain_id.clone(), connection.client_id)
                .await?;

            info!(
                client_type = %client_info.client_type,
                ibc_interface = %client_info.ibc_interface,
                "client info",
            );

            ChainEvent {
                chain_id,
                client_info,
                counterparty_chain_id,
                tx_hash: packet_response.tx_hash,
                provable_height: EventProvableHeight::Min(Height::new(
                    packet_response.provable_height,
                )),
                ibc_spec_id: IbcUnion::ID,
                event: into_value(FullEvent::PacketSend(PacketSend {
                    packet_data: packet_response.packet.data,
                    packet: PacketMetadata {
                        source_channel: ChannelMetadata {
                            channel_id,
                            version: channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: connection.client_id,
                                connection_id: channel.connection_id,
                            },
                        },
                        destination_channel: ChannelMetadata {
                            channel_id: channel.counterparty_channel_id.unwrap(),
                            version: channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: connection.counterparty_client_id,
                                connection_id: connection.counterparty_connection_id.unwrap(),
                            },
                        },
                        timeout_timestamp: packet_response.packet.timeout_timestamp,
                    },
                })),
            }
        };

        Ok(data(event))
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all)]
    async fn run_pass(
        &self,
        _: &Extensions,
        _: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult::default())
    }

    #[instrument(skip_all)]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::MakePacketEvent(MakePacketEvent {
                chain_id,
                channel_id,
                packet_hash,
            }) => {
                self.make_packet_event(e.voyager_client()?, chain_id, channel_id, packet_hash)
                    .await
            }
        }
    }

    #[instrument]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}
