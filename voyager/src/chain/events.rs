use std::str::FromStr;

use unionlabs::ibc::core::{channel::order::Order, client::height::Height};

macro_rules! event {
    (
        $(
            #[event(tag = $tag:literal $(, deprecated($($dep:literal),+)$(,)?)?)]
            pub struct $Struct:ident {
                $(
                    $(#[doc = $doc:literal])*
                    $(#[parse($parse:expr)])?
                    pub $field:ident: $field_ty:ty,
                )+
            }
        )+
    ) => {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum IbcEvent {
            $(
                $Struct($Struct),
            )+
        }

        impl IbcEvent {
            pub fn try_from_tendermint_event(
                event: unionlabs::tendermint::abci::event::Event,
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
            pub struct $Struct {
                $(
                    $(#[doc = $doc])*
                    pub $field: $field_ty,
                )+
            }


            impl TryFrom<unionlabs::tendermint::abci::event::Event> for $Struct {
                type Error = TryFromTendermintEventError;

                fn try_from(value: unionlabs::tendermint::abci::event::Event) -> Result<Self, Self::Error> {
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
        found: unionlabs::tendermint::abci::event::Event,
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
    #[event(tag = "create_client")]
    pub struct CreateClient {
        pub client_id: String,
        pub client_type: String,
        #[parse(Height::from_str)]
        pub consensus_height: Height,
    }

    #[event(tag = "update_client", deprecated("consensus_height"))]
    pub struct UpdateClient {
        pub client_id: String,
        pub client_type: String,
        #[parse(|s: &str| s.split(',').map(Height::from_str).collect::<Result<_, _>>())]
        pub consensus_heights: Vec<Height>,
        pub header: String,
    }

    #[event(tag = "client_misbehaviour")]
    pub struct ClientMisbehaviour {
        pub client_id: String,
        pub client_type: String,
        #[parse(Height::from_str)]
        pub consensus_height: Height,
    }

    #[event(tag = "submit_evidence")]
    pub struct SubmitEvidence {
        pub evidence_hash: String,
    }

    #[event(tag = "connection_open_init")]
    pub struct ConnectionOpenInit {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        /// NOTE: This will always be empty.
        pub counterparty_connection_id: String,
    }

    #[event(tag = "connection_open_try")]
    pub struct ConnectionOpenTry {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }

    #[event(tag = "connection_open_ack")]
    pub struct ConnectionOpenAck {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }

    #[event(tag = "connection_open_confirm")]
    pub struct ConnectionOpenConfirm {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }

    #[event(tag = "channel_open_init")]
    pub struct ChannelOpenInit {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub connection_id: String,
    }

    #[event(tag = "channel_open_try")]
    pub struct ChannelOpenTry {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
        pub version: String,
    }

    #[event(tag = "channel_open_ack")]
    pub struct ChannelOpenAck {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }

    #[event(tag = "channel_open_confirm")]
    pub struct ChannelOpenConfirm {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }

    #[event(
        tag = "write_acknowledgement",
        deprecated("packet_data", "packet_ack", "packet_connection"),
    )]
    pub struct WriteAcknowledgement {
        #[parse(hex::decode)]
        pub packet_data_hex: Vec<u8>,
        #[parse(Height::from_str)]
        pub packet_timeout_height: Height,
        #[parse(u64::from_str)]
        pub packet_timeout_timestamp: u64,
        #[parse(u64::from_str)]
        pub packet_sequence: u64,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        #[parse(hex::decode)]
        pub packet_ack_hex: Vec<u8>,
        pub connection_id: String,
    }

    #[event(
        tag = "recv_packet",
        deprecated("packet_data", "packet_connection"),
    )]
    pub struct RecvPacket {
        #[parse(hex::decode)]
        pub packet_data_hex: Vec<u8>,
        #[parse(Height::from_str)]
        pub packet_timeout_height: Height,
        #[parse(u64::from_str)]
        pub packet_timeout_timestamp: u64,
        #[parse(u64::from_str)]
        pub packet_sequence: u64,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        #[parse(Order::from_str)]
        pub packet_channel_ordering: Order,
        pub connection_id: String,
    }


    #[event(
        tag = "send_packet",
        deprecated("packet_data", "packet_connection"),
    )]
    pub struct SendPacket {
        #[parse(hex::decode)]
        pub packet_data_hex: Vec<u8>,
        #[parse(Height::from_str)]
        pub packet_timeout_height: Height,
        #[parse(u64::from_str)]
        pub packet_timeout_timestamp: u64,
        #[parse(u64::from_str)]
        pub packet_sequence: u64,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        #[parse(Order::from_str)]
        pub packet_channel_ordering: Order,
        pub connection_id: String,
    }

    #[event(
        tag = "acknowledge_packet",
        deprecated("packet_connection"),
    )]
    pub struct AcknowledgePacket {
        #[parse(Height::from_str)]
        pub packet_timeout_height: Height,
        #[parse(u64::from_str)]
        pub packet_timeout_timestamp: u64,
        #[parse(u64::from_str)]
        pub packet_sequence: u64,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        #[parse(Order::from_str)]
        pub packet_channel_ordering: Order,
        pub connection_id: String,
    }

    #[event(tag = "timeout_packet")]
    pub struct TimeoutPacket {
        #[parse(Height::from_str)]
        pub packet_timeout_height: Height,
        #[parse(u64::from_str)]
        pub packet_timeout_timestamp: u64,
        #[parse(u64::from_str)]
        pub packet_sequence: u64,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        #[parse(Order::from_str)]
        pub packet_channel_ordering: Order,
        pub connection_id: String,
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        ibc::core::client::height::HeightFromStrError,
        tendermint::abci::{event::Event, event_attribute::EventAttribute},
    };

    use super::*;

    #[test]
    fn event_conversion() {
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
                connection_id: "connection-11".to_string(),
                client_id: "08-wasm-1".to_string(),
                counterparty_client_id: "cometbls-new-0".to_string(),
                counterparty_connection_id: "connection-6".to_string(),
            })
        );

        assert_eq!(
            UpdateClient::try_from(Event {
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
                        value: "1-1".to_string(),
                        index: true,
                    },
                    EventAttribute {
                        key: "consensus_height".to_string(),
                        value: "this can be anything because it's ignored anyways".to_string(),
                        index: true,
                    },
                    EventAttribute {
                        key: "header".to_string(),
                        value: "header".to_string(),
                        index: true,
                    },
                ],
            }),
            Ok(UpdateClient {
                client_id: "client_id".to_string(),
                client_type: "client_type".to_string(),
                consensus_heights: vec![Height {
                    revision_number: 1,
                    revision_height: 1,
                }],
                header: "header".to_string(),
            })
        );

        assert_eq!(
            UpdateClient::try_from(Event {
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

        let event = Event {
            ty: "really_cool_event".to_string(),
            attributes: vec![],
        };

        assert_eq!(
            ConnectionOpenConfirm::try_from(event.clone()),
            Err(TryFromTendermintEventError::IncorrectType {
                expected: "connection_open_confirm",
                found: event,
            })
        );

        assert_eq!(
            ConnectionOpenConfirm::try_from(Event {
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
