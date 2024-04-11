use core::{num::NonZeroU64, str::FromStr};

use macros::apply;

use crate::{
    ibc::core::{channel::order::Order, client::height::Height},
    id::{ChannelId, ConnectionId, PortId},
};

macro_rules! event {
    (
        pub enum $Enum:ident$(<$($generics:ident),+>)? {
            $(
                #[event($(generics($($struct_generics:ident),*),)? tag = $tag:literal $(, deprecated($($dep:literal),+)$(,)?)?)]
                $Struct:ident {
                    $(
                        $(#[doc = $doc:literal])*
                        $(#[parse($parse:expr)])?
                        $(#[serde($serde:meta)])?
                        $(#[debug($debug:meta)])?
                        $field:ident: $field_ty:ty
                    ),+$(,)?
                },
            )+
        }
    ) => {
        #[derive(::macros::Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
        #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
        pub enum $Enum$(<$($generics),*>)? {
            $(
                $Struct($Struct$(<$($struct_generics),+>)?),
            )+
        }

        impl$(<$($generics),+>)? IbcEvent$(<$($generics),+>)?
        where
            $(
                $(
                    $generics: FromStr,
                    <$generics as FromStr>::Err: std::error::Error,
                )+
            )?
        {
            #[must_use]
            pub fn try_from_tendermint_event(
                event: crate::tendermint::abci::event::Event,
            ) -> Option<Result<Self, TryFromTendermintEventError>> {
                // to silence unused variable warnings on the last repetition of the following block
                let _event = event;

                $(
                    let _event = match $Struct::try_from(_event) {
                        Ok(ok) => return Some(Ok(Self::$Struct(ok))),
                        Err(err) => match err {
                            TryFromTendermintEventError::IncorrectType { expected: _, found } => found,
                            _ => return Some(Err(err)),
                        },
                    };
                )+

                None
            }
        }

        $(
            #[derive(::macros::Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
            pub struct $Struct$(<$($struct_generics),+>)? {
                $(
                    $(#[doc = $doc])*
                    $(#[serde($serde)])?
                    $(#[debug($debug)])?
                    pub $field: $field_ty,
                )+
            }


            impl$(<$($struct_generics),+>)? TryFrom<crate::tendermint::abci::event::Event> for $Struct$(<$($struct_generics),+>)?
            where
                $(
                    $(
                        $struct_generics: FromStr,
                        <$struct_generics as FromStr>::Err: std::error::Error,
                    )+
                )?
            {
                type Error = TryFromTendermintEventError;

                fn try_from(value: crate::tendermint::abci::event::Event) -> Result<Self, Self::Error> {
                    const DEPRECATED: &[&'static str] = &[$($($dep),+)?];

                    if value.ty != $tag {
                        return Err(TryFromTendermintEventError::IncorrectType {
                            expected: $tag,
                            found: value,
                        });
                    }

                    $(
                        let mut $field = None::<(usize, _)>;
                        // let mut $field = None;
                    )+

                    for (idx, attr) in value.attributes.into_iter().enumerate() {
                        match &*attr.key {
                            $(
                                stringify!($field) => match $field {
                                    Some(first_occurrence) => {
                                        return Err(TryFromTendermintEventError::DuplicateField {
                                            key: attr.key,
                                            first_occurrence: first_occurrence.0,
                                            second_occurrence: idx,
                                        })
                                    }
                                    None => $field = Some((
                                        idx,
                                        (Ok(attr.value)$(
                                            .and_then(|value: String| {
                                                #[allow(clippy::redundant_closure_call)]
                                                (($parse)(&value))
                                                .map_err(|err| {
                                                    TryFromTendermintEventError::Parse {
                                                        field: stringify!($field),
                                                        error: err.to_string(),
                                                    }
                                                })
                                            }))?
                                        )?
                                    ))
                                },
                            )+
                            // TODO(aeryz): this is newly added to cosmos-sdk, until we understand what to do with this, ignore
                            "msg_index" => {}
                            key => {
                                if !DEPRECATED.contains(&key) {
                                    return Err(TryFromTendermintEventError::UnknownField(attr.key))
                                }
                            },
                        }
                    }

                    Ok(Self {
                        $(
                            $field: $field
                                .ok_or(TryFromTendermintEventError::MissingField(stringify!($field)))?
                                .1
                        ),+
                    })
                }
            }
        )+
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryFromTendermintEventError {
    IncorrectType {
        expected: &'static str,
        found: crate::tendermint::abci::event::Event,
    },
    DuplicateField {
        key: String,
        first_occurrence: usize,
        second_occurrence: usize,
    },
    MissingField(&'static str),
    UnknownField(String),
    Parse {
        field: &'static str,
        /// The stringified parse error.
        // NOTE: Basically just `Box<dyn Error + Send + Sync>` with `+ Clone + PartialEq`
        error: String,
    },
}

// https://github.com/cosmos/ibc-go/blob/5c7f28634ecf9b6f275bfd5712778fedcf06d80d/docs/ibc/events.md
#[apply(event)]
pub enum IbcEvent<ClientId, ClientType, CounterpartyClientId> {
    #[event(generics(ClientId, ClientType), tag = "create_client")]
    CreateClient {
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(ClientType::from_str)]
        client_type: ClientType,
        #[parse(Height::from_str)]
        consensus_height: Height,
    },

    #[event(
        generics(ClientId, ClientType),
        tag = "update_client",
        deprecated("consensus_height", "header")
    )]
    UpdateClient {
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(ClientType::from_str)]
        client_type: ClientType,
        #[parse(|s: &str| s.split(',').map(Height::from_str).collect::<Result<_, _>>())]
        consensus_heights: Vec<Height>,
    },

    #[event(generics(ClientId, ClientType), tag = "client_misbehaviour")]
    ClientMisbehaviour {
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(ClientType::from_str)]
        client_type: ClientType,
        #[parse(Height::from_str)]
        consensus_height: Height,
    },

    #[event(tag = "submit_evidence")]
    SubmitEvidence { evidence_hash: String },

    #[event(
        generics(ClientId, CounterpartyClientId),
        tag = "connection_open_init",
        deprecated("counterparty_connection_id")
    )]
    ConnectionOpenInit {
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(CounterpartyClientId::from_str)]
        counterparty_client_id: CounterpartyClientId,
    },

    #[event(generics(ClientId, CounterpartyClientId), tag = "connection_open_try")]
    ConnectionOpenTry {
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(CounterpartyClientId::from_str)]
        counterparty_client_id: CounterpartyClientId,
        #[parse(ConnectionId::from_str)]
        counterparty_connection_id: ConnectionId,
    },

    #[event(generics(ClientId, CounterpartyClientId), tag = "connection_open_ack")]
    ConnectionOpenAck {
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(CounterpartyClientId::from_str)]
        counterparty_client_id: CounterpartyClientId,
        #[parse(ConnectionId::from_str)]
        counterparty_connection_id: ConnectionId,
    },

    #[event(
        generics(ClientId, CounterpartyClientId),
        tag = "connection_open_confirm"
    )]
    ConnectionOpenConfirm {
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
        #[parse(ClientId::from_str)]
        client_id: ClientId,
        #[parse(CounterpartyClientId::from_str)]
        counterparty_client_id: CounterpartyClientId,
        #[parse(ConnectionId::from_str)]
        counterparty_connection_id: ConnectionId,
    },

    #[event(tag = "channel_open_init", deprecated("counterparty_channel_id"))]
    ChannelOpenInit {
        #[parse(PortId::from_str)]
        port_id: PortId,
        #[parse(ChannelId::from_str)]
        channel_id: ChannelId,
        #[parse(PortId::from_str)]
        counterparty_port_id: PortId,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
        version: String,
    },

    #[event(tag = "channel_open_try")]
    ChannelOpenTry {
        #[parse(PortId::from_str)]
        port_id: PortId,
        #[parse(ChannelId::from_str)]
        channel_id: ChannelId,
        #[parse(PortId::from_str)]
        counterparty_port_id: PortId,
        #[parse(ChannelId::from_str)]
        counterparty_channel_id: ChannelId,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
        version: String,
    },

    #[event(tag = "channel_open_ack")]
    ChannelOpenAck {
        #[parse(PortId::from_str)]
        port_id: PortId,
        #[parse(ChannelId::from_str)]
        channel_id: ChannelId,
        #[parse(PortId::from_str)]
        counterparty_port_id: PortId,
        #[parse(ChannelId::from_str)]
        counterparty_channel_id: ChannelId,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },

    #[event(tag = "channel_open_confirm")]
    ChannelOpenConfirm {
        #[parse(PortId::from_str)]
        port_id: PortId,
        #[parse(ChannelId::from_str)]
        channel_id: ChannelId,
        #[parse(PortId::from_str)]
        counterparty_port_id: PortId,
        #[parse(ChannelId::from_str)]
        counterparty_channel_id: ChannelId,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },

    #[event(
        tag = "write_acknowledgement",
        deprecated("packet_data", "packet_ack", "packet_connection")
    )]
    WriteAcknowledgement {
        #[parse(hex::decode)]
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        packet_data_hex: Vec<u8>,
        #[parse(Height::from_str)]
        packet_timeout_height: Height,
        #[parse(u64::from_str)]
        packet_timeout_timestamp: u64,
        #[parse(NonZeroU64::from_str)]
        packet_sequence: NonZeroU64,
        #[parse(PortId::from_str)]
        packet_src_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_src_channel: ChannelId,
        #[parse(PortId::from_str)]
        packet_dst_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_dst_channel: ChannelId,
        #[parse(hex::decode)]
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        packet_ack_hex: Vec<u8>,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },

    #[event(tag = "recv_packet", deprecated("packet_data", "packet_connection"))]
    RecvPacket {
        #[parse(hex::decode)]
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        packet_data_hex: Vec<u8>,
        #[parse(Height::from_str)]
        packet_timeout_height: Height,
        #[parse(u64::from_str)]
        packet_timeout_timestamp: u64,
        #[parse(NonZeroU64::from_str)]
        packet_sequence: NonZeroU64,
        #[parse(PortId::from_str)]
        packet_src_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_src_channel: ChannelId,
        #[parse(PortId::from_str)]
        packet_dst_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_dst_channel: ChannelId,
        #[parse(Order::from_str)]
        packet_channel_ordering: Order,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },

    #[event(tag = "send_packet", deprecated("packet_data", "packet_connection"))]
    SendPacket {
        #[parse(hex::decode)]
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        packet_data_hex: Vec<u8>,
        #[parse(Height::from_str)]
        // TODO: Make this generic height instead of concrete
        packet_timeout_height: Height,
        #[parse(u64::from_str)]
        packet_timeout_timestamp: u64,
        #[parse(NonZeroU64::from_str)]
        packet_sequence: NonZeroU64,
        #[parse(PortId::from_str)]
        packet_src_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_src_channel: ChannelId,
        #[parse(PortId::from_str)]
        packet_dst_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_dst_channel: ChannelId,
        #[parse(Order::from_str)]
        packet_channel_ordering: Order,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },

    #[event(tag = "acknowledge_packet", deprecated("packet_connection"))]
    AcknowledgePacket {
        #[parse(Height::from_str)]
        packet_timeout_height: Height,
        #[parse(u64::from_str)]
        packet_timeout_timestamp: u64,
        #[parse(NonZeroU64::from_str)]
        packet_sequence: NonZeroU64,
        #[parse(PortId::from_str)]
        packet_src_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_src_channel: ChannelId,
        #[parse(PortId::from_str)]
        packet_dst_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_dst_channel: ChannelId,
        #[parse(Order::from_str)]
        packet_channel_ordering: Order,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },

    #[event(tag = "timeout_packet")]
    TimeoutPacket {
        #[parse(Height::from_str)]
        packet_timeout_height: Height,
        #[parse(u64::from_str)]
        packet_timeout_timestamp: u64,
        #[parse(NonZeroU64::from_str)]
        packet_sequence: NonZeroU64,
        #[parse(PortId::from_str)]
        packet_src_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_src_channel: ChannelId,
        #[parse(PortId::from_str)]
        packet_dst_port: PortId,
        #[parse(ChannelId::from_str)]
        packet_dst_channel: ChannelId,
        #[parse(Order::from_str)]
        packet_channel_ordering: Order,
        #[parse(ConnectionId::from_str)]
        connection_id: ConnectionId,
    },
}

