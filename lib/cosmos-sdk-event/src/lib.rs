#[doc(hidden)]
pub use cometbft_types;
use cometbft_types::abci::event::Event;

#[macro_export]
macro_rules! event {
    (
        pub enum $Enum:ident {
            $(
                #[event(tag = $tag:literal $(, deprecated($($dep:literal),+)$(,)?)?)]
                $Struct:ident {
                    $(
                        $(#[doc = $doc:literal])*
                        $(#[parse($parse:expr)])?
                        $(#[serde($serde:meta)])?
                        $field:ident: $field_ty:ty
                    ),+$(,)?
                },
            )+
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, enumorph::Enumorph)]
        #[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
        pub enum $Enum {
            $(
                $Struct($Struct),
            )+
        }

        impl $Enum {
            #[must_use]
            pub fn try_from_tendermint_event(
                event: $crate::cometbft_types::abci::event::Event,
            ) -> Option<Result<Self, $crate::TryFromTendermintEventError>> {
                // to silence unused variable warnings on the last repetition of the following block
                let _event = event;

                $(
                    let _event = match $Struct::try_from(_event) {
                        Ok(ok) => return Some(Ok(Self::$Struct(ok))),
                        Err(err) => match err {
                            $crate::TryFromTendermintEventError::IncorrectType { expected: _, found } => found,
                            _ => return Some(Err(err)),
                        },
                    };
                )+

                None
            }
        }

        $(
            #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct $Struct {
                $(
                    $(#[doc = $doc])*
                    $(#[serde($serde)])?
                    pub $field: $field_ty,
                )+
            }


            impl TryFrom<$crate::cometbft_types::abci::event::Event> for $Struct {
                type Error = $crate::TryFromTendermintEventError;

                fn try_from(value: $crate::cometbft_types::abci::event::Event) -> Result<Self, Self::Error> {
                    const DEPRECATED: &[&'static str] = &[$($($dep),+)?];

                    if value.ty != $tag {
                        return Err($crate::TryFromTendermintEventError::IncorrectType {
                            expected: $tag,
                            found: value,
                        });
                    }

                    $(
                        let mut $field = None::<(usize, _)>;
                    )+

                    for (idx, attr) in value.attributes.into_iter().enumerate() {
                        match &*attr.key {
                            $(
                                stringify!($field) => match $field {
                                    Some(first_occurrence) => {
                                        return Err($crate::TryFromTendermintEventError::DuplicateField {
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
                                                    $crate::TryFromTendermintEventError::AttributeValueParse {
                                                        field: stringify!($field),
                                                        error: unionlabs::ErrorReporter(err).to_string(),
                                                    }
                                                })
                                            }))?
                                        )?
                                    ))
                                },
                            )+
                            // TODO(aeryz): this is newly added to cosmos-sdk, until we understand what to do with this, ignore
                            "msg_index" => {}
                            "event_index" => {}
                            "tx_index" => {}
                            "_contract_address" => {}
                            key => {
                                if !DEPRECATED.contains(&key) {
                                    return Err($crate::TryFromTendermintEventError::UnknownAttribute(attr.key))
                                }
                            },
                        }
                    }

                    Ok(Self {
                        $(
                            $field: $field
                                .ok_or($crate::TryFromTendermintEventError::MissingAttribute(stringify!($field)))?
                                .1
                        ),+
                    })
                }
            }
        )+
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromTendermintEventError {
    #[error("incorrect type, expected `{expected}` but found `{}`", found.ty)]
    IncorrectType {
        expected: &'static str,
        found: Event,
    },
    #[error(
        "duplicate field `{key}` (first occurrence index {first_occurrence}, \
        second occurrence index {second_occurrence})"
    )]
    DuplicateField {
        key: String,
        first_occurrence: usize,
        second_occurrence: usize,
    },
    #[error("missing attribute `{0}`")]
    MissingAttribute(&'static str),
    #[error("unknown attribute `{0}`")]
    UnknownAttribute(String),
    #[error("unable to parse value for attribute `{field}`: {error}")]
    AttributeValueParse {
        field: &'static str,
        /// The stringified parse error.
        // NOTE: Basically just `Box<dyn Error + Send + Sync>` with `+ Clone + PartialEq`
        error: String,
    },
}

// #[cfg(test)]
// mod tests {
//     mod event_conversion {
//         use cometbft_types::abci::{event::Event, event_attribute::EventAttribute};
//         use unionlabs::{ibc::core::client::height::Height, id::ConnectionId};

//         use crate::{
//             ConnectionOpenConfirm, CreateClient, TryFromTendermintEventError, UpdateClient,
//         };

//         #[test]
//         fn success() {
//             assert_eq!(
//                 ConnectionOpenConfirm::try_from(Event {
//                     ty: "connection_open_confirm".to_string(),
//                     attributes: vec![
//                         EventAttribute {
//                             key: "connection_id".to_string(),
//                             value: "connection-11".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "client_id".to_string(),
//                             value: "08-wasm-1".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "counterparty_client_id".to_string(),
//                             value: "cometbls-new-0".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "counterparty_connection_id".to_string(),
//                             value: "connection-6".to_string(),
//                             index: true,
//                         },
//                     ],
//                 }),
//                 Ok(ConnectionOpenConfirm {
//                     connection_id: ConnectionId::new(11),
//                     client_id: "08-wasm-1".parse().unwrap(),
//                     counterparty_client_id: "cometbls-new-0".parse().unwrap(),
//                     counterparty_connection_id: ConnectionId::new(6),
//                 })
//             );
//         }

