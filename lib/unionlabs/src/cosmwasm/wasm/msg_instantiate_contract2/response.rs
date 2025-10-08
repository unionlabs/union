use serde::{Deserialize, Serialize};

use crate::primitives::{Bech32, Bytes, H256, encoding::Base64};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgInstantiateContract2Response {
    pub address: Bech32<H256>,
    pub data: Bytes<Base64>,
}

#[doc(hidden)] // TODO: Do this to all proto and ethabi modules
pub mod proto {

    use super::MsgInstantiateContract2Response;
    use crate::{
        impl_proto_via_try_from_into,
        primitives::{Bech32DecodeError, FixedBytesError},
    };

    impl_proto_via_try_from_into!(MsgInstantiateContract2Response => protos::cosmwasm::wasm::v1::MsgInstantiateContract2Response);

    impl From<MsgInstantiateContract2Response>
        for protos::cosmwasm::wasm::v1::MsgInstantiateContract2Response
    {
        fn from(value: MsgInstantiateContract2Response) -> Self {
            Self {
                address: value.address.to_string(),
                data: value.data.into(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgInstantiateContract2Response>
        for MsgInstantiateContract2Response
    {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgInstantiateContract2Response,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                address: value.address.parse()?,
                data: value.data.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid address")]
        Address(#[from] Bech32DecodeError<FixedBytesError>),
    }
}
