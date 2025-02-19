#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::type_complexity,
    clippy::needless_borrows_for_generic_args,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

pub mod channel_handshake {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        fn channel_open_ack(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                u32,
                String,
                u32,
                Vec<u8>,
                u64,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(channel_handshake).parse().unwrap(),
                })
                .into(),
                stringify!(channel_open_ack).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_4).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_5).unwrap(),
                ],
            )
        }
        fn channel_open_confirm(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                u32,
                Vec<u8>,
                u64,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(channel_handshake).parse().unwrap(),
                })
                .into(),
                stringify!(channel_open_confirm).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                ],
            )
        }
        fn channel_open_init(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                Vec<u8>,
                u32,
                String,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(channel_handshake).parse().unwrap(),
                })
                .into(),
                stringify!(channel_open_init).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                ],
            )
        }
        fn channel_open_try(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                u32,
                u32,
                Vec<u8>,
                String,
                String,
                Vec<u8>,
                u64,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(channel_handshake).parse().unwrap(),
                })
                .into(),
                stringify!(channel_open_try).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_4).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_5).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_6).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_7).unwrap(),
                ],
            )
        }
    }
}

pub mod acknowledge_packet {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        fn acknowledge_packet(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8, _9): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                Vec<u32>,
                Vec<u32>,
                Vec<Vec<u8>>,
                Vec<u64>,
                Vec<u64>,
                Vec<Vec<u8>>,
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                Vec<u8>,
                u64,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(acknowledge_packet).parse().unwrap(),
                })
                .into(),
                stringify!(acknowledge_packet).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_4).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_5).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_6).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_7).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_8).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_9).unwrap(),
                ],
            )
        }
    }
}

pub mod move_in_cosmos_client {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
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
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(move_in_cosmos_client).parse().unwrap(),
                            },
                            name: stringify!(get_vault_addr).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::Raw,
                )
            >(value)?;
            Ok(
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::from_raw(
                        ret_0
                    ),
                ).0
            )
        }
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct CreateLensClient {
        pub client_id: u32,
        pub l1_client_id: u32,
        pub l2_client_id: u32,
        pub l2_chain_id: String,
    }
}

pub mod channel {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct Channel {
        pub state: u8,
        pub connection_id: u32,
        pub counterparty_channel_id: u32,
        pub counterparty_port_id: Vec<u8>,
        pub version: String,
    }
}

pub mod dispatcher {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
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
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(dispatcher).parse().unwrap(),
                            },
                            name: stringify!(get_vault_addr).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::Raw,
                )
            >(value)?;
            Ok(
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::from_raw(
                        ret_0
                    ),
                ).0
            )
        }
    }
}

pub mod recv_packet {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        fn recv_packet(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8, _9): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                Vec<u32>,
                Vec<u32>,
                Vec<Vec<u8>>,
                Vec<u64>,
                Vec<u64>,
                Vec<Vec<u8>>,
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                Vec<u8>,
                u64,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(recv_packet).parse().unwrap(),
                })
                .into(),
                stringify!(recv_packet).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_4).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_5).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_6).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_7).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_8).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_9).unwrap(),
                ],
            )
        }
    }
}

pub mod timeout_packet {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        fn timeout_packet(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8): (
                ::move_bindgen::aptos_types::account_address::AccountAddress,
                u32,
                u32,
                Vec<u8>,
                u64,
                u64,
                Vec<u8>,
                u64,
                u64,
            ),
            (t0,): (impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>,),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(timeout_packet).parse().unwrap(),
                })
                .into(),
                stringify!(timeout_packet).parse().unwrap(),
                vec![t0.into().into()],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_4).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_5).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_6).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_7).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_8).unwrap(),
                ],
            )
        }
    }
}

pub mod connection_end {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ConnectionEnd {
        pub state: u64,
        pub client_id: u32,
        pub counterparty_client_id: u32,
        pub counterparty_connection_id: u32,
    }
}

