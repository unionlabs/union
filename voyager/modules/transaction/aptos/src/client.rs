#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]

pub mod ibc_api {
    pub trait ClientExt {
        fn client(&self) -> &::aptos_rest_client::Client;
        fn channel_open_ack(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5): (
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<Vec<u8>>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc_api).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_ack).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_5).0).unwrap(),
                ],
            )
        }
        fn channel_open_confirm(
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
                    name: stringify!(ibc_api).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_confirm).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                ],
            )
        }
        fn channel_open_init(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4): (
                impl ::move_bindgen::IntoTypeTagged<Vec<String>>,
                impl ::move_bindgen::IntoTypeTagged<u8>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
                impl ::move_bindgen::IntoTypeTagged<String>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc_api).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_init).parse().unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_0).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_1).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_2).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_3).0).unwrap(),
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_4).0).unwrap(),
                ],
            )
        }
        fn channel_open_try(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
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
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc_api).parse().unwrap(),
                }
                .into(),
                stringify!(channel_open_try).parse().unwrap(),
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
                    name: stringify!(ibc_api).parse().unwrap(),
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
                    name: stringify!(ibc_api).parse().unwrap(),
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
                    name: stringify!(ibc_api).parse().unwrap(),
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
                    name: stringify!(ibc_api).parse().unwrap(),
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
                    name: stringify!(ibc_api).parse().unwrap(),
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
        fn recv_packet(
            &self,
            contract_address: ::aptos_types::account_address::AccountAddress,
            (_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12): (
                impl ::move_bindgen::IntoTypeTagged<String>,
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
                impl ::move_bindgen::IntoTypeTagged<u64>,
                impl ::move_bindgen::IntoTypeTagged<u64>,
            ),
        ) -> ::aptos_types::transaction::EntryFunction {
            ::aptos_types::transaction::EntryFunction::new(
                ::aptos_rest_client::aptos_api_types::MoveModuleId {
                    address: contract_address.into(),
                    name: stringify!(ibc_api).parse().unwrap(),
                }
                .into(),
                stringify!(recv_packet).parse().unwrap(),
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
                    bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(_12).0)
                        .unwrap(),
                ],
            )
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
                    name: stringify!(ibc_api).parse().unwrap(),
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
}