impl<ClientId, ClientType, CounterpartyClientId>
    IbcEvent<ClientId, ClientType, CounterpartyClientId>
{
    pub fn name(&self) -> &'static str {
        match self {
            IbcEvent::CreateClient(_) => "CreateClient",
            IbcEvent::UpdateClient(_) => "UpdateClient",
            IbcEvent::ClientMisbehaviour(_) => "ClientMisbehaviour",
            IbcEvent::SubmitEvidence(_) => "SubmitEvidence",
            IbcEvent::ConnectionOpenInit(_) => "ConnectionOpenInit",
            IbcEvent::ConnectionOpenTry(_) => "ConnectionOpenTry",
            IbcEvent::ConnectionOpenAck(_) => "ConnectionOpenAck",
            IbcEvent::ConnectionOpenConfirm(_) => "ConnectionOpenConfirm",
            IbcEvent::ChannelOpenInit(_) => "ChannelOpenInit",
            IbcEvent::ChannelOpenTry(_) => "ChannelOpenTry",
            IbcEvent::ChannelOpenAck(_) => "ChannelOpenAck",
            IbcEvent::ChannelOpenConfirm(_) => "ChannelOpenConfirm",
            IbcEvent::WriteAcknowledgement(_) => "WriteAcknowledgement",
            IbcEvent::RecvPacket(_) => "RecvPacket",
            IbcEvent::SendPacket(_) => "SendPacket",
            IbcEvent::AcknowledgePacket(_) => "AcknowledgePacket",
            IbcEvent::TimeoutPacket(_) => "TimeoutPacket",
        }
    }
}

