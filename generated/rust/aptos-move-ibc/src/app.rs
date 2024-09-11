#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

pub mod ibc {
    pub trait ClientExt {
        fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;
        fn acknowledge_packet(
            &self,
            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12): (
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
                impl ::move_bindgen::IntoTypeTagged<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
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
                impl ::move_bindgen::IntoTypeTagged<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
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
                impl ::move_bindgen::IntoTypeTagged<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
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
                impl ::move_bindgen::IntoTypeTagged<
                    ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes,
                >,
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
    pub struct TimedOutEvent {
        pub dummy_field: bool,
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
    pub struct AcknowledgedEvent {
        pub dummy_field: bool,
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
    pub struct RingEvent {
        pub ping: bool,
    }
}
