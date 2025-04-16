use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base64, Bytes};

use crate::{bech32::Bech32, cosmwasm::wasm::access_config::AccessConfig};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgStoreCode {
    pub sender: Bech32<Bytes>,
    pub wasm_byte_code: Bytes<Base64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiate_permission: Option<AccessConfig>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use unionlabs_primitives::Bytes;

    use super::MsgStoreCode;
    use crate::{bech32::Bech32, cosmwasm::wasm::access_config, impl_proto_via_try_from_into, Msg};

    impl_proto_via_try_from_into!(MsgStoreCode => protos::cosmwasm::wasm::v1::MsgStoreCode);

    impl From<MsgStoreCode> for protos::cosmwasm::wasm::v1::MsgStoreCode {
        fn from(value: MsgStoreCode) -> Self {
            Self {
                sender: value.sender.to_string(),
                wasm_byte_code: value.wasm_byte_code.to_vec(),
                instantiate_permission: value.instantiate_permission.map(Into::into),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgStoreCode> for MsgStoreCode {
        type Error = Error;

        fn try_from(value: protos::cosmwasm::wasm::v1::MsgStoreCode) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.sender.parse()?,
                wasm_byte_code: value.wasm_byte_code.into(),
                instantiate_permission: value
                    .instantiate_permission
                    .map(TryInto::try_into)
                    .transpose()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[from] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid instantiate permissions")]
        InstantiatePermissions(#[from] access_config::proto::Error),
    }

    impl Msg for MsgStoreCode {
        type Response = super::response::MsgStoreCodeResponse;
    }
}