#[cfg(test)]
mod tests {
    mod event_conversion {
        use crate::{
            events::{
                ConnectionOpenConfirm, CreateClient, TryFromTendermintEventError, UpdateClient,
            },
            ibc::core::client::height::{Height, HeightFromStrError},
            tendermint::abci::{event::Event, event_attribute::EventAttribute},
        };

        #[test]
        fn success() {
            assert_eq!(
                ConnectionOpenConfirm::try_from(Event {
                    ty: "connection_open_confirm".to_string(),
                    attributes: vec![
                        EventAttribute {
                            key: "connection_id".to_string(),
                            value: "connection-11".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "client_id".to_string(),
                            value: "08-wasm-1".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "counterparty_client_id".to_string(),
                            value: "cometbls-new-0".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "counterparty_connection_id".to_string(),
                            value: "connection-6".to_string(),
                            index: true,
                        },
                    ],
                }),
                Ok(ConnectionOpenConfirm {
                    connection_id: "connection-11".parse().unwrap(),
                    client_id: "08-wasm-1".to_string(),
                    counterparty_client_id: "cometbls-new-0".to_string(),
                    counterparty_connection_id: "connection-6".parse().unwrap(),
                })
            );
        }

        #[test]
        fn deprecated_field() {
            let attributes = vec![
                EventAttribute {
                    key: "client_id".to_string(),
                    value: "client_id".to_string(),
                    index: true,
                },
                EventAttribute {
                    key: "client_type".to_string(),
                    value: "client_type".to_string(),
                    index: true,
                },
                EventAttribute {
                    key: "consensus_heights".to_string(),
                    value: "1-1".to_string(),
                    index: true,
                },
                EventAttribute {
                    key: "header".to_string(),
                    value: "01".to_string(),
                    index: true,
                },
            ];

            let deprecated_attr = EventAttribute {
                key: "consensus_height".to_string(),
                value: "this can be anything because it's ignored anyways".to_string(),
                index: true,
            };

            let expected_event = UpdateClient {
                client_id: "client_id".to_string(),
                client_type: "client_type".to_string(),
                consensus_heights: vec![Height {
                    revision_number: 1,
                    revision_height: 1,
                }],
            };

            assert_eq!(
                UpdateClient::try_from(Event {
                    ty: "update_client".to_string(),
                    attributes: attributes.clone(),
                }),
                Ok(expected_event.clone())
            );

            assert_eq!(
                UpdateClient::try_from(Event {
                    ty: "update_client".to_string(),
                    attributes: attributes.into_iter().chain([deprecated_attr]).collect(),
                }),
                Ok(expected_event)
            );
        }

