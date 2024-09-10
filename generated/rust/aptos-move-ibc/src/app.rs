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
    pub struct RingEvent {
        pub ping: bool,
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
    pub struct TimedOutEvent {
        pub dummy_field: bool,
    }
}