//         #[test]
//         fn deprecated_field() {
//             let attributes = vec![
//                 EventAttribute {
//                     key: "client_id".to_string(),
//                     value: "client_id-1".to_string(),
//                     index: true,
//                 },
//                 EventAttribute {
//                     key: "client_type".to_string(),
//                     value: "client_type".to_string(),
//                     index: true,
//                 },
//                 EventAttribute {
//                     key: "consensus_heights".to_string(),
//                     value: "1-1".to_string(),
//                     index: true,
//                 },
//                 EventAttribute {
//                     key: "header".to_string(),
//                     value: "01".to_string(),
//                     index: true,
//                 },
//             ];

//             let deprecated_attr = EventAttribute {
//                 key: "consensus_height".to_string(),
//                 value: "this can be anything because it's ignored anyways".to_string(),
//                 index: true,
//             };

//             let expected_event = UpdateClient {
//                 client_id: "client_id-1".parse().unwrap(),
//                 client_type: "client_type".to_string(),
//                 consensus_heights: vec![Height::new_with_revision(1, 1)],
//             };

//             assert_eq!(
//                 UpdateClient::try_from(Event {
//                     ty: "update_client".to_string(),
//                     attributes: attributes.clone(),
//                 }),
//                 Ok(expected_event.clone())
//             );

//             assert_eq!(
//                 UpdateClient::try_from(Event {
//                     ty: "update_client".to_string(),
//                     attributes: attributes.into_iter().chain([deprecated_attr]).collect(),
//                 }),
//                 Ok(expected_event)
//             );
//         }

//         #[test]
//         fn parse() {
//             assert_eq!(
//                 UpdateClient::try_from(Event {
//                     ty: "update_client".to_string(),
//                     attributes: vec![
//                         EventAttribute {
//                             key: "client_id".to_string(),
//                             value: "client_id-1".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "client_type".to_string(),
//                             value: "client_type".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "consensus_heights".to_string(),
//                             value: "180cm".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "header".to_string(),
//                             value: "header".to_string(),
//                             index: true,
//                         },
//                     ],
//                 }),
//                 Err(TryFromTendermintEventError::AttributeValueParse {
//                     field: "consensus_heights",
//                     error: "invalid height string".to_owned()
//                 })
//             );
//         }

//         #[test]
//         fn missing_field() {
//             assert_eq!(
//                 ConnectionOpenConfirm::try_from(Event {
//                     ty: "connection_open_confirm".to_string(),
//                     attributes: vec![
//                         EventAttribute {
//                             key: "connection_id".to_string(),
//                             value: "connection-11".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "counterparty_client_id".to_string(),
//                             value: "cometbls-new-0".to_string(),
//                             index: true,
//                         },
//                         EventAttribute {
//                             key: "counterparty_connection_id".to_string(),
//                             value: "connection-6".to_string(),
//                             index: true,
//                         },
//                     ],
//                 }),
//                 Err(TryFromTendermintEventError::MissingAttribute("client_id"))
//             );
//         }

//         #[test]
//         fn incorrect_type() {
//             let event = Event {
//                 ty: "really_cool_event".to_string(),
//                 attributes: vec![],
//             };

//             assert_eq!(
//                 ConnectionOpenConfirm::try_from(event.clone()),
//                 Err(TryFromTendermintEventError::IncorrectType {
//                     expected: "connection_open_confirm",
//                     found: event,
//                 })
//             );
//         }

//         #[test]
//         fn unknown_field() {
//             assert_eq!(
//                 ConnectionOpenConfirm::try_from(Event {
//                     ty: "connection_open_confirm".to_string(),
//                     attributes: vec![EventAttribute {
//                         key: "abracadabra".to_string(),
//                         value: "doesn't matter".to_string(),
//                         index: true,
//                     },],
//                 }),
//                 Err(TryFromTendermintEventError::UnknownAttribute(
//                     "abracadabra".to_string()
//                 ))
//             );
//         }

//         #[test]
//         fn create() {
//             let client_type = "07-tendermint";
//             let client_id = "07-tendermint-0";
//             let consensus_height = "1-88";

//             let create_client_event = Event {
//                 ty: "create_client".to_owned(),
//                 attributes: [
//                     EventAttribute {
//                         key: "client_id".to_owned(),
//                         value: client_id.to_owned(),
//                         index: true,
//                     },
//                     EventAttribute {
//                         key: "client_type".to_owned(),
//                         value: client_type.to_owned(),
//                         index: true,
//                     },
//                     EventAttribute {
//                         key: "consensus_height".to_owned(),
//                         value: consensus_height.to_owned(),
//                         index: true,
//                     },
//                     EventAttribute {
//                         key: "msg_index".to_owned(),
//                         value: "0".to_owned(),
//                         index: true,
//                     },
//                 ]
//                 .to_vec(),
//             };

//             assert_eq!(
//                 CreateClient::try_from(create_client_event).unwrap(),
//                 CreateClient {
//                     client_id: client_id.parse().unwrap(),
//                     client_type: client_type.to_owned(),
//                     consensus_height: consensus_height.parse().unwrap(),
//                 }
//             );
//         }
//     }
// }