pub mod ibc {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn client_id_to_type(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (u32,),
        ) -> ::core::result::Result<String, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(client_id_to_type).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <String as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((<String as ::move_bindgen::MoveOutputType>::from_raw(ret_0),).0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn client_state(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (u32,),
        ) -> ::core::result::Result<Vec<u8>, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(client_state).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <Vec<u8> as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((<Vec<u8> as ::move_bindgen::MoveOutputType>::from_raw(ret_0),).0)
        }
        fn connection_open_ack(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3): (u32, u32, Vec<u8>, u64),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(connection_open_ack).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                ],
            )
        }
        fn connection_open_confirm(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2): (u32, Vec<u8>, u64),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(connection_open_confirm).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                ],
            )
        }
        fn connection_open_init(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (u32, u32),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(connection_open_init).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                ],
            )
        }
        fn connection_open_try(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4): (u32, u32, u32, Vec<u8>, u64),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(connection_open_try).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_3).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_4).unwrap(),
                ],
            )
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, ?_1, )
        )]
        async fn consensus_state(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0, _1): (u32, u64),
        ) -> ::core::result::Result<Vec<u8>, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(consensus_state).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap(),
                            ::move_bindgen::serde_json
                                ::to_value(&<u64 as ::move_bindgen::MoveOutputType>::into_raw(_1))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <Vec<u8> as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((<Vec<u8> as ::move_bindgen::MoveOutputType>::from_raw(ret_0),).0)
        }
        fn create_client(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2): (String, Vec<u8>, Vec<u8>),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(create_client).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_2).unwrap(),
                ],
            )
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn get_channel(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (u32,),
        ) -> ::core::result::Result<
            Option<super::channel::Channel>,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_channel).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <Option<super::channel::Channel> as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((
                <Option<super::channel::Channel> as ::move_bindgen::MoveOutputType>::from_raw(
                    ret_0,
                ),
            )
                .0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn get_commitment(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (Vec<u8>,),
        ) -> ::core::result::Result<Vec<u8>, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_commitment).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(
                                    &<Vec<u8> as ::move_bindgen::MoveOutputType>::into_raw(_0)
                                )
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <Vec<u8> as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((<Vec<u8> as ::move_bindgen::MoveOutputType>::from_raw(ret_0),).0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn get_connection(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (u32,),
        ) -> ::core::result::Result<
            Option<super::connection_end::ConnectionEnd>,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_connection).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<
                (
                    <Option<super::connection_end::ConnectionEnd> as ::move_bindgen::MoveOutputType>::Raw,
                )
            >(value)?;
            Ok(
                (
                    <Option<super::connection_end::ConnectionEnd> as ::move_bindgen::MoveOutputType>::from_raw(
                        ret_0
                    ),
                ).0
            )
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn get_counterparty_connection(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (u32,),
        ) -> ::core::result::Result<u32, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_counterparty_connection).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <u32 as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((<u32 as ::move_bindgen::MoveOutputType>::from_raw(ret_0),).0)
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, ?_0, )
        )]
        async fn get_module(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
            (_0,): (u32,),
        ) -> ::core::result::Result<
            ::move_bindgen::aptos_rest_client::aptos_api_types::Address,
            ::move_bindgen::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_module).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![
                            ::move_bindgen::serde_json
                                ::to_value(&<u32 as ::move_bindgen::MoveOutputType>::into_raw(_0))
                                .unwrap()
                        ],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::Raw,
                )
            >(value)?;
            Ok(
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::from_raw(
                        ret_0
                    ),
                ).0
            )
        }
        #[::move_bindgen::tracing::instrument(
            skip_all,
            fields(%contract_address, ?ledger_version, )
        )]
        async fn get_next_channel_sequence(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<u64, ::move_bindgen::aptos_rest_client::error::RestError>
        {
            let response = self
                .client()
                .view(
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_next_channel_sequence).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<(
                <u64 as ::move_bindgen::MoveOutputType>::Raw,
            )>(value)?;
            Ok((<u64 as ::move_bindgen::MoveOutputType>::from_raw(ret_0),).0)
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
                    &(::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: contract_address.into(),
                                name: stringify!(ibc).parse().unwrap(),
                            },
                            name: stringify!(get_vault_addr).parse().unwrap(),
                        },
                        type_arguments: vec![],
                        arguments: vec![],
                    }),
                    ledger_version
                ).await?
                .into_inner();
            let value = ::move_bindgen::serde_json::Value::from(response);
            ::move_bindgen::tracing::debug!(% value, "fetched response");
            let (ret_0,) = ::move_bindgen::serde_json::from_value::<
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::Raw,
                )
            >(value)?;
            Ok(
                (
                    <::move_bindgen::aptos_rest_client::aptos_api_types::Address as ::move_bindgen::MoveOutputType>::from_raw(
                        ret_0
                    ),
                ).0
            )
        }
        fn submit_misbehaviour(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (u32, Vec<u8>),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(submit_misbehaviour).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                ],
            )
        }
        fn update_client(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1): (u32, Vec<u8>),
        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction {
            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                (::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc).parse().unwrap(),
                })
                .into(),
                stringify!(update_client).parse().unwrap(),
                vec![],
                vec![
                    ::move_bindgen::bcs::to_bytes(&_0).unwrap(),
                    ::move_bindgen::bcs::to_bytes(&_1).unwrap(),
                ],
            )
        }
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct CreateClient {
        pub client_id: u32,
        pub client_type: String,
        pub counterparty_chain_id: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ChannelOpenAck {
        pub port_id: String,
        pub channel_id: u32,
        pub counterparty_port_id: Vec<u8>,
        pub counterparty_channel_id: u32,
        pub connection_id: u32,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct WriteAck {
        pub packet: super::packet::Packet,
        pub acknowledgement: Vec<u8>,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct PacketSend {
        pub source_channel_id: u32,
        pub destination_channel_id: u32,
        pub data: Vec<u8>,
        pub timeout_height: u64,
        pub timeout_timestamp: u64,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct UpdateClient {
        pub client_id: u32,
        pub client_type: String,
        pub counterparty_height: u64,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ConnectionOpenTry {
        pub connection_id: u32,
        pub client_id: u32,
        pub counterparty_client_id: u32,
        pub counterparty_connection_id: u32,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ChannelOpenTry {
        pub port_id: String,
        pub channel_id: u32,
        pub counterparty_port_id: Vec<u8>,
        pub counterparty_channel_id: u32,
        pub connection_id: u32,
        pub version: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct PacketRecv {
        pub packet: super::packet::Packet,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct RecvIntentPacket {
        pub packet: super::packet::Packet,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct PacketAck {
        pub packet: super::packet::Packet,
        pub acknowledgement: Vec<u8>,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct TimeoutPacket {
        pub packet: super::packet::Packet,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct SubmitMisbehaviour {
        pub client_id: u32,
        pub client_type: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ChannelOpenInit {
        pub port_id: String,
        pub channel_id: u32,
        pub counterparty_port_id: Vec<u8>,
        pub connection_id: u32,
        pub version: String,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ConnectionOpenAck {
        pub connection_id: u32,
        pub client_id: u32,
        pub counterparty_client_id: u32,
        pub counterparty_connection_id: u32,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ChannelOpenConfirm {
        pub port_id: String,
        pub channel_id: u32,
        pub counterparty_port_id: Vec<u8>,
        pub counterparty_channel_id: u32,
        pub connection_id: u32,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ConnectionOpenConfirm {
        pub connection_id: u32,
        pub client_id: u32,
        pub counterparty_client_id: u32,
        pub counterparty_connection_id: u32,
    }
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct ConnectionOpenInit {
        pub connection_id: u32,
        pub client_id: u32,
        pub counterparty_client_id: u32,
    }
}

pub mod packet {
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        ::move_bindgen::serde::Serialize,
        ::move_bindgen::serde::Deserialize,
        ::move_bindgen::MoveOutputType,
    )]
    #[serde(crate = "::move_bindgen::serde")]
    pub struct Packet {
        pub source_channel_id: u32,
        pub destination_channel_id: u32,
        pub data: Vec<u8>,
        pub timeout_height: u64,
        pub timeout_timestamp: u64,
    }
}
