#![feature(extract_if)]

use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use cometbft_types::abci::event_attribute::EventAttribute;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use unionlabs::{bech32::Bech32, primitives::H256};

/// Wrapper around a strongly-typed enum of on-chain events, containing well-known attributes that
/// are added automatically by various modules in the Cosmos SDK.
///
/// The event enum must be an [adjacently tagged], with `tag = "type", content = "attributes"`.
///
/// # Example
///
/// ```rust
/// use cosmos_sdk_event::CosmosSdkEvent;
/// use cometbft_types::abci::{event::Event, event_attribute::EventAttribute};
///
/// #[derive(serde::Deserialize)]
/// #[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
/// pub enum MyEvent {
///     #[serde(rename = "wasm-update_client")]
///     WasmUpdateClient(WasmUpdateClient),
/// }
///
/// #[derive(serde::Deserialize)]
/// pub struct WasmUpdateClient {
///     #[serde(with = "::serde_utils::string")]
///     pub client_id: u32,
///     #[serde(with = "::serde_utils::string")]
///     pub counterparty_height: u64,
/// }
///
/// let raw_event = Event {
///     ty: "wasm-update_client".to_owned(),
///     attributes: [
///         EventAttribute {
///             key: "_contract_address".to_owned(),
///             value: "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme".to_owned(),
///             index: true,
///         },
///         EventAttribute {
///             key: "client_id".to_owned(),
///             value: "7".to_owned(),
///             index: true,
///         },
///         EventAttribute {
///             key: "counterparty_height".to_owned(),
///             value: "7340414".to_owned(),
///             index: true,
///         },
///         EventAttribute {
///             key: "event_index".to_owned(),
///             value: "2".to_owned(),
///             index: true,
///         },
///         EventAttribute {
///             key: "msg_index".to_owned(),
///             value: "0".to_owned(),
///             index: true,
///         },
///         EventAttribute {
///             key: "tx_index".to_owned(),
///             value: "0".to_owned(),
///             index: false,
///         }
///     ].to_vec()
/// };
///
/// let event = CosmosSdkEvent::<MyEvent>::new(raw_event).unwrap();
///
/// assert!(matches!(event.event, MyEvent::WasmUpdateClient(..)));
/// ```
///
/// [adjacently tagged]: https://serde.rs/enum-representations.html#adjacently-tagged
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CosmosSdkEvent<T> {
    pub msg_index: Option<u32>,
    pub event_index: Option<u32>,
    pub tx_index: Option<u32>,
    /// If this event was emitted by a cosmwasm contract, `_contract_address` is added as an
    /// attribute, with the value corresponding to the address that emitted the event.
    pub contract_address: Option<Bech32<H256>>,
    pub event: T,
}

