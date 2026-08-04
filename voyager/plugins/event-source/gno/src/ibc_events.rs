use std::{error::Error, str::FromStr};

use ibc_union_spec::{ChannelId, ClientId, ConnectionId, Timestamp};
use serde::{Deserialize, Serialize};
use tracing::warn;
use unionlabs::primitives::{Bytes, H256};
use voyager_sdk::rpc::{RpcError, RpcResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "@type", content = "@value")]
pub enum IbcEvent {
    CreateClient {
        client_id: ClientId,
        client_type: String,
    },

    UpdateClient {
        client_id: ClientId,
        counterparty_height: u64,
    },

    ConnectionOpenInit {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
    },

    ConnectionOpenTry {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
    },

    ConnectionOpenAck {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
    },

    ConnectionOpenConfirm {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
    },

    ChannelOpenInit(ChannelEvent),

    ChannelOpenTry(ChannelEvent),

    ChannelOpenAck(ChannelEvent),

    ChannelOpenConfirm(ChannelEvent),

    PacketSend {
        packet_hash: H256,
        packet_data: Bytes,
        source_channel_id: ChannelId,
        destination_channel_id: ChannelId,
        timeout_timestamp: Timestamp,
    },

    BatchSend {
        packet_hash: H256,
        batch_hash: H256,
        channel_id: ChannelId,
    },

    PacketRecv {
        packet_hash: H256,
        destination_channel_id: ChannelId,
        maker_msg: Bytes,
    },

    PacketAck {
        packet_hash: H256,
        source_channel_id: ChannelId,
        acknowledgement: Bytes,
    },

