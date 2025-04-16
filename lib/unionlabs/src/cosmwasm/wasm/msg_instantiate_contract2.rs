use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base64, Bytes};

use crate::{bech32::Bech32, cosmos::base::coin::Coin};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgInstantiateContract2 {
    pub sender: Bech32<Bytes>,
    pub admin: Bech32<Bytes>,
    pub code_id: NonZeroU64,
    pub label: String,
    pub msg: Bytes<Base64>,
    pub funds: Vec<Coin>,
    pub salt: Bytes<Base64>,
    pub fix_msg: bool,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use unionlabs_primitives::Bytes;

    use super::MsgInstantiateContract2;
    use crate::{bech32::Bech32, cosmos::base::coin, impl_proto_via_try_from_into, Msg};

    impl_proto_via_try_from_into!(MsgInstantiateContract2 => protos::cosmwasm::wasm::v1::MsgInstantiateContract2);

    impl From<MsgInstantiateContract2> for protos::cosmwasm::wasm::v1::MsgInstantiateContract2 {
        fn from(value: MsgInstantiateContract2) -> Self {
            Self {
                sender: value.sender.to_string(),
                admin: value.admin.to_string(),
                code_id: value.code_id.get(),
                label: value.label,
                msg: value.msg.to_vec(),
                funds: value.funds.into_iter().map(Into::into).collect(),
                salt: value.salt.into(),
                fix_msg: value.fix_msg,
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgInstantiateContract2> for MsgInstantiateContract2 {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgInstantiateContract2,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.sender.parse().map_err(Error::Sender)?,
                admin: value.admin.parse().map_err(Error::Admin)?,
                code_id: value.code_id.try_into().map_err(|_| Error::CodeId)?,
                label: value.label,
                msg: value.msg.into(),
                funds: value
                    .funds
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                salt: value.salt.into(),
                fix_msg: value.fix_msg,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[source] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid admin")]
        Admin(#[source] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid code id, must be > 0")]
        CodeId,
        #[error("invalid funds")]
        Funds(#[from] coin::proto::Error),
    }

    impl Msg for MsgInstantiateContract2 {
        type Response = super::response::MsgInstantiateContract2Response;
    }
}
