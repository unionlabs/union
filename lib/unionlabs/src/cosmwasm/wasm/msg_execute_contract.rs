use serde::{Deserialize, Serialize};

use crate::{
    cosmos::base::coin::Coin,
    primitives::{Bech32, Bytes, H256, encoding::Base64},
};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgExecuteContract {
    pub sender: Bech32<Bytes>,
    pub contract: Bech32<H256>,
    pub msg: Bytes<Base64>,
    pub funds: Vec<Coin>,
}

#[cfg(feature = "proto")]
pub mod proto {

    use unionlabs_primitives::{Bech32DecodeError, FixedBytesError};

    use super::MsgExecuteContract;
    use crate::{Msg, cosmos::base::coin, impl_proto_via_try_from_into};

    impl_proto_via_try_from_into!(MsgExecuteContract => protos::cosmwasm::wasm::v1::MsgExecuteContract);

    impl From<MsgExecuteContract> for protos::cosmwasm::wasm::v1::MsgExecuteContract {
        fn from(value: MsgExecuteContract) -> Self {
            Self {
                sender: value.sender.to_string(),
                contract: value.contract.to_string(),
                msg: value.msg.to_vec(),
                funds: value.funds.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgExecuteContract> for MsgExecuteContract {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgExecuteContract,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sender: value.sender.parse().map_err(Error::Sender)?,
                contract: value.contract.parse().map_err(Error::Contract)?,
                msg: value.msg.into(),
                funds: value
                    .funds
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid sender")]
        Sender(#[source] Bech32DecodeError),
        #[error("invalid contract")]
        Contract(#[source] Bech32DecodeError<FixedBytesError>),
        #[error("invalid funds")]
        Funds(#[from] coin::proto::Error),
    }

    impl Msg for MsgExecuteContract {
        type Response = super::response::MsgExecuteContractResponse;
    }
}
