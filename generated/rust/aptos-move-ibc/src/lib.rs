#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

pub mod app;

pub mod channel {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = channel)]
    pub struct Counterparty {
        pub port_id: String,
        pub channel_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = channel)]
    pub struct Channel {
        pub state: u8,
        pub ordering: u8,
        pub counterparty: super::channel::Counterparty,
        pub connection_hops: Vec<String>,
        pub version: String,
    }
}

pub mod connection_end {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = connection_end)]
    pub struct Counterparty {
        pub client_id: String,
        pub connection_id: String,
        pub prefix: super::connection_end::MerklePrefix,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = connection_end)]
    pub struct ConnectionEnd {
        pub client_id: String,
        pub versions: Vec<super::connection_end::Version>,
        pub state: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
        pub delay_period: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
        pub counterparty: super::connection_end::Counterparty,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = connection_end)]
    pub struct MerklePrefix {
        pub key_prefix: ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = connection_end)]
    pub struct Version {
        pub identifier: String,
        pub features: Vec<String>,
    }
}

pub mod height {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = height)]
    pub struct Height {
        pub revision_number: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
        pub revision_height: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
    }
}

pub mod ibc {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, )
        )]
        async fn client_state(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(client_state).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(
                ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
            )>(value)?;
            Ok(ret.0)
        }
        fn connection_open_ack(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
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
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_ack).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_7).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_8).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn connection_open_confirm(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_confirm).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn connection_open_init(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_init).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn connection_open_try(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
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
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_try).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_7).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_8).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_9).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_10).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_11).0,
                    )
                    .unwrap(),
                ],
            )
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, %_1, %_2, )
        )]
        async fn consensus_state(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2): (String, u64, u64),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(consensus_state).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                            ::move_bindgen::serde_json::to_value(::move_bindgen::aptos_rest_client::aptos_api_types::U64::from(_1))
                            .unwrap(),
                            ::move_bindgen::serde_json::to_value(::move_bindgen::aptos_rest_client::aptos_api_types::U64::from(_2))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(
                ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
            )>(value)?;
            Ok(ret.0)
        }
        fn create_client(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(create_client).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                ],
            )
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, %_1, )
        )]
        async fn get_channel(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (String, String),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::move_bindgen::MoveOption<super::channel::Channel>,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_channel).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                            ::move_bindgen::serde_json::to_value(String::from(_1))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(
                ::move_bindgen::MoveOption<super::channel::Channel>,
            )>(value)?;
            Ok(ret.0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, )
        )]
        async fn get_connection(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::move_bindgen::MoveOption<super::connection_end::ConnectionEnd>,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_connection).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(
                ::move_bindgen::MoveOption<super::connection_end::ConnectionEnd>,
            )>(value)?;
            Ok(ret.0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, )
        )]
        async fn get_connection_commitment(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::move_bindgen::MoveOption<
                ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
            >,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_connection_commitment).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(
                ::move_bindgen::MoveOption<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
            )>(value)?;
            Ok(ret.0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, %_1, )
        )]
        async fn get_next_sequence_recv(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (String, String),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<u64, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_next_sequence_recv).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                            ::move_bindgen::serde_json::to_value(String::from(_1))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(u64,)>(value)?;
            Ok(ret.0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, %_0, %_1, )
        )]
        async fn get_next_sequence_send(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (String, String),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<u64, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_next_sequence_send).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json::to_value(String::from(_0))
                            .unwrap(),
                            ::move_bindgen::serde_json::to_value(String::from(_1))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(u64,)>(value)?;
            Ok(ret.0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, )
        )]
        async fn get_vault_addr(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            ::move_bindgen::aptos_rest_client::aptos_api_types::Address,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
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
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let ret = ::move_bindgen::serde_json::from_value::<(
                ::move_bindgen::aptos_rest_client::aptos_api_types::Address,
            )>(value)?;
            Ok(ret.0)
        }
        fn update_client(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(update_client).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn acknowledge_packet(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12): (
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(acknowledge_packet).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_7).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_8).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_9).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_10).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_11).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_12).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn channel_close_confirm(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(channel_close_confirm).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn channel_close_init(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(channel_close_init).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn channel_open_ack(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_ack).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn channel_open_confirm(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_confirm).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn channel_open_init(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4): (
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<u8>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_init).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                ],
            )
        }
        fn channel_open_try(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8): (
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<u8>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_try).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_6).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_7).0,
                    )
                    .unwrap(),
                    ::move_bindgen::bcs::to_bytes(
                        &::move_bindgen::IntoTypeTagged::into_type_tagged(_8).0,
                    )
                    .unwrap(),
                ],
            )
        }
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenInit {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenAck {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenConfirm {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct SendPacket {
        pub sequence: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
        pub source_port: String,
        pub source_channel: String,
        pub timeout_height: super::height::Height,
        pub timeout_timestamp: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
        pub data: ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ClientUpdated {
        pub client_id: String,
        pub client_type: String,
        pub height: super::height::Height,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenInit {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub connection_id: String,
        pub version: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct TimeoutPacket {
        pub packet: super::packet::Packet,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenConfirm {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenTry {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
        pub version: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ChannelOpenAck {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct WriteAcknowledgement {
        pub packet: super::packet::Packet,
        pub acknowledgement: ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ClientCreatedEvent {
        pub client_id: String,
        pub client_type: String,
        pub consensus_height: super::height::Height,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct AcknowledgePacket {
        pub packet: super::packet::Packet,
        pub acknowledgement: ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct ConnectionOpenTry {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = ibc)]
    pub struct RecvPacket {
        pub packet: super::packet::Packet,
    }
}

pub mod packet {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::TypeTagged,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    #[type_tag(module = packet)]
    pub struct Packet {
        pub sequence: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
        pub source_port: String,
        pub source_channel: String,
        pub destination_port: String,
        pub destination_channel: String,
        pub data: ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
        pub timeout_height: super::height::Height,
        pub timeout_timestamp: ::move_bindgen::aptos_rest_client::aptos_api_types::U64,
    }
}