impl<T: DeserializeOwned> CosmosSdkEvent<T> {
    pub fn new(mut raw: cometbft_types::abci::event::Event) -> Result<Self, Error> {
        Ok(Self {
            msg_index: pull_attr(&mut raw.attributes, "msg_index")?,
            event_index: pull_attr(&mut raw.attributes, "event_index")?,
            tx_index: pull_attr(&mut raw.attributes, "tx_index")?,
            contract_address: pull_attr(&mut raw.attributes, "_contract_address")?,
            event: {
                let map = raw
                    .attributes
                    .into_iter()
                    .map(|attr| (attr.key, attr.value))
                    .collect::<BTreeMap<String, String>>();

                serde_json::from_value(serde_json::json!({
                    "type": raw.ty,
                    "attributes": map,
                }))?
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error parsing well-known key `{key}`: {error}")]
    WellKnownKeyParse { key: &'static str, error: String },
    #[error("duplicate well-known key `{key}`")]
    DuplicateWellKnownKey { key: &'static str },
    #[error("error deserializing event")]
    Deserialize(#[from] serde_json::Error),
}

fn pull_attr<T: FromStr<Err: Display>>(
    attrs: &mut Vec<EventAttribute>,
    key: &'static str,
) -> Result<Option<T>, Error> {
    let mut found = attrs.extract_if(|attr| attr.key == key).peekable();

    match found.next() {
        Some(attr) => {
            if found.peek().is_none() {
                attr.value
                    .parse()
                    .map_err(|e: T::Err| Error::WellKnownKeyParse {
                        key,
                        error: e.to_string(),
                    })
                    .map(Some)
            } else {
                Err(Error::DuplicateWellKnownKey { key })
            }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use cometbft_types::abci::{event::Event, event_attribute::EventAttribute};
    use serde::{Deserialize, Serialize};

    use crate::CosmosSdkEvent;

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
    pub enum MyEvent {
        ConnectionOpenConfirm(ConnectionOpenConfirm),
    }

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct ConnectionOpenConfirm {
        #[serde(with = "::serde_utils::string")]
        pub connection_id: u32,
        #[serde(with = "::serde_utils::string")]
        pub client_id: u32,
        #[serde(with = "::serde_utils::string")]
        pub counterparty_client_id: u32,
        #[serde(with = "::serde_utils::string")]
        pub counterparty_connection_id: u32,
    }

    #[test]
    fn success() {
        assert_eq!(
            CosmosSdkEvent::<MyEvent>::new(Event {
                ty: "connection_open_confirm".to_string(),
                attributes: vec![
                    EventAttribute {
                        index: true,
                        key: "_contract_address".to_string(),
                        value: "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                            .to_string(),
                    },
                    EventAttribute {
                        index: true,
                        key: "event_index".to_string(),
                        value: "3".to_string(),
                    },
                    EventAttribute {
                        index: true,
                        key: "msg_index".to_string(),
                        value: "0".to_string(),
                    },
                    EventAttribute {
                        index: false,
                        key: "tx_index".to_string(),
                        value: "0".to_string(),
                    },
                    EventAttribute {
                        key: "connection_id".to_string(),
                        value: "11".to_string(),
                        index: true,
                    },
                    EventAttribute {
                        key: "client_id".to_string(),
                        value: "1".to_string(),
                        index: true,
                    },
                    EventAttribute {
                        key: "counterparty_client_id".to_string(),
                        value: "0".to_string(),
                        index: true,
                    },
                    EventAttribute {
                        key: "counterparty_connection_id".to_string(),
                        value: "6".to_string(),
                        index: true,
                    },
                ],
            })
            .unwrap(),
            CosmosSdkEvent {
                msg_index: Some(0),
                event_index: Some(3),
                tx_index: Some(0),
                contract_address: Some(
                    "union17e93ukhcyesrvu72cgfvamdhyracghrx4f7ww89rqjg944ntdegscxepme"
                        .parse()
                        .unwrap()
                ),
                event: MyEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                    connection_id: 11,
                    client_id: 1,
                    counterparty_client_id: 0,
                    counterparty_connection_id: 6,
                })
            }
        );
    }

    // #[test]
    // fn deprecated_field() {
    //     let attributes = vec![
    //         EventAttribute {
    //             key: "client_id".to_string(),
    //             value: "client_id-1".to_string(),
    //             index: true,
    //         },
    //         EventAttribute {
    //             key: "client_type".to_string(),
    //             value: "client_type".to_string(),
    //             index: true,
    //         },
    //         EventAttribute {
    //             key: "consensus_heights".to_string(),
    //             value: "1-1".to_string(),
    //             index: true,
    //         },
    //         EventAttribute {
    //             key: "header".to_string(),
    //             value: "01".to_string(),
    //             index: true,
    //         },
    //     ];

    //     let deprecated_attr = EventAttribute {
    //         key: "consensus_height".to_string(),
    //         value: "this can be anything because it's ignored anyways".to_string(),
    //         index: true,
    //     };

    //     let expected_event = UpdateClient {
    //         client_id: "client_id-1".parse().unwrap(),
    //         client_type: "client_type".to_string(),
    //         consensus_heights: vec![Height::new_with_revision(1, 1)],
    //     };

    //     assert_eq!(
    //         UpdateClient::try_from(Event {
    //             ty: "update_client".to_string(),
    //             attributes: attributes.clone(),
    //         }),
    //         Ok(expected_event.clone())
    //     );

    //     assert_eq!(
    //         UpdateClient::try_from(Event {
    //             ty: "update_client".to_string(),
    //             attributes: attributes.into_iter().chain([deprecated_attr]).collect(),
    //         }),
    //         Ok(expected_event)
    //     );
    // }

    // #[test]
    // fn parse() {
    //     assert_eq!(
    //         UpdateClient::try_from(Event {
    //             ty: "update_client".to_string(),
    //             attributes: vec![
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
    //                     value: "180cm".to_string(),
    //                     index: true,
    //                 },
    //                 EventAttribute {
    //                     key: "header".to_string(),
    //                     value: "header".to_string(),
    //                     index: true,
    //                 },
    //             ],
    //         }),
    //         Err(TryFromTendermintEventError::AttributeValueParse {
    //             field: "consensus_heights",
    //             error: "invalid height string".to_owned()
    //         })
    //     );
    // }

    // #[test]
    // fn missing_field() {
    //     assert_eq!(
    //         ConnectionOpenConfirm::try_from(Event {
    //             ty: "connection_open_confirm".to_string(),
    //             attributes: vec![
    //                 EventAttribute {
    //                     key: "connection_id".to_string(),
    //                     value: "connection-11".to_string(),
    //                     index: true,
    //                 },
    //                 EventAttribute {
    //                     key: "counterparty_client_id".to_string(),
    //                     value: "cometbls-new-0".to_string(),
    //                     index: true,
    //                 },
    //                 EventAttribute {
    //                     key: "counterparty_connection_id".to_string(),
    //                     value: "connection-6".to_string(),
    //                     index: true,
    //                 },
    //             ],
    //         }),
    //         Err(TryFromTendermintEventError::MissingAttribute("client_id"))
    //     );
    // }

    // #[test]
    // fn incorrect_type() {
    //     let event = Event {
    //         ty: "really_cool_event".to_string(),
    //         attributes: vec![],
    //     };

    //     assert_eq!(
    //         ConnectionOpenConfirm::try_from(event.clone()),
    //         Err(TryFromTendermintEventError::IncorrectType {
    //             expected: "connection_open_confirm",
    //             found: event,
    //         })
    //     );
    // }

    // #[test]
    // fn unknown_field() {
    //     assert_eq!(
    //         ConnectionOpenConfirm::try_from(Event {
    //             ty: "connection_open_confirm".to_string(),
    //             attributes: vec![EventAttribute {
    //                 key: "abracadabra".to_string(),
    //                 value: "doesn't matter".to_string(),
    //                 index: true,
    //             },],
    //         }),
    //         Err(TryFromTendermintEventError::UnknownAttribute(
    //             "abracadabra".to_string()
    //         ))
    //     );
    // }

    // #[test]
    // fn create() {
    //     let client_type = "07-tendermint";
    //     let client_id = "07-tendermint-0";
    //     let consensus_height = "1-88";

    //     let create_client_event = Event {
    //         ty: "create_client".to_owned(),
    //         attributes: [
    //             EventAttribute {
    //                 key: "client_id".to_owned(),
    //                 value: client_id.to_owned(),
    //                 index: true,
    //             },
    //             EventAttribute {
    //                 key: "client_type".to_owned(),
    //                 value: client_type.to_owned(),
    //                 index: true,
    //             },
    //             EventAttribute {
    //                 key: "consensus_height".to_owned(),
    //                 value: consensus_height.to_owned(),
    //                 index: true,
    //             },
    //             EventAttribute {
    //                 key: "msg_index".to_owned(),
    //                 value: "0".to_owned(),
    //                 index: true,
    //             },
    //         ]
    //         .to_vec(),
    //     };

    //     assert_eq!(
    //         CreateClient::try_from(create_client_event).unwrap(),
    //         CreateClient {
    //             client_id: client_id.parse().unwrap(),
    //             client_type: client_type.to_owned(),
    //             consensus_height: consensus_height.parse().unwrap(),
    //         }
    //     );
    // }
}
