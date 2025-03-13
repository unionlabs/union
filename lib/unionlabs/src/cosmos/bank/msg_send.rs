use serde::{Deserialize, Serialize};
use unionlabs_primitives::Bytes;

use crate::{bech32::Bech32, cosmos::base::coin::Coin};

pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSend {
    pub from_address: Bech32<Bytes>,
    pub to_address: Bech32<Bytes>,
    pub amount: Vec<Coin>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use unionlabs_primitives::Bytes;

    use super::MsgSend;
    use crate::{bech32::Bech32, cosmos::base::coin, impl_proto_via_try_from_into, Msg};

    impl_proto_via_try_from_into!(MsgSend => protos::cosmos::bank::v1beta1::MsgSend);

    impl From<MsgSend> for protos::cosmos::bank::v1beta1::MsgSend {
        fn from(value: MsgSend) -> Self {
            Self {
                from_address: value.from_address.to_string(),
                to_address: value.to_address.to_string(),
                amount: value.amount.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<protos::cosmos::bank::v1beta1::MsgSend> for MsgSend {
        type Error = Error;

        fn try_from(value: protos::cosmos::bank::v1beta1::MsgSend) -> Result<Self, Self::Error> {
            Ok(Self {
                from_address: value.from_address.parse().map_err(Error::FromAddress)?,
                to_address: value.to_address.parse().map_err(Error::ToAddress)?,
                amount: value
                    .amount
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid from address")]
        FromAddress(#[source] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid to address")]
        ToAddress(#[source] <Bech32<Bytes> as FromStr>::Err),
        #[error("invalid amount")]
        InstantiatePermissions(#[from] coin::proto::Error),
    }

    impl Msg for MsgSend {
        type Response = super::response::MsgSendResponse;
    }
}
