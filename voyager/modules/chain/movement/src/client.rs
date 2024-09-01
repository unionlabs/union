#![allow(async_fn_in_trait,
non_snake_case,
clippy::useless_conversion,
clippy::unused_unit,
clippy::too_many_arguments)]

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
                            module: aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
                            },
                            name: stringify!(client_state).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(::aptos_rest_client::aptos_api_types::MoveStructTag
                            { address : "0x1".parse().unwrap(), module : "string".parse()
                            .unwrap(), name : "String".parse().unwrap(),
                            generic_type_params : vec![], }),
                        ],
                        arguments: vec![serde_json::to_value(String::from(_0)).unwrap(),],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<
                (Vec<u8>,),
            >(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        async fn connection_open_init(
            &self,
            _0: String,
            _1: String,
            _2: Vec<String>,
            _3: String,
            _4: String,
            _5: Vec<u8>,
            _6: u64,
        ) -> () {
            todo!()
        }
        async fn connection_open_try(
            &self,
            _0: String,
            _1: String,
            _2: Vec<u8>,
            _3: u64,
            _4: String,
            _5: Vec<u8>,
            _6: Vec<String>,
            _7: Vec<Vec<String>>,
            _8: Vec<u8>,
            _9: Vec<u8>,
            _10: u64,
            _11: u64,
        ) -> () {
            todo!()
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
                            module: aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
                            },
                            name: stringify!(consensus_state).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(::aptos_rest_client::aptos_api_types::MoveStructTag
                            { address : "0x1".parse().unwrap(), module : "string".parse()
                            .unwrap(), name : "String".parse().unwrap(),
                            generic_type_params : vec![], }),
                            ::aptos_rest_client::aptos_api_types::MoveType::U64,
                            ::aptos_rest_client::aptos_api_types::MoveType::U64,
                        ],
                        arguments: vec![
                            serde_json::to_value(String::from(_0)).unwrap(),
                            serde_json::to_value(::aptos_rest_client::aptos_api_types::U64::from(_1))
                            .unwrap(),
                            serde_json::to_value(::aptos_rest_client::aptos_api_types::U64::from(_2))
                            .unwrap(),
                        ],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<
                (Vec<u8>,),
            >(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        async fn create_client<T0, T1>(&self, _0: String, _1: T0, _2: T1) -> () {
            todo!()
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
                            module: aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
                            },
                            name: stringify!(get_connection).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(::aptos_rest_client::aptos_api_types::MoveStructTag
                            { address : "0x1".parse().unwrap(), module : "string".parse()
                            .unwrap(), name : "String".parse().unwrap(),
                            generic_type_params : vec![], }),
                        ],
                        arguments: vec![serde_json::to_value(String::from(_0)).unwrap(),],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<
                (super::connection_end::ConnectionEnd,),
            >(::serde_json::Value::from(response))?;
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
                            module: aptos_rest_client::aptos_api_types::MoveModuleId {
                                address: self.module_address().into(),
                                name: stringify!(Core).parse().unwrap(),
                            },
                            name: stringify!(get_connection_commitment).parse().unwrap(),
                        },
                        type_arguments: vec![
                            ::aptos_rest_client::aptos_api_types::MoveType::Struct(::aptos_rest_client::aptos_api_types::MoveStructTag
                            { address : "0x1".parse().unwrap(), module : "string".parse()
                            .unwrap(), name : "String".parse().unwrap(),
                            generic_type_params : vec![], }),
                        ],
                        arguments: vec![serde_json::to_value(String::from(_0)).unwrap(),],
                    },
                    ledger_version,
                )
                .await?
                .into_inner();
            let ret = ::serde_json::from_value::<
                (Vec<u8>,),
            >(::serde_json::Value::from(response))?;
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
                            module: aptos_rest_client::aptos_api_types::MoveModuleId {
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
            let ret = ::serde_json::from_value::<
                (::aptos_rest_client::aptos_api_types::Address,),
            >(::serde_json::Value::from(response))?;
            Ok(ret.0)
        }
        async fn hackerman(&self) -> () {
            todo!()
        }
    }
}

pub mod ics23 {}

pub mod height {
    #[macros::model]
    pub struct Height {
        revision_number: ::aptos_rest_client::aptos_api_types::U64,
        revision_height: ::aptos_rest_client::aptos_api_types::U64,
    }
}

pub mod packet {
    #[macros::model]
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
    pub struct Channel {
        state: u8,
        ordering: u8,
        counterparty: super::channel::Counterparty,
        connection_hops: Vec<String>,
        version: String,
    }
    #[macros::model]
    pub struct Counterparty {
        port_id: String,
        channel_id: String,
    }
}

pub mod LightClient {}

pub mod proto_utils {}

pub mod IBCCommitment {}

pub mod connection_end {
    #[macros::model]
    pub struct MerklePrefix {
        key_prefix: Vec<u8>,
    }
    #[macros::model]
    pub struct Version {
        identifier: String,
        features: Vec<String>,
    }
    #[macros::model]
    pub struct Counterparty {
        client_id: String,
        connection_id: String,
        prefix: super::connection_end::MerklePrefix,
    }
    #[macros::model]
    pub struct ConnectionEnd {
        client_id: String,
        versions: Vec<super::connection_end::Version>,
        state: ::aptos_rest_client::aptos_api_types::U64,
        delay_period: ::aptos_rest_client::aptos_api_types::U64,
        counterparty: super::connection_end::Counterparty,
    }
}
