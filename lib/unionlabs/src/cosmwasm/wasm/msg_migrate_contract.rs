use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::primitives::{Bech32, Bytes, H256, encoding::Base64};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgMigrateContract {
    pub sender: Bech32<Bytes>,
    pub contract: Bech32<H256>,
    pub code_id: NonZeroU64,
    pub msg: Bytes<Base64>,
}

#[cfg(feature = "proto")]
pub mod proto {

    use unionlabs_primitives::{Bech32DecodeError, FixedBytesError};

    use super::MsgMigrateContract;
    use crate::{Msg, impl_proto_via_try_from_into};

    impl_proto_via_try_from_into!(MsgMigrateContract => protos::cosmwasm::wasm::v1::MsgMigrateContract);

    impl From<MsgMigrateContract> for protos::cosmwasm::wasm::v1::MsgMigrateContract {
        fn from(value: MsgMigrateContract) -> Self {
            Self {
                sender: value.sender.to_string(),
                contract: value.contract.to_string(),
                code_id: value.code_id.get(),
                msg: value.msg.to_vec(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgMigrateContract> for MsgMigrateContract {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgMigrateContract,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.sender.parse().map_err(Error::Sender)?,
                contract: value.contract.parse().map_err(Error::Contract)?,
                code_id: value.code_id.try_into().map_err(|_| Error::CodeId)?,
                msg: value.msg.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[from] Bech32DecodeError),
        #[error("invalid contract")]
        Contract(#[source] Bech32DecodeError<FixedBytesError>),
        #[error("invalid code id, must be > 0")]
        CodeId,
    }

    impl Msg for MsgMigrateContract {
        type Response = super::response::MsgMigrateContractResponse;
    }
}
