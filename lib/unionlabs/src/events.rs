use core::str::FromStr;

use crate::{
    ibc::core::{channel::order::Order, client::height::Height},
    id::{ChannelId, ConnectionId},
    EmptyString,
};

macro_rules! event {
    (
        pub enum $Enum:ident$(<$($generics:ident),+>)? {
            $(
                #[event(tag = $tag:literal $(, deprecated($($dep:literal),+)$(,)?)?)]
                $Struct:ident$(<$($struct_generics:ident),*>)? {
                    $(
                        $(#[doc = $doc:literal])*
                        $(#[parse($parse:expr)])?
                        $(#[serde($serde:meta)])?
                        $field:ident: $field_ty:ty,
                    )+
                },
            )+
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
            #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            pub struct $Struct$(<$($struct_generics),+>)? {
                $(
                    $(#[doc = $doc])*
                    $(#[serde($serde)])?
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
event! {
    pub enum IbcEvent<ClientId, ClientType, CounterpartyClientId> {
        #[event(tag = "create_client")]
        CreateClient<ClientId, ClientType> {
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientType::from_str)]
            client_type: ClientType,
            #[parse(Height::from_str)]
            consensus_height: Height,
        },

        #[event(tag = "update_client", deprecated("consensus_height"))]
        UpdateClient<ClientId, ClientType> {
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientType::from_str)]
            client_type: ClientType,
            #[parse(|s: &str| s.split(',').map(Height::from_str).collect::<Result<_, _>>())]
            consensus_heights: Vec<Height>,
            #[parse(hex::decode)]
            #[serde(with = "::serde_utils::hex_string")]
            header: Vec<u8>,
        },

        #[event(tag = "client_misbehaviour")]
        ClientMisbehaviour<ClientId, ClientType> {
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientType::from_str)]
            client_type: ClientType,
            #[parse(Height::from_str)]
            consensus_height: Height,
        },

        #[event(tag = "submit_evidence")]
        SubmitEvidence {
            evidence_hash: String,
        },

        #[event(tag = "connection_open_init")]
        ConnectionOpenInit<ClientId, CounterpartyClientId> {
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(CounterpartyClientId::from_str)]
            counterparty_client_id: CounterpartyClientId,
            #[parse(EmptyString::from_str)]
            counterparty_connection_id: EmptyString,
        },

        #[event(tag = "connection_open_try")]
        ConnectionOpenTry<ClientId, CounterpartyClientId> {
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(CounterpartyClientId::from_str)]
            counterparty_client_id: CounterpartyClientId,
            #[parse(ConnectionId::from_str)]
            counterparty_connection_id: ConnectionId,
        },

        #[event(tag = "connection_open_ack")]
        ConnectionOpenAck<ClientId, CounterpartyClientId> {
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(CounterpartyClientId::from_str)]
            counterparty_client_id: CounterpartyClientId,
            #[parse(ConnectionId::from_str)]
            counterparty_connection_id: ConnectionId,
        },

        #[event(tag = "connection_open_confirm")]
        ConnectionOpenConfirm<ClientId, CounterpartyClientId> {
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(CounterpartyClientId::from_str)]
            counterparty_client_id: CounterpartyClientId,
            #[parse(ConnectionId::from_str)]
            counterparty_connection_id: ConnectionId,
        },

        #[event(tag = "channel_open_init")]
        ChannelOpenInit {
            port_id: String,
            #[parse(ChannelId::from_str)]
            channel_id: ChannelId,
            #[parse(EmptyString::from_str)]
            counterparty_channel_id: EmptyString,
            counterparty_port_id: String,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
            version: String,
        },

        #[event(tag = "channel_open_try")]
        ChannelOpenTry {
            port_id: String,
            #[parse(ChannelId::from_str)]
            channel_id: ChannelId,
            counterparty_port_id: String,
            #[parse(ChannelId::from_str)]
            counterparty_channel_id: ChannelId,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
            version: String,
        },

        #[event(tag = "channel_open_ack")]
        ChannelOpenAck {
            port_id: String,
            #[parse(ChannelId::from_str)]
            channel_id: ChannelId,
            counterparty_port_id: String,
            #[parse(ChannelId::from_str)]
            counterparty_channel_id: ChannelId,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
        },

        #[event(tag = "channel_open_confirm")]
        ChannelOpenConfirm {
            port_id: String,
            #[parse(ChannelId::from_str)]
            channel_id: ChannelId,
            counterparty_port_id: String,
            #[parse(ChannelId::from_str)]
            counterparty_channel_id: ChannelId,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
        },

        #[event(
            tag = "write_acknowledgement",
            deprecated("packet_data", "packet_ack", "packet_connection"),
        )]
        WriteAcknowledgement {
            #[parse(hex::decode)]
            packet_data_hex: Vec<u8>,
            #[parse(Height::from_str)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(u64::from_str)]
            packet_sequence: u64,
            packet_src_port: String,
            #[parse(ChannelId::from_str)]
            packet_src_channel: ChannelId,
            packet_dst_port: String,
            #[parse(ChannelId::from_str)]
            packet_dst_channel: ChannelId,
            #[parse(hex::decode)]
            packet_ack_hex: Vec<u8>,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
        },

        #[event(
            tag = "recv_packet",
            deprecated("packet_data", "packet_connection"),
        )]
        RecvPacket {
            #[parse(hex::decode)]
            #[serde(with = "::serde_utils::hex_string")]
            packet_data_hex: Vec<u8>,
            #[parse(Height::from_str)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(u64::from_str)]
            packet_sequence: u64,
            packet_src_port: String,
            #[parse(ChannelId::from_str)]
            packet_src_channel: ChannelId,
            packet_dst_port: String,
            #[parse(ChannelId::from_str)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
        },


        #[event(
            tag = "send_packet",
            deprecated("packet_data", "packet_connection"),
        )]
        SendPacket {
            #[parse(hex::decode)]
            #[serde(with = "::serde_utils::hex_string")]
            packet_data_hex: Vec<u8>,
            #[parse(Height::from_str)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(u64::from_str)]
            packet_sequence: u64,
            packet_src_port: String,
            #[parse(ChannelId::from_str)]
            packet_src_channel: ChannelId,
            packet_dst_port: String,
            #[parse(ChannelId::from_str)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
        },

        #[event(
            tag = "acknowledge_packet",
            deprecated("packet_connection"),
        )]
        AcknowledgePacket {
            #[parse(Height::from_str)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(u64::from_str)]
            packet_sequence: u64,
            packet_src_port: String,
            #[parse(ChannelId::from_str)]
            packet_src_channel: ChannelId,
            packet_dst_port: String,
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
            #[parse(u64::from_str)]
            packet_sequence: u64,
            packet_src_port: String,
            #[parse(ChannelId::from_str)]
            packet_src_channel: ChannelId,
            packet_dst_port: String,
            #[parse(ChannelId::from_str)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str)]
            connection_id: ConnectionId,
        },
    }
}

#[cfg(test)]
mod tests {
    mod event_conversion {
        use crate::{
            events::{ConnectionOpenConfirm, TryFromTendermintEventError, UpdateClient},
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
                header: vec![0x01],
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
    }
}
