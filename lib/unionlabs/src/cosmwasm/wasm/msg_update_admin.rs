use serde::{Deserialize, Serialize};
use unionlabs_primitives::{Bytes, H256};

use crate::bech32::Bech32;

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgUpdateAdmin {
    pub sender: Bech32<Bytes>,
    pub new_admin: Bech32<Bytes>,
    pub contract: Bech32<H256>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use unionlabs_primitives::{Bytes, H256};

    use super::MsgUpdateAdmin;
    use crate::{bech32::Bech32, impl_proto_via_try_from_into, Msg};

    impl_proto_via_try_from_into!(MsgUpdateAdmin => protos::cosmwasm::wasm::v1::MsgUpdateAdmin);

    impl From<MsgUpdateAdmin> for protos::cosmwasm::wasm::v1::MsgUpdateAdmin {
        fn from(value: MsgUpdateAdmin) -> Self {
            Self {
                sender: value.sender.to_string(),
                new_admin: value.new_admin.to_string(),
                contract: value.contract.to_string(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgUpdateAdmin> for MsgUpdateAdmin {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgUpdateAdmin,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.sender.parse().map_err(Error::Sender)?,
                new_admin: value.new_admin.parse().map_err(Error::NewAdmin)?,
                contract: value.contract.parse().map_err(Error::Contract)?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[source] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid new admin")]
        NewAdmin(#[source] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid contract")]
        Contract(#[source] <Bech32<H256> as FromStr>::Err),
    }

    impl Msg for MsgUpdateAdmin {
        type Response = super::response::MsgUpdateAdminResponse;
    }
}
