macro_rules! event {
    (
        $(
            #[tag($tag:literal)]
            pub struct $Struct:ident {
                $(
                    $(#[$meta:meta])*
                    pub $field:ident: $field_ty:ty,
                )+
            }
        )+
    ) => {
        $(
            #[derive(Debug, Clone, PartialEq)]
            pub struct $Struct {
                $(
                    $(#[$meta])*
                    pub $field: $field_ty,
                )+
            }


            impl TryFrom<unionlabs::tendermint::abci::event::Event> for $Struct {
                type Error = TryFromTendermintEventError;

                #[allow(deprecated)]
                fn try_from(value: unionlabs::tendermint::abci::event::Event) -> Result<Self, Self::Error> {
                    if value.ty != $tag {
                        return Err(TryFromTendermintEventError::IncorrectType {
                            expected: $tag,
                            found: value.ty,
                        });
                    }

                    $(
                        let mut $field = None;
                    )+

                    for (idx, attr) in value.attributes.into_iter().enumerate() {
                        match &*attr.key {
                            $(
                                stringify!($field) => match $field {
                                    Some(first_occurrence) => {
                                        return Err(TryFromTendermintEventError::DuplicateField {
                                            key: attr.key,
                                            first_occurrence,
                                            second_occurrence: (idx, attr.value),
                                        })
                                    }
                                    None => $field = Some((idx, attr.value)),
                                },
                            )+
                            _ => return Err(TryFromTendermintEventError::UnknownField(attr.key)),
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
        found: String,
    },
    DuplicateField {
        key: String,
        first_occurrence: (usize, String),
        second_occurrence: (usize, String),
    },
    MissingField(&'static str),
    UnknownField(String),
}

// https://github.com/cosmos/ibc-go/blob/5c7f28634ecf9b6f275bfd5712778fedcf06d80d/docs/ibc/events.md
event! {
    #[tag("create_client")]
    pub struct CreateClient {
        pub client_id: String,
        pub client_type: String,
        pub consensus_height: String,
    }

    #[tag("update_client")]
    pub struct UpdateClient {
        pub client_id: String,
        pub client_type: String,
        #[deprecated = "use consensus_heights"]
        pub consensus_height: String,
        pub consensus_heights: String,
        pub header: String,
    }

    #[tag("client_misbehaviour")]
    pub struct ClientMisbehaviour {
        pub client_id: String,
        pub client_type: String,
        pub consensus_height: String,
    }

    #[tag("submit_evidence")]
    pub struct SubmitEvidence {
        pub evidence_hash: String,
    }

    #[tag("connection_open_init")]
    pub struct ConnectionOpenInit {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        /// NOTE: This will always be empty.
        pub counterparty_connection_id: String,
    }

    #[tag("connection_open_try")]
    pub struct ConnectionOpenTry {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }

    #[tag("connection_open_ack")]
    pub struct ConnectionOpenAck {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }

    #[tag("connection_open_confirm")]
    pub struct ConnectionOpenConfirm {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }

    #[tag("channel_open_init")]
    pub struct ChannelOpenInit {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        /// NOTE: This will always be empty
        pub counterparty_channel_id: String,
        pub connection_id: String,
        pub version: String,
    }

    #[tag("channel_open_try")]
    pub struct ChannelOpenTry {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
        pub version: String,
    }

    #[tag("channel_open_ack")]
    pub struct ChannelOpenAck {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }

    #[tag("channel_open_confirm")]
    pub struct ChannelOpenConfirm {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }

    #[tag("write_acknowledgement")]
    pub struct WriteAcknowledgement {
        #[deprecated = "this is stringified bytes which may not be valid utf8"]
        pub packet_data: String,
        pub packet_data_hex: String,
        pub packet_timeout_height: String,
        pub packet_timeout_timestamp: String,
        pub packet_sequence: String,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        #[deprecated = "this is stringified bytes which may not be valid utf8"]
        pub packet_ack: String,
        pub packet_ack_hex: String,
        #[deprecated = "use connection_id"]
        pub packet_connection: String,
        pub connection_id: String,
    }

    #[tag("recv_packet")]
    pub struct RecvPacket {
        #[deprecated = "this is stringified bytes which may not be valid utf8"]
        pub packet_data: String,
        pub packet_data_hex: String,
        pub packet_timeout_height: String,
        pub packet_timeout_timestamp: String,
        pub packet_sequence: String,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        pub packet_channel_ordering: String,
        #[deprecated = "use connection_id"]
        pub packet_connection: String,
        pub connection_id: String,
    }

    #[tag("send_packet")]
    pub struct SendPacket {
        #[deprecated = "this is stringified bytes which may not be valid utf8"]
        pub packet_data: String,
        pub packet_data_hex: String,
        pub packet_timeout_height: String,
        pub packet_timeout_timestamp: String,
        pub packet_sequence: String,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        pub packet_channel_ordering: String,
        #[deprecated = "use connection_id"]
        pub packet_connection: String,
        pub connection_id: String,
    }

    #[tag("acknowledge_packet")]
    pub struct AcknowledgePacket {
        pub packet_timeout_height: String,
        pub packet_timeout_timestamp: String,
        pub packet_sequence: String,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        pub packet_channel_ordering: String,
        #[deprecated = "use connection_id"]
        pub packet_connection: String,
        pub connection_id: String,
    }

    #[tag("timeout_packet")]
    pub struct TimeoutPacket {
        pub packet_timeout_height: String,
        pub packet_timeout_timestamp: String,
        pub packet_sequence: String,
        pub packet_src_port: String,
        pub packet_src_channel: String,
        pub packet_dst_port: String,
        pub packet_dst_channel: String,
        pub packet_channel_ordering: String,
        pub connection_id: String,
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::tendermint::abci::{event::Event, event_attribute::EventAttribute};

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

        assert_eq!(
            ConnectionOpenConfirm::try_from(Event {
                ty: "really_cool_event".to_string(),
                attributes: vec![],
            }),
            Err(TryFromTendermintEventError::IncorrectType {
                expected: "connection_open_confirm",
                found: "really_cool_event".to_string(),
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
