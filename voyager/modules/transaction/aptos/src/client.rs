#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

pub mod ibc {
    pub trait ClientExt {
        fn client(&self) -> &::aptos_rest_client::Client;
        async fn client_state(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<Vec<u8>, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(client_state).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                        ],
                        arguments: vec![serde_json::to_value(String::from(_0)).unwrap()],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(Vec<u8>,)>(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        fn connection_open_ack(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_ack).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_7).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_8).0).unwrap(),
                ],
            )
        }
        fn connection_open_confirm(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_confirm).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                ],
            )
        }
        fn connection_open_init(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_init).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0).unwrap(),
                ],
            )
        }
        fn connection_open_try(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<Vec<String>>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_try).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_7).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_8).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_9).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_10).0)
                        .unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_11).0)
                        .unwrap(),
                ],
            )
        }
        async fn consensus_state(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2): (String, u64, u64),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<Vec<u8>, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(consensus_state).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                            ::aptos_rest_client::aptos_api_types::MoveType::U64,
                            ::aptos_rest_client::aptos_api_types::MoveType::U64,
                        ],
                        arguments: vec![
                            serde_json::to_value(String::from(_0)).unwrap(),
                            serde_json::to_value(::aptos_rest_client::aptos_api_types::U64::from(
                                _1,
                            ))
                            .unwrap(),
                            serde_json::to_value(::aptos_rest_client::aptos_api_types::U64::from(
                                _2,
                            ))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(Vec<u8>,)>(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        fn create_client(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(create_client).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                ],
            )
        }
        async fn get_channel(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1): (String, String),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            Option<super::channel::Channel>,
            ::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_channel).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                        ],
                        arguments: vec![
                            serde_json::to_value(String::from(_0)).unwrap(),
                            serde_json::to_value(String::from(_1)).unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(Option<super::channel::Channel>,)>(
                ::serde_json::Value::from(response),
            )?;
            Ok(ret.0)
        }
        async fn get_connection(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            Option<super::connection_end::ConnectionEnd>,
            ::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_connection).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                        ],
                        arguments: vec![serde_json::to_value(String::from(_0)).unwrap()],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(Option<super::connection_end::ConnectionEnd>,)>(
                ::serde_json::Value::from(response),
            )?;
            Ok(ret.0)
        }
        async fn get_connection_commitment(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<Option<Vec<u8>>, ::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_connection_commitment).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                        ],
                        arguments: vec![serde_json::to_value(String::from(_0)).unwrap()],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(Option<Vec<u8>>,)>(::serde_json::Value::from(
                response,
            ))?;
            Ok(ret.0)
        }
        async fn get_next_sequence_recv(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1): (String, String),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<u64, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_next_sequence_recv).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                        ],
                        arguments: vec![
                            serde_json::to_value(String::from(_0)).unwrap(),
                            serde_json::to_value(String::from(_1)).unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(u64,)>(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        async fn get_next_sequence_send(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1): (String, String),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<u64, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_next_sequence_send).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(
                                ::aptos_rest_client::aptos_api_types::MoveStructTag {
                                    address: "0x1".parse().unwrap(),
                                    module: "string".parse().unwrap(),
                                    name: "String".parse().unwrap(),
                                    generic_type_params: vec![],
                                },
                            ),
                        ],
                        arguments: vec![
                            serde_json::to_value(String::from(_0)).unwrap(),
                            serde_json::to_value(String::from(_1)).unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(u64,)>(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        async fn get_vault_addr(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::aptos_rest_client::aptos_api_types::Address,
            ::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_vault_addr).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<(::aptos_rest_client::aptos_api_types::Address,)>(
                ::serde_json::Value::from(response),
            )?;
            Ok(ret.0)
        }
        fn update_client(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(update_client).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                ],
            )
        }
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ClientUpdated {
        pub client_id: String,
        pub client_type: String,
        pub height: super::height::Height,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct SendPacket {
        pub sequence: ::aptos_rest_client::aptos_api_types::U64,
        pub source_port: String,
        pub source_channel: String,
        pub timeout_height: super::height::Height,
        pub timeout_timestamp: ::aptos_rest_client::aptos_api_types::U64,
        pub data: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenConfirm {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenAck {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct WriteAcknowledgement {
        pub packet: super::packet::Packet,
        pub acknowledgement: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenInit {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenAck {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenInit {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub connection_id: String,
        pub version: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenTry {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenConfirm {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct TimeoutPacket {
        pub packet: super::packet::Packet,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct AcknowledgePacket {
        pub packet: super::packet::Packet,
        pub acknowledgement: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenTry {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
        pub version: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct RecvPacket {
        pub packet: super::packet::Packet,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = ibc)]
    pub struct ClientCreatedEvent {
        pub client_id: String,
        pub client_type: String,
        pub consensus_height: super::height::Height,
    }
}

pub mod channel {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = channel)]
    pub struct Counterparty {
        pub port_id: String,
        pub channel_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = channel)]
    pub struct Channel {
        pub state: u8,
        pub ordering: u8,
        pub counterparty: super::channel::Counterparty,
        pub connection_hops: Vec<String>,
        pub version: String,
    }
}

pub mod packet {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = packet)]
    pub struct Packet {
        pub sequence: ::aptos_rest_client::aptos_api_types::U64,
        pub source_port: String,
        pub source_channel: String,
        pub destination_port: String,
        pub destination_channel: String,
        pub data: Vec<u8>,
        pub timeout_height: super::height::Height,
        pub timeout_timestamp: ::aptos_rest_client::aptos_api_types::U64,
    }
}

pub mod height {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = height)]
    pub struct Height {
        pub revision_number: ::aptos_rest_client::aptos_api_types::U64,
        pub revision_height: ::aptos_rest_client::aptos_api_types::U64,
    }
}

pub mod connection_end {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct Version {
        pub identifier: String,
        pub features: Vec<String>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct MerklePrefix {
        pub key_prefix: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct ConnectionEnd {
        pub client_id: String,
        pub versions: Vec<super::connection_end::Version>,
        pub state: ::aptos_rest_client::aptos_api_types::U64,
        pub delay_period: ::aptos_rest_client::aptos_api_types::U64,
        pub counterparty: super::connection_end::Counterparty,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct Counterparty {
        pub client_id: String,
        pub connection_id: String,
        pub prefix: super::connection_end::MerklePrefix,
    }
}
