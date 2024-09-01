#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

pub mod Core {
    pub trait ClientExt {
        fn client(&self) -> &::aptos_rest_client::Client;
        fn module_address(&self) -> ::aptos_types::account_address::AccountAddress;
        async fn client_state(
            &self,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<Vec<u8>, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
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
        fn connection_open_init(
            &self,
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
            let (values, type_args): (Vec<_>, Vec<_>) = vec![
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_0);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_1);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_2);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<String> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_3);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_4);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_5);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_6);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <u64 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
            ]
            .into_iter()
            .unzip();
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: self.module_address().into(),
                    name: stringify!(Core).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_init).parse().unwrap(),
                type_args,
                values,
            )
        }
        fn connection_open_try(
            &self,
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
            let (values, type_args): (Vec<_>, Vec<_>) = vec![
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_0);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_1);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_2);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_3);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <u64 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_4);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_5);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_6);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<String> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_7);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<Vec<String>> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_8);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_9);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_10);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <u64 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_11);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <u64 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
            ]
            .into_iter()
            .unzip();
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: self.module_address().into(),
                    name: stringify!(Core).parse().unwrap(),
                }
                .into(),
                stringify!(connection_open_try).parse().unwrap(),
                type_args,
                values,
            )
        }
        async fn consensus_state(
            &self,
            (_0, _1, _2): (String, u64, u64),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<Vec<u8>, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
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
        fn create_client<
            T0: ::serde::Serialize + ::move_bindgen::TypeTagged,
            T1: ::serde::Serialize + ::move_bindgen::TypeTagged,
        >(
            &self,
            (_0, _1, _2): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<T0>,
                impl ::move_bindgen::IntoTypeTagged<T1>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            let (values, type_args): (Vec<_>, Vec<_>) = vec![
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_0);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <String as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_1);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <T0 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_2);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <T1 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
            ]
            .into_iter()
            .unzip();
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: self.module_address().into(),
                    name: stringify!(Core).parse().unwrap(),
                }
                .into(),
                stringify!(create_client).parse().unwrap(),
                type_args,
                values,
            )
        }
        async fn get_connection(
            &self,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<
            super::connection_end::ConnectionEnd,
            ::aptos_rest_client::error::RestError,
        > {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
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
            let ret = ::serde_json::from_value::<(super::connection_end::ConnectionEnd,)>(
                ::serde_json::Value::from(response),
            )?;
            Ok(ret.0)
        }
        async fn get_connection_commitment(
            &self,
            (_0,): (String,),
            ledger_version: Option<u64>,
        ) -> ::core::result::Result<Vec<u8>, ::aptos_rest_client::error::RestError> {
            let response = self
                .client()
                .view(
                    &::aptos_rest_client::aptos_api_types::ViewRequest {
                        function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                            module: ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
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
            let ret = ::serde_json::from_value::<(Vec<u8>,)>(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        async fn get_vault_addr(
            &self,
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
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
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
        fn hackerman(&self) -> ::aptos_types::transaction::EntryFunction {
            let (values, type_args): (Vec<_>, Vec<_>) = vec![].into_iter().unzip();
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: self.module_address().into(),
                    name: stringify!(Core).parse().unwrap(),
                }
                .into(),
                stringify!(hackerman).parse().unwrap(),
                type_args,
                values,
            )
        }
    }
}

pub mod ics23 {}

pub mod height {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = height)]
    pub struct Height {
        revision_number: ::aptos_rest_client::aptos_api_types::U64,
        revision_height: ::aptos_rest_client::aptos_api_types::U64,
    }
}

pub mod packet {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = packet)]
    pub struct Packet {
        sequence: ::aptos_rest_client::aptos_api_types::U64,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        data: Vec<u8>,
        timeout_height: super::height::Height,
        timeout_timestamp: ::aptos_rest_client::aptos_api_types::U64,
    }
}

pub mod channel {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = channel)]
    pub struct Counterparty {
        port_id: String,
        channel_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = channel)]
    pub struct Channel {
        state: u8,
        ordering: u8,
        counterparty: super::channel::Counterparty,
        connection_hops: Vec<String>,
        version: String,
    }
}

pub mod LightClient {}

pub mod proto_utils {}

pub mod IBCCommitment {}

pub mod connection_end {
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct Counterparty {
        client_id: String,
        connection_id: String,
        prefix: super::connection_end::MerklePrefix,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct Version {
        identifier: String,
        features: Vec<String>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct ConnectionEnd {
        client_id: String,
        versions: Vec<super::connection_end::Version>,
        state: ::aptos_rest_client::aptos_api_types::U64,
        delay_period: ::aptos_rest_client::aptos_api_types::U64,
        counterparty: super::connection_end::Counterparty,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct MerklePrefix {
        key_prefix: Vec<u8>,
    }
}
