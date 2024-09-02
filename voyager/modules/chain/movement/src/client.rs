#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

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
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct MerklePrefix {
        pub key_prefix: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = connection_end)]
    pub struct Version {
        pub identifier: String,
        pub features: Vec<String>,
    }
}

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
        fn create_client(
            &self,
            (_0, _1, _2): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
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
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
                    )
                },
                {
                    let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_2);
                    (
                        bcs::to_bytes(&t).unwrap(),
                        <Vec<u8> as ::move_bindgen::TypeTagged>::type_tag(ctx),
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
        fn hackerman<T0: ::serde::Serialize + ::move_bindgen::TypeTagged>(
            &self,
            (_0,): (impl ::move_bindgen::IntoTypeTagged<T0>,),
        ) -> ::aptos_types::transaction::EntryFunction {
            let (values, type_args): (Vec<_>, Vec<_>) = vec![{
                let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(_0);
                (
                    bcs::to_bytes(&t).unwrap(),
                    <T0 as ::move_bindgen::TypeTagged>::type_tag(ctx),
                )
            }]
            .into_iter()
            .unzip();
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
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct ChannelOpenAck {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct WriteAcknowledgement {
        pub packet: super::packet::Packet,
        pub acknowledgement: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct Hackerman<T0> {
        pub t: T0,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
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
    #[type_tag(module = Core)]
    pub struct ConnectionOpenTry {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct ConnectionOpenAck {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct ChannelOpenInit {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub connection_id: String,
        pub version: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct RecvPacket {
        pub packet: super::packet::Packet,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
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
    #[type_tag(module = Core)]
    pub struct ConnectionOpenConfirm {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
        pub counterparty_connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct ClientCreatedEvent {
        pub client_id: String,
        pub client_type: String,
        pub consensus_height: super::height::Height,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct ConnectionOpenInit {
        pub connection_id: String,
        pub client_id: String,
        pub counterparty_client_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct AcknowledgePacket {
        pub packet: super::packet::Packet,
        pub acknowledgement: Vec<u8>,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct ChannelOpenConfirm {
        pub port_id: String,
        pub channel_id: String,
        pub counterparty_port_id: String,
        pub counterparty_channel_id: String,
        pub connection_id: String,
    }
    #[macros::model]
    #[derive(::move_bindgen::TypeTagged)]
    #[type_tag(module = Core)]
    pub struct TimeoutPacket {
        pub packet: super::packet::Packet,
    }
}