        #[test]
        fn parse() {
            assert_eq!(
                UpdateClient::<String, String>::try_from(Event {
                    ty: "update_client".to_string(),
                    attributes: vec![
                        EventAttribute {
                            key: "client_id".to_string(),
                            value: "client_id".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "client_type".to_string(),
                            value: "client_type".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "consensus_heights".to_string(),
                            value: "180cm".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "header".to_string(),
                            value: "header".to_string(),
                            index: true,
                        },
                    ],
                }),
                Err(TryFromTendermintEventError::Parse {
                    field: "consensus_heights",
                    error: HeightFromStrError::Invalid.to_string(),
                })
            );
        }

        #[test]
        fn missing_field() {
            assert_eq!(
                ConnectionOpenConfirm::<String, String>::try_from(Event {
                    ty: "connection_open_confirm".to_string(),
                    attributes: vec![
                        EventAttribute {
                            key: "connection_id".to_string(),
                            value: "connection-11".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "counterparty_client_id".to_string(),
                            value: "cometbls-new-0".to_string(),
                            index: true,
                        },
                        EventAttribute {
                            key: "counterparty_connection_id".to_string(),
                            value: "connection-6".to_string(),
                            index: true,
                        },
                    ],
                }),
                Err(TryFromTendermintEventError::MissingField("client_id"))
            );
        }