    WriteAck {
        packet_hash: H256,
        destination_channel_id: ChannelId,
        acknowledgement: Bytes,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelEvent {
    pub port_id: String,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: Option<ChannelId>,
    pub connection_client_id: ClientId,
    pub connection_counterparty_client_id: ClientId,
    pub connection_counterparty_connection_id: ConnectionId,
    pub version: String,
}

impl IbcEvent {
    pub fn is_trivial(&self) -> bool {
        matches!(
            self,
            Self::CreateClient { .. }
                | Self::UpdateClient { .. }
                | Self::PacketRecv { .. }
                // | Self::PacketIntentRecv { .. }
                // | Self::PacketTimeout { .. }
                | Self::PacketAck { .. }
        )
    }

    pub fn from_gno_event(gno_event: gno_rpc::types::event::TmEvent) -> RpcResult<Option<Self>> {
        fn attr<T: FromStr<Err: Error>>(
            attrs: &[gno_rpc::types::EventAttribute],
            ty: &str,
        ) -> RpcResult<T> {
            attrs
                .iter()
                .find_map(|a| (a.key == ty).then(|| a.value.parse()))
                .ok_or_else(|| RpcError::fatal_from_message(format!("key {ty} not found")))?
                .map_err(RpcError::fatal(format!("error parsing value for key {ty}")))
        }

        fn chunked_attr(attrs: &[gno_rpc::types::EventAttribute], ty: &str) -> RpcResult<Bytes> {
            // If no _size field is present, the value was emitted as a single plain attribute.
            let Some(hex_str_len) = attr::<usize>(attrs, &format!("{ty}_size")).ok() else {
                return attr(attrs, ty);
            };

            // Collect raw chunk strings. gno splits the full "0x<hex>" string into
            // 4096-char slices, so only chunk[0] carries the "0x" prefix; subsequent
            // chunks are bare hex continuations.
            let combined: String = (0..)
                .map_while(|i| {
                    attrs
                        .iter()
                        .find_map(|a| (a.key == format!("{ty}[{i}]")).then(|| a.value.clone()))
                })
                .collect();

            if combined.len() != hex_str_len {
                return Err(RpcError::fatal_from_message(format!(
                    "key {ty} incomplete: got {} hex chars, expected {hex_str_len}",
                    combined.len(),
                )));
            }

            combined
                .parse::<Bytes>()
                .map_err(RpcError::fatal(format!("error parsing hex for key {ty}")))
        }

        let attrs = gno_event
            .attrs
            .ok_or_else(|| RpcError::fatal_from_message("no attributes on event"))?;

        let parse_channel_event = |attrs: Vec<gno_rpc::types::EventAttribute>| {
            <RpcResult<_>>::Ok(ChannelEvent {
                port_id: attr(&attrs, "port_id")?,
                channel_id: attr(&attrs, "channel_id")?,
                counterparty_port_id: attr(&attrs, "counterparty_port_id")?,
                // TODO: Fix this once this is no longer emitted on init
                counterparty_channel_id: attr::<ChannelId>(&attrs, "counterparty_channel_id").ok(),
                connection_client_id: attr(&attrs, "connection_client_id")?,
                connection_counterparty_client_id: attr(
                    &attrs,
                    "connection_counterparty_client_id",
                )?,
                connection_counterparty_connection_id: attr(
                    &attrs,
                    "connection_counterparty_connection_id",
                )?,
                version: attr(&attrs, "version")?,
            })
        };

        Ok(Some(match &*gno_event.ty {
            "CreateClient" => IbcEvent::CreateClient {
                client_id: attr(&attrs, "client_id")?,
                client_type: attr(&attrs, "client_type")?,
            },
            "UpdateClient" => IbcEvent::UpdateClient {
                client_id: attr(&attrs, "client_id")?,
                counterparty_height: attr(&attrs, "height")?,
            },
            "ConnectionOpenInit" => IbcEvent::ConnectionOpenInit {
                connection_id: attr(&attrs, "connection_id")?,
                client_id: attr(&attrs, "client_id")?,
                counterparty_client_id: attr(&attrs, "counterparty_client_id")?,
            },
            "ConnectionOpenTry" => IbcEvent::ConnectionOpenTry {
                connection_id: attr(&attrs, "connection_id")?,
                client_id: attr(&attrs, "client_id")?,
                counterparty_client_id: attr(&attrs, "counterparty_client_id")?,
                counterparty_connection_id: attr(&attrs, "counterparty_connection_id")?,
            },
            "ConnectionOpenAck" => IbcEvent::ConnectionOpenAck {
                connection_id: attr(&attrs, "connection_id")?,
                client_id: attr(&attrs, "client_id")?,
                counterparty_client_id: attr(&attrs, "counterparty_client_id")?,
                counterparty_connection_id: attr(&attrs, "counterparty_connection_id")?,
            },
            "ConnectionOpenConfirm" => IbcEvent::ConnectionOpenConfirm {
                connection_id: attr(&attrs, "connection_id")?,
                client_id: attr(&attrs, "client_id")?,
                counterparty_client_id: attr(&attrs, "counterparty_client_id")?,
                counterparty_connection_id: attr(&attrs, "counterparty_connection_id")?,
            },
            "ChannelOpenInit" => IbcEvent::ChannelOpenInit(parse_channel_event(attrs)?),
            "ChannelOpenTry" => IbcEvent::ChannelOpenTry(parse_channel_event(attrs)?),
            "ChannelOpenAck" => IbcEvent::ChannelOpenAck(parse_channel_event(attrs)?),
            "ChannelOpenConfirm" => IbcEvent::ChannelOpenConfirm(parse_channel_event(attrs)?),
            "PacketRecv" => IbcEvent::PacketRecv {
                packet_hash: attr(&attrs, "packet_hash")?,
                destination_channel_id: attr(&attrs, "destination_channel_id")?,
                maker_msg: chunked_attr(&attrs, "maker_msg")?,
            },
            "PacketSend" => IbcEvent::PacketSend {
                packet_hash: attr(&attrs, "packet_hash")?,
                packet_data: chunked_attr(&attrs, "packet_data")?,
                source_channel_id: attr(&attrs, "source_channel_id")?,
                destination_channel_id: attr(&attrs, "destination_channel_id")?,
                timeout_timestamp: attr(&attrs, "timeout_timestamp")?,
            },
            "BatchSend" => IbcEvent::BatchSend {
                packet_hash: attr(&attrs, "packet_hash")?,
                batch_hash: attr(&attrs, "batch_hash")?,
                channel_id: attr(&attrs, "channel_id")?,
            },
            "PacketAck" => IbcEvent::PacketAck {
                packet_hash: attr(&attrs, "packet_hash")?,
                source_channel_id: attr(&attrs, "source_channel_id")?,
                acknowledgement: chunked_attr(&attrs, "acknowledgement")?,
            },
            "WriteAck" => IbcEvent::WriteAck {
                packet_hash: attr(&attrs, "packet_hash")?,
                destination_channel_id: attr(&attrs, "destination_channel_id")?,
                acknowledgement: chunked_attr(&attrs, "acknowledgement")?,
            },
            event => {
                warn!("unknown event: {event}");
                return Ok(None);
            }
        }))
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            IbcEvent::CreateClient { .. } => "create_client",
            IbcEvent::UpdateClient { .. } => "update_client",
            IbcEvent::ConnectionOpenInit { .. } => "connection_open_init",
            IbcEvent::ConnectionOpenTry { .. } => "connection_open_try",
            IbcEvent::ConnectionOpenAck { .. } => "connection_open_ack",
            IbcEvent::ConnectionOpenConfirm { .. } => "connection_open_confirm",
            IbcEvent::ChannelOpenInit { .. } => "channel_open_init",
            IbcEvent::ChannelOpenTry { .. } => "channel_open_try",
            IbcEvent::ChannelOpenAck { .. } => "channel_open_ack",
            IbcEvent::ChannelOpenConfirm(ChannelEvent { .. }) => "channel_open_confirm",
            IbcEvent::PacketRecv { .. } => "recv_packet",
            IbcEvent::PacketSend { .. } => "send_packet",
            IbcEvent::BatchSend { .. } => "batch_send",
            IbcEvent::PacketAck { .. } => "acknowledge_packet",
            IbcEvent::WriteAck { .. } => "write_ack",
        }
    }
}
