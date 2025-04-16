use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};
use unionlabs_primitives::Bytes;

use crate::{bech32::Bech32, cosmwasm::wasm::access_config::AccessConfig};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgUpdateInstantiateConfig {
    pub sender: Bech32<Bytes>,
    pub code_id: NonZeroU64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_instantiate_permission: Option<AccessConfig>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use unionlabs_primitives::Bytes;

    use super::MsgUpdateInstantiateConfig;
    use crate::{bech32::Bech32, cosmwasm::wasm::access_config, impl_proto_via_try_from_into, Msg};

    impl_proto_via_try_from_into!(MsgUpdateInstantiateConfig => protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfig);

    impl From<MsgUpdateInstantiateConfig> for protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfig {
        fn from(value: MsgUpdateInstantiateConfig) -> Self {
            Self {
                sender: value.sender.to_string(),
                code_id: value.code_id.get(),
                new_instantiate_permission: value.new_instantiate_permission.map(Into::into),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfig>
        for MsgUpdateInstantiateConfig
    {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfig,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.sender.parse()?,
                code_id: value.code_id.try_into().map_err(|_| Error::CodeId)?,
                new_instantiate_permission: value
                    .new_instantiate_permission
                    .map(TryInto::try_into)
                    .transpose()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[from] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid code id, must be > 0")]
        CodeId,
        #[error("invalid instantiate permissions")]
        InstantiatePermissions(#[from] access_config::proto::Error),
    }

    impl Msg for MsgUpdateInstantiateConfig {
        type Response = super::response::MsgUpdateInstantiateConfigResponse;
    }
}