        #[test]
        fn incorrect_type() {
            let event = Event {
                ty: "really_cool_event".to_string(),
                attributes: vec![],
            };

            assert_eq!(
                ConnectionOpenConfirm::<String, String>::try_from(event.clone()),
                Err(TryFromTendermintEventError::IncorrectType {
                    expected: "connection_open_confirm",
                    found: event,
                })
            );
        }

        #[test]
        fn unknown_field() {
            assert_eq!(
                ConnectionOpenConfirm::<String, String>::try_from(Event {
                    ty: "connection_open_confirm".to_string(),
                    attributes: vec![EventAttribute {
                        key: "abracadabra".to_string(),
                        value: "doesn't matter".to_string(),
                        index: true,
                    },],
                }),
                Err(TryFromTendermintEventError::UnknownField(
                    "abracadabra".to_string()
                ))
            );
        }

        #[test]
        fn create() {
            let client_type = "07-tendermint";
            let client_id = "07-tendermint-0";
            let consensus_height = "1-88";

            let create_client_event = Event {
                ty: "create_client".to_owned(),
                attributes: [
                    EventAttribute {
                        key: "client_id".to_owned(),
                        value: client_id.to_owned(),
                        index: true,
                    },
                    EventAttribute {
                        key: "client_type".to_owned(),
                        value: client_type.to_owned(),
                        index: true,
                    },
                    EventAttribute {
                        key: "consensus_height".to_owned(),
                        value: consensus_height.to_owned(),
                        index: true,
                    },
                    EventAttribute {
                        key: "msg_index".to_owned(),
                        value: "0".to_owned(),
                        index: true,
                    },
                ]
                .to_vec(),
            };

            assert_eq!(
                CreateClient::<String, String>::try_from(create_client_event).unwrap(),
                CreateClient {
                    client_id: client_id.to_owned(),
                    client_type: client_type.to_owned(),
                    consensus_height: consensus_height.parse().unwrap(),
                }
            );
        }
    }
}
