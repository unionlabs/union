use serde::{Deserialize, Serialize};

use crate::{
    cosmwasm::wasm::access_config::AccessConfig,
    primitives::{Bech32, Bytes, H256, encoding::Base64},
};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgStoreAndMigrateContract {
    #[serde(rename = "authority")]
    pub sender: Bech32<Bytes>,
    pub wasm_byte_code: Bytes<Base64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiate_permission: Option<AccessConfig>,
    pub contract: Bech32<H256>,
    pub msg: Bytes<Base64>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::MsgStoreAndMigrateContract;
    use crate::{
        Msg,
        cosmwasm::wasm::access_config,
        impl_proto_via_try_from_into,
        primitives::{Bech32DecodeError, FixedBytesError},
    };

    impl_proto_via_try_from_into!(MsgStoreAndMigrateContract => protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContract);

    impl From<MsgStoreAndMigrateContract> for protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContract {
        fn from(value: MsgStoreAndMigrateContract) -> Self {
            Self {
                authority: value.sender.to_string(),
                wasm_byte_code: value.wasm_byte_code.to_vec(),
                instantiate_permission: value.instantiate_permission.map(Into::into),
                contract: value.contract.to_string(),
                msg: value.msg.to_vec(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContract>
        for MsgStoreAndMigrateContract
    {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContract,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.authority.parse().map_err(Error::Sender)?,
                wasm_byte_code: value.wasm_byte_code.into(),
                instantiate_permission: value
                    .instantiate_permission
                    .map(TryInto::try_into)
                    .transpose()?,
                contract: value.contract.parse().map_err(Error::Contract)?,
                msg: value.msg.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[source] Bech32DecodeError),
        #[error("invalid instantiate permissions")]
        InstantiatePermissions(#[from] access_config::proto::Error),
        #[error("invalid contract")]
        Contract(#[source] Bech32DecodeError<FixedBytesError>),
    }

    impl Msg for MsgStoreAndMigrateContract {
        type Response = super::response::MsgStoreAndMigrateContractResponse;
    }
}
